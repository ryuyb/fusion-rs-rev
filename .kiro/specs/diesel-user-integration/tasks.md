# Implementation Plan: Diesel Async User Integration

## Overview

实现 diesel_async + bb8 + PostgreSQL 的用户数据访问层，包含数据库迁移、模型定义、Repository 实现和错误处理。

## Tasks

- [ ] 1. 配置 Diesel 和数据库迁移
  - [x] 1.1 更新 Cargo.toml 添加 diesel_async 和相关依赖
    - 添加 diesel、diesel-async (bb8 feature)、dotenvy、thiserror
    - _Requirements: 5.1, 5.2_
  - [x] 1.2 初始化 Diesel 配置
    - 运行 `diesel setup` 创建 diesel.toml 和 migrations 目录
    - _Requirements: 1.1_
  - [x] 1.3 创建 users 表迁移文件
    - 编写 up.sql 创建 users 表和 updated_at 触发器
    - 编写 down.sql 删除表和触发器
    - _Requirements: 1.2, 2.1-2.7_
  - [x] 1.4 运行迁移并生成 schema
    - 执行 `diesel migration run`
    - 验证 src/schema.rs 生成正确
    - _Requirements: 1.3, 3.1, 3.2, 3.3_

- [x] 2. 实现错误处理模块
  - [x] 2.1 创建 src/error/mod.rs 和 app_error.rs
    - 定义 AppError 枚举，包含 Database、Pool、Env、NotFound 变体
    - 实现 From trait 用于错误转换
    - _Requirements: 6.7_

- [x] 3. 实现数据库连接池模块
   - [x] 3.1 创建 src/db/mod.rs 和 pool.rs
    - 定义 AsyncDbPool 类型别名
    - 实现 establish_async_connection_pool 异步函数
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 4. 实现数据模型
  - [x] 4.1 创建 src/models/mod.rs 和 domain.rs
    - 定义 User 结构体 (Queryable, Selectable, Serialize)
    - 定义 NewUser 结构体 (Insertable, Deserialize)
    - 定义 UpdateUser 结构体 (AsChangeset, Deserialize)
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [x] 5. 实现 User Repository
  - [x] 5.1 创建 src/repositories/mod.rs 和 user_repo.rs
    - 实现 UserRepository 结构体持有 AsyncDbPool
    - 实现 new() 构造函数
    - _Requirements: 7.2_
  - [x] 5.2 实现 create 方法
    - 异步插入新用户并返回创建的 User
    - _Requirements: 6.1_
  - [x] 5.3 实现 find_by_id 方法
    - 异步根据 id 查询用户，返回 Option<User>
    - _Requirements: 6.2_
  - [x] 5.4 实现 find_by_email 方法
    - 异步根据 email 查询用户，返回 Option<User>
    - _Requirements: 6.3_
  - [x] 5.5 实现 list_all 方法
    - 异步获取所有用户列表
    - _Requirements: 6.4_
  - [x] 5.6 实现 update 方法
    - 异步更新用户并返回更新后的 User
    - _Requirements: 6.5_
  - [x] 5.7 实现 delete 方法
    - 异步删除用户并返回影响行数
    - _Requirements: 6.6_
  - [x] 5.8 实现 Repositories 聚合结构
    - 创建 Repositories 结构体聚合所有 repository
    - 实现 Clone trait 便于 Axum State 共享
    - _Requirements: 7.2_

- [ ] 6. Checkpoint - 验证基础实现
  - 确保代码编译通过
  - 确保数据库迁移成功
  - 如有问题请询问用户

- [ ] 7. 集成到 main.rs
  - [ ] 7.1 更新 main.rs 添加模块声明
    - 添加 db、models、repositories、error、schema 模块
    - _Requirements: 7.1-7.5_
  - [ ] 7.2 添加连接池初始化示例代码
    - 演示如何创建连接池和 Repositories
    - _Requirements: 5.4_

- [ ] 8. Checkpoint - 最终验证
  - 确保所有代码编译通过
  - 确保模块结构符合项目规范
  - 如有问题请询问用户

## Notes

- 任务按依赖顺序排列，需顺序执行
- 数据库迁移需要本地 PostgreSQL 运行
- Property-based tests 可在后续迭代中添加
- Axum handler 集成将在后续 API 开发中完成
