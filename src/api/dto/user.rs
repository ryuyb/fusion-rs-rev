//! User-related DTOs for API requests and responses.

use crate::models::{NewUser, UpdateUser, User};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

// ============================================================================
// Request DTOs
// ============================================================================

/// Request body for creating a new user.
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 20, message = "Username must be between 3 and 20 characters"))]
    #[schema(min_length = 3, max_length = 20)]
    pub username: String,
    #[validate(email(message = "Invalid email format"))]
    #[schema(format = "email")]
    pub email: String,
    #[validate(length(min = 6, max = 30, message = "Password must be between 6 and 30 characters"))]
    #[schema(format = "password", min_length = 6, max_length = 30)]
    pub password: String,
}

impl CreateUserRequest {
    /// Converts the request DTO into a NewUser model for database insertion.
    pub fn into_new_user(self) -> NewUser {
        NewUser {
            username: self.username,
            email: self.email,
            password: self.password,
        }
    }
}

/// Request body for updating a user.
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 20, message = "Username must be between 3 and 20 characters"))]
    pub username: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    #[schema(format = "email")]
    pub email: Option<String>,
    #[validate(length(min = 6, max = 30, message = "Password must be between 6 and 30 characters"))]
    pub password: Option<String>,
}

impl UpdateUserRequest {
    /// Converts the request DTO into an UpdateUser model for database update.
    pub fn into_update_user(self) -> UpdateUser {
        UpdateUser {
            username: self.username,
            email: self.email,
            password: self.password,
        }
    }
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Response body for user data (excludes sensitive fields like password).
#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            updated_at: user.updated_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        }
    }
}
