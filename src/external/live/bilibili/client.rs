use super::types::{BiliAnchorData, BiliResponse, BiliRoomData};
use crate::error::AppError;
use crate::external::client::HTTP_CLIENT;
use crate::external::live::platform::LivePlatform;
use crate::external::live::provider::LivePlatformProvider;
use crate::external::live::types::{AnchorInfo, LiveStatus, RoomInfo};
use async_trait::async_trait;

const ROOM_INFO_API: &str = "https://api.live.bilibili.com/room/v1/Room/get_info";
const ANCHOR_INFO_API: &str = "https://api.live.bilibili.com/live_user/v1/Master/info";

pub struct BilibiliLive;

impl BilibiliLive {
    pub fn new() -> Self {
        Self
    }

    fn make_error(message: impl Into<String>, source: Option<anyhow::Error>) -> AppError {
        AppError::ExternalApi {
            platform: "bilibili".into(),
            message: message.into(),
            source,
        }
    }
}

impl Default for BilibiliLive {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LivePlatformProvider for BilibiliLive {
    fn platform(&self) -> LivePlatform {
        LivePlatform::Bilibili
    }

    async fn get_room_info(&self, room_id: &str) -> crate::error::AppResult<RoomInfo> {
        let url = format!("{}?room_id={}", ROOM_INFO_API, room_id);
        let resp = HTTP_CLIENT
            .get(&url)
            .send()
            .await
            .map_err(|e: reqwest::Error| Self::make_error("request failed", Some(e.into())))?;

        let data: BiliResponse<BiliRoomData> = resp
            .json()
            .await
            .map_err(|e: reqwest::Error| Self::make_error("invalid response", Some(e.into())))?;

        if data.code != 0 {
            return Err(Self::make_error(
                format!("API error code: {}", data.code),
                None,
            ));
        }

        let d = data.data;
        Ok(RoomInfo {
            room_id: d.room_id.to_string(),
            title: d.title,
            live_status: match d.live_status {
                1 => LiveStatus::Live,
                2 => LiveStatus::Replay,
                _ => LiveStatus::Offline,
            },
            online: d.online,
            cover_url: Some(d.user_cover),
            area_name: Some(d.area_name),
        })
    }

    async fn get_anchor_info(&self, uid: &str) -> crate::error::AppResult<AnchorInfo> {
        let url = format!("{}?uid={}", ANCHOR_INFO_API, uid);
        let resp = HTTP_CLIENT
            .get(&url)
            .send()
            .await
            .map_err(|e: reqwest::Error| Self::make_error("request failed", Some(e.into())))?;

        let data: BiliResponse<BiliAnchorData> = resp
            .json()
            .await
            .map_err(|e: reqwest::Error| Self::make_error("invalid response", Some(e.into())))?;

        if data.code != 0 {
            return Err(Self::make_error(
                format!("API error code: {}", data.code),
                None,
            ));
        }

        let d = data.data;
        Ok(AnchorInfo {
            uid: d.info.uid.to_string(),
            name: d.info.uname,
            avatar_url: Some(d.info.face),
            follower_count: Some(d.follower_num),
            room_id: Some(d.room_id.to_string()),
        })
    }
}
