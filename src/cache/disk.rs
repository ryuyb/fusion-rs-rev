//! Disk cache implementation with per-entry TTL support.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use cached::IOCached;
use cached::stores::DiskCache as CachedDiskCache;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::cache::{AppCache, CacheError};
use crate::config::settings::DiskCacheConfig;

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    value: Vec<u8>,
    expires_at: Option<u64>, // Unix timestamp in seconds
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now >= exp
        } else {
            false
        }
    }
}

/// Disk-based cache with per-entry TTL.
pub struct DiskCache {
    store: Mutex<CachedDiskCache<String, Vec<u8>>>,
    default_ttl: u64,
}

impl DiskCache {
    pub fn new(config: &DiskCacheConfig, cache_name: &str) -> Result<Self, CacheError> {
        // Set a very long lifespan - we manage TTL ourselves via CacheEntry
        let store = CachedDiskCache::new(cache_name)
            .set_disk_directory(&config.directory)
            .set_lifespan(Duration::from_secs(86400 * 365)) // 1 year max
            .build()
            .map_err(|e| CacheError::Connection(e.to_string()))?;
        Ok(Self {
            store: Mutex::new(store),
            default_ttl: config.ttl_seconds,
        })
    }
}

#[async_trait]
impl AppCache for DiskCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let key_string = key.to_string();
        let store = self.store.lock().await;

        let bytes = store
            .cache_get(&key_string)
            .map_err(|e| CacheError::Operation(e.to_string()))?;

        if let Some(bytes) = bytes
            && let Ok(entry) = serde_json::from_slice::<CacheEntry>(&bytes)
        {
            if !entry.is_expired() {
                return Ok(Some(entry.value));
            }
            // Entry expired, remove it
            let _ = store.cache_remove(&key_string);
        }
        Ok(None)
    }

    async fn set(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl_seconds: Option<u64>,
    ) -> Result<(), CacheError> {
        let store = self.store.lock().await;

        let ttl = ttl_seconds.unwrap_or(self.default_ttl);
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + ttl;

        let entry = CacheEntry {
            value,
            expires_at: Some(expires_at),
        };

        let bytes =
            serde_json::to_vec(&entry).map_err(|e| CacheError::Serialization(e.to_string()))?;

        store
            .cache_set(key.to_string(), bytes)
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<(), CacheError> {
        let key_string = key.to_string();
        let store = self.store.lock().await;
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
