use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DouyuResponse<T> {
    pub error: i32,
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DouyuRoomData {
    pub room_id: String,
    pub room_name: String,
    pub room_status: String,
    pub owner_name: String,
    pub avatar: String,
    pub room_thumb: String,
    pub cate_name: String,
    pub online: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DouyuBetardResponse {
    pub room: DouyuBetardRoom,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DouyuBetardRoom {
    pub room_id: i64,
    pub room_name: String,
    pub show_status: i32,
    #[serde(rename = "videoLoop")]
    pub video_loop: i32,
    pub owner_uid: i64,
    pub owner_name: String,
    pub avatar: DouyuAvatar,
    #[serde(default)]
    pub room_pic: String,
    #[serde(default)]
    pub second_lvl_name: String,
    #[serde(default)]
    pub iol: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DouyuAvatar {
    pub big: Option<String>,
    pub middle: Option<String>,
    pub small: Option<String>,
}

impl DouyuAvatar {
    pub fn get_best(&self) -> Option<String> {
        self.big
            .as_ref()
            .or(self.middle.as_ref())
            .or(self.small.as_ref())
            .cloned()
    }
}
