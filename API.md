# API 文档

## 概述

BYR jsDelivr Service 提供了一个简单的 REST API 来访问 npm 包的文件。

## 基础 URL

```
http://localhost:3000
```

## 端点

### 1. 主页

```
GET /
```

返回服务的主页和使用说明。

**响应**: HTML

---

### 2. 获取包的入口文件（最新版本）

```
GET /{package}
```

返回包的最新版本的入口文件。

**参数**:
- `package`: 包名（必需）

**入口文件解析规则**:
1. `package.json` 中的 `jsdelivr` 字段
2. `exports["."].default` 或 `exports["."]`
3. `main` 字段
4. 默认 `index.js`

**示例**:
```bash
curl http://localhost:3000/react
curl http://localhost:3000/lodash
curl http://localhost:3000/@vue/runtime-core
```

**响应**: 
- **成功**: 200 OK，文件内容
- **失败**: 404 Not Found

**Content-Type**: 根据文件扩展名自动设置

---

### 3. 获取指定版本的入口文件

```
GET /{package}@{version}
```

返回包的指定版本的入口文件。

**参数**:
- `package`: 包名（必需）
- `version`: 版本号或范围（必需）

**版本格式支持**:
- 精确版本: `1.2.3`
- 语义化版本范围: `^1.2.0`, `~1.2.0`, `>=1.0.0`
- dist-tags: `latest`, `next`, `beta`

**示例**:
```bash
curl http://localhost:3000/react@18.0.0
curl http://localhost:3000/vue@^3.0.0
curl http://localhost:3000/lodash@latest
```

**响应**: 
- **成功**: 200 OK，文件内容
- **失败**: 404 Not Found

---

### 4. 列出目录内容

```
GET /{package}@{version}/
GET /{package}@{version}/{path}/
```

列出包的目录内容（注意末尾的 `/`）。

**参数**:
- `package`: 包名（必需）
- `version`: 版本号或范围（必需）
- `path`: 子目录路径（可选）

**示例**:
```bash
# 列出根目录
curl http://localhost:3000/vue@3.3.4/

# 列出子目录
curl http://localhost:3000/vue@3.3.4/dist/
curl http://localhost:3000/lodash@4.17.21/fp/
```

**响应**: 
- **成功**: 200 OK，HTML 格式的目录列表
- **失败**: 404 Not Found

---

### 5. 获取指定文件

```
GET /{package}@{version}/{path}
```

返回包中的指定文件内容。

**参数**:
- `package`: 包名（必需）
- `version`: 版本号或范围（必需）
- `path`: 文件路径（必需）

**示例**:
```bash
curl http://localhost:3000/react@18.0.0/index.js
curl http://localhost:3000/vue@3.3.4/dist/vue.runtime.esm-browser.js
curl http://localhost:3000/lodash@4.17.21/lodash.min.js
curl http://localhost:3000/@babel/core@7.22.0/package.json
```

**响应**: 
- **成功**: 200 OK，文件内容
- **失败**: 404 Not Found

**Content-Type**: 根据文件扩展名自动设置

---

## Content-Type 映射

| 扩展名 | Content-Type |
|--------|-------------|
| `.js`, `.mjs`, `.cjs` | `application/javascript; charset=utf-8` |
| `.json` | `application/json; charset=utf-8` |
| `.css` | `text/css; charset=utf-8` |
| `.html`, `.htm` | `text/html; charset=utf-8` |
| `.xml` | `application/xml; charset=utf-8` |
| `.txt`, `.md` | `text/plain; charset=utf-8` |
| `.svg` | `image/svg+xml` |
| `.png` | `image/png` |
| `.jpg`, `.jpeg` | `image/jpeg` |
| `.gif` | `image/gif` |
| `.webp` | `image/webp` |
| `.woff` | `font/woff` |
| `.woff2` | `font/woff2` |
| `.ttf` | `font/ttf` |
| `.eot` | `application/vnd.ms-fontobject` |
| `.ts` | `text/typescript; charset=utf-8` |
| `.tsx` | `text/typescript; charset=utf-8` |
| `.jsx` | `text/javascript; charset=utf-8` |
| `.map` | `application/json; charset=utf-8` |
| `.wasm` | `application/wasm` |
| 其他 | `application/octet-stream` |

---

## 错误响应

### 404 Not Found

当请求的包、版本或文件不存在时返回。

```json
Package not found
```

### 400 Bad Request

当请求格式不正确时返回。

```json
Invalid Request: ...
```

### 500 Internal Server Error

当服务器内部错误时返回。

```json
Internal Error: ...
```

---

## 缓存

### 元信息缓存
- **时长**: 5 分钟
- **容量**: 1000 条
- **策略**: LRU

### 包文件缓存
- **时长**: 1 小时
- **容量**: 500 个包
- **策略**: LRU

缓存会在内存中自动管理，超时或达到容量上限时自动清理。

---

## 限制

1. 最大包文件大小：无限制（受内存限制）
2. 并发请求：取决于系统资源
3. 缓存大小：元信息 1000 条，包文件 500 个

---

## 性能优化建议

1. **使用缓存**: 相同的请求会从缓存中返回，大幅提升响应速度
2. **精确版本**: 使用精确版本号（如 `1.2.3`）比版本范围（如 `^1.0.0`）更快
3. **CDN 分发**: 可以在前面加一层 CDN 进一步提升性能
4. **反向代理**: 使用 Nginx 等反向代理可以提供额外的缓存和压缩

---

## 健康检查

```bash
curl http://localhost:3000/
```

如果服务正常运行，会返回 200 OK 和主页内容。

---

## 配置

通过环境变量配置：

```bash
# npm Registry 地址
export REGISTRY=https://registry.npmjs.org

# 服务端口
export PORT=3000

# 日志级别
export RUST_LOG=byr_jsdelivr=info
```

---

## 完整示例

### 使用 React

```bash
# 获取最新版本
curl http://localhost:3000/react

# 获取指定版本
curl http://localhost:3000/react@18.2.0

# 列出文件
curl http://localhost:3000/react@18.2.0/

# 获取 package.json
curl http://localhost:3000/react@18.2.0/package.json
```

### 使用 Vue

```bash
# 获取最新版本
curl http://localhost:3000/vue

# 使用版本范围
curl http://localhost:3000/vue@^3.0.0

# 获取运行时文件
curl http://localhost:3000/vue@3.3.4/dist/vue.runtime.esm-browser.js

# 列出 dist 目录
curl http://localhost:3000/vue@3.3.4/dist/
```

### 使用 scoped 包

```bash
# 获取 @vue/runtime-core
curl http://localhost:3000/@vue/runtime-core@3.3.4/

# 获取文件
curl http://localhost:3000/@vue/runtime-core@3.3.4/dist/runtime-core.esm-bundler.js
```

### 在 HTML 中使用

```html
<!DOCTYPE html>
<html>
<head>
    <title>Test</title>
</head>
<body>
    <!-- 使用最新版本 -->
    <script src="http://localhost:3000/vue"></script>
    
    <!-- 使用指定版本 -->
    <script src="http://localhost:3000/vue@3.3.4/dist/vue.global.js"></script>
    
    <!-- ESM 方式 -->
    <script type="module">
        import Vue from 'http://localhost:3000/vue@3.3.4/dist/vue.esm-browser.js';
        // ...
    </script>
</body>
</html>
```
