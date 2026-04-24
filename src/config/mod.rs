pub mod schema;

use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

use crate::error::ConfigError;
pub use schema::Config;

const CONFIG_QUALIFIER: &str = "ai";
const CONFIG_ORGANIZATION: &str = "legali";
const CONFIG_APP: &str = "indexer";

pub fn config_path() -> Result<PathBuf, ConfigError> {
    let dirs = ProjectDirs::from(CONFIG_QUALIFIER, CONFIG_ORGANIZATION, CONFIG_APP)
        .ok_or(ConfigError::DirectoryNotFound)?;
    Ok(dirs.config_dir().join("config.toml"))
}

pub fn load() -> Result<Config, ConfigError> {
    let path = config_path()?;

    let base = if path.exists() {
        let contents = fs::read_to_string(&path)?;
        toml::from_str::<Config>(&contents)?
    } else {
        Config::default()
    };

    Ok(apply_env_overrides(base))
}

fn apply_env_overrides(mut config: Config) -> Config {
    if let Ok(key) = std::env::var("INDEXER_INDEXNOW_API_KEY") {
        config.indexnow.api_key = Some(key);
    }
    if let Ok(location) = std::env::var("INDEXER_INDEXNOW_KEY_LOCATION") {
        config.indexnow.key_location = Some(location);
    }
    if let Ok(method) = std::env::var("INDEXER_DEFAULT_METHOD") {
        config.general.default_method = method;
    }
    if let Ok(rate) = std::env::var("INDEXER_RATE_LIMIT_PER_MINUTE") {
        if let Ok(parsed) = rate.parse::<u32>() {
            config.general.rate_limit_per_minute = parsed;
        }
    }
    config
}

pub fn write_default(path: &PathBuf) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let default_toml = r#"[indexnow]
# api_key = "your-api-key-here"
# key_location = "https://example.com/your-api-key.txt"

[general]
default_method = "all"
rate_limit_per_minute = 60
"#;
    fs::write(path, default_toml)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.general.default_method, "all");
        assert_eq!(config.general.rate_limit_per_minute, 60);
        assert!(config.indexnow.api_key.is_none());
        assert!(config.indexnow.key_location.is_none());
    }

    #[test]
    fn test_config_env_override() {
        // Use a unique key to avoid test pollution
        std::env::set_var("INDEXER_INDEXNOW_API_KEY", "test-key-123");
        std::env::set_var("INDEXER_DEFAULT_METHOD", "indexnow");
        std::env::set_var("INDEXER_RATE_LIMIT_PER_MINUTE", "30");

        let config = apply_env_overrides(Config::default());

        std::env::remove_var("INDEXER_INDEXNOW_API_KEY");
        std::env::remove_var("INDEXER_DEFAULT_METHOD");
        std::env::remove_var("INDEXER_RATE_LIMIT_PER_MINUTE");

        assert_eq!(config.indexnow.api_key.as_deref(), Some("test-key-123"));
        assert_eq!(config.general.default_method, "indexnow");
        assert_eq!(config.general.rate_limit_per_minute, 30);
    }

    #[test]
    fn test_config_parse_toml() {
        let toml_str = r#"
[indexnow]
api_key = "my-key"
key_location = "https://example.com/my-key.txt"

[general]
default_method = "google"
rate_limit_per_minute = 10
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.indexnow.api_key.as_deref(), Some("my-key"));
        assert_eq!(config.general.default_method, "google");
        assert_eq!(config.general.rate_limit_per_minute, 10);
    }
}
