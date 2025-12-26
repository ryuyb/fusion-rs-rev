//! Data Transfer Objects for API requests and responses.
//!
//! DTOs are organized by domain:
//! - `user` - User-related request/response DTOs
//! - `error` - Common error response DTOs
//! - `pagination` - Pagination-related DTOs

mod error;
mod pagination;
mod user;

pub use error::ErrorResponse;
pub use pagination::{PagedResponse, PaginationParams};
pub use user::{CreateUserRequest, UpdateUserRequest, UserResponse};
