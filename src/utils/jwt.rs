use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};

/// Token type enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    /// Access token for API authentication (short-lived)
    Access,
    /// Refresh token for obtaining new access tokens (long-lived)
    Refresh,
}

/// JWT Claims structure containing user information and token metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// User email
    pub email: String,
    /// Username
    pub username: String,
    /// Token type (access or refresh)
    pub token_type: TokenType,
    /// Issued at (timestamp)
    pub iat: i64,
    /// Expiration time (timestamp)
    pub exp: i64,
}

impl Claims {
    /// Creates new claims for a user
    ///
    /// # Arguments
    /// * `user_id` - The user's ID
    /// * `email` - The user's email
    /// * `username` - The user's username
    /// * `token_type` - The type of token (Access or Refresh)
    /// * `expiration_hours` - Token validity duration in hours
    pub fn new(
        user_id: i32,
        email: String,
        username: String,
        token_type: TokenType,
        expiration_hours: i64,
    ) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(expiration_hours);
        
        Self {
            sub: user_id.to_string(),
            email,
            username,
            token_type,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        }
    }
}

/// Generates a JWT token for a user
///
/// # Arguments
/// * `user_id` - The user's ID
/// * `email` - The user's email
/// * `username` - The user's username
/// * `token_type` - The type of token (Access or Refresh)
/// * `secret` - The secret key for signing the token
/// * `expiration_hours` - Token validity duration in hours
///
/// # Returns
/// The encoded JWT token string
///
/// # Example
/// ```
/// let token = generate_token(1, "user@example.com".to_string(), "user".to_string(), TokenType::Access, "secret", 1)?;
/// ```
pub fn generate_token(
    user_id: i32,
    email: String,
    username: String,
    token_type: TokenType,
    secret: &str,
    expiration_hours: i64,
) -> AppResult<String> {
    let claims = Claims::new(user_id, email, username, token_type, expiration_hours);
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal {
        source: anyhow::anyhow!("Failed to generate JWT token: {}", e),
    })
}

/// Generates an access token (short-lived)
///
/// # Arguments
/// * `user_id` - The user's ID
/// * `email` - The user's email
/// * `username` - The user's username
/// * `secret` - The secret key for signing the token
/// * `expiration_hours` - Token validity duration in hours (typically 1-24 hours)
///
/// # Returns
/// The encoded access token string
pub fn generate_access_token(
    user_id: i32,
    email: String,
    username: String,
    secret: &str,
    expiration_hours: i64,
) -> AppResult<String> {
    generate_token(user_id, email, username, TokenType::Access, secret, expiration_hours)
}

/// Generates a refresh token (long-lived)
///
/// # Arguments
/// * `user_id` - The user's ID
/// * `email` - The user's email
/// * `username` - The user's username
/// * `secret` - The secret key for signing the token
/// * `expiration_hours` - Token validity duration in hours (typically 168-720 hours / 7-30 days)
///
/// # Returns
/// The encoded refresh token string
pub fn generate_refresh_token(
    user_id: i32,
    email: String,
    username: String,
    secret: &str,
    expiration_hours: i64,
) -> AppResult<String> {
    generate_token(user_id, email, username, TokenType::Refresh, secret, expiration_hours)
}

/// Generates both access and refresh tokens
///
/// # Arguments
/// * `user_id` - The user's ID
/// * `email` - The user's email
/// * `username` - The user's username
/// * `secret` - The secret key for signing the tokens
/// * `access_expiration_hours` - Access token validity duration in hours
/// * `refresh_expiration_hours` - Refresh token validity duration in hours
///
/// # Returns
/// A tuple of (access_token, refresh_token)
pub fn generate_token_pair(
    user_id: i32,
    email: String,
    username: String,
    secret: &str,
    access_expiration_hours: i64,
    refresh_expiration_hours: i64,
) -> AppResult<(String, String)> {
    let access_token = generate_access_token(
        user_id,
        email.clone(),
        username.clone(),
        secret,
        access_expiration_hours,
    )?;
    
    let refresh_token = generate_refresh_token(
        user_id,
        email,
        username,
        secret,
        refresh_expiration_hours,
    )?;
    
    Ok((access_token, refresh_token))
}

/// Validates and decodes a JWT token
///
/// # Arguments
/// * `token` - The JWT token string to validate
/// * `secret` - The secret key for verifying the token
/// * `expected_type` - Optional expected token type to validate against
///
/// # Returns
/// The decoded claims if the token is valid
///
/// # Example
/// ```
/// let claims = validate_token(&token, "secret", Some(TokenType::Access))?;
/// println!("User ID: {}", claims.sub);
/// ```
pub fn validate_token(
    token: &str,
    secret: &str,
    expected_type: Option<TokenType>,
) -> AppResult<Claims> {
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::Unauthorized {
            message: "Token has expired".to_string(),
        },
        jsonwebtoken::errors::ErrorKind::InvalidToken => AppError::Unauthorized {
            message: "Invalid token".to_string(),
        },
        jsonwebtoken::errors::ErrorKind::InvalidSignature => AppError::Unauthorized {
            message: "Invalid token signature".to_string(),
        },
        _ => AppError::Unauthorized {
            message: format!("Token validation failed: {}", e),
        },
    })?;

    // Validate token type if specified
    if let Some(expected) = expected_type {
        if claims.token_type != expected {
            return Err(AppError::Unauthorized {
                message: format!(
                    "Invalid token type: expected {:?}, got {:?}",
                    expected, claims.token_type
                ),
            });
        }
    }

    Ok(claims)
}

/// Validates an access token
///
/// # Arguments
/// * `token` - The JWT token string to validate
/// * `secret` - The secret key for verifying the token
///
/// # Returns
/// The decoded claims if the token is valid and is an access token
pub fn validate_access_token(token: &str, secret: &str) -> AppResult<Claims> {
    validate_token(token, secret, Some(TokenType::Access))
}

/// Validates a refresh token
///
/// # Arguments
/// * `token` - The JWT token string to validate
/// * `secret` - The secret key for verifying the token
///
/// # Returns
/// The decoded claims if the token is valid and is a refresh token
pub fn validate_refresh_token(token: &str, secret: &str) -> AppResult<Claims> {
    validate_token(token, secret, Some(TokenType::Refresh))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test_secret_key_for_jwt_testing";

    #[test]
    fn test_generate_token() {
        let token = generate_token(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            TokenType::Access,
            TEST_SECRET,
            24,
        );
        
        assert!(token.is_ok());
        let token_str = token.unwrap();
        assert!(!token_str.is_empty());
        assert!(token_str.contains('.'));
    }

    #[test]
    fn test_generate_access_token() {
        let token = generate_access_token(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            TEST_SECRET,
            1,
        );
        
        assert!(token.is_ok());
    }

    #[test]
    fn test_generate_refresh_token() {
        let token = generate_refresh_token(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            TEST_SECRET,
            168,
        );
        
        assert!(token.is_ok());
    }

    #[test]
    fn test_generate_token_pair() {
        let result = generate_token_pair(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            TEST_SECRET,
            1,
            168,
        );
        
        assert!(result.is_ok());
        let (access_token, refresh_token) = result.unwrap();
        assert!(!access_token.is_empty());
        assert!(!refresh_token.is_empty());
        assert_ne!(access_token, refresh_token);
    }

    #[test]
    fn test_validate_token_success() {
        let token = generate_token(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            TokenType::Access,
            TEST_SECRET,
            24,
        )
        .unwrap();
        
        let claims = validate_token(&token, TEST_SECRET, None);
        assert!(claims.is_ok());
        
        let claims = claims.unwrap();
        assert_eq!(claims.sub, "1");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn test_validate_access_token() {
        let token = generate_access_token(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            TEST_SECRET,
            1,
        )
        .unwrap();
        
        let claims = validate_access_token(&token, TEST_SECRET);
        assert!(claims.is_ok());
        assert_eq!(claims.unwrap().token_type, TokenType::Access);
    }

    #[test]
    fn test_validate_refresh_token() {
        let token = generate_refresh_token(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            TEST_SECRET,
            168,
        )
        .unwrap();
        
        let claims = validate_refresh_token(&token, TEST_SECRET);
        assert!(claims.is_ok());
        assert_eq!(claims.unwrap().token_type, TokenType::Refresh);
    }

    #[test]
    fn test_validate_wrong_token_type() {
        let access_token = generate_access_token(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            TEST_SECRET,
            1,
        )
        .unwrap();
        
        // Try to validate access token as refresh token
        let result = validate_refresh_token(&access_token, TEST_SECRET);
        assert!(result.is_err());
        
        if let Err(AppError::Unauthorized { message }) = result {
            assert!(message.contains("Invalid token type"));
        } else {
            panic!("Expected Unauthorized error for wrong token type");
        }
    }

    #[test]
    fn test_validate_token_invalid_secret() {
        let token = generate_token(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            TokenType::Access,
            TEST_SECRET,
            24,
        )
        .unwrap();
        
        let result = validate_token(&token, "wrong_secret", None);
        assert!(result.is_err());
        
        if let Err(AppError::Unauthorized { message }) = result {
            assert!(message.contains("signature"));
        } else {
            panic!("Expected Unauthorized error");
        }
    }

    #[test]
    fn test_validate_token_invalid_format() {
        let result = validate_token("invalid.token.format", TEST_SECRET, None);
        assert!(result.is_err());
        
        if let Err(AppError::Unauthorized { message }) = result {
            assert!(message.contains("Invalid token") || message.contains("validation"));
        } else {
            panic!("Expected Unauthorized error");
        }
    }

    #[test]
    fn test_expired_token() {
        // Create a token that expires immediately (0 hours)
        let token = generate_token(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            TokenType::Access,
            TEST_SECRET,
            -1, // Negative hours to create an already expired token
        )
        .unwrap();
        
        let result = validate_token(&token, TEST_SECRET, None);
        assert!(result.is_err());
        
        if let Err(AppError::Unauthorized { message }) = result {
            assert!(message.contains("expired"));
        } else {
            panic!("Expected Unauthorized error for expired token");
        }
    }

    #[test]
    fn test_claims_structure() {
        let claims = Claims::new(
            42,
            "user@example.com".to_string(),
            "username".to_string(),
            TokenType::Refresh,
            24,
        );
        
        assert_eq!(claims.sub, "42");
        assert_eq!(claims.email, "user@example.com");
        assert_eq!(claims.username, "username");
        assert_eq!(claims.token_type, TokenType::Refresh);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_token_type_serialization() {
        let access_claims = Claims::new(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            TokenType::Access,
            1,
        );
        
        let json = serde_json::to_string(&access_claims).unwrap();
        assert!(json.contains("\"token_type\":\"access\""));
        
        let refresh_claims = Claims::new(
            1,
            "test@example.com".to_string(),
            "testuser".to_string(),
            TokenType::Refresh,
            168,
        );
        
        let json = serde_json::to_string(&refresh_claims).unwrap();
        assert!(json.contains("\"token_type\":\"refresh\""));
    }
}
