use std::sync::Arc;
use std::{fs::File, io::BufReader};
use tokio::sync::{RwLock, RwLockReadGuard};

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

    pub async fn load_config(&self, path: &str) -> anyhow::Result<()> {
        let config = Config::read(path)?;
        let mut write_lock = self.config.write().await;
        *write_lock = config;
        Ok(())
    }

    pub async fn get_config(&self) -> RwLockReadGuard<'_, Config> {
        let read_lock = self.config.read().await;
        read_lock
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
pub async fn get_config() -> RwLockReadGuard<'static, Config> {
    CONFIG_MANAGER.get_config().await
}

pub async fn load_config(path: &str) -> anyhow::Result<()> {
    CONFIG_MANAGER.load_config(path).await
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

    #[tokio::test]
    async fn test_config_manager_new() {
        let manager = ConfigManager::new();
        let config = manager.get_config().await;
        assert_eq!(config.debug, false);
        assert_eq!(config.server.port, 8006);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.stores.len(), 0);
    }

    #[tokio::test]
    async fn test_config_from_file() {
        let manager = ConfigManager::new();
        manager.load_config("config.yaml").await.unwrap();
        let config = manager.get_config().await;
        assert_eq!(config.debug, true);
        assert_eq!(config.server.port, 8006);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_ne!(config.stores.len(), 0);
    }
}
