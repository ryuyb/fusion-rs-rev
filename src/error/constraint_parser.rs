use std::collections::HashMap;
use std::sync::OnceLock;
use regex::Regex;

/// Utility for parsing PostgreSQL constraint violation messages.
///
/// This parser uses regex patterns to extract structured information from
/// database constraint violation messages, with caching for performance.
pub struct ConstraintParser;

/// Compiled regex patterns for constraint parsing, cached for performance
struct RegexPatterns {
    key_value: Regex,
    column_name: Regex,
    table_name: Regex,
    #[allow(dead_code)]
    constraint_name: Regex,
}

impl RegexPatterns {
    fn new() -> Self {
        Self {
            // Matches "Key (field)=(value)" pattern in PostgreSQL messages
            key_value: Regex::new(r"Key \(([^)]+)\)=\(([^)]*)\)").unwrap(),
            // Matches column names in quotes
            column_name: Regex::new(r#"column "([^"]+)""#).unwrap(),
            // Matches table names in quotes
            table_name: Regex::new(r#"table "([^"]+)""#).unwrap(),
            // Matches constraint names in quotes
            constraint_name: Regex::new(r#"constraint "([^"]+)""#).unwrap(),
        }
    }
}

/// Global regex patterns cache
static REGEX_PATTERNS: OnceLock<RegexPatterns> = OnceLock::new();

impl ConstraintParser {
    /// Gets the cached regex patterns, initializing them if necessary
    fn patterns() -> &'static RegexPatterns {
        REGEX_PATTERNS.get_or_init(RegexPatterns::new)
    }

    /// Parses a unique constraint violation message to extract structured information.
    ///
    /// Attempts to extract entity, field, and value from PostgreSQL unique constraint
    /// violation messages using regex patterns and constraint name analysis.
    ///
    /// # Arguments
    /// * `message` - The database error message
    /// * `constraint_name` - Optional constraint name from the database
    ///
    /// # Returns
    /// Optional tuple of (entity, field, value) if parsing succeeds
    ///
    /// # Examples
    /// ```
    /// use crate::error::constraint_parser::ConstraintParser;
    /// 
    /// let message = "duplicate key value violates unique constraint \"users_email_key\"\nDETAIL: Key (email)=(test@example.com) already exists.";
    /// let result = ConstraintParser::parse_unique_violation(message, Some("users_email_key"));
    /// assert_eq!(result, Some(("users".to_string(), "email".to_string(), "test@example.com".to_string())));
    /// ```
    pub fn parse_unique_violation(
        message: &str,
        constraint_name: Option<&str>,
    ) -> Option<(String, String, String)> {
        // Try to parse from constraint name first (e.g., "users_email_key")
        if let Some(constraint) = constraint_name {
            if let Some((entity, field)) = Self::parse_constraint_name(constraint) {
                // Extract value from message using regex
                if let Some(value) = Self::extract_value_from_message(message) {
                    return Some((entity, field, value));
                }
                // Fallback to generic value if we can't parse it
                return Some((entity, field, "duplicate_value".to_string()));
            }
        }

        // Fallback: try to parse from the error message directly
        if let Some((field, value)) = Self::extract_key_value_from_message(message) {
            // Try to infer entity from context or use generic
            let entity = Self::extract_table_from_message(message)
                .unwrap_or_else(|| "resource".to_string());
            return Some((entity, field, value));
        }

        None
    }

    /// Parses a not null constraint violation message.
    ///
    /// Extracts entity and field information from PostgreSQL not null constraint
    /// violation messages.
    ///
    /// # Arguments
    /// * `message` - The database error message
    /// * `constraint_name` - Optional constraint name from the database
    ///
    /// # Returns
    /// Optional tuple of (entity, field) if parsing succeeds
    ///
    /// # Examples
    /// ```
    /// use crate::error::constraint_parser::ConstraintParser;
    /// 
    /// let message = "null value in column \"email\" violates not-null constraint";
    /// let result = ConstraintParser::parse_not_null_violation(message, None);
    /// assert_eq!(result, Some(("resource".to_string(), "email".to_string())));
    /// ```
    pub fn parse_not_null_violation(
        message: &str,
        constraint_name: Option<&str>,
    ) -> Option<(String, String)> {
        // Try to extract field from message using regex
        if let Some(field) = Self::extract_column_from_message(message) {
            let entity = Self::extract_table_from_message(message)
                .or_else(|| {
                    constraint_name.and_then(|c| Self::parse_constraint_name(c).map(|(e, _)| e))
                })
                .unwrap_or_else(|| "resource".to_string());
            return Some((entity, field));
        }

        None
    }

    /// Parses a foreign key constraint violation message.
    ///
    /// Extracts entity, field, and referenced value information from PostgreSQL
    /// foreign key constraint violation messages.
    ///
    /// # Arguments
    /// * `message` - The database error message
    /// * `constraint_name` - Optional constraint name from the database
    ///
    /// # Returns
    /// Optional tuple of (entity, field, referenced_value) if parsing succeeds
    ///
    /// # Examples
    /// ```
    /// use crate::error::constraint_parser::ConstraintParser;
    /// 
    /// let message = "insert or update on table \"posts\" violates foreign key constraint \"posts_user_id_fkey\"\nDETAIL: Key (user_id)=(999) is not present in table \"users\".";
    /// let result = ConstraintParser::parse_foreign_key_violation(message, Some("posts_user_id_fkey"));
    /// assert_eq!(result, Some(("posts".to_string(), "user_id".to_string(), "999".to_string())));
    /// ```
    pub fn parse_foreign_key_violation(
        message: &str,
        constraint_name: Option<&str>,
    ) -> Option<(String, String, String)> {
        // Try to parse from constraint name (e.g., "posts_user_id_fkey")
        if let Some(constraint) = constraint_name {
            if let Some((entity, field)) = Self::parse_foreign_key_constraint_name(constraint) {
                if let Some(value) = Self::extract_value_from_message(message) {
                    return Some((entity, field, value));
                }
                return Some((entity, field, "invalid_reference".to_string()));
            }
        }

        // Fallback: parse from message
        if let Some((field, value)) = Self::extract_key_value_from_message(message) {
            let entity = Self::extract_table_from_message(message)
                .unwrap_or_else(|| "resource".to_string());
            return Some((entity, field, value));
        }

        None
    }

    /// Parses a check constraint violation message.
    ///
    /// Extracts entity and field information from PostgreSQL check constraint
    /// violation messages.
    ///
    /// # Arguments
    /// * `message` - The database error message
    /// * `constraint_name` - Optional constraint name from the database
    ///
    /// # Returns
    /// Optional tuple of (entity, field) if parsing succeeds
    ///
    /// # Examples
    /// ```
    /// use crate::error::constraint_parser::ConstraintParser;
    /// 
    /// let message = "new row for relation \"users\" violates check constraint \"users_age_check\"";
    /// let result = ConstraintParser::parse_check_violation(message, Some("users_age_check"));
    /// assert_eq!(result, Some(("users".to_string(), "age".to_string())));
    /// ```
    pub fn parse_check_violation(
        message: &str,
        constraint_name: Option<&str>,
    ) -> Option<(String, String)> {
        // Try to parse from constraint name
        if let Some(constraint) = constraint_name {
            if let Some((entity, field)) = Self::parse_constraint_name(constraint) {
                return Some((entity, field));
            }
        }

        // Fallback: try to extract from message
        if let Some(field) = Self::extract_column_from_message(message) {
            let entity = Self::extract_table_from_message(message)
                .unwrap_or_else(|| "resource".to_string());
            return Some((entity, field));
        }

        None
    }

    /// Parses a constraint name to extract entity and field information.
    ///
    /// Handles common PostgreSQL constraint naming patterns:
    /// - "users_email_key" -> ("users", "email")
    /// - "posts_title_idx" -> ("posts", "title")
    /// - "users_age_check" -> ("users", "age")
    ///
    /// # Arguments
    /// * `constraint_name` - The constraint name to parse
    ///
    /// # Returns
    /// Optional tuple of (entity, field) if parsing succeeds
    pub fn parse_constraint_name(constraint_name: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = constraint_name.split('_').collect();
        if parts.len() >= 3 {
            let entity = parts[0].to_string();
            let field = parts[1].to_string();
            return Some((entity, field));
        }
        None
    }

    /// Parses a foreign key constraint name to extract entity and field information.
    ///
    /// Handles patterns like "posts_user_id_fkey" -> ("posts", "user_id")
    ///
    /// # Arguments
    /// * `constraint_name` - The foreign key constraint name to parse
    ///
    /// # Returns
    /// Optional tuple of (entity, field) if parsing succeeds
    pub fn parse_foreign_key_constraint_name(constraint_name: &str) -> Option<(String, String)> {
        if constraint_name.ends_with("_fkey") {
            let without_suffix = &constraint_name[..constraint_name.len() - 5]; // Remove "_fkey"
            let parts: Vec<&str> = without_suffix.split('_').collect();
            if parts.len() >= 2 {
                let entity = parts[0].to_string();
                let field = parts[1..].join("_"); // Handle multi-part field names like "user_id"
                return Some((entity, field));
            }
        }
        None
    }

    /// Extracts a column name from a database error message using regex.
    ///
    /// Looks for patterns like "column \"field_name\"" in PostgreSQL messages.
    ///
    /// # Arguments
    /// * `message` - The database error message
    ///
    /// # Returns
    /// Optional field name if found
    pub fn extract_column_from_message(message: &str) -> Option<String> {
        let patterns = Self::patterns();
        patterns.column_name
            .captures(message)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extracts a table name from a database error message using regex.
    ///
    /// Looks for patterns like "table \"table_name\"" in PostgreSQL messages.
    ///
    /// # Arguments
    /// * `message` - The database error message
    ///
    /// # Returns
    /// Optional table name if found
    pub fn extract_table_from_message(message: &str) -> Option<String> {
        let patterns = Self::patterns();
        patterns.table_name
            .captures(message)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extracts key-value pairs from database error messages using regex.
    ///
    /// Looks for patterns like "Key (field)=(value)" in PostgreSQL messages.
    ///
    /// # Arguments
    /// * `message` - The database error message
    ///
    /// # Returns
    /// Optional tuple of (field, value) if found
    pub fn extract_key_value_from_message(message: &str) -> Option<(String, String)> {
        let patterns = Self::patterns();
        patterns.key_value
            .captures(message)
            .and_then(|caps| {
                let field = caps.get(1)?.as_str().to_string();
                let value = caps.get(2)?.as_str().to_string();
                Some((field, value))
            })
    }

    /// Extracts a value from a database error message.
    ///
    /// First tries the Key (field)=(value) pattern, then falls back to
    /// other value extraction methods.
    ///
    /// # Arguments
    /// * `message` - The database error message
    ///
    /// # Returns
    /// Optional value string if found
    pub fn extract_value_from_message(message: &str) -> Option<String> {
        // First try the Key (field)=(value) pattern
        if let Some((_, value)) = Self::extract_key_value_from_message(message) {
            return Some(value);
        }

        // Fallback: look for quoted strings using simple string search
        // This is a fallback for cases where regex might not match
        if let Some(start) = message.find('"') {
            if let Some(end) = message[start + 1..].find('"') {
                return Some(message[start + 1..start + 1 + end].to_string());
            }
        }

        None
    }

    /// Attempts to parse any constraint violation message and return structured information.
    ///
    /// This is a convenience method that tries to parse the message as different
    /// constraint types and returns the first successful parse.
    ///
    /// # Arguments
    /// * `message` - The database error message
    /// * `constraint_name` - Optional constraint name from the database
    ///
    /// # Returns
    /// Optional structured constraint information as a HashMap
    #[allow(dead_code)]
    pub fn parse_constraint_violation(
        message: &str,
        constraint_name: Option<&str>,
    ) -> Option<HashMap<String, String>> {
        // Try unique constraint first
        if let Some((entity, field, value)) = Self::parse_unique_violation(message, constraint_name) {
            let mut result = HashMap::new();
            result.insert("type".to_string(), "unique".to_string());
            result.insert("entity".to_string(), entity);
            result.insert("field".to_string(), field);
            result.insert("value".to_string(), value);
            return Some(result);
        }

        // Try foreign key constraint
        if let Some((entity, field, value)) = Self::parse_foreign_key_violation(message, constraint_name) {
            let mut result = HashMap::new();
            result.insert("type".to_string(), "foreign_key".to_string());
            result.insert("entity".to_string(), entity);
            result.insert("field".to_string(), field);
            result.insert("referenced_value".to_string(), value);
            return Some(result);
        }

        // Try not null constraint
        if let Some((entity, field)) = Self::parse_not_null_violation(message, constraint_name) {
            let mut result = HashMap::new();
            result.insert("type".to_string(), "not_null".to_string());
            result.insert("entity".to_string(), entity);
            result.insert("field".to_string(), field);
            return Some(result);
        }

        // Try check constraint
        if let Some((entity, field)) = Self::parse_check_violation(message, constraint_name) {
            let mut result = HashMap::new();
            result.insert("type".to_string(), "check".to_string());
            result.insert("entity".to_string(), entity);
            result.insert("field".to_string(), field);
            return Some(result);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unique_violation_with_constraint_name() {
        let message = "duplicate key value violates unique constraint \"users_email_key\"\nDETAIL: Key (email)=(test@example.com) already exists.";
        let result = ConstraintParser::parse_unique_violation(message, Some("users_email_key"));
        assert_eq!(result, Some(("users".to_string(), "email".to_string(), "test@example.com".to_string())));
    }

    #[test]
    fn test_parse_unique_violation_without_constraint_name() {
        let message = "duplicate key value violates unique constraint\nDETAIL: Key (username)=(john_doe) already exists.";
        let result = ConstraintParser::parse_unique_violation(message, None);
        assert_eq!(result, Some(("resource".to_string(), "username".to_string(), "john_doe".to_string())));
    }

    #[test]
    fn test_parse_not_null_violation() {
        let message = "null value in column \"email\" violates not-null constraint";
        let result = ConstraintParser::parse_not_null_violation(message, None);
        assert_eq!(result, Some(("resource".to_string(), "email".to_string())));
    }

    #[test]
    fn test_parse_not_null_violation_with_table() {
        let message = "null value in column \"email\" of relation \"users\" violates not-null constraint";
        let result = ConstraintParser::parse_not_null_violation(message, None);
        assert_eq!(result, Some(("resource".to_string(), "email".to_string())));
    }

    #[test]
    fn test_parse_foreign_key_violation() {
        let message = "insert or update on table \"posts\" violates foreign key constraint \"posts_user_id_fkey\"\nDETAIL: Key (user_id)=(999) is not present in table \"users\".";
        let result = ConstraintParser::parse_foreign_key_violation(message, Some("posts_user_id_fkey"));
        assert_eq!(result, Some(("posts".to_string(), "user_id".to_string(), "999".to_string())));
    }

    #[test]
    fn test_parse_check_violation() {
        let message = "new row for relation \"users\" violates check constraint \"users_age_check\"";
        let result = ConstraintParser::parse_check_violation(message, Some("users_age_check"));
        assert_eq!(result, Some(("users".to_string(), "age".to_string())));
    }

    #[test]
    fn test_parse_constraint_name() {
        let result = ConstraintParser::parse_constraint_name("users_email_key");
        assert_eq!(result, Some(("users".to_string(), "email".to_string())));
        
        let result = ConstraintParser::parse_constraint_name("posts_title_idx");
        assert_eq!(result, Some(("posts".to_string(), "title".to_string())));
        
        let result = ConstraintParser::parse_constraint_name("invalid");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_foreign_key_constraint_name() {
        let result = ConstraintParser::parse_foreign_key_constraint_name("posts_user_id_fkey");
        assert_eq!(result, Some(("posts".to_string(), "user_id".to_string())));
        
        let result = ConstraintParser::parse_foreign_key_constraint_name("comments_post_id_fkey");
        assert_eq!(result, Some(("comments".to_string(), "post_id".to_string())));
        
        let result = ConstraintParser::parse_foreign_key_constraint_name("not_a_foreign_key");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_column_from_message() {
        let message = "null value in column \"email\" violates not-null constraint";
        let result = ConstraintParser::extract_column_from_message(message);
        assert_eq!(result, Some("email".to_string()));
        
        let message = "no column found here";
        let result = ConstraintParser::extract_column_from_message(message);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_table_from_message() {
        let message = "insert or update on table \"posts\" violates foreign key constraint";
        let result = ConstraintParser::extract_table_from_message(message);
        assert_eq!(result, Some("posts".to_string()));
        
        let message = "no table found here";
        let result = ConstraintParser::extract_table_from_message(message);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_key_value_from_message() {
        let message = "duplicate key value violates unique constraint \"users_email_key\"\nDETAIL: Key (email)=(test@example.com) already exists.";
        let result = ConstraintParser::extract_key_value_from_message(message);
        assert_eq!(result, Some(("email".to_string(), "test@example.com".to_string())));
        
        let message = "Key (user_id)=(123) is not present in table";
        let result = ConstraintParser::extract_key_value_from_message(message);
        assert_eq!(result, Some(("user_id".to_string(), "123".to_string())));
    }

    #[test]
    fn test_extract_value_from_message() {
        let message = "Key (email)=(test@example.com) already exists";
        let result = ConstraintParser::extract_value_from_message(message);
        assert_eq!(result, Some("test@example.com".to_string()));
        
        let message = "some error with \"quoted_value\" in it";
        let result = ConstraintParser::extract_value_from_message(message);
        assert_eq!(result, Some("quoted_value".to_string()));
    }

    #[test]
    fn test_parse_constraint_violation_unique() {
        let message = "duplicate key value violates unique constraint \"users_email_key\"\nDETAIL: Key (email)=(test@example.com) already exists.";
        let result = ConstraintParser::parse_constraint_violation(message, Some("users_email_key"));
        
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.get("type"), Some(&"unique".to_string()));
        assert_eq!(parsed.get("entity"), Some(&"users".to_string()));
        assert_eq!(parsed.get("field"), Some(&"email".to_string()));
        assert_eq!(parsed.get("value"), Some(&"test@example.com".to_string()));
    }

    #[test]
    fn test_parse_constraint_violation_not_null() {
        let message = "null value in column \"email\" violates not-null constraint";
        let result = ConstraintParser::parse_constraint_violation(message, None);
        
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.get("type"), Some(&"not_null".to_string()));
        assert_eq!(parsed.get("entity"), Some(&"resource".to_string()));
        assert_eq!(parsed.get("field"), Some(&"email".to_string()));
    }

    #[test]
    fn test_regex_patterns_caching() {
        // Test that patterns are cached by calling multiple times
        let patterns1 = ConstraintParser::patterns();
        let patterns2 = ConstraintParser::patterns();
        
        // They should be the same instance (pointer equality)
        assert!(std::ptr::eq(patterns1, patterns2));
    }

    #[test]
    fn test_graceful_parsing_failures() {
        // Test that parsing failures return None gracefully
        let message = "completely unrelated error message";
        let result = ConstraintParser::parse_unique_violation(message, None);
        assert_eq!(result, None);
        
        let result = ConstraintParser::parse_not_null_violation(message, None);
        assert_eq!(result, None);
        
        let result = ConstraintParser::parse_foreign_key_violation(message, None);
        assert_eq!(result, None);
        
        let result = ConstraintParser::parse_check_violation(message, None);
        assert_eq!(result, None);
        
        let result = ConstraintParser::parse_constraint_violation(message, None);
        assert_eq!(result, None);
    }
}