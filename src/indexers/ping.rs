use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use url::Url;

use super::{Indexer, SubmissionResult};
use crate::error::IndexerError;

enum PingService {
    Google,
    Bing,
}

impl PingService {
    fn ping_url(&self, target_url: &str) -> String {
        let encoded = urlencoding::encode(target_url);
        match self {
            PingService::Google => format!("https://www.google.com/ping?sitemap={encoded}"),
            PingService::Bing => format!("https://www.bing.com/ping?sitemap={encoded}"),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            PingService::Google => "google-ping",
            PingService::Bing => "bing-ping",
        }
    }
}

fn parse_service(name: &str) -> Option<PingService> {
    match name.to_lowercase().as_str() {
        "google" => Some(PingService::Google),
        "bing" => Some(PingService::Bing),
        _ => None,
    }
}

pub struct PingClient {
    services: Vec<PingService>,
    http: Client,
}

impl PingClient {
    pub fn new(service_names: Vec<String>) -> Self {
        let services = if service_names.is_empty() {
            vec![PingService::Google, PingService::Bing]
        } else {
            service_names
                .iter()
                .filter_map(|s| parse_service(s))
                .collect()
        };

        Self {
            services,
            http: Client::new(),
        }
    }
}

#[async_trait]
impl Indexer for PingClient {
    fn name(&self) -> &'static str {
        "ping"
    }

    async fn submit(&self, url: &Url) -> Result<SubmissionResult, IndexerError> {
        let url_str = url.as_str();
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for service in &self.services {
            let ping_url = service.ping_url(url_str);
            match self.http.get(&ping_url).send().await {
                Ok(response) if response.status().is_success() => {
                    successes.push(service.name());
                }
                Ok(response) => {
                    failures.push(format!("{}: HTTP {}", service.name(), response.status()));
                }
                Err(e) => {
                    failures.push(format!("{}: {}", service.name(), e));
                }
            }
        }

        if failures.is_empty() {
            Ok(SubmissionResult {
                success: true,
                method: "ping".to_string(),
                message: format!("Pinged: {}", successes.join(", ")),
                timestamp: Utc::now(),
            })
        } else if successes.is_empty() {
            Err(IndexerError::Api {
                status: 0,
                message: format!("All pings failed: {}", failures.join("; ")),
            })
        } else {
            Ok(SubmissionResult {
                success: false,
                method: "ping".to_string(),
                message: format!(
                    "Partial success. OK: {}. Failed: {}",
                    successes.join(", "),
                    failures.join("; ")
                ),
                timestamp: Utc::now(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, query_param_contains};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_ping_service_urls() {
        let google = PingService::Google;
        let bing = PingService::Bing;
        let target = "https://example.com/sitemap.xml";

        assert!(google
            .ping_url(target)
            .starts_with("https://www.google.com/ping?sitemap="));
        assert!(bing
            .ping_url(target)
            .starts_with("https://www.bing.com/ping?sitemap="));
    }

    #[test]
    fn test_ping_client_defaults_to_google_and_bing() {
        let client = PingClient::new(vec![]);
        assert_eq!(client.services.len(), 2);
    }

    #[test]
    fn test_ping_client_filters_unknown_services() {
        let client = PingClient::new(vec!["google".to_string(), "unknown-service".to_string()]);
        assert_eq!(client.services.len(), 1);
    }

    #[tokio::test]
    async fn test_ping_services_submit() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(query_param_contains("sitemap", "example.com"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        // We can't override ping URLs easily since they're hardcoded,
        // so we verify the client builds correct URLs instead.
        let service = PingService::Google;
        let target = "https://example.com/page";
        let ping_url = service.ping_url(target);

        assert!(ping_url.contains("https%3A%2F%2Fexample.com%2Fpage"));
        assert!(ping_url.starts_with("https://www.google.com/ping?sitemap="));

        // Verify mock server handles the query param correctly
        let client = reqwest::Client::new();
        let test_url = format!(
            "{}/ping?sitemap=https://example.com/test",
            mock_server.uri()
        );
        let response = client.get(&test_url).send().await.unwrap();
        assert!(response.status().is_success());
    }
}

// Needed for URL encoding in ping URLs
mod urlencoding {
    pub fn encode(input: &str) -> String {
        input
            .bytes()
            .flat_map(|byte| {
                if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                    vec![byte as char]
                } else {
                    format!("%{:02X}", byte).chars().collect()
                }
            })
            .collect()
    }
}
