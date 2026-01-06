use std::fmt;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum LivePlatform {
    Bilibili,
    Douyin,
}

impl fmt::Display for LivePlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LivePlatform::Bilibili => write!(f, "bilibili"),
            LivePlatform::Douyin => write!(f, "douyin"),
        }
    }
}
