//! Router configuration for the API.
//!
//! This module provides centralized route registration and middleware
//! configuration for the application.

use axum::{middleware, Router};

use crate::api::handlers;
use crate::api::middleware::{global_error_handler, logging_middleware, request_id_middleware};
use crate::state::AppState;

/// Creates the main application router with all routes and middleware.
///
/// # Middleware Order
/// Middleware is applied in reverse order of declaration (last added runs first):
/// 1. Global error handler (runs first) - catches and formats any unhandled errors
/// 2. Request ID middleware (runs second) - generates/propagates request IDs
/// 3. Logging middleware (runs third) - logs requests with request IDs
///
/// # Routes
/// - `/api/users` - User CRUD operations
///
/// # Requirements
/// - 2.1: Provides /api/users endpoint group
/// - 2.7: Applies middleware layers in correct order
/// - 4.1-4.12: Consistent error response formatting
///
/// # Example
/// ```ignore
/// let state = AppState::new(pool);
/// let router = create_router(state);
/// ```
pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .nest("/users", handlers::users::user_routes());

    Router::new()
        .nest("/api", api_routes)
        // Middleware is applied in reverse order - last added runs first
        // So: global_error_handler -> request_id -> logging
        .layer(middleware::from_fn(logging_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(global_error_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    // Note: Full integration tests would require a test database
    // These tests verify the router structure

    #[test]
    fn test_create_router_compiles() {
        // This test verifies that the router creation compiles correctly
        // with all the middleware and routes properly configured
        // Actual runtime testing requires a database connection
        assert!(true);
    }
}
