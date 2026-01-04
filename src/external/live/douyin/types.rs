use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DouyinEnterRoomResp {
    pub status_code: i32,
    pub data: Option<DouyinEnterRoomData>,
}

#[derive(Debug, Deserialize)]
pub struct DouyinEnterRoomData {
    pub data: Option<Vec<DouyinRoomDetail>>,
    pub room_status: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct DouyinRoomDetail {
    pub id_str: Option<String>,
    pub title: Option<String>,
    pub cover: Option<DouyinCover>,
    pub game_data: Option<DouyinGameData>,
    pub room_view_stats: Option<DouyinRoomViewStats>,
}

#[derive(Debug, Deserialize)]
pub struct DouyinCover {
    pub url_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct DouyinGameData {
    pub game_tag_info: Option<DouyinGameTagInfo>,
}

#[derive(Debug, Deserialize)]
pub struct DouyinGameTagInfo {
    pub game_tag_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DouyinRoomViewStats {
    pub display_value: Option<u64>,
}
