# 消息推送系统实施计划

## 概述

为 fusion-rs 项目设计并实现一个可扩展的消息推送系统，支持 Email 和 Webhook 两种推送方式，具备良好的抽象设计以便将来扩展更多推送方式。

## 核心设计原则

- **遵循现有架构**: 严格遵循项目的分层架构（Model → Repository → Service → API）
- **SOLID 原则**: 通过 Trait 抽象实现开闭原则（OCP），易于扩展新的推送方式
- **DRY 原则**: 通过 Factory 模式避免重复的 Provider 创建逻辑
- **KISS 原则**: 使用 JSONB 存储配置，避免为每种渠道类型创建单独的表
- **YAGNI 原则**: 只实现当前明确需要的功能，不预设未来需求

## 架构设计

### 数据库设计

#### 1. notification_channels 表（推送渠道配置）
```sql
- id: SERIAL PRIMARY KEY
- user_id: INTEGER (外键关联 users)
- channel_type: VARCHAR(50) ('email', 'webhook')
- name: VARCHAR(100) (用户自定义名称，方便识别)
- config: JSONB (渠道特定配置，灵活存储)
- enabled: BOOLEAN (启用/禁用)
- priority: INTEGER (优先级，数字越小优先级越高)
- created_at, updated_at: TIMESTAMP
```

**索引**: user_id, enabled, priority

#### 2. notification_logs 表（推送历史记录）
```sql
- id: SERIAL PRIMARY KEY
- channel_id: INTEGER (外键关联 notification_channels)
- message: TEXT (推送的消息内容)
- status: VARCHAR(20) ('pending', 'success', 'failed', 'retrying')
- error_message: TEXT (失败时的错误信息)
- retry_count: INTEGER (重试次数)
- sent_at: TIMESTAMP (实际发送时间)
- created_at, updated_at: TIMESTAMP
```

**索引**: channel_id, status, created_at DESC

### 推送抽象设计

#### NotificationProvider Trait（核心抽象）
```rust
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    async fn send(&self, message: &NotificationMessage) -> AppResult<NotificationResult>;
    fn validate_config(config: &JsonValue) -> AppResult<()> where Self: Sized;
    fn provider_name(&self) -> &'static str;
}
```

#### 具体实现
- **EmailProvider**: 使用 lettre 库发送邮件
- **WebhookProvider**: 使用现有的 HTTP_CLIENT 发送 HTTP 请求
- **ProviderFactory**: 根据渠道类型创建对应的 Provider

### 模块组织结构

```
src/
├── models/
│   ├── notification_channel.rs (新增)
│   └── notification_log.rs (新增)
├── repositories/
│   ├── notification_channel_repo.rs (新增)
│   └── notification_log_repo.rs (新增)
├── services/
│   └── notification_service.rs (新增)
├── notifications/ (新增目录)
│   ├── mod.rs
│   ├── provider.rs (核心 Trait)
│   ├── factory.rs (Provider Factory)
│   └── providers/
│       ├── email.rs
│       └── webhook.rs
├── api/
│   ├── dto/
│   │   └── notification.rs (新增)
│   └── handlers/
│       └── notifications.rs (新增)
└── config/
    └── settings.rs (添加 EmailConfig)
```

## 实施步骤

### 第 1 步: 数据库迁移

**文件**:
- `migrations/2025-12-30-XXXXXX_create_notification_channels/up.sql`
- `migrations/2025-12-30-XXXXXX_create_notification_channels/down.sql`
- `migrations/2025-12-30-XXXXXX_create_notification_logs/up.sql`
- `migrations/2025-12-30-XXXXXX_create_notification_logs/down.sql`

**操作**:
1. 创建迁移文件
2. 编写 SQL 创建表和索引
3. 添加 `diesel_manage_updated_at` 触发器
4. 编写 down.sql 回滚逻辑
5. 运行 `diesel migration run` 生成 schema.rs

### 第 2 步: 数据模型层（Model）

**文件**:
- `src/models/notification_channel.rs` (新增)
- `src/models/notification_log.rs` (新增)
- `src/models/mod.rs` (更新)

**实现**:
- 遵循三层模型设计：Query Model、Insert Model、Update Model
- 为 NotificationChannel 添加 Queryable、Insertable、AsChangeset 等 Diesel derive
- 为 NotificationLog 添加相应的 derive
- 定义 ChannelType 和 NotificationStatus 枚举

### 第 3 步: 推送抽象层（Notification Providers）

**文件**:
- `src/notifications/mod.rs` (新增)
- `src/notifications/provider.rs` (新增)
- `src/notifications/providers/mod.rs` (新增)
- `src/notifications/providers/email.rs` (新增)
- `src/notifications/providers/webhook.rs` (新增)
- `src/notifications/factory.rs` (新增)

**实现**:
1. 定义 `NotificationProvider` trait（核心抽象）
2. 定义 `NotificationMessage` 和 `NotificationResult` 结构
3. 实现 `EmailProvider`（使用 lettre）
4. 实现 `WebhookProvider`（使用现有 HTTP_CLIENT）
5. 实现 `ProviderFactory`（工厂模式创建 Provider）

**关键点**:
- EmailProvider 需要注入 SMTP mailer
- WebhookProvider 复用项目现有的 `HTTP_CLIENT`
- 每个 Provider 实现配置验证逻辑

### 第 4 步: 仓储层（Repository）

**文件**:
- `src/repositories/notification_channel_repo.rs` (新增)
- `src/repositories/notification_log_repo.rs` (新增)
- `src/repositories/mod.rs` (更新)

**实现**:
1. NotificationChannelRepository: create, find_by_id, list_by_user, list_enabled_by_user, update, delete
2. NotificationLogRepository: create, update, list_by_channel
3. 在 Repositories 聚合中添加这两个 Repository

**遵循模式**: 与 UserRepository 保持一致的异步模式

### 第 5 步: 业务逻辑层（Service）

**文件**:
- `src/services/notification_service.rs` (新增)
- `src/services/mod.rs` (更新)

**实现**:
- NotificationService 提供以下方法：
  - 渠道管理: create_channel, list_user_channels, get_channel, update_channel, delete_channel
  - 推送发送: send_to_user, send_to_channel
  - 日志查询: get_channel_logs
- 在 create_channel 和 update_channel 时验证配置
- send_to_user 向用户所有启用的渠道发送（按优先级）
- 在 Services 聚合中添加 NotificationService

**关键点**:
- 注入 ProviderFactory 用于创建 Provider
- 推送时先创建 pending 状态的日志，再发送，最后更新状态

### 第 6 步: 配置管理

**文件**:
- `src/config/settings.rs` (更新)
- `config/default.toml` (更新)

**实现**:
1. 在 settings.rs 添加 EmailConfig 结构
2. 在 Settings 结构中添加 `email: EmailConfig` 字段
3. 在 default.toml 添加 email 配置段

**EmailConfig 字段**:
- enabled: bool
- smtp_host: String
- smtp_port: u16
- smtp_username: String
- smtp_password: String
- from: String (默认发件人)
- use_tls: bool

### 第 7 步: 应用状态更新（AppState）

**文件**:
- `src/state.rs` (更新)
- `src/server.rs` (可能需要更新初始化逻辑)

**实现**:
1. 在 AppState::new 中初始化 SMTP mailer（如果启用）
2. 创建 ProviderFactory 并传入 mailer 和 HTTP_CLIENT
3. 将 ProviderFactory 传递给 NotificationService

**关键点**:
- ProviderFactory 使用 Arc 包装以支持多线程共享
- 只在 email.enabled=true 时创建 mailer

### 第 8 步: API 层（Handler 和 DTO）

**文件**:
- `src/api/dto/notification.rs` (新增)
- `src/api/handlers/notifications.rs` (新增)
- `src/api/routes.rs` (更新)

**实现**:
1. 定义 DTO: CreateChannelRequest, UpdateChannelRequest, ChannelResponse, SendNotificationRequest
2. 实现 Handler: create_channel, list_channels, get_channel, update_channel, delete_channel, send_notification
3. 在 routes.rs 注册通知相关路由
4. 使用 utoipa 添加 OpenAPI 文档注解

**路由设计**:
- POST /api/notifications/channels - 创建推送渠道
- GET /api/notifications/channels - 列出用户的所有渠道
- GET /api/notifications/channels/:id - 获取渠道详情
- PUT /api/notifications/channels/:id - 更新渠道
- DELETE /api/notifications/channels/:id - 删除渠道
- POST /api/notifications/send - 发送通知
- GET /api/notifications/channels/:id/logs - 获取推送历史

### 第 9 步: 依赖管理

**文件**: `Cargo.toml` (更新)

**新增依赖**:
```toml
lettre = { version = "0.11", features = ["tokio1-rustls-tls", "smtp-transport", "builder"] }
async-trait = "0.1"
```

**说明**:
- lettre: Rust 生态最成熟的邮件库，支持异步
- async-trait: 用于 NotificationProvider trait 的异步方法
- reqwest 和 serde_json 已存在，无需添加

### 第 10 步: 错误处理

**文件**: `src/error/app_error.rs` (更新)

**实现**:
在 AppError 枚举中添加:
```rust
#[error("Notification error: {message}")]
Notification { message: String },
```

## 关键文件清单

### 必须创建的文件（按顺序）

1. **迁移文件**
   - `migrations/2025-12-30-XXXXXX_create_notification_channels/up.sql`
   - `migrations/2025-12-30-XXXXXX_create_notification_channels/down.sql`
   - `migrations/2025-12-30-XXXXXX_create_notification_logs/up.sql`
   - `migrations/2025-12-30-XXXXXX_create_notification_logs/down.sql`

2. **模型层**
   - `src/models/notification_channel.rs`
   - `src/models/notification_log.rs`

3. **推送抽象层**
   - `src/notifications/mod.rs`
   - `src/notifications/provider.rs`
   - `src/notifications/factory.rs`
   - `src/notifications/providers/mod.rs`
   - `src/notifications/providers/email.rs`
   - `src/notifications/providers/webhook.rs`

4. **仓储层**
   - `src/repositories/notification_channel_repo.rs`
   - `src/repositories/notification_log_repo.rs`

5. **服务层**
   - `src/services/notification_service.rs`

6. **API 层**
   - `src/api/dto/notification.rs`
   - `src/api/handlers/notifications.rs`

### 必须修改的文件

1. `src/models/mod.rs` - 添加新模型导出
2. `src/repositories/mod.rs` - 更新 Repositories 聚合
3. `src/services/mod.rs` - 更新 Services 聚合
4. `src/config/settings.rs` - 添加 EmailConfig
5. `src/state.rs` - 注入 ProviderFactory
6. `src/api/routes.rs` - 注册通知路由
7. `src/error/app_error.rs` - 添加通知错误类型
8. `Cargo.toml` - 添加 lettre 和 async-trait 依赖
9. `config/default.toml` - 添加 email 配置段

## 配置示例

### config/default.toml 新增内容

```toml
# Email Configuration
[email]
enabled = false
smtp_host = ""
smtp_port = 587
smtp_username = ""
smtp_password = ""
from = "noreply@example.com"
use_tls = true
```

### 环境变量覆盖

```bash
FUSION_EMAIL__ENABLED=true
FUSION_EMAIL__SMTP_HOST=smtp.gmail.com
FUSION_EMAIL__SMTP_USERNAME=your-email@gmail.com
FUSION_EMAIL__SMTP_PASSWORD=your-app-password
FUSION_EMAIL__FROM=noreply@example.com
```

## 使用示例

### 创建 Email 推送渠道

```bash
POST /api/notifications/channels
{
  "channel_type": "email",
  "name": "我的邮箱",
  "config": {
    "to": "user@example.com"
  },
  "enabled": true,
  "priority": 0
}
```

### 创建 Webhook 推送渠道

```bash
POST /api/notifications/channels
{
  "channel_type": "webhook",
  "name": "钉钉机器人",
  "config": {
    "url": "https://oapi.dingtalk.com/robot/send?access_token=xxx",
    "method": "POST",
    "headers": {
      "Content-Type": "application/json"
    }
  },
  "enabled": true,
  "priority": 1
}
```

### 发送通知

```bash
POST /api/notifications/send
{
  "user_id": 1,
  "subject": "系统通知",
  "body": "您的订单已发货",
  "metadata": {
    "order_id": "12345"
  }
}
```

## 扩展性说明

### 新增推送方式（例如 SMS）

1. 创建 `src/notifications/providers/sms.rs`
2. 实现 `NotificationProvider` trait
3. 在 `ProviderFactory::create_provider` 中添加 "sms" 分支
4. 更新配置（如需要）
5. **无需修改**: Model、Repository、Service 层

### 新增推送配置字段

由于使用 JSONB 存储配置，可以直接在创建渠道时传入新字段，无需修改数据库表结构。

## 测试策略

1. **单元测试**: 每个 Provider 的 send 和 validate_config 方法
2. **集成测试**: NotificationService 的完整推送流程
3. **数据库测试**: Repository 层的 CRUD 操作
4. **API 测试**: Handler 层的请求响应

## 注意事项

1. **安全性**: SMTP 密码应通过环境变量配置，不要提交到代码仓库
2. **错误处理**: 单个渠道推送失败不影响其他渠道
3. **幂等性**: 推送日志记录在发送前创建，确保可追溯
4. **性能**: 多个渠道推送可以考虑并发执行（当前为串行）
5. **重试机制**: 当前不实现自动重试（YAGNI），可在后续迭代中添加

## 文档更新

完成实施后需要更新以下文档：

1. **README.md** - 添加通知系统功能说明
2. **API 文档** - 通过 utoipa 自动生成 OpenAPI 文档
3. **配置文档** - 说明 email 配置项的作用

## 预期成果

1. ✅ 支持 Email 和 Webhook 两种推送方式
2. ✅ 用户可以配置多个推送渠道（包括相同类型的多个配置）
3. ✅ 支持启用/禁用和优先级设置
4. ✅ 完整的推送历史记录
5. ✅ 良好的扩展性，易于添加新的推送方式
6. ✅ 遵循项目现有的架构模式和代码风格
7. ✅ 完整的 OpenAPI 文档
