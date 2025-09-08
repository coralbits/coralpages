// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

use crate::cache::types::Cache;
use async_trait::async_trait;
use redis::AsyncCommands;
use tracing::{debug, error};

pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    pub fn new(url: &str) -> anyhow::Result<Self> {
        debug!("Creating redis cache with url: {}", url);
        let client = redis::Client::open(url)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get(&self, key: &str) -> Option<String> {
        let mut client = match self.client.get_multiplexed_async_connection().await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to get redis client: {}", e);
                return None;
            }
        };
        if let Ok(data) = client.get(key).await {
            Some(data)
        } else {
            None
        }
    }

    async fn set(&self, key: &str, value: &str) {
        let mut client = match self.client.get_multiplexed_async_connection().await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to get redis client: {}", e);
                return;
            }
        };
        match client.set::<&str, &str, ()>(key, value).await {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to set redis key: {}", e);
            }
        };
    }

    async fn delete(&self, key: &str) -> Option<()> {
        let mut client = match self.client.get_multiplexed_async_connection().await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to get redis client: {}", e);
                return None;
            }
        };
        let ret = match client.del::<&str, ()>(key).await {
            Ok(_) => Some(()),
            Err(e) => {
                error!("Failed to delete redis key: {}", e);
                None
            }
        };
        ret
    }
}
