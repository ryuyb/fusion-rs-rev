//! Middleware components for request processing.
//!
//! This module contains middleware for logging, request ID tracking,
//! and error handling.

mod error_handler;
mod logging;
mod request_id;

pub use error_handler::{global_error_handler, handle_json_rejection, handle_path_rejection, handle_query_rejection};
pub use logging::logging_middleware;
pub use request_id::{request_id_middleware, RequestId};
