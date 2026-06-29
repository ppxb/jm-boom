# JM Boom Project Notes

本文件给后续接手的 Codex 使用，目标是快速理解项目约束、当前架构。内容应保持短而准，避免记录已经完成的过程流水账。

## 接手约定

- 项目是 Tauri 2 + React 19 + Vite + TanStack Router/Query + Zustand + shadcn/ui + Rust reqwest。
- 当前优先桌面端体验，小屏适配不是近期重点。
- 不要手动编辑 `src/routeTree.gen.ts`，路由树由 TanStack Router 插件、HMR 或 build 生成。
- 读取或写入包含中文的文件时必须显式使用 UTF-8，避免 Windows 终端默认编码导致乱码。
- 可以并行读取文件或搜索；文件编辑使用单次 `apply_patch`，不要重复或并行套同一补丁。
- `cargo check`、`vite build`、`tsc` 等会占用输出目录或 build lock 的验证命令不要并行重复触发。
- 前端和 Rust 通信用 Tauri `invoke`，项目不使用 axios。
- UI 优先复用 shadcn/ui 已安装组件；需要新的 shadcn 组件时，告诉用户安装，不要自己重造复杂基础组件。
- 项目已有 `floating-nav`，不要新增传统 sidebar。
- Rust 依赖可直接添加并用 `cargo check` 下载；前端依赖不要直接安装，先告诉用户。
- 用户此前表示暂时不要纠结 `lint`；不要因为 `lint` 结果扩大无关改动。
- 每次大的变动后，同步用户相应的 `commit msg`。

## 验证命令

- 常规前端验证优先跑 `bun run build`。
- 如果 Windows 环境没有 `bun`，跑：
  - `node_modules/.bin/tsc.exe --noEmit`
  - `node_modules/.bin/vite.exe build`
- 改 `src-tauri` Rust 代码后跑：
  - 工作目录 `src-tauri`
  - `cargo check`
- 只有在新增或调整测试时再跑相关 `cargo test`。

## 发布流程

- 发布前必须同步修改三处版本号：`package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`。
- 改完 Rust crate 版本后运行 `cargo check`，更新并验证 `src-tauri/Cargo.lock` 中的 `jm-boom` 版本。
- 每个发布 tag 必须有对应发布日志：`docs/release-notes/<tag>.md`，例如 `docs/release-notes/v0.2.2.md`。
- 发布日志按上一个 tag 到本次发布的 commit 生成，建议先看提交范围：
  - `git describe --tags --abbrev=0`
  - `git log <previous-tag>..HEAD --oneline`
- GitHub Actions 会校验发布日志包含以下四个板块，为空则不保留标题：
  - `## 功能`
  - `## 优化`
  - `## 修复`
  - `## 其他`
- 发布日志不需要带类似 “# JM Boom v0.2.2” 的版本号。
- 每条发布日志记录尽量带对应 commit hash 链接，格式示例：`- 设置页进入时自动检查更新 ([abc1234](https://github.com/ppxb/jm-boom/commit/abc1234))`。
- TAURI_SIGNING_PRIVATE_KEY 用户已完成配置，发布由用户手动通过 Github Workflow 完成。

## 参考项目

- API / 数据源主要参考同级 `Breeze-plugin-JmComic`。
- `Breeze` 更适合作为桌面/跨平台阅读器排版与翻页参考。

## 项目结构

- 前端路由在 `src/routes`。
- 普通页面在 `_app` 下，阅读页是独立路由 `src/routes/reader/$comicId.tsx`，不挂 `_app`，所以没有 `floating-nav`。
- 全局用户状态在 `src/stores/user-store.ts`。
- 设置状态在 `src/stores/settings-store.ts`，使用 Zustand persist，当前包含 API endpoint、阅读缓存上限、代理和封面隐私模式。
- 阅读历史在 `src/stores/reading-history-store.ts`，persist key 是 `jm-boom-reading-history`。
- Rust API 已拆分：
  - `src-tauri/src/api.rs`：远程设置、搜索、首页、列表、排行、收藏、详情、评论、登录、签到、代理等普通接口。
  - `src-tauri/src/reader.rs`：阅读 manifest、图片下载、解扰、缓存、reader cache 管理。
  - `src-tauri/src/updater.rs`：应用更新检查和安装。
  - `src-tauri/src/lib.rs`：Tauri command 包装和 invoke handler 注册。

## 当前功能

- 首页：`src/routes/_app/index.tsx`，使用 `getHomeFeed`，信息流纵向展示，右侧轻量导航，含 BackTop。
- 列表页：`src/routes/_app/list.tsx`，复用 `getHomeSectionList`，承接首页分区、周榜、排行等列表入口。
- 每周推荐：`src/routes/_app/weekly.tsx`，使用 shadcn `Tabs`/`Select`，items 用 `ComicGrid`。
- 排行榜：`src/routes/_app/ranking.tsx`，使用 `getHomeSectionList` + `ranking-filters`。
- 收藏：`src/routes/_app/favorites.tsx`，依赖登录态，使用 `getFavoriteComics`。
- 详情页：`src/routes/_app/comic/$comicId.tsx`，展示详情、章节、相关推荐、评论 Drawer，支持收藏切换。
- 历史观看：`src/routes/_app/history.tsx`，以作品 `comicId` 为唯一键覆盖历史，`chapterId` 保存当前章节/read id；点击历史进入 `/reader/$comicId` 并带 `pageIndex` 恢复进度。
- 阅读页：`src/routes/reader/$comicId.tsx` + `src/features/reader/*`，`$comicId` 语义是 read/chapter id，作品 id 来自 search `albumId`。
- 设置页：`src/routes/_app/settings.tsx`，动态发现 API endpoint，支持 HTTP/SOCKS5 代理、主题、封面隐私、缓存上限、缓存管理和应用更新。
- 个人中心：`src/routes/_app/me.tsx`，登录态来自 Zustand，支持签到数据和签到操作。

## Rust / Tauri API 要点

- 主要 commands：`get_remote_setting`、`discover_api_endpoints`、`search_comics`、`get_home_feed`、`get_home_section_list`、`get_week_filters`、`get_week_items`、`get_comic_detail`、`toggle_comic_favorite`、`get_favorite_comics`、`get_comic_comments`、`login`、`get_sign_in_data`、`sign_in`、`clear_session`、`configure_network_proxy`、`get_reader_cache_stats`、`clear_reader_cache`、`open_reader_cache_dir`、`get_comic_read_manifest`、`get_comic_read_page`、`check_app_update`、`install_app_update`。
- 移动端 API `tokenparam` 版本对齐 Breeze 插件为 `2.0.20`。
- API endpoint 动态发现从 `newsvr-2025.txt` 解密 host 池，合并本地 fallback 后测速；业务请求只规范化 http/https endpoint，空 endpoint 回退到 `https://www.cdnhth.club`。
- Rust 侧支持全局代理配置，`configure_network_proxy` 会重建共享 reqwest client；HTTP/SOCKS5 依赖 reqwest `socks` feature。
- 登录使用共享 reqwest client 和全局 JWT token 维持会话，重新登录前和退出时调用 `clear_session`。
- 小说分组已过滤；详情接口遇到小说或书库内容会返回不支持提示，当前只实现漫画图片阅读链路。

## 阅读器行为

- 阅读页左右方向键翻页，`Esc` 返回，左右透明点击区也可翻页。
- 图片使用 `h-screen w-screen object-contain`，尽量占满视口但不裁切。
- 顶部 toolbar 鼠标移动时出现，短暂后隐藏；键盘操作时隐藏。
- 底部悬浮进度条显示页码；最后一页且存在下一章时显示“下一章”按钮。
- `pageIndex` search 作为初始页，`useReaderPages` 会规范化并在 manifest 返回页数后 clamp。
- 阅读进度只在已有章节标题且 `pageCount > 0` 时写入历史，避免保存无效页数。

## 阅读器当前实现

- `get_comic_read_manifest` 请求 `/chapter` 接口，用 `serde_json` 解析响应，构建图片页 manifest。
- `get_comic_read_page` 下载图片、必要时解扰、写入 app cache 的 `reader` 目录，再通过 Tauri asset protocol 给前端显示。
- `tauri.conf.json` 已允许 `$APPCACHE/reader/**`。
- 缓存上限默认约 512MB，设置页可选 128MB 到 2GB；修剪目标约为上限的 82%。
- 非扰图直接缓存原始 bytes，不再整图解码；冷缓存返回宽高 `0/0`，前端当前不依赖宽高排版。
- 扰图缓存输出 lossy WebP，当前使用 `webp` crate / libwebp，质量 `75.0`，缓存文件为 `.webp`。
- 扰图解码先 `image::load_from_memory`，转 RGB 后按 raw row copy 重排；主要成本通常在 `load_ms` 和 `encode_ms`。
- 前端 `useReaderPages` 当前页请求传 `requestOrigin: 'visible'`；相邻预取页传 `requestOrigin: 'prefetch'`。
- 当前预取在前端用 `queryClient.prefetchQuery` 串行执行，半径固定为 `BREEZE_ROW_PREFETCH_RADIUS = 1`，顺序为 `+1, -1`。
- 预取在当前页 ready 后触发；同一窗口用 `prefetchKeyRef` 去重。
- Rust 侧有同缓存路径 single-flight 锁，避免同一页重复冷处理。
- 缓存写入使用同目录 `.tmp` 文件，完成后 rename 为正式缓存文件；缓存统计和清理跳过 `.tmp`，避免并发读取半写文件。
- 当前阅读器性能日志包含 `origin=visible|prefetch`、`lock_wait_ms`、`download_ms`、`cache_ms`、`load_ms`、`reorder_ms`、`encode_ms`、`write_ms`、`cleanup_ms`、`source_kb`、`output_kb`、`ratio`、`delta_kb`。
- 当前没有单独的 Rust `prefetch_comic_read_pages` command，也没有设置页“预载数量/阅读线路”选项；如需恢复，先重新设计前后端契约。

## 重构优化 TODO LIST

重构原则：每个 Phase 尽量保持行为不变；先移动和收边界，再做语义调整；每轮完成后同步建议 `commit msg`。

### Phase 0：建立重构基线

- [ ] 跑一次当前基线验证：前端 `bun run build`，涉及 Rust 时在 `src-tauri` 跑 `cargo check`。
- [ ] 记录当前大文件和高风险边界：`src-tauri/src/api.rs`、`src-tauri/src/reader.rs`、`src-tauri/src/download.rs`、`src/routes/_app/settings.tsx`。
- [ ] 不做功能改动，只确认重构起点可构建。
- 建议 commit msg：`chore: establish refactor baseline`。

### Phase 1：运行日志与 DEBUG 日志治理

- [ ] 去掉或门控发布版中的临时调试输出：`eprintln!`、`console.debug`、过量 reader 性能日志等。
- [ ] 引入应用打包后的运行日志，用于用户反馈问题排查；默认记录 `info/warn/error`，开发调参日志仅在 debug 或显式开关下启用。
- [ ] 日志写入应用日志目录，提供“打开日志目录”或“导出诊断日志”入口；日志需有大小限制或轮转策略。
- [ ] 日志内容避免敏感信息：不记录密码、JWT、完整鉴权 header；endpoint 可记录 host/耗时/状态，必要时脱敏。
- [ ] 统一 Rust 侧日志入口，优先考虑 `tracing`/`log` facade 或 Tauri 日志插件；前端错误后续接入同一诊断路径。
- 建议 commit msg：`feat(logging): add runtime diagnostics and gate debug logs`。

### Phase 2：前端基础清理

- [ ] 抽 `src/lib/api/tauri.ts`，统一 `tauriInvoke<T>()`，移除各 `src/lib/api/*.ts` 中重复的 `ensureTauriRuntime`。
- [ ] 抽 `src/lib/query-keys.ts`，集中管理 TanStack Query key，避免缓存 key 散落。
- [ ] 抽 `src/lib/format.ts`，统一 `formatBytes`、`formatDuration` 等格式化函数。
- [ ] 修掉 `src/features/comic-detail/page.tsx` 中的 `any`，保持 TypeScript strict 友好。
- 建议 commit msg：`refactor(frontend): centralize api helpers and shared utilities`。

### Phase 3：拆分设置页

- [ ] 将 `src/routes/_app/settings.tsx` 拆到 `src/features/settings/`，路由文件仅保留 `createFileRoute` 和页面入口。
- [ ] 按 section 拆分：版本更新、外观、API endpoint、代理、缓存、隐私模式。
- [ ] 抽 `SettingRow`、`SectionTitle`、endpoint 选择/测速展示等稳定组件或 hooks。
- [ ] 保持 UI 和交互行为不变。
- 建议 commit msg：`refactor(settings): split settings page into feature modules`。

### Phase 4：拆分 Rust API 与收窄 serde 兼容

- [ ] 将 `src-tauri/src/api.rs` 拆为 `api/error.rs`、`client.rs`、`auth.rs`、`codec.rs`、`setting.rs`、`search.rs`、`home.rs`、`comic.rs`、`user.rs`、`models.rs` 等模块。
- [ ] 第一轮只移动代码，不改 Tauri command 名称和前端契约。
- [ ] 将 `deserialize_*_from_any` 抽到 `serde_ext.rs`，再按字段逐步收窄为明确语义：`string_or_number`、`bool_or_int`、`string_vec_from_array_or_string` 等。
- [ ] 只有展示型脏字段保留 lossy helper，并在命名中体现 `lossy`，避免吞掉上游 schema 变化。
- 建议 commit msg：`refactor(rust): split api module and isolate serde helpers`。

### Phase 5：拆分阅读器 Rust 与前端 hooks

- [ ] 将 `src-tauri/src/reader.rs` 拆为 `reader/types.rs`、`manifest.rs`、`page.rs`、`cache.rs`、`image_decode.rs`。
- [ ] 保留 `get_comic_read_page` 的 `index` 语义、缓存路径命名规则和阅读页 search 参数。
- [ ] 拆分 `src/features/reader/use-reader-pages.ts`：导航状态、manifest query、page query、prefetch 策略分离。
- [ ] 保证可见页优先返回，不被后台预取拖慢；重新评估是否提前触发预取。
- [ ] 检查 `ReaderImage` loading 状态，避免图片已可用但界面仍停顿。
- 建议 commit msg：`refactor(reader): separate manifest page cache and prefetch logic`。

### Phase 6：拆分下载模块

- [ ] 拆分 `src/routes/_app/downloads.tsx`，抽 `DownloadTaskCard`、`DownloadEmptyState`、`DeleteTaskDialog`、`useDownloadTasks`。
- [ ] 将 `src-tauri/src/download.rs` 拆为 `download/types.rs`、`manager.rs`、`storage.rs`、`paths.rs`、`progress.rs`。
- [ ] 保持下载任务状态机、持久化格式、任务目录规则不变。
- 建议 commit msg：`refactor(downloads): split download ui and manager modules`。

### Phase 7：收尾与架构文档同步

- [ ] 清理重复 helper、过时 fallback、无用兼容写法；不因 lint 扩大无关改动。
- [ ] 后续排版方向参考 Breeze：单页 `contain` 当前已接近；滚动模式可学习图片尺寸缓存；双页/RTL/LTR 支持尚未实现。
- [ ] 更新本文件中已经变化的项目结构和实现说明。
- 建议 commit msg：`chore: document refactored module boundaries`。

## 注意事项

- 历史观看依赖 `currentIndex`、`pageCount`、`chapterId`、`albumId` 等前端字段；优化图片解码和缓存时不要改 `get_comic_read_page` 的 `index` 语义、缓存路径命名规则或阅读页 search 参数。
- 阅读缓存路径规则目前是章节 read id 目录下 `0001.ext`；不要随意变更，否则热缓存命中和历史恢复体验会受影响。
- `src/features/reader/reader-bars.tsx` 可能有用户侧小改动，页码显示是 `{current} / {pageCount}`，不要无关回滚。
