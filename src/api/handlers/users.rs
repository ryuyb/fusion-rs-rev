//! User CRUD request handlers.
//!
//! Provides HTTP handlers for user management operations.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

use crate::api::dto::{CreateUserRequest, UpdateUserRequest, UserResponse};
use crate::error::AppError;
use crate::state::AppState;

/// Creates user-related routes.
///
/// Routes:
/// - GET /        - List all users
/// - POST /       - Create a new user
/// - GET /:id     - Get user by ID
/// - PUT /:id     - Update user by ID
/// - DELETE /:id  - Delete user by ID
pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
}

/// GET /api/users - List all users
///
/// Returns a JSON array of all users.
async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let users = state.services.users.list_users().await?;
    let responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/users/:id - Get user by ID
///
/// Returns the user with the specified ID or 404 if not found.
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.services.users.get_user(id).await?;
    Ok(Json(UserResponse::from(user)))
}

/// POST /api/users - Create new user
///
/// Creates a new user from the JSON request body.
/// Returns 201 Created with the created user data.
async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let new_user = payload.into_new_user();
    let user = state.services.users.create_user(new_user).await?;
    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

/// PUT /api/users/:id - Update user
///
/// Updates the user with the specified ID.
/// Returns the updated user data.
async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let update_data = payload.into_update_user();
    let user = state.services.users.update_user(id, update_data).await?;
    Ok(Json(UserResponse::from(user)))
}

/// DELETE /api/users/:id - Delete user
///
/// Deletes the user with the specified ID.
/// Returns 204 No Content on success.
async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let deleted = state.services.users.delete_user(id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}
