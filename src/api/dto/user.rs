//! User-related DTOs for API requests and responses.

use serde::{Deserialize, Serialize};
use crate::models::{NewUser, UpdateUser, User};

// ============================================================================
// Request DTOs
// ============================================================================

/// Request body for creating a new user.
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
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
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
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
#[derive(Debug, Serialize)]
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
