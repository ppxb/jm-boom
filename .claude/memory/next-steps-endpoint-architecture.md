---
name: next-steps-endpoint-architecture
description: 下一步工作：实现 JM API 多端点架构和核心逻辑迁移
metadata:
  type: project
---

## 下一步核心工作

### 当前状态
- ✅ 项目重构完成（前后端分离）
- ✅ 在 `feat/docker` 分支
- ✅ 后端骨架就绪（Rust + axum）
- ✅ 前端迁移完成（web/）
- ✅ Docker 配置完成
- ✅ 多端点架构设计完成（见 `docs/endpoint-architecture.md`）

### 核心工作优先级

#### Phase 1: JM API 端点管理（高优先级）

**目标**：实现智能端点发现和管理

1. **端点发现模块** (`server/src/jm/discovery.rs`)
   - 从远程配置获取端点列表（AES 解密）
   - 配置 URL 和解密密钥已知
   - 参考：`archive-src-tauri/src/api/setting.rs:164-254`

2. **端点管理器** (`server/src/jm/endpoint_manager.rs`)
   - 自动探测所有端点延迟
   - 智能选择最优端点
   - 后台健康检查（30 分钟间隔）
   - 故障自动切换

3. **暴露端点管理 API** (`server/src/api/endpoints.rs`)
   - `GET /api/endpoints` - 查询端点列表和状态
   - `POST /api/endpoints/probe` - 主动触发探测

#### Phase 2: JM API 客户端（高优先级）

**目标**：实现与 JM 服务的通信

1. **HTTP 客户端** (`server/src/jm/client.rs`)
   - 基于 reqwest
   - 统一请求头（Token、Tokenparam）
   - 代理支持
   - 参考：`archive-src-tauri/src/api/client.rs`

2. **认证模块** (`server/src/jm/auth.rs`)
   - SettingAuth 结构
   - Token 生成逻辑
   - 参考：`archive-src-tauri/src/api/auth.rs`

3. **数据模型** (`server/src/jm/models.rs`)
   - API 响应结构
   - 漫画、章节、搜索等模型
   - 参考：`archive-src-tauri/src/api/models.rs`

#### Phase 3: 图片处理（高优先级）

**目标**：实现图片解扰和缓存

1. **图片解扰算法** (`server/src/reader/decoder.rs`)
   - 从 Tauri 版本迁移
   - 参考：`archive-src-tauri/src/reader/image_decode.rs`

2. **章节清单处理** (`server/src/reader/manifest.rs`)
   - 解析章节页面信息
   - 参考：`archive-src-tauri/src/reader/manifest.rs`

3. **完善缓存模块** (`server/src/cache/mod.rs`)
   - 图片缓存已有骨架
   - 集成解扰算法
   - WebP 转换

#### Phase 4: API 实现（中优先级）

**目标**：实现完整的业务 API

1. **搜索 API** (`server/src/api/search.rs`)
   - 已有骨架，需对接 JM 客户端

2. **漫画详情 API** (`server/src/api/comics.rs`)
   - 获取漫画信息、章节列表

3. **阅读器 API** (`server/src/api/reader.rs`)
   - 章节清单
   - 图片代理（含解扰）

#### Phase 5: 前端对接（中优先级）

**目标**：前端适配新架构

1. **API 客户端适配** (`web/src/lib/api/`)
   - 从 Tauri IPC 改为 HTTP
   - 已有代码，需调整 endpoint

2. **端点选择 UI** (`web/src/features/settings/`)
   - 显示端点列表和延迟
   - 自动/手动切换

### 推荐开始顺序

1. **端点发现** → 解决多端点问题（基础设施）
2. **JM 客户端** → 能调用 JM API
3. **图片解扰** → 核心功能
4. **API 实现** → 对外服务
5. **前端对接** → 完整流程

### 技术债务

- 修复 Rust 警告（未使用的变量和字段）
- 前端构建优化（bundle 过大警告）
- 补充单元测试

**Why**: 多端点是 JM 服务的基础设施，必须先解决。然后是客户端和核心解扰算法，最后才能打通端到端流程。

**How to apply**: 按上述 Phase 1-5 顺序实施，每个 Phase 完成后测试验证再进入下一个。
