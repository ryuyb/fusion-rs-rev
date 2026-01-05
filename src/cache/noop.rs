//! NoOp cache implementation.
//!
//! Used when caching is disabled. All operations are no-ops.

use async_trait::async_trait;

use crate::cache::{AppCache, CacheError};

/// A no-operation cache that doesn't store anything.
///
/// Used when `cache.enabled = false` in configuration.
pub struct NoOpCache;

impl NoOpCache {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoOpCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AppCache for NoOpCache {
    async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        Ok(None)
    }

    async fn set(
        &self,
        _key: &str,
        _value: Vec<u8>,
        _ttl_seconds: Option<u64>,
    ) -> Result<(), CacheError> {
        Ok(())
    }

    async fn remove(&self, _key: &str) -> Result<(), CacheError> {
        Ok(())
    }

    async fn clear(&self) -> Result<(), CacheError> {
        Ok(())
    }
}
