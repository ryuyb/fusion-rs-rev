//! Authentication handlers for login and token management.

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use crate::api::doc::AUTH_TAG;
use crate::error::AppResult;
use crate::state::AppState;

/// Login request payload
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// User's email address
    #[schema(example = "user@example.com")]
    pub email: String,
    /// User's password (plain text)
    #[schema(example = "password123")]
    pub password: String,
}

/// Register request payload
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    /// Username (unique)
    #[schema(example = "john_doe")]
    pub username: String,
    /// User's email address (unique)
    #[schema(example = "user@example.com")]
    pub email: String,
    /// User's password (plain text, will be hashed)
    #[schema(example = "password123")]
    pub password: String,
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

/// Creates the authentication routes
///
/// # Routes
/// - `POST /login` - Authenticate user and get tokens
/// - `POST /register` - Register new user and get tokens
pub fn auth_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(login))
        .routes(routes!(register))
}

/// POST /api/auth/login - Authenticate user
///
/// Authenticates a user with email and password, returns JWT tokens.
#[utoipa::path(
    post,
    path = "/login",
    tag = AUTH_TAG,
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<(StatusCode, Json<LoginResponse>)> {
    // Authenticate user and generate tokens using JWT config from state
    let (user, access_token, refresh_token) = state
        .services
        .users
        .authenticate(
            &payload.email,
            &payload.password,
            &state.jwt_config.secret,
            state.jwt_config.access_token_expiration,
            state.jwt_config.refresh_token_expiration,
        )
        .await?;

    let response = LoginResponse {
        user: user.into(),
        access_token,
        refresh_token,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// POST /api/auth/register - Register new user
///
/// Creates a new user account and returns JWT tokens.
#[utoipa::path(
    post,
    path = "/register",
    tag = AUTH_TAG,
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Invalid request data"),
        (status = 409, description = "User already exists")
    )
)]
async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<RegisterResponse>)> {
    // Create new user (password will be hashed automatically by the service)
    let new_user = crate::models::NewUser {
        username: payload.username,
        email: payload.email.clone(),
        password: payload.password.clone(),
    };

    let user = state.services.users.create_user(new_user).await?;

    // Generate tokens for the newly registered user
    let (access_token, refresh_token) = crate::utils::jwt::generate_token_pair(
        user.id,
        user.email.clone(),
        user.username.clone(),
        &state.jwt_config.secret,
        state.jwt_config.access_token_expiration,
        state.jwt_config.refresh_token_expiration,
    )?;

    let response = RegisterResponse {
        user: user.into(),
        access_token,
        refresh_token,
    };

    Ok((StatusCode::CREATED, Json(response)))
}
