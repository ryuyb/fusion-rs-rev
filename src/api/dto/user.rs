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
    #[validate(length(
        min = 3,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    #[schema(min_length = 3, max_length = 20, example = "john_doe")]
    pub username: String,
    #[validate(email(message = "Invalid email format"))]
    #[schema(format = "email", example = "john@example.com")]
    pub email: String,
    #[validate(length(
        min = 6,
        max = 30,
        message = "Password must be between 6 and 30 characters"
    ))]
    #[schema(
        format = "password",
        min_length = 6,
        max_length = 30,
        example = "password123"
    )]
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
    #[validate(length(
        min = 3,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    #[schema(example = "jane_doe")]
    pub username: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    #[schema(format = "email", example = "jane@example.com")]
    pub email: Option<String>,
    #[validate(length(
        min = 6,
        max = 30,
        message = "Password must be between 6 and 30 characters"
    ))]
    #[schema(format = "password", example = "newpassword123")]
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
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "john_doe")]
    pub username: String,
    #[schema(example = "john@example.com")]
    pub email: String,
    #[schema(example = "2024-01-15T10:30:00.000Z")]
    pub created_at: String,
    #[schema(example = "2024-01-20T14:45:30.000Z")]
    pub updated_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at.to_jiff().to_string(),
            updated_at: user.updated_at.to_jiff().to_string(),
        }
    }
}
