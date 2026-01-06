use super::types::{MpApiResponse, MpData};
use crate::error::{AppError, AppResult};
use crate::external::client::HTTP_CLIENT;
use crate::external::live::platform::LivePlatform;
use crate::external::live::provider::LivePlatformProvider;
use crate::external::live::types::{AnchorInfo, LiveStatus, RoomInfo, RoomStatusInfo};
use async_trait::async_trait;
use std::collections::HashMap;

const MP_API: &str = "https://mp.huya.com/cache.php";

pub struct HuyaLive;

impl HuyaLive {
    pub fn new() -> Self {
        Self
    }

    fn make_error(message: impl Into<String>, source: Option<anyhow::Error>) -> AppError {
        AppError::ExternalApi {
            platform: "huya".into(),
            message: message.into(),
            source,
        }
    }

    fn parse_live_status(data: &MpData) -> LiveStatus {
        let is_live = data.real_live_status.as_deref() == Some("ON")
            && data.live_status.as_deref() == Some("ON");
        let is_replay = data.live_data.introduction.starts_with("【回放】");

        if is_replay {
            LiveStatus::Replay
        } else if is_live {
            LiveStatus::Live
        } else {
            LiveStatus::Offline
        }
    }

    async fn fetch_mp_data(&self, room_id: &str) -> AppResult<MpData> {
        let url = format!(
            "{}?do=profileRoom&m=Live&roomid={}&showSecret=1",
            MP_API, room_id
        );

        let resp = HTTP_CLIENT
            .get(&url)
            .send()
            .await
            .map_err(|e| Self::make_error(format!("request failed: {}", e), Some(e.into())))?
            .error_for_status()
            .map_err(|e| Self::make_error(format!("HTTP error: {}", e), Some(e.into())))?;

        let api_resp: MpApiResponse = resp
            .json()
            .await
            .map_err(|e| Self::make_error(format!("invalid JSON: {}", e), Some(e.into())))?;

        if api_resp.status != 200 {
            return Err(Self::make_error(
                format!("API error: {}", api_resp.message),
                None,
            ));
        }

        api_resp
            .data
            .ok_or_else(|| Self::make_error("no data in response", None))
    }
}

impl Default for HuyaLive {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LivePlatformProvider for HuyaLive {
    fn platform(&self) -> LivePlatform {
        LivePlatform::Huya
    }

    async fn get_room_info(&self, room_id: &str) -> AppResult<RoomInfo> {
        let data = self.fetch_mp_data(room_id).await?;

        Ok(RoomInfo {
            room_id: data
                .profile_info
                .profile_room
                .clone()
                .unwrap_or_else(|| room_id.to_string()),
            uid: data.profile_info.uid.to_string(),
            title: data.live_data.introduction.clone(),
            live_status: Self::parse_live_status(&data),
            online: data.live_data.user_count.unwrap_or(0),
            cover_url: Some(data.live_data.screenshot),
            area_name: data.live_data.game_full_name,
        })
    }

    async fn get_anchor_info(&self, uid: &str) -> AppResult<AnchorInfo> {
        let data = self.fetch_mp_data(uid).await?;

        Ok(AnchorInfo {
            uid: data.profile_info.uid.to_string(),
            name: data.profile_info.nick,
            avatar_url: Some(data.profile_info.avatar180),
            follower_count: data.profile_info.activity_count,
            room_id: data.profile_info.profile_room,
        })
    }

    async fn get_rooms_status_by_uids(
        &self,
        uids: &[&str],
    ) -> AppResult<HashMap<String, RoomStatusInfo>> {
        let mut result = HashMap::new();

        for uid in uids {
            match self.fetch_mp_data(uid).await {
                Ok(data) => {
                    let info = RoomStatusInfo {
                        uid: data.profile_info.uid.to_string(),
                        room_id: data
                            .profile_info
                            .profile_room
                            .clone()
                            .unwrap_or_else(|| uid.to_string()),
                        title: data.live_data.introduction.clone(),
                        live_status: Self::parse_live_status(&data),
                        online: data.live_data.user_count.unwrap_or(0),
                        uname: data.profile_info.nick,
                        face: Some(data.profile_info.avatar180),
                        cover_url: Some(data.live_data.screenshot),
                        area_name: data.live_data.game_full_name,
                    };
                    result.insert(uid.to_string(), info);
                }
                Err(_) => continue,
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_returns_huya() {
        let client = HuyaLive::new();
        assert_eq!(client.platform(), LivePlatform::Huya);
    }

    #[test]
    fn test_default_impl() {
        let _client: HuyaLive = Default::default();
    }

    #[test]
    fn test_make_error_without_source() {
        let err = HuyaLive::make_error("test error", None);
        match err {
            AppError::ExternalApi {
                platform,
                message,
                source,
            } => {
                assert_eq!(platform, "huya");
                assert_eq!(message, "test error");
                assert!(source.is_none());
            }
            _ => panic!("Expected ExternalApi error"),
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_get_room_info_real_api() {
        let client = HuyaLive::new();
        let result = client.get_room_info("660000").await;
        if let Err(e) = &result {
            eprintln!("Error: {}", e);
        }
        assert!(result.is_ok());
        let room = result.unwrap();
        assert!(!room.room_id.is_empty());
        assert!(!room.title.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_get_anchor_info_real_api() {
        let client = HuyaLive::new();
        let result = client.get_anchor_info("660000").await;
        if let Err(e) = &result {
            eprintln!("Error: {}", e);
        }
        assert!(result.is_ok());
        let anchor = result.unwrap();
        assert!(!anchor.name.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_get_rooms_status_by_uids_real_api() {
        let client = HuyaLive::new();
        let result = client.get_rooms_status_by_uids(&["660000", "600000"]).await;
        assert!(result.is_ok());
        let map = result.unwrap();
        assert!(!map.is_empty());
    }
}
