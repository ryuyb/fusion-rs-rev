mod bilibili;
mod platform;
mod provider;
mod types;

pub use bilibili::BilibiliLive;
pub use platform::LivePlatform;
pub use provider::LivePlatformProvider;
pub use types::{AnchorInfo, LiveStatus, RoomInfo, RoomStatusInfo};

use std::sync::LazyLock;

static BILIBILI: LazyLock<BilibiliLive> = LazyLock::new(BilibiliLive::new);

pub fn get_provider(platform: LivePlatform) -> &'static dyn LivePlatformProvider {
    match platform {
        LivePlatform::Bilibili => &*BILIBILI,
    }
}
