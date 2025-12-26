//! Configuration error types

use thiserror::Error;

/// Configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    /// Failed to parse configuration
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    /// Validation error with field and message
    #[error("Validation error: {field} - {message}")]
    ValidationError {
        /// The field that failed validation
        field: String,
        /// The validation error message
        message: String,
    },

    /// Environment variable error
    #[error("Environment variable error: {0}")]
    EnvVarError(String),

    /// Mutual exclusivity error
    #[error("Mutual exclusivity error: {0}")]
    MutualExclusivityError(String),

    /// Generic configuration error from config crate
    #[error("Configuration error: {0}")]
    Other(#[from] config::ConfigError),
}

impl ConfigError {
    /// Create a new validation error
    pub fn validation<S: Into<String>>(field: S, message: S) -> Self {
        ConfigError::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a new file not found error
    pub fn file_not_found<S: Into<String>>(path: S) -> Self {
        ConfigError::FileNotFound(path.into())
    }

    /// Create a new mutual exclusivity error
    pub fn mutual_exclusivity<S: Into<String>>(message: S) -> Self {
        ConfigError::MutualExclusivityError(message.into())
    }
}
