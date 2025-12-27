use crate::error::DatabaseErrorConverter;
use axum::extract::rejection::{FormRejection, JsonRejection, QueryRejection};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represents a validation error for a specific field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFieldError {
    pub field: String,
    pub message: String,
}

/// Application-wide error type that represents all possible errors in the system.
///
/// This enum provides comprehensive error handling with structured information
/// for different error scenarios, supporting automatic conversion from anyhow
/// and detailed context for debugging and user feedback.
#[derive(Error, Debug)]
pub enum AppError {
    /// Resource not found error with entity, field, and value information
    #[error("Resource not found: {entity} with {field}={value}")]
    NotFound {
        entity: String,
        field: String,
        value: String,
    },

    /// Duplicate entry error for unique constraint violations
    #[error("Duplicate entry: {entity}.{field} = '{value}' already exists")]
    Duplicate {
        entity: String,
        field: String,
        value: String,
    },

    /// Validation error with field-specific details
    #[error("Validation failed for {field}: {reason}")]
    Validation { field: String, reason: String },

    /// Multiple validation errors with field-specific details
    #[error("Validation failed for multiple fields")]
    ValidationErrors { errors: Vec<ValidationFieldError> },

    /// Bad request error with descriptive message
    #[error("Bad request: {message}")]
    BadRequest { message: String },

    /// Unprocessable content error with descriptive message
    #[error("Unprocessable content: {message}")]
    UnprocessableContent { message: String },

    /// Unauthorized access error with authentication message
    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },

    /// Forbidden access error with authorization message
    #[error("Forbidden: {message}")]
    Forbidden { message: String },

    /// Database operation error with operation context
    #[error("Database operation failed: {operation}")]
    Database {
        operation: String,
        #[source]
        source: anyhow::Error,
    },

    /// Configuration error with key information
    #[error("Configuration error: {key}")]
    Configuration {
        key: String,
        #[source]
        source: anyhow::Error,
    },

    /// Connection pool error
    #[error("Connection pool error")]
    ConnectionPool {
        #[source]
        source: anyhow::Error,
    },

    /// Internal error for unexpected failures
    #[error("Internal error")]
    Internal {
        #[source]
        source: anyhow::Error,
    },
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError::Internal { source: error }
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(error: diesel::result::Error) -> Self {
        DatabaseErrorConverter::convert_diesel_error(error, "database operation")
    }
}

impl From<crate::config::error::ConfigError> for AppError {
    fn from(error: crate::config::error::ConfigError) -> Self {
        match error {
            crate::config::error::ConfigError::ValidationError { field, message } => {
                AppError::Validation {
                    field,
                    reason: message,
                }
            }
            crate::config::error::ConfigError::FileNotFound(path) => AppError::Configuration {
                key: "config_file".to_string(),
                source: anyhow::anyhow!("Configuration file not found: {}", path),
            },
            crate::config::error::ConfigError::ParseError(msg) => AppError::Configuration {
                key: "config_parse".to_string(),
                source: anyhow::anyhow!("Configuration parse error: {}", msg),
            },
            crate::config::error::ConfigError::EnvVarError(msg) => AppError::Configuration {
                key: "environment_variable".to_string(),
                source: anyhow::anyhow!("Environment variable error: {}", msg),
            },
            crate::config::error::ConfigError::MutualExclusivityError(msg) => {
                AppError::Configuration {
                    key: "mutual_exclusivity".to_string(),
                    source: anyhow::anyhow!("Mutual exclusivity error: {}", msg),
                }
            }
            crate::config::error::ConfigError::Other(config_err) => AppError::Configuration {
                key: "config_crate".to_string(),
                source: anyhow::anyhow!("Config crate error: {}", config_err),
            },
        }
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(error: argon2::password_hash::Error) -> Self {
        AppError::Internal {
            source: anyhow::anyhow!("Password hash error: {}", error),
        }
    }
}

impl From<argon2::password_hash::phc::Error> for AppError {
    fn from(error: argon2::password_hash::phc::Error) -> Self {
        AppError::Internal {
            source: anyhow::anyhow!("Password hash PHC error: {}", error),
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let field_errors: Vec<ValidationFieldError> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    let message = error
                        .message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("{} validation failed", field));
                    ValidationFieldError {
                        field: field.to_string(),
                        message,
                    }
                })
            })
            .collect();

        AppError::ValidationErrors {
            errors: field_errors,
        }
    }
}

impl From<FormRejection> for AppError {
    fn from(value: FormRejection) -> Self {
        AppError::BadRequest {
            message: value.to_string(),
        }
    }
}

impl From<JsonRejection> for AppError {
    fn from(value: JsonRejection) -> Self {
        AppError::BadRequest {
            message: value.to_string(),
        }
    }
}

impl From<QueryRejection> for AppError {
    fn from(value: QueryRejection) -> Self {
        AppError::BadRequest {
            message: value.to_string(),
        }
    }
}

/// Type alias for Result with AppError to simplify function signatures
pub type AppResult<T> = Result<T, AppError>;
