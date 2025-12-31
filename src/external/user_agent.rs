use rand::seq::IndexedRandom;
use std::sync::LazyLock;

/// Browser types supported in the User-Agent pool
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Browser {
    Chrome,
    Firefox,
    Safari,
    Edge,
}

/// Platform types (specific operating systems)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    Mac,
    Android,
    Ios,
}

/// User-Agent pool organized by browser and platform
pub struct UserAgentPool {
    chrome_windows: &'static [&'static str],
    chrome_linux: &'static [&'static str],
    chrome_mac: &'static [&'static str],
    chrome_android: &'static [&'static str],
    chrome_ios: &'static [&'static str],
    firefox_windows: &'static [&'static str],
    firefox_linux: &'static [&'static str],
    firefox_mac: &'static [&'static str],
    firefox_android: &'static [&'static str],
    firefox_ios: &'static [&'static str],
    safari_mac: &'static [&'static str],
    safari_ios: &'static [&'static str],
    edge_windows: &'static [&'static str],
    edge_mac: &'static [&'static str],
}

impl UserAgentPool {
    /// Creates a new User-Agent pool with predefined user agents
    const fn new() -> Self {
        Self {
            chrome_windows: &[
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
            ],
            chrome_linux: &[
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
            ],
            chrome_mac: &[
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
            ],
            chrome_android: &[
                "Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.6778.104 Mobile Safari/537.36",
                "Mozilla/5.0 (Linux; Android 13) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.6723.86 Mobile Safari/537.36",
                "Mozilla/5.0 (Linux; Android 14; SM-S928B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.6778.104 Mobile Safari/537.36",
            ],
            chrome_ios: &[
                "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/131.0.6778.103 Mobile/15E148 Safari/604.1",
                "Mozilla/5.0 (iPad; CPU OS 18_1_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/131.0.6778.103 Mobile/15E148 Safari/604.1",
            ],
            firefox_windows: &[
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:132.0) Gecko/20100101 Firefox/132.0",
            ],
            firefox_linux: &[
                "Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0",
                "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0",
            ],
            firefox_mac: &[
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.7; rv:133.0) Gecko/20100101 Firefox/133.0",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.6; rv:132.0) Gecko/20100101 Firefox/132.0",
            ],
            firefox_android: &[
                "Mozilla/5.0 (Android 14; Mobile; rv:133.0) Gecko/133.0 Firefox/133.0",
                "Mozilla/5.0 (Android 13; Mobile; rv:132.0) Gecko/132.0 Firefox/132.0",
            ],
            firefox_ios: &[
                "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/133.0 Mobile/15E148 Safari/605.1.15",
            ],
            safari_mac: &[
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_7_1) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1.1 Safari/605.1.15",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Safari/605.1.15",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_6_1) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15",
            ],
            safari_ios: &[
                "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1.1 Mobile/15E148 Safari/604.1",
                "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1",
                "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1",
                "Mozilla/5.0 (iPad; CPU OS 18_1_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1.1 Mobile/15E148 Safari/604.1",
            ],
            edge_windows: &[
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0",
            ],
            edge_mac: &[
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0",
            ],
        }
    }

    /// Gets a random User-Agent for the specified browser and platform
    ///
    /// # Arguments
    /// * `browser` - The browser type (Chrome, Firefox, Safari, Edge)
    /// * `platform` - The platform type (Windows, Linux, Mac, Android, Ios)
    ///
    /// # Returns
    /// A randomly selected User-Agent string matching the criteria, or a fallback if the combination is not available
    ///
    /// # Example
    /// ```rust
    /// use crate::external::user_agent::{USER_AGENT_POOL, Browser, Platform};
    ///
    /// // Get Chrome on Windows
    /// let chrome_win = USER_AGENT_POOL.get(Browser::Chrome, Platform::Windows);
    ///
    /// // Get Firefox on Android
    /// let firefox_android = USER_AGENT_POOL.get(Browser::Firefox, Platform::Android);
    ///
    /// // Get Safari on Ios
    /// let safari_ios = USER_AGENT_POOL.get(Browser::Safari, Platform::Ios);
    /// ```
    pub fn get(&self, browser: Browser, platform: Platform) -> &'static str {
        let mut rng = rand::rng();
        let pool = match (browser, platform) {
            // Chrome combinations
            (Browser::Chrome, Platform::Windows) => self.chrome_windows,
            (Browser::Chrome, Platform::Linux) => self.chrome_linux,
            (Browser::Chrome, Platform::Mac) => self.chrome_mac,
            (Browser::Chrome, Platform::Android) => self.chrome_android,
            (Browser::Chrome, Platform::Ios) => self.chrome_ios,

            // Firefox combinations
            (Browser::Firefox, Platform::Windows) => self.firefox_windows,
            (Browser::Firefox, Platform::Linux) => self.firefox_linux,
            (Browser::Firefox, Platform::Mac) => self.firefox_mac,
            (Browser::Firefox, Platform::Android) => self.firefox_android,
            (Browser::Firefox, Platform::Ios) => self.firefox_ios,

            // Safari combinations (only Mac and Ios)
            (Browser::Safari, Platform::Mac) => self.safari_mac,
            (Browser::Safari, Platform::Ios) => self.safari_ios,
            (Browser::Safari, Platform::Windows) => {
                // Safari doesn't run on Windows, fallback to Chrome Windows
                self.chrome_windows
            }
            (Browser::Safari, Platform::Linux) => {
                // Safari doesn't run on Linux, fallback to Chrome Linux
                self.chrome_linux
            }
            (Browser::Safari, Platform::Android) => {
                // Safari doesn't run on Android, fallback to Chrome Android
                self.chrome_android
            }

            // Edge combinations (only Windows and Mac)
            (Browser::Edge, Platform::Windows) => self.edge_windows,
            (Browser::Edge, Platform::Mac) => self.edge_mac,
            (Browser::Edge, Platform::Linux) => {
                // Edge on Linux is rare, fallback to Chrome Linux
                self.chrome_linux
            }
            (Browser::Edge, Platform::Android) => {
                // Edge mobile on Android uses Chromium, fallback to Chrome Android
                self.chrome_android
            }
            (Browser::Edge, Platform::Ios) => {
                // Edge on Ios uses Safari WebKit, fallback to Chrome Ios
                self.chrome_ios
            }
        };

        pool.choose(&mut rng).copied().unwrap_or(pool[0])
    }

    /// Gets a completely random User-Agent from all available options
    ///
    /// # Example
    /// ```rust
    /// use crate::external::user_agent::USER_AGENT_POOL;
    ///
    /// let random_ua = USER_AGENT_POOL.random();
    /// ```
    pub fn random(&self) -> &'static str {
        let mut rng = rand::rng();
        let all_pools = [
            self.chrome_windows,
            self.chrome_linux,
            self.chrome_mac,
            self.chrome_android,
            self.chrome_ios,
            self.firefox_windows,
            self.firefox_linux,
            self.firefox_mac,
            self.firefox_android,
            self.firefox_ios,
            self.safari_mac,
            self.safari_ios,
            self.edge_windows,
            self.edge_mac,
        ];

        let selected_pool = all_pools.choose(&mut rng).unwrap();
        selected_pool
            .choose(&mut rng)
            .copied()
            .unwrap_or(selected_pool[0])
    }
}

/// Global User-Agent pool instance
pub static USER_AGENT_POOL: LazyLock<UserAgentPool> = LazyLock::new(UserAgentPool::new);

/// Generates a random Chrome Windows User-Agent string
///
/// This is a convenience function for backward compatibility.
/// For more control, use `USER_AGENT_POOL.get(Browser, Platform)` directly.
///
/// # Returns
/// A randomly selected Chrome Windows User-Agent string
///
/// # Example
/// ```rust
/// use crate::external::user_agent::random_user_agent;
///
/// let ua = random_user_agent();
/// ```
pub fn random_user_agent() -> &'static str {
    USER_AGENT_POOL.get(Browser::Chrome, Platform::Windows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrome_windows() {
        let ua = USER_AGENT_POOL.get(Browser::Chrome, Platform::Windows);
        assert!(ua.contains("Chrome/"), "Should be Chrome user agent");
        assert!(ua.contains("Windows"), "Should be Windows");
    }

    #[test]
    fn test_chrome_linux() {
        let ua = USER_AGENT_POOL.get(Browser::Chrome, Platform::Linux);
        assert!(ua.contains("Chrome/"), "Should be Chrome user agent");
        assert!(ua.contains("Linux"), "Should be Linux");
    }

    #[test]
    fn test_chrome_mac() {
        let ua = USER_AGENT_POOL.get(Browser::Chrome, Platform::Mac);
        assert!(ua.contains("Chrome/"), "Should be Chrome user agent");
        assert!(ua.contains("Macintosh"), "Should be Mac");
    }

    #[test]
    fn test_chrome_android() {
        let ua = USER_AGENT_POOL.get(Browser::Chrome, Platform::Android);
        assert!(ua.contains("Chrome/"), "Should be Chrome user agent");
        assert!(ua.contains("Android"), "Should be Android");
    }

    #[test]
    fn test_chrome_ios() {
        let ua = USER_AGENT_POOL.get(Browser::Chrome, Platform::Ios);
        assert!(ua.contains("CriOS"), "Should be Chrome for iOS (CriOS)");
        assert!(
            ua.contains("iPhone") || ua.contains("iPad"),
            "Should be iOS"
        );
    }

    #[test]
    fn test_firefox_platforms() {
        let win_ua = USER_AGENT_POOL.get(Browser::Firefox, Platform::Windows);
        assert!(win_ua.contains("Firefox/"), "Should be Firefox");
        assert!(win_ua.contains("Windows"), "Should be Windows");

        let linux_ua = USER_AGENT_POOL.get(Browser::Firefox, Platform::Linux);
        assert!(linux_ua.contains("Firefox/"), "Should be Firefox");
        assert!(linux_ua.contains("Linux"), "Should be Linux");

        let mac_ua = USER_AGENT_POOL.get(Browser::Firefox, Platform::Mac);
        assert!(mac_ua.contains("Firefox/"), "Should be Firefox");
        assert!(mac_ua.contains("Macintosh"), "Should be Mac");

        let android_ua = USER_AGENT_POOL.get(Browser::Firefox, Platform::Android);
        assert!(android_ua.contains("Firefox/"), "Should be Firefox");
        assert!(android_ua.contains("Android"), "Should be Android");

        let ios_ua = USER_AGENT_POOL.get(Browser::Firefox, Platform::Ios);
        assert!(ios_ua.contains("FxiOS"), "Should be Firefox for iOS");
    }

    #[test]
    fn test_safari_platforms() {
        let mac_ua = USER_AGENT_POOL.get(Browser::Safari, Platform::Mac);
        assert!(mac_ua.contains("Safari/"), "Should be Safari");
        assert!(mac_ua.contains("Macintosh"), "Should be Mac");

        let ios_ua = USER_AGENT_POOL.get(Browser::Safari, Platform::Ios);
        assert!(ios_ua.contains("Safari"), "Should be Safari");
        assert!(
            ios_ua.contains("iPhone") || ios_ua.contains("iPad"),
            "Should be iOS"
        );
    }

    #[test]
    fn test_safari_fallback() {
        // Safari doesn't exist on Windows/Linux/Android, should fallback to Chrome
        let win_ua = USER_AGENT_POOL.get(Browser::Safari, Platform::Windows);
        assert!(win_ua.contains("Chrome/"), "Should fallback to Chrome");

        let linux_ua = USER_AGENT_POOL.get(Browser::Safari, Platform::Linux);
        assert!(linux_ua.contains("Chrome/"), "Should fallback to Chrome");

        let android_ua = USER_AGENT_POOL.get(Browser::Safari, Platform::Android);
        assert!(android_ua.contains("Chrome/"), "Should fallback to Chrome");
    }

    #[test]
    fn test_edge_platforms() {
        let win_ua = USER_AGENT_POOL.get(Browser::Edge, Platform::Windows);
        assert!(win_ua.contains("Edg/"), "Should be Edge");
        assert!(win_ua.contains("Windows"), "Should be Windows");

        let mac_ua = USER_AGENT_POOL.get(Browser::Edge, Platform::Mac);
        assert!(mac_ua.contains("Edg/"), "Should be Edge");
        assert!(mac_ua.contains("Macintosh"), "Should be Mac");
    }

    #[test]
    fn test_edge_fallback() {
        // Edge on Linux/Android/Ios uses Chromium or fallback
        let linux_ua = USER_AGENT_POOL.get(Browser::Edge, Platform::Linux);
        assert!(linux_ua.contains("Chrome/"), "Should fallback to Chrome");
    }

    #[test]
    fn test_random_user_agent() {
        let ua = USER_AGENT_POOL.random();
        assert!(!ua.is_empty(), "Random UA should not be empty");
    }

    #[test]
    fn test_backward_compatibility() {
        let ua = random_user_agent();
        assert!(ua.contains("Chrome/"), "Should be Chrome user agent");
        assert!(
            ua.contains("Windows"),
            "Should be Windows Chrome for backward compatibility"
        );
    }
}
