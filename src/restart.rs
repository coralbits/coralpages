use anyhow::Result;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{broadcast, Notify};
use tracing::{info, warn};

use crate::config;

/// A restart manager that handles graceful server restarts
pub struct RestartManager {
    shutdown_tx: broadcast::Sender<()>,
    restart_notify: Arc<Notify>,
    listen_addr: String,
}

impl RestartManager {
    pub fn new(listen_addr: String) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let restart_notify = Arc::new(Notify::new());

        Self {
            shutdown_tx,
            restart_notify,
            listen_addr,
        }
    }

    /// Get a receiver for shutdown signals
    pub fn get_shutdown_receiver(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Get the restart notification handle
    pub fn get_restart_notify(&self) -> Arc<Notify> {
        self.restart_notify.clone()
    }

    /// Trigger a server restart
    pub fn restart(&self) {
        info!("Restart signal received, restarting server...");
        self.restart_notify.notify_one();
    }

    /// Shutdown the server
    pub fn shutdown(&self) {
        info!("Shutdown signal received, stopping server...");
        let _ = self.shutdown_tx.send(());
    }

    pub fn enable_restart_with_signal(&self, sigkind: SignalKind) -> Result<()> {
        let mut sighup = signal(sigkind)?;
        let restart_notify = self.restart_notify.clone();
        // Spawn signal handler task
        tokio::spawn(async move {
            loop {
                if sighup.recv().await.is_some() {
                    info!("SIGHUP received, triggering restart...");
                    restart_notify.notify_one();
                }
            }
        });
        Ok(())
    }

    /// Run the server with restart capability
    pub async fn run_with_restart<F, Fut>(&self, server_fn: F) -> Result<()>
    where
        F: Fn(String, broadcast::Receiver<()>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let server_fn = std::sync::Arc::new(server_fn);

        loop {
            info!("Starting server on {}", self.listen_addr);

            // Create a new shutdown receiver for this server instance
            let shutdown_rx = self.get_shutdown_receiver();

            // Start the server in a task
            let mut server_task = tokio::spawn({
                let listen_addr = self.listen_addr.clone();
                let server_fn = server_fn.clone();
                async move { server_fn(listen_addr, shutdown_rx).await }
            });

            // Wait for either restart signal or server completion
            tokio::select! {
                _ = self.restart_notify.notified() => {
                    info!("Restart signal received, stopping current server...");
                    // Send shutdown signal to the server
                    self.shutdown();

                    // Wait for the server to stop
                    match server_task.await {
                        Ok(result) => {
                            if let Err(e) = result {
                                warn!("Server stopped with error: {}", e);
                            } else {
                                info!("Server stopped gracefully");
                            }
                        }
                        Err(e) => {
                            warn!("Server task panicked: {}", e);
                        }
                    }

                    // Reload configuration before restarting
                    info!("Reloading configuration...");
                    if let Err(e) = config::reload_config().await {
                        warn!("Failed to reload configuration: {}", e);
                    }

                    info!("Restarting server...");
                }
                result = &mut server_task => {
                    match result {
                        Ok(Ok(())) => {
                            info!("Server completed normally");
                            return Ok(());
                        }
                        Ok(Err(e)) => {
                            warn!("Server error: {}", e);
                            return Err(e);
                        }
                        Err(e) => {
                            warn!("Server task panicked: {}", e);
                            return Err(anyhow::anyhow!("Server task panicked: {}", e));
                        }
                    }
                }
            }
        }
    }
}
