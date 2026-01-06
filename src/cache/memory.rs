//! Memory cache implementation with per-entry TTL support.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::cache::{AppCache, CacheError};
use crate::config::settings::MemoryCacheConfig;

struct CacheEntry {
    value: Vec<u8>,
    expires_at: Option<Instant>,
}

/// In-memory cache with size limit and per-entry TTL.
pub struct MemoryCache {
    store: Mutex<HashMap<String, CacheEntry>>,
    max_size: usize,
    default_ttl: Duration,
}

impl MemoryCache {
    pub fn new(config: &MemoryCacheConfig) -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
            max_size: config.max_size,
            default_ttl: Duration::from_secs(config.ttl_seconds),
        }
    }

    fn evict_expired(store: &mut HashMap<String, CacheEntry>) {
        let now = Instant::now();
        store.retain(|_, entry| entry.expires_at.is_none_or(|exp| exp > now));
    }
}

#[async_trait]
impl AppCache for MemoryCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let mut store = self.store.lock().await;

        if let Some(entry) = store.get(key) {
            if entry.expires_at.is_none_or(|exp| exp > Instant::now()) {
                return Ok(Some(entry.value.clone()));
            }
            store.remove(key);
        }
        Ok(None)
    }

    async fn set(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl_seconds: Option<u64>,
    ) -> Result<(), CacheError> {
        let mut store = self.store.lock().await;

        // Evict expired entries if at capacity
        if store.len() >= self.max_size {
            Self::evict_expired(&mut store);
        }

        let ttl = ttl_seconds
            .map(Duration::from_secs)
            .unwrap_or(self.default_ttl);
        let expires_at = Some(Instant::now() + ttl);

        store.insert(key.to_string(), CacheEntry { value, expires_at });
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<(), CacheError> {
        let mut store = self.store.lock().await;
        store.remove(key);
        Ok(())
    }

    async fn clear(&self) -> Result<(), CacheError> {
        let mut store = self.store.lock().await;
        store.clear();
        Ok(())
    }
}
