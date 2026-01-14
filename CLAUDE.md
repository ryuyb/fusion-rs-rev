# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

fusion-rs is a production-ready Rust web application built with Axum, Diesel (PostgreSQL), and Tokio. It provides a complete backend API with user authentication, notification system, job scheduling, and external live streaming platform integrations (Bilibili and Douyin).

## Common Commands

### Development
```bash
export DATABASE_URL="postgres://postgres:postgres@localhost/fusion_rs"  # Set database URL
cargo build                    # Debug build
cargo build --release          # Release build (optimized)
cargo run                      # Run with default config
cargo run -- serve             # Explicit serve command
cargo run -- serve --dry-run   # Validate config without starting
cargo run -- migrate           # Run database migrations
cargo fmt                      # Format code
cargo clippy                   # Run linter
```

### Testing
```bash
cargo test                     # Run all tests
cargo test <test_name>         # Run single test by name
cargo test <module>::          # Run tests in module (e.g., cargo test logger::)
cargo test -- --ignored        # Run ignored tests (network tests)
```

### Database
```bash
export DATABASE_URL="postgres://postgres:postgres@localhost/fusion_rs"
diesel migration run           # Apply migrations
diesel migration revert        # Revert last migration
diesel print-schema            # Regenerate schema.rs (don't edit manually)
```

## Architecture

### Layered Architecture Pattern
```
API Layer (Handlers) → Service Layer (Business Logic) → Repository Layer (Data Access)
```

### Key Directories
- `src/api/handlers/` - Request handlers by domain (auth, users, notifications, jobs)
- `src/api/dto/` - Data Transfer Objects with validation
- `src/api/middleware/` - Axum middleware (auth, logging, error handling)
- `src/services/` - Business logic layer
- `src/repositories/` - Data access layer
- `src/models/` - Diesel models (Query, Insert, Update)
- `src/schema.rs` - Auto-generated Diesel schema (DO NOT EDIT)
- `src/external/live/` - Live streaming platform integrations (Bilibili, Douyin)
- `src/jobs/` - Background job scheduling system
- `src/config/` - Configuration management (TOML files)
- `src/error/` - Centralized error handling

### Dependency Injection via AppState
```rust
pub struct AppState {
    pub services: Services,           // All business logic services
    pub db_pool: AsyncDbPool,          // Direct DB access
    pub jwt_config: JwtConfig,         // JWT configuration
    pub scheduler: Option<Arc<JobScheduler>>, // Optional job scheduler
}
```

## Code Patterns

### Three-Model Pattern (Diesel)
For each entity, define three models:
```rust
#[derive(Queryable, Selectable)]
pub struct User { ... }           // Reading from database

#[derive(Insertable)]
pub struct NewUser { ... }        // Inserting new records

#[derive(AsChangeset, Default)]
pub struct UpdateUser { ... }     // Updating existing records
```

### Error Handling
All functions return `AppResult<T>` = `Result<T, AppError>`:
```rust
pub async fn get_user(&self, id: i32) -> AppResult<User> {
    self.repo.find_by_id(id).await?.ok_or(AppError::NotFound {
        entity: "user".to_string(),
        field: "id".to_string(),
        value: id.to_string(),
    })
}
```

Common error variants: `NotFound`, `Duplicate`, `Validation`, `BadRequest`, `Unauthorized`, `Forbidden`, `Database`, `ExternalApi`, `Internal`

### Trait-Based Extensibility

#### Live Platform Provider
```rust
#[async_trait]
pub trait LivePlatformProvider: Send + Sync {
    fn platform(&self) -> LivePlatform;
    async fn get_room_info(&self, room_id: &str) -> AppResult<RoomInfo>;
    async fn get_anchor_info(&self, uid: &str) -> AppResult<AnchorInfo>;
    async fn get_rooms_status_by_uids(&self, uids: &[&str]) -> AppResult<HashMap<String, RoomStatusInfo>>;
}
```

Factory pattern: `get_provider(platform: LivePlatform) -> &'static dyn LivePlatformProvider`

#### Notification Provider
```rust
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    async fn send(&self, message: &NotificationMessage) -> AppResult<NotificationResult>;
    fn validate_config(config: &JsonValue) -> AppResult<()>;
    fn provider_name(&self) -> &'static str;
}
```

#### Job Task
```rust
#[async_trait]
pub trait JobTask: Send + Sync + Debug {
    fn task_type() -> &'static str where Self: Sized;
    async fn execute(&self, ctx: JobContext) -> JobResult<()>;
    fn description(&self) -> Option<String>;
}
```

## External Integrations

### Bilibili Live Platform
- Simple HTTP GET/POST requests
- Endpoints: room info, anchor info, batch status
- Live status mapping: 0=offline, 1=live, 2=replay

### Douyin Live Platform
- Complex anti-bot measures (cookie management, a_bogus generation, signature signing)
- User-Agent rotation
- Short URL resolution
- Regex-based data extraction from HTML
- Batch operations with chunking (3 concurrent requests)
- Key files: `src/external/live/douyin/abogus.rs`, `src/external/live/douyin/sign.rs`

### Shared HTTP Client
Use the shared `HTTP_CLIENT` (LazyLock) in `src/external/client.rs` for all external API calls.

## Configuration System

### Priority (highest to lowest)
1. Environment variables (`FUSION_*`)
2. `local.toml` (not committed)
3. `{environment}.toml` (development.toml, production.toml)
4. `default.toml` (base configuration)

### Sections
- **Application** - name, version
- **Server** - host, port, timeouts
- **Database** - url, connection pool settings, auto_migrate
- **JWT** - secret, access_token_expiration, refresh_token_expiration
- **Logger** - level, console/file output, rotation, compression
- **Jobs** - enabled, job_timeout, max_retries, retry_delay, history_retention_days

## Job Scheduling System

### Architecture
- **JobScheduler** - Loads enabled jobs from database, schedules based on cron expressions
- **JobExecutor** - Manages execution lifecycle, records history, handles retries with exponential backoff
- **JobRegistry** - Registers task types and creates task instances from job payload

### Concurrency Control
1. No concurrency (`allow_concurrent=false`) - Only one instance at a time
2. Limited concurrency (`allow_concurrent=true, max_concurrent=N`) - Up to N instances
3. Unlimited concurrency (`allow_concurrent=true, max_concurrent=NULL`) - No limit

## Adding New Features

### Adding a New Entity
1. Create migration in `migrations/`
2. Run `diesel migration run` to update `schema.rs`
3. Create model in `src/models/` (Query, Insert, Update)
4. Create repository in `src/repositories/`
5. Create service in `src/services/`
6. Create DTOs in `src/api/dto/`
7. Create handlers in `src/api/handlers/`
8. Register routes in `src/api/routes.rs`
9. Update `Repositories` and `Services` aggregators

### Adding External Integration
1. Create module in `src/external/`
2. Define trait for provider interface
3. Implement trait for specific provider
4. Add factory function or enum for provider selection
5. Use shared `HTTP_CLIENT` for requests
6. Add comprehensive error handling

### Adding Background Job
1. Create task struct in `src/jobs/tasks/`
2. Implement `JobTask` trait
3. Register task in `JobRegistry`
4. Create job via API or database
5. Job scheduler will execute automatically

## Code Style

### Imports (ordered, grouped by blank lines)
```rust
use std::path::PathBuf;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
```

### Naming Conventions
- Types: `PascalCase` (UserService, AppError)
- Functions: `snake_case` (create_user)
- Constants: `SCREAMING_SNAKE_CASE`
- DB tables: `snake_case` plural (users)

### DTOs with Validation
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

## Git Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <description>

[optional body]
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `ci`

Rules:
- Use imperative mood ("add" not "added")
- Lowercase first letter after type
- No period at end
- Keep subject line under 72 characters

## Key Dependencies

- `axum 0.8` - Web framework
- `diesel 2.3 + diesel-async 0.7` - PostgreSQL ORM with async support
- `tokio` - Async runtime
- `reqwest` - HTTP client for external APIs
- `tokio-cron-scheduler 0.15` - Background job scheduling
- `utoipa + utoipa-swagger-ui` - OpenAPI documentation
- `validator` - Request validation
- `jsonwebtoken` - JWT authentication
- `argon2` - Password hashing

## Security Features

- Password hashing with Argon2
- JWT authentication (access tokens: 1 hour, refresh tokens: 7 days)
- CORS configuration
- Request validation using `validator` crate
- SQL injection prevention via Diesel query builder
- Static linking (bundled PostgreSQL and OpenSSL)

## Documentation

- Code comments on public APIs
- OpenAPI/Swagger auto-generated from utoipa annotations
- Design docs in `docs/` (Chinese):
  - `project-structure.md` - Directory layout
  - `notification-design.md` - Notification system design
  - `job-design.md` - Job scheduler design
- `AGENTS.md` - Coding guidelines for AI assistants
