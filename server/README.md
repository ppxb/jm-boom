# JM Boom Server

Rust 后端服务，提供 REST API 和图片预加载功能。

## 开发

```bash
# 运行
cargo run

# 监听 http://localhost:3000
```

## 数据目录

```
data/
├── app.db              # SQLite 数据库
└── cache/
    └── images/         # 图片缓存
        └── {chapter_id}/
            ├── 1.webp
            ├── 2.webp
            └── ...
```

## API 端点

- `GET /health` - 健康检查
- `GET /api/search` - 搜索
- `GET /api/comics/:id` - 漫画详情
- `GET /api/reader/:chapter_id/manifest` - 章节清单
- `GET /api/reader/:chapter_id/pages/:page` - 获取图片
- `POST /api/auth/login` - 登录

## 迁移任务

- [ ] 从 src-tauri/src/api 迁移 JM API 调用逻辑
- [ ] 从 src-tauri/src/reader 迁移图片解扰算法
- [ ] 完善预加载管道
- [ ] 实现用户认证和会话管理
