use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::Serialize;
use url::Url;

use super::{Indexer, SubmissionResult};
use crate::error::IndexerError;

const INDEXNOW_ENDPOINT: &str = "https://api.indexnow.org/indexnow";

pub struct IndexNowClient {
    api_key: String,
    http: Client,
    endpoint: String,
}

#[derive(Serialize)]
struct IndexNowRequest<'a> {
    host: &'a str,
    key: &'a str,
    #[serde(rename = "urlList")]
    url_list: Vec<&'a str>,
}

impl IndexNowClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http: Client::new(),
            endpoint: INDEXNOW_ENDPOINT.to_string(),
        }
    }

    /// For testing: override the endpoint.
    #[cfg(test)]
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = endpoint;
        self
    }
}

#[async_trait]
impl Indexer for IndexNowClient {
    fn name(&self) -> &'static str {
        "indexnow"
    }

    async fn submit(&self, url: &Url) -> Result<SubmissionResult, IndexerError> {
        let host = url
            .host_str()
            .ok_or_else(|| IndexerError::InvalidUrl(url.to_string()))?;

        let url_str = url.as_str();
        let body = IndexNowRequest {
            host,
            key: &self.api_key,
            url_list: vec![url_str],
        };

        let response = self.http.post(&self.endpoint).json(&body).send().await?;

        let status = response.status();

        if status.is_success() || status.as_u16() == 202 {
            return Ok(SubmissionResult {
                success: true,
                method: "indexnow".to_string(),
                message: format!("Submitted successfully (HTTP {})", status.as_u16()),
                timestamp: Utc::now(),
            });
        }

        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        Err(IndexerError::Api {
            status: status.as_u16(),
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn make_client(server_url: &str, api_key: &str) -> IndexNowClient {
        IndexNowClient::new(api_key.to_string()).with_endpoint(server_url.to_string())
    }

    #[tokio::test]
    async fn test_indexnow_submit_single_url() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/indexnow"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let endpoint = format!("{}/indexnow", mock_server.uri());
        let client = make_client(&endpoint, "test-api-key").await;

        let url = Url::parse("https://example.com/page").unwrap();
        let result = client.submit(&url).await.unwrap();

        assert!(result.success);
        assert_eq!(result.method, "indexnow");
    }

    #[tokio::test]
    async fn test_indexnow_submit_sends_correct_body() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/indexnow"))
            .and(wiremock::matchers::body_json(serde_json::json!({
                "host": "example.com",
                "key": "my-secret-key",
                "urlList": ["https://example.com/blog/post-1"]
            })))
            .respond_with(ResponseTemplate::new(202))
            .mount(&mock_server)
            .await;

        let endpoint = format!("{}/indexnow", mock_server.uri());
        let client = make_client(&endpoint, "my-secret-key").await;

        let url = Url::parse("https://example.com/blog/post-1").unwrap();
        let result = client.submit(&url).await.unwrap();

        assert!(result.success);
    }

    #[tokio::test]
    async fn test_indexnow_submit_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/indexnow"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&mock_server)
            .await;

        let endpoint = format!("{}/indexnow", mock_server.uri());
        let client = make_client(&endpoint, "bad-key").await;

        let url = Url::parse("https://example.com/page").unwrap();
        let err = client.submit(&url).await.unwrap_err();

        match err {
            IndexerError::Api { status, .. } => assert_eq!(status, 400),
            other => panic!("Expected Api error, got: {other}"),
        }
    }
}
