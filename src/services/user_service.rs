//! User service for business logic operations.
//!
//! Provides a higher-level API for user operations, encapsulating
//! business rules and coordinating with the repository layer.

use crate::error::{AppError, AppResult};
use crate::models::{NewUser, UpdateUser, User};
use crate::repositories::UserRepository;

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
    pub async fn create_user(&self, new_user: NewUser) -> AppResult<User> {
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
    pub async fn update_user(&self, id: i32, update_data: UpdateUser) -> AppResult<User> {
        // Verify user exists first
        self.get_user(id).await?;
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
}
