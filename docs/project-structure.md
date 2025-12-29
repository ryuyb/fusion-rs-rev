# Rust Backend API 项目结构

基于 Axum + Diesel + Reqwest + 定时任务的项目目录设计。

## 目录结构

```
src/
├── main.rs                     # 应用入口，启动服务器
├── lib.rs                      # 模块导出
├── config/                     # 配置管理
│   ├── mod.rs
│   └── settings.rs             # 环境变量、配置加载
├── api/                        # HTTP 路由层
│   ├── mod.rs
│   ├── routes.rs               # 路由注册
│   ├── handlers/               # 请求处理器，按业务分组
│   │   ├── mod.rs
│   │   ├── users.rs
│   │   └── orders.rs
│   └── middleware/             # 自定义 Axum 中间件
│       ├── mod.rs
│       ├── auth.rs             # 认证中间件
│       ├── logging.rs          # 请求日志
│       ├── request_id.rs       # 请求追踪 ID
│       └── error_handler.rs    # 全局错误处理
├── services/                   # 业务逻辑层
│   ├── mod.rs
│   ├── user_service.rs
│   └── order_service.rs
├── repositories/               # 数据访问层 (Diesel)
│   ├── mod.rs
│   ├── user_repo.rs
│   └── order_repo.rs
├── models/                     # 数据模型
│   ├── mod.rs
│   ├── domain.rs               # 业务领域模型
│   └── dto.rs                  # 请求/响应 DTO
├── schema.rs                   # Diesel 自动生成的 schema
├── db/                         # 数据库连接管理
│   ├── mod.rs
│   └── pool.rs
├── external/                   # 外部 API 集成 (Reqwest)
│   ├── mod.rs
│   ├── client.rs               # HTTP 客户端封装
│   └── integrations/           # 按第三方服务分组
│       ├── mod.rs
│       ├── payment_api.rs
│       └── weather_api.rs
├── jobs/                       # 定时任务
│   ├── mod.rs
│   ├── scheduler.rs            # 任务调度器
│   └── tasks/
│       ├── mod.rs
│       ├── sync_data.rs
│       └── cleanup.rs
├── error/                      # 统一错误处理
│   ├── mod.rs
│   └── app_error.rs
└── utils/                      # 工具函数
    ├── mod.rs
    └── helpers.rs
```

## 分层说明

| 层级 | 目录 | 职责 |
|------|------|------|
| 路由层 | `api/handlers` | 接收请求、参数校验、调用 service |
| 业务层 | `services` | 业务逻辑、事务编排 |
| 数据层 | `repositories` | 数据库 CRUD 操作 |
| 外部集成 | `external` | 第三方 API 调用封装 |
| 定时任务 | `jobs` | 后台任务调度与执行 |

## 推荐依赖

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
diesel = { version = "2", features = ["postgres", "r2d2"] }
reqwest = { version = "0.12", features = ["json"] }
tokio-cron-scheduler = "0.10"
serde = { version = "1", features = ["derive"] }
dotenvy = "0.15"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## 自定义中间件示例

```rust
// src/api/middleware/auth.rs
use axum::{extract::Request, middleware::Next, response::Response};

pub async fn auth_middleware(request: Request, next: Next) -> Response {
    // 验证逻辑
    next.run(request).await
}
```

## 路由注册示例

```rust
// src/api/routes.rs
use axum::{middleware, Router};

pub fn create_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/users", get(list_users))
        .layer(middleware::from_fn(auth_middleware));

    Router::new()
        .merge(protected)
        .layer(middleware::from_fn(logging_middleware))
        .with_state(state)
}
```
