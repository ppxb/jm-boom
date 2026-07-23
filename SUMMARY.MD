# JM Boom 代码审计摘要

> 面向后续 AI / 协作者的可读摘要。基于 `feat/docker` 分支通读：`server/`（约 1 万行 Rust）与 `web/src/`（约 1 万行 TS/TSX）。
>
> 目标：指出重复、臃肿、低效、不清晰处及优化顺序。**不是**缺陷清单，也不是要求立刻大重构。
>
> **前端专审**（更深、可独立落地）：见 [`FRONTEND-SUMMARY.MD`](./FRONTEND-SUMMARY.MD)。

---

## 1. 项目现状（先建立心智模型）

### 架构

- 已放弃 Tauri；结构为 `web/`（React）+ `server/`（Rust/Axum）。
- 前端：React 19、Vite、TanStack Router/Query、Zustand、shadcn/ui；包管理用 **Bun**（不要用 npm/pnpm 改锁）。
- 后端：匿名 JM API 代理、端点探测/故障切换、阅读图片解扰、缓存、下载队列、收藏、阅读历史。
- Docker 单容器同源部署：Axum 托管 `web/dist`，数据在 `/app/data`。

### 关键约束（改代码时不要破坏）

- 下载：有界队列（最多 2 个漫画任务并发，每任务最多 5 页 job）；Manager 管状态与 worker；暂停/取消/删除用通知退出；恢复/删除用 **generation** 隔离旧 worker。
- 下载文件：`data/downloads/<task_id>/`；章节 manifest 在 SQLite；上游不可用时阅读器回退离线 manifest/文件。
- 阅读：仅需解扰时再编 WebP；JPEG/PNG/WebP/GIF 可原样缓存；URL 含缓存算法版本。
- 阅读与下载共用 **PageMaterializer**（single-flight、优先级、解扰、缓存写入）。
- 封面：同源 `/api/covers/:id`；详情主封面完成后再预热阅读图。
- 默认无 CORS；确需跨域用 `JM_BOOM_CORS_ORIGINS`。
- 不要手动改 `web/src/routeTree.gen.ts`。
- 产品：移动端优先；主导航始终底部胶囊；书架/探索/收藏/下载/设置用 `AppPage`。

### 整体评价

- **没有明显屎山**。核心热路径（阅读物化、下载状态机、预加载 scheduler、endpoint failover）质量较高。
- 主要债务：横切样板重复、少量列表/下载路径上的全量拷贝、部分身份映射与默认配置噪音。
- **不要为“看起来更干净”拆碎阅读器 hook 或重写下载状态机。**

---

## 2. 做得好的地方（不要过度优化）

1. 阅读与下载共用 `PageMaterializer` + `KeyedLock` single-flight + `ImageWorkBudget` 优先级。
2. 下载 generation 隔离；暂停/取消通过 watch 退出。
3. 无需解扰时原样缓存（不盲目转 WebP）。
4. 前端 `ReaderPreloadScheduler`：scope 归属、并发泵、完成路径上限。
5. `queryKeys` 集中；收藏/历史对 `comicState` 乐观更新。
6. 条漫虚拟列表（`@tanstack/react-virtual`）+ 图片尺寸缓存防 reflow。
7. endpoint 探测与业务请求解耦；阅读器 hook 按职责拆分（session / preload / history / keyboard / auto-read）——文件多但清晰，**不要合并成巨型 hook**。

---

## 3. 问题清单（按影响）

### 3.1 重复实现（优先收敛）

| ID | 位置 | 问题 | 建议 |
|----|------|------|------|
| D1 | `server/src/api/favorites.rs`, `history.rs`, `comics.rs`, `covers.rs`, `settings.rs` | `validate_comic_id` / `validate_page` / `internal_error` 复制粘贴 | 抽 `api/validation.rs`：`require_numeric_id`、`require_page`、`storage_error` |
| D3 | `web/src/routes/_app/explore/{list,ranking,search,weekly}.tsx` | loading / error / empty / grid / pagination 模板重复 | 抽 `ComicListQueryView` 或等价组件 |
| D4 | `explore/ranking.tsx` vs `features/section-list/section-utils.ts` | `parseRankingCategory/Order` 与 `parseListCategory/Order` 重复 | ranking 复用 section-utils |
| D5 | `page_materializer.rs`（图片下载）, `application/cover.rs` | invalidate img host → 重拉 host → 重试 重复 | 抽 `download_with_img_host_retry` |

### 3.2 代码臃肿 / 噪音

| ID | 位置 | 问题 | 建议 |
|----|------|------|------|
| B1 | `web/src/lib/api/comic-summary.ts`, `comic.ts` | `mapComicSummary` / 大部分 `mapComicDetail` 是身份拷贝；真正有价值的是评论 `htmlToText` | 列表/详情用类型 alias 或直接返回；仅评论保留 map |
| B2 | 多处 `useQuery` | 重复写已在 `providers.tsx` 的默认：`staleTime`、`refetchOnMount: false`、`refetchOnWindowFocus: false` | 只在例外（书架 focus refetch、下载 1s poll、设置 always）显式写 |
| B3 | `page_materializer.rs` (~721), `download/manager.rs` (~720) | 业务 + 大段 `#[cfg(test)]` 混在同一文件 | 测试外移到 `*_tests.rs`；**业务不必再拆碎** |
| B4 | `api/comics.rs` 等 | Domain → API DTO 字段对拷过厚；`FavoriteResponse` 与 `ComicSummaryResponse` 高度重叠 | 能直接 serde domain 就直出；只在 `cover_url` 等改写处 map |

### 3.3 效率

| ID | 位置 | 问题 | 建议 |
|----|------|------|------|
| P1 | `download/manager.rs` `list()` | 每次 `values().cloned()` 全量 clone；pause/resume/cancel/remove/enqueue 都以 `list()` 结尾；前端活跃任务 **1s 轮询** | 短期：`Arc` 或精简 DTO；中期：mutation 返回 diff；长期 SSE（规模小时可缓） |
| P2 | `web/.../use-download-tasks.ts` `removeDownloadTasks` | 批量删除串行 N 次 API，每次还停 worker + 删目录 + list | 后端 `POST /downloads/remove` body `taskIds[]` |
| P3 | `application/history.rs` `remove_many` | 循环单条 `DELETE` | `WHERE comic_id IN (...)` 一次删 |
| P4 | `favorite.rs` / `history.rs` | 巨型元组 `query_as`，未用 `FromRow` | `#[derive(sqlx::FromRow)]` |
| P5 | `web/src/lib/api/comic.ts` `htmlToText` | 每条评论 `new DOMParser()` | 评论量大时再优化；可 strip tag 或复用 parser |
| P6 | `web/src/lib/api/client.ts` | 所有请求（含 GET）强制 `Content-Type: application/json` | 仅 POST/PUT 且有 body 时设置 |

### 3.4 清晰度 / 一致性

| ID | 位置 | 问题 | 建议 |
|----|------|------|------|
| C1 | `HttpError` / reader `ApiError` / `DownloadError` / `CoverError` / `JmError` | 多层错误，最终都转 HTTP；转换样板重复 | 统一 `From<Xxx> for HttpError`，handler 直接 `?` |
| C2 | `bookshelf.tsx` vs `favorites.tsx` | loading UI、空态、分页夹页策略不一致；书架有多选删、收藏仅清空 | 共享 `CollectionPageShell` |
| C3 | `explore/ranking` vs `explore/list?mode=ranking` | 双入口、筛选组件不完全共用 | 定一个权威路径，另一个 redirect 或完全由 list 承载 |
| C4 | `download-task-card.tsx` | 手写 `` `/api/covers/${id}` `` | 前端统一 `coverPath(id)` |

---

## 4. 关键文件索引（方便定位）

### 后端（偏大 / 核心）

- `server/src/page_materializer.rs` — 页面物化（阅读+下载共享）
- `server/src/download/manager.rs` — 下载任务状态机入口
- `server/src/download/worker.rs` — 任务执行与分页 job
- `server/src/image_work.rs` — 网络/解码优先级预算
- `server/src/endpoint/` — 探测、failover、选择
- `server/src/application/favorite.rs`, `history.rs` — SQLite 持久化
- `server/src/api/favorites.rs`, `history.rs`, `comics.rs`, `reader.rs` — HTTP 层

### 前端（偏大 / 核心）

- `web/src/features/reader/` — 阅读器（page、strip、session、preload、history）
- `web/src/lib/reader-preload.ts` — 预加载调度
- `web/src/lib/api/client.ts` — 同源 API 客户端
- `web/src/lib/query-keys.ts` — React Query keys
- `web/src/routes/_app/explore/*` — 探索列表页
- `web/src/features/downloads/` — 下载页与轮询
- `web/src/stores/settings-store.ts` — 阅读器设置（Zustand persist）

---

## 5. 推荐落地顺序

### 第一批：低风险、高收益（约 1–2 天）

1. D1 抽取 API 校验与 storage error helper
2. B1 去掉身份 map
3. B2 去掉与 Query 默认重复的选项
4. D3 + D4 探索列表壳与 ranking 解析复用
5. P3 历史批量 DELETE 单 SQL

**约束：** 不改阅读/下载状态机语义；行为对外可观察结果应不变。

### 第二批：可感性能（按需）

1. P2 下载 bulk remove API + 前端批量删  
2. P1 减少下载 list 全量 clone  
3. D5 封面/阅读 img-host 重试公共函数  
4. P4 SQLite `FromRow`  

### 第三批：结构整理（有空再做）

1. B3 测试文件外移  
2. C3 ranking/list 路径收敛  
3. C2 书架/收藏 page shell  
4. C1 错误 `From` 统一  
5. P6 client Content-Type  

---

## 6. 明确不要做的事

- 不要重写下载 Manager / generation 模型，除非有明确 bug。  
- 不要把阅读器 hooks 合并成单文件“上帝 hook”。  
- 不要为预加载、物化、缓存再加一层无必要抽象。  
- 不要用 npm/pnpm 改写 Bun 锁；不要手改 `routeTree.gen.ts`。  
- 不要加过多兜底代码；保持简洁（见 `AGENTS.md`）。  
- 业务 API 目前基本不鉴权（仅轻量访问门禁）；升级 Session/JWT 是独立后续能力。  

---

## 7. 给后续 AI 的工作指令模板

若被要求“按 SUMMARY 清理代码”，默认范围：

1. **只做第一批**，除非用户明确要求第二/三批。  
2. 每项改动保持行为兼容；补或跑现有测试：  
   - 前端：`bun x tsc --noEmit`、`bun run lint`  
   - 后端：`cargo test --locked`、`cargo fmt --check`、`cargo clippy --locked -- -D warnings`  
3. 中文文件读写用 UTF-8。  
4. 不要顺手改无关格式/重命名全库。  
5. 修复后若某条已解决，可从本文件删除或标注 `[done]`，避免摘要腐烂。  

---

## 8. 一句话结论

横切样板重复 + 下载/列表全量路径噪音是主债；核心热路径已经健康。优先做**校验/列表壳/去身份 map/批量 DELETE**，再考虑下载 bulk API 与 list clone 优化。
