//! Health check endpoint handlers.
//!
//! This module provides health check functionality for monitoring
//! and load balancer health checks. Health checks directly access
//! the database connection pool for efficient connectivity testing.

use crate::api::doc::HEALTH_TAG;
use crate::state::AppState;
use axum::{Router, extract::State, http::StatusCode, response::Json, routing::get};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Health check response structure.
///
/// Provides information about the application's health status
/// and various system components.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Overall health status
    pub status: HealthStatus,
    /// Application version
    pub version: String,
    /// Timestamp of the health check (ISO 8601 format)
    #[schema(value_type = String, format = DateTime)]
    pub timestamp: String,
    /// Detailed checks for various components
    pub checks: HashMap<String, ComponentHealth>,
}

/// Health status enumeration.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Some non-critical issues
    Degraded,
    /// Critical issues present
    Unhealthy,
}

/// Individual component health information.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ComponentHealth {
    /// Component status
    pub status: HealthStatus,
    /// Optional message with details
    pub message: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
}

/// Creates health check routes.
///
/// # Routes
/// - `GET /health` - Basic health check
/// - `GET /health/ready` - Readiness probe
/// - `GET /health/live` - Liveness probe
pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
}

/// Basic health check endpoint.
///
/// Returns comprehensive health information including database connectivity.
///
/// # Responses
/// - `200 OK` - Service is healthy
/// - `503 Service Unavailable` - Service is unhealthy
///
/// # Example Response
/// ```json
/// {
///   "status": "healthy",
///   "version": "0.1.0",
///   "timestamp": "2024-01-01T12:00:00Z",
///   "checks": {
///     "database": {
///       "status": "healthy",
///       "message": "Connected",
///       "response_time_ms": 5
///     }
///   }
/// }
/// ```
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy", body = HealthResponse)
    ),
    tag = HEALTH_TAG
)]
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let mut checks = HashMap::new();
    let mut overall_status = HealthStatus::Healthy;

    // Check database connectivity
    let db_check = check_database(&state).await;
    if matches!(db_check.status, HealthStatus::Unhealthy) {
        overall_status = HealthStatus::Unhealthy;
    } else if matches!(db_check.status, HealthStatus::Degraded)
        && matches!(overall_status, HealthStatus::Healthy)
    {
        overall_status = HealthStatus::Degraded;
    }
    checks.insert("database".to_string(), db_check);

    let response = HealthResponse {
        status: overall_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        checks,
    };

    match response.status {
        HealthStatus::Healthy => Ok(Json(response)),
        HealthStatus::Degraded => Ok(Json(response)),
        HealthStatus::Unhealthy => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// Readiness probe endpoint.
///
/// Indicates whether the service is ready to accept traffic.
/// Checks all dependencies including database connectivity.
///
/// # Responses
/// - `200 OK` - Service is ready
/// - `503 Service Unavailable` - Service is not ready
#[utoipa::path(
    get,
    path = "/health/ready",
    responses(
        (status = 200, description = "Service is ready"),
        (status = 503, description = "Service is not ready")
    ),
    tag = HEALTH_TAG
)]
pub async fn readiness_check(State(state): State<AppState>) -> StatusCode {
    // Check if database is accessible
    let db_check = check_database(&state).await;

    match db_check.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded | HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    }
}

/// Liveness probe endpoint.
///
/// Indicates whether the service is alive and should not be restarted.
/// This is a lightweight check that doesn't test external dependencies.
///
/// # Responses
/// - `200 OK` - Service is alive
#[utoipa::path(
    get,
    path = "/health/live",
    responses(
        (status = 200, description = "Service is alive")
    ),
    tag = HEALTH_TAG
)]
pub async fn liveness_check() -> StatusCode {
    // Simple liveness check - if we can respond, we're alive
    StatusCode::OK
}

/// Check database connectivity by directly accessing the connection pool.
///
/// This function bypasses the service layer and directly tests the database
/// connection pool to provide a more accurate health check.
async fn check_database(state: &AppState) -> ComponentHealth {
    let start_time = std::time::Instant::now();

    // Try to get a connection from the pool
    match state.db_pool.get().await {
        Ok(mut conn) => {
            // Try a simple query to verify the connection works
            use diesel_async::RunQueryDsl;

            match diesel::sql_query("SELECT 1").execute(&mut conn).await {
                Ok(_) => ComponentHealth {
                    status: HealthStatus::Healthy,
                    message: Some("Connected".to_string()),
                    response_time_ms: Some(start_time.elapsed().as_millis() as u64),
                },
                Err(e) => ComponentHealth {
                    status: HealthStatus::Unhealthy,
                    message: Some(format!("Query failed: {}", e)),
                    response_time_ms: Some(start_time.elapsed().as_millis() as u64),
                },
            }
        }
        Err(e) => ComponentHealth {
            status: HealthStatus::Unhealthy,
            message: Some(format!("Connection failed: {}", e)),
            response_time_ms: Some(start_time.elapsed().as_millis() as u64),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"healthy\"");
    }

    #[test]
    fn test_component_health_creation() {
        let health = ComponentHealth {
            status: HealthStatus::Healthy,
            message: Some("All good".to_string()),
            response_time_ms: Some(10),
        };

        assert!(matches!(health.status, HealthStatus::Healthy));
        assert_eq!(health.message, Some("All good".to_string()));
        assert_eq!(health.response_time_ms, Some(10));
    }

    #[tokio::test]
    async fn test_liveness_check() {
        let result = liveness_check().await;
        assert_eq!(result, StatusCode::OK);
    }

    #[test]
    fn test_health_response_creation() {
        let mut checks = HashMap::new();
        checks.insert(
            "test".to_string(),
            ComponentHealth {
                status: HealthStatus::Healthy,
                message: Some("OK".to_string()),
                response_time_ms: Some(5),
            },
        );

        let response = HealthResponse {
            status: HealthStatus::Healthy,
            version: "0.1.0".to_string(),
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            checks,
        };

        assert!(matches!(response.status, HealthStatus::Healthy));
        assert_eq!(response.version, "0.1.0");
        assert_eq!(response.checks.len(), 1);
    }

    #[test]
    fn test_database_health_check_structure() {
        // Test that ComponentHealth can represent database check results
        let healthy_db = ComponentHealth {
            status: HealthStatus::Healthy,
            message: Some("Connected".to_string()),
            response_time_ms: Some(10),
        };

        let unhealthy_db = ComponentHealth {
            status: HealthStatus::Unhealthy,
            message: Some("Connection failed: timeout".to_string()),
            response_time_ms: Some(5000),
        };

        assert!(matches!(healthy_db.status, HealthStatus::Healthy));
        assert!(matches!(unhealthy_db.status, HealthStatus::Unhealthy));
        assert!(
            unhealthy_db
                .message
                .as_ref()
                .unwrap()
                .contains("Connection failed")
        );
    }
}
