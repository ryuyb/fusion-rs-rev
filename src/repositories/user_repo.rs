//! User repository for async database operations.
//!
//! Provides CRUD operations for the users table using diesel_async.

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::db::AsyncDbPool;
use crate::error::AppError;
use crate::models::{NewUser, UpdateUser, User};

/// User repository holding an async connection pool.
///
/// Since `AsyncDbPool` (bb8::Pool) internally uses `Arc`, cloning is cheap
/// (just reference count increment). No need for `Arc<UserRepository>`.
#[derive(Clone)]
pub struct UserRepository {
    pool: AsyncDbPool,
}

impl UserRepository {
    /// Creates a new UserRepository with the given connection pool.
    pub fn new(pool: AsyncDbPool) -> Self {
        Self { pool }
    }

    /// Creates a new user in the database.
    ///
    /// # Arguments
    /// * `new_user` - The user data to insert
    ///
    /// # Returns
    /// The created user with generated id and timestamps
    pub async fn create(&self, new_user: NewUser) -> Result<User, AppError> {
        use crate::schema::users::dsl::*;
        let mut conn = self.pool.get().await?;

        diesel::insert_into(users)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::from)
    }

    /// Finds a user by their ID.
    ///
    /// # Arguments
    /// * `user_id` - The user's ID
    ///
    /// # Returns
    /// `Some(User)` if found, `None` otherwise
    pub async fn find_by_id(&self, user_id: i32) -> Result<Option<User>, AppError> {
        use crate::schema::users::dsl::*;
        let mut conn = self.pool.get().await?;

        users
            .filter(id.eq(user_id))
            .select(User::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::from)
    }


    /// Finds a user by their email address.
    ///
    /// # Arguments
    /// * `user_email` - The user's email address
    ///
    /// # Returns
    /// `Some(User)` if found, `None` otherwise
    pub async fn find_by_email(&self, user_email: &str) -> Result<Option<User>, AppError> {
        use crate::schema::users::dsl::*;
        let mut conn = self.pool.get().await?;

        users
            .filter(email.eq(user_email))
            .select(User::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::from)
    }

    /// Lists all users in the database.
    ///
    /// # Returns
    /// A vector of all users
    pub async fn list_all(&self) -> Result<Vec<User>, AppError> {
        use crate::schema::users::dsl::*;
        let mut conn = self.pool.get().await?;

        users
            .select(User::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::from)
    }

    /// Updates a user's data.
    ///
    /// # Arguments
    /// * `user_id` - The user's ID
    /// * `update_data` - The fields to update (None fields are ignored)
    ///
    /// # Returns
    /// The updated user
    pub async fn update(&self, user_id: i32, update_data: UpdateUser) -> Result<User, AppError> {
        use crate::schema::users::dsl::*;
        let mut conn = self.pool.get().await?;

        diesel::update(users.filter(id.eq(user_id)))
            .set(&update_data)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::from)
    }

    /// Deletes a user from the database.
    ///
    /// # Arguments
    /// * `user_id` - The user's ID
    ///
    /// # Returns
    /// The number of affected rows (0 or 1)
    pub async fn delete(&self, user_id: i32) -> Result<usize, AppError> {
        use crate::schema::users::dsl::*;
        let mut conn = self.pool.get().await?;

        diesel::delete(users.filter(id.eq(user_id)))
            .execute(&mut conn)
            .await
            .map_err(AppError::from)
    }
}
