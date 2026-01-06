//! Live platform handlers for fetching room and anchor information.

use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::doc::LIVE_TAG;
use crate::api::dto::{
    LiveAnchorResponse, LiveRoomResponse, LiveRoomStatusResponse, LiveStatusBatchRequest,
};
use crate::error::AppResult;
use crate::external::live::LivePlatform;
use crate::state::AppState;
use crate::utils::validate::ValidatedJson;

/// Register live platform routes.
pub fn live_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_room_info))
        .routes(routes!(get_anchor_info))
        .routes(routes!(get_rooms_status_by_uids))
}

/// GET /api/live/{platform}/rooms/{room_id} - Get live room information.
#[utoipa::path(
    get,
    path = "/{platform}/rooms/{room_id}",
    tag = LIVE_TAG,
    params(
        ("platform" = LivePlatform, Path, description = "Live platform"),
        ("room_id" = String, Path, description = "Room ID or URL")
    ),
    responses(
        (status = 200, description = "Live room information", body = LiveRoomResponse)
    )
)]
async fn get_room_info(
    State(state): State<AppState>,
    Path((platform, room_id)): Path<(LivePlatform, String)>,
) -> AppResult<Json<LiveRoomResponse>> {
    let info = state
        .services
        .live
        .get_room_info(platform, &room_id)
        .await?;
    Ok(Json(info.into()))
}

/// GET /api/live/{platform}/anchors/{uid} - Get anchor information.
#[utoipa::path(
    get,
    path = "/{platform}/anchors/{uid}",
    tag = LIVE_TAG,
    params(
        ("platform" = LivePlatform, Path, description = "Live platform"),
        ("uid" = String, Path, description = "Anchor UID")
    ),
    responses(
        (status = 200, description = "Anchor information", body = LiveAnchorResponse)
    )
)]
async fn get_anchor_info(
    State(state): State<AppState>,
    Path((platform, uid)): Path<(LivePlatform, String)>,
) -> AppResult<Json<LiveAnchorResponse>> {
    let info = state.services.live.get_anchor_info(platform, &uid).await?;
    Ok(Json(info.into()))
}

/// POST /api/live/{platform}/anchors/status - Get room status for anchors by UID.
#[utoipa::path(
    post,
    path = "/{platform}/anchors/status",
    tag = LIVE_TAG,
    request_body = LiveStatusBatchRequest,
    params(
        ("platform" = LivePlatform, Path, description = "Live platform")
    ),
    responses(
        (status = 200, description = "Room status map keyed by uid", body = HashMap<String, LiveRoomStatusResponse>)
    )
)]
async fn get_rooms_status_by_uids(
    State(state): State<AppState>,
    Path(platform): Path<LivePlatform>,
    ValidatedJson(req): ValidatedJson<LiveStatusBatchRequest>,
) -> AppResult<Json<HashMap<String, LiveRoomStatusResponse>>> {
    let uid_refs: Vec<&str> = req.uids.iter().map(String::as_str).collect();
    let status_map = state
        .services
        .live
        .get_rooms_status_by_uids(platform, &uid_refs)
        .await?
        .into_iter()
        .map(|(uid, status)| (uid, LiveRoomStatusResponse::from(status)))
        .collect();

    Ok(Json(status_map))
}
