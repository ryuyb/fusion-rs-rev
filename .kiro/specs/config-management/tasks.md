# Implementation Plan: Config Management

## Overview

基于 `config` crate 实现分层配置管理模块，支持 TOML 配置文件、环境变量覆盖和多环境配置。实现将按照模块结构逐步构建，确保每个组件都经过验证。

## Tasks

- [x] 1. 创建配置模块基础结构
  - [x] 1.1 创建 `src/config/mod.rs` 模块入口文件
    - 导出所有公共类型和函数
    - _Requirements: 模块结构_
  - [x] 1.2 创建 `src/config/error.rs` 错误类型
    - 定义 `ConfigError` 枚举，包含 FileNotFound、ParseError、ValidationError、EnvVarError、MutualExclusivityError
    - 实现 `thiserror::Error` trait
    - _Requirements: 错误处理_
  - [x] 1.3 创建 `src/config/environment.rs` 环境枚举
    - 定义 `Environment` 枚举（Development, Test, Staging, Production）
    - 实现 `FromStr`、`Default`、`as_str()` 方法
    - 实现 `from_env()` 从 `FUSION_APP_ENV` 读取
    - _Requirements: 2.1, 2.2, 2.3_

- [x] 2. 实现配置结构体
  - [x] 2.1 创建 `src/config/settings.rs` 配置结构
    - 定义 `ApplicationConfig` 结构体（name, version）及默认值
    - 定义 `ServerConfig` 结构体（host, port, request_timeout, keep_alive_timeout）及默认值
    - 定义 `DatabaseConfig` 结构体（url, max_connections, min_connections, connection_timeout）及默认值
    - 定义 `LoggerSettings` 及子结构（ConsoleSettings, FileSettings, RotationSettings）
    - 定义 `Settings` 主结构体
    - 实现 `Serialize` 和 `Deserialize` traits
    - _Requirements: 5.1-5.3, 6.1-6.6, 7.1-7.2, 7.4-7.6, 8.1-8.2, 8.4, 10.1_
  - [ ]* 2.2 编写 Settings 结构体单元测试
    - 测试默认值正确性
    - 测试序列化/反序列化
    - _Requirements: 5.2, 5.3, 6.3-6.6, 7.4-7.6_

- [x] 3. 实现配置验证
  - [x] 3.1 创建 `src/config/validation.rs` 验证逻辑
    - 实现 `ServerConfig::validate()` - 端口范围检查
    - 实现 `DatabaseConfig::validate()` - URL 非空、连接数范围检查
    - 实现 `LoggerSettings::validate()` - 日志级别、格式验证
    - 实现 `Settings::validate()` - 调用所有子验证
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_
  - [ ]* 3.2 编写属性测试：配置验证
    - **Property 6: Configuration Validation**
    - 生成无效端口值，验证拒绝
    - 生成无效 URL，验证拒绝
    - 生成 min > max 连接数，验证拒绝
    - **Validates: Requirements 9.1, 9.2, 9.4, 9.5**

- [x] 4. 实现 LoggerSettings 到 LoggerConfig 转换
  - [x] 4.1 在 `settings.rs` 中实现 `LoggerSettings::into_logger_config()`
    - 转换 ConsoleSettings → ConsoleConfig
    - 转换 FileSettings → FileConfig
    - 转换 RotationSettings → RotationConfig
    - 处理转换错误
    - _Requirements: 8.3_
  - [ ]* 4.2 编写属性测试：Logger 配置转换
    - **Property 5: Logger Settings Conversion**
    - 生成有效 LoggerSettings，验证转换后 LoggerConfig 有效
    - **Validates: Requirements 8.3**

- [x] 5. 实现配置加载器
  - [x] 5.1 创建 `src/config/loader.rs` 配置加载器
    - 实现 `ConfigLoader::new()` - 读取环境变量确定配置目录/文件
    - 检查 `FUSION_CONFIG_DIR` 和 `FUSION_CONFIG_FILE` 互斥性
    - 实现 `ConfigLoader::load()` - 构建 config::Config 并反序列化
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  - [x] 5.2 实现分层配置加载
    - 添加 default.toml（必需）
    - 添加 {environment}.toml（可选）
    - 添加 local.toml（可选）
    - 添加 FUSION_ 前缀环境变量
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 4.1, 4.2, 4.3_
  - [ ]* 5.3 编写属性测试：配置优先级
    - **Property 1: Configuration Precedence**
    - 在多个源中设置相同键，验证最高优先级生效
    - **Validates: Requirements 1.1, 1.2**
  - [ ]* 5.4 编写属性测试：可选文件处理
    - **Property 2: Optional File Graceful Handling**
    - 测试缺失可选文件时加载成功
    - **Validates: Requirements 1.4**
  - [ ]* 5.5 编写属性测试：环境文件加载
    - **Property 3: Environment-Based File Loading**
    - 测试不同环境加载对应文件
    - **Validates: Requirements 2.1, 2.4**
  - [ ]* 5.6 编写属性测试：环境变量映射
    - **Property 4: Environment Variable Mapping**
    - 测试 FUSION_ 前缀和双下划线分隔符映射
    - **Validates: Requirements 4.1, 4.2, 4.3**

- [x] 6. Checkpoint - 核心功能验证
  - 确保所有测试通过，如有问题请询问用户

- [x] 7. 实现往返序列化测试
  - [x]* 7.1 编写属性测试：Settings 往返序列化
    - **Property 7: Settings Round-Trip Serialization**
    - 生成有效 Settings，序列化为 TOML 后反序列化，验证等价
    - **Validates: Requirements 10.4**

- [x] 8. 创建默认配置文件
  - [x] 8.1 创建 `config/default.toml` 默认配置文件
    - 包含所有配置节的默认值
    - 添加注释说明各配置项
    - _Requirements: 1.1_
  - [x] 8.2 创建 `config/development.toml` 开发环境配置
    - 开发环境特定配置
    - _Requirements: 2.3, 2.4_
  - [x] 8.3 创建 `config/production.toml` 生产环境配置示例
    - 生产环境推荐配置
    - _Requirements: 2.3, 2.4_
  - [x] 8.4 更新 `.gitignore` 忽略 `config/local.toml`
    - 防止本地配置提交到版本控制
    - _Requirements: 1.1_

- [ ] 9. 集成到主程序
  - [x] 9.1 更新 `src/main.rs` 使用配置模块
    - 加载配置
    - 使用 LoggerSettings 初始化日志
    - 打印加载的配置信息
    - _Requirements: 集成_
  - [ ] 9.2 更新 `src/lib.rs` 导出配置模块（如存在）
    - _Requirements: 模块导出_

- [ ] 10. Final Checkpoint - 完整功能验证
  - 确保所有测试通过
  - 验证配置加载流程正常工作
  - 如有问题请询问用户

## Notes

- 标记 `*` 的任务为可选测试任务，可跳过以加快 MVP 开发
- 每个任务引用具体需求以确保可追溯性
- 属性测试验证通用正确性属性
- Checkpoint 任务确保增量验证
