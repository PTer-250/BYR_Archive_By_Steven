# 开发和部署指南

## 开发环境搭建

### 1. 安装 Rust

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 或使用 Homebrew (macOS)
brew install rust

# 验证安装
rustc --version
cargo --version
```

### 2. 克隆项目

```bash
git clone <repository-url>
cd BYR_Archive_By_Steven
```

### 3. 安装依赖并编译

```bash
# 检查依赖
cargo check

# 运行测试
cargo test

# 开发模式运行
cargo run

# 生产模式编译
cargo build --release
```

### 4. 配置环境变量

```bash
# 复制环境变量示例文件
cp .env.example .env

# 编辑 .env 文件
export REGISTRY=https://registry.npmjs.org
export PORT=3000
export RUST_LOG=byr_jsdelivr=debug
```

---

## 本地开发

### 启动开发服务器

```bash
# 方式 1: 使用 cargo
RUST_LOG=debug cargo run

# 方式 2: 使用启动脚本
./start.sh

# 方式 3: 编译后运行
cargo build --release
./target/release/byr-jsdelivr
```

### 查看日志

```bash
# 设置日志级别
export RUST_LOG=byr_jsdelivr=debug,tower_http=debug

# 查看详细日志
cargo run
```

### 测试服务

```bash
# 测试主页
curl http://localhost:3000/

# 测试包访问
curl http://localhost:3000/react

# 使用测试页面
open test.html  # 或在浏览器中打开 http://localhost:3000 后手动访问 test.html
```

---

## 生产部署

### 方式 1: 直接部署二进制文件

```bash
# 1. 编译 release 版本
cargo build --release

# 2. 复制二进制文件到服务器
scp target/release/byr-jsdelivr user@server:/opt/byr-jsdelivr/

# 3. 在服务器上配置环境变量
cat > /etc/systemd/system/byr-jsdelivr.service <<EOF
[Unit]
Description=BYR jsDelivr Service
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/byr-jsdelivr
Environment="REGISTRY=https://registry.npmjs.org"
Environment="PORT=3000"
Environment="RUST_LOG=byr_jsdelivr=info"
ExecStart=/opt/byr-jsdelivr/byr-jsdelivr
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# 4. 启动服务
sudo systemctl daemon-reload
sudo systemctl enable byr-jsdelivr
sudo systemctl start byr-jsdelivr

# 5. 查看状态
sudo systemctl status byr-jsdelivr
```

### 方式 2: Docker 部署

```bash
# 1. 构建镜像
docker build -t byr-jsdelivr .

# 2. 运行容器
docker run -d \
  --name byr-jsdelivr \
  -p 3000:3000 \
  -e REGISTRY=https://registry.npmjs.org \
  -e PORT=3000 \
  --restart unless-stopped \
  byr-jsdelivr

# 3. 查看日志
docker logs -f byr-jsdelivr

# 4. 停止容器
docker stop byr-jsdelivr

# 5. 删除容器
docker rm byr-jsdelivr
```

### 方式 3: Docker Compose 部署

```bash
# 1. 启动服务
docker-compose up -d

# 2. 查看日志
docker-compose logs -f

# 3. 停止服务
docker-compose down

# 4. 重新构建
docker-compose up -d --build
```

---

## Nginx 反向代理配置

### 基础配置

```nginx
upstream byr_jsdelivr {
    server 127.0.0.1:3000;
}

server {
    listen 80;
    server_name cdn.example.com;

    location / {
        proxy_pass http://byr_jsdelivr;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # 缓存配置
        proxy_cache_bypass $http_upgrade;
        proxy_no_cache $http_upgrade;
    }
}
```

### 带缓存的配置

```nginx
# 定义缓存路径
proxy_cache_path /var/cache/nginx/byr_jsdelivr 
    levels=1:2 
    keys_zone=byr_jsdelivr_cache:10m 
    max_size=1g 
    inactive=60m 
    use_temp_path=off;

upstream byr_jsdelivr {
    server 127.0.0.1:3000;
}

server {
    listen 80;
    server_name cdn.example.com;

    location / {
        proxy_pass http://byr_jsdelivr;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # 启用缓存
        proxy_cache byr_jsdelivr_cache;
        proxy_cache_valid 200 1h;
        proxy_cache_valid 404 5m;
        proxy_cache_use_stale error timeout updating http_500 http_502 http_503 http_504;
        proxy_cache_lock on;
        
        # 添加缓存状态头
        add_header X-Cache-Status $upstream_cache_status;
        
        # Gzip 压缩
        gzip on;
        gzip_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript;
    }
}
```

### HTTPS 配置

```nginx
server {
    listen 443 ssl http2;
    server_name cdn.example.com;

    ssl_certificate /path/to/ssl/cert.pem;
    ssl_certificate_key /path/to/ssl/key.pem;
    
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    location / {
        proxy_pass http://byr_jsdelivr;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 80;
    server_name cdn.example.com;
    return 301 https://$server_name$request_uri;
}
```

---

## 监控和维护

### 健康检查

```bash
# 简单检查
curl http://localhost:3000/

# 监控脚本
cat > /usr/local/bin/check-byr-jsdelivr.sh <<'EOF'
#!/bin/bash
if ! curl -f http://localhost:3000/ > /dev/null 2>&1; then
    echo "Service is down, restarting..."
    systemctl restart byr-jsdelivr
fi
EOF

chmod +x /usr/local/bin/check-byr-jsdelivr.sh

# 添加到 crontab
crontab -e
# */5 * * * * /usr/local/bin/check-byr-jsdelivr.sh
```

### 日志管理

```bash
# 使用 journalctl 查看 systemd 服务日志
sudo journalctl -u byr-jsdelivr -f

# 查看最近的日志
sudo journalctl -u byr-jsdelivr -n 100

# 查看特定时间的日志
sudo journalctl -u byr-jsdelivr --since "2023-10-01" --until "2023-10-02"
```

### 性能监控

```bash
# 使用 htop 查看资源占用
htop -p $(pgrep byr-jsdelivr)

# 使用 systemd-cgtop 监控
systemd-cgtop

# 查看网络连接
ss -tlnp | grep 3000
```

---

## 性能优化

### 1. 调整缓存大小

编辑 `src/cache.rs`:

```rust
// 增加缓存容量
metadata_cache: Cache::builder()
    .max_capacity(5000)  // 增加到 5000
    .time_to_live(Duration::from_secs(300))
    .build(),

package_cache: Cache::builder()
    .max_capacity(2000)  // 增加到 2000
    .time_to_live(Duration::from_secs(3600))
    .build(),
```

### 2. 使用持久化缓存

可以考虑将缓存改为持久化存储（Redis、文件系统等）。

### 3. 多实例部署

```bash
# 启动多个实例
PORT=3001 cargo run &
PORT=3002 cargo run &
PORT=3003 cargo run &

# Nginx 负载均衡
upstream byr_jsdelivr {
    server 127.0.0.1:3001;
    server 127.0.0.1:3002;
    server 127.0.0.1:3003;
}
```

### 4. 使用 CDN

将服务部署在 Nginx 后面，并使用 Cloudflare、AWS CloudFront 等 CDN 服务。

---

## 故障排查

### 服务无法启动

```bash
# 检查端口是否被占用
lsof -i :3000

# 检查日志
sudo journalctl -u byr-jsdelivr -n 50

# 检查环境变量
systemctl show byr-jsdelivr -p Environment
```

### 包无法下载

```bash
# 测试 Registry 连接
curl https://registry.npmjs.org/react

# 检查网络
ping registry.npmjs.org

# 查看详细日志
RUST_LOG=debug cargo run
```

### 内存占用过高

```bash
# 查看内存使用
ps aux | grep byr-jsdelivr

# 减小缓存容量（编辑 src/cache.rs）
# 或重启服务清空缓存
systemctl restart byr-jsdelivr
```

---

## 安全建议

1. **使用 HTTPS**: 始终通过 HTTPS 提供服务
2. **限制访问**: 使用防火墙限制访问源
3. **速率限制**: 在 Nginx 中配置速率限制
4. **更新依赖**: 定期更新 Rust 依赖
5. **监控日志**: 定期检查异常访问

### Nginx 速率限制

```nginx
limit_req_zone $binary_remote_addr zone=cdn_limit:10m rate=100r/s;

server {
    location / {
        limit_req zone=cdn_limit burst=50 nodelay;
        proxy_pass http://byr_jsdelivr;
    }
}
```

---

## 更新和升级

```bash
# 1. 拉取最新代码
git pull

# 2. 重新编译
cargo build --release

# 3. 重启服务
sudo systemctl restart byr-jsdelivr

# 或使用 Docker
docker-compose down
docker-compose up -d --build
```

---

## 备份和恢复

由于服务是无状态的（缓存在内存中），不需要特别的备份。但建议：

1. 备份配置文件
2. 备份环境变量设置
3. 记录自定义的修改

---

## 常见问题

### Q: 如何使用私有 npm registry？

```bash
export REGISTRY=https://your-private-registry.com
cargo run
```

### Q: 如何增加缓存时间？

编辑 `src/cache.rs` 中的 `time_to_live` 参数。

### Q: 支持哪些版本格式？

- 精确版本: `1.2.3`
- 语义化范围: `^1.0.0`, `~1.2.0`, `>=1.0.0`
- dist-tags: `latest`, `next`, `beta`

### Q: 如何查看缓存状态？

目前缓存状态不对外暴露，可以通过日志查看缓存命中情况。

---

## 联系支持

如有问题，请提交 Issue 或联系开发者。
