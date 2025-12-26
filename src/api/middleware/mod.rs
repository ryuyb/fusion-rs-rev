//! Middleware components for request processing.
//!
//! This module contains middleware for logging, request ID tracking,
//! error handling, and authentication.

mod auth;
mod error_handler;
mod logging;
mod request_id;

pub use auth::{auth_middleware, optional_auth_middleware, AuthUser};
pub use error_handler::global_error_handler;
pub use logging::logging_middleware;
pub use request_id::{request_id_middleware, RequestId};
