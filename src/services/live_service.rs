//! Live streaming platform service.

use std::collections::HashMap;

use crate::cache::app_cached;
use crate::error::AppResult;
use crate::external::live::{AnchorInfo, LivePlatform, RoomInfo, RoomStatusInfo, get_provider};

/// Service for interacting with live streaming platforms.
#[derive(Clone, Default)]
pub struct LiveService;

impl LiveService {
    pub fn new() -> Self {
        Self
    }

    /// Get room information from a live streaming platform.
    pub async fn get_room_info(
        &self,
        platform: LivePlatform,
        room_id: &str,
    ) -> AppResult<RoomInfo> {
        get_provider(platform).get_room_info(room_id).await
    }

    /// Get anchor information from a live streaming platform.
    #[app_cached(name = "anchor_info", ttl = 3600, key = platform, key = uid)]
    pub async fn get_anchor_info(
        &self,
        platform: LivePlatform,
        uid: &str,
    ) -> AppResult<AnchorInfo> {
        get_provider(platform).get_anchor_info(uid).await
    }

    /// Get room status for multiple anchors by their UIDs.
    pub async fn get_rooms_status_by_uids(
        &self,
        platform: LivePlatform,
        uids: &[&str],
    ) -> AppResult<HashMap<String, RoomStatusInfo>> {
        get_provider(platform).get_rooms_status_by_uids(uids).await
    }
}
