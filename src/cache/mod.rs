//! Cache module providing runtime-configurable caching with multiple backends.
//!
//! This module provides a unified caching interface that supports:
//! - Memory cache (in-process, fastest)
//! - Disk cache (persistent, file-based)
//! - Redis cache (distributed, network-based)
//!
//! # Configuration
//!
//! Configure caching in your TOML config file:
//!
//! ```toml
//! [cache]
//! enabled = true
//! backend = "memory"  # or "disk" or "redis"
//!
//! [cache.memory]
//! max_size = 1000
//! ttl_seconds = 300
//!
//! [cache.disk]
//! directory = "cache"
//! ttl_seconds = 300
//!
//! [cache.redis]
//! url = "redis://127.0.0.1:6379"
//! ttl_seconds = 300
//! pool_size = 4
//! connection_timeout = 5
//! key_prefix = "fusion"
//! tls_enabled = false
//! ```
//!
//! # Usage
//!
//! Use the `app_cached!` macro to define cached functions:
//!
//! ```ignore
//! app_cached! {
//!     name = "room_info",
//!     ttl = 60,
//!     key = |room_id: &str| room_id.to_string(),
//!     async fn get_cached_room_info(cache: &CacheManager, room_id: &str) -> AppResult<RoomInfo> {
//!         provider.get_room_info(room_id).await
//!     }
//! }
//! ```

mod disk;
mod error;
#[macro_use]
mod macros;
mod manager;
mod memory;
mod noop;
mod redis;
mod traits;

pub use error::CacheError;
pub use fusion_macros::app_cached;
pub use manager::{CacheManager, get_cache, init_cache};
pub use traits::AppCache;

// Re-export config types
pub use crate::config::settings::{
    CacheBackend, CacheConfig, DiskCacheConfig, MemoryCacheConfig, RedisCacheConfig,
};
