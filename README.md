# JM Boom - Docker 版本

前后端分离的跨平台禁漫天堂第三方客户端。

## 🚀 快速开始

### 使用 Docker（推荐）

```bash
docker-compose up -d
```

访问 http://localhost:3000

### 开发模式

**后端：**
```bash
cd server
cargo run
```

**前端：**
```bash
cd web
npm install
npm run dev
```

## 📁 项目结构

```
jm-boom/
├── server/           # Rust 后端
│   ├── src/
│   │   ├── api/      # HTTP API
│   │   ├── reader/   # 图片预加载
│   │   ├── cache/    # 缓存管理
│   │   └── main.rs
│   └── Cargo.toml
├── web/              # React 前端
│   ├── src/
│   └── package.json
├── Dockerfile
└── docker-compose.yml
```

## ✨ 架构优势

- **前后端分离**：独立开发、独立部署
- **高性能预加载**：并发下载 8 张，并发解码 4 张
- **Docker 一键部署**：开箱即用
- **移动端友好**：响应式设计

## 🎯 开发计划

### Phase 1: 基础框架 ✅
- [x] 项目结构搭建
- [x] 后端骨架（axum）
- [x] 前端 API 客户端
- [x] Docker 配置

### Phase 2: 核心迁移（进行中）
- [ ] JM API 调用逻辑迁移
- [ ] 图片解扰算法迁移
- [ ] 预加载管道实现
- [ ] 前端接口对接

### Phase 3: 功能完善
- [ ] 用户认证
- [ ] 收藏和历史
- [ ] 下载管理
- [ ] 移动端 UI 优化

## 📝 原 Tauri 版本

原桌面版代码保留在 `master` 分支。

## 📄 License

MIT
