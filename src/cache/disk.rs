//! Disk cache implementation using cached::DiskCache.

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use cached::IOCached;
use cached::stores::DiskCache as CachedDiskCache;

use crate::cache::{AppCache, CacheError};
use crate::config::settings::DiskCacheConfig;

/// Disk-based cache with TTL.
pub struct DiskCache {
    store: Mutex<CachedDiskCache<String, Vec<u8>>>,
}

impl DiskCache {
    pub fn new(config: &DiskCacheConfig, cache_name: &str) -> Result<Self, CacheError> {
        let store = CachedDiskCache::new(cache_name)
            .set_disk_directory(&config.directory)
            .set_lifespan(Duration::from_secs(config.ttl_seconds))
            .build()
            .map_err(|e| CacheError::Connection(e.to_string()))?;
        Ok(Self {
            store: Mutex::new(store),
        })
    }
}

#[async_trait]
impl AppCache for DiskCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let key_string = key.to_string();
        let store = self
            .store
            .lock()
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        store
            .cache_get(&key_string)
            .map_err(|e| CacheError::Operation(e.to_string()))
    }

    async fn set(
        &self,
        key: &str,
        value: Vec<u8>,
        _ttl_seconds: Option<u64>,
    ) -> Result<(), CacheError> {
        let store = self
            .store
            .lock()
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        store
            .cache_set(key.to_string(), value)
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<(), CacheError> {
        let key_string = key.to_string();
        let store = self
            .store
            .lock()
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        store
            .cache_remove(&key_string)
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        Ok(())
    }

    async fn clear(&self) -> Result<(), CacheError> {
        // DiskCache doesn't have a built-in clear method
        Ok(())
    }
}
