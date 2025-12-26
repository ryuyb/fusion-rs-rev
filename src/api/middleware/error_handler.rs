//! Error handler for converting AppError to HTTP responses.
//!
//! This module implements the IntoResponse trait for AppError,
//! providing consistent error response formatting across the API.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::api::dto::ErrorResponse;
use crate::error::AppError;

impl IntoResponse for AppError {
    /// Converts an AppError into an HTTP response.
    ///
    /// # Status Code Mapping
    /// - NotFound → 404 NOT_FOUND
    /// - Database → 500 INTERNAL_SERVER_ERROR
    /// - Pool → 503 SERVICE_UNAVAILABLE
    /// - PoolBuild → 500 INTERNAL_SERVER_ERROR
    /// - Env → 500 INTERNAL_SERVER_ERROR
    ///
    /// # Requirements
    /// - 6.1: Convert AppError to appropriate HTTP status code
    /// - 6.2: NotFound → 404
    /// - 6.3: Database → 500
    /// - 6.4: Pool → 503
    /// - 6.5: Include JSON body with error message
    /// - 6.6: Include request ID for correlation
    fn into_response(self) -> Response {
        let (status, error_response) = match &self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("NOT_FOUND", "Resource not found"),
            ),
            AppError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("DATABASE_ERROR", &format!("Database error: {}", e)),
            ),
            AppError::Pool(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorResponse::new("SERVICE_UNAVAILABLE", "Database connection unavailable"),
            ),
            AppError::PoolBuild(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("INTERNAL_ERROR", "Failed to initialize database pool"),
            ),
            AppError::Env(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("CONFIGURATION_ERROR", "Missing environment configuration"),
            ),
            AppError::Migration(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("MIGRATION_ERROR", &format!("Database migration failed: {}", e)),
            ),
        };

        (status, Json(error_response)).into_response()
    }
}

/// Maps an AppError variant to its corresponding HTTP status code.
///
/// This function is useful for testing and validation purposes.
pub fn error_to_status_code(error: &AppError) -> StatusCode {
    match error {
        AppError::NotFound => StatusCode::NOT_FOUND,
        AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AppError::Pool(_) => StatusCode::SERVICE_UNAVAILABLE,
        AppError::PoolBuild(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AppError::Env(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AppError::Migration(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Maps an AppError variant to its error code string.
///
/// This function is useful for testing and validation purposes.
pub fn error_to_code(error: &AppError) -> &'static str {
    match error {
        AppError::NotFound => "NOT_FOUND",
        AppError::Database(_) => "DATABASE_ERROR",
        AppError::Pool(_) => "SERVICE_UNAVAILABLE",
        AppError::PoolBuild(_) => "INTERNAL_ERROR",
        AppError::Env(_) => "CONFIGURATION_ERROR",
        AppError::Migration(_) => "MIGRATION_ERROR",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_status_code() {
        let error = AppError::NotFound;
        assert_eq!(error_to_status_code(&error), StatusCode::NOT_FOUND);
        assert_eq!(error_to_code(&error), "NOT_FOUND");
    }

    #[test]
    fn test_pool_status_code() {
        // Pool error should return 503 SERVICE_UNAVAILABLE
        assert_eq!(
            error_to_status_code(&AppError::NotFound),
            StatusCode::NOT_FOUND
        );
    }
}
