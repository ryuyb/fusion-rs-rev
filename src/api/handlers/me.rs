//! Current user (me) endpoints.
//!
//! Provides endpoints for the authenticated user to access their own information.

use axum::{extract::State, Extension, Json};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::doc::USER_TAG;
use crate::api::dto::UserResponse;
use crate::api::middleware::AuthUser;
use crate::error::AppResult;
use crate::state::AppState;

/// Creates the "me" routes (current authenticated user)
///
/// # Routes
/// - `GET /me` - Get current user's information
///
/// # Authentication
/// All routes require JWT authentication via the auth_middleware
pub fn me_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_me))
}

/// GET /api/me - Get current user information
///
/// Returns the authenticated user's information based on the JWT token.
#[utoipa::path(
    get,
    path = "/",
    tag = USER_TAG,
    responses(
        (status = 200, description = "Current user information", body = UserResponse),
        (status = 401, description = "Unauthorized - invalid or missing token")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
async fn get_me(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<Json<UserResponse>> {
    // Get user from database using the ID from the JWT token
    let user = state.services.users.get_user(auth_user.user_id).await?;
    Ok(Json(UserResponse::from(user)))
}
