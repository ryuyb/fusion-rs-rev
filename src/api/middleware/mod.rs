//! Middleware components for request processing.
//!
//! This module contains middleware for logging, request ID tracking,
//! and error handling.

mod error_handler;
mod logging;
mod request_id;

pub use error_handler::{error_to_code, error_to_status_code};
pub use logging::logging_middleware;
pub use request_id::{request_id_middleware, RequestId, REQUEST_ID_HEADER};
