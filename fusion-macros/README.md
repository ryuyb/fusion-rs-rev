# fusion-macros

Procedural macros for the fusion-rs project.

## `#[app_cached]` Attribute Macro

An attribute macro that adds caching functionality to async functions.

### Usage

```rust
use fusion_rs::cache::app_cached;
use fusion_rs::error::AppResult;

#[app_cached(name = "user_data", ttl = 300, key = user_id)]
pub async fn get_user_data(user_id: i32) -> AppResult<String> {
    // Expensive operation here
    Ok(format!("User data for {}", user_id))
}
```

### Parameters

- `name`: Cache name prefix (required, string literal)
- `ttl`: Time-to-live in seconds (optional, integer)
- `key`: Parameter name(s) to use for cache key generation (required, identifier)

### How It Works

The macro wraps your async function with caching logic:

1. Generates a cache key using the format: `"{name}:{key_value}"`
2. Checks if a cached value exists
3. If found, deserializes and returns it
4. If not found, executes the original function
5. On success, serializes and caches the result with the specified TTL

### Requirements

- Function must be async
- Return type must be `AppResult<T>` where `T: Serialize + DeserializeOwned`
- The global cache must be initialized via `fusion_rs::cache::init_cache()`

### Examples

See `examples/usage.rs` for more examples.
