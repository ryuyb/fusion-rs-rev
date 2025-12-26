# Axum 错误处理指南

本指南介绍了如何将 axum 的各种错误转换为统一的 `ErrorResponse` 格式。

## 概述

我们的错误处理系统提供了以下功能：

1. **AppError 自动转换**：`AppError` 实现了 `IntoResponse` trait，可以自动转换为 HTTP 响应
2. **Axum 内置错误处理**：提供了处理 JSON、路径参数、查询参数错误的函数
3. **全局错误处理中间件**：捕获任何未处理的错误并转换为统一格式
4. **统一的错误响应格式**：所有错误都使用 `ErrorResponse` 结构

## 核心组件

### 1. AppError 枚举

`AppError` 是我们的主要错误类型，包含以下变体：

```rust
pub enum AppError {
    NotFound { entity: String, field: String, value: String },
    Duplicate { entity: String, field: String, value: String },
    Validation { field: String, reason: String },
    BadRequest { message: String },
    UnprocessableContent { message: String },
    Unauthorized { message: String },
    Forbidden { message: String },
    Database { operation: String, source: anyhow::Error },
    Configuration { key: String, source: anyhow::Error },
    ConnectionPool { source: anyhow::Error },
    Internal { source: anyhow::Error },
}
```

### 2. ErrorResponse 结构

统一的错误响应格式：

```rust
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub request_id: Option<String>,
}
```

### 3. 错误处理函数

我们提供了以下函数来处理 axum 的内置错误：

- `handle_json_rejection(JsonRejection) -> Response`
- `handle_path_rejection(PathRejection) -> Response`
- `handle_query_rejection(QueryRejection) -> Response`

### 4. 全局错误处理中间件

`global_error_handler` 中间件会捕获任何未处理的错误并转换为统一的 `ErrorResponse` 格式。

支持的 HTTP 状态码转换：
- 400 BAD_REQUEST → "BAD_REQUEST"
- 404 NOT_FOUND → "NOT_FOUND"  
- 405 METHOD_NOT_ALLOWED → "METHOD_NOT_ALLOWED"
- 408 REQUEST_TIMEOUT → "REQUEST_TIMEOUT"
- 413 PAYLOAD_TOO_LARGE → "PAYLOAD_TOO_LARGE"
- 415 UNSUPPORTED_MEDIA_TYPE → "UNSUPPORTED_MEDIA_TYPE"
- 500 INTERNAL_SERVER_ERROR → "INTERNAL_SERVER_ERROR"
- 502 BAD_GATEWAY → "BAD_GATEWAY"
- 503 SERVICE_UNAVAILABLE → "SERVICE_UNAVAILABLE"
- 504 GATEWAY_TIMEOUT → "GATEWAY_TIMEOUT"

## 使用方法

### 1. 基本的 AppError 使用

```rust
use crate::error::AppError;

async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, AppError> {
    if id == 0 {
        return Err(AppError::NotFound {
            entity: "user".to_string(),
            field: "id".to_string(),
            value: id.to_string(),
        });
    }
    
    // ... 查找用户逻辑
    Ok(Json(user))
}
```

### 2. 处理 JSON 解析错误

```rust
use axum::extract::rejection::JsonRejection;
use crate::api::middleware::handle_json_rejection;

async fn create_user(
    payload: Result<Json<CreateUserRequest>, JsonRejection>,
) -> Response {
    match payload {
        Ok(Json(user_data)) => {
            // 正常处理逻辑
            (StatusCode::CREATED, Json(created_user)).into_response()
        }
        Err(rejection) => {
            // 使用我们的错误处理函数
            handle_json_rejection(rejection)
        }
    }
}
```

### 3. 处理路径参数错误

```rust
use axum::extract::rejection::PathRejection;
use crate::api::middleware::handle_path_rejection;

async fn get_user(
    path: Result<Path<u32>, PathRejection>,
) -> Response {
    match path {
        Ok(Path(user_id)) => {
            // 正常处理逻辑
            Json(user).into_response()
        }
        Err(rejection) => {
            handle_path_rejection(rejection)
        }
    }
}
```

### 4. 配置全局错误处理中间件

```rust
use crate::api::middleware::global_error_handler;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api", api_routes)
        .layer(middleware::from_fn(logging_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(global_error_handler))  // 添加全局错误处理
        .with_state(state)
}
```

## 错误响应格式

所有错误都会转换为以下 JSON 格式：

```json
{
  "code": "NOT_FOUND",
  "message": "Resource not found: user with id=123",
  "details": {
    "entity": "user",
    "field": "id",
    "value": "123"
  },
  "request_id": "req-456"
}
```

## HTTP 状态码映射

| AppError 变体 | HTTP 状态码 | 错误代码 |
|---------------|-------------|----------|
| NotFound | 404 | NOT_FOUND |
| Duplicate | 409 | DUPLICATE_ENTRY |
| Validation | 400 | VALIDATION_ERROR |
| BadRequest | 400 | BAD_REQUEST |
| UnprocessableContent | 422 | UNPROCESSABLE_CONTENT |
| Unauthorized | 401 | UNAUTHORIZED |
| Forbidden | 403 | FORBIDDEN |
| Database | 500 | DATABASE_ERROR |
| Configuration | 500 | CONFIGURATION_ERROR |
| ConnectionPool | 503 | SERVICE_UNAVAILABLE |
| Internal | 500 | INTERNAL_ERROR |

## 最佳实践

1. **使用 AppError**：对于业务逻辑错误，使用 `AppError` 枚举
2. **处理 axum 错误**：对于 axum 内置错误，使用提供的处理函数
3. **添加全局中间件**：确保添加全局错误处理中间件作为最后的安全网
4. **包含上下文信息**：在错误中包含足够的上下文信息以便调试
5. **敏感信息处理**：确保不在错误响应中暴露敏感信息

## 示例

查看 `examples/error_handling_demo.rs` 文件以获取完整的使用示例。

## 测试

所有错误处理功能都有相应的测试。运行以下命令来执行测试：

```bash
cargo test api::middleware::error_handler
```