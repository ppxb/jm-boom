---
name: next-steps-core-migration
description: 下一步核心工作：JM API 客户端、图片解扰和 API 实现的迁移优先级
metadata:
  type: project
---

## 下一步核心工作（修正版）

### 当前状态
- ✅ 项目重构完成（前后端分离）
- ✅ 在 `feat/docker` 分支
- ✅ 后端骨架就绪（Rust + axum）
- ✅ 前端迁移完成（web/）
- ✅ **端点选择 UI 已存在**（`web/src/features/settings/api-endpoint-section.tsx`）
- ✅ **前端已有端点探测逻辑**（调用 Tauri IPC，需改为 HTTP）
- 🔄 端点管理有基础实现，但需要后端支持

### 核心工作优先级（重新排序）

#### Phase 1: JM API 客户端（最高优先级）⭐⭐⭐

**目标**：能够调用 JM API，获取数据

**文件迁移：**
1. **认证模块** (`server/src/jm/auth.rs`)
   - 从 `archive-src-tauri/src/api/auth.rs` 迁移
   - SettingAuth 结构
   - Token 生成逻辑

2. **HTTP 客户端** (`server/src/jm/client.rs`)
   - 从 `archive-src-tauri/src/api/client.rs` 迁移
   - 统一请求头（Token、Tokenparam、User-Agent）
   - 代理支持
   - JWT token 管理

3. **数据模型** (`server/src/jm/models.rs`)
   - 从 `archive-src-tauri/src/api/models.rs` 迁移
   - ApiResponse、SearchPayload、ComicDetail 等

4. **错误处理** (`server/src/jm/error.rs`)
   - 从 `archive-src-tauri/src/api/error.rs` 迁移

**验证点：** 能调用 JM API 并解析响应

#### Phase 2: 图片解扰算法（最高优先级）⭐⭐⭐

**目标**：实现图片解扰核心算法

**文件迁移：**
1. **图片解扰** (`server/src/reader/decoder.rs`)
   - 从 `archive-src-tauri/src/reader/image_decode.rs` 迁移（111 行，已读取）
   - 核心算法：`reorder_scrambled_rgb_rows`
   - 分段计算：`segmentation_count`
   - WebP 编码

2. **阅读器类型** (`server/src/reader/types.rs`)
   - 从 `archive-src-tauri/src/reader/types.rs` 迁移
   - ReaderManifest、ReaderPage 等

3. **章节清单** (`server/src/reader/manifest.rs`)
   - 从 `archive-src-tauri/src/reader/manifest.rs` 迁移

**验证点：** 能解扰图片并缓存

#### Phase 3: 核心 API 实现（高优先级）⭐⭐

**目标**：实现完整的业务 API

**API 模块：**
1. **搜索 API** (`server/src/api/search.rs`)
   - 从 `archive-src-tauri/src/api/search.rs` 迁移
   - 对接 JM 客户端

2. **漫画 API** (`server/src/api/comics.rs`)
   - 从 `archive-src-tauri/src/api/comic.rs` 迁移
   - 漫画详情、章节列表

3. **首页 API** (`server/src/api/home.rs`)
   - 从 `archive-src-tauri/src/api/home.rs` 迁移
   - 首页 feed、分类列表

4. **阅读器 API** (`server/src/api/reader.rs`)
   - 完善现有骨架
   - 章节清单、图片代理（含解扰）

5. **用户 API** (`server/src/api/user.rs`)
   - 从 `archive-src-tauri/src/api/user.rs` 迁移
   - 登录、收藏、历史

**验证点：** 端到端流程跑通

#### Phase 4: 端点管理完善（中优先级）⭐

**目标**：后端支持端点探测和管理

**实现：**
1. **端点发现** (`server/src/jm/discovery.rs`)
   - 从 `archive-src-tauri/src/api/setting.rs:164-254` 迁移
   - 远程配置获取（AES 解密）
   - 端点探测

2. **端点管理 API** (`server/src/api/endpoints.rs`)
   - `GET /api/endpoints` - 查询端点列表和状态
   - `POST /api/endpoints/probe` - 主动触发探测

3. **前端对接** (`web/src/lib/api/setting.ts`)
   - 从 Tauri IPC 改为 HTTP 调用

**验证点：** 前端能看到端点列表并测速

#### Phase 5: 前端 API 客户端适配（中优先级）⭐

**目标**：前端从 Tauri IPC 切换到 HTTP

**文件修改：**
- `web/src/lib/api/*.ts` - 所有 API 文件
- 从 `tauriInvoke()` 改为 `fetch()` 或 `axios`
- 适配新的 HTTP 接口

**验证点：** 前端完整功能可用

### 迁移文件清单

**需要迁移的核心文件（35 个 Rust 文件）：**

```
archive-src-tauri/src/
├── api/
│   ├── auth.rs          → server/src/jm/auth.rs
│   ├── client.rs        → server/src/jm/client.rs
│   ├── codec.rs         → server/src/jm/codec.rs
│   ├── comic.rs         → server/src/api/comics.rs
│   ├── error.rs         → server/src/jm/error.rs
│   ├── home.rs          → server/src/api/home.rs
│   ├── models.rs        → server/src/jm/models.rs
│   ├── search.rs        → server/src/api/search.rs
│   ├── setting.rs       → server/src/jm/discovery.rs (部分)
│   └── user.rs          → server/src/api/user.rs
├── reader/
│   ├── image_decode.rs  → server/src/reader/decoder.rs ✅ 已读取
│   ├── manifest.rs      → server/src/reader/manifest.rs
│   ├── types.rs         → server/src/reader/types.rs
│   └── ...
└── ...
```

### 推荐实施顺序

**第一天：** Phase 1（JM API 客户端）
1. 迁移 auth.rs、error.rs
2. 迁移 client.rs
3. 迁移 models.rs（核心模型）
4. 测试：能调用 JM API 获取数据

**第二天：** Phase 2（图片解扰）
1. 迁移 image_decode.rs → decoder.rs
2. 迁移 types.rs、manifest.rs
3. 集成到缓存模块
4. 测试：能解扰图片

**第三天：** Phase 3（API 实现）
1. 实现搜索、漫画详情 API
2. 完善阅读器 API（含图片代理）
3. 测试：后端 API 可用

**第四天：** Phase 4 + 5（端点管理 + 前端对接）
1. 端点发现和探测
2. 前端 API 客户端改造
3. 端到端测试

### 关键依赖

- `image` crate - 图片处理
- `webp` crate - WebP 编码
- `md5` crate - 解扰算法
- `aes` + `ecb` crate - 端点配置解密

**Why**: 端点选择 UI 已存在，端点管理可以后做。当务之急是让核心功能（API 客户端 + 图片解扰）先跑起来。

**How to apply**: 
1. 先实现 Phase 1（JM 客户端），让后端能调用 JM API
2. 再实现 Phase 2（图片解扰），这是核心价值
3. Phase 3 打通端到端流程
4. Phase 4/5 是优化和完善
