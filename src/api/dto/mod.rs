//! Data Transfer Objects for API requests and responses.
//!
//! DTOs are organized by domain:
//! - `user` - User-related request/response DTOs
//! - `error` - Common error response DTOs

mod error;
mod user;

pub use error::ErrorResponse;
pub use user::{CreateUserRequest, UpdateUserRequest, UserResponse};
