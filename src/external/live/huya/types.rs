use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct MpApiResponse {
    pub status: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<MpData>,
}

#[derive(Debug, Deserialize)]
pub(super) struct MpData {
    #[serde(rename = "realLiveStatus")]
    pub real_live_status: Option<String>,
    #[serde(rename = "liveStatus")]
    pub live_status: Option<String>,
    #[serde(rename = "profileInfo")]
    pub profile_info: ProfileInfo,
    #[serde(rename = "liveData")]
    pub live_data: LiveData,
}

#[derive(Debug, Deserialize)]
pub(super) struct ProfileInfo {
    #[serde(deserialize_with = "deserialize_uid")]
    pub uid: i64,
    pub nick: String,
    pub avatar180: String,
    #[serde(
        rename = "profileRoom",
        default,
        deserialize_with = "deserialize_optional_room_id"
    )]
    pub profile_room: Option<String>,
    #[serde(rename = "activityCount", default)]
    pub activity_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(super) struct LiveData {
    pub introduction: String,
    pub screenshot: String,
    #[serde(rename = "gameFullName", default)]
    pub game_full_name: Option<String>,
    #[serde(rename = "userCount", default)]
    pub user_count: Option<u64>,
}

fn deserialize_uid<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum UidValue {
        Int(i64),
        Str(String),
    }

    match UidValue::deserialize(deserializer)? {
        UidValue::Int(i) => Ok(i),
        UidValue::Str(s) => s.parse().map_err(D::Error::custom),
    }
}

fn deserialize_optional_room_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RoomIdValue {
        Int(i64),
        Str(String),
    }

    match Option::<RoomIdValue>::deserialize(deserializer)? {
        Some(RoomIdValue::Int(i)) => Ok(Some(i.to_string())),
        Some(RoomIdValue::Str(s)) => Ok(Some(s)),
        None => Ok(None),
    }
}
