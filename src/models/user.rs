use diesel::prelude::*;
use jiff_diesel::DateTime;
use serde::Deserialize;

/// User model for reading from database
/// Derives Queryable for SELECT operations and Selectable for type-safe column selection
#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

/// NewUser model for inserting new records
/// Derives Insertable for INSERT operations
#[derive(Debug, Insertable, Deserialize, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// UpdateUser model for partial updates
/// Derives AsChangeset for UPDATE operations with optional fields
#[derive(Debug, AsChangeset, Deserialize, Clone, Default)]
#[diesel(table_name = crate::schema::users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}
