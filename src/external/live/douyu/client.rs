use super::types::{DouyuBetardResponse, DouyuResponse, DouyuRoomData};
use crate::error::{AppError, AppResult};
use crate::external::client::HTTP_CLIENT;
use crate::external::live::platform::LivePlatform;
use crate::external::live::provider::LivePlatformProvider;
use crate::external::live::types::{AnchorInfo, LiveStatus, RoomInfo, RoomStatusInfo};
use async_trait::async_trait;
use std::collections::HashMap;

const ROOM_INFO_API: &str = "https://open.douyucdn.cn/api/RoomApi/room";
const BETARD_API: &str = "https://www.douyu.com/betard";

pub struct DouyuLive;

impl DouyuLive {
    pub fn new() -> Self {
        Self
    }

    fn make_error(message: impl Into<String>, source: Option<anyhow::Error>) -> AppError {
        AppError::ExternalApi {
            platform: "douyu".into(),
            message: message.into(),
            source,
        }
    }

    fn parse_live_status(show_status: i32, video_loop: i32) -> LiveStatus {
        if show_status == 1 && video_loop == 0 {
            LiveStatus::Live
        } else if show_status == 1 && video_loop == 1 {
            LiveStatus::Replay
        } else {
            LiveStatus::Offline
        }
    }

    async fn get_betard_info(&self, room_id: &str) -> AppResult<DouyuBetardResponse> {
        let url = format!("{}/{}", BETARD_API, room_id);
        let resp = HTTP_CLIENT
            .get(&url)
            .header("Referer", "https://www.douyu.com/")
            .send()
            .await
            .map_err(|e| {
                Self::make_error(
                    format!("get_betard_info({}) request failed: {}", room_id, e),
                    Some(e.into()),
                )
            })?
            .error_for_status()
            .map_err(|e| {
                Self::make_error(
                    format!("get_betard_info({}) HTTP error: {}", room_id, e),
                    Some(e.into()),
                )
            })?;

        let data: DouyuBetardResponse = resp.json().await.map_err(|e| {
            Self::make_error(
                format!("get_betard_info({}) invalid JSON: {}", room_id, e),
                Some(e.into()),
            )
        })?;

        Ok(data)
    }

    #[allow(dead_code)]
    async fn get_room_api_info(&self, room_id: &str) -> AppResult<DouyuRoomData> {
        let url = format!("{}/{}", ROOM_INFO_API, room_id);
        let resp = HTTP_CLIENT
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                Self::make_error(
                    format!("get_room_api_info({}) request failed: {}", room_id, e),
                    Some(e.into()),
                )
            })?
            .error_for_status()
            .map_err(|e| {
                Self::make_error(
                    format!("get_room_api_info({}) HTTP error: {}", room_id, e),
                    Some(e.into()),
                )
            })?;

        let data: DouyuResponse<DouyuRoomData> = resp.json().await.map_err(|e| {
            Self::make_error(
                format!("get_room_api_info({}) invalid JSON: {}", room_id, e),
                Some(e.into()),
            )
        })?;

        if data.error != 0 {
            return Err(Self::make_error(
                format!(
                    "get_room_api_info({}) API error code: {}",
                    room_id, data.error
                ),
                None,
            ));
        }

        Ok(data.data)
    }
}

impl Default for DouyuLive {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LivePlatformProvider for DouyuLive {
    fn platform(&self) -> LivePlatform {
        LivePlatform::Douyu
    }

    async fn get_room_info(&self, room_id: &str) -> AppResult<RoomInfo> {
        let betard_info = self.get_betard_info(room_id).await?;
        let room = &betard_info.room;

        Ok(RoomInfo {
            room_id: room.room_id.to_string(),
            uid: room.owner_uid.to_string(),
            title: room.room_name.clone(),
            live_status: Self::parse_live_status(room.show_status, room.video_loop),
            online: room.iol,
            cover_url: if room.room_pic.is_empty() {
                None
            } else {
                Some(room.room_pic.clone())
            },
            area_name: if room.second_lvl_name.is_empty() {
                None
            } else {
                Some(room.second_lvl_name.clone())
            },
        })
    }

    async fn get_anchor_info(&self, uid: &str) -> AppResult<AnchorInfo> {
        let betard_info = self.get_betard_info(uid).await?;
        let room = &betard_info.room;

        Ok(AnchorInfo {
            uid: room.owner_uid.to_string(),
            name: room.owner_name.clone(),
            avatar_url: room.avatar.get_best(),
            follower_count: None,
            room_id: Some(room.room_id.to_string()),
        })
    }

    async fn get_rooms_status_by_uids(
        &self,
        uids: &[&str],
    ) -> AppResult<HashMap<String, RoomStatusInfo>> {
        let mut result = HashMap::new();

        for uid in uids {
            match self.get_betard_info(uid).await {
                Ok(betard_info) => {
                    let room = &betard_info.room;
                    result.insert(
                        uid.to_string(),
                        RoomStatusInfo {
                            uid: room.owner_uid.to_string(),
                            room_id: room.room_id.to_string(),
                            title: room.room_name.clone(),
                            live_status: Self::parse_live_status(room.show_status, room.video_loop),
                            online: room.iol,
                            uname: room.owner_name.clone(),
                            face: room.avatar.get_best(),
                            cover_url: if room.room_pic.is_empty() {
                                None
                            } else {
                                Some(room.room_pic.clone())
                            },
                            area_name: if room.second_lvl_name.is_empty() {
                                None
                            } else {
                                Some(room.second_lvl_name.clone())
                            },
                        },
                    );
                }
                Err(e) => {
                    return Err(Self::make_error(
                        format!("get_rooms_status_by_uids failed for uid {}: {}", uid, e),
                        Some(e.into()),
                    ));
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_returns_douyu() {
        let client = DouyuLive::new();
        assert_eq!(client.platform(), LivePlatform::Douyu);
    }

    #[test]
    fn test_default_impl() {
        let _client: DouyuLive = Default::default();
    }

    #[test]
    fn test_make_error_without_source() {
        let err = DouyuLive::make_error("test error", None);
        match err {
            AppError::ExternalApi {
                platform,
                message,
                source,
            } => {
                assert_eq!(platform, "douyu");
                assert_eq!(message, "test error");
                assert!(source.is_none());
            }
            _ => panic!("Expected ExternalApi error"),
        }
    }

    #[test]
    fn test_parse_live_status() {
        assert_eq!(DouyuLive::parse_live_status(1, 0), LiveStatus::Live);
        assert_eq!(DouyuLive::parse_live_status(1, 1), LiveStatus::Replay);
        assert_eq!(DouyuLive::parse_live_status(0, 0), LiveStatus::Offline);
        assert_eq!(DouyuLive::parse_live_status(2, 0), LiveStatus::Offline);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_get_room_info_real_api() {
        let client = DouyuLive::new();
        let result = client.get_room_info("288016").await;
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let room = result.unwrap();
        assert_eq!(room.room_id, "288016");
        assert!(!room.title.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_get_anchor_info_real_api() {
        let client = DouyuLive::new();
        let result = client.get_anchor_info("288016").await;
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let anchor = result.unwrap();
        assert!(!anchor.name.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_get_rooms_status_by_uids_real_api() {
        let client = DouyuLive::new();
        let result = client.get_rooms_status_by_uids(&["288016", "606118"]).await;
        assert!(result.is_ok());
        let map = result.unwrap();
        assert!(!map.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_get_room_api_info_real_api() {
        let client = DouyuLive::new();
        let result = client.get_room_api_info("71415").await;
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let room = result.unwrap();
        assert_eq!(room.room_id, "71415");
        assert!(!room.room_name.is_empty());
        assert!(!room.owner_name.is_empty());
    }
}
