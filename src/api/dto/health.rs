//! Health check DTOs for API responses.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Health check response structure.
///
/// Provides information about the application's health status
/// and various system components.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "status": "healthy",
    "version": "0.1.0",
    "timestamp": "2024-01-01T12:00:00.000Z",
    "checks": {
        "database": {
            "status": "healthy",
            "message": "Connected",
            "response_time_ms": 5
        }
    }
}))]
pub struct HealthResponse {
    /// Overall health status
    #[schema(example = "healthy")]
    pub status: HealthStatus,
    /// Application version
    #[schema(example = "0.1.0")]
    pub version: String,
    /// Timestamp of the health check (ISO 8601 format)
    #[schema(value_type = String, format = DateTime, example = "2024-01-01T12:00:00.000Z")]
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
#[schema(example = json!({
    "status": "healthy",
    "message": "Connected",
    "response_time_ms": 5
}))]
pub struct ComponentHealth {
    /// Component status
    #[schema(example = "healthy")]
    pub status: HealthStatus,
    /// Optional message with details
    #[schema(example = "Connected")]
    pub message: Option<String>,
    /// Response time in milliseconds
    #[schema(example = 5)]
    pub response_time_ms: Option<u64>,
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
