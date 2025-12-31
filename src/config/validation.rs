//! Configuration validation logic
//!
//! This module provides validation methods for all configuration structures
//! to ensure configuration values are within acceptable ranges and formats.

use crate::config::error::ConfigError;
use crate::config::settings::{
    DatabaseConfig, FileSettings, LoggerSettings, ServerConfig, Settings,
};

/// Valid log levels
const VALID_LOG_LEVELS: &[&str] = &["trace", "debug", "info", "warn", "error"];

/// Valid log formats
const VALID_LOG_FORMATS: &[&str] = &["full", "compact", "json"];

/// Valid rotation strategies
const VALID_ROTATION_STRATEGIES: &[&str] = &["size", "time", "count", "combined"];

impl ServerConfig {
    /// Validate server configuration
    ///
    /// # Validation Rules
    /// - Port must be between 1 and 65535
    /// - Request timeout must be greater than 0
    /// - Keep-alive timeout must be greater than 0
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate port range (1-65535)
        if self.port == 0 {
            return Err(ConfigError::validation(
                "server.port",
                "Port must be between 1 and 65535. Please specify a valid port number.",
            ));
        }

        // Validate request timeout
        if self.request_timeout == 0 {
            return Err(ConfigError::validation(
                "server.request_timeout",
                "Request timeout must be greater than 0 seconds.",
            ));
        }

        // Validate keep-alive timeout
        if self.keep_alive_timeout == 0 {
            return Err(ConfigError::validation(
                "server.keep_alive_timeout",
                "Keep-alive timeout must be greater than 0 seconds.",
            ));
        }

        Ok(())
    }
}

impl DatabaseConfig {
    /// Validate database configuration
    ///
    /// # Validation Rules
    /// - URL must not be empty
    /// - URL must have a valid database URL format
    /// - Max connections must be greater than 0
    /// - Min connections must be greater than 0
    /// - Min connections must not exceed max connections
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate URL is not empty
        if self.url.is_empty() {
            return Err(ConfigError::validation(
                "database.url",
                "Database URL is required. Please specify a valid database connection string.",
            ));
        }

        // Validate URL format (basic check for common database URL schemes)
        if !self.is_valid_database_url() {
            return Err(ConfigError::validation(
                "database.url",
                "Invalid database URL format. Expected format: scheme://[user:password@]host[:port]/database",
            ));
        }

        // Validate max connections
        if self.max_connections == 0 {
            return Err(ConfigError::validation(
                "database.max_connections",
                "Max connections must be greater than 0.",
            ));
        }

        // Validate min connections
        if self.min_connections == 0 {
            return Err(ConfigError::validation(
                "database.min_connections",
                "Min connections must be greater than 0.",
            ));
        }

        // Validate min <= max connections
        if self.min_connections > self.max_connections {
            return Err(ConfigError::ValidationError {
                field: "database.min_connections".to_string(),
                message: format!(
                    "Min connections ({}) cannot exceed max connections ({}).",
                    self.min_connections, self.max_connections
                ),
            });
        }

        Ok(())
    }

    /// Check if the database URL has a valid format
    fn is_valid_database_url(&self) -> bool {
        // Check for common database URL schemes
        let valid_schemes = [
            "postgres://",
            "postgresql://",
            "mysql://",
            "sqlite://",
            "sqlite:",
        ];

        valid_schemes
            .iter()
            .any(|scheme| self.url.starts_with(scheme))
    }
}

impl FileSettings {
    /// Validate file settings
    fn validate(&self) -> Result<(), ConfigError> {
        // If file logging is enabled, path must not be empty
        if self.enabled && self.path.trim().is_empty() {
            return Err(ConfigError::validation(
                "logger.file.path",
                "File path is required when file logging is enabled.",
            ));
        }

        // Validate log format
        if !VALID_LOG_FORMATS.contains(&self.format.to_lowercase().as_str()) {
            return Err(ConfigError::ValidationError {
                field: "logger.file.format".to_string(),
                message: format!(
                    "Invalid log format '{}'. Valid formats are: {}",
                    self.format,
                    VALID_LOG_FORMATS.join(", ")
                ),
            });
        }

        // Validate rotation strategy
        if !VALID_ROTATION_STRATEGIES.contains(&self.rotation.strategy.to_lowercase().as_str()) {
            return Err(ConfigError::ValidationError {
                field: "logger.file.rotation.strategy".to_string(),
                message: format!(
                    "Invalid rotation strategy '{}'. Valid strategies are: {}",
                    self.rotation.strategy,
                    VALID_ROTATION_STRATEGIES.join(", ")
                ),
            });
        }

        Ok(())
    }
}

impl LoggerSettings {
    /// Validate logger settings
    ///
    /// # Validation Rules
    /// - Log level must be one of: trace, debug, info, warn, error
    /// - If file logging is enabled, path must not be empty
    /// - Log format must be one of: full, compact, json
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate log level
        if !VALID_LOG_LEVELS.contains(&self.level.to_lowercase().as_str()) {
            return Err(ConfigError::ValidationError {
                field: "logger.level".to_string(),
                message: format!(
                    "Invalid log level '{}'. Valid levels are: {}",
                    self.level,
                    VALID_LOG_LEVELS.join(", ")
                ),
            });
        }

        // Validate file settings
        self.file.validate()?;

        Ok(())
    }
}

impl Settings {
    /// Validate all configuration settings
    ///
    /// This method validates all sub-configurations and returns the first
    /// validation error encountered.
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.server.validate()?;
        self.database.validate()?;
        self.logger.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::RotationSettings;

    // ========================================================================
    // ServerConfig validation tests
    // ========================================================================

    #[test]
    fn test_server_config_valid() {
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_server_config_invalid_port_zero() {
        let config = ServerConfig {
            port: 0,
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "server.port")
        );
    }

    #[test]
    fn test_server_config_valid_port_boundaries() {
        // Port 1 should be valid
        let config = ServerConfig {
            port: 1,
            ..Default::default()
        };
        assert!(config.validate().is_ok());

        // Port 65535 should be valid
        let config = ServerConfig {
            port: 65535,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_server_config_invalid_request_timeout() {
        let config = ServerConfig {
            request_timeout: 0,
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "server.request_timeout")
        );
    }

    #[test]
    fn test_server_config_invalid_keep_alive_timeout() {
        let config = ServerConfig {
            keep_alive_timeout: 0,
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "server.keep_alive_timeout")
        );
    }

    // ========================================================================
    // DatabaseConfig validation tests
    // ========================================================================

    #[test]
    fn test_database_config_valid() {
        let config = DatabaseConfig {
            url: "postgres://localhost/test".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_database_config_empty_url() {
        let config = DatabaseConfig::default();
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "database.url")
        );
    }

    #[test]
    fn test_database_config_invalid_url_format() {
        let config = DatabaseConfig {
            url: "invalid-url".to_string(),
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "database.url")
        );
    }

    #[test]
    fn test_database_config_valid_url_schemes() {
        let valid_urls = [
            "postgres://localhost/db",
            "postgresql://user:pass@localhost:5432/db",
            "mysql://localhost/db",
            "sqlite://path/to/db.sqlite",
            "sqlite:memory:",
        ];

        for url in valid_urls {
            let config = DatabaseConfig {
                url: url.to_string(),
                ..Default::default()
            };
            assert!(config.validate().is_ok(), "URL should be valid: {}", url);
        }
    }

    #[test]
    fn test_database_config_invalid_max_connections() {
        let config = DatabaseConfig {
            url: "postgres://localhost/test".to_string(),
            max_connections: 0,
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "database.max_connections")
        );
    }

    #[test]
    fn test_database_config_invalid_min_connections() {
        let config = DatabaseConfig {
            url: "postgres://localhost/test".to_string(),
            min_connections: 0,
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "database.min_connections")
        );
    }

    #[test]
    fn test_database_config_min_exceeds_max() {
        let config = DatabaseConfig {
            url: "postgres://localhost/test".to_string(),
            max_connections: 5,
            min_connections: 10,
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "database.min_connections")
        );
    }

    // ========================================================================
    // LoggerSettings validation tests
    // ========================================================================

    #[test]
    fn test_logger_settings_valid() {
        let settings = LoggerSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_logger_settings_valid_levels() {
        let valid_levels = ["trace", "debug", "info", "warn", "error", "INFO", "Debug"];

        for level in valid_levels {
            let settings = LoggerSettings {
                level: level.to_string(),
                ..Default::default()
            };
            assert!(
                settings.validate().is_ok(),
                "Level should be valid: {}",
                level
            );
        }
    }

    #[test]
    fn test_logger_settings_invalid_level() {
        let settings = LoggerSettings {
            level: "invalid".to_string(),
            ..Default::default()
        };
        let err = settings.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "logger.level")
        );
    }

    #[test]
    fn test_logger_settings_file_enabled_empty_path() {
        let settings = LoggerSettings {
            file: FileSettings {
                enabled: true,
                path: "".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let err = settings.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "logger.file.path")
        );
    }

    #[test]
    fn test_logger_settings_file_disabled_empty_path_ok() {
        let settings = LoggerSettings {
            file: FileSettings {
                enabled: false,
                path: "".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_logger_settings_invalid_format() {
        let settings = LoggerSettings {
            file: FileSettings {
                format: "invalid".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let err = settings.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "logger.file.format")
        );
    }

    #[test]
    fn test_logger_settings_valid_formats() {
        let valid_formats = ["full", "compact", "json", "FULL", "Compact"];

        for format in valid_formats {
            let settings = LoggerSettings {
                file: FileSettings {
                    format: format.to_string(),
                    ..Default::default()
                },
                ..Default::default()
            };
            assert!(
                settings.validate().is_ok(),
                "Format should be valid: {}",
                format
            );
        }
    }

    #[test]
    fn test_logger_settings_invalid_rotation_strategy() {
        let settings = LoggerSettings {
            file: FileSettings {
                rotation: RotationSettings {
                    strategy: "invalid".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let err = settings.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "logger.file.rotation.strategy")
        );
    }

    // ========================================================================
    // Settings validation tests
    // ========================================================================

    #[test]
    fn test_settings_valid() {
        let settings = Settings {
            database: DatabaseConfig {
                url: "postgres://localhost/test".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_settings_invalid_server() {
        let settings = Settings {
            server: ServerConfig {
                port: 0,
                ..Default::default()
            },
            database: DatabaseConfig {
                url: "postgres://localhost/test".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let err = settings.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "server.port")
        );
    }

    #[test]
    fn test_settings_invalid_database() {
        let settings = Settings::default();
        let err = settings.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "database.url")
        );
    }

    #[test]
    fn test_settings_invalid_logger() {
        let settings = Settings {
            database: DatabaseConfig {
                url: "postgres://localhost/test".to_string(),
                ..Default::default()
            },
            logger: LoggerSettings {
                level: "invalid".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let err = settings.validate().unwrap_err();
        assert!(
            matches!(err, ConfigError::ValidationError { field, .. } if field == "logger.level")
        );
    }
}
