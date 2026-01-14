# AGENTS.md - Coding Agent Guidelines for fusion-rs

## Project Overview

Rust web application using Axum, Diesel (PostgreSQL), and tokio. Layered architecture: handlers -> services -> repositories.

## Build & Test Commands

```bash
export DATABASE_URL="postgres://postgres:postgres@localhost/fusion_rs"  # Set database URL
cargo build                    # Debug build
cargo build --release          # Release build
cargo run                      # Run with default config
cargo run -- serve             # Explicit serve command
cargo run -- migrate           # Run database migrations
cargo fmt                      # Format code
cargo clippy                   # Run linter
cargo test                     # Run all tests
cargo test <test_name>         # Run single test by name
cargo test <module>::          # Run tests in module (e.g., cargo test logger::)
```

Or with inline DATABASE_URL:
```bash
DATABASE_URL="postgres://postgres:postgres@localhost/fusion_rs" diesel migration run
```

## Project Structure

```
src/
  api/handlers/         # Request handlers
  api/dto/              # Data Transfer Objects
  api/middleware/       # Axum middleware
  services/             # Business logic
  repositories/         # Data access
  models/               # Diesel models
  schema.rs             # Diesel schema (auto-generated, don't edit)
  config/               # Configuration
  error/                # Error types
  jobs/                 # Background jobs
  logger/               # Logging
  utils/                # Utilities
```

## Code Style

### Imports (ordered, grouped by blank lines)
```rust
use std::path::PathBuf;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
```

### Naming
- Types: `PascalCase` (UserService, AppError)
- Functions: `snake_case` (create_user)
- Constants: `SCREAMING_SNAKE_CASE`
- DB tables: `snake_case` plural (users)

### Error Handling - Use `AppError` and `AppResult<T>`
```rust
pub async fn get_user(&self, id: i32) -> AppResult<User> {
    self.repo.find_by_id(id).await?.ok_or(AppError::NotFound {
        entity: "user".to_string(),
        field: "id".to_string(),
        value: id.to_string(),
    })
}
```
Variants: `NotFound`, `Duplicate`, `Validation`, `BadRequest`, `Unauthorized`, `Forbidden`, `Database`, `Internal`

### Models (Diesel) - Three types per entity
```rust
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
pub struct User { ... }           // Reading

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser { ... }        // Inserting

#[derive(AsChangeset, Default)]
#[diesel(table_name = crate::schema::users)]
pub struct UpdateUser { ... }     // Updating
```

### DTOs with validation
```rust
#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 20))]
    pub username: String,
}
```

### Handlers
```rust
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<UserResponse>> {
    let user = state.services.users.get_user(id).await?;
    Ok(Json(UserResponse::from(user)))
}
```

### Services (business logic)
```rust
#[derive(Clone)]
pub struct UserService { repo: UserRepository }

impl UserService {
    pub async fn create_user(&self, mut new_user: NewUser) -> AppResult<User> {
        new_user.password = hash_password(&new_user.password)?;
        self.repo.create(new_user).await
    }
}
```

### Repositories (data access)
```rust
pub async fn find_by_id(&self, user_id: i32) -> AppResult<Option<User>> {
    use crate::schema::users::dsl::*;
    let mut conn = self.pool.get().await.map_err(|e| AppError::ConnectionPool {
        source: anyhow::Error::from(e),
    })?;
    users.filter(id.eq(user_id)).first(&mut conn).await.optional().map_err(AppError::from)
}
```

### Testing (proptest for property-based)
```rust
proptest! {
    #[test]
    fn property_valid_configs(enabled in any::<bool>()) {
        prop_assume!(enabled);
        prop_assert!(config.validate().is_ok());
    }
}
```

## Key Dependencies

- `axum` - Web framework
- `diesel` + `diesel-async` - Async PostgreSQL ORM
- `tokio` - Async runtime
- `serde` / `thiserror` / `validator` / `utoipa` / `proptest`

## Database

- Migrations: `migrations/` - run with `diesel migration run`
- Schema: `src/schema.rs` (auto-generated) - regenerate with `diesel print-schema`

**Required environment setup:**
```bash
export DATABASE_URL="postgres://postgres:postgres@localhost/fusion_rs"
```

Or for commands that don't support env vars:
```bash
DATABASE_URL="postgres://postgres:postgres@localhost/fusion_rs" diesel migration run
```

## Configuration

TOML files in `config/`. Environment variables override file settings.

## Git Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <description>

[optional body]
```

### Types
- `feat` - New feature
- `fix` - Bug fix
- `refactor` - Code change that neither fixes a bug nor adds a feature
- `docs` - Documentation only
- `test` - Adding or updating tests
- `chore` - Maintenance tasks (deps, configs)
- `perf` - Performance improvement
- `ci` - CI/CD changes

### Examples
```
feat: add user authentication endpoint
fix: resolve database connection timeout
refactor: extract validation logic to separate module
docs: update API documentation
chore: upgrade diesel to 2.3
```

### Rules
- Use imperative mood ("add" not "added")
- Lowercase first letter after type
- No period at end
- Keep subject line under 72 characters
