//! Cache manager that dispatches to the configured backend.

use std::sync::Arc;

use tokio::sync::OnceCell;

use crate::cache::disk::DiskCache;
use crate::cache::memory::MemoryCache;
use crate::cache::noop::NoOpCache;
use crate::cache::redis::RedisCache;
use crate::cache::{AppCache, CacheError};
use crate::config::settings::{CacheBackend, CacheConfig};

/// Global cache manager instance.
static CACHE: OnceCell<CacheManager> = OnceCell::const_new();

/// Initialize the global cache manager.
///
/// This should be called once during application startup.
/// Subsequent calls will return the existing instance.
pub async fn init_cache(
    config: CacheConfig,
    cache_name: &str,
) -> Result<&'static CacheManager, CacheError> {
    CACHE
        .get_or_try_init(|| async { CacheManager::new(config, cache_name).await })
        .await
}

/// Get the global cache manager.
///
/// Returns `None` if the cache has not been initialized.
pub fn get_cache() -> Option<&'static CacheManager> {
    CACHE.get()
}

/// Cache manager that provides access to the configured cache backend.
#[derive(Clone)]
pub struct CacheManager {
    backend: Arc<dyn AppCache>,
    config: CacheConfig,
}

impl CacheManager {
    /// Create a new cache manager with the given configuration.
    ///
    /// If caching is disabled, a NoOpCache is used.
    pub async fn new(config: CacheConfig, cache_name: &str) -> Result<Self, CacheError> {
        let backend: Arc<dyn AppCache> = if !config.enabled {
            Arc::new(NoOpCache::new())
        } else {
            match config.backend {
                CacheBackend::Memory => Arc::new(MemoryCache::new(&config.memory)),
                CacheBackend::Disk => Arc::new(DiskCache::new(&config.disk, cache_name)?),
                CacheBackend::Redis => Arc::new(RedisCache::new(&config.redis, cache_name).await?),
            }
        };

        Ok(Self { backend, config })
    }

    /// Get a reference to the cache backend.
    pub fn backend(&self) -> &Arc<dyn AppCache> {
        &self.backend
    }

    /// Get the cache configuration.
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Check if caching is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    // ========================================================================
    // AppCache proxy methods
    // ========================================================================

    /// Get a value from the cache.
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        self.backend.get(key).await
    }

    /// Set a value in the cache.
    pub async fn set(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl_seconds: Option<u64>,
    ) -> Result<(), CacheError> {
        self.backend.set(key, value, ttl_seconds).await
    }

    /// Remove a value from the cache.
    pub async fn remove(&self, key: &str) -> Result<(), CacheError> {
        self.backend.remove(key).await
    }

    /// Clear all values from the cache.
    pub async fn clear(&self) -> Result<(), CacheError> {
        self.backend.clear().await
    }
}
