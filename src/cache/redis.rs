//! Redis cache implementation using bb8 connection pool.

use async_trait::async_trait;
use bb8::{Pool, PooledConnection};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, RedisError};

use crate::cache::{AppCache, CacheError};
use crate::config::settings::RedisCacheConfig;

type RedisPool = Pool<Client>;

/// Redis-based cache with bb8 connection pool.
pub struct RedisCache {
    pool: RedisPool,
    key_prefix: String,
    default_ttl: u64,
}

impl RedisCache {
    pub async fn new(config: &RedisCacheConfig, cache_name: &str) -> Result<Self, CacheError> {
        let client =
            Client::open(config.url.as_str()).map_err(|e| CacheError::Connection(e.to_string()))?;

        let pool = Pool::builder()
            .max_size(config.pool_size)
            .connection_timeout(std::time::Duration::from_secs(config.connection_timeout))
            .build(client)
            .await
            .map_err(|e| CacheError::Connection(e.to_string()))?;

        let key_prefix = format!("{}:{}", config.key_prefix, cache_name);

        Ok(Self {
            pool,
            key_prefix,
            default_ttl: config.ttl_seconds,
        })
    }

    fn prefixed_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    async fn get_conn(&self) -> Result<PooledConnection<'_, Client>, CacheError> {
        self.pool
            .get()
            .await
            .map_err(|e| CacheError::Connection(e.to_string()))
    }
}

#[async_trait]
impl AppCache for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let mut conn: PooledConnection<'_, Client> = self.get_conn().await?;
        let prefixed = self.prefixed_key(key);

        let conn_ref: &mut MultiplexedConnection = &mut conn;
        conn_ref
            .get(&prefixed)
            .await
            .map_err(|e: RedisError| CacheError::Operation(e.to_string()))
    }

    async fn set(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl_seconds: Option<u64>,
    ) -> Result<(), CacheError> {
        let mut conn: PooledConnection<'_, Client> = self.get_conn().await?;
        let prefixed = self.prefixed_key(key);
        let ttl = ttl_seconds.unwrap_or(self.default_ttl);

        let conn_ref: &mut MultiplexedConnection = &mut conn;
        conn_ref
            .set_ex::<_, _, ()>(&prefixed, value, ttl)
            .await
            .map_err(|e| CacheError::Operation(e.to_string()))
    }

    async fn remove(&self, key: &str) -> Result<(), CacheError> {
        let mut conn: PooledConnection<'_, Client> = self.get_conn().await?;
        let prefixed = self.prefixed_key(key);

        let conn_ref: &mut MultiplexedConnection = &mut conn;
        conn_ref
            .del::<_, ()>(&prefixed)
            .await
            .map_err(|e| CacheError::Operation(e.to_string()))
    }

    async fn clear(&self) -> Result<(), CacheError> {
        let mut conn: PooledConnection<'_, Client> = self.get_conn().await?;
        let pattern = format!("{}:*", self.key_prefix);

        let conn_ref: &mut MultiplexedConnection = &mut conn;
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(conn_ref)
            .await
            .map_err(|e: RedisError| CacheError::Operation(e.to_string()))?;

        if !keys.is_empty() {
            let conn_ref: &mut MultiplexedConnection = &mut conn;
            conn_ref
                .del::<_, ()>(keys)
                .await
                .map_err(|e| CacheError::Operation(e.to_string()))?;
        }

        Ok(())
    }
}
