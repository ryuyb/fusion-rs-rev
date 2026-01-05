# fusion-rs

A production-ready Rust web application built with Axum, Diesel, and Tokio. Provides a complete backend API with user authentication, notification system, job scheduling, and live streaming platform integrations.

## Features

- **User Authentication** - JWT-based authentication with access and refresh tokens
- **User Management** - Complete CRUD operations for user accounts
- **Notification System** - Flexible notification channels with webhook support
- **Job Scheduling** - Cron-based background job system with retry logic and concurrency control
- **Live Platform Integration** - Support for Bilibili and Douyin live streaming platforms
- **OpenAPI Documentation** - Auto-generated Swagger UI for API exploration
- **Structured Logging** - Comprehensive logging with file rotation and compression
- **Database Migrations** - Diesel-powered PostgreSQL migrations

## Prerequisites

- Rust 1.70+ (with cargo)
- PostgreSQL 12+
- Diesel CLI (for database migrations)

## Installation

### 1. Install Diesel CLI

```bash
cargo install diesel_cli --no-default-features --features postgres
```

### 2. Clone the repository

```bash
git clone <repository-url>
cd fusion-rs
```

### 3. Set up the database

Create a PostgreSQL database and set the connection URL:

```bash
export DATABASE_URL="postgresql://username:password@localhost/fusion_rs"
```

Run migrations:

```bash
diesel migration run
```

### 4. Configure the application

Create a `config/local.toml` file (not committed to git):

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "postgresql://username:password@localhost/fusion_rs"

[jwt]
secret = "your-secret-key-here"
access_token_expiration = 3600
refresh_token_expiration = 604800

[logger]
level = "info"
```

See `config/default.toml` for all available configuration options.

## Usage

### Running the server

```bash
# Run with default configuration
cargo run

# Run with explicit serve command
cargo run -- serve

# Validate configuration without starting
cargo run -- serve --dry-run

# Override configuration via CLI
cargo run -- serve --host 0.0.0.0 --port 3000 --log-level debug
```

### Running database migrations

```bash
cargo run -- migrate
```

## API Documentation

Once the server is running, access the interactive API documentation at:

- Swagger UI: `http://localhost:8080/swagger-ui`
- OpenAPI JSON: `http://localhost:8080/api-docs/openapi.json`

### Authentication

#### Register a new user

```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "securepassword"
  }'
```

#### Login

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "securepassword"
  }'
```

Response:
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

#### Refresh token

```bash
curl -X POST http://localhost:8080/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "eyJ..."
  }'
```

### Protected Endpoints

Include the access token in the Authorization header:

```bash
curl -X GET http://localhost:8080/api/me \
  -H "Authorization: Bearer eyJ..."
```

### Key Endpoints

**Authentication**
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - User login
- `POST /api/auth/refresh` - Refresh access token

**Users**
- `GET /api/me` - Get current user info
- `GET /api/users` - List users (paginated)
- `POST /api/users` - Create user
- `GET /api/users/:id` - Get user by ID
- `PUT /api/users/:id` - Update user
- `DELETE /api/users/:id` - Delete user

**Notifications**
- `GET /api/notifications/channels` - List notification channels
- `POST /api/notifications/channels` - Create notification channel
- `PUT /api/notifications/channels/:id` - Update channel
- `DELETE /api/notifications/channels/:id` - Delete channel
- `POST /api/notifications/send` - Send notification

**Jobs**
- `GET /api/jobs` - List scheduled jobs
- `POST /api/jobs` - Create scheduled job
- `PUT /api/jobs/:id` - Update job
- `DELETE /api/jobs/:id` - Delete job
- `GET /api/jobs/:id/executions` - Get job execution history

**Health**
- `GET /health` - Health check endpoint

## Configuration

Configuration is loaded from TOML files in the `config/` directory with the following priority (highest to lowest):

1. Environment variables (`FUSION_*`)
2. `config/local.toml` (not committed)
3. `config/{environment}.toml` (development.toml, production.toml)
4. `config/default.toml` (base configuration)

### Configuration Sections

**Application**
```toml
[application]
name = "fusion-rs"
version = "0.1.0"
```

**Server**
```toml
[server]
host = "127.0.0.1"
port = 8080
request_timeout = 30
shutdown_timeout = 10
```

**Database**
```toml
[database]
url = "postgresql://localhost/fusion_rs"
max_connections = 10
min_connections = 2
connection_timeout = 30
idle_timeout = 600
max_lifetime = 1800
auto_migrate = false
```

**JWT**
```toml
[jwt]
secret = "your-secret-key"
access_token_expiration = 3600      # 1 hour
refresh_token_expiration = 604800   # 7 days
```

**Logger**
```toml
[logger]
level = "info"
console_enabled = true
file_enabled = true
file_path = "logs/fusion-rs.log"
rotation_strategy = "size"
max_file_size = 10485760  # 10MB
max_backup_count = 5
compress_rotated = true
```

**Jobs**
```toml
[jobs]
enabled = true
job_timeout = 300
max_retries = 3
retry_delay = 60
history_retention_days = 30
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests in a module
cargo test logger::

# Run ignored tests (network tests)
cargo test -- --ignored
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run linter with all warnings
cargo clippy -- -W clippy::all
```

### Database Management

```bash
# Create a new migration
diesel migration generate migration_name

# Run migrations
diesel migration run

# Revert last migration
diesel migration revert

# Regenerate schema.rs (after migrations)
diesel print-schema > src/schema.rs
```

## Live Platform Integration

### Bilibili

Fetch live room information:

```rust
use fusion_rs::external::live::{get_provider, LivePlatform};

let provider = get_provider(LivePlatform::Bilibili);
let room_info = provider.get_room_info("123456").await?;
let anchor_info = provider.get_anchor_info("789012").await?;
```

### Douyin

Fetch Douyin live room information:

```rust
use fusion_rs::external::live::{get_provider, LivePlatform};

let provider = get_provider(LivePlatform::Douyin);
let room_info = provider.get_room_info("https://live.douyin.com/123456").await?;
let anchor_info = provider.get_anchor_info("MS4wLjABAAAA...").await?;
```

## Job Scheduling

Create a scheduled job via API:

```bash
curl -X POST http://localhost:8080/api/jobs \
  -H "Authorization: Bearer eyJ..." \
  -H "Content-Type: application/json" \
  -d '{
    "job_name": "cleanup_old_logs",
    "job_type": "data_cleanup",
    "cron_expression": "0 0 * * *",
    "enabled": true,
    "allow_concurrent": false,
    "max_retries": 3,
    "payload": {
      "retention_days": 30
    },
    "description": "Clean up old job execution logs"
  }'
```

The job scheduler will automatically execute jobs based on their cron expressions.

## Architecture

The project follows a layered architecture pattern:

```
API Layer (Handlers) → Service Layer (Business Logic) → Repository Layer (Data Access)
```

Key directories:
- `src/api/` - HTTP handlers, DTOs, middleware
- `src/services/` - Business logic
- `src/repositories/` - Database access
- `src/models/` - Diesel models
- `src/external/` - External API integrations
- `src/jobs/` - Background job system
- `src/config/` - Configuration management

For detailed architecture documentation, see `CLAUDE.md` and `AGENTS.md`.

## Security

- Passwords are hashed using Argon2
- JWT tokens for authentication (access tokens: 1 hour, refresh tokens: 7 days)
- SQL injection prevention via Diesel query builder
- Request validation using the `validator` crate
- CORS configuration for cross-origin requests

## License

Copyright (C) 2025

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
