//! AppCache trait definition.

use async_trait::async_trait;

use crate::cache::CacheError;

/// Trait for cache operations.
///
/// All cache backends must implement this trait to provide a unified interface.
#[async_trait]
pub trait AppCache: Send + Sync {
    /// Get a value from the cache.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;

    /// Set a value in the cache with optional TTL override.
    async fn set(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl_seconds: Option<u64>,
    ) -> Result<(), CacheError>;

    /// Remove a value from the cache.
    async fn remove(&self, key: &str) -> Result<(), CacheError>;

    /// Clear all values from the cache.
    async fn clear(&self) -> Result<(), CacheError>;
}
