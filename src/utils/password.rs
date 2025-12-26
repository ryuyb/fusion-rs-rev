use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use crate::error::AppResult;

/// Hash a password using Argon2id
///
/// # Arguments
/// * `password` - The plain text password to hash
///
/// # Returns
/// * `AppResult<String>` - The hashed password string or an error
///
/// # Example
/// ```
/// let hashed = hash_password("my_secure_password")?;
/// ```
pub fn hash_password(password: &str) -> AppResult<String> {
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes())?
        .to_string();
    
    Ok(password_hash)
}

/// Verify a password against a hash
///
/// # Arguments
/// * `password` - The plain text password to verify
/// * `password_hash` - The hashed password to verify against
///
/// # Returns
/// * `AppResult<bool>` - True if password matches, false otherwise
///
/// # Example
/// ```
/// let is_valid = verify_password("my_secure_password", &hashed_password)?;
/// ```
pub fn verify_password(
    password: &str,
    password_hash: &str,
) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    let argon2 = Argon2::default();
    
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test_password_123";
        let hash = hash_password(password).expect("Failed to hash password");
        
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_verify_password_success() {
        let password = "test_password_123";
        let hash = hash_password(password).expect("Failed to hash password");
        
        let result = verify_password(password, &hash).expect("Failed to verify password");
        assert!(result);
    }

    #[test]
    fn test_verify_password_failure() {
        let password = "test_password_123";
        let wrong_password = "wrong_password";
        let hash = hash_password(password).expect("Failed to hash password");
        
        let result = verify_password(wrong_password, &hash).expect("Failed to verify password");
        assert!(!result);
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let password = "test_password_123";
        let hash1 = hash_password(password).expect("Failed to hash password");
        let hash2 = hash_password(password).expect("Failed to hash password");
        
        // Different salts should produce different hashes
        assert_ne!(hash1, hash2);
        
        // But both should verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }
}
