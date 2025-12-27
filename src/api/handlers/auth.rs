//! Authentication handlers for login and token management.

use axum::{extract::State, http::StatusCode, Json};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::doc::AUTH_TAG;
use crate::api::dto::{
    LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest,
    RegisterResponse,
};
use crate::error::AppResult;
use crate::state::AppState;
use crate::utils::jwt::{generate_token_pair, validate_refresh_token};

/// Creates the authentication routes
///
/// # Routes
/// - `POST /login` - Authenticate user and get tokens
/// - `POST /register` - Register new user and get tokens
/// - `POST /refresh` - Refresh access token using refresh token
pub fn auth_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(login))
        .routes(routes!(register))
        .routes(routes!(refresh_token))
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
    let (access_token, refresh_token) = generate_token_pair(
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

/// POST /api/auth/refresh - Refresh access token
///
/// Validates the refresh token and issues new access and refresh tokens.
#[utoipa::path(
    post,
    path = "/refresh",
    tag = AUTH_TAG,
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Tokens refreshed successfully", body = RefreshTokenResponse),
        (status = 401, description = "Invalid or expired refresh token")
    )
)]
async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> AppResult<Json<RefreshTokenResponse>> {
    // Validate the refresh token
    let claims = validate_refresh_token(&payload.refresh_token, &state.jwt_config.secret)?;

    // Parse user ID from claims
    let user_id: i32 = claims.sub.parse().map_err(|_| crate::error::AppError::Unauthorized {
        message: "Invalid user ID in token".to_string(),
    })?;

    // Verify user still exists
    let user = state.services.users.get_user(user_id).await?;

    // Generate new token pair
    let (access_token, refresh_token) = generate_token_pair(
        user.id,
        user.email.clone(),
        user.username.clone(),
        &state.jwt_config.secret,
        state.jwt_config.access_token_expiration,
        state.jwt_config.refresh_token_expiration,
    )?;

    let response = RefreshTokenResponse {
        access_token,
        refresh_token,
    };

    Ok(Json(response))
}
