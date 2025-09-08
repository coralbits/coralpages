// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

use std::collections::HashMap;

use crate::cache::types::Cache;
use async_trait::async_trait;
use tokio::sync::RwLock;

pub struct InMemCache {
    cache: RwLock<HashMap<String, String>>,
}

impl InMemCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Cache for InMemCache {
    async fn get(&self, key: &str) -> Option<String> {
        self.cache.read().await.get(key).cloned()
    }

    async fn set(&self, key: &str, value: &str) {
        self.cache
            .write()
            .await
            .insert(key.to_string(), value.to_string());
    }

    async fn delete(&self, key: &str) -> Option<()> {
        let ret = self.cache.write().await.remove(key);
        if ret.is_some() {
            Some(())
        } else {
            None
        }
    }
}
