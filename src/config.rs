use std::sync::Arc;
use std::{fs::File, io::BufReader};
use tokio::sync::{RwLock, RwLockReadGuard};

use notify::{RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub debug: bool,
    pub server: ServerConfig,
    pub stores: Vec<StoreConfig>,
    pub pdf: Option<PdfConfig>,
    pub cache: Option<CacheConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheConfig {
    pub backend: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PdfConfig {
    pub chromium_path: String,
    pub temp_dir: String,
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
        let config = config.postprocess();
        Ok(config)
    }
    fn postprocess(mut self) -> Self {
        if let Some(pdf) = self.pdf.as_mut() {
            if pdf.temp_dir.starts_with("$HOME") {
                pdf.temp_dir = pdf
                    .temp_dir
                    .replace("$HOME", &std::env::var("HOME").unwrap());
            }
        }
        self
    }

    pub fn empty() -> Self {
        Self {
            debug: false,
            pdf: None,
            cache: None,
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
    _watcher_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::empty())),
            _watcher_handle: Arc::new(RwLock::new(None)),
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

    /// Start watching the config file for changes and automatically reload it
    pub async fn watch_config(&self, path: &str) -> anyhow::Result<()> {
        let config_path = Path::new(path).to_path_buf();
        let config_manager = Arc::new(self.config.clone());
        let path_string = path.to_string();

        // Spawn the file watcher in a separate task
        tokio::spawn(async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);

            // Create the file watcher
            let mut watcher =
                notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
                    Ok(event) => {
                        let _ = tx.blocking_send(event);
                    }
                    Err(e) => {
                        error!("Error receiving event: {}", e);
                    }
                })
                .expect("Failed to create file watcher");

            // Watch the config file
            if let Err(e) = watcher.watch(&config_path, RecursiveMode::NonRecursive) {
                error!("Failed to watch config file: {}", e);
                return;
            }

            info!("Started watching config_file={}", path_string);

            // Listen for file change events
            loop {
                let event = rx.recv().await;
                let event = match event {
                    Some(event) => event,
                    None => {
                        error!("Error receiving event");
                        continue;
                    }
                };

                info!("Event received: {:?}", event);
                match event.kind {
                    notify::EventKind::Access(notify::event::AccessKind::Close(
                        notify::event::AccessMode::Write,
                    )) => {
                        info!("Write close event detected, reloading config...");
                        Self::reload_config_static(&config_manager, &path_string).await;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Static method to reload config (used by the watcher)
    async fn reload_config_static(config: &Arc<RwLock<Config>>, path: &str) {
        info!("Reloading config from {}", path);
        let new_config = match Config::read(path) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to reload config: {}", e);
                return;
            }
        };
        info!("Waiting for write lock");
        let mut write_lock = config.write().await;
        info!("Write lock acquired");
        *write_lock = new_config;
        info!("Config reloaded successfully from {}", path);
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

pub async fn watch_config(path: &str) -> anyhow::Result<()> {
    CONFIG_MANAGER.watch_config(path).await
}

pub async fn get_debug() -> bool {
    let config = CONFIG_MANAGER.get_config().await;
    config.debug
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

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
        assert_eq!(config.debug, false);
        assert_eq!(config.server.port, 8006);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_ne!(config.stores.len(), 0);
    }

    #[tokio::test]
    async fn test_config_file_watching() {
        // Create a temporary config file
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_str().unwrap();

        // Write initial config
        let initial_config = r#"
debug: false
server:
  port: 8006
  host: "0.0.0.0"
stores: []
"#;
        fs::write(config_path, initial_config).unwrap();

        // Create config manager and load initial config
        let manager = ConfigManager::new();
        manager.load_config(config_path).await.unwrap();

        // Verify initial config
        let config = { manager.get_config().await.clone() };
        assert_eq!(config.debug, false);
        assert_eq!(config.server.port, 8006);
        assert_eq!(config.stores.len(), 0);

        // Start watching the config file
        manager.watch_config(config_path).await.unwrap();

        // Wait a bit for the watcher to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Modify the config file
        let modified_config = r#"
debug: true
server:
  port: 9000
  host: "127.0.0.1"
stores:
  - name: "test-store"
    type: "file"
    path: "/tmp/test"
"#;
        fs::write(config_path, modified_config).unwrap();

        // Wait for the file change to be detected and processed
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Verify the config has been updated
        let updated_config = { manager.get_config().await.clone() };
        assert_eq!(updated_config.debug, true);
        assert_eq!(updated_config.server.port, 9000);
        assert_eq!(updated_config.server.host, "127.0.0.1");
        assert_eq!(updated_config.stores.len(), 1);
        assert_eq!(updated_config.stores[0].name, "test-store");
        assert_eq!(updated_config.stores[0].store_type, "file");
        assert_eq!(updated_config.stores[0].path, "/tmp/test");

        // do it again
        fs::write(config_path, initial_config).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        let updated_config = { manager.get_config().await.clone() };
        assert_eq!(updated_config.debug, false);
        assert_eq!(updated_config.server.port, 8006);
        assert_eq!(updated_config.server.host, "0.0.0.0");
        assert_eq!(updated_config.stores.len(), 0);

        // Clean up
        temp_file.close().unwrap();
    }
}
