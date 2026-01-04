use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct BiliResponse<T> {
    pub code: i32,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub(super) struct BiliRoomData {
    pub room_id: u64,
    pub title: String,
    pub live_status: u8,
    pub online: u64,
    pub user_cover: String,
    pub area_name: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct BiliAnchorData {
    pub info: BiliAnchorInfo,
    pub follower_num: u64,
    pub room_id: u64,
}

#[derive(Debug, Deserialize)]
pub(super) struct BiliAnchorInfo {
    pub uid: u64,
    pub uname: String,
    pub face: String,
}
