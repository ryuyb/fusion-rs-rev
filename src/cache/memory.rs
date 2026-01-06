//! Memory cache implementation with per-entry TTL support using DashMap.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use dashmap::DashMap;

use crate::cache::{AppCache, CacheError};
use crate::config::settings::MemoryCacheConfig;

struct CacheEntry {
    value: Vec<u8>,
    expires_at: Instant,
}

/// In-memory cache with size limit and per-entry TTL using DashMap for concurrent access.
pub struct MemoryCache {
    store: DashMap<String, CacheEntry>,
    max_size: usize,
    default_ttl: Duration,
    op_counter: AtomicU64,
}

const EVICTION_INTERVAL: u64 = 100; // Evict every N operations

impl MemoryCache {
    pub fn new(config: &MemoryCacheConfig) -> Self {
        Self {
            store: DashMap::new(),
            max_size: config.max_size,
            default_ttl: Duration::from_secs(config.ttl_seconds),
            op_counter: AtomicU64::new(0),
        }
    }

    fn evict_expired(&self) {
        let now = Instant::now();
        self.store.retain(|_, entry| entry.expires_at > now);
    }

    fn maybe_evict(&self) {
        let count = self.op_counter.fetch_add(1, Ordering::Relaxed);
        // Evict periodically or when approaching capacity
        if count.is_multiple_of(EVICTION_INTERVAL) || self.store.len() >= self.max_size * 3 / 4 {
            self.evict_expired();
        }
    }
}

#[async_trait]
impl AppCache for MemoryCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        self.maybe_evict();
        if let Some(entry) = self.store.get(key) {
            if entry.expires_at > Instant::now() {
                return Ok(Some(entry.value.clone()));
            }
            drop(entry);
            self.store.remove(key);
        }
        Ok(None)
    }

    async fn set(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl_seconds: Option<u64>,
    ) -> Result<(), CacheError> {
        self.maybe_evict();

        let ttl = ttl_seconds
            .map(Duration::from_secs)
            .unwrap_or(self.default_ttl);
        let expires_at = Instant::now() + ttl;

        self.store
            .insert(key.to_string(), CacheEntry { value, expires_at });
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<(), CacheError> {
        self.store.remove(key);
        Ok(())
    }

    async fn clear(&self) -> Result<(), CacheError> {
        self.store.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> MemoryCacheConfig {
        MemoryCacheConfig {
            max_size: 2,
            ttl_seconds: 3600,
        }
    }

    #[tokio::test]
    async fn test_get_set() {
        let cache = MemoryCache::new(&test_config());
        cache.set("key", b"value".to_vec(), None).await.unwrap();
        assert_eq!(cache.get("key").await.unwrap(), Some(b"value".to_vec()));
    }

    #[tokio::test]
    async fn test_remove() {
        let cache = MemoryCache::new(&test_config());
        cache.set("key", b"value".to_vec(), None).await.unwrap();
        cache.remove("key").await.unwrap();
        assert_eq!(cache.get("key").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_clear() {
        let cache = MemoryCache::new(&test_config());
        cache.set("k1", b"v1".to_vec(), None).await.unwrap();
        cache.set("k2", b"v2".to_vec(), None).await.unwrap();
        cache.clear().await.unwrap();
        assert_eq!(cache.get("k1").await.unwrap(), None);
        assert_eq!(cache.get("k2").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = MemoryCache::new(&test_config());
        cache.set("key", b"value".to_vec(), Some(1)).await.unwrap();
        assert_eq!(cache.get("key").await.unwrap(), Some(b"value".to_vec()));
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert_eq!(cache.get("key").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_max_size_eviction() {
        let cache = MemoryCache::new(&test_config());
        cache.set("k1", b"v1".to_vec(), Some(1)).await.unwrap();
        cache.set("k2", b"v2".to_vec(), None).await.unwrap();
        tokio::time::sleep(Duration::from_secs(2)).await;
        cache.set("k3", b"v3".to_vec(), None).await.unwrap();
        assert_eq!(cache.get("k1").await.unwrap(), None);
        assert_eq!(cache.get("k2").await.unwrap(), Some(b"v2".to_vec()));
        assert_eq!(cache.get("k3").await.unwrap(), Some(b"v3".to_vec()));
    }
}
