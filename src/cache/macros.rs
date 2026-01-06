//! Cache macros for simplified caching.

/// Macro for defining cached async functions with runtime backend selection.
///
/// This macro wraps an async function with caching logic, automatically handling
/// cache key generation, retrieval, and storage.
///
/// # Usage
///
/// ```ignore
/// // With explicit cache parameter
/// app_cached! {
///     name = "room_info",
///     ttl = 60,
///     key = |room_id: &str| format!("room:{}", room_id),
///     async fn get_cached_room_info(cache: &CacheManager, room_id: &str) -> AppResult<RoomInfo> {
///         provider.get_room_info(room_id).await
///     }
/// }
///
/// // Using global cache (no cache parameter)
/// app_cached! {
///     name = "user_info",
///     key = |user_id: i32| user_id.to_string(),
///     async fn get_cached_user(user_id: i32) -> AppResult<User> {
///         db.find_user(user_id).await
///     }
/// }
/// ```
///
/// # Parameters
///
/// - `name`: A unique name for this cache (used as prefix for cache keys)
/// - `ttl` (optional): Time-to-live in seconds for cached values
/// - `key`: A closure that generates the cache key from function arguments
/// - `async fn`: The async function definition
#[macro_export]
macro_rules! app_cached {
    // Version with explicit cache parameter
    (
        name = $cache_name:literal,
        $(ttl = $ttl:expr,)?
        key = |$($key_arg:ident : $key_ty:ty),* $(,)?| $key_expr:expr,
        async fn $fn_name:ident($cache_param:ident : &CacheManager $(, $arg:ident : $arg_ty:ty)* $(,)?) -> $ret_ty:ty $body:block
    ) => {
        pub async fn $fn_name(
            $cache_param: &$crate::cache::CacheManager,
            $($arg: $arg_ty),*
        ) -> $ret_ty {
            let cache_key = {
                $(let $key_arg: $key_ty = &$arg;)*
                format!("{}:{}", $cache_name, $key_expr)
            };

            if let Ok(Some(cached_bytes)) = $cache_param.get(&cache_key).await {
                if let Ok(cached_value) = serde_json::from_slice(&cached_bytes) {
                    return Ok(cached_value);
                }
            }

            let result: $ret_ty = (|| async $body)().await;

            if let Ok(ref value) = result {
                if let Ok(bytes) = serde_json::to_vec(value) {
                    let ttl = $crate::app_cached!(@ttl $($ttl)?);
                    let _ = $cache_param.set(&cache_key, bytes, ttl).await;
                }
            }

            result
        }
    };

    // Version using global cache (no cache parameter)
    (
        name = $cache_name:literal,
        $(ttl = $ttl:expr,)?
        key = |$($key_arg:ident : $key_ty:ty),* $(,)?| $key_expr:expr,
        async fn $fn_name:ident($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret_ty:ty $body:block
    ) => {
        pub async fn $fn_name($($arg: $arg_ty),*) -> $ret_ty {
            if let Some(cache) = $crate::cache::get_cache() {
                let cache_key = {
                    $(let $key_arg: $key_ty = &$arg;)*
                    format!("{}:{}", $cache_name, $key_expr)
                };

                if let Ok(Some(cached_bytes)) = cache.get(&cache_key).await {
                    if let Ok(cached_value) = serde_json::from_slice(&cached_bytes) {
                        return Ok(cached_value);
                    }
                }

                let result: $ret_ty = (|| async $body)().await;

                if let Ok(ref value) = result {
                    if let Ok(bytes) = serde_json::to_vec(value) {
                        let ttl = $crate::app_cached!(@ttl $($ttl)?);
                        let _ = cache.set(&cache_key, bytes, ttl).await;
                    }
                }

                result
            } else {
                (|| async $body)().await
            }
        }
    };

    (@ttl) => { None };
    (@ttl $ttl:expr) => { Some($ttl) };
}
