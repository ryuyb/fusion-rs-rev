//! JWT authentication middleware.
//!
//! Provides middleware for validating JWT tokens and extracting user claims.

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;
use crate::utils::jwt::{validate_access_token, Claims};

/// Extension type for authenticated user information
///
/// This is added to request extensions after successful authentication
/// and can be extracted in handlers using `Extension<AuthUser>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    /// User ID from JWT claims
    pub user_id: i32,
    /// User email from JWT claims
    pub email: String,
    /// Username from JWT claims
    pub username: String,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        Self {
            user_id: claims.sub.parse().unwrap_or(0),
            email: claims.email,
            username: claims.username,
        }
    }
}

/// JWT authentication middleware
///
/// Validates the JWT token from the Authorization header and adds
/// the authenticated user information to request extensions.
///
/// # Headers
/// Expects: `Authorization: Bearer <token>`
///
/// # Errors
/// Returns 401 Unauthorized if:
/// - Authorization header is missing
/// - Token format is invalid
/// - Token validation fails
/// - Token has expired
///
/// # Example
/// ```ignore
/// Router::new()
///     .route("/protected", get(handler))
///     .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
/// ```
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized {
            message: "Missing authorization header".to_string(),
        })?;

    // Extract token from "Bearer <token>" format
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized {
            message: "Invalid authorization header format. Expected: Bearer <token>".to_string(),
        })?;

    // Validate token and extract claims
    let claims = validate_access_token(token, &state.jwt_config.secret)?;

    // Convert claims to AuthUser and add to request extensions
    let auth_user = AuthUser::from(claims);
    request.extensions_mut().insert(auth_user);

    // Continue to the next middleware/handler
    Ok(next.run(request).await)
}

/// Optional JWT authentication middleware
///
/// Similar to `auth_middleware` but doesn't fail if the token is missing.
/// If a valid token is provided, it adds the user to extensions.
/// If no token or invalid token, continues without authentication.
///
/// Useful for endpoints that have different behavior for authenticated vs anonymous users.
///
/// # Example
/// ```ignore
/// Router::new()
///     .route("/optional", get(handler))
///     .layer(middleware::from_fn_with_state(state.clone(), optional_auth_middleware))
/// ```
#[allow(dead_code)]
pub async fn optional_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract and validate token, but don't fail if missing
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = validate_access_token(token, &state.jwt_config.secret) {
                    let auth_user = AuthUser::from(claims);
                    request.extensions_mut().insert(auth_user);
                }
            }
        }
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::JwtConfig;
    use crate::utils::jwt::{generate_access_token, TokenType};

    fn create_test_jwt_config() -> JwtConfig {
        JwtConfig {
            secret: "test_secret_key_at_least_32_characters_long".to_string(),
            access_token_expiration: 1,
            refresh_token_expiration: 168,
        }
    }

    #[test]
    fn test_auth_user_from_claims() {
        let claims = Claims {
            sub: "123".to_string(),
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            token_type: TokenType::Access,
            iat: 0,
            exp: 9999999999,
        };

        let auth_user = AuthUser::from(claims);
        assert_eq!(auth_user.user_id, 123);
        assert_eq!(auth_user.email, "test@example.com");
        assert_eq!(auth_user.username, "testuser");
    }

    #[test]
    fn test_auth_user_from_claims_invalid_id() {
        let claims = Claims {
            sub: "invalid".to_string(),
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            token_type: TokenType::Access,
            iat: 0,
            exp: 9999999999,
        };

        let auth_user = AuthUser::from(claims);
        assert_eq!(auth_user.user_id, 0); // Falls back to 0 on parse error
    }

    #[test]
    fn test_generate_valid_token() {
        let config = create_test_jwt_config();
        let token = generate_access_token(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            &config.secret,
            config.access_token_expiration,
        );
        assert!(token.is_ok());
    }
}
