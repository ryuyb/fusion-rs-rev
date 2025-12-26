//! User service for business logic operations.
//!
//! Provides a higher-level API for user operations, encapsulating
//! business rules and coordinating with the repository layer.

use crate::error::{AppError, AppResult};
use crate::models::{NewUser, UpdateUser, User};
use crate::repositories::UserRepository;
use crate::utils::password::{hash_password, verify_password};
use crate::utils::jwt::generate_token_pair;

/// User service for handling user-related business logic.
///
/// This service wraps the `UserRepository` and provides business-level
/// operations. Since `UserRepository` uses `Arc` internally via the
/// connection pool, cloning is cheap.
#[derive(Clone)]
pub struct UserService {
    repo: UserRepository,
}

impl UserService {
    /// Creates a new UserService with the given repository.
    pub fn new(repo: UserRepository) -> Self {
        Self { repo }
    }

    /// Creates a new user.
    ///
    /// # Arguments
    /// * `new_user` - The user data to create
    ///
    /// # Returns
    /// The created user with generated id and timestamps
    ///
    /// # Note
    /// The password will be automatically hashed before storing
    pub async fn create_user(&self, mut new_user: NewUser) -> AppResult<User> {
        // Hash the password before storing
        new_user.password = hash_password(&new_user.password)?;
        self.repo.create(new_user).await
    }

    /// Gets a user by their ID.
    ///
    /// # Arguments
    /// * `id` - The user's ID
    ///
    /// # Returns
    /// The user if found, or `NotFound` error
    pub async fn get_user(&self, id: i32) -> AppResult<User> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound {
                entity: "user".to_string(),
                field: "id".to_string(),
                value: id.to_string(),
            })
    }

    /// Gets a user by their email address.
    ///
    /// # Arguments
    /// * `email` - The user's email address
    ///
    /// # Returns
    /// `Some(User)` if found, `None` otherwise
    pub async fn get_user_by_email(&self, email: &str) -> AppResult<Option<User>> {
        self.repo.find_by_email(email).await
    }

    /// Lists all users.
    ///
    /// # Returns
    /// A vector of all users
    pub async fn list_users(&self) -> AppResult<Vec<User>> {
        self.repo.list_all().await
    }

    /// Lists users with pagination.
    ///
    /// # Arguments
    /// * `offset` - Number of records to skip
    /// * `limit` - Maximum number of records to return
    ///
    /// # Returns
    /// A tuple of (users, total_count)
    pub async fn list_users_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<User>, i64)> {
        self.repo.list_paginated(offset, limit).await
    }

    /// Updates a user's data.
    ///
    /// # Arguments
    /// * `id` - The user's ID
    /// * `update_data` - The fields to update
    ///
    /// # Returns
    /// The updated user
    ///
    /// # Note
    /// If password is provided, it will be automatically hashed before storing
    pub async fn update_user(&self, id: i32, mut update_data: UpdateUser) -> AppResult<User> {
        // Verify user exists first
        self.get_user(id).await?;
        
        // Hash the password if it's being updated
        if let Some(password) = update_data.password {
            update_data.password = Some(hash_password(&password)?);
        }
        
        self.repo.update(id, update_data).await
    }

    /// Deletes a user.
    ///
    /// # Arguments
    /// * `id` - The user's ID
    ///
    /// # Returns
    /// `true` if the user was deleted, `false` if not found
    pub async fn delete_user(&self, id: i32) -> AppResult<bool> {
        let affected = self.repo.delete(id).await?;
        Ok(affected > 0)
    }

    /// Verifies a user's password.
    ///
    /// # Arguments
    /// * `email` - The user's email address
    /// * `password` - The plain text password to verify
    ///
    /// # Returns
    /// `Some(User)` if credentials are valid, `None` if user not found or password incorrect
    pub async fn verify_credentials(&self, email: &str, password: &str) -> AppResult<Option<User>> {
        if let Some(user) = self.get_user_by_email(email).await? {
            if verify_password(password, &user.password)? {
                return Ok(Some(user));
            }
        }
        Ok(None)
    }

    /// Authenticates a user and generates JWT tokens.
    ///
    /// # Arguments
    /// * `email` - The user's email address
    /// * `password` - The plain text password
    /// * `jwt_secret` - The secret key for signing the JWT
    /// * `access_expiration_hours` - Access token validity duration in hours (typically 1-24)
    /// * `refresh_expiration_hours` - Refresh token validity duration in hours (typically 168-720)
    ///
    /// # Returns
    /// A tuple of (User, access_token, refresh_token) if credentials are valid, or Unauthorized error
    pub async fn authenticate(
        &self,
        email: &str,
        password: &str,
        jwt_secret: &str,
        access_expiration_hours: i64,
        refresh_expiration_hours: i64,
    ) -> AppResult<(User, String, String)> {
        let user = self
            .verify_credentials(email, password)
            .await?
            .ok_or(AppError::Unauthorized {
                message: "Invalid email or password".to_string(),
            })?;

        let (access_token, refresh_token) = generate_token_pair(
            user.id,
            user.email.clone(),
            user.username.clone(),
            jwt_secret,
            access_expiration_hours,
            refresh_expiration_hours,
        )?;

        Ok((user, access_token, refresh_token))
    }
}
