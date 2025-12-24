# Implementation Plan: Advanced Logger (Simplified)

## Overview

本实现计划基于 `tracing-subscriber` 现有的 Layer 系统，最大化复用内置功能，仅实现必要的自定义组件（轮转 Writer 和压缩）。

## Tasks

- [x] 1. 设置项目结构和核心依赖
  - 更新 Cargo.toml 添加必要的依赖项
  - 创建模块结构和基础文件
  - _Requirements: 9.1_

- [x] 2. 实现配置系统
  - [x] 2.1 定义配置结构体和枚举（已完成）
  - [x] 2.2 编写配置系统的属性测试
    - **Property 16: 配置管理**
    - **Property 17: 配置验证**

- [x] 3. 实现轮转文件 Writer
  - [x] 3.1 创建 RotatingFileWriter
    - 实现 `MakeWriter` trait
    - 支持基于大小的轮转
    - 支持基于时间的轮转
    - _Requirements: 6.1, 6.2, 6.5, 7.1, 7.2, 7.3, 7.4_

  - [x] 3.2 实现 RotationManager
    - 文件重命名逻辑
    - 旧文件清理逻辑
    - _Requirements: 6.2, 7.5_

  - [x] 3.3 编写轮转功能的属性测试
    - **Property 10: 文件轮转触发**
    - **Property 11: 文件数量控制**
    - **Property 12: 轮转策略支持**
    - **Property 13: 默认轮转策略**

- [x] 4. 实现压缩处理器
  - [x] 4.1 创建 CompressionHandler
    - 实现 gzip 压缩
    - 压缩后删除原文件
    - _Requirements: 8.1, 8.2, 8.4, 8.5_

  - [x] 4.2 编写压缩功能的属性测试
    - **Property 14: 文件压缩控制**
    - **Property 15: 压缩格式支持**

- [x] 5. 实现主初始化函数
  - [x] 5.1 创建 init_logger 函数
    - 构建控制台 fmt::Layer（使用内置格式化）
    - 构建文件 fmt::Layer（配合 RotatingFileWriter）
    - 支持 Full/Compact/Json 格式切换
    - 集成 EnvFilter 进行级别过滤
    - _Requirements: 1.1-1.4, 2.1-2.4, 3.1-3.5, 4.1-4.4, 5.1-5.7_

  - [x] 5.2 编写核心功能的属性测试
    - **Property 1: 控制台输出控制**
    - **Property 2: 文件输出控制**
    - **Property 3: 日志级别过滤**
    - **Property 4: 颜色格式化控制**
    - **Property 5: TTY环境检测**
    - **Property 6: 文件路径处理**
    - **Property 7: 文件写入模式**
    - **Property 8: 日志格式化**
    - **Property 9: 默认格式使用**

- [x] 6. 检查点 - 确保核心功能正常工作
  - 运行所有测试
  - 验证基本日志功能

- [x] 7. 实现错误处理
  - [x] 7.1 定义错误类型
    - 使用 thiserror 定义 LoggerError
    - _Requirements: 10.1, 10.3_

  - [x] 7.2 实现错误恢复策略
    - 文件写入失败时的降级处理
    - _Requirements: 10.2, 10.4_

  - [x] 7.3 编写错误处理的属性测试
    - **Property 19: 错误容错处理**
    - **Property 20: 权限错误处理**
    - **Property 21: 输出独立性**

- [ ] 8. 实现动态配置（可选）
  - [x] 8.1 支持运行时修改日志级别
    - 使用 `tracing_subscriber::reload` 功能
    - _Requirements: 9.5_

  - [x] 8.2 编写动态配置的属性测试
    - **Property 18: 动态配置更新**

- [x] 9. 清理和重构
  - [x] 9.1 删除不再需要的自定义代码
    - 移除 ConsoleFormatter（使用内置）
    - 移除 FileFormatter（使用内置）
    - 移除自定义 Layer 实现
    - 简化 subscriber.rs

  - [x] 9.2 更新模块导出

- [ ] 10. 集成测试和文档
  - [ ] 10.1 编写集成测试
  - [ ] 10.2 添加使用示例

- [ ] 11. 最终检查点
  - 确保所有测试通过

## 简化说明

相比原计划，新计划：

1. **移除的任务**：
   - 自定义 ConsoleLayer 实现 → 使用 `fmt::layer()`
   - 自定义 FileLayer 实现 → 使用 `fmt::layer()` + 自定义 Writer
   - 自定义格式化器 → 使用 `compact()` / `json()`
   - 自定义颜色处理 → 使用 `with_ansi()`

2. **保留的任务**：
   - 配置系统（已完成）
   - 轮转 Writer（必须自定义）
   - 压缩处理器（必须自定义）
   - 错误处理

3. **预计代码量**：从 ~1000 行减少到 ~300 行

## Notes

- 充分利用 `tracing-subscriber` 的内置功能
- 只在必要时实现自定义组件
- 保持与原需求的完全兼容
