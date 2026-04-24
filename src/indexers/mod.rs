pub mod google;
pub mod indexnow;
pub mod ping;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use url::Url;

use crate::config::Config;
use crate::error::IndexerError;

pub use google::GoogleIndexingClient;
pub use indexnow::IndexNowClient;
pub use ping::PingClient;

#[derive(Debug)]
pub struct SubmissionResult {
    pub success: bool,
    pub method: String,
    pub message: String,
    #[allow(dead_code)]
    pub timestamp: DateTime<Utc>,
}

#[async_trait]
pub trait Indexer: Send + Sync {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    async fn submit(&self, url: &Url) -> Result<SubmissionResult, IndexerError>;
}

pub fn get_indexers(config: &Config, method: &str) -> Vec<Box<dyn Indexer>> {
    let effective_method = if method == "all" {
        config.general.default_method.as_str()
    } else {
        method
    };

    match effective_method {
        "indexnow" => indexnow_client(config).into_iter().collect(),
        "google" => google_client(config).into_iter().collect(),
        "ping" => vec![Box::new(PingClient::new(config.ping.services.clone()))],
        _ => {
            // "all" or the default — include every configured indexer
            let mut indexers: Vec<Box<dyn Indexer>> = Vec::new();
            indexers.extend(indexnow_client(config));
            indexers.extend(google_client(config));
            indexers.push(Box::new(PingClient::new(config.ping.services.clone())));
            indexers
        }
    }
}

fn indexnow_client(config: &Config) -> Option<Box<dyn Indexer>> {
    config
        .indexnow
        .api_key
        .clone()
        .map(|key| -> Box<dyn Indexer> { Box::new(IndexNowClient::new(key)) })
}

fn google_client(config: &Config) -> Option<Box<dyn Indexer>> {
    config
        .google
        .service_account_json
        .clone()
        .map(|path| -> Box<dyn Indexer> { Box::new(GoogleIndexingClient::new(path)) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{
        GeneralConfig, GoogleConfig, IndexNowConfig, PingConfig, StorageConfig,
    };

    fn config_with(
        indexnow_key: Option<&str>,
        google_sa: Option<&str>,
        ping_services: Vec<&str>,
        default_method: &str,
    ) -> Config {
        Config {
            indexnow: IndexNowConfig {
                api_key: indexnow_key.map(String::from),
                key_location: None,
            },
            google: GoogleConfig {
                service_account_json: google_sa.map(String::from),
            },
            ping: PingConfig {
                services: ping_services.into_iter().map(String::from).collect(),
            },
            general: GeneralConfig {
                default_method: default_method.to_string(),
                rate_limit_per_minute: 60,
            },
            storage: StorageConfig { path: None },
        }
    }

    #[test]
    fn test_get_indexers_returns_correct_clients_for_indexnow() {
        let config = config_with(Some("my-key"), None, vec![], "indexnow");
        let indexers = get_indexers(&config, "indexnow");
        assert_eq!(indexers.len(), 1);
        assert_eq!(indexers[0].name(), "indexnow");
    }

    #[test]
    fn test_get_indexers_returns_correct_clients_for_google() {
        let config = config_with(None, Some("/path/to/sa.json"), vec![], "google");
        let indexers = get_indexers(&config, "google");
        assert_eq!(indexers.len(), 1);
        assert_eq!(indexers[0].name(), "google");
    }

    #[test]
    fn test_get_indexers_returns_ping_client() {
        let config = config_with(None, None, vec!["google", "bing"], "ping");
        let indexers = get_indexers(&config, "ping");
        assert_eq!(indexers.len(), 1);
        assert_eq!(indexers[0].name(), "ping");
    }

    #[test]
    fn test_get_indexers_all_includes_configured_indexers() {
        let config = config_with(Some("key"), Some("/sa.json"), vec![], "all");
        let indexers = get_indexers(&config, "all");
        // indexnow + google + ping
        assert_eq!(indexers.len(), 3);
    }

    #[test]
    fn test_get_indexers_all_skips_unconfigured() {
        let config = config_with(None, None, vec![], "all");
        let indexers = get_indexers(&config, "all");
        // only ping (always present), indexnow and google are absent (no config)
        assert_eq!(indexers.len(), 1);
        assert_eq!(indexers[0].name(), "ping");
    }

    #[test]
    fn test_get_indexers_method_all_defers_to_default_method() {
        // When method == "all" it uses default_method from config
        let config = config_with(Some("key"), None, vec![], "indexnow");
        let indexers = get_indexers(&config, "all");
        assert_eq!(indexers.len(), 1);
        assert_eq!(indexers[0].name(), "indexnow");
    }
}
