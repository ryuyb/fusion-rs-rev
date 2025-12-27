//! Authentication-related Data Transfer Objects

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Login request payload
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    /// User's email address
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "user@example.com", format = "email")]
    pub email: String,
    /// User's password (plain text)
    #[validate(length(min = 6, max = 30, message = "Password must be between 6 and 30 characters"))]
    #[schema(example = "password123", format = "password", min_length = 6, max_length = 30)]
    pub password: String,
}

/// Register request payload
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct RegisterRequest {
    /// Username (unique)
    #[validate(length(min = 3, max = 20, message = "Username must be between 3 and 20 characters"))]
    #[schema(example = "john_doe", min_length = 3, max_length = 20)]
    pub username: String,
    /// User's email address (unique)
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "user@example.com", format = "email")]
    pub email: String,
    /// User's password (plain text, will be hashed)
    #[validate(length(min = 6, max = 30, message = "Password must be between 6 and 30 characters"))]
    #[schema(example = "password123", format = "password", min_length = 6, max_length = 30)]
    pub password: String,
}

/// Refresh token request payload
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct RefreshTokenRequest {
    /// Refresh token
    #[validate(length(min = 1, message = "Refresh token cannot be empty"))]
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGc...")]
    pub refresh_token: String,
}

/// Login response with user info and tokens
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    /// User information
    pub user: UserInfo,
    /// Access token (short-lived)
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGc...")]
    pub access_token: String,
    /// Refresh token (long-lived)
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGc...")]
    pub refresh_token: String,
}

/// Register response with user info and tokens
#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterResponse {
    /// User information
    pub user: UserInfo,
    /// Access token (short-lived)
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGc...")]
    pub access_token: String,
    /// Refresh token (long-lived)
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGc...")]
    pub refresh_token: String,
}

/// Refresh token response with new tokens
#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshTokenResponse {
    /// New access token (short-lived)
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGc...")]
    pub access_token: String,
    /// New refresh token (long-lived)
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGc...")]
    pub refresh_token: String,
}

/// User information in response
#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    /// User ID
    #[schema(example = 1)]
    pub id: i32,
    /// Username
    #[schema(example = "john_doe")]
    pub username: String,
    /// Email address
    #[schema(example = "user@example.com")]
    pub email: String,
}

impl From<crate::models::User> for UserInfo {
    fn from(user: crate::models::User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
        }
    }
}
