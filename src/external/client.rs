use std::sync::LazyLock;
use std::time::Duration;

use super::user_agent::random_user_agent;

/// Global HTTP client instance with optimized configuration
///
/// This client is initialized lazily on first access and reused across the application.
///
/// # Benefits
/// - **Connection pooling**: Reuses TCP connections for better performance
/// - **DNS caching**: Reduces DNS lookup overhead
/// - **Memory efficiency**: Single client instance for the entire application
///
/// # Features
/// - **Random Chrome Windows User-Agent**: Uses a random Chrome Windows User-Agent on initialization
/// - **Compression**: Supports gzip, deflate, brotli, and zstd compression
/// - **HTTP/2**: Full HTTP/2 support with adaptive window sizing and keep-alive
/// - **Cookie store**: Automatically manages cookies across requests
/// - **Timeouts**: 30s request timeout, 10s connect timeout
/// - **Security**: Uses Rustls for TLS (no OpenSSL dependency)
///
/// # Example
/// ```rust
/// use crate::external::client::HTTP_CLIENT;
///
/// async fn fetch_data() -> Result<String, reqwest::Error> {
///     let response = HTTP_CLIENT
///         .get("https://api.example.com/data")
///         .send()
///         .await?;
///
///     response.text().await
/// }
/// ```
///
/// # Advanced Usage
/// ```rust
/// use crate::external::client::HTTP_CLIENT;
/// use crate::external::user_agent::{USER_AGENT_POOL, Browser, Platform};
///
/// async fn fetch_with_custom_ua() -> Result<String, reqwest::Error> {
///     // Override the default User-Agent for a specific request
///     let custom_ua = USER_AGENT_POOL.get(Browser::Firefox, Platform::Android);
///
///     let response = HTTP_CLIENT
///         .get("https://api.example.com/data")
///         .header("User-Agent", custom_ua)
///         .send()
///         .await?;
///
///     response.text().await
/// }
/// ```
pub static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        // Timeouts
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        // Connection pooling
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        // HTTP/2 settings
        .http2_adaptive_window(true)
        .http2_keep_alive_interval(Duration::from_secs(10))
        .http2_keep_alive_timeout(Duration::from_secs(20))
        // Enable compression (gzip, deflate, brotli, zstd)
        .gzip(true)
        .deflate(true)
        .brotli(true)
        .zstd(true)
        // Security
        .https_only(false)
        .use_rustls_tls()
        // Features
        .cookie_store(true)
        // Random Chrome Windows User-Agent
        .user_agent(random_user_agent())
        .build()
        .expect("Failed to build HTTP client")
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_initialization() {
        // Access the client to ensure it initializes without panicking
        let _ = &*HTTP_CLIENT;
    }

    #[tokio::test]
    async fn test_client_basic_request() {
        // This test requires internet connectivity
        let result = HTTP_CLIENT.get("https://httpbin.org/get").send().await;

        assert!(result.is_ok(), "Failed to make basic HTTP request");
    }

    #[tokio::test]
    async fn test_client_has_user_agent() {
        // Verify that the client sends a User-Agent header
        let result = HTTP_CLIENT
            .get("https://httpbin.org/user-agent")
            .send()
            .await;

        assert!(result.is_ok(), "Failed to make request");

        if let Ok(response) = result {
            let json: serde_json::Value = response.json().await.unwrap();
            let user_agent = json["user-agent"].as_str().unwrap();

            // Verify it's a Chrome user agent
            assert!(
                user_agent.contains("Chrome/"),
                "User agent should contain Chrome"
            );
        }
    }
}
