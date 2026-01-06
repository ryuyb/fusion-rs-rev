# Quick Reference: #[app_cached] Macro

## Syntax

```rust
#[app_cached(name = "cache_name", ttl = seconds, key = param_name)]
pub async fn function_name(param_name: Type) -> AppResult<ReturnType> {
    // function body
}
```

## Parameters

| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `name` | Yes | String literal | Cache namespace/prefix |
| `ttl` | No | Integer | Time-to-live in seconds |
| `key` | Yes | Identifier | Parameter name for cache key |

## Examples

### Basic Usage
```rust
#[app_cached(name = "user", ttl = 300, key = user_id)]
pub async fn get_user(&self, user_id: i32) -> AppResult<User> {
    self.db.find_user(user_id).await
}
```

### Without TTL (no expiration)
```rust
#[app_cached(name = "config", key = key)]
pub async fn get_config(&self, key: &str) -> AppResult<String> {
    self.load_config(key).await
}
```

### Long TTL (24 hours)
```rust
#[app_cached(name = "douyin_url", ttl = 86400, key = url)]
pub async fn resolve_url(&self, url: &str) -> AppResult<String> {
    self.fetch_and_parse(url).await
}
```

## Cache Key Format

The generated cache key follows the pattern: `"{name}:{key_value}"`

Examples:
- `name = "user", key = user_id` with `user_id = 123` → `"user:123"`
- `name = "room", key = room_id` with `room_id = "abc"` → `"room:abc"`

## Requirements

1. Function must be `async`
2. Return type must be `AppResult<T>`
3. Return type `T` must implement `Serialize + DeserializeOwned`
4. Cache must be initialized before use

## Comparison with Declarative Macro

### Attribute Macro (New)
```rust
#[app_cached(name = "room", ttl = 60, key = room_id)]
pub async fn get_room(&self, room_id: &str) -> AppResult<Room> {
    self.fetch_room(room_id).await
}
```

### Declarative Macro (Existing)
```rust
app_cached! {
    name = "room",
    ttl = 60,
    key = |room_id: &str| room_id.to_string(),
    async fn get_room(room_id: &str) -> AppResult<Room> {
        fetch_room(room_id).await
    }
}
```

## When to Use

Use `#[app_cached]` when:
- You want attribute-style syntax
- You have simple key generation (single parameter)
- You prefer method-style functions

Use `app_cached!` when:
- You need complex key generation logic
- You want to define standalone cached functions
- You need explicit cache parameter passing
