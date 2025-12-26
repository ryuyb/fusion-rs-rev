//! Application state for Axum web framework.
//!
//! Contains shared services and resources that are accessible
//! across all request handlers.

use crate::db::AsyncDbPool;
use crate::repositories::Repositories;
use crate::services::Services;

/// Application state containing all shared services.
///
/// This struct is designed to be used with Axum's State extractor.
/// Cloning is cheap since Services uses Arc internally via the
/// underlying database connection pool.
#[derive(Clone)]
pub struct AppState {
    /// All business logic services
    pub services: Services,
}

impl AppState {
    /// Creates a new AppState from a database connection pool.
    ///
    /// Initializes all repositories and services from the provided pool.
    ///
    /// # Arguments
    /// * `pool` - The async database connection pool
    ///
    /// # Example
    /// ```ignore
    /// let pool = establish_async_connection_pool().await?;
    /// let state = AppState::new(pool);
    /// ```
    pub fn new(pool: AsyncDbPool) -> Self {
        let repos = Repositories::new(pool);
        let services = Services::new(repos);
        Self { services }
    }
}
