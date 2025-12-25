# Requirements Document

## Introduction

在 Rust 后端项目中集成 Diesel ORM 和 PostgreSQL 数据库，创建 user 表并实现完整的数据访问层。遵循项目结构规范，实现用户数据的 CRUD 操作，包含自动时间戳管理功能。

## Glossary

- **Diesel**: Rust 的类型安全 ORM 框架
- **Schema**: Diesel 自动生成的数据库表结构定义
- **Migration**: 数据库迁移文件，用于版本化管理数据库结构变更
- **Repository**: 数据访问层，封装数据库 CRUD 操作
- **DTO**: Data Transfer Object，用于请求/响应的数据传输对象
- **Domain_Model**: 业务领域模型，表示数据库实体

## Requirements

### Requirement 1: 数据库迁移配置

**User Story:** As a developer, I want to set up Diesel migrations, so that I can version control database schema changes.

#### Acceptance Criteria

1. WHEN the project is initialized, THE Migration_System SHALL create a migrations directory structure
2. WHEN a migration is created, THE Migration_System SHALL generate up.sql and down.sql files
3. WHEN diesel migration run is executed, THE Migration_System SHALL apply pending migrations to the database

### Requirement 2: User 表结构定义

**User Story:** As a developer, I want to create a user table with proper fields, so that I can store user information.

#### Acceptance Criteria

1. THE User_Table SHALL contain an id field of PostgreSQL SERIAL type as primary key
2. THE User_Table SHALL contain a username field of VARCHAR(255) type with NOT NULL constraint
3. THE User_Table SHALL contain an email field of VARCHAR(255) type with NOT NULL and UNIQUE constraint
4. THE User_Table SHALL contain a password field of VARCHAR(255) type with NOT NULL constraint
5. THE User_Table SHALL contain a created_at field of TIMESTAMP type with default value of CURRENT_TIMESTAMP
6. THE User_Table SHALL contain an updated_at field of TIMESTAMP type with default value of CURRENT_TIMESTAMP
7. WHEN a record is updated, THE Database_Trigger SHALL automatically update the updated_at field to current timestamp

### Requirement 3: Diesel Schema 生成

**User Story:** As a developer, I want Diesel to generate type-safe schema definitions, so that I can interact with the database safely.

#### Acceptance Criteria

1. WHEN migrations are applied, THE Diesel_CLI SHALL generate schema.rs file automatically
2. THE Schema_Definition SHALL include all table columns with correct Rust types
3. THE Schema_Definition SHALL be placed in src/schema.rs following project structure

### Requirement 4: Domain Model 定义

**User Story:** As a developer, I want to define Rust structs that map to database tables, so that I can work with typed data.

#### Acceptance Criteria

1. THE User_Model SHALL derive Queryable trait for reading from database
2. THE User_Model SHALL derive Selectable trait for type-safe column selection
3. THE NewUser_Model SHALL derive Insertable trait for inserting new records
4. THE UpdateUser_Model SHALL derive AsChangeset trait for partial updates
5. WHEN serializing models, THE Serialization_System SHALL convert timestamps to appropriate format

### Requirement 5: 数据库连接池配置

**User Story:** As a developer, I want to configure database connection pooling, so that I can efficiently manage database connections.

#### Acceptance Criteria

1. THE Connection_Pool SHALL use r2d2 connection pool manager
2. THE Connection_Pool SHALL read DATABASE_URL from environment variables
3. IF DATABASE_URL is not set, THEN THE Connection_Pool SHALL return a descriptive error
4. THE Connection_Pool SHALL be accessible as shared application state

### Requirement 6: User Repository 实现

**User Story:** As a developer, I want a repository layer for user operations, so that I can perform CRUD operations on users.

#### Acceptance Criteria

1. WHEN creating a user, THE User_Repository SHALL insert a new record and return the created user with generated id
2. WHEN querying a user by id, THE User_Repository SHALL return Option containing the user if found
3. WHEN querying a user by email, THE User_Repository SHALL return Option containing the user if found
4. WHEN listing users, THE User_Repository SHALL return a Vec of all users
5. WHEN updating a user, THE User_Repository SHALL apply partial updates and return the updated user
6. WHEN deleting a user, THE User_Repository SHALL remove the record and return the number of affected rows
7. IF a database operation fails, THEN THE User_Repository SHALL return a descriptive error

### Requirement 7: 项目结构遵循

**User Story:** As a developer, I want the code to follow the project structure guidelines, so that the codebase remains organized.

#### Acceptance Criteria

1. THE Database_Module SHALL be placed in src/db/ directory
2. THE Repository_Module SHALL be placed in src/repositories/ directory
3. THE Model_Module SHALL be placed in src/models/ directory
4. THE Schema_File SHALL be placed in src/schema.rs
5. THE Error_Module SHALL be placed in src/error/ directory
