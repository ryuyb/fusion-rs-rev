mod bilibili;
mod douyin;
mod douyu;
mod huya;
mod platform;
mod provider;
mod types;

pub use bilibili::BilibiliLive;
pub use douyin::DouyinLive;
pub use douyu::DouyuLive;
pub use huya::HuyaLive;
pub use platform::LivePlatform;
pub use provider::LivePlatformProvider;
pub use types::{AnchorInfo, LiveStatus, RoomInfo, RoomStatusInfo};

use std::sync::LazyLock;

static BILIBILI: LazyLock<BilibiliLive> = LazyLock::new(BilibiliLive::new);
static DOUYIN: LazyLock<DouyinLive> = LazyLock::new(DouyinLive::new);
static DOUYU: LazyLock<DouyuLive> = LazyLock::new(DouyuLive::new);
static HUYA: LazyLock<HuyaLive> = LazyLock::new(HuyaLive::new);

pub fn get_provider(platform: LivePlatform) -> &'static dyn LivePlatformProvider {
    match platform {
        LivePlatform::Bilibili => &*BILIBILI,
        LivePlatform::Douyin => &*DOUYIN,
        LivePlatform::Douyu => &*DOUYU,
        LivePlatform::Huya => &*HUYA,
    }
}
