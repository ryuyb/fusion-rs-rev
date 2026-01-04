use super::platform::LivePlatform;
use super::types::{AnchorInfo, RoomInfo, RoomStatusInfo};
use crate::error::AppResult;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait LivePlatformProvider: Send + Sync {
    fn platform(&self) -> LivePlatform;
    async fn get_room_info(&self, room_id: &str) -> AppResult<RoomInfo>;
    async fn get_anchor_info(&self, uid: &str) -> AppResult<AnchorInfo>;
    async fn get_rooms_status_by_uids(
        &self,
        uids: &[&str],
    ) -> AppResult<HashMap<String, RoomStatusInfo>>;
}
