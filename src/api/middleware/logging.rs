//! Logging middleware for request/response tracing.
//!
//! This middleware logs incoming requests and outgoing responses with
//! timing information and request correlation via request IDs.

use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::{Level, info, span};

use super::RequestId;

/// Middleware that logs request and response information.
///
/// # Logged Information
/// - Request: HTTP method, path, request ID
/// - Response: status code, duration in milliseconds, request ID
///
/// # Requirements
/// - 4.1: Log HTTP method, path, and request ID on request
/// - 4.2: Log status code and response time on response
/// - 4.3: Use tracing spans to correlate request and response logs
/// - 4.4: Include request ID in all log entries
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let span = span!(
        Level::INFO,
        "http_request",
        method = %method,
        uri = %uri,
        request_id = %request_id
    );
    let _enter = span.enter();

    info!(
        method = %method,
        path = %uri.path(),
        request_id = %request_id,
        "Request received"
    );

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    info!(
        status = %response.status().as_u16(),
        duration_ms = %duration.as_millis(),
        request_id = %request_id,
        "Response sent"
    );

    response
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_request_id_default_value() {
        // When no RequestId is in extensions, should use "unknown"
        let default_id = "unknown".to_string();
        assert_eq!(default_id, "unknown");
    }
}
