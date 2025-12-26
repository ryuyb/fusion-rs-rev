# Fusion-RS Docker 多架构构建

使用 Docker Buildx 和 TARGETARCH 进行跨平台编译，支持 x86_64 和 ARM64 架构。

## 🚀 快速开始

### 使用 Docker Buildx（推荐）

```bash
# 创建 buildx builder（首次使用）
docker buildx create --name fusion-builder --use

# 构建多架构镜像并推送到 Registry
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t your-registry.com/fusion-rs:latest \
  --push .

# 本地构建特定架构
docker buildx build \
  --platform linux/amd64 \
  -t fusion-rs:amd64 \
  --load .
```

### 使用构建脚本

```bash
# 本地构建两个架构
./build-multiarch.sh

# 构建并推送到 Registry
REGISTRY=your-registry.com ./build-multiarch.sh

# 构建并测试镜像
TEST_IMAGES=true ./build-multiarch.sh
```

### 传统构建（当前平台）

```bash
# 构建当前平台镜像
docker build -t fusion-rs:latest .
```

## 🏗️ 特性

- **TARGETARCH 支持**: 使用 Docker Buildx 的原生多架构支持
- **Zig 交叉编译**: 使用 Zig 作为链接器，简化跨平台构建
- **静态链接**: 完全静态链接的二进制文件
- **最小镜像**: 使用 `scratch` 基础镜像，极小的镜像大小
- **安全**: 非 root 用户运行
- **缓存优化**: 依赖构建缓存，加速重复构建

## 🔧 环境变量

- `RUST_LOG`: 日志级别 (默认: info)
- `FUSION_SERVER__HOST`: 服务器地址 (默认: 0.0.0.0)  
- `FUSION_SERVER__PORT`: 服务器端口 (默认: 8080)
- `FUSION_DATABASE__URL`: 数据库连接字符串

## 🐳 运行容器

```bash
# 运行多架构镜像（自动选择架构）
docker run -p 8080:8080 fusion-rs:latest

# 指定架构运行
docker run --platform linux/amd64 -p 8080:8080 fusion-rs:latest
docker run --platform linux/arm64 -p 8080:8080 fusion-rs:latest

# 使用 Docker Compose
docker-compose up -d
```

## 📊 架构验证

```bash
# 检查镜像支持的架构
docker buildx imagetools inspect fusion-rs:latest

# 查看镜像详细信息
docker buildx imagetools inspect fusion-rs:latest --format "{{json .}}"

# 测试不同架构
docker run --rm --platform linux/amd64 fusion-rs:latest /app/fusion-rs --version
docker run --rm --platform linux/arm64 fusion-rs:latest /app/fusion-rs --version
```

## 🚀 CI/CD 集成

### GitHub Actions 示例

```yaml
name: Build Multi-Arch Docker Image

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v3
    
    - name: Login to Registry
      uses: docker/login-action@v3
      with:
        registry: ghcr.io
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    
    - name: Build and push
      uses: docker/build-push-action@v5
      with:
        context: .
        platforms: linux/amd64,linux/arm64
        push: true
        tags: ghcr.io/${{ github.repository }}:latest
        cache-from: type=gha
        cache-to: type=gha,mode=max
```

## 🔍 故障排除

### 常见问题

1. **Buildx 不可用**
   ```bash
   # 安装 buildx
   docker buildx install
   
   # 创建 builder
   docker buildx create --name fusion-builder --use
   ```

2. **跨架构构建失败**
   ```bash
   # 启用 QEMU 模拟
   docker run --privileged --rm tonistiigi/binfmt --install all
   ```

3. **镜像无法加载**
   ```bash
   # 本地构建只能加载一个架构
   docker buildx build --platform linux/amd64 -t fusion-rs:amd64 --load .
   ```

## 📦 镜像大小

使用 `scratch` 基础镜像和静态链接：

- 镜像大小: ~15MB
- 无额外依赖
- 快速启动时间