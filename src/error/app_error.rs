use crate::error::DatabaseErrorConverter;
use thiserror::Error;

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

/// Type alias for Result with AppError to simplify function signatures
pub type AppResult<T> = Result<T, AppError>;
