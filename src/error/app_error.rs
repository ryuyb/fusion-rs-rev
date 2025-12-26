use diesel_async::pooled_connection::bb8;
use thiserror::Error;

/// Application-wide error type for database and repository operations.
///
/// This enum provides structured error handling for all data access layer operations,
/// with automatic conversion from underlying error types via the `From` trait.
#[derive(Debug, Error)]
pub enum AppError {
    /// Database operation error (query, insert, update, delete failures)
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    /// Connection pool runtime error (pool exhausted, connection timeout)
    #[error("Connection pool error: {0}")]
    Pool(#[from] bb8::RunError),

    /// Connection pool build error (failed to create pool)
    #[error("Connection pool build error: {0}")]
    PoolBuild(#[from] diesel_async::pooled_connection::PoolError),

    /// Environment variable error (missing or invalid configuration)
    #[error("Environment error: {0}")]
    Env(#[from] std::env::VarError),

    /// Database migration error
    #[error("Migration error: {0}")]
    Migration(Box<dyn std::error::Error + Send + Sync>),

    /// Resource not found error
    #[error("Resource not found")]
    NotFound,
}
