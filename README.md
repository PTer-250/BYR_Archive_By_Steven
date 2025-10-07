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

---

## 📘 项目总结报告（BYR_Archive_By_Steven）

> 本章节为本次考核 / 实战开发的完整总结，包含设计思路、技术选型、实现细节、问题分析、优缺点、心路历程与改进展望。旨在为后续维护者或评审者提供一个系统、立体的全景视角。

### 1. 设计思路概述

目标是在 npm 官方（或私有） Registry 与用户之间构建一个“轻量版 jsDelivr”中转 / 缓存层：

1. 提供对 npm 包的 文件 / 目录 / 入口文件 的 HTTP 访问能力；
2. 可通过 URL 表达版本（精确 / 范围 / dist-tag），并自动解析语义化版本；
3. 保持极简、高性能、易部署（单二进制 + 可选 Docker）；
4. 支持企业/内网场景，通过环境变量切换私有 npm registry；
5. 提供基础缓存，降低重复访问带来的延迟与外网请求开销；
6. 结构清晰模块化，方便未来迭代：构建、压缩、打包、持久化缓存、监控等。

因此架构选择“最小可用 + 可持续扩展”的策略：先满足核心链路（解析 → 拉取 → 入口推断 → 响应），再补充增强特性（语义化版本、缓存、目录索引、文档与 CI）。

### 2. 架构与模块划分

| 模块 | 文件 | 职责 | 说明 |
|------|------|------|------|
| 启动与路由 | `main.rs` | 依赖注入 / 路由注册 / 服务启动 | 入口，集中管理全局状态与 Axum 组合 |
| 错误处理 | `error.rs` | 统一错误枚举与 HTTP 映射 | 保持调用链简洁，日志聚合 |
| 缓存层 | `cache.rs` | 元信息缓存 + 包文件缓存 (moka) | 以 TTL + LRU 控制内存占用 |
| Registry 交互 | `npm.rs` | 获取包元信息 / 下载 & 解压 tarball | 专注 I/O 与解压逻辑 |
| 包语义逻辑 | `package.rs` | URL 解析 / 入口文件推断 / 包装数据结构 | 业务逻辑核心聚集点 |
| 版本解析 | `semver_utils.rs` | 语义化范围、dist-tags 解析 | 使用 `node-semver` 库；内部抽象方便扩展 |
| 响应构造 | `response.rs` | 文件响应 / 目录索引 HTML / MIME 识别 | 纯粹输出层，便于替换风格 |

整体依赖流向：
```
main ─▶ (npm, package, semver_utils, response, cache, error)
      └── package ─▶ (npm, cache)
      └── response ─▶ (mime_guess)
```

### 3. 关键流程说明

#### 3.1 URL 解析
输入路径示例：
```
/react                       => 包 + latest 入口文件
/react@18.2.0                => 指定版本入口文件
/lodash@^4.17.0/             => 语义化范围 + 目录列表
/vue@3.3.4/dist/vue.global.js => 精确文件访问
/@vue/runtime-core@3.3.4/    => scoped 包目录
```
解析结果三元组：`(package_name, version_spec_opt, file_path_opt)`

#### 3.2 版本解析策略
1. 未指定版本 → 使用 `dist-tags.latest`
2. 直接命中 `dist-tags` → 返回映射版本
3. 精确存在于 `versions` → 返回
4. 否则解析为范围（^ ~ >= <= etc.）→ 过滤排序 → 选择最大满足版本
5. 均失败 → 404

#### 3.3 包获取流程
```
fetch_package_metadata → (缓存命中? 否则 请求 /{package})
  ↓
resolve_version → 得到具体 version
  ↓
fetch_package → 取缓存 or 下载 tarball (dist.tarball)
  ↓
download_and_extract_tarball → flate2 + tar 解压 → HashMap<path, bytes>
  ↓
resolve_entry_file → jsdelivr > exports["."] > main > index.js
  ↓
file_response / directory_listing
```

#### 3.4 缓存策略
| 类型 | Key 形式 | TTL | 上限 | 说明 |
|------|---------|-----|------|------|
| 元信息 JSON | `metadata:{name}` | 300s | 1000 | 与官方 cache-control 对齐 |
| 包文件数据 | `package:{name}@{version}` | 3600s | 500 | 版本内容不变，适合长 TTL |

### 4. 实现中的问题与解决方案

| 问题 | 现象 | 解决方式 | 备注 |
|------|------|----------|------|
| http crate 版本冲突 | reqwest / axum 引入不同版本 http | 避免直接比较 `StatusCode` 类型，使用 `as_u16()` | 降低耦合 |
| tar 解压所有权/借用报错 | `entry.path()` 后继续可变借用 | 先转为 `PathBuf` 再使用 | Rust 所有权思维再校准 |
| scoped 包解析复杂 | `@scope/name@version/path` 歧义 | 分段扫描第二个 `@`/`/` 实现有限状态机式解析 | 覆盖主要场景 |
| 目录列表重复项 | 嵌套路径派生多次 | 收集后 `sort + dedup` | 简化逻辑优先 |
| MIME 猜测 vs 手工映射 | js/ts/json map 差异 | 保留自定义映射优先级，其它 fallback `application/octet-stream` | 可扩展 |
| 语义化范围解析边界 | `pre-release` 排序问题 | 直接使用 `node-semver` crate 内部实现 | 减少重复造轮子 |
| 缓存一致性 | 更新（几乎不发生） vs 读取 | 由于 npm 版本不可变，牺牲强一致换取简单 | 可后续引入软失效策略 |
| 错误分层 | 同时要返回状态又要日志 | 定义 `AppError` 并实现 `IntoResponse` | 可扩展更多 Variant |

### 5. 方案优点与不足

#### ✅ 优点
1. **结构清晰**：关注点分离，便于维护与扩展；
2. **性能可控**：内存缓存 + 只拉取真实访问包；
3. **部署简单**：单二进制 + Docker 支持 + Compose；
4. **可演进空间大**：可按需接入构建/压缩/CDN/鉴权/限流；
5. **语义化版本支持完善**：适配主流范围写法；
6. **极少依赖外部服务**：无需数据库 / Redis 即可运行。

#### ⚠️ 不足（已知限制）
1. **无持久化缓存**：重启后需重新 warm-up；
2. **未做内容转换**：缺少 TS 编译 / Tree Shaking / 压缩；
3. **无监控指标暴露**：暂不支持 Prometheus；
4. **并发下载限流缺失**：极端高并发下可能同时竞争同一包；
5. **缺少预取策略**：热门包主动预热未实现；
6. **错误页面样式简陋**：适合后期 UX 优化。

### 6. 心路历程（开发日志式思考）

| 阶段 | 心态 & 决策 | 产出 |
|------|-------------|------|
| 需求拆解 | 先圈定“最小闭环”替代完整仿真 jsDelivr | 路由/入口/下载/缓存四大块确定 |
| 架构定型 | 纠结是否引入中间件层（鉴权/限流）→ 暂时放弃 | 保持纯粹，主干清晰 |
| 版本解析 | 手写 Range 解析 vs 引库 → 选择引库 | 降低 bug 风险，聚焦业务 |
| 缓存选择 | HashMap + 定时清理 vs 第三方 | 采用 moka，节省时间 |
| 错误模型 | 是否全用 anyhow? → 改用显式枚举 | 前后端语义更清晰 |
| 目录功能 | 是否做 JSON API? → 先做 HTML 方便浏览器直接用 | 未来可扩展 `?format=json` |
| 打磨阶段 | 补文档 / CI / Makefile / demo 脚本 | 面向交付标准化 |
| 回顾 | 剩余优化点很多但核心目标已达成 | 保持迭代节奏感 |

### 7. 性能与可扩展性思考（简版）

| 场景 | 当前表现 | 可行优化 |
|------|----------|----------|
| 热门包高频访问 | 命中内存缓存，极快 | 增加本地磁盘 / Redis 二级缓存 |
| 冷启动 | 首次请求下载 + 解压 | 预热机制 / 启动加载热门清单 |
| 大包（数万文件） | 内存 HashMap 占用增大 | 流式解压 + 按需加载（lazy read） |
| 并发重复请求同一包 | 可能发生 N 次并行下载 | 加入“正在下载”去重（futures::Shared） |
| 可观测性 | 依赖日志 | 集成 metrics + tracing exporter |

### 8. 可能的下一步迭代路线图 (Roadmap)

1. 持久化缓存层（本地磁盘索引 or Redis）
2. 文件转换：TS→JS、ESM/CJS 统一、压缩（esbuild/rolldown）
3. CDN 头部优化（ETag、Last-Modified、自定义 Cache-Control）
4. 访问频次统计与热门包预热机制
5. 限流与防护（IP 限制 / Token 鉴权 / Web Application Firewall 集成）
6. 健康检查与 `/metrics`（Prometheus 导出）
7. 提供 JSON API（例如 `?meta` 查询元信息、`?list` 输出目录 JSON）
8. 支持 Range 请求 / 流式响应（针对大文件）
9. 添加端到端性能基准（criterion / vegeta / wrk）
10. 多实例部署 + 反向代理（Nginx） + 灰度升级策略

### 9. 若未实现全部功能的价值说明

即便某些高级特性（压缩 / 持久化 / 构建转换）尚未落地，本实现已经：
* 覆盖用户最核心访问链路；
* 为后续功能留出清晰扩展点；
* 文档化程度高，降低交接成本；
* 技术选型偏主流生态（Axum + Tokio + Reqwest + Moka），学习与招聘成本低。

这在工程上是一个“可以立即上架使用，同时又具备演进弹性”的良好平衡点。

### 10. 个人收获 / 感悟

1. 把“看似庞杂的 CDN 体系”拆分为纯粹的“请求解析 + 文件分发”后实现变得顺畅；
2. Rust 的所有权与异步生态，配合清晰的模块边界，能快速建立稳定内核；
3. 适度借助第三方库（moka / node-semver）提升交付速度，而不是重复造轮子；
4. 文档与可维护性是“功能完成度”之外的第二生命线；
5. 先交付一个正确、可运行的 MVP，有助于后续迭代建立信心与方向感。

### 11. 总结

本项目以“最小可行 + 易演进”为核心设计理念，在较短周期内实现了：
* 完整的包获取、版本解析、入口分发、目录索引、缓存链路；
* 结构清晰、具备持续扩展基础的代码架构；
* 文档、脚本、CI 较完善的配套交付；
* 明确的不足与可落地的后续路线。

> 如果你计划将其用于生产，请优先补充：并发下载去重、持久化缓存、监控指标与限流策略。

欢迎继续迭代与探索！🚀

