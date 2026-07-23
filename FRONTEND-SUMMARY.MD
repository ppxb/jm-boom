# JM Boom 前端代码审计摘要

> 面向后续 AI / 协作者。范围：`web/src/`（约 1.06 万行 TS/TSX，~140 文件，不含 `routeTree.gen.ts`）。
>
> 与全栈摘要 `SUMMARY.MD` 互补：本文件只谈前端，结论可独立落地。**不是**缺陷清单，也不是要求立刻大重构。
>
> 通读重点：`lib/api/*`、`features/reader/*`、`features/downloads/*`、`routes/_app/explore/*`、书架/收藏、详情与设置。

---

## 1. 前端现状（心智模型）

### 技术栈与分层

| 层 | 路径 | 职责 |
|----|------|------|
| 路由 | `routes/` | TanStack Router 文件路由；`_app` 底部导航壳；阅读器独立路由 |
| 特性 | `features/` | 页面业务（阅读器、详情、下载、首页、设置） |
| API | `lib/api/` | 同源 `fetch` 封装与 DTO 映射 |
| 状态 | React Query + Zustand | 服务端状态用 Query；阅读器偏好用 `settings-store` persist |
| UI | `components/` + shadcn | 通用壳（`AppPage`、封面网格、分页）与 UI 原语 |
| 领域类型 | `domain/comic.ts` | 漫画摘要/详情类型（很薄，合理） |

### 体量分布（约）

| 目录 | 文件 | 行数 | 说明 |
|------|------|------|------|
| `features/` | 57 | ~5180 | 主战场；阅读器约占一半 |
| `components/` | 43 | ~2744 | 含 shadcn，业务组件不多 |
| `routes/` | 16 | ~1338 | 探索列表与书架偏厚 |
| `lib/` | 20 | ~1110 | API 客户端与预加载调度质量高 |
| 其余 | — | ~200 | store / domain / 入口 |

最大单文件（业务向）：

- `reader-strip-window.tsx` ~369
- `reader-image.tsx` ~267
- `comic-detail/hero.tsx` ~259
- `reader-page.tsx` ~252
- `explore/search.tsx` ~241
- `download-task-card.tsx` ~235

### 产品与架构约束（改代码时不要破坏）

- 包管理用 **Bun**；不要用 npm/pnpm 改锁。
- 不要手改 `routeTree.gen.ts`。
- 移动端优先；主导航始终底部胶囊。
- 书架 / 探索 / 收藏 / 下载 / 设置用 `AppPage`；详情与阅读器独立布局。
- 访问门禁：`sessionStorage` + 登录页；业务 API 暂不鉴权（见 `AGENTS.md`）。
- 阅读器视口用 `dvh`；切章从第一页按章节 ID 重建会话；条漫滚动容器纳入顶部重置。
- 详情主封面完成后再预热阅读图。

### 整体评价

- **没有明显屎山。** 阅读器热路径（session / preload / strip 虚拟列表 / 尺寸缓存）和 API 层设计清晰。
- 主债集中在：**列表页样板重复、Query 默认选项噪音、身份 map、少量一致性与可取消性缺口**。
- **不要为“看起来更干净”合并阅读器 hooks，或重写 `ReaderPreloadScheduler` / 条漫虚拟列表。**

---

## 2. 做得好的地方（不要过度优化）

1. **`ReaderPreloadScheduler`**（`lib/reader-preload.ts`）：scope 归属、并发泵、abort 无主 job、完成路径上限 1000。
2. **阅读器 hook 按职责拆分**：session / preload / history / keyboard / auto-read / chapter-info / next-chapter-prefetch —— 文件多但边界清楚。
3. **条漫**：`@tanstack/react-virtual` + `loadedImageSizes` 防 reflow；切章过渡与无下一章时去掉底部空白。
4. **`queryKeys` 集中**；收藏/历史对 `comicState` 做局部乐观更新。
5. **下载 mutation** 用服务端返回的完整 `list` 写回 cache，模式统一（`useTaskMutation`）。
6. **`CACHE` / `READER` / `UI` 常量**集中在 `lib/constants.ts`，避免魔法数散落。
7. **Providers 默认 Query 策略**合理：列表长 stale、默认不 focus refetch、可重试错误判断。
8. **设置 store**：persist + partialize + 枚举/范围 clamp，干净。
9. **同源 API 客户端**：错误体解析、`retryable`、204、blob 分流都到位。
10. **测试虽少，但落在易错纯逻辑**：章节 link、条漫窗口、下一章预取阈值 —— 方向对。

---

## 3. 问题清单（按影响）

### 3.1 重复实现（优先收敛）

| ID | 位置 | 问题 | 建议 |
|----|------|------|------|
| F-D2 | `explore/{list,ranking,search,weekly}.tsx` + `home/page.tsx` | loading / error / empty / grid / pagination 模板重复 | 抽 `ComicListQueryView`（或 `QueryState` + 插槽） |
| F-D3 | `explore/ranking.tsx` vs `section-list/section-utils.ts` | `parseRankingCategory/Order` 与 `parseListCategory/Order` 重复 | ranking 直接复用 section-utils |
| F-D4 | `use-settings-data.ts` 等 | mutation `onError: toast.error(error instanceof Error ? ...)` 复制 10+ 次 | 抽 `toastApiError(error, fallback?)` |
| F-D5 | 书架 / 收藏 / 多处 | `EmptyState` 失败重试块几乎一字不差 | 可并入 F-D2，或小工具 `QueryErrorState({ onRetry })` |
| F-D6 | `bookshelf.tsx` / `favorites.tsx` | 清空后 `setQueriesData({ queryKey: ['jm-comic-state'] }, ...)` 硬编码前缀 | `queryKeys` 增加 `comicStateRoot()` / `comicStateAll`，禁止散落字面量 |

### 3.2 代码臃肿 / 噪音

| ID | 位置 | 问题 | 建议 |
|----|------|------|------|
| F-B1 | `lib/api/comic-summary.ts`, `comic.ts` `mapComicDetail` | 身份字段对拷；真正有价值的是评论 `htmlToText` | 列表/详情用类型 alias 或直接返回；仅评论保留 map（同 `SUMMARY` B1） |
| F-B2 | 多处 `useQuery` | 重复写与 `providers.tsx` **完全相同** 的 `staleTime: LIST_*`、`refetchOnMount: false`、`refetchOnWindowFocus: false` | 只在例外处显式写：书架 focus refetch、下载 1s poll、设置 always、评论/详情/阅读器不同 stale |
| F-B3 | `explore/ranking` vs `explore/list?mode=ranking` | 双入口、筛选 UI 不完全共用 | 定权威路径，另一个 redirect 或完全由 list 承载（同 `SUMMARY` C3） |
| F-B4 | `bookshelf` vs `favorites` | 同为“集合页”，loading UI、空态、分页夹页、是否多选不一致 | 共享 `CollectionPageShell`（同 `SUMMARY` C2）；功能差异用 props |
| F-B5 | `use-settings-data.ts` | 5 个 mutation 结构相同，仅 key/message 不同 | 可选：小工厂 `useSettingsMutation(fn, key, message)`；**收益低于 F-D4，可后做** |

### 3.3 效率

| ID | 位置 | 问题 | 建议 |
|----|------|------|------|
| F-P1 | `use-download-tasks.ts` `removeDownloadTasks` | 批量删除串行 N 次 API | 依赖后端 bulk remove；前端一次 mutation（同 `SUMMARY` P2） |
| F-P2 | 下载页活跃任务 | 1s 全量 list 轮询 | 前端侧可先保持；规模变大再 SSE/diff（同 `SUMMARY` P1） |
| F-P3 | `lib/api/client.ts` | 所有请求强制 `Content-Type: application/json` | 仅有 body 的 POST/PUT 设置（同 `SUMMARY` P6） |
| F-P4 | `lib/api/comic.ts` `htmlToText` | 每条评论 `new DOMParser()` | 评论量大时再优化；可复用 parser 或轻量 strip |
| F-P5 | `lib/api/*` + 几乎全部 `queryFn` | **未把 React Query 的 `signal` 传给 `fetch`**；仅预加载支持 AbortSignal | `request` / `apiClient.get|post|...` 支持 `signal`；`queryFn: ({ signal }) => ...`。切换页时避免无用请求占满连接 |
| F-P6 | `reader-image.tsx` `loadedImageSizes` | 全局 `Map` 无上限；长会话跨多本漫画持续增长 | 简单 LRU（例如 200～500 条）或按 `readId` 清理 |
| F-P7 | `preloadComicReadPage` | fetch blob → decode → revoke；随后 `<img src=path>` 再走一遍缓存路径 | **有意设计**（提前解码 + HTTP 缓存命中）；不要为“省一次 fetch”拆掉，除非实测缓存未命中 |
| F-P8 | `use-reader-history-sync` | 每 5s `invalidateQueries(readingHistory)` | 无 active observer 时成本低；若以后有常驻订阅，可改为只 `setQueryData` / 离开时 invalidate |
| F-P9 | `getFilterCounts` | 对 taskList 多次 `filter` | 一次 reduce 即可；任务量小时可忽略 |

### 3.4 清晰度 / 一致性

| ID | 位置 | 问题 | 建议 |
|----|------|------|------|
| F-C1 | `download-task-card.tsx` | 手写 `` `/api/covers/${id}` `` | 统一 `coverPath(id)`（同 `SUMMARY` C4） |
| F-C2 | `lib/api/reader.ts` | 唯一仍手写 snake_case → camelCase 的映射（`chapter_id` / `url`） | 中期：后端统一 camelCase；短期：在文件头注释“唯一 snake_case 适配点” |
| F-C3 | `ComicCard` `linkProps` | `to: string` 等宽松类型，丢失 Router 类型安全 | 可用 TanStack Router 的 `LinkProps` 子集，或保持现状但别再扩大 |
| F-C4 | 探索 `search` | 排序用本地 `Select`，其它页用 `FilterSelect` | 视觉/交互可统一到 `FilterSelect` |
| F-C5 | 收藏页 | 清空时 `setPage(1)`；书架有 `pageCount` clamp effect，收藏没有 | 共享壳时一并统一夹页策略 |
| F-C6 | Query 默认 stale | 全局默认 `LIST_STALE_TIME`（30min）；详情/评论/阅读器靠局部覆盖 | **正确**；清理 F-B2 时不要误删真正不同的 `DETAIL_*` / `READER_*` / `COMMENTS_*` |
| F-C7 | 访问门禁 | 仅前端 `sessionStorage`，业务 API 不鉴权 | 已知产品债；升级 Session/JWT 是独立能力，不要在列表清理里顺手做 |

### 3.5 测试与可维护性

| ID | 位置 | 问题 | 建议 |
|----|------|------|------|
| F-T1 | `web/src` | 仅 3 个测试文件，全在 reader | 优先补**纯函数**：`section-utils` 解析、`filters`、`client` 错误解析、history page clamp |
| F-T2 | 阅读器 | strip / session 集成测成本高 | 保持现状；只锁纯逻辑与边界（已有方向正确） |
| F-T3 | lint | `oxlint`，无组件测试框架脚本 | 若补测，沿用现有 `*.test.ts` 跑法（Bun），不要引入第二套 runner 除非必要 |

---

## 4. 关键文件索引

### 热路径（改前先读）

- `features/reader/reader-page.tsx` — 阅读器组装
- `features/reader/use-reader-session.ts` — 页码/窗口/切章 scope
- `features/reader/reader-strip-window.tsx` — 条漫虚拟列表
- `features/reader/reader-image.tsx` — 单页/双页展示与尺寸缓存
- `lib/reader-preload.ts` — 预加载调度
- `lib/api/reader.ts` — manifest + preload fetch
- `features/reader/use-reader-history-sync.ts` — 节流写历史

### 列表与集合

- `routes/_app/explore/{list,ranking,search,weekly}.tsx`
- `features/section-list/section-utils.ts`, `section-filters.tsx`
- `routes/_app/bookshelf.tsx`, `favorites.tsx`
- `features/history/use-history-selection.ts`
- `features/downloads/*`

### 横切

- `lib/api/client.ts` — 同源客户端
- `lib/query-keys.ts` — Query keys
- `components/providers.tsx` — Query 默认
- `stores/settings-store.ts` — 阅读偏好
- `lib/access-gate.ts` + `routes/login.tsx` — 访问门禁

---

## 5. 推荐落地顺序（仅前端）

### 第一批：低风险、高收益（约 1 天）

1. **F-B2** 去掉与 Query 默认重复的选项（保留真正例外）
2. **F-B1** 去掉身份 map（保留评论 html 清洗）
3. **F-D3** ranking 解析复用 section-utils
4. **F-C1** `coverPath` + 替换手写封面 URL
5. **F-D6** `queryKeys` 补 comicState 前缀 helper，去掉字面量
6. **F-D4** `toastApiError` 收敛 mutation 错误提示

**约束：** 不改阅读器语义、不改预加载策略、不改下载状态展示逻辑；对外可观察行为不变。

### 第二批：可感结构 / 性能（按需）

1. **F-D2 + F-D5** 探索列表 / 查询态壳
2. **F-B4** 书架/收藏 `CollectionPageShell`
3. **F-P5** API 客户端透传 `AbortSignal`
4. **F-P6** `loadedImageSizes` LRU
5. **F-P1** 下载 bulk remove（需后端配合）

### 第三批：有空再做

1. **F-B3** ranking / list 路径收敛
2. **F-P3** Content-Type
3. **F-C2** reader API 字段命名与后端对齐
4. **F-T1** 纯函数测试补齐
5. **F-C4** 搜索排序控件统一

---

## 6. 明确不要做的事

- 不要把阅读器 hooks 合并成单文件“上帝 hook”。
- 不要重写 `ReaderPreloadScheduler` 或条漫 virtualizer，除非有明确 bug / 性能证据。
- 不要为预加载再加一层抽象（scheduler 已经够用）。
- 不要大改 `components/ui/*`（shadcn 生成物）。
- 不要用 npm/pnpm 改 Bun 锁；不要手改 `routeTree.gen.ts`。
- 不要在本轮“清理样板”时顺手做访问鉴权升级。
- 不要加过多兜底代码（见 `AGENTS.md`）。

---

## 7. 与全栈 `SUMMARY.MD` 的映射

| 前端本文件 | 全栈 SUMMARY |
|------------|--------------|
| F-D2 | D3 |
| F-D3 | D4 |
| F-B1 | B1 |
| F-B2 | B2 |
| F-B3 | C3 |
| F-B4 | C2 |
| F-P1 | P2 |
| F-P2 | P1（前端侧） |
| F-P3 | P6 |
| F-P4 | P5 |
| F-C1 | C4 |
| F-P5 / F-P6 / F-D4 / F-D6 / F-T* | **本轮新增**，全栈摘要未展开 |

若两边冲突，以**更新时间更近**的条目为准；修复后请两边同步标 `[done]` 或删除，避免摘要腐烂。

---

## 8. 给后续 AI 的工作指令模板

若被要求“按 FRONTEND-SUMMARY 清理前端”，默认范围：

1. **只做第一批**，除非用户明确要求第二/三批。
2. 行为兼容；验证：
   - `bun x tsc --noEmit`
   - `bun run lint`
   - 若动到纯函数，补或跑相关 `*.test.ts`
3. 中文文件读写用 UTF-8。
4. 不要顺手改无关格式/全库重命名。
5. 修完后把本文件对应条目标 `[done]` 或删除。

---

## 9. 一句话结论

前端主债是**列表/Query 样板与身份 map**，不是阅读器架构。热路径已经健康；优先做 **Query 去噪、去身份 map、ranking 解析复用、coverPath、toast/queryKey 小工具**，再考虑列表壳、AbortSignal 与尺寸缓存 LRU。
