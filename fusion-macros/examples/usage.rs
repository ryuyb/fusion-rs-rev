//! Example usage of the #[app_cached] attribute macro
//!
//! This example demonstrates how to use the attribute macro for caching.

use fusion_rs::cache::app_cached;
use fusion_rs::error::AppResult;

// Example 1: Simple caching with a single key parameter
#[app_cached(name = "user_data", ttl = 300, key = user_id)]
pub async fn get_user_data(user_id: i32) -> AppResult<String> {
    // Simulate expensive operation
    Ok(format!("User data for {}", user_id))
}

// Example 2: Caching with string key
#[app_cached(name = "room_info", ttl = 60, key = room_id)]
pub async fn get_room_info(room_id: &str) -> AppResult<String> {
    // Simulate API call
    Ok(format!("Room info for {}", room_id))
}

// Example 3: Caching without TTL (uses default)
#[app_cached(name = "config_data", key = config_key)]
pub async fn get_config(config_key: &str) -> AppResult<String> {
    // Simulate config lookup
    Ok(format!("Config value for {}", config_key))
}

// Example 4: Multiple parameters (will use all for cache key if not specified)
#[app_cached(name = "multi_key", ttl = 120, key = param1)]
pub async fn get_multi_data(param1: &str, param2: i32) -> AppResult<String> {
    // Only param1 is used for cache key
    Ok(format!("Data for {} and {}", param1, param2))
}

fn main() {
    println!("See the function definitions above for usage examples");
}
