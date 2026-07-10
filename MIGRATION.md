# 项目重构说明

## 当前分支：feat/docker

这个分支将 JM Boom 从 Tauri 桌面应用重构为前后端分离的 Web 应用。

## 项目结构

```
jm-boom/
├── server/                # Rust 后端
│   ├── src/
│   │   ├── api/          # HTTP API 路由
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── comics.rs
│   │   │   ├── reader.rs
│   │   │   └── search.rs
│   │   ├── reader/       # 图片处理和预加载
│   │   │   ├── mod.rs
│   │   │   └── preloader.rs
│   │   ├── cache/        # 图片缓存管理
│   │   │   └── mod.rs
│   │   ├── storage/      # 数据库
│   │   └── main.rs       # 主入口
│   ├── migrations/       # 数据库迁移
│   │   └── 001_init.sql
│   └── Cargo.toml
│
├── web/                   # React 前端（从根目录迁移）
│   ├── src/              # 前端源码（复用现有）
│   │   └── api/
│   │       └── client.ts # HTTP API 客户端
│   ├── public/
│   ├── package.json
│   ├── vite.config.ts
│   └── tsconfig.json
│
├── archive-src-tauri/     # 原 Tauri 代码（供迁移参考）
│
├── Dockerfile             # 多阶段构建
├── docker-compose.yml     # 一键部署
└── README.md              # 更新的文档

```

## 已完成

✅ **分支创建**
- 创建 `feat/docker` 分支

✅ **后端骨架**
- axum 服务器框架
- API 路由结构
- 图片缓存系统（支持 LRU）
- 智能预加载管道（并发下载 8 张，并发解码 4 张）
- SQLite 数据库集成

✅ **前端适配**
- 创建 HTTP API 客户端
- 环境变量配置（开发/生产）
- 代码从根目录迁移到 web/

✅ **Docker 配置**
- 多阶段构建 Dockerfile
- docker-compose 配置

✅ **项目清理**
- 移除根目录的前端配置文件
- 原 Tauri 代码归档到 `archive-src-tauri/`
- 更新 .gitignore

## 待完成（迁移任务）

### Phase 1: 核心逻辑迁移
- [ ] **JM API 调用**（从 `archive-src-tauri/src/api/`）
  - client.rs → server/src/api/jm_client.rs
  - auth.rs → 认证逻辑
  - codec.rs → 加密解密

- [ ] **图片处理**（从 `archive-src-tauri/src/reader/`）
  - image_decode.rs → server/src/reader/decoder.rs
  - manifest.rs → 章节清单处理
  - 实现完整的预加载管道

- [ ] **数据库**（从 `archive-src-tauri/src/storage/`）
  - 用户、收藏、历史表结构
  - 迁移脚本

### Phase 2: 前端对接
- [ ] 改造现有 hooks，使用 HTTP API
- [ ] 图片加载改为 HTTP URL
- [ ] 阅读器预加载优化

### Phase 3: 功能完善
- [ ] 用户认证和会话管理
- [ ] 下载管理
- [ ] WebSocket 实时推送
- [ ] 移动端 UI 优化

## 如何开始开发

### 后端

```bash
cd server
cargo run
```

服务器启动在 `http://localhost:3000`

### 前端

```bash
cd web
npm install  # 或 bun install
npm run dev
```

开发服务器在 `http://localhost:5173`

### Docker

```bash
docker-compose up --build
```

## 性能提升

相比 Tauri 版本：

| 项目 | Tauri 版本 | Docker 版本 |
|------|-----------|------------|
| 并发下载 | 1-2 张 | 8 张 |
| 并发解码 | 1 张 | 4 张 |
| 预加载 | 前端手动 | 后端自动 |
| 部署方式 | 桌面安装包 | Docker 一键部署 |
| 移动端支持 | ❌ | ✅（响应式） |

## 注意事项

- 原 Tauri 代码保留在 `archive-src-tauri/` 供迁移参考
- **不要删除 `archive-src-tauri/`**，迁移完成前需要频繁查看
- 前端代码已经复制到 `web/`，可以直接改造

## 下一步

1. 从 `archive-src-tauri/src/api/client.rs` 开始迁移 JM API 调用逻辑
2. 测试后端能否成功调用 JM API
3. 迁移图片解扰算法
4. 实现完整的预加载管道
