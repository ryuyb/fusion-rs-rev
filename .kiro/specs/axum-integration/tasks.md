# Implementation Plan: Axum Integration

## Overview

本实现计划将 Axum Web 框架集成到 fusion-rs 项目中。按照增量开发方式，从基础设施开始，逐步构建完整的 API 层。

## Tasks

- [x] 1. 添加依赖并创建 API 模块结构
  - 更新 Cargo.toml 添加 uuid 依赖
  - 创建 `src/api/mod.rs` 模块入口
  - 创建 `src/api/handlers/mod.rs`
  - 创建 `src/api/middleware/mod.rs`
  - 创建 `src/api/dto/mod.rs`
  - 更新 `src/main.rs` 引入 api 模块
  - _Requirements: 2.1_

- [x] 2. 实现 AppState 和基础设施
  - [x] 2.1 创建 AppState 结构
    - 创建 `src/state.rs`
    - 实现 AppState 包含 Services
    - 实现 Clone trait
    - 实现 `AppState::new()` 方法
    - _Requirements: 1.1, 1.2, 1.3_

- [x] 3. 实现 DTO 层
  - [x] 3.1 创建请求 DTO
    - 创建 `src/api/dto/request.rs`
    - 实现 CreateUserRequest 结构
    - 实现 UpdateUserRequest 结构
    - 实现 into_new_user() 和 into_update_user() 方法
    - _Requirements: 3.1_

  - [x] 3.2 创建响应 DTO
    - 创建 `src/api/dto/response.rs`
    - 实现 UserResponse 结构（排除 password 字段）
    - 实现 From<User> for UserResponse
    - 实现 ErrorResponse 结构
    - _Requirements: 8.1, 8.2, 8.3, 8.4_

  - [ ]* 3.3 编写 UserResponse 属性测试
    - **Property 2: UserResponse Password Exclusion**
    - **Property 3: Timestamp ISO 8601 Format**
    - **Validates: Requirements 8.1, 8.3**

- [x] 4. 实现中间件层
  - [x] 4.1 实现 Request ID 中间件
    - 创建 `src/api/middleware/request_id.rs`
    - 实现 RequestId 结构
    - 实现 request_id_middleware 函数
    - 处理 X-Request-ID header 传递和生成
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

  - [ ]* 4.2 编写 Request ID 属性测试
    - **Property 1: Request ID Consistency**
    - **Validates: Requirements 5.1, 5.2, 5.3**

  - [x] 4.3 实现 Logging 中间件
    - 创建 `src/api/middleware/logging.rs`
    - 实现 logging_middleware 函数
    - 记录请求方法、路径、request_id
    - 记录响应状态码和耗时
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

  - [x] 4.4 实现错误处理
    - 创建 `src/api/middleware/error_handler.rs`
    - 为 AppError 实现 IntoResponse trait
    - 映射 AppError 到 HTTP 状态码
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_

  - [ ]* 4.5 编写错误处理属性测试
    - **Property 4: Error Status Code Mapping**
    - **Property 5: Error Response Structure**
    - **Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5**

- [ ] 5. Checkpoint - 确保所有测试通过
  - 运行 `cargo test`
  - 确保所有属性测试通过
  - 如有问题请询问用户

- [x] 6. 实现 Handler 层
  - [x] 6.1 实现用户处理器
    - 创建 `src/api/handlers/users.rs`
    - 实现 user_routes() 函数
    - 实现 list_users handler
    - 实现 get_user handler
    - 实现 create_user handler
    - 实现 update_user handler
    - 实现 delete_user handler
    - _Requirements: 2.2, 2.3, 2.4, 2.5, 2.6, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_

- [x] 7. 实现路由注册
  - [x] 7.1 创建路由配置
    - 创建 `src/api/routes.rs`
    - 实现 create_router() 函数
    - 注册 /api/users 路由组
    - 应用中间件层（正确顺序）
    - _Requirements: 2.1, 2.7_

- [x] 8. 更新 main.rs 启动服务器
  - [x] 8.1 集成 Axum 服务器
    - 更新 `src/main.rs`
    - 初始化数据库连接池
    - 创建 AppState
    - 创建 Router
    - 启动 Axum 服务器
    - 实现优雅关闭
    - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [ ] 9. Final Checkpoint - 确保所有测试通过
  - 运行 `cargo test`
  - 运行 `cargo build`
  - 确保编译无错误
  - 如有问题请询问用户

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
