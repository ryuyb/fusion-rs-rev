# Requirements Document

## Introduction

本功能将 Axum Web 框架集成到 fusion-rs 项目中，实现完整的 HTTP API 层。包括路由注册、请求处理器、中间件（日志、请求追踪 ID、全局错误处理）以及将 Services 注入到 Axum State 中。不包含认证中间件的实现。

## Glossary

- **Axum_Router**: Axum 框架的路由器，负责将 HTTP 请求路由到对应的处理器
- **Handler**: 请求处理器，接收 HTTP 请求并返回响应
- **Middleware**: 中间件，在请求处理前后执行的逻辑层
- **AppState**: 应用状态，包含 Services 等共享资源，通过 Axum 的 State 机制注入到处理器
- **Request_ID**: 请求追踪标识符，用于在日志中关联同一请求的所有操作
- **DTO**: Data Transfer Object，用于 API 请求和响应的数据结构

## Requirements

### Requirement 1: 应用状态管理

**User Story:** 作为开发者，我希望将 Services 封装到 AppState 中，以便在所有请求处理器中共享业务逻辑服务。

#### Acceptance Criteria

1. THE AppState SHALL contain a Services instance for accessing business logic
2. THE AppState SHALL implement Clone trait for Axum state sharing
3. WHEN AppState is created, THE System SHALL initialize database connection pool and all services

### Requirement 2: 路由注册

**User Story:** 作为开发者，我希望有一个集中的路由注册机制，以便管理所有 API 端点。

#### Acceptance Criteria

1. THE Axum_Router SHALL provide a `/api/users` endpoint group for user operations
2. WHEN a GET request is made to `/api/users`, THE Handler SHALL return a list of all users
3. WHEN a GET request is made to `/api/users/{id}`, THE Handler SHALL return the user with the specified ID
4. WHEN a POST request is made to `/api/users` with valid user data, THE Handler SHALL create a new user
5. WHEN a PUT request is made to `/api/users/{id}` with update data, THE Handler SHALL update the specified user
6. WHEN a DELETE request is made to `/api/users/{id}`, THE Handler SHALL delete the specified user
7. THE Axum_Router SHALL apply middleware layers in the correct order

### Requirement 3: 用户请求处理器

**User Story:** 作为开发者，我希望有完整的用户 CRUD 处理器，以便通过 HTTP API 管理用户数据。

#### Acceptance Criteria

1. WHEN a valid NewUser JSON is received, THE Handler SHALL deserialize it and call UserService.create_user
2. WHEN an invalid JSON body is received, THE Handler SHALL return a 400 Bad Request error with details
3. WHEN a user is successfully created, THE Handler SHALL return 201 Created with the user data
4. WHEN a user is not found, THE Handler SHALL return 404 Not Found error
5. WHEN listing users, THE Handler SHALL return 200 OK with an array of users
6. WHEN a user is successfully updated, THE Handler SHALL return 200 OK with the updated user data
7. WHEN a user is successfully deleted, THE Handler SHALL return 204 No Content

### Requirement 4: 请求日志中间件

**User Story:** 作为运维人员，我希望所有 HTTP 请求都被记录日志，以便监控和调试 API 调用。

#### Acceptance Criteria

1. WHEN a request is received, THE Middleware SHALL log the HTTP method, path, and request ID
2. WHEN a response is sent, THE Middleware SHALL log the status code and response time
3. THE Middleware SHALL use tracing spans to correlate request and response logs
4. THE Middleware SHALL include the request ID in all log entries for the request

### Requirement 5: 请求追踪 ID 中间件

**User Story:** 作为开发者，我希望每个请求都有唯一的追踪 ID，以便在分布式系统中追踪请求流程。

#### Acceptance Criteria

1. WHEN a request does not contain X-Request-ID header, THE Middleware SHALL generate a new UUID
2. WHEN a request contains X-Request-ID header, THE Middleware SHALL use the provided value
3. THE Middleware SHALL add the request ID to the response headers as X-Request-ID
4. THE Middleware SHALL store the request ID in request extensions for downstream access

### Requirement 6: 全局错误处理

**User Story:** 作为开发者，我希望有统一的错误处理机制，以便返回一致的错误响应格式。

#### Acceptance Criteria

1. WHEN an AppError occurs, THE Handler SHALL convert it to an appropriate HTTP status code
2. WHEN AppError::NotFound occurs, THE Handler SHALL return 404 status code
3. WHEN AppError::Database occurs, THE Handler SHALL return 500 status code
4. WHEN AppError::Pool occurs, THE Handler SHALL return 503 status code
5. THE Error_Response SHALL include a JSON body with error message and optional details
6. THE Error_Response SHALL include the request ID for correlation

### Requirement 7: HTTP 服务器启动

**User Story:** 作为运维人员，我希望能够启动 HTTP 服务器监听配置的地址和端口。

#### Acceptance Criteria

1. WHEN the application starts, THE Server SHALL bind to the configured host and port from Settings
2. WHEN the server starts successfully, THE System SHALL log the listening address
3. IF the server fails to bind, THEN THE System SHALL return an error with details
4. THE Server SHALL use graceful shutdown when receiving termination signals

### Requirement 8: API 响应 DTO

**User Story:** 作为开发者，我希望有专门的 DTO 结构用于 API 响应，以便控制返回给客户端的数据格式。

#### Acceptance Criteria

1. THE UserResponse DTO SHALL exclude sensitive fields like password from the response
2. THE UserResponse DTO SHALL include id, username, email, created_at, and updated_at fields
3. WHEN serializing User to UserResponse, THE System SHALL convert timestamps to ISO 8601 format
4. THE Error_Response DTO SHALL include error code, message, and optional details fields
