//! Data Transfer Objects for API requests and responses.
//!
//! DTOs are organized by domain:
//! - `auth` - Authentication-related request/response DTOs
//! - `user` - User-related request/response DTOs
//! - `notification` - Notification-related request/response DTOs
//! - `job` - Job scheduling-related request/response DTOs
//! - `health` - Health check-related response DTOs
//! - `live` - Live platform integration DTOs
//! - `error` - Common error response DTOs
//! - `pagination` - Pagination-related DTOs

mod auth;
mod error;
mod health;
mod job;
mod live;
mod notification;
mod pagination;
mod user;

pub use auth::{
    LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest,
    RegisterResponse,
};
pub use error::ErrorResponse;
pub use health::{ComponentHealth, HealthResponse, HealthStatus};
pub use job::{CreateJobRequest, JobExecutionResponse, JobResponse, UpdateJobRequest};
pub use live::{
    LiveAnchorResponse, LiveRoomResponse, LiveRoomStatusResponse, LiveStatusBatchRequest,
    LiveStatusResponse,
};
pub use notification::{
    ChannelResponse, CreateChannelRequest, LogResponse, SendNotificationRequest, SendToUserRequest,
    UpdateChannelRequest,
};
pub use pagination::{PagedResponse, PaginationParams};
pub use user::{CreateUserRequest, UpdateUserRequest, UserResponse};
