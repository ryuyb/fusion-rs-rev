//! Database connection pool module.
//!
//! Provides async PostgreSQL connection pooling using diesel_async with bb8.

mod pool;

pub use pool::{AsyncDbPool, MIGRATIONS, establish_async_connection_pool};
