//! Memory cache implementation using cached::TimedSizedCache.

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use cached::{Cached, TimedSizedCache};

use crate::cache::{AppCache, CacheError};
use crate::config::settings::MemoryCacheConfig;

/// In-memory cache with size limit and TTL.
pub struct MemoryCache {
    store: Mutex<TimedSizedCache<String, Vec<u8>>>,
}

impl MemoryCache {
    pub fn new(config: &MemoryCacheConfig) -> Self {
        let store = TimedSizedCache::with_size_and_lifespan(
            config.max_size,
            Duration::from_secs(config.ttl_seconds),
        );
        Self {
            store: Mutex::new(store),
        }
    }
}

#[async_trait]
impl AppCache for MemoryCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        Ok(store.cache_get(key).cloned())
    }

    async fn set(
        &self,
        key: &str,
        value: Vec<u8>,
        _ttl_seconds: Option<u64>,
    ) -> Result<(), CacheError> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        store.cache_set(key.to_string(), value);
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<(), CacheError> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        store.cache_remove(key);
        Ok(())
    }

    async fn clear(&self) -> Result<(), CacheError> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        store.cache_clear();
        Ok(())
    }
}
