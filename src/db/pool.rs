//! Async database connection pool implementation.
//!
//! Uses bb8 connection pool manager with diesel_async for PostgreSQL connections.

use std::time::Duration;

use crate::config::DatabaseConfig;
use crate::error::AppError;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Async connection pool type alias.
///
/// bb8::Pool internally uses Arc, so Clone is cheap (just reference count increment).
/// Structures holding AsyncDbPool can derive Clone without additional Arc wrapping.
pub type AsyncDbPool = Pool<AsyncPgConnection>;

/// Creates an async database connection pool from configuration.
///
/// Uses the provided DatabaseConfig to configure the connection pool with:
/// - Database URL from config (falls back to DATABASE_URL env var if empty)
/// - Maximum connections
/// - Minimum idle connections
/// - Connection timeout
///
/// # Arguments
///
/// * `config` - Database configuration containing URL and pool settings
///
/// # Returns
///
/// Returns `Ok(AsyncDbPool)` on success, or `AppError` on failure.
///
/// # Errors
///
/// - `AppError::Env` - If database URL is not configured and DATABASE_URL env var is not set
/// - `AppError::PoolBuild` - If connection pool creation fails
///
/// # Example
///
/// ```ignore
/// let db_config = DatabaseConfig {
///     url: "postgres://localhost/mydb".to_string(),
///     max_connections: 10,
///     min_connections: 1,
///     connection_timeout: 30,
/// };
/// let pool = establish_async_connection_pool(&db_config).await?;
/// let mut conn = pool.get().await?;
/// ```
pub async fn establish_async_connection_pool(
    config: &DatabaseConfig,
) -> Result<AsyncDbPool, AppError> {
    let database_url = config.url.clone();
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url.clone());

    let pool = Pool::builder()
        .max_size(config.max_connections)
        .min_idle(Some(config.min_connections))
        .connection_timeout(Duration::from_secs(config.connection_timeout))
        .build(manager)
        .await?;

    // Run pending migrations if auto_migrate is enabled
    if config.auto_migrate {
        tracing::info!("Running database migrations...");
        
        let migrations_result = tokio::task::spawn_blocking(move || {
            use diesel::pg::PgConnection;
            use diesel::Connection;

            let mut conn = PgConnection::establish(&database_url)
                .map_err(|e| AppError::Migration(Box::new(e)))?;
            let applied = conn
                .run_pending_migrations(MIGRATIONS)
                .map_err(|e| AppError::Migration(e))?;
            // Convert to owned strings to avoid lifetime issues
            let migration_names: Vec<String> = applied.iter().map(|m| m.to_string()).collect();
            Ok::<_, AppError>(migration_names)
        })
        .await
        .map_err(|e| AppError::Migration(Box::new(e)))??;

        if migrations_result.is_empty() {
            tracing::info!("Database migrations completed: no pending migrations");
        } else {
            tracing::info!(
                count = migrations_result.len(),
                migrations = ?migrations_result,
                "Database migrations completed successfully"
            );
        }
    }

    Ok(pool)
}
