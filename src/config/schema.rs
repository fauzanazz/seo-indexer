use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub indexnow: IndexNowConfig,
    #[serde(default)]
    pub google: GoogleConfig,
    #[serde(default)]
    pub ping: PingConfig,
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexNowConfig {
    pub api_key: Option<String>,
    pub key_location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoogleConfig {
    pub service_account_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PingConfig {
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub default_method: String,
    pub rate_limit_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageConfig {
    /// Path to the SQLite database. Defaults to ~/.local/share/indexer/submissions.db
    pub path: Option<String>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_method: "all".to_string(),
            rate_limit_per_minute: 60,
        }
    }
}
