//! Cache error types.

use thiserror::Error;

/// Errors that can occur during cache operations.
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache operation failed: {0}")]
    Operation(String),

    #[error("Cache connection failed: {0}")]
    Connection(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Cache not initialized")]
    NotInitialized,
}
