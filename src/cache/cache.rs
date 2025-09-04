use once_cell::sync::Lazy;
use std::sync::Arc;
use std::sync::RwLock;

use crate::cache::inmem::InMemCache;
use crate::cache::redis::RedisCache;
use crate::cache::types::Cache;

// default to in-memory cache
static CACHE: Lazy<RwLock<Arc<dyn Cache + Send + Sync>>> =
    Lazy::new(|| RwLock::new(Arc::new(InMemCache::new())));

// Main API - returns Arc (cloning Arc is cheap, just increments reference count)
// use this to get a cache instance
pub fn cache() -> Arc<dyn Cache + Send + Sync> {
    CACHE.read().unwrap().clone()
}

// use this to set the cache instance
pub async fn set_cache(backend: &str, url: &str) -> anyhow::Result<()> {
    let new_cache: Arc<dyn Cache + Send + Sync> = match backend {
        "inmem" => Arc::new(InMemCache::new()),
        "redis" => Arc::new(RedisCache::new(url)?),
        _ => return Err(anyhow::anyhow!("Invalid cache backend: {}", backend)),
    };
    *CACHE.write().unwrap() = new_cache;
    Ok(())
}
