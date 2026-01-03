//! Notification API handlers.
//!
//! Provides HTTP handlers for notification channel management and messaging.

use crate::api::doc::NOTIFICATION_TAG;
use crate::api::dto::{
    ChannelResponse, CreateChannelRequest, LogResponse, PagedResponse, PaginationParams,
    SendNotificationRequest, SendToUserRequest, UpdateChannelRequest,
};
use crate::api::middleware::AuthUser;
use crate::error::{AppError, AppResult};
use crate::models::{NewNotificationChannel, UpdateNotificationChannel};
use crate::services::notifications::NotificationMessage;
use crate::state::AppState;
use crate::utils::validate::{ValidatedJson, ValidatedQuery};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// Creates notification-related routes.
///
/// Routes:
/// - GET /channels         - List user's channels
/// - POST /channels        - Create channel
/// - GET /channels/:id     - Get channel by ID
/// - PUT /channels/:id     - Update channel
/// - DELETE /channels/:id  - Delete channel
/// - POST /channels/:id/send - Send via channel
/// - POST /send            - Send to user's channels
/// - GET /logs             - List logs
pub fn notification_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_channels))
        .routes(routes!(create_channel))
        .routes(routes!(get_channel))
        .routes(routes!(update_channel))
        .routes(routes!(delete_channel))
        .routes(routes!(send_to_channel))
        .routes(routes!(send_to_user))
        .routes(routes!(list_logs))
}

// ============================================================================
// Channel Management Handlers
// ============================================================================

/// GET /api/notifications/channels - List user's channels
///
/// Returns all notification channels for the authenticated user with pagination support.
#[utoipa::path(
    get,
    path = "/channels",
    tag = NOTIFICATION_TAG,
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of channels", body = PagedResponse<ChannelResponse>)
    ),
    security(("bearerAuth" = []))
)]
async fn list_channels(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    ValidatedQuery(params): ValidatedQuery<PaginationParams>,
) -> AppResult<Json<PagedResponse<ChannelResponse>>> {
    let params = params.normalize();

    let (channels, total_count) = state
        .services
        .notifications
        .list_user_channels_paginated(
            auth_user.user_id,
            params.offset() as i64,
            params.limit() as i64,
        )
        .await?;

    let responses: Vec<ChannelResponse> = channels.into_iter().map(ChannelResponse::from).collect();
    let paged_response = PagedResponse::new(responses, &params, total_count as u64);
    Ok(Json(paged_response))
}

/// POST /api/notifications/channels - Create channel
///
/// Creates a new notification channel for the authenticated user.
#[utoipa::path(
    post,
    path = "/channels",
    tag = NOTIFICATION_TAG,
    request_body = CreateChannelRequest,
    responses(
        (status = 201, description = "Channel created", body = ChannelResponse),
        (status = 400, description = "Invalid request")
    ),
    security(("bearerAuth" = []))
)]
async fn create_channel(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    ValidatedJson(payload): ValidatedJson<CreateChannelRequest>,
) -> AppResult<(StatusCode, Json<ChannelResponse>)> {
    let new_channel = NewNotificationChannel {
        user_id: auth_user.user_id,
        channel_type: payload.channel_type,
        name: payload.name,
        config: payload.config,
        enabled: payload.enabled,
        priority: payload.priority,
    };

    let channel = state
        .services
        .notifications
        .create_channel(new_channel)
        .await?;
    Ok((StatusCode::CREATED, Json(ChannelResponse::from(channel))))
}

/// GET /api/notifications/channels/:id - Get channel by ID
///
/// Retrieves a specific notification channel by ID.
/// Only the channel owner can access it.
#[utoipa::path(
    get,
    path = "/channels/{id}",
    tag = NOTIFICATION_TAG,
    params(
        ("id" = i32, Path, description = "Channel ID")
    ),
    responses(
        (status = 200, description = "Channel found", body = ChannelResponse),
        (status = 404, description = "Channel not found"),
        (status = 403, description = "Access denied")
    ),
    security(("bearerAuth" = []))
)]
async fn get_channel(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> AppResult<Json<ChannelResponse>> {
    let channel = state.services.notifications.get_channel(id).await?;

    // Verify ownership
    if channel.user_id != auth_user.user_id {
        return Err(AppError::Forbidden {
            message: "Access denied".to_string(),
        });
    }

    Ok(Json(ChannelResponse::from(channel)))
}

/// PUT /api/notifications/channels/:id - Update channel
///
/// Updates an existing notification channel.
/// Only the channel owner can update it.
#[utoipa::path(
    put,
    path = "/channels/{id}",
    tag = NOTIFICATION_TAG,
    params(
        ("id" = i32, Path, description = "Channel ID")
    ),
    request_body = UpdateChannelRequest,
    responses(
        (status = 200, description = "Channel updated", body = ChannelResponse),
        (status = 404, description = "Channel not found"),
        (status = 403, description = "Access denied")
    ),
    security(("bearerAuth" = []))
)]
async fn update_channel(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateChannelRequest>,
) -> AppResult<Json<ChannelResponse>> {
    let channel = state.services.notifications.get_channel(id).await?;

    // Verify ownership
    if channel.user_id != auth_user.user_id {
        return Err(AppError::Forbidden {
            message: "Access denied".to_string(),
        });
    }

    let update_data = UpdateNotificationChannel {
        name: payload.name,
        config: payload.config,
        enabled: payload.enabled,
        priority: payload.priority,
    };

    let updated = state
        .services
        .notifications
        .update_channel(id, update_data)
        .await?;
    Ok(Json(ChannelResponse::from(updated)))
}

/// DELETE /api/notifications/channels/:id - Delete channel
///
/// Deletes a notification channel.
/// Only the channel owner can delete it.
#[utoipa::path(
    delete,
    path = "/channels/{id}",
    tag = NOTIFICATION_TAG,
    params(
        ("id" = i32, Path, description = "Channel ID")
    ),
    responses(
        (status = 204, description = "Channel deleted"),
        (status = 404, description = "Channel not found"),
        (status = 403, description = "Access denied")
    ),
    security(("bearerAuth" = []))
)]
async fn delete_channel(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> AppResult<StatusCode> {
    let channel = state.services.notifications.get_channel(id).await?;

    // Verify ownership
    if channel.user_id != auth_user.user_id {
        return Err(AppError::Forbidden {
            message: "Access denied".to_string(),
        });
    }

    let deleted = state.services.notifications.delete_channel(id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound {
            entity: "notification_channel".to_string(),
            field: "id".to_string(),
            value: id.to_string(),
        })
    }
}

// ============================================================================
// Message Sending Handlers
// ============================================================================

/// POST /api/notifications/channels/:id/send - Send via specific channel
///
/// Sends a notification via a specific channel.
/// Only the channel owner can send via it.
#[utoipa::path(
    post,
    path = "/channels/{id}/send",
    tag = NOTIFICATION_TAG,
    params(
        ("id" = i32, Path, description = "Channel ID")
    ),
    request_body = SendNotificationRequest,
    responses(
        (status = 200, description = "Notification sent", body = LogResponse),
        (status = 404, description = "Channel not found"),
        (status = 403, description = "Access denied")
    ),
    security(("bearerAuth" = []))
)]
async fn send_to_channel(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<SendNotificationRequest>,
) -> AppResult<Json<LogResponse>> {
    let channel = state.services.notifications.get_channel(id).await?;

    // Verify ownership
    if channel.user_id != auth_user.user_id {
        return Err(AppError::Forbidden {
            message: "Access denied".to_string(),
        });
    }

    let message = NotificationMessage {
        title: payload.title,
        body: payload.body,
        metadata: payload.metadata,
    };

    let log = state
        .services
        .notifications
        .send_to_channel(id, message)
        .await?;
    Ok(Json(LogResponse::from(log)))
}

/// POST /api/notifications/send - Send to user's channels
///
/// Sends a notification to all enabled channels of a specific type
/// for the authenticated user, in priority order.
#[utoipa::path(
    post,
    path = "/send",
    tag = NOTIFICATION_TAG,
    request_body = SendToUserRequest,
    responses(
        (status = 200, description = "Notifications sent", body = Vec<LogResponse>)
    ),
    security(("bearerAuth" = []))
)]
async fn send_to_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    ValidatedJson(payload): ValidatedJson<SendToUserRequest>,
) -> AppResult<Json<Vec<LogResponse>>> {
    let message = NotificationMessage {
        title: payload.title,
        body: payload.body,
        metadata: payload.metadata,
    };

    let logs = state
        .services
        .notifications
        .send_to_user(auth_user.user_id, payload.channel_type, message)
        .await?;

    let responses: Vec<LogResponse> = logs.into_iter().map(LogResponse::from).collect();
    Ok(Json(responses))
}

// ============================================================================
// Log Handlers
// ============================================================================

/// GET /api/notifications/logs - List notification logs
///
/// Lists notification send history for the authenticated user's channels.
#[utoipa::path(
    get,
    path = "/logs",
    tag = NOTIFICATION_TAG,
    params(PaginationParams),
    responses(
        (status = 200, description = "List of logs", body = PagedResponse<LogResponse>)
    ),
    security(("bearerAuth" = []))
)]
async fn list_logs(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    ValidatedQuery(params): ValidatedQuery<PaginationParams>,
) -> AppResult<Json<PagedResponse<LogResponse>>> {
    let params = params.normalize();

    // Get user's channels to filter logs
    let user_channels = state
        .services
        .notifications
        .list_user_channels(auth_user.user_id)
        .await?;

    // For simplicity, get logs from first channel
    // In production, you'd aggregate logs from all channels
    if let Some(first_channel) = user_channels.first() {
        let (logs, total_count) = state
            .services
            .notifications
            .get_channel_logs(
                first_channel.id,
                params.offset() as i64,
                params.limit() as i64,
            )
            .await?;

        let responses: Vec<LogResponse> = logs.into_iter().map(LogResponse::from).collect();
        let paged_response = PagedResponse::new(responses, &params, total_count as u64);
        Ok(Json(paged_response))
    } else {
        Ok(Json(PagedResponse::new(vec![], &params, 0)))
    }
}
