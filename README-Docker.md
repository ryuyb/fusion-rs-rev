# Fusion-RS Docker 多架构构建

使用 Zig 进行跨平台编译，支持 x86_64 和 ARM64 架构。

## 🚀 快速开始

### 构建特定架构

```bash
# 构建 AMD64 镜像
docker build --target amd64 -t fusion-rs:amd64 .

# 构建 ARM64 镜像  
docker build --target arm64 -t fusion-rs:arm64 .
```

### 使用构建脚本

```bash
# 构建两个架构的镜像
./build-multiarch.sh

# 构建并推送到 Registry
REGISTRY=your-registry.com ./build-multiarch.sh
```

### 运行容器

```bash
# 运行 AMD64 版本
docker run -p 8080:8080 fusion-rs:latest-amd64

# 运行 ARM64 版本
docker run -p 8080:8080 fusion-rs:latest-arm64

# 使用 Docker Compose（默认 AMD64）
docker-compose up -d

# 使用 Docker Compose 指定架构
DOCKER_TARGET=arm64 docker-compose up -d
```

## 🏗️ 特性

- **Zig 交叉编译**: 使用 Zig 作为链接器，简化跨平台构建
- **静态链接**: 完全静态链接的二进制文件
- **最小镜像**: 使用 `scratch` 基础镜像，极小的镜像大小
- **安全**: 非 root 用户运行
- **多架构**: 同时支持 x86_64 和 ARM64

## 📦 镜像大小对比

使用 `scratch` 基础镜像和静态链接，镜像大小显著减小：

- 传统 Alpine 镜像: ~50MB
- 当前 scratch 镜像: ~15MB

## 🔧 环境变量

- `RUST_LOG`: 日志级别 (默认: info)
- `FUSION_SERVER__HOST`: 服务器地址 (默认: 0.0.0.0)  
- `FUSION_SERVER__PORT`: 服务器端口 (默认: 8080)
- `FUSION_DATABASE__URL`: 数据库连接字符串

## 🐳 Docker Compose

```yaml
# 使用特定架构
DOCKER_TARGET=arm64 docker-compose up -d

# 查看日志
docker-compose logs -f app
```

## 🚀 生产部署

```bash
# 构建生产镜像
docker build --target amd64 -t fusion-rs:prod .

# 运行生产容器
docker run -d \
  --name fusion-rs-prod \
  -p 8080:8080 \
  -e FUSION_DATABASE__URL="postgres://..." \
  -e RUST_LOG="warn" \
  fusion-rs:prod
```