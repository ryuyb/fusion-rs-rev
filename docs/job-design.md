# 定时任务调度系统实施计划

## 概述

在 fusion-rs 项目中实现基于 **apalis** 的定时任务调度系统，支持 Cron 表达式、动态管理、失败重试、历史记录和告警通知。

## 用户需求总结

- ✅ **功能范围**：调度器框架 + 示例任务（数据清理）
- ✅ **失败处理**：日志记录 + 自动重试 + 执行历史 + 告警通知
- ✅ **管理方式**：支持运行时动态管理（API）
- ✅ **技术选型**：apalis 任务队列库
- ✅ **配置需求**：启用开关、Cron 表达式、全局并发数量
- ✅ **任务级并发控制**：每个任务可配置是否允许并行运行或最大并行数量

---

## 技术架构

### 核心技术栈

- **apalis-sql (v0.7)** - 使用 PostgreSQL 后端存储任务
- **apalis-cron (v0.7)** - Cron 表达式支持
- **cron (v0.13)** - Cron 表达式解析和验证
- **diesel + diesel-async** - 复用现有数据库连接池

### 选型理由

1. **PostgreSQL 后端**：复用现有基础设施，无需 Redis
2. **apalis 生态**：成熟的 Rust 任务队列，支持分布式
3. **持久化**：任务配置和历史存储在数据库，服务重启不丢失

---

## 实施步骤

### 第一阶段：基础设施（优先级：P0）

#### 1. 添加依赖

**文件：`Cargo.toml`**

```toml
[dependencies]
# 任务调度核心
apalis = { version = "0.7", features = ["limit", "tracing"] }
apalis-sql = { version = "0.7", features = ["postgres", "tokio-comp"] }
apalis-cron = { version = "0.7" }
cron = "0.13"
```

#### 2. 数据库 Schema

**创建文件：`migrations/{timestamp}_create_jobs_tables/up.sql`**

核心表设计：

```sql
-- 任务定义表
CREATE TABLE jobs (
    id SERIAL PRIMARY KEY,
    job_name VARCHAR(255) NOT NULL UNIQUE,
    job_type VARCHAR(100) NOT NULL,
    cron_expression VARCHAR(255),
    enabled BOOLEAN NOT NULL DEFAULT true,

    -- 并发控制（新增）
    allow_concurrent BOOLEAN NOT NULL DEFAULT false,
    max_concurrent INTEGER, -- NULL 表示无限制

    max_retries INTEGER NOT NULL DEFAULT 3,
    retry_delay_seconds INTEGER NOT NULL DEFAULT 60,
    timeout_seconds INTEGER NOT NULL DEFAULT 300,
    payload JSONB,
    last_run_at TIMESTAMP,
    last_run_status VARCHAR(50),
    next_run_at TIMESTAMP,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(255),

    -- 约束
    CHECK (max_concurrent IS NULL OR max_concurrent > 0)
);

-- 任务执行历史表
CREATE TABLE job_executions (
    id BIGSERIAL PRIMARY KEY,
    job_id INTEGER NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    job_name VARCHAR(255) NOT NULL,
    execution_id UUID NOT NULL DEFAULT gen_random_uuid(),
    worker_id VARCHAR(255),
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    duration_ms INTEGER,
    status VARCHAR(50) NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    error_details JSONB,
    result JSONB
);

-- 索引
CREATE INDEX idx_jobs_enabled ON jobs(enabled) WHERE enabled = true;
CREATE INDEX idx_jobs_next_run ON jobs(next_run_at) WHERE enabled = true;
CREATE INDEX idx_job_executions_job_id ON job_executions(job_id);
CREATE INDEX idx_job_executions_status ON job_executions(status);
```

**创建文件：`migrations/{timestamp}_create_jobs_tables/down.sql`**

```sql
DROP TABLE IF EXISTS job_executions CASCADE;
DROP TABLE IF EXISTS jobs CASCADE;
```

**执行迁移：**
```bash
diesel migration run
```

#### 3. 更新 Schema

**文件：`src/schema.rs`**

运行 `diesel print-schema` 后，将生成的 `jobs` 和 `job_executions` 表定义添加到 schema.rs。

#### 4. 扩展配置系统

**文件：`src/config/settings.rs`**

在文件末尾（第 535 行后）添加：

```rust
// ============================================================================
// Jobs Configuration
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobsConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_worker_concurrency")]
    pub worker_concurrency: usize,

    #[serde(default = "default_poll_interval")]
    pub poll_interval: u64,

    #[serde(default = "default_job_timeout")]
    pub job_timeout: u64,

    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,

    #[serde(default = "default_history_retention_days")]
    pub history_retention_days: u32,
}

fn default_worker_concurrency() -> usize { 4 }
fn default_poll_interval() -> u64 { 5 }
fn default_job_timeout() -> u64 { 300 }
fn default_max_retries() -> u32 { 3 }
fn default_retry_delay() -> u64 { 60 }
fn default_history_retention_days() -> u32 { 30 }

impl Default for JobsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            worker_concurrency: default_worker_concurrency(),
            poll_interval: default_poll_interval(),
            job_timeout: default_job_timeout(),
            max_retries: default_max_retries(),
            retry_delay: default_retry_delay(),
            history_retention_days: default_history_retention_days(),
        }
    }
}

impl JobsConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.enabled && self.worker_concurrency == 0 {
            return Err(ConfigError::ValidationError {
                field: "jobs.worker_concurrency".to_string(),
                message: "Worker concurrency must be at least 1".to_string(),
            });
        }
        Ok(())
    }
}
```

**修改 Settings 结构（第 502-522 行）：**

```rust
pub struct Settings {
    #[serde(default)]
    pub application: ApplicationConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub jwt: JwtConfig,
    #[serde(default)]
    pub logger: LoggerSettings,

    // 新增
    #[serde(default)]
    pub jobs: JobsConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            application: ApplicationConfig::default(),
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            jwt: JwtConfig::default(),
            logger: LoggerSettings::default(),
            jobs: JobsConfig::default(), // 新增
        }
    }
}
```

**修改 Settings::validate() 方法：**

在配置验证链中添加：

```rust
impl Settings {
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.server.validate()?;
        self.database.validate()?;
        self.logger.validate()?;
        self.jobs.validate()?; // 新增
        Ok(())
    }
}
```

#### 5. 更新配置文件

**文件：`config/default.toml`**

在文件末尾添加：

```toml
# -----------------------------------------------------------------------------
# Jobs Scheduler Configuration
# -----------------------------------------------------------------------------
[jobs]
enabled = false
worker_concurrency = 4
poll_interval = 5
job_timeout = 300
max_retries = 3
retry_delay = 60
history_retention_days = 30
```

**文件：`config/development.toml`**

```toml
[jobs]
enabled = true
worker_concurrency = 2
poll_interval = 10
history_retention_days = 7
```

---

### 第二阶段：核心模块（优先级：P0）

#### 6. 创建 Jobs 模块结构

```
src/jobs/
├── mod.rs                 # 模块导出
├── error.rs               # 错误定义
├── types.rs               # JobTask trait 和类型
├── models.rs              # Diesel 数据模型
├── repositories.rs        # 数据访问层
├── scheduler.rs           # 调度器核心
├── tasks/                 # 具体任务实现
│   ├── mod.rs
│   └── data_cleanup.rs    # 示例任务
└── api/                   # 任务管理 API
    ├── mod.rs
    ├── handlers.rs
    └── dto.rs
```

#### 7. 实现核心类型

**文件：`src/jobs/error.rs`**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JobError {
    #[error("任务执行失败: {0}")]
    ExecutionFailed(String),

    #[error("任务超时")]
    Timeout,

    #[error("任务配置无效: {0}")]
    InvalidConfig(String),

    #[error("Cron 表达式无效: {0}")]
    InvalidCronExpression(String),

    #[error("任务未找到: {0}")]
    JobNotFound(String),

    #[error("数据库错误: {0}")]
    DatabaseError(#[from] diesel::result::Error),

    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("内部错误: {0}")]
    InternalError(String),
}
```

**文件：`src/jobs/types.rs`**

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct JobContext {
    pub execution_id: uuid::Uuid,
    pub job_name: String,
    pub retry_count: u32,
    pub db_pool: crate::db::AsyncDbPool,
}

pub type JobResult<T = ()> = Result<T, crate::jobs::error::JobError>;

#[async_trait]
pub trait JobTask: Send + Sync + std::fmt::Debug {
    fn task_type() -> &'static str where Self: Sized;
    async fn execute(&self, ctx: JobContext) -> JobResult<()>;
    fn description(&self) -> Option<String> { None }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending, Running, Success, Failed, Timeout,
}
```

**文件：`src/jobs/models.rs`**

定义 `Job` 和 `JobExecution` 的 Diesel 模型，包括：
- `Job` - Queryable 结构（包含 allow_concurrent、max_concurrent 字段）
- `NewJob` - Insertable 结构
- `UpdateJob` - AsChangeset 结构
- `JobExecution` - 执行历史模型

关键字段：
```rust
pub struct Job {
    // ... 其他字段
    pub allow_concurrent: bool,        // 是否允许并行
    pub max_concurrent: Option<i32>,   // 最大并发数（None 表示无限制）
    // ...
}
```

**文件：`src/jobs/repositories.rs`**

实现 `JobRepository`，提供：
- `create_job()` - 创建任务
- `get_enabled_jobs()` - 查询启用的任务
- `get_job_by_id()` - 按 ID 查询
- `update_job()` - 更新任务
- `delete_job()` - 删除任务

**文件：`src/jobs/mod.rs`**

```rust
pub mod scheduler;
pub mod models;
pub mod repositories;
pub mod types;
pub mod tasks;
pub mod api;
pub mod error;

pub use scheduler::JobScheduler;
pub use types::{JobTask, JobContext, JobResult};
pub use error::JobError;
```

---

### 第三阶段：调度器实现（优先级：P0）

#### 8. 实现调度器核心

**文件：`src/jobs/scheduler.rs`**

```rust
use crate::config::JobsConfig;
use crate::db::AsyncDbPool;
use crate::jobs::error::JobError;
use crate::jobs::repositories::JobRepository;
use tokio::task::JoinHandle;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct JobScheduler {
    config: JobsConfig,
    db_pool: AsyncDbPool,
    repository: JobRepository,
    worker_handle: Option<JoinHandle<()>>,

    // 任务并发控制（新增）
    running_counts: Arc<RwLock<HashMap<String, usize>>>,
}

impl JobScheduler {
    pub async fn new(db_pool: AsyncDbPool, config: JobsConfig) -> Result<Self, JobError> {
        config.validate()
            .map_err(|e| JobError::InvalidConfig(e.to_string()))?;

        let repository = JobRepository::new(db_pool.clone());

        Ok(Self {
            config,
            db_pool,
            repository,
            worker_handle: None,
            running_counts: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn start(&mut self) -> Result<(), JobError> {
        if !self.config.enabled {
            tracing::info!("任务调度器已禁用");
            return Ok(());
        }

        tracing::info!(
            worker_concurrency = self.config.worker_concurrency,
            "启动任务调度器"
        );

        // 使用 apalis-sql 初始化 PostgreSQL 存储
        // 启动工作线程
        // 注册任务处理器
        // 实现任务级并发控制逻辑

        Ok(())
    }

    // 检查任务是否可以执行（并发控制）
    async fn can_execute_job(&self, job_name: &str, job: &crate::jobs::models::Job) -> bool {
        if !job.allow_concurrent {
            let counts = self.running_counts.read().await;
            if let Some(count) = counts.get(job_name) {
                if *count > 0 {
                    tracing::debug!(
                        job_name = %job_name,
                        "任务不允许并行，跳过执行"
                    );
                    return false;
                }
            }
        } else if let Some(max_concurrent) = job.max_concurrent {
            let counts = self.running_counts.read().await;
            if let Some(count) = counts.get(job_name) {
                if *count >= max_concurrent as usize {
                    tracing::debug!(
                        job_name = %job_name,
                        current_count = *count,
                        max_concurrent,
                        "任务并发数已达上限，跳过执行"
                    );
                    return false;
                }
            }
        }

        true
    }

    // 增加运行计数
    async fn increment_running_count(&self, job_name: &str) {
        let mut counts = self.running_counts.write().await;
        *counts.entry(job_name.to_string()).or_insert(0) += 1;
    }

    // 减少运行计数
    async fn decrement_running_count(&self, job_name: &str) {
        let mut counts = self.running_counts.write().await;
        if let Some(count) = counts.get_mut(job_name) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                counts.remove(job_name);
            }
        }
    }

    pub async fn stop(&mut self) -> Result<(), JobError> {
        if let Some(handle) = self.worker_handle.take() {
            tracing::info!("停止任务调度器");
            handle.abort();
        }
        Ok(())
    }

    pub fn repository(&self) -> &JobRepository {
        &self.repository
    }
}
```

#### 9. 集成到 Server

**文件：`src/server.rs`**

在 `Server::run()` 方法中（第 92-97 行后）添加：

```rust
// 创建应用状态
let state = AppState::new(pool.clone(), self.settings.jwt.clone());
tracing::info!("Application state created");

// 初始化并启动任务调度器（如果启用）
let mut scheduler = None;
if self.settings.jobs.enabled {
    tracing::info!("初始化任务调度器");

    self.settings.jobs.validate().map_err(|e| {
        tracing::error!(error = %e, "Jobs 配置验证失败");
        anyhow::anyhow!("Jobs configuration validation failed: {}", e)
    })?;

    let mut job_scheduler = crate::jobs::scheduler::JobScheduler::new(
        pool.clone(),
        self.settings.jobs.clone(),
    ).await?;

    job_scheduler.start().await?;
    scheduler = Some(job_scheduler);
}

// 创建路由
let router = create_router(state);
```

在 `shutdown_signal()` 调用后添加清理逻辑：

```rust
axum::serve(listener, router)
    .with_graceful_shutdown(shutdown_signal())
    .await?;

// 停止调度器
if let Some(mut sched) = scheduler {
    sched.stop().await?;
}

tracing::info!("Server shutdown complete");
```

---

### 第四阶段：示例任务（优先级：P1）

#### 10. 实现数据清理任务

**文件：`src/jobs/tasks/data_cleanup.rs`**

```rust
use crate::jobs::error::JobError;
use crate::jobs::types::{JobContext, JobResult, JobTask};
use crate::schema::job_executions;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCleanupTask {
    pub retention_days: i64,
}

#[async_trait]
impl JobTask for DataCleanupTask {
    fn task_type() -> &'static str {
        "data_cleanup"
    }

    async fn execute(&self, ctx: JobContext) -> JobResult<()> {
        tracing::info!(
            execution_id = %ctx.execution_id,
            job_name = %ctx.job_name,
            retention_days = self.retention_days,
            "开始执行数据清理任务"
        );

        let cutoff_date = Utc::now().naive_utc() - Duration::days(self.retention_days);

        let mut conn = ctx.pool.get().await.map_err(|e| {
            JobError::DatabaseError(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            ))
        })?;

        let deleted_count = diesel::delete(
            job_executions::table.filter(
                job_executions::started_at
                    .lt(cutoff_date)
                    .and(job_executions::status.ne("running"))
            )
        )
        .execute(&mut conn)
        .await
        .map_err(JobError::DatabaseError)?;

        tracing::info!(
            execution_id = %ctx.execution_id,
            deleted_count,
            "数据清理任务完成"
        );

        Ok(())
    }

    fn description(&self) -> Option<String> {
        Some(format!("清理 {} 天前的任务执行历史", self.retention_days))
    }
}

impl Default for DataCleanupTask {
    fn default() -> Self {
        Self { retention_days: 30 }
    }
}
```

**文件：`src/jobs/tasks/mod.rs`**

```rust
pub mod data_cleanup;
pub use data_cleanup::DataCleanupTask;
```

---

### 第五阶段：动态管理 API（优先级：P1）

#### 11. 实现 API DTO

**文件：`src/jobs/api/dto.rs`**

定义：
- `CreateJobRequest` - 创建任务请求
  ```rust
  pub struct CreateJobRequest {
      pub job_name: String,
      pub job_type: String,
      pub cron_expression: Option<String>,
      pub enabled: bool,

      // 并发控制（新增）
      pub allow_concurrent: bool,
      pub max_concurrent: Option<i32>,

      pub max_retries: i32,
      pub payload: Option<JsonValue>,
      pub description: Option<String>,
  }
  ```
- `UpdateJobRequest` - 更新任务请求
- `JobResponse` - 任务详情响应（包含并发控制字段）
- `JobExecutionResponse` - 执行历史响应

#### 12. 实现 API Handlers

**文件：`src/jobs/api/handlers.rs`**

实现端点：
- `POST /api/jobs` - 创建任务
- `GET /api/jobs` - 查询任务列表
- `GET /api/jobs/{id}` - 查询任务详情
- `PUT /api/jobs/{id}` - 更新任务
- `DELETE /api/jobs/{id}` - 删除任务
- `POST /api/jobs/{id}/pause` - 暂停任务
- `POST /api/jobs/{id}/resume` - 恢复任务
- `GET /api/jobs/{id}/executions` - 查询执行历史

**文件：`src/jobs/api/mod.rs`**

```rust
pub mod handlers;
pub mod dto;
```

#### 13. 集成到路由

**文件：`src/api/routes.rs`**

在受保护路由中（第 51 行后）添加：

```rust
let protected_routes = OpenApiRouter::new()
    .nest("/me", handlers::me::me_routes())
    .nest("/users", handlers::users::user_routes())
    .nest("/jobs", crate::jobs::api::handlers::job_routes()) // 新增
    .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));
```

---

## 关键文件清单

### 需要修改的现有文件

1. `Cargo.toml` - 添加 apalis 相关依赖
2. `src/config/settings.rs` - 扩展 JobsConfig（第 535 行后）
3. `src/server.rs` - 集成调度器启动和停止（第 92-120 行）
4. `src/api/routes.rs` - 添加 jobs API 路由（第 51 行后）
5. `config/default.toml` - 添加 jobs 配置段（文件末尾）
6. `config/development.toml` - 开发环境配置覆盖（文件末尾）

### 需要创建的新文件

7. `migrations/{timestamp}_create_jobs_tables/up.sql` - 数据库 schema
8. `migrations/{timestamp}_create_jobs_tables/down.sql` - 回滚脚本
9. `src/jobs/mod.rs` - 模块导出
10. `src/jobs/error.rs` - 错误定义
11. `src/jobs/types.rs` - JobTask trait
12. `src/jobs/models.rs` - Diesel 模型
13. `src/jobs/repositories.rs` - 数据访问层
14. `src/jobs/scheduler.rs` - 调度器核心
15. `src/jobs/tasks/mod.rs` - 任务模块
16. `src/jobs/tasks/data_cleanup.rs` - 示例任务
17. `src/jobs/api/mod.rs` - API 模块
18. `src/jobs/api/dto.rs` - API DTO
19. `src/jobs/api/handlers.rs` - API handlers

### 需要更新的文件（自动生成）

20. `src/schema.rs` - 运行 `diesel print-schema` 后更新

---

## 设计原则体现

### SOLID 原则

- **单一职责（SRP）**：
  - `JobRepository` 仅负责数据访问
  - `JobScheduler` 仅负责调度逻辑
  - 每个任务实现独立的 `JobTask` trait

- **开闭原则（OCP）**：
  - 通过 `JobTask` trait 扩展新任务，无需修改调度器
  - 配置系统通过 `serde` 自动反序列化，易于扩展

- **里氏替换原则（LSP）**：
  - 所有实现 `JobTask` 的任务可互换使用

- **接口隔离原则（ISP）**：
  - `JobTask` trait 仅定义必需方法
  - Repository 接口职责明确

- **依赖反转原则（DIP）**：
  - 调度器依赖 `JobTask` trait 抽象，而非具体实现
  - 通过 `AsyncDbPool` 抽象数据库访问

### KISS & DRY

- **KISS**：使用成熟的 apalis 库，避免重复造轮子
- **DRY**：复用现有的数据库连接池、配置系统、日志框架

---

## 验证和测试

### 功能验证清单

- [ ] 配置加载和验证正常
- [ ] 数据库 migrations 成功
- [ ] 调度器启动和停止正常
- [ ] 示例任务执行成功
- [ ] 任务失败自动重试
- [ ] 执行历史正确记录
- [ ] API 端点正常工作
- [ ] 动态创建/删除/暂停任务
- [ ] 优雅关闭不丢失任务
- [ ] **任务并发控制正常工作**（新增）
  - [ ] 不允许并行的任务只运行一个实例
  - [ ] 有最大并发限制的任务不超过配置值
  - [ ] 无限制的任务可以同时运行多个实例

### 测试策略

1. **单元测试**：Repository、Task 实现
2. **集成测试**：调度器 + 数据库
3. **端到端测试**：完整工作流

---

## 风险和注意事项

### 潜在风险

1. **apalis 版本兼容性**：确保 apalis-sql 0.7 与 diesel-async 兼容
2. **数据库连接池耗尽**：任务执行时注意连接池大小
3. **长时间运行任务**：需要合理设置 timeout
4. **分布式场景**：PostgreSQL SKIP LOCKED 机制需测试

### 缓解措施

- 严格版本锁定，避免破坏性升级
- 监控数据库连接池使用率
- 配置合理的超时和重试策略
- 多实例部署前进行充分测试

---

## 实施时间估算

| 阶段 | 任务 | 预计时间 |
|-----|------|---------|
| 1 | 基础设施（依赖、schema、配置） | 2 小时 |
| 2 | 核心模块（types、models、repository） | 3 小时 |
| 3 | 调度器实现（scheduler.rs + 集成） | 4 小时 |
| 4 | 示例任务（data_cleanup） | 1 小时 |
| 5 | 动态管理 API | 3 小时 |
| 6 | 测试和文档 | 2 小时 |
| **总计** | | **15 小时** |

---

## 并发控制详细说明

### 三种并发模式

#### 模式 1：完全禁止并行
```sql
allow_concurrent = false
max_concurrent = NULL
```
- 同一任务只能有一个实例在运行
- 如果已有实例运行，新的调度会被跳过
- 适用于：数据库备份、数据迁移等不能同时执行的任务

#### 模式 2：限制最大并发数
```sql
allow_concurrent = true
max_concurrent = 3  -- 最多同时运行 3 个实例
```
- 允许多个实例并行，但不超过 max_concurrent
- 超过限制时，新的调度会被跳过
- 适用于：API 调用、数据处理等有资源限制的任务

#### 模式 3：无限制并行
```sql
allow_concurrent = true
max_concurrent = NULL
```
- 任意数量的实例可以同时运行
- 没有并发限制
- 适用于：日志清理、缓存预热等轻量级任务

### 实现机制

调度器维护一个内存中的并发计数器 `running_counts: HashMap<job_name, count>`：

1. **执行前检查**：
   ```rust
   if !allow_concurrent && running_count > 0 {
       skip_execution();
   }
   if max_concurrent.is_some() && running_count >= max_concurrent {
       skip_execution();
   }
   ```

2. **执行中计数**：
   - 任务开始时：`increment_running_count(job_name)`
   - 任务结束时（成功或失败）：`decrement_running_count(job_name)`

3. **分布式场景**：
   - 单实例部署：内存计数器即可
   - 多实例部署：需使用 PostgreSQL advisory locks 或 Redis 分布式锁

### 配置示例

```toml
# 示例 1：每日数据库备份（不允许并行）
[[jobs.jobs]]
job_name = "daily_backup"
cron = "0 2 * * *"
allow_concurrent = false
max_concurrent = null

# 示例 2：定期 API 同步（最多 3 个并行）
[[jobs.jobs]]
job_name = "api_sync"
cron = "*/10 * * * *"
allow_concurrent = true
max_concurrent = 3

# 示例 3：缓存清理（无限制）
[[jobs.jobs]]
job_name = "cache_cleanup"
cron = "0 * * * *"
allow_concurrent = true
max_concurrent = null
```

---

## 下一步行动

1. ✅ 用户确认计划（包含并发控制需求）
2. 执行第一阶段（基础设施）
3. 逐步实施各阶段
4. 持续测试和验证（包括并发控制测试）
5. 完成文档更新
