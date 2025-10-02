# BYR jsDelivr Service

一个最简化的 jsDelivr 风格的 CDN 服务，用于提供 npm 包的文件访问服务。

## 功能特性

- ✅ 从 npm Registry 获取包的元信息和文件
- ✅ 支持自定义 Registry（通过环境变量）
- ✅ 智能入口文件解析（jsdelivr → exports → main）
- ✅ 目录列表展示
- ✅ 文件内容返回（正确的 Content-Type）
- ✅ 语义化版本解析（支持 ^、~、latest 等）
- ✅ 内存缓存（元信息 5 分钟，包文件 1 小时）
- ✅ 高性能 Rust 实现

## 快速开始

### 安装依赖

```bash
# 确保已安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 编译运行

```bash
# 开发模式
cargo run

# 生产模式（优化编译）
cargo build --release
./target/release/byr-jsdelivr
```

### 配置

通过环境变量配置：

```bash
# 设置自定义 Registry（可选，默认使用官方 npm registry）
export REGISTRY=https://registry.npmjs.org

# 设置服务端口（可选，默认 3000）
export PORT=3000

# 运行
cargo run
```

## 使用示例

### 1. 获取包的入口文件（最新版本）

```bash
# 访问 React 的入口文件
curl http://localhost:3000/react

# 访问 Vue 的入口文件
curl http://localhost:3000/vue
```

### 2. 获取指定版本的入口文件

```bash
# 访问 React 18.0.0 的入口文件
curl http://localhost:3000/react@18.0.0

# 使用语义化版本范围
curl http://localhost:3000/react@^18.0.0
```

### 3. 获取指定文件

```bash
# 获取 lodash 的具体文件
curl http://localhost:3000/lodash@4.17.21/lodash.js

# 获取 Vue 的运行时文件
curl http://localhost:3000/vue@3.3.4/dist/vue.runtime.esm-browser.js
```

### 4. 列出目录内容

```bash
# 列出包根目录
curl http://localhost:3000/vue@3.3.4/

# 列出子目录
curl http://localhost:3000/vue@3.3.4/dist/
```

### 5. scoped 包

```bash
# 访问 scoped 包
curl http://localhost:3000/@vue/runtime-core@3.3.4/

# 获取 scoped 包的文件
curl http://localhost:3000/@vue/runtime-core@3.3.4/dist/runtime-core.esm-bundler.js
```

## URL 路由规则

- `/package` - 获取最新版本的入口文件
- `/package@version` - 获取指定版本的入口文件
- `/package@version/` - 列出包根目录（注意末尾斜杠）
- `/package@version/path/to/file` - 获取指定文件
- `/package@^1.2.0` - 使用语义化版本范围
- `/@scope/package@version/file` - scoped 包

## 入口文件解析规则

1. 检查 `package.json` 中的 `jsdelivr` 字段
2. 检查 `exports["."]` 配置：
   - 如果是对象，使用 `default` 字段
   - 如果是字符串，直接使用
3. 使用 `main` 字段
4. 默认尝试 `index.js`

## 缓存策略

- **元信息缓存**：5 分钟（遵循 npm registry 的 cache-control）
- **包文件缓存**：1 小时（版本文件不变）
- **最大容量**：
  - 元信息：1000 条
  - 包文件：500 个

## 技术栈

- **Web 框架**：Axum
- **HTTP 客户端**：reqwest
- **异步运行时**：Tokio
- **压缩解压**：flate2 + tar
- **版本解析**：node-semver
- **缓存**：moka
- **日志**：tracing

## Docker 部署

```dockerfile
# 见 Dockerfile
docker build -t byr-jsdelivr .
docker run -p 3000:3000 -e REGISTRY=https://registry.npmjs.org byr-jsdelivr
```

## 性能特点

- 内存高效：LRU 缓存自动管理
- 并发友好：异步 I/O，支持高并发
- 快速响应：本地缓存减少远程请求
- 低延迟：Rust 编译优化

## 项目结构

```
src/
├── main.rs           # 主入口和路由
├── error.rs          # 错误处理
├── cache.rs          # 缓存管理
├── npm.rs            # npm registry 交互
├── package.rs        # 包解析和获取
├── semver_utils.rs   # 语义化版本处理
└── response.rs       # HTTP 响应生成
```

## 开发

```bash
# 运行测试
cargo test

# 检查代码
cargo clippy

# 格式化代码
cargo fmt

# 查看日志（开发模式）
RUST_LOG=debug cargo run
```

## 限制和注意事项

1. 不支持 npm 包的构建和转换（如 TypeScript 编译）
2. 不支持自动压缩和优化（可后续添加）
3. 缓存仅在内存中，重启后清空
4. 适合内网部署或小规模使用

## License

MIT

## 作者

Steven - BYR Archive Project
