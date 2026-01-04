use super::sign::get_ac_signature;
use crate::error::AppError;
use crate::external::client::HTTP_CLIENT;
use crate::external::live::platform::LivePlatform;
use crate::external::live::provider::LivePlatformProvider;
use crate::external::live::types::{AnchorInfo, RoomInfo, RoomStatusInfo};
use crate::external::user_agent::{Browser, Platform, USER_AGENT_POOL};
use async_trait::async_trait;
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

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
        let resp = HTTP_CLIENT
            .get(short_url)
            .send()
            .await
            .map_err(|e| {
                Self::make_error(
                    format!("resolve_short_url({}) request failed: {}", short_url, e),
                    Some(e.into()),
                )
            })?
            .error_for_status()
            .map_err(|e| {
                Self::make_error(
                    format!("resolve_short_url({}) HTTP error: {}", short_url, e),
                    Some(e.into()),
                )
            })?;

        let final_url = resp.url().to_string();

        if final_url.contains("/user/") {
            if let Ok(url) = reqwest::Url::parse(&final_url)
                && let Some((_, sec_uid)) = url.query_pairs().find(|(k, _)| k == "sec_uid")
            {
                return self
                    .parse_user(&format!("https://www.douyin.com/user/{}", sec_uid))
                    .await;
            }
            return Err(Self::make_error(
                format!(
                    "resolve_short_url({}) redirected to user page but sec_uid not found in URL: {}",
                    short_url, final_url
                ),
                None,
            ));
        }

        let body = resp.text().await.map_err(|e| {
            Self::make_error(
                format!("resolve_short_url({}) read body failed: {}", short_url, e),
                Some(e.into()),
            )
        })?;

        let re = Regex::new(r#""webRid\\":\\"(\d+)\\""#).unwrap();
        if let Some(caps) = re.captures(&body)
            && let Some(web_rid) = caps.get(1)
        {
            return Ok(web_rid.as_str().to_string());
        }

        Err(Self::make_error(
            format!(
                "resolve_short_url({}) webRid pattern not found in response, final_url: {}",
                short_url, final_url
            ),
            None,
        ))
    }

    async fn parse_user(&self, url: &str) -> crate::error::AppResult<String> {
        let nonce = Self::generate_nonce();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let ua = USER_AGENT_POOL.get(Browser::Chrome, Platform::Windows);
        let signature = get_ac_signature(timestamp, url, &nonce, ua);
        let cookie = format!("__ac_nonce={}; __ac_signature={}", nonce, signature);

        let resp = HTTP_CLIENT
            .get(url)
            .header("User-Agent", ua)
            .header("Cookie", cookie)
            .send()
            .await
            .map_err(|e| {
                Self::make_error(
                    format!("parse_user({}) request failed: {}", url, e),
                    Some(e.into()),
                )
            })?
            .error_for_status()
            .map_err(|e| {
                Self::make_error(
                    format!("parse_user({}) HTTP error: {}", url, e),
                    Some(e.into()),
                )
            })?;

        let body = resp.text().await.map_err(|e| {
            Self::make_error(
                format!("parse_user({}) read body failed: {}", url, e),
                Some(e.into()),
            )
        })?;

        let re = Regex::new(r#"\\\"uniqueId\\\":\\\"(.*?)\\\""#).unwrap();
        if let Some(caps) = re.captures(&body)
            && let Some(unique_id) = caps.get(1)
        {
            return Ok(unique_id.as_str().to_string());
        }

        Err(Self::make_error(
            format!("parse_user({}) uniqueId pattern not found in response", url),
            None,
        ))
    }

    fn generate_nonce() -> String {
        const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::rng();
        (0..21)
            .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
            .collect()
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

    async fn get_room_info(&self, _room_id: &str) -> crate::error::AppResult<RoomInfo> {
        todo!("DouyinLive::get_room_info")
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
}
