use std::sync::{Arc, RwLock};
use std::{fs::File, io::BufReader};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub debug: bool,
    pub server: ServerConfig,
    pub stores: Vec<StoreConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub store_type: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Config {
    pub fn read(path: &str) -> anyhow::Result<Self> {
        let file = File::open(path)
            .map_err(|e| anyhow::anyhow!("Failed to open config file {}: {}", path, e))?;
        let reader = BufReader::new(file);
        let config: Config = serde_yaml::from_reader(reader)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file {}: {}", path, e))?;
        Ok(config)
    }

    pub fn empty() -> Self {
        Self {
            debug: false,
            server: ServerConfig {
                port: 8006,
                host: "0.0.0.0".to_string(),
            },
            stores: Vec::new(),
        }
    }
}

pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::empty())),
        }
    }

    pub fn load_config(&self, path: &str) -> anyhow::Result<()> {
        let config = Config::read(path)?;
        let mut write_lock = self
            .config
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
        *write_lock = config;
        Ok(())
    }

    pub fn get_config(&self) -> anyhow::Result<Config> {
        let read_lock = self
            .config
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))?;
        Ok(read_lock.clone())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

// Global config manager instance
pub static CONFIG_MANAGER: once_cell::sync::Lazy<ConfigManager> =
    once_cell::sync::Lazy::new(ConfigManager::new);

// Convenience functions for backward compatibility
pub fn get_config() -> anyhow::Result<Config> {
    CONFIG_MANAGER.get_config()
}

pub fn load_config(path: &str) -> anyhow::Result<()> {
    CONFIG_MANAGER.load_config(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_empty() {
        let config = Config::empty();
        assert_eq!(config.debug, false);
        assert_eq!(config.server.port, 8006);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.stores.len(), 0);
    }

    #[test]
    fn test_config_manager_new() {
        let manager = ConfigManager::new();
        let config = manager.get_config().unwrap();
        assert_eq!(config.debug, false);
        assert_eq!(config.server.port, 8006);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.stores.len(), 0);
    }

    #[test]
    fn test_config_from_file() {
        let manager = ConfigManager::new();
        manager.load_config("config.yaml").unwrap();
        let config = manager.get_config().unwrap();
        assert_eq!(config.debug, true);
        assert_eq!(config.server.port, 8006);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_ne!(config.stores.len(), 0);
    }
}
