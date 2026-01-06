//! Memory cache implementation with per-entry TTL support using DashMap.

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
}

impl MemoryCache {
    pub fn new(config: &MemoryCacheConfig) -> Self {
        Self {
            store: DashMap::new(),
            max_size: config.max_size,
            default_ttl: Duration::from_secs(config.ttl_seconds),
        }
    }

    fn evict_expired(&self) {
        let now = Instant::now();
        self.store.retain(|_, entry| entry.expires_at > now);
    }
}

#[async_trait]
impl AppCache for MemoryCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
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
        // Evict expired entries if at capacity
        if self.store.len() >= self.max_size {
            self.evict_expired();
        }

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
