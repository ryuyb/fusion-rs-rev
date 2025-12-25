//! Repository layer for data access operations.
//!
//! Provides async CRUD operations for all domain entities.

mod user_repo;

pub use user_repo::UserRepository;

use crate::db::AsyncDbPool;

/// Aggregates all repositories for convenient access.
///
/// This struct is designed to be used as Axum application state.
/// Since `AsyncDbPool` uses `Arc` internally, cloning is cheap.
#[derive(Clone)]
pub struct Repositories {
    pub users: UserRepository,
}

impl Repositories {
    /// Creates a new Repositories instance with all repositories initialized.
    ///
    /// # Arguments
    /// * `pool` - The async database connection pool
    pub fn new(pool: AsyncDbPool) -> Self {
        Self {
            users: UserRepository::new(pool),
        }
    }
}
