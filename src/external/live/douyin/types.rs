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
    pub user: Option<DouyinBaseInfo>,
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
pub struct DouyinAvatarThumb {
    pub url_list: Option<Vec<String>>,
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

#[derive(Debug, Deserialize)]
pub struct DouyinUserProfileResp {
    pub status_code: i32,
    pub data: Option<DouyinUserProfileData>,
}

#[derive(Debug, Deserialize)]
pub struct DouyinUserProfileData {
    pub user_profile: Option<DouyinUserProfile>,
    pub user_data: Option<DouyinUserData>,
}

#[derive(Debug, Deserialize)]
pub struct DouyinUserProfile {
    pub base_info: Option<DouyinBaseInfo>,
    pub follow_info: Option<DouyinFollowInfo>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DouyinBaseInfo {
    pub id_str: Option<String>,
    pub nickname: Option<String>,
    pub avatar_thumb: Option<DouyinAvatarThumb>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DouyinUserData {
    pub web_rid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DouyinFollowInfo {
    pub follower_count: Option<u64>,
}
