//! Error response DTOs.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Standard error response format with flexible details.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ErrorResponse {
    /// Creates a new error response with code and message.
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    /// Adds details to the error response.
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Creates a validation error response with field-specific details.
    pub fn validation_error(field: &str, reason: &str) -> Self {
        Self {
            code: "VALIDATION_ERROR".to_string(),
            message: format!("Validation failed for {}: {}", field, reason),
            details: Some(json!({
                "field": field,
                "reason": reason
            })),
        }
    }

    /// Creates a not found error response with entity and identifier details.
    pub fn not_found_error(entity: &str, field: &str, value: &str) -> Self {
        Self {
            code: "NOT_FOUND".to_string(),
            message: format!("Resource not found: {} with {}={}", entity, field, value),
            details: Some(json!({
                "entity": entity,
                "field": field,
                "value": value
            })),
        }
    }

    /// Creates a duplicate error response with conflict details.
    pub fn duplicate_error(entity: &str, field: &str, value: &str) -> Self {
        Self {
            code: "DUPLICATE_ENTRY".to_string(),
            message: format!("Duplicate entry: {}.{} = '{}' already exists", entity, field, value),
            details: Some(json!({
                "entity": entity,
                "field": field,
                "value": value
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_error_response_new() {
        let error = ErrorResponse::new("TEST_ERROR", "Test message");
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test message");
        assert!(error.details.is_none());
    }

    #[test]
    fn test_error_response_with_details() {
        let error = ErrorResponse::new("TEST_ERROR", "Test message")
            .with_details(json!({"key": "value"}));
        
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test message");
        assert!(error.details.is_some());
        assert_eq!(error.details.unwrap(), json!({"key": "value"}));
    }

    #[test]
    fn test_validation_error() {
        let error = ErrorResponse::validation_error("email", "invalid format");
        
        assert_eq!(error.code, "VALIDATION_ERROR");
        assert_eq!(error.message, "Validation failed for email: invalid format");
        assert!(error.details.is_some());
        
        let details = error.details.unwrap();
        assert_eq!(details["field"], "email");
        assert_eq!(details["reason"], "invalid format");
    }

    #[test]
    fn test_not_found_error() {
        let error = ErrorResponse::not_found_error("user", "id", "123");
        
        assert_eq!(error.code, "NOT_FOUND");
        assert_eq!(error.message, "Resource not found: user with id=123");
        assert!(error.details.is_some());
        
        let details = error.details.unwrap();
        assert_eq!(details["entity"], "user");
        assert_eq!(details["field"], "id");
        assert_eq!(details["value"], "123");
    }

    #[test]
    fn test_duplicate_error() {
        let error = ErrorResponse::duplicate_error("user", "email", "test@example.com");
        
        assert_eq!(error.code, "DUPLICATE_ENTRY");
        assert_eq!(error.message, "Duplicate entry: user.email = 'test@example.com' already exists");
        assert!(error.details.is_some());
        
        let details = error.details.unwrap();
        assert_eq!(details["entity"], "user");
        assert_eq!(details["field"], "email");
        assert_eq!(details["value"], "test@example.com");
    }

    #[test]
    fn test_serialization_round_trip() {
        let original = ErrorResponse::validation_error("email", "invalid format");
        
        let json_str = serde_json::to_string(&original).unwrap();
        let deserialized: ErrorResponse = serde_json::from_str(&json_str).unwrap();
        
        assert_eq!(original.code, deserialized.code);
        assert_eq!(original.message, deserialized.message);
        assert_eq!(original.details, deserialized.details);
    }
}
