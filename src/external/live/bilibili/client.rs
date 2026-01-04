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
            .map_err(|e: reqwest::Error| {
                Self::make_error(
                    format!("get_room_info({}) request failed: {}", room_id, e),
                    Some(e.into()),
                )
            })?
            .error_for_status()
            .map_err(|e: reqwest::Error| {
                Self::make_error(
                    format!("get_room_info({}) HTTP error: {}", room_id, e),
                    Some(e.into()),
                )
            })?;

        let data: BiliResponse<BiliRoomData> = resp.json().await.map_err(|e: reqwest::Error| {
            Self::make_error(
                format!("get_room_info({}) invalid JSON: {}", room_id, e),
                Some(e.into()),
            )
        })?;

        if data.code != 0 {
            return Err(Self::make_error(
                format!("get_room_info({}) API error code: {}", room_id, data.code),
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
            .map_err(|e: reqwest::Error| {
                Self::make_error(
                    format!("get_anchor_info({}) request failed: {}", uid, e),
                    Some(e.into()),
                )
            })?
            .error_for_status()
            .map_err(|e: reqwest::Error| {
                Self::make_error(
                    format!("get_anchor_info({}) HTTP error: {}", uid, e),
                    Some(e.into()),
                )
            })?;

        let data: BiliResponse<BiliAnchorData> =
            resp.json().await.map_err(|e: reqwest::Error| {
                Self::make_error(
                    format!("get_anchor_info({}) invalid JSON: {}", uid, e),
                    Some(e.into()),
                )
            })?;

        if data.code != 0 {
            return Err(Self::make_error(
                format!("get_anchor_info({}) API error code: {}", uid, data.code),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_returns_bilibili() {
        let client = BilibiliLive::new();
        assert_eq!(client.platform(), LivePlatform::Bilibili);
    }

    #[test]
    fn test_default_impl() {
        let _client: BilibiliLive = Default::default();
    }

    #[test]
    fn test_make_error_without_source() {
        let err = BilibiliLive::make_error("test error", None);
        match err {
            AppError::ExternalApi {
                platform,
                message,
                source,
            } => {
                assert_eq!(platform, "bilibili");
                assert_eq!(message, "test error");
                assert!(source.is_none());
            }
            _ => panic!("Expected ExternalApi error"),
        }
    }

    #[test]
    fn test_make_error_with_source() {
        let source_err = anyhow::anyhow!("source error");
        let err = BilibiliLive::make_error("test error", Some(source_err));
        match err {
            AppError::ExternalApi {
                platform,
                message,
                source,
            } => {
                assert_eq!(platform, "bilibili");
                assert_eq!(message, "test error");
                assert!(source.is_some());
            }
            _ => panic!("Expected ExternalApi error"),
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_get_room_info_real_api() {
        let client = BilibiliLive::new();
        let result = client.get_room_info("1").await;
        assert!(result.is_ok());
        let room = result.unwrap();
        assert_eq!(room.room_id, "5440");
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_get_anchor_info_real_api() {
        let client = BilibiliLive::new();
        let result = client.get_anchor_info("9617619").await;
        assert!(result.is_ok());
        let anchor = result.unwrap();
        assert_eq!(anchor.uid, "9617619");
        assert!(!anchor.name.is_empty());
    }
}
