//! Configuration settings structures for fusion-rs
//!
//! This module defines all configuration structures that can be loaded from
//! TOML files and environment variables.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::error::ConfigError;
use crate::logger::{
    ConsoleConfig, FileConfig, LogFormat, LoggerConfig, RotationConfig, RotationStrategy,
};

// ============================================================================
// Default value functions
// ============================================================================

fn default_app_name() -> String {
    "fusion-rs".to_string()
}

fn default_app_version() -> String {
    crate::pkg_version().to_string()
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_request_timeout() -> u64 {
    30
}

fn default_keep_alive_timeout() -> u64 {
    75
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    1
}

fn default_connection_timeout() -> u64 {
    30
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_true() -> bool {
    true
}

fn default_log_path() -> String {
    "logs/app.log".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

fn default_rotation_strategy() -> String {
    "size".to_string()
}

fn default_max_size() -> u64 {
    10 * 1024 * 1024 // 10MB
}

fn default_max_files() -> usize {
    5
}

fn default_jwt_secret() -> String {
    String::new()
}

fn default_access_token_expiration() -> i64 {
    1 // 1 hour
}

fn default_refresh_token_expiration() -> i64 {
    168 // 7 days (168 hours)
}

// ============================================================================
// Application Configuration
// ============================================================================

/// Application basic information configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Application name
    #[serde(default = "default_app_name")]
    pub name: String,

    /// Application version
    #[serde(default = "default_app_version")]
    pub version: String,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            version: default_app_version(),
        }
    }
}

// ============================================================================
// Server Configuration
// ============================================================================

/// Axum HTTP server configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,

    /// Keep-alive timeout in seconds
    #[serde(default = "default_keep_alive_timeout")]
    pub keep_alive_timeout: u64,
}

impl ServerConfig {
    /// Get the full server address as "host:port"
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            request_timeout: default_request_timeout(),
            keep_alive_timeout: default_keep_alive_timeout(),
        }
    }
}

// ============================================================================
// Database Configuration
// ============================================================================

/// Diesel database connection configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    #[serde(default)]
    pub url: String,

    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum number of connections in the pool
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,

    /// Whether to automatically run pending migrations on startup
    #[serde(default)]
    pub auto_migrate: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connection_timeout: default_connection_timeout(),
            auto_migrate: false,
        }
    }
}

// ============================================================================
// JWT Configuration
// ============================================================================

/// JWT authentication configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Secret key for signing JWT tokens
    /// IMPORTANT: This should be a strong, random string in production
    /// and should be kept secret (use environment variables)
    #[serde(default = "default_jwt_secret")]
    pub secret: String,

    /// Access token expiration time in hours
    #[serde(default = "default_access_token_expiration")]
    pub access_token_expiration: i64,

    /// Refresh token expiration time in hours
    #[serde(default = "default_refresh_token_expiration")]
    pub refresh_token_expiration: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: default_jwt_secret(),
            access_token_expiration: default_access_token_expiration(),
            refresh_token_expiration: default_refresh_token_expiration(),
        }
    }
}

impl JwtConfig {
    /// Validates the JWT configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.secret.is_empty() {
            return Err(ConfigError::ValidationError {
                field: "jwt.secret".to_string(),
                message: "JWT secret cannot be empty".to_string(),
            });
        }

        if self.secret.len() < 32 {
            return Err(ConfigError::ValidationError {
                field: "jwt.secret".to_string(),
                message: "JWT secret should be at least 32 characters for security".to_string(),
            });
        }

        if self.access_token_expiration <= 0 {
            return Err(ConfigError::ValidationError {
                field: "jwt.access_token_expiration".to_string(),
                message: "Access token expiration must be positive".to_string(),
            });
        }

        if self.refresh_token_expiration <= 0 {
            return Err(ConfigError::ValidationError {
                field: "jwt.refresh_token_expiration".to_string(),
                message: "Refresh token expiration must be positive".to_string(),
            });
        }

        if self.access_token_expiration >= self.refresh_token_expiration {
            return Err(ConfigError::ValidationError {
                field: "jwt".to_string(),
                message: "Refresh token expiration should be longer than access token expiration"
                    .to_string(),
            });
        }

        Ok(())
    }
}

// ============================================================================
// Logger Settings (compatible with existing LoggerConfig)
// ============================================================================

/// Console output settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsoleSettings {
    /// Whether console output is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Whether to use colored output
    #[serde(default = "default_true")]
    pub colored: bool,
}

impl Default for ConsoleSettings {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            colored: default_true(),
        }
    }
}

/// Rotation settings for file logging
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotationSettings {
    /// Rotation strategy: "size", "time", "count", or "combined"
    #[serde(default = "default_rotation_strategy")]
    pub strategy: String,

    /// Maximum file size in bytes before rotation
    #[serde(default = "default_max_size")]
    pub max_size: u64,

    /// Maximum number of rotated files to keep
    #[serde(default = "default_max_files")]
    pub max_files: usize,

    /// Whether to compress rotated files
    #[serde(default)]
    pub compress: bool,
}

impl Default for RotationSettings {
    fn default() -> Self {
        Self {
            strategy: default_rotation_strategy(),
            max_size: default_max_size(),
            max_files: default_max_files(),
            compress: false,
        }
    }
}

/// File output settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSettings {
    /// Whether file output is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Path to the log file
    #[serde(default = "default_log_path")]
    pub path: String,

    /// Whether to append to existing file
    #[serde(default = "default_true")]
    pub append: bool,

    /// Log format: "full", "compact", or "json"
    #[serde(default = "default_log_format")]
    pub format: String,

    /// Rotation settings
    #[serde(default)]
    pub rotation: RotationSettings,
}

impl Default for FileSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            path: default_log_path(),
            append: default_true(),
            format: default_log_format(),
            rotation: RotationSettings::default(),
        }
    }
}

/// Logger configuration settings (compatible with existing LoggerConfig)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggerSettings {
    /// Log level: "trace", "debug", "info", "warn", "error"
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Console output settings
    #[serde(default)]
    pub console: ConsoleSettings,

    /// File output settings
    #[serde(default)]
    pub file: FileSettings,
}

impl Default for LoggerSettings {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            console: ConsoleSettings::default(),
            file: FileSettings::default(),
        }
    }
}

impl LoggerSettings {
    /// Convert LoggerSettings to LoggerConfig
    ///
    /// This method transforms the configuration file representation into
    /// the runtime LoggerConfig used by the logger module.
    pub fn into_logger_config(self) -> Result<LoggerConfig, ConfigError> {
        let console_config = self.console.into_console_config();
        let file_config = self.file.into_file_config()?;

        LoggerConfig::new(console_config, file_config, self.level).map_err(|e| {
            ConfigError::ValidationError {
                field: "logger".to_string(),
                message: e.to_string(),
            }
        })
    }
}

impl ConsoleSettings {
    /// Convert ConsoleSettings to ConsoleConfig
    pub fn into_console_config(self) -> ConsoleConfig {
        ConsoleConfig::new(self.enabled, self.colored)
    }
}

impl FileSettings {
    /// Convert FileSettings to FileConfig
    pub fn into_file_config(self) -> Result<FileConfig, ConfigError> {
        let format = self.parse_format()?;
        let rotation_config = self.rotation.into_rotation_config()?;

        FileConfig::new(
            self.enabled,
            PathBuf::from(self.path),
            self.append,
            format,
            rotation_config,
        )
        .map_err(|e| ConfigError::ValidationError {
            field: "logger.file".to_string(),
            message: e.to_string(),
        })
    }

    /// Parse the format string into LogFormat enum
    fn parse_format(&self) -> Result<LogFormat, ConfigError> {
        self.format
            .parse::<LogFormat>()
            .map_err(|e| ConfigError::ValidationError {
                field: "logger.file.format".to_string(),
                message: e.to_string(),
            })
    }
}

impl RotationSettings {
    /// Convert RotationSettings to RotationConfig
    pub fn into_rotation_config(self) -> Result<RotationConfig, ConfigError> {
        let strategy = self.parse_strategy()?;

        RotationConfig::new(strategy, self.max_size, self.max_files, self.compress).map_err(|e| {
            ConfigError::ValidationError {
                field: "logger.file.rotation".to_string(),
                message: e.to_string(),
            }
        })
    }

    /// Parse the strategy string into RotationStrategy enum
    fn parse_strategy(&self) -> Result<RotationStrategy, ConfigError> {
        match self.strategy.to_lowercase().as_str() {
            "size" => Ok(RotationStrategy::Size),
            "count" => Ok(RotationStrategy::Count),
            "combined" => Ok(RotationStrategy::Combined),
            // Time-based strategies with time unit suffix
            "time" | "time_daily" | "daily" => {
                Ok(RotationStrategy::Time(crate::logger::TimeUnit::Daily))
            }
            "time_hourly" | "hourly" => Ok(RotationStrategy::Time(crate::logger::TimeUnit::Hourly)),
            "time_weekly" | "weekly" => Ok(RotationStrategy::Time(crate::logger::TimeUnit::Weekly)),
            "time_monthly" | "monthly" => {
                Ok(RotationStrategy::Time(crate::logger::TimeUnit::Monthly))
            }
            _ => Err(ConfigError::ValidationError {
                field: "logger.file.rotation.strategy".to_string(),
                message: format!(
                    "Invalid rotation strategy '{}'. Valid strategies are: size, time, daily, hourly, weekly, monthly, count, combined",
                    self.strategy
                ),
            }),
        }
    }
}

// ============================================================================
// Jobs Configuration
// ============================================================================

fn default_job_timeout() -> u64 {
    300
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    60
}

fn default_retry_backoff() -> f64 {
    2.0
}

fn default_history_retention_days() -> u32 {
    30
}

fn default_cache_ttl() -> u64 {
    300
}

fn default_cache_max_size() -> usize {
    1000
}

fn default_cache_directory() -> String {
    "cache".to_string()
}

fn default_redis_url() -> String {
    "redis://127.0.0.1:6379".to_string()
}

fn default_redis_pool_size() -> u32 {
    4
}

fn default_redis_connection_timeout() -> u64 {
    5
}

fn default_redis_key_prefix() -> String {
    "fusion".to_string()
}

/// Job scheduling configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobsConfig {
    /// Whether job scheduling is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Job execution timeout in seconds
    #[serde(default = "default_job_timeout")]
    pub job_timeout: u64,

    /// Maximum number of retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Initial retry delay in seconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,

    /// Retry backoff multiplier
    #[serde(default = "default_retry_backoff")]
    pub retry_backoff_multiplier: f64,

    /// Execution history retention in days
    #[serde(default = "default_history_retention_days")]
    pub history_retention_days: u32,
}

impl Default for JobsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            job_timeout: default_job_timeout(),
            max_retries: default_max_retries(),
            retry_delay: default_retry_delay(),
            retry_backoff_multiplier: default_retry_backoff(),
            history_retention_days: default_history_retention_days(),
        }
    }
}

// ============================================================================
// Cache Configuration
// ============================================================================

/// Cache backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CacheBackend {
    #[default]
    Memory,
    Disk,
    Redis,
}

/// Memory cache configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryCacheConfig {
    /// Maximum number of entries in the cache
    #[serde(default = "default_cache_max_size")]
    pub max_size: usize,

    /// Time-to-live in seconds
    #[serde(default = "default_cache_ttl")]
    pub ttl_seconds: u64,
}

impl Default for MemoryCacheConfig {
    fn default() -> Self {
        Self {
            max_size: default_cache_max_size(),
            ttl_seconds: default_cache_ttl(),
        }
    }
}

/// Disk cache configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskCacheConfig {
    /// Directory to store cache files
    #[serde(default = "default_cache_directory")]
    pub directory: String,

    /// Time-to-live in seconds
    #[serde(default = "default_cache_ttl")]
    pub ttl_seconds: u64,
}

impl Default for DiskCacheConfig {
    fn default() -> Self {
        Self {
            directory: default_cache_directory(),
            ttl_seconds: default_cache_ttl(),
        }
    }
}

/// Redis cache configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedisCacheConfig {
    /// Redis connection URL
    #[serde(default = "default_redis_url")]
    pub url: String,

    /// Time-to-live in seconds
    #[serde(default = "default_cache_ttl")]
    pub ttl_seconds: u64,

    /// Connection pool size
    #[serde(default = "default_redis_pool_size")]
    pub pool_size: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_redis_connection_timeout")]
    pub connection_timeout: u64,

    /// Key prefix for all cache entries
    #[serde(default = "default_redis_key_prefix")]
    pub key_prefix: String,

    /// Whether to use TLS
    #[serde(default)]
    pub tls_enabled: bool,
}

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self {
            url: default_redis_url(),
            ttl_seconds: default_cache_ttl(),
            pool_size: default_redis_pool_size(),
            connection_timeout: default_redis_connection_timeout(),
            key_prefix: default_redis_key_prefix(),
            tls_enabled: false,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CacheConfig {
    /// Whether caching is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Cache backend type
    #[serde(default)]
    pub backend: CacheBackend,

    /// Memory cache settings
    #[serde(default)]
    pub memory: MemoryCacheConfig,

    /// Disk cache settings
    #[serde(default)]
    pub disk: DiskCacheConfig,

    /// Redis cache settings
    #[serde(default)]
    pub redis: RedisCacheConfig,
}

// ============================================================================
// Main Settings Structure
// ============================================================================

/// Complete application settings
///
/// This structure represents the entire configuration that can be loaded
/// from TOML files and environment variables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Settings {
    /// Application information
    #[serde(default)]
    pub application: ApplicationConfig,

    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// JWT authentication configuration
    #[serde(default)]
    pub jwt: JwtConfig,

    /// Logger configuration
    #[serde(default)]
    pub logger: LoggerSettings,

    /// Job scheduling configuration
    #[serde(default)]
    pub jobs: JobsConfig,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ========================================================================
    // Arbitrary implementations for property-based testing
    // ========================================================================

    fn arb_application_config() -> impl Strategy<Value = ApplicationConfig> {
        (
            "[a-z][a-z0-9-]{0,20}",                 // name: valid app name
            "[0-9]{1,2}\\.[0-9]{1,2}\\.[0-9]{1,2}", // version: semver-like
        )
            .prop_map(|(name, version)| ApplicationConfig { name, version })
    }

    fn arb_server_config() -> impl Strategy<Value = ServerConfig> {
        (
            prop_oneof![
                Just("127.0.0.1".to_string()),
                Just("0.0.0.0".to_string()),
                Just("localhost".to_string()),
            ],
            1u16..=65535u16, // valid port range
            1u64..=300u64,   // request_timeout
            1u64..=300u64,   // keep_alive_timeout
        )
            .prop_map(
                |(host, port, request_timeout, keep_alive_timeout)| ServerConfig {
                    host,
                    port,
                    request_timeout,
                    keep_alive_timeout,
                },
            )
    }

    fn arb_database_config() -> impl Strategy<Value = DatabaseConfig> {
        (
            prop_oneof![
                Just("postgres://localhost/test".to_string()),
                Just("postgres://user:pass@host:5432/db".to_string()),
                Just("sqlite://./test.db".to_string()),
            ],
            1u32..=100u32, // max_connections
            1u32..=10u32,  // min_connections
            1u64..=120u64, // connection_timeout
        )
            .prop_map(
                |(url, max_connections, min_connections, connection_timeout)| {
                    // Ensure min <= max
                    let min = min_connections.min(max_connections);
                    DatabaseConfig {
                        url,
                        max_connections,
                        min_connections: min,
                        connection_timeout,
                        auto_migrate: false,
                    }
                },
            )
    }

    fn arb_jwt_config() -> impl Strategy<Value = JwtConfig> {
        (
            "[a-zA-Z0-9]{32,64}", // secret: 32-64 chars
            1i64..=24i64,         // access_token_expiration: 1-24 hours
            25i64..=720i64,       // refresh_token_expiration: 25-720 hours (ensure > access)
        )
            .prop_map(
                |(secret, access_token_expiration, refresh_token_expiration)| JwtConfig {
                    secret,
                    access_token_expiration,
                    refresh_token_expiration,
                },
            )
    }

    fn arb_console_settings() -> impl Strategy<Value = ConsoleSettings> {
        (any::<bool>(), any::<bool>())
            .prop_map(|(enabled, colored)| ConsoleSettings { enabled, colored })
    }

    fn arb_rotation_settings() -> impl Strategy<Value = RotationSettings> {
        (
            prop_oneof![
                Just("size".to_string()),
                Just("count".to_string()),
                Just("combined".to_string()),
                Just("daily".to_string()),
                Just("hourly".to_string()),
                Just("weekly".to_string()),
                Just("monthly".to_string()),
            ],
            1024u64..=100_000_000u64, // max_size
            1usize..=20usize,         // max_files
            any::<bool>(),            // compress
        )
            .prop_map(
                |(strategy, max_size, max_files, compress)| RotationSettings {
                    strategy,
                    max_size,
                    max_files,
                    compress,
                },
            )
    }

    fn arb_file_settings() -> impl Strategy<Value = FileSettings> {
        (
            any::<bool>(), // enabled
            prop_oneof![
                Just("logs/app.log".to_string()),
                Just("logs/test.log".to_string()),
                Just("/var/log/app.log".to_string()),
            ],
            any::<bool>(), // append
            prop_oneof![
                Just("json".to_string()),
                Just("full".to_string()),
                Just("compact".to_string()),
            ],
            arb_rotation_settings(),
        )
            .prop_map(|(enabled, path, append, format, rotation)| FileSettings {
                enabled,
                path,
                append,
                format,
                rotation,
            })
    }

    fn arb_logger_settings() -> impl Strategy<Value = LoggerSettings> {
        (
            prop_oneof![
                Just("trace".to_string()),
                Just("debug".to_string()),
                Just("info".to_string()),
                Just("warn".to_string()),
                Just("error".to_string()),
            ],
            arb_console_settings(),
            arb_file_settings(),
        )
            .prop_map(|(level, console, file)| LoggerSettings {
                level,
                console,
                file,
            })
    }

    fn arb_jobs_config() -> impl Strategy<Value = JobsConfig> {
        (
            any::<bool>(),   // enabled
            60u64..=600u64,  // job_timeout
            0u32..=5u32,     // max_retries
            10u64..=300u64,  // retry_delay
            1.0f64..=3.0f64, // retry_backoff_multiplier
            1u32..=90u32,    // history_retention_days
        )
            .prop_map(
                |(
                    enabled,
                    job_timeout,
                    max_retries,
                    retry_delay,
                    retry_backoff_multiplier,
                    history_retention_days,
                )| {
                    JobsConfig {
                        enabled,
                        job_timeout,
                        max_retries,
                        retry_delay,
                        retry_backoff_multiplier,
                        history_retention_days,
                    }
                },
            )
    }

    fn arb_cache_config() -> impl Strategy<Value = CacheConfig> {
        Just(CacheConfig::default())
    }

    fn arb_settings() -> impl Strategy<Value = Settings> {
        (
            arb_application_config(),
            arb_server_config(),
            arb_database_config(),
            arb_jwt_config(),
            arb_logger_settings(),
            arb_jobs_config(),
            arb_cache_config(),
        )
            .prop_map(
                |(application, server, database, jwt, logger, jobs, cache)| Settings {
                    application,
                    server,
                    database,
                    jwt,
                    logger,
                    jobs,
                    cache,
                },
            )
    }

    // ========================================================================
    // Property-based tests
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: config-management, Property 7: Settings Round-Trip Serialization
        /// *For any* valid Settings instance, serializing to TOML and then deserializing
        /// back SHALL produce an equivalent Settings instance.
        /// **Validates: Requirements 10.4**
        #[test]
        fn prop_settings_round_trip_serialization(settings in arb_settings()) {
            // Serialize to TOML
            let toml_str = toml::to_string(&settings)
                .expect("Settings should serialize to TOML");

            // Deserialize back
            let deserialized: Settings = toml::from_str(&toml_str)
                .expect("TOML should deserialize back to Settings");

            // Verify equivalence
            prop_assert_eq!(settings, deserialized);
        }
    }

    // ========================================================================
    // Unit tests
    // ========================================================================

    #[test]
    fn test_application_config_defaults() {
        let config = ApplicationConfig::default();
        assert_eq!(config.name, "fusion-rs");
        assert_eq!(config.version, crate::pkg_version());
    }

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert_eq!(config.request_timeout, 30);
        assert_eq!(config.keep_alive_timeout, 75);
    }

    #[test]
    fn test_server_config_address() {
        let config = ServerConfig::default();
        assert_eq!(config.address(), "127.0.0.1:3000");
    }

    #[test]
    fn test_database_config_defaults() {
        let config = DatabaseConfig::default();
        assert_eq!(config.url, "");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
        assert_eq!(config.connection_timeout, 30);
    }

    #[test]
    fn test_jwt_config_defaults() {
        let config = JwtConfig::default();
        assert_eq!(config.secret, "");
        assert_eq!(config.access_token_expiration, 1);
        assert_eq!(config.refresh_token_expiration, 168);
    }

    #[test]
    fn test_jwt_config_validate_empty_secret() {
        let config = JwtConfig {
            secret: "".to_string(),
            access_token_expiration: 1,
            refresh_token_expiration: 168,
        };
        let result = config.validate();
        assert!(result.is_err());
        if let Err(ConfigError::ValidationError { field, message }) = result {
            assert_eq!(field, "jwt.secret");
            assert!(message.contains("cannot be empty"));
        }
    }

    #[test]
    fn test_jwt_config_validate_short_secret() {
        let config = JwtConfig {
            secret: "short".to_string(),
            access_token_expiration: 1,
            refresh_token_expiration: 168,
        };
        let result = config.validate();
        assert!(result.is_err());
        if let Err(ConfigError::ValidationError { field, message }) = result {
            assert_eq!(field, "jwt.secret");
            assert!(message.contains("at least 32 characters"));
        }
    }

    #[test]
    fn test_jwt_config_validate_negative_access_expiration() {
        let config = JwtConfig {
            secret: "a".repeat(32),
            access_token_expiration: -1,
            refresh_token_expiration: 168,
        };
        let result = config.validate();
        assert!(result.is_err());
        if let Err(ConfigError::ValidationError { field, .. }) = result {
            assert_eq!(field, "jwt.access_token_expiration");
        }
    }

    #[test]
    fn test_jwt_config_validate_negative_refresh_expiration() {
        let config = JwtConfig {
            secret: "a".repeat(32),
            access_token_expiration: 1,
            refresh_token_expiration: -1,
        };
        let result = config.validate();
        assert!(result.is_err());
        if let Err(ConfigError::ValidationError { field, .. }) = result {
            assert_eq!(field, "jwt.refresh_token_expiration");
        }
    }

    #[test]
    fn test_jwt_config_validate_access_longer_than_refresh() {
        let config = JwtConfig {
            secret: "a".repeat(32),
            access_token_expiration: 100,
            refresh_token_expiration: 50,
        };
        let result = config.validate();
        assert!(result.is_err());
        if let Err(ConfigError::ValidationError { field, message }) = result {
            assert_eq!(field, "jwt");
            assert!(message.contains("Refresh token expiration should be longer"));
        }
    }

    #[test]
    fn test_jwt_config_validate_success() {
        let config = JwtConfig {
            secret: "a".repeat(32),
            access_token_expiration: 1,
            refresh_token_expiration: 168,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_console_settings_defaults() {
        let settings = ConsoleSettings::default();
        assert!(settings.enabled);
        assert!(settings.colored);
    }

    #[test]
    fn test_rotation_settings_defaults() {
        let settings = RotationSettings::default();
        assert_eq!(settings.strategy, "size");
        assert_eq!(settings.max_size, 10 * 1024 * 1024);
        assert_eq!(settings.max_files, 5);
        assert!(!settings.compress);
    }

    #[test]
    fn test_file_settings_defaults() {
        let settings = FileSettings::default();
        assert!(!settings.enabled);
        assert_eq!(settings.path, "logs/app.log");
        assert!(settings.append);
        assert_eq!(settings.format, "json");
    }

    #[test]
    fn test_logger_settings_defaults() {
        let settings = LoggerSettings::default();
        assert_eq!(settings.level, "info");
        assert!(settings.console.enabled);
        assert!(!settings.file.enabled);
    }

    #[test]
    fn test_jobs_config_defaults() {
        let config = JobsConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.job_timeout, 300);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay, 60);
        assert_eq!(config.retry_backoff_multiplier, 2.0);
        assert_eq!(config.history_retention_days, 30);
    }

    #[test]
    fn test_settings_defaults() {
        let settings = Settings::default();
        assert_eq!(settings.application.name, "fusion-rs");
        assert_eq!(settings.server.port, 3000);
        assert_eq!(settings.database.max_connections, 10);
        assert_eq!(settings.jwt.access_token_expiration, 1);
        assert_eq!(settings.jwt.refresh_token_expiration, 168);
        assert_eq!(settings.logger.level, "info");
        assert!(!settings.jobs.enabled);
        assert_eq!(settings.jobs.job_timeout, 300);
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let settings = Settings::default();
        let toml_str = toml::to_string(&settings).expect("Failed to serialize");
        let deserialized: Settings = toml::from_str(&toml_str).expect("Failed to deserialize");
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_settings_deserialize_partial() {
        let toml_str = r#"
            [application]
            name = "my-app"
            
            [server]
            port = 8080
        "#;

        let settings: Settings = toml::from_str(toml_str).expect("Failed to deserialize");
        assert_eq!(settings.application.name, "my-app");
        assert_eq!(settings.application.version, "0.1.0"); // default
        assert_eq!(settings.server.port, 8080);
        assert_eq!(settings.server.host, "127.0.0.1"); // default
    }

    #[test]
    fn test_settings_deserialize_full() {
        let toml_str = r#"
            [application]
            name = "test-app"
            version = "1.0.0"
            
            [server]
            host = "0.0.0.0"
            port = 8080
            request_timeout = 60
            keep_alive_timeout = 120
            
            [database]
            url = "postgres://localhost/test"
            max_connections = 20
            min_connections = 5
            connection_timeout = 60
            
            [logger]
            level = "debug"
            
            [logger.console]
            enabled = true
            colored = false
            
            [logger.file]
            enabled = true
            path = "logs/test.log"
            append = false
            format = "compact"
            
            [logger.file.rotation]
            strategy = "time"
            max_size = 5242880
            max_files = 10
            compress = true
        "#;

        let settings: Settings = toml::from_str(toml_str).expect("Failed to deserialize");

        assert_eq!(settings.application.name, "test-app");
        assert_eq!(settings.application.version, "1.0.0");

        assert_eq!(settings.server.host, "0.0.0.0");
        assert_eq!(settings.server.port, 8080);
        assert_eq!(settings.server.request_timeout, 60);
        assert_eq!(settings.server.keep_alive_timeout, 120);

        assert_eq!(settings.database.url, "postgres://localhost/test");
        assert_eq!(settings.database.max_connections, 20);
        assert_eq!(settings.database.min_connections, 5);
        assert_eq!(settings.database.connection_timeout, 60);

        assert_eq!(settings.logger.level, "debug");
        assert!(settings.logger.console.enabled);
        assert!(!settings.logger.console.colored);
        assert!(settings.logger.file.enabled);
        assert_eq!(settings.logger.file.path, "logs/test.log");
        assert!(!settings.logger.file.append);
        assert_eq!(settings.logger.file.format, "compact");
        assert_eq!(settings.logger.file.rotation.strategy, "time");
        assert_eq!(settings.logger.file.rotation.max_size, 5242880);
        assert_eq!(settings.logger.file.rotation.max_files, 10);
        assert!(settings.logger.file.rotation.compress);
    }

    // ========================================================================
    // LoggerSettings to LoggerConfig conversion tests
    // ========================================================================

    #[test]
    fn test_console_settings_into_console_config() {
        let settings = ConsoleSettings {
            enabled: true,
            colored: false,
        };
        let config = settings.into_console_config();
        assert!(config.enabled);
        assert!(!config.colored);
    }

    #[test]
    fn test_rotation_settings_into_rotation_config_size() {
        let settings = RotationSettings {
            strategy: "size".to_string(),
            max_size: 1024 * 1024,
            max_files: 3,
            compress: true,
        };
        let config = settings.into_rotation_config().expect("Should convert");
        assert_eq!(config.strategy, RotationStrategy::Size);
        assert_eq!(config.max_size, 1024 * 1024);
        assert_eq!(config.max_files, 3);
        assert!(config.compress);
    }

    #[test]
    fn test_rotation_settings_into_rotation_config_time_variants() {
        // Test "time" defaults to daily
        let settings = RotationSettings {
            strategy: "time".to_string(),
            ..Default::default()
        };
        let config = settings.into_rotation_config().expect("Should convert");
        assert_eq!(
            config.strategy,
            RotationStrategy::Time(crate::logger::TimeUnit::Daily)
        );

        // Test "hourly"
        let settings = RotationSettings {
            strategy: "hourly".to_string(),
            ..Default::default()
        };
        let config = settings.into_rotation_config().expect("Should convert");
        assert_eq!(
            config.strategy,
            RotationStrategy::Time(crate::logger::TimeUnit::Hourly)
        );

        // Test "weekly"
        let settings = RotationSettings {
            strategy: "weekly".to_string(),
            ..Default::default()
        };
        let config = settings.into_rotation_config().expect("Should convert");
        assert_eq!(
            config.strategy,
            RotationStrategy::Time(crate::logger::TimeUnit::Weekly)
        );

        // Test "monthly"
        let settings = RotationSettings {
            strategy: "monthly".to_string(),
            ..Default::default()
        };
        let config = settings.into_rotation_config().expect("Should convert");
        assert_eq!(
            config.strategy,
            RotationStrategy::Time(crate::logger::TimeUnit::Monthly)
        );
    }

    #[test]
    fn test_rotation_settings_into_rotation_config_invalid_strategy() {
        let settings = RotationSettings {
            strategy: "invalid".to_string(),
            ..Default::default()
        };
        let result = settings.into_rotation_config();
        assert!(result.is_err());
        if let Err(ConfigError::ValidationError { field, message }) = result {
            assert_eq!(field, "logger.file.rotation.strategy");
            assert!(message.contains("Invalid rotation strategy"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_file_settings_into_file_config() {
        let settings = FileSettings {
            enabled: true,
            path: "logs/test.log".to_string(),
            append: false,
            format: "json".to_string(),
            rotation: RotationSettings::default(),
        };
        let config = settings.into_file_config().expect("Should convert");
        assert!(config.enabled);
        assert_eq!(config.path, PathBuf::from("logs/test.log"));
        assert!(!config.append);
        assert_eq!(config.format, LogFormat::Json);
    }

    #[test]
    fn test_file_settings_into_file_config_all_formats() {
        for (format_str, expected) in [
            ("full", LogFormat::Full),
            ("compact", LogFormat::Compact),
            ("json", LogFormat::Json),
            ("FULL", LogFormat::Full),       // case insensitive
            ("Compact", LogFormat::Compact), // case insensitive
        ] {
            let settings = FileSettings {
                format: format_str.to_string(),
                ..Default::default()
            };
            let config = settings.into_file_config().expect("Should convert");
            assert_eq!(
                config.format, expected,
                "Format {} should convert",
                format_str
            );
        }
    }

    #[test]
    fn test_file_settings_into_file_config_invalid_format() {
        let settings = FileSettings {
            format: "invalid".to_string(),
            ..Default::default()
        };
        let result = settings.into_file_config();
        assert!(result.is_err());
        if let Err(ConfigError::ValidationError { field, .. }) = result {
            assert_eq!(field, "logger.file.format");
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_logger_settings_into_logger_config() {
        let settings = LoggerSettings {
            level: "debug".to_string(),
            console: ConsoleSettings {
                enabled: true,
                colored: true,
            },
            file: FileSettings {
                enabled: false,
                ..Default::default()
            },
        };
        let config = settings.into_logger_config().expect("Should convert");
        assert_eq!(config.level, "debug");
        assert!(config.console.enabled);
        assert!(config.console.colored);
        assert!(!config.file.enabled);
    }

    #[test]
    fn test_logger_settings_into_logger_config_with_file() {
        let settings = LoggerSettings {
            level: "info".to_string(),
            console: ConsoleSettings {
                enabled: true,
                colored: false,
            },
            file: FileSettings {
                enabled: true,
                path: "logs/app.log".to_string(),
                append: true,
                format: "compact".to_string(),
                rotation: RotationSettings {
                    strategy: "size".to_string(),
                    max_size: 5 * 1024 * 1024,
                    max_files: 10,
                    compress: true,
                },
            },
        };
        let config = settings.into_logger_config().expect("Should convert");
        assert_eq!(config.level, "info");
        assert!(config.console.enabled);
        assert!(!config.console.colored);
        assert!(config.file.enabled);
        assert_eq!(config.file.path, PathBuf::from("logs/app.log"));
        assert!(config.file.append);
        assert_eq!(config.file.format, LogFormat::Compact);
        assert_eq!(config.file.rotation.strategy, RotationStrategy::Size);
        assert_eq!(config.file.rotation.max_size, 5 * 1024 * 1024);
        assert_eq!(config.file.rotation.max_files, 10);
        assert!(config.file.rotation.compress);
    }

    #[test]
    fn test_logger_settings_into_logger_config_invalid_level() {
        let settings = LoggerSettings {
            level: "invalid".to_string(),
            console: ConsoleSettings::default(),
            file: FileSettings::default(),
        };
        let result = settings.into_logger_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_logger_settings_into_logger_config_both_disabled() {
        let settings = LoggerSettings {
            level: "info".to_string(),
            console: ConsoleSettings {
                enabled: false,
                colored: false,
            },
            file: FileSettings {
                enabled: false,
                ..Default::default()
            },
        };
        let result = settings.into_logger_config();
        assert!(result.is_err());
    }
}
