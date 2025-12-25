//! Async database connection pool implementation.
//!
//! Uses bb8 connection pool manager with diesel_async for PostgreSQL connections.

use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;

use crate::error::AppError;

/// Async connection pool type alias.
///
/// bb8::Pool internally uses Arc, so Clone is cheap (just reference count increment).
/// Structures holding AsyncDbPool can derive Clone without additional Arc wrapping.
pub type AsyncDbPool = Pool<AsyncPgConnection>;

/// Creates an async database connection pool.
///
/// Reads DATABASE_URL from environment variables and establishes a connection pool.
///
/// # Returns
///
/// Returns `Ok(AsyncDbPool)` on success, or `AppError` on failure.
///
/// # Errors
///
/// - `AppError::Env` - If DATABASE_URL environment variable is not set
/// - `AppError::PoolBuild` - If connection pool creation fails
///
/// # Example
///
/// ```ignore
/// let pool = establish_async_connection_pool().await?;
/// let mut conn = pool.get().await?;
/// ```
pub async fn establish_async_connection_pool() -> Result<AsyncDbPool, AppError> {
    let database_url = std::env::var("DATABASE_URL")?;
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder().build(config).await?;
    Ok(pool)
}
