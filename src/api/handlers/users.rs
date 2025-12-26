//! User CRUD request handlers.
//!
//! Provides HTTP handlers for user management operations.

use crate::api::doc::USER_TAG;
use crate::api::dto::{CreateUserRequest, UpdateUserRequest, UserResponse};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// Creates user-related routes.
///
/// Routes:
/// - GET /        - List all users
/// - POST /       - Create a new user
/// - GET /:id     - Get user by ID
/// - PUT /:id     - Update user by ID
/// - DELETE /:id  - Delete user by ID
pub fn user_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_users))
        .routes(routes!(create_user))
        .routes(routes!(get_user))
        .routes(routes!(update_user))
        .routes(routes!(delete_user))
}

/// GET /api/users - List all users
///
/// Returns a JSON array of all users.
#[utoipa::path(
    get,
    path = "/",
    tag = USER_TAG,
    responses(
        (status = 200, description = "List users by page", body = Vec<UserResponse>)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn list_users(State(state): State<AppState>) -> AppResult<Json<Vec<UserResponse>>> {
    let users = state.services.users.list_users().await?;
    let responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/users/:id - Get user by ID
///
/// Returns the user with the specified ID or 404 if not found.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = USER_TAG,
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<UserResponse>> {
    let user = state.services.users.get_user(id).await?;
    Ok(Json(UserResponse::from(user)))
}

/// POST /api/users - Create new user
///
/// Creates a new user from the JSON request body.
/// Returns 201 Created with the created user data.
#[utoipa::path(
    post,
    path = "/",
    tag = USER_TAG,
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Invalid request data")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    let new_user = payload.into_new_user();
    let user = state.services.users.create_user(new_user).await?;
    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

/// PUT /api/users/:id - Update user
///
/// Updates the user with the specified ID.
/// Returns the updated user data.
#[utoipa::path(
    put,
    path = "/{id}",
    tag = USER_TAG,
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 404, description = "User not found"),
        (status = 400, description = "Invalid request data")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    let update_data = payload.into_update_user();
    let user = state.services.users.update_user(id, update_data).await?;
    Ok(Json(UserResponse::from(user)))
}

/// DELETE /api/users/:id - Delete user
///
/// Deletes the user with the specified ID.
/// Returns 204 No Content on success.
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = USER_TAG,
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn delete_user(State(state): State<AppState>, Path(id): Path<i32>) -> AppResult<StatusCode> {
    let deleted = state.services.users.delete_user(id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound {
            entity: "user".to_string(),
            field: "id".to_string(),
            value: id.to_string(),
        })
    }
}
