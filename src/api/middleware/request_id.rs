//! Request ID middleware for request tracing.
//!
//! This middleware ensures every request has a unique identifier for tracing
//! and correlation purposes. It either uses an existing X-Request-ID header
//! or generates a new UUID.

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Header name for request ID.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Request ID stored in request extensions for downstream access.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Middleware that ensures every request has a unique request ID.
///
/// # Behavior
/// - If the request contains an X-Request-ID header, uses that value
/// - If no header is present, generates a new UUID v4
/// - Stores the request ID in request extensions for downstream handlers
/// - Adds the request ID to the response headers
///
/// # Requirements
/// - 5.1: Generate new UUID when X-Request-ID header is missing
/// - 5.2: Use provided X-Request-ID header value when present
/// - 5.3: Add request ID to response headers
/// - 5.4: Store request ID in request extensions
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    // Get existing request ID from header or generate new one
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Store in request extensions for downstream access
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    // Process request
    let mut response = next.run(request).await;

    // Add request ID to response headers
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(REQUEST_ID_HEADER), value);
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_struct_clone() {
        let id = RequestId("test-id".to_string());
        let cloned = id.clone();
        assert_eq!(id.0, cloned.0);
    }

    #[test]
    fn test_request_id_header_constant() {
        assert_eq!(REQUEST_ID_HEADER, "x-request-id");
    }
}
