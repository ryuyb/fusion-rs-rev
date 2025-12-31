use crate::error::{AppError, ConstraintParser};
use diesel::result::{DatabaseErrorKind, Error as DieselError};

/// Utility for converting database errors to structured AppError variants.
///
/// This converter handles Diesel database errors and transforms them into
/// appropriate AppError variants with structured information extracted from
/// constraint violation messages.
pub struct DatabaseErrorConverter;

impl DatabaseErrorConverter {
    /// Converts a Diesel error to an appropriate AppError variant.
    ///
    /// # Arguments
    /// * `error` - The Diesel error to convert
    /// * `operation` - Description of the database operation that failed
    ///
    /// # Returns
    /// An AppError variant appropriate for the type of database error
    pub fn convert_diesel_error(error: DieselError, operation: &str) -> AppError {
        match error {
            DieselError::DatabaseError(kind, info) => {
                Self::convert_database_error(kind, info, operation)
            }
            DieselError::NotFound => AppError::NotFound {
                entity: "resource".to_string(),
                field: "id".to_string(),
                value: "unknown".to_string(),
            },
            other => AppError::Database {
                operation: operation.to_string(),
                source: anyhow::Error::from(other),
            },
        }
    }

    /// Converts a database error with detailed constraint information.
    ///
    /// # Arguments
    /// * `kind` - The type of database error
    /// * `info` - Detailed error information from the database
    /// * `operation` - Description of the database operation that failed
    ///
    /// # Returns
    /// An AppError variant with structured constraint violation information
    fn convert_database_error(
        kind: DatabaseErrorKind,
        info: Box<dyn diesel::result::DatabaseErrorInformation + Send + Sync>,
        operation: &str,
    ) -> AppError {
        let message = info.message();
        let constraint_name = info.constraint_name();

        match kind {
            DatabaseErrorKind::UniqueViolation => {
                if let Some((entity, field, value)) =
                    ConstraintParser::parse_unique_violation(message, constraint_name)
                {
                    AppError::Duplicate {
                        entity,
                        field,
                        value,
                    }
                } else {
                    AppError::Database {
                        operation: operation.to_string(),
                        source: anyhow::Error::msg(format!(
                            "Unique constraint violation: {}",
                            message
                        )),
                    }
                }
            }
            DatabaseErrorKind::NotNullViolation => {
                if let Some((entity, field)) =
                    ConstraintParser::parse_not_null_violation(message, constraint_name)
                {
                    AppError::Validation {
                        field,
                        reason: format!("Field is required for {}", entity),
                    }
                } else {
                    AppError::Database {
                        operation: operation.to_string(),
                        source: anyhow::Error::msg(format!(
                            "Not null constraint violation: {}",
                            message
                        )),
                    }
                }
            }
            DatabaseErrorKind::ForeignKeyViolation => {
                if let Some((entity, field, referenced_value)) =
                    ConstraintParser::parse_foreign_key_violation(message, constraint_name)
                {
                    AppError::Validation {
                        field,
                        reason: format!(
                            "Invalid reference to {} with value '{}'",
                            entity, referenced_value
                        ),
                    }
                } else {
                    AppError::Database {
                        operation: operation.to_string(),
                        source: anyhow::Error::msg(format!(
                            "Foreign key constraint violation: {}",
                            message
                        )),
                    }
                }
            }
            DatabaseErrorKind::CheckViolation => {
                if let Some((entity, field)) =
                    ConstraintParser::parse_check_violation(message, constraint_name)
                {
                    AppError::Validation {
                        field,
                        reason: format!("Check constraint failed for {} field", entity),
                    }
                } else {
                    AppError::Database {
                        operation: operation.to_string(),
                        source: anyhow::Error::msg(format!(
                            "Check constraint violation: {}",
                            message
                        )),
                    }
                }
            }
            _ => AppError::Database {
                operation: operation.to_string(),
                source: anyhow::Error::msg(format!("Database error: {}", message)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::result::{DatabaseErrorKind, Error as DieselError};

    // Mock database error information for testing
    struct MockDatabaseErrorInfo {
        message: String,
        constraint_name: Option<String>,
    }

    impl diesel::result::DatabaseErrorInformation for MockDatabaseErrorInfo {
        fn message(&self) -> &str {
            &self.message
        }

        fn details(&self) -> Option<&str> {
            None
        }

        fn hint(&self) -> Option<&str> {
            None
        }

        fn table_name(&self) -> Option<&str> {
            None
        }

        fn column_name(&self) -> Option<&str> {
            None
        }

        fn constraint_name(&self) -> Option<&str> {
            self.constraint_name.as_deref()
        }

        fn statement_position(&self) -> Option<i32> {
            None
        }
    }

    #[test]
    fn test_convert_not_found_error() {
        let error = DieselError::NotFound;
        let result = DatabaseErrorConverter::convert_diesel_error(error, "find user");

        match result {
            AppError::NotFound {
                entity,
                field,
                value,
            } => {
                assert_eq!(entity, "resource");
                assert_eq!(field, "id");
                assert_eq!(value, "unknown");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_convert_unique_violation_with_constraint_name() {
        let info = MockDatabaseErrorInfo {
            message: "duplicate key value violates unique constraint \"users_email_key\"\nDETAIL: Key (email)=(test@example.com) already exists.".to_string(),
            constraint_name: Some("users_email_key".to_string()),
        };

        let error = DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, Box::new(info));

        let result = DatabaseErrorConverter::convert_diesel_error(error, "insert user");

        match result {
            AppError::Duplicate {
                entity,
                field,
                value,
            } => {
                assert_eq!(entity, "users");
                assert_eq!(field, "email");
                assert_eq!(value, "test@example.com");
            }
            _ => panic!("Expected Duplicate error, got: {:?}", result),
        }
    }

    #[test]
    fn test_convert_not_null_violation() {
        let info = MockDatabaseErrorInfo {
            message: "null value in column \"email\" violates not-null constraint".to_string(),
            constraint_name: None,
        };

        let error = DieselError::DatabaseError(DatabaseErrorKind::NotNullViolation, Box::new(info));

        let result = DatabaseErrorConverter::convert_diesel_error(error, "insert user");

        match result {
            AppError::Validation { field, reason } => {
                assert_eq!(field, "email");
                assert!(reason.contains("required"));
            }
            _ => panic!("Expected Validation error, got: {:?}", result),
        }
    }

    #[test]
    fn test_parse_constraint_name() {
        let result = ConstraintParser::parse_constraint_name("users_email_key");
        assert_eq!(result, Some(("users".to_string(), "email".to_string())));

        let result = ConstraintParser::parse_constraint_name("posts_user_id_fkey");
        assert_eq!(result, Some(("posts".to_string(), "user".to_string())));
    }

    #[test]
    fn test_extract_key_value_from_message() {
        let message = "duplicate key value violates unique constraint \"users_email_key\"\nDETAIL: Key (email)=(test@example.com) already exists.";
        let result = ConstraintParser::extract_key_value_from_message(message);
        assert_eq!(
            result,
            Some(("email".to_string(), "test@example.com".to_string()))
        );
    }

    #[test]
    fn test_extract_column_from_message() {
        let message = "null value in column \"email\" violates not-null constraint";
        let result = ConstraintParser::extract_column_from_message(message);
        assert_eq!(result, Some("email".to_string()));
    }

    #[test]
    fn test_parse_foreign_key_constraint_name() {
        let result = ConstraintParser::parse_foreign_key_constraint_name("posts_user_id_fkey");
        assert_eq!(result, Some(("posts".to_string(), "user_id".to_string())));

        let result = ConstraintParser::parse_foreign_key_constraint_name("comments_post_id_fkey");
        assert_eq!(
            result,
            Some(("comments".to_string(), "post_id".to_string()))
        );
    }

    #[test]
    fn test_convert_foreign_key_violation() {
        let info = MockDatabaseErrorInfo {
            message: "insert or update on table \"posts\" violates foreign key constraint \"posts_user_id_fkey\"\nDETAIL: Key (user_id)=(999) is not present in table \"users\".".to_string(),
            constraint_name: Some("posts_user_id_fkey".to_string()),
        };

        let error =
            DieselError::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, Box::new(info));

        let result = DatabaseErrorConverter::convert_diesel_error(error, "insert post");

        match result {
            AppError::Validation { field, reason } => {
                assert_eq!(field, "user_id");
                assert!(reason.contains("Invalid reference"));
                assert!(reason.contains("999"));
            }
            _ => panic!("Expected Validation error, got: {:?}", result),
        }
    }

    #[test]
    fn test_convert_check_violation() {
        let info = MockDatabaseErrorInfo {
            message: "new row for relation \"users\" violates check constraint \"users_age_check\""
                .to_string(),
            constraint_name: Some("users_age_check".to_string()),
        };

        let error = DieselError::DatabaseError(DatabaseErrorKind::CheckViolation, Box::new(info));

        let result = DatabaseErrorConverter::convert_diesel_error(error, "insert user");

        match result {
            AppError::Validation { field, reason } => {
                assert_eq!(field, "age");
                assert!(reason.contains("Check constraint failed"));
            }
            _ => panic!("Expected Validation error, got: {:?}", result),
        }
    }
}
