#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveStatus {
    Offline,
    Live,
    Replay,
}

#[derive(Debug, Clone)]
pub struct RoomInfo {
    pub room_id: String,
    pub title: String,
    pub live_status: LiveStatus,
    pub online: u64,
    pub cover_url: Option<String>,
    pub area_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AnchorInfo {
    pub uid: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub follower_count: Option<u64>,
    pub room_id: Option<String>,
}
