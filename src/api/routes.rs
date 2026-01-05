//! Router configuration for the API.
//!
//! This module provides centralized route registration and middleware
//! configuration for the application.

use crate::api::doc::ApiDoc;
use crate::api::handlers;
use crate::api::middleware::{
    auth_middleware, global_error_handler, logging_middleware, request_id_middleware,
};
use crate::state::AppState;
use axum::{Router, middleware};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

/// Creates the main application router with all routes and middleware.
///
/// # Middleware Order
/// Middleware is applied in reverse order of declaration (last added runs first):
/// 1. Global error handler (runs first) - catches and formats any unhandled errors
/// 2. Request ID middleware (runs second) - generates/propagates request IDs
/// 3. Logging middleware (runs third) - logs requests with request IDs
///
/// # Routes
/// - `/health` - Health check endpoints
/// - `/api/auth/login` - Login endpoint - public
/// - `/api/auth/register` - Register endpoint - public
/// - `/api/auth/refresh` - Refresh token endpoint - public
/// - `/api/live` - Live platform endpoints - public
/// - `/api/me` - Current user endpoint - requires authentication
/// - `/api/users` - User CRUD operations - requires authentication
/// - `/api/jobs` - Job management endpoints - requires authentication
/// - `/api/notifications` - Notification endpoints - requires authentication
///
/// # Requirements
/// - 2.1: Provides /api/users endpoint group
/// - 2.7: Applies middleware layers in correct order
/// - 4.1-4.12: Consistent error response formatting
///
/// # Example
/// ```ignore
/// let state = AppState::new(pool, jwt_config);
/// let router = create_router(state);
/// ```
pub fn create_router(state: AppState) -> Router {
    // Public auth routes
    let public_auth_routes = OpenApiRouter::new().nest("/auth", handlers::auth::auth_routes());
    let live_routes = OpenApiRouter::new().nest("/live", handlers::live::live_routes());

    // Protected routes (authentication required)
    let protected_routes = OpenApiRouter::new()
        .nest("/me", handlers::me::me_routes())
        .nest("/users", handlers::users::user_routes())
        .nest(
            "/notifications",
            handlers::notifications::notification_routes(),
        )
        .nest("/jobs", handlers::jobs::job_routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let api_routes = OpenApiRouter::new()
        .merge(public_auth_routes)
        .merge(live_routes)
        .merge(protected_routes);

    let (router, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", api_routes)
        .merge(handlers::health::health_routes())
        .split_for_parts();

    router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi.clone()))
        // Middleware is applied in reverse order - last added runs first
        // So: global_error_handler -> request_id -> logging
        .layer(middleware::from_fn(logging_middleware))
        .layer(middleware::from_fn(global_error_handler))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
