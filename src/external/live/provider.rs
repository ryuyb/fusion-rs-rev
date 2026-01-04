use super::platform::LivePlatform;
use super::types::{AnchorInfo, RoomInfo};
use crate::error::AppResult;
use async_trait::async_trait;

#[async_trait]
pub trait LivePlatformProvider: Send + Sync {
    fn platform(&self) -> LivePlatform;
    async fn get_room_info(&self, room_id: &str) -> AppResult<RoomInfo>;
    async fn get_anchor_info(&self, uid: &str) -> AppResult<AnchorInfo>;
}
