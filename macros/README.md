# macros

Procedural macros for the fusion-rs project.

## `#[app_cached]` Attribute Macro

An attribute macro that adds caching functionality to async functions.

### Syntax

```rust
#[app_cached(name = "cache_name", ttl = seconds, key = param_name)]
pub async fn function_name(param_name: Type) -> AppResult<ReturnType> {
    // function body
}
```

### Parameters

| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `name` | Yes | String literal | Cache namespace/prefix |
| `ttl` | No | Integer | Time-to-live in seconds |
| `key` | No | Identifier | Parameter name for cache key (defaults to all params) |

### Examples

```rust
use crate::cache::app_cached;
use crate::error::AppResult;

// Basic usage with TTL
#[app_cached(name = "user", ttl = 300, key = user_id)]
pub async fn get_user(&self, user_id: i32) -> AppResult<User> {
    self.db.find_user(user_id).await
}

// Without TTL (uses default)
#[app_cached(name = "config", key = key)]
pub async fn get_config(&self, key: &str) -> AppResult<String> {
    self.load_config(key).await
}

// Long TTL (24 hours)
#[app_cached(name = "resolved_url", ttl = 86400, key = url)]
pub async fn resolve_url(&self, url: &str) -> AppResult<String> {
    self.fetch_and_parse(url).await
}
```

### Cache Key Format

Generated cache key follows the pattern: `"{name}:{key_value}"`

Examples:
- `name = "user", key = user_id` with `user_id = 123` -> `"user:123"`
- `name = "room", key = room_id` with `room_id = "abc"` -> `"room:abc"`

### How It Works

The macro wraps your async function with caching logic:

1. Generates a cache key from the specified parameter
2. Checks if a cached value exists
3. If found, deserializes and returns it
4. If not found, executes the original function
5. On success, serializes and caches the result with the specified TTL
6. Logs errors via `tracing::warn!` without failing the function

### Requirements

- Function must be `async`
- Return type must be `AppResult<T>` where `T: Serialize + DeserializeOwned`
- Cache must be initialized via `crate::cache::init_cache()`

### Features

- **Automatic cache key generation** - Uses format `"{name}:{key_value}"`
- **Optional TTL** - Specify cache expiration in seconds
- **Flexible key selection** - Choose which parameter(s) to use for cache key
- **Graceful degradation** - Falls back to original function if cache unavailable
- **Error logging** - Cache failures are logged but don't break the function
