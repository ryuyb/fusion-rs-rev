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
        let store = self.store.lock().await;
        let db = store.connection();

        // Use sled's clear (more efficient than iterating)
        db.clear()
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        db.flush()
            .map_err(|e| CacheError::Operation(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_config() -> DiskCacheConfig {
        let dir = tempdir().unwrap();
        DiskCacheConfig {
            directory: dir.path().to_str().unwrap().to_string(),
            ttl_seconds: 3600,
        }
    }

    #[tokio::test]
    async fn test_get_set() {
        let cache = DiskCache::new(&test_config(), "test_get_set").unwrap();
        cache.set("key", b"value".to_vec(), None).await.unwrap();
        assert_eq!(cache.get("key").await.unwrap(), Some(b"value".to_vec()));
    }

    #[tokio::test]
    async fn test_remove() {
        let cache = DiskCache::new(&test_config(), "test_remove").unwrap();
        cache.set("key", b"value".to_vec(), None).await.unwrap();
        cache.remove("key").await.unwrap();
        assert_eq!(cache.get("key").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = DiskCache::new(&test_config(), "test_ttl").unwrap();
        cache.set("key", b"value".to_vec(), Some(1)).await.unwrap();
        assert_eq!(cache.get("key").await.unwrap(), Some(b"value".to_vec()));
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert_eq!(cache.get("key").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_clear() {
        let cache = DiskCache::new(&test_config(), "test_clear").unwrap();
        cache.set("k1", b"v1".to_vec(), None).await.unwrap();
        cache.set("k2", b"v2".to_vec(), None).await.unwrap();
        cache.clear().await.unwrap();
        assert_eq!(cache.get("k1").await.unwrap(), None);
        assert_eq!(cache.get("k2").await.unwrap(), None);
    }
}
