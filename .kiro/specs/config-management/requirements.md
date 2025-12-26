# Requirements Document

## Introduction

本文档定义了 fusion-rs 项目的配置管理模块需求。该模块使用 `config` crate 实现分层配置加载，支持 TOML 格式配置文件、环境变量覆盖，以及多环境配置管理。配置内容涵盖应用信息、服务器设置、数据库连接和日志系统。

## Glossary

- **Config_Manager**: 配置管理器，负责加载、合并和提供配置访问
- **Config_Source**: 配置来源，包括配置文件和环境变量
- **Config_Layer**: 配置层级，按优先级从低到高依次覆盖
- **Settings**: 应用程序的完整配置结构
- **Application_Config**: 应用程序基本信息配置（名称、版本）
- **Server_Config**: Axum 服务器配置（主机、端口等）
- **Database_Config**: Diesel 数据库连接配置
- **Logger_Config**: 日志系统配置

## Requirements

### Requirement 1: 分层配置加载

**User Story:** As a developer, I want to load configuration from multiple sources with clear precedence, so that I can have sensible defaults while allowing environment-specific overrides.

#### Acceptance Criteria

1. THE Config_Manager SHALL load configuration in the following order (later sources override earlier):
   - `default.toml` (基础默认配置)
   - `{environment}.toml` (环境特定配置，如 `development.toml`, `production.toml`)
   - `local.toml` (本地开发覆盖，不提交到版本控制)
   - 以 `FUSION_` 为前缀的环境变量

2. WHEN a configuration key exists in multiple sources, THE Config_Manager SHALL use the value from the highest priority source

3. WHEN `default.toml` does not exist, THE Config_Manager SHALL return an error indicating the missing required file

4. WHEN environment-specific or local configuration files do not exist, THE Config_Manager SHALL continue loading without error

### Requirement 2: 环境配置

**User Story:** As a developer, I want to specify the application environment, so that the correct environment-specific configuration is loaded.

#### Acceptance Criteria

1. THE Config_Manager SHALL read the environment name from the `FUSION_APP_ENV` environment variable

2. WHEN `FUSION_APP_ENV` is not set, THE Config_Manager SHALL default to `development` environment

3. THE Config_Manager SHALL support at minimum the following environments: `development`, `test`, `staging`, `production`

4. WHEN an environment is specified, THE Config_Manager SHALL attempt to load `{environment}.toml` from the configuration directory

### Requirement 3: 配置目录和文件指定

**User Story:** As a developer, I want to customize where configuration files are located, so that I can organize my project structure flexibly.

#### Acceptance Criteria

1. THE Config_Manager SHALL read the configuration directory from the `FUSION_CONFIG_DIR` environment variable

2. WHEN `FUSION_CONFIG_DIR` is not set, THE Config_Manager SHALL default to the `config` directory relative to the project root

3. THE Config_Manager SHALL read a specific configuration file path from the `FUSION_CONFIG_FILE` environment variable

4. WHEN `FUSION_CONFIG_FILE` is set, THE Config_Manager SHALL load only that single file and skip the layered loading process

5. IF both `FUSION_CONFIG_DIR` and `FUSION_CONFIG_FILE` are set, THEN THE Config_Manager SHALL return an error indicating the mutual exclusivity

### Requirement 4: 环境变量覆盖

**User Story:** As a developer, I want to override configuration values via environment variables, so that I can configure the application in containerized environments without modifying files.

#### Acceptance Criteria

1. THE Config_Manager SHALL read environment variables prefixed with `FUSION_`

2. WHEN an environment variable `FUSION_SERVER_PORT` is set, THE Config_Manager SHALL map it to `server.port` in the configuration

3. THE Config_Manager SHALL use double underscore (`__`) as the separator for nested configuration keys (e.g., `FUSION_DATABASE__URL` maps to `database.url`)

4. WHEN an environment variable value cannot be parsed to the expected type, THE Config_Manager SHALL return a descriptive error

### Requirement 5: Application 配置

**User Story:** As a developer, I want to configure basic application information, so that it can be used for identification and versioning.

#### Acceptance Criteria

1. THE Settings SHALL include an `application` section with `name` and `version` fields

2. WHEN `application.name` is not configured, THE Config_Manager SHALL default to `fusion-rs`

3. WHEN `application.version` is not configured, THE Config_Manager SHALL default to `0.1.0`

### Requirement 6: Server 配置

**User Story:** As a developer, I want to configure the Axum HTTP server, so that I can control how the application listens for requests.

#### Acceptance Criteria

1. THE Settings SHALL include a `server` section with configurable fields for Axum server

2. THE Server_Config SHALL include at minimum: `host`, `port`, `request_timeout`, `keep_alive_timeout`

3. WHEN `server.host` is not configured, THE Config_Manager SHALL default to `127.0.0.1`

4. WHEN `server.port` is not configured, THE Config_Manager SHALL default to `3000`

5. WHEN `server.request_timeout` is not configured, THE Config_Manager SHALL default to `30` seconds

6. WHEN `server.keep_alive_timeout` is not configured, THE Config_Manager SHALL default to `75` seconds

### Requirement 7: Database 配置

**User Story:** As a developer, I want to configure the Diesel database connection, so that the application can connect to the correct database.

#### Acceptance Criteria

1. THE Settings SHALL include a `database` section with Diesel-compatible configuration

2. THE Database_Config SHALL include at minimum: `url`, `max_connections`, `min_connections`, `connection_timeout`

3. WHEN `database.url` is not configured, THE Config_Manager SHALL return an error during validation

4. WHEN `database.max_connections` is not configured, THE Config_Manager SHALL default to `10`

5. WHEN `database.min_connections` is not configured, THE Config_Manager SHALL default to `1`

6. WHEN `database.connection_timeout` is not configured, THE Config_Manager SHALL default to `30` seconds

### Requirement 8: Logger 配置

**User Story:** As a developer, I want to configure the logging system via configuration files, so that I can control log output without code changes.

#### Acceptance Criteria

1. THE Settings SHALL include a `logger` section compatible with the existing `LoggerConfig` structure

2. THE Logger_Config in settings SHALL include: `level`, `console` (with `enabled`, `colored`), and `file` (with `enabled`, `path`, `append`, `format`, `rotation`)

3. WHEN logger configuration is provided, THE Config_Manager SHALL produce a valid `LoggerConfig` instance

4. WHEN logger configuration is not provided, THE Config_Manager SHALL use `LoggerConfig::default()`

### Requirement 9: 配置验证

**User Story:** As a developer, I want configuration to be validated on load, so that I catch configuration errors early.

#### Acceptance Criteria

1. WHEN configuration is loaded, THE Config_Manager SHALL validate all required fields are present

2. WHEN configuration is loaded, THE Config_Manager SHALL validate all values are within acceptable ranges

3. IF validation fails, THEN THE Config_Manager SHALL return a descriptive error indicating which field failed and why

4. THE Config_Manager SHALL validate that `database.url` is a valid database connection string format

5. THE Config_Manager SHALL validate that `server.port` is within the valid port range (1-65535)

### Requirement 10: 配置序列化与反序列化

**User Story:** As a developer, I want configuration structures to be serializable, so that I can inspect and export the current configuration.

#### Acceptance Criteria

1. THE Settings structure SHALL implement `Serialize` and `Deserialize` traits

2. WHEN serializing Settings to TOML, THE Config_Manager SHALL produce valid TOML output

3. WHEN deserializing TOML to Settings, THE Config_Manager SHALL correctly parse all nested structures

4. FOR ALL valid Settings instances, serializing then deserializing SHALL produce an equivalent Settings instance (round-trip property)
