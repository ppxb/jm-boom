# 快速开始

## 当前状态

✅ 项目骨架已搭建完成  
✅ 前端代码已迁移到 `web/`  
✅ 后端 API 框架已就绪  
⏳ 核心逻辑待迁移（从 `archive-src-tauri/`）

## 立即测试

### 1. 测试后端能否启动

```bash
cd server
cargo run
```

预期输出：
```
JM Boom Server starting...
Server listening on 0.0.0.0:3000
```

访问 http://localhost:3000/health 应该返回 `OK`

### 2. 测试前端

```bash
cd web
bun install  # 或 npm install
bun run dev
```

前端会在 http://localhost:5173 启动

## 下一步：开始迁移

### 优先级 1：JM API 客户端

从 `archive-src-tauri/src/api/client.rs` 迁移到 `server/src/api/jm_client.rs`

这个文件包含：
- HTTP 请求封装
- 请求签名
- 错误处理

### 优先级 2：图片解扰算法

从 `archive-src-tauri/src/reader/image_decode.rs` 迁移到 `server/src/reader/decoder.rs`

这是性能核心，需要完整实现：
- 图片分块重组
- WebP 编码

### 优先级 3：章节清单

从 `archive-src-tauri/src/reader/manifest.rs` 迁移章节信息获取逻辑

## 开发建议

1. **保留 archive-src-tauri/** - 频繁参考原代码
2. **先跑通一个完整流程** - 搜索 → 详情 → 阅读
3. **逐步替换前端调用** - 一个页面一个页面改造

## 文档

- `MIGRATION.md` - 详细的迁移说明
- `server/README.md` - 后端开发文档
- `README.md` - 项目总览
