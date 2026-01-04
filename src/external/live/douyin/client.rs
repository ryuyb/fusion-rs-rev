use super::abogus::ABogus;
use super::sign::get_ac_signature;
use super::types::DouyinEnterRoomResp;
use crate::error::AppError;
use crate::external::client::HTTP_CLIENT;
use crate::external::live::platform::LivePlatform;
use crate::external::live::provider::LivePlatformProvider;
use crate::external::live::types::{AnchorInfo, LiveStatus, RoomInfo, RoomStatusInfo};
use crate::external::user_agent::{Browser, Platform, USER_AGENT_POOL};
use async_trait::async_trait;
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

const ENTER_ROOM_API: &str = "https://live.douyin.com/webcast/room/web/enter/";
const LIVE_HOME_URL: &str = "https://live.douyin.com/";

static COOKIE_CACHE: LazyLock<RwLock<Option<CookieCache>>> = LazyLock::new(|| RwLock::new(None));

struct CookieCache {
    cookies: String,
    timestamp: u64,
}

pub struct DouyinLive;

impl DouyinLive {
    pub fn new() -> Self {
        Self
    }

    fn make_error(message: impl Into<String>, source: Option<anyhow::Error>) -> AppError {
        AppError::ExternalApi {
            platform: "douyin".into(),
            message: message.into(),
            source,
        }
    }

    pub async fn resolve_short_url(&self, short_url: &str) -> crate::error::AppResult<String> {
        if let Some(caps) = Regex::new(r"^https?://live\.douyin\.com/([a-zA-Z0-9]+)")
            .unwrap()
            .captures(short_url)
        {
            return Ok(caps[1].to_string());
        }

        let resp = HTTP_CLIENT.get(short_url).send().await.map_err(|e| {
            Self::make_error(
                format!("resolve_short_url({short_url}) failed: {e}"),
                Some(e.into()),
            )
        })?;
        let final_url = resp.url().to_string();

        if final_url.contains("/user/") {
            let sec_uid = reqwest::Url::parse(&final_url)
                .ok()
                .and_then(|u| {
                    u.query_pairs()
                        .find(|(k, _)| k == "sec_uid")
                        .map(|(_, v)| v.to_string())
                })
                .ok_or_else(|| {
                    Self::make_error(
                        format!("resolve_short_url({short_url}) sec_uid not found"),
                        None,
                    )
                })?;
            return self
                .parse_user(&format!("https://www.douyin.com/user/{sec_uid}"))
                .await;
        }

        let body = resp.text().await.map_err(|e| {
            Self::make_error(
                format!("resolve_short_url({short_url}) read body failed: {e}"),
                Some(e.into()),
            )
        })?;

        Regex::new(r#""webRid\\":\\"(\d+)\\""#)
            .unwrap()
            .captures(&body)
            .map(|c| c[1].to_string())
            .ok_or_else(|| {
                Self::make_error(
                    format!("resolve_short_url({short_url}) webRid not found"),
                    None,
                )
            })
    }

    async fn parse_user(&self, url: &str) -> crate::error::AppResult<String> {
        let nonce = Self::generate_nonce();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let ua = USER_AGENT_POOL.get(Browser::Chrome, Platform::Windows);
        let cookie = format!(
            "__ac_nonce={nonce}; __ac_signature={}",
            get_ac_signature(timestamp, url, &nonce, ua)
        );

        let body = HTTP_CLIENT
            .get(url)
            .header("User-Agent", ua)
            .header("Cookie", cookie)
            .send()
            .await
            .map_err(|e| {
                Self::make_error(format!("parse_user({url}) failed: {e}"), Some(e.into()))
            })?
            .text()
            .await
            .map_err(|e| {
                Self::make_error(
                    format!("parse_user({url}) read body failed: {e}"),
                    Some(e.into()),
                )
            })?;

        Regex::new(r#"\\\"uniqueId\\\":\\\"(.*?)\\\""#)
            .unwrap()
            .captures(&body)
            .map(|c| c[1].to_string())
            .ok_or_else(|| Self::make_error(format!("parse_user({url}) uniqueId not found"), None))
    }

    fn generate_nonce() -> String {
        const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::rng();
        (0..21)
            .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
            .collect()
    }

    async fn get_cookie() -> crate::error::AppResult<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        {
            let cache = COOKIE_CACHE.read().await;
            if let Some(ref c) = *cache
                && now - c.timestamp < 6 * 60 * 60
            {
                return Ok(c.cookies.clone());
            }
        }

        let resp = HTTP_CLIENT
            .get(LIVE_HOME_URL)
            .send()
            .await
            .map_err(|e| Self::make_error(format!("get_cookie failed: {e}"), Some(e.into())))?;

        let cookies: String = resp
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .filter_map(|s| s.split(';').next())
            .collect::<Vec<_>>()
            .join("; ");

        if cookies.is_empty() {
            return Err(Self::make_error("get_cookie: no cookies in response", None));
        }

        if !cookies.contains("ttwid") {
            let mut cache = COOKIE_CACHE.write().await;
            if let Some(ref mut c) = *cache {
                c.timestamp += 60 * 60;
                return Ok(c.cookies.clone());
            }
        }

        let mut cache = COOKIE_CACHE.write().await;
        *cache = Some(CookieCache {
            cookies: cookies.clone(),
            timestamp: now,
        });
        Ok(cookies)
    }
}

impl Default for DouyinLive {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LivePlatformProvider for DouyinLive {
    fn platform(&self) -> LivePlatform {
        LivePlatform::Douyin
    }

    async fn get_room_info(&self, room_id: &str) -> crate::error::AppResult<RoomInfo> {
        let cookies = Self::get_cookie().await?;
        let ua = USER_AGENT_POOL.get(Browser::Chrome, Platform::Windows);

        let params = format!(
            "aid=6383&live_id=1&device_platform=web&language=zh-CN&enter_from=web_live&cookie_enabled=true&screen_width=1920&screen_height=1080&browser_language=zh-CN&browser_platform=Win32&browser_name=Chrome&browser_version=131.0.0.0&web_rid={}&Room-Enter-User-Login-Ab=0&is_need_double_stream=false",
            room_id
        );

        let mut abogus = ABogus::new(ua);
        let a_bogus = abogus.generate(&params);
        let url = format!("{}?{}&a_bogus={}", ENTER_ROOM_API, params, a_bogus);

        let resp = HTTP_CLIENT
            .get(&url)
            .header("User-Agent", ua)
            .header("Cookie", &cookies)
            .send()
            .await
            .map_err(|e| {
                Self::make_error(
                    format!("get_room_info({}) request failed: {}", room_id, e),
                    Some(e.into()),
                )
            })?
            .error_for_status()
            .map_err(|e| {
                Self::make_error(
                    format!("get_room_info({}) HTTP error: {}", room_id, e),
                    Some(e.into()),
                )
            })?;

        let data: DouyinEnterRoomResp = resp.json().await.map_err(|e| {
            Self::make_error(
                format!("get_room_info({}) invalid JSON: {}", room_id, e),
                Some(e.into()),
            )
        })?;

        if data.status_code != 0 {
            return Err(Self::make_error(
                format!(
                    "get_room_info({}) API error code: {}",
                    room_id, data.status_code
                ),
                None,
            ));
        }

        let room_data = data.data.ok_or_else(|| {
            Self::make_error(
                format!("get_room_info({}) no data in response", room_id),
                None,
            )
        })?;
        let room = room_data.data.and_then(|d| d.into_iter().next());
        let is_living = room_data.room_status == Some(0);

        Ok(RoomInfo {
            room_id: room
                .as_ref()
                .and_then(|r| r.id_str.clone())
                .unwrap_or_else(|| room_id.to_string()),
            title: room
                .as_ref()
                .and_then(|r| r.title.clone())
                .unwrap_or_default(),
            live_status: if is_living {
                LiveStatus::Live
            } else {
                LiveStatus::Offline
            },
            online: room
                .as_ref()
                .and_then(|r| r.room_view_stats.as_ref())
                .and_then(|s| s.display_value)
                .unwrap_or(0),
            cover_url: room
                .as_ref()
                .and_then(|r| r.cover.as_ref())
                .and_then(|c| c.url_list.as_ref())
                .and_then(|l| l.first().cloned()),
            area_name: room
                .as_ref()
                .and_then(|r| r.game_data.as_ref())
                .and_then(|g| g.game_tag_info.as_ref())
                .and_then(|t| t.game_tag_name.clone())
                .filter(|s| !s.is_empty()),
        })
    }

    async fn get_anchor_info(&self, _uid: &str) -> crate::error::AppResult<AnchorInfo> {
        todo!("DouyinLive::get_anchor_info")
    }

    async fn get_rooms_status_by_uids(
        &self,
        _uids: &[&str],
    ) -> crate::error::AppResult<HashMap<String, RoomStatusInfo>> {
        todo!("DouyinLive::get_rooms_status_by_uids")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_returns_douyin() {
        let client = DouyinLive::new();
        assert_eq!(client.platform(), LivePlatform::Douyin);
    }

    #[test]
    fn test_default_impl() {
        let _client: DouyinLive = Default::default();
    }

    #[test]
    fn test_make_error_without_source() {
        let err = DouyinLive::make_error("test error", None);
        match err {
            AppError::ExternalApi {
                platform,
                message,
                source,
            } => {
                assert_eq!(platform, "douyin");
                assert_eq!(message, "test error");
                assert!(source.is_none());
            }
            _ => panic!("Expected ExternalApi error"),
        }
    }

    #[test]
    fn test_generate_nonce_length() {
        let nonce = DouyinLive::generate_nonce();
        assert_eq!(nonce.len(), 21);
    }

    #[test]
    fn test_generate_nonce_chars() {
        let nonce = DouyinLive::generate_nonce();
        assert!(nonce.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_resolve_short_url_real_api() {
        let client = DouyinLive::new();
        let result = client
            .resolve_short_url("https://live.douyin.com/359765653648")
            .await;
        assert!(result.is_ok(), "Failed: {:?}", result.err());
        let room_id = result.unwrap();
        assert!(!room_id.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_get_room_info_real_api() {
        let client = DouyinLive::new();
        let result = client.get_room_info("913983320367").await;
        assert!(result.is_ok(), "Failed: {:?}", result.err());
        let room = result.unwrap();
        assert!(!room.room_id.is_empty());
    }
}
