use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(
        "Failed to read config file: {0}\nCheck that the file exists and you have read permission."
    )]
    Io(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}\nVerify your TOML syntax at https://toml.io/en/")]
    Parse(#[from] toml::de::Error),
    #[error(
        "Could not determine config directory. Set HOME or XDG_CONFIG_HOME environment variables."
    )]
    DirectoryNotFound,
}

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API returned error {status}: {message}")]
    Api { status: u16, message: String },
    #[allow(dead_code)]
    #[error("Missing API key for '{indexer}'. Run `indexer config init` to set it up.")]
    MissingApiKey { indexer: &'static str },
    #[error("Invalid URL '{0}'. Make sure it starts with http:// or https://")]
    InvalidUrl(String),
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: String,
        source: std::io::Error,
    },
    #[error("Failed to parse service account JSON: {0}\nEnsure the file is a valid Google Cloud service account key.")]
    ServiceAccountParse(String),
    #[error("JWT signing failed: {0}")]
    JwtSign(String),
    #[error("Failed to exchange auth token: {0}\nCheck your service account permissions in Google Cloud Console.")]
    AuthTokenExchange(String),
}
