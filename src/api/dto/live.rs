//! Live platform DTOs for API requests and responses.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::external::live::{AnchorInfo, LiveStatus, RoomInfo, RoomStatusInfo};

/// Normalized live status for responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum LiveStatusResponse {
    Offline,
    Live,
    Replay,
}

impl From<LiveStatus> for LiveStatusResponse {
    fn from(status: LiveStatus) -> Self {
        match status {
            LiveStatus::Offline => Self::Offline,
            LiveStatus::Live => Self::Live,
            LiveStatus::Replay => Self::Replay,
        }
    }
}

/// Live room information response.
#[derive(Debug, Serialize, ToSchema)]
pub struct LiveRoomResponse {
    pub room_id: String,
    pub uid: String,
    pub title: String,
    pub live_status: LiveStatusResponse,
    pub online: u64,
    pub cover_url: Option<String>,
    pub area_name: Option<String>,
}

impl From<RoomInfo> for LiveRoomResponse {
    fn from(info: RoomInfo) -> Self {
        Self {
            room_id: info.room_id,
            uid: info.uid,
            title: info.title,
            live_status: info.live_status.into(),
            online: info.online,
            cover_url: info.cover_url,
            area_name: info.area_name,
        }
    }
}

/// Anchor information response.
#[derive(Debug, Serialize, ToSchema)]
pub struct LiveAnchorResponse {
    pub uid: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub follower_count: Option<u64>,
    pub room_id: Option<String>,
}

impl From<AnchorInfo> for LiveAnchorResponse {
    fn from(info: AnchorInfo) -> Self {
        Self {
            uid: info.uid,
            name: info.name,
            avatar_url: info.avatar_url,
            follower_count: info.follower_count,
            room_id: info.room_id,
        }
    }
}

/// Live room status response for batch queries.
#[derive(Debug, Serialize, ToSchema)]
pub struct LiveRoomStatusResponse {
    pub uid: String,
    pub room_id: String,
    pub title: String,
    pub live_status: LiveStatusResponse,
    pub online: u64,
    pub uname: String,
    pub face: Option<String>,
    pub cover_url: Option<String>,
    pub area_name: Option<String>,
}

impl From<RoomStatusInfo> for LiveRoomStatusResponse {
    fn from(info: RoomStatusInfo) -> Self {
        Self {
            uid: info.uid,
            room_id: info.room_id,
            title: info.title,
            live_status: info.live_status.into(),
            online: info.online,
            uname: info.uname,
            face: info.face,
            cover_url: info.cover_url,
            area_name: info.area_name,
        }
    }
}

/// Request body for batch room status queries.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LiveStatusBatchRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "uids must contain between 1 and 50 entries"
    ))]
    pub uids: Vec<String>,
}
