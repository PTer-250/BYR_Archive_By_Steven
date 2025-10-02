# 快速启动指南

## ⚡️ 5 分钟快速开始

### 1️⃣ 确保已安装 Rust

```bash
# 检查 Rust 是否已安装
rustc --version

# 如果未安装，运行：
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2️⃣ 启动服务

```bash
# 方式 A: 开发模式（快速启动）
cargo run

# 方式 B: 生产模式（优化性能）
cargo build --release
./target/release/byr-jsdelivr

# 方式 C: 使用启动脚本
chmod +x start.sh
./start.sh
```

### 3️⃣ 测试服务

```bash
# 打开浏览器访问
open http://localhost:3000

# 或使用 curl 测试
curl http://localhost:3000/react
```

---

## 🐳 使用 Docker（推荐）

```bash
# 构建并启动
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down
```

---

## 🎯 常用示例

### 获取包文件

```bash
# React 最新版本
curl http://localhost:3000/react

# Vue 指定版本
curl http://localhost:3000/vue@3.3.4

# Lodash 工具函数
curl http://localhost:3000/lodash@4.17.21/lodash.js

# scoped 包
curl http://localhost:3000/@vue/runtime-core@3.3.4/
```

### 在 HTML 中使用

```html
<!-- 直接引入 -->
<script src="http://localhost:3000/vue"></script>

<!-- 指定版本 -->
<script src="http://localhost:3000/react@18.2.0/umd/react.production.min.js"></script>

<!-- ESM 方式 -->
<script type="module">
  import Vue from 'http://localhost:3000/vue@3.3.4/dist/vue.esm-browser.js';
</script>
```

---

## ⚙️ 配置

### 环境变量

```bash
# 自定义 npm registry
export REGISTRY=https://your-registry.com

# 自定义端口
export PORT=8080

# 日志级别
export RUST_LOG=debug

# 启动
cargo run
```

### 配置文件

```bash
# 复制示例配置
cp .env.example .env

# 编辑配置
vim .env

# 使用配置启动
source .env && cargo run
```

---

## 📊 性能特点

- ⚡️ **极速响应**: Rust 原生性能
- 💾 **智能缓存**: 元信息 5 分钟，包文件 1 小时
- 🔄 **高并发**: 异步 I/O，支持大量并发请求
- 📦 **自动解压**: 自动下载并解压 npm tarball
- 🎯 **版本解析**: 支持语义化版本和 dist-tags

---

## 🌐 在线测试

启动服务后，打开 `test.html` 进行交互式测试：

```bash
# 在浏览器中打开
open test.html
```

或直接访问：`http://localhost:3000/` 查看服务主页。

---

## 📚 更多文档

- [README.md](README.md) - 项目概述和使用说明
- [API.md](API.md) - 完整的 API 文档
- [DEPLOYMENT.md](DEPLOYMENT.md) - 部署和运维指南

---

## 🆘 遇到问题？

### 服务无法启动

```bash
# 检查端口占用
lsof -i :3000

# 使用其他端口
PORT=8080 cargo run
```

### 包下载失败

```bash
# 检查网络连接
curl https://registry.npmjs.org/react

# 使用镜像
export REGISTRY=https://registry.npmmirror.com
cargo run
```

### 查看详细日志

```bash
RUST_LOG=debug cargo run
```

---

## ✨ 功能亮点

✅ 入口文件智能解析（jsdelivr → exports → main）  
✅ 语义化版本支持（^1.0.0, ~1.2.0, latest）  
✅ 目录列表展示  
✅ 正确的 Content-Type 响应  
✅ LRU 缓存策略  
✅ Scoped 包支持  
✅ 自定义 Registry  

---

## 📝 License

MIT License

---

**准备好了吗？立即开始：**

```bash
cargo run
```

然后访问 http://localhost:3000 🚀
