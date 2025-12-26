# Health Check Endpoints

Fusion-RS 提供了多个健康检查端点，用于监控和负载均衡器的健康检查。

## 📋 可用端点

### 1. 综合健康检查
```
GET /health
```

返回详细的健康信息，包括数据库连接状态。

**响应示例：**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "timestamp": "2024-01-01T12:00:00Z",
  "checks": {
    "database": {
      "status": "healthy",
      "message": "Connected",
      "response_time_ms": 5
    }
  }
}
```

**状态码：**
- `200 OK` - 服务健康
- `503 Service Unavailable` - 服务不健康

### 2. 就绪检查
```
GET /health/ready
```

用于 Kubernetes 就绪探针，检查服务是否准备好接收流量。

**状态码：**
- `200 OK` - 服务就绪
- `503 Service Unavailable` - 服务未就绪

### 3. 存活检查
```
GET /health/live
```

用于 Kubernetes 存活探针，检查服务是否存活（轻量级检查）。

**状态码：**
- `200 OK` - 服务存活

## 🔧 健康状态

### 状态类型
- `healthy` - 所有系统正常运行
- `degraded` - 存在一些非关键问题
- `unhealthy` - 存在关键问题

### 检查项目
- **database** - 数据库连接和查询测试

## 🐳 Docker 健康检查

Dockerfile 中已配置自动健康检查：

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1
```

## ☸️ Kubernetes 配置

### 就绪探针
```yaml
readinessProbe:
  httpGet:
    path: /health/ready
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 5
```

### 存活探针
```yaml
livenessProbe:
  httpGet:
    path: /health/live
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
```

## 📊 监控集成

### Prometheus 监控
健康检查端点可以与 Prometheus 集成：

```yaml
- job_name: 'fusion-rs'
  static_configs:
    - targets: ['fusion-rs:8080']
  metrics_path: /health
  scrape_interval: 30s
```

### 负载均衡器配置

#### Nginx
```nginx
upstream fusion_backend {
    server fusion-rs-1:8080;
    server fusion-rs-2:8080;
}

location /health {
    access_log off;
    return 200 "healthy\n";
    add_header Content-Type text/plain;
}
```

#### HAProxy
```
backend fusion_servers
    option httpchk GET /health/ready
    server fusion1 fusion-rs-1:8080 check
    server fusion2 fusion-rs-2:8080 check
```

## 🧪 测试健康检查

```bash
# 基本健康检查
curl http://localhost:8080/health

# 就绪检查
curl http://localhost:8080/health/ready

# 存活检查
curl http://localhost:8080/health/live

# 检查响应时间
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:8080/health
```

其中 `curl-format.txt` 内容：
```
     time_namelookup:  %{time_namelookup}\n
        time_connect:  %{time_connect}\n
     time_appconnect:  %{time_appconnect}\n
    time_pretransfer:  %{time_pretransfer}\n
       time_redirect:  %{time_redirect}\n
  time_starttransfer:  %{time_starttransfer}\n
                     ----------\n
          time_total:  %{time_total}\n
```