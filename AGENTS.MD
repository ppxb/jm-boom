# JM Boom Project Notes

本文档只记录当前架构约束、未完成的优化方向和验证要求。内容应保持短而准；问题解决后及时删除或更新对应条目，不记录迁移历史、版本流水账或一次性验证结果。

## 架构与数据边界

- 项目由 `web/` React 前端与 `server/` Rust/Axum 后端组成。前端使用 React 19、Vite、TanStack Router/Query、Zustand 和 shadcn/ui。
- 后端统一负责匿名 JM API 请求、端点探测与故障切换、图片解扰与缓存、下载任务、收藏和阅读历史。
- 收藏与阅读历史是无登录的单实例共享数据，持久化到服务端 SQLite，不依赖浏览器本地状态或上游云收藏。阅读历史不设条数上限；书架与收藏固定每页 20 条，不足或等于一页时隐藏分页器。
- 详情页按漫画 ID 查询收藏与历史状态；收藏、进度和删除写接口不返回完整集合列表。
- API DTO 与 domain 类型保持边界。封面 URL 等传输层改写继续在 API 层完成，不为减少字段映射而让 domain 类型直接承担 HTTP serde。
- 不要添加大量兜底分支或无必要抽象，优先保持代码简洁、高效、可读。

## 阅读与下载约束

- 下载使用服务端有界队列：最多同时运行 2 个漫画任务，每个任务最多 5 个页面 job。Manager 统一控制状态转换与 worker 重调度；暂停、取消和删除通过通知退出，恢复与删除使用 generation 隔离旧 worker。
- 下载文件保存在 `data/downloads/<task_id>/`，章节 manifest 持久化到 SQLite。上游不可用时阅读器回退到离线 manifest 和页面文件；删除任务时同步删除 manifest 与文件目录。
- 阅读与下载共用页面物化服务以及同一套 single-flight、图片优先级、下载重试、解扰和缓存写入流程，不要复制第二套图片处理路径。
- 上游单张图片响应硬限制为 64 MiB。仅需解扰的图片进入受尺寸、像素数和解码分配限制保护的解码流程；无需解扰的 JPEG、PNG、WebP 和 GIF 原样缓存，不额外解码，仅解扰结果重新编码为 WebP。
- 阅读缓存 URL 包含缓存算法版本。漫画封面通过同源 `/api/covers/:id` 代理并原样写入服务端图片缓存；详情页主封面加载后才预热阅读图片，列表封面继续懒加载。
- 阅读器完整视口容器使用 `dvh`。切换章节时按章节 ID 从第一页重建会话；条漫滚动容器参与 Router 顶部重置；仅有下一章时显示继续滚动过渡区并支持移动端上滑切章。

## 前端与部署约定

- 前端依赖、类型检查、构建、lint 和格式验证统一使用 Bun；不要使用 pnpm，也不要用 npm 改写锁文件。
- 不要手动编辑 `web/src/routeTree.gen.ts`，路由树由 TanStack Router 插件生成。
- 产品界面移动端优先并完整适配桌面端。主导航固定为书架、探索、收藏、下载、设置，在所有尺寸下保持底部胶囊形态；漫画详情与阅读器隐藏主导航。
- 书架、探索、收藏、下载和设置使用 `AppPage` 页面外壳；漫画详情与阅读器保留独立布局。探索承载推荐、每周、排行、搜索和首页分区列表。
- 服务端默认不启用 CORS，生产和 Vite 开发代理均按同源访问；跨域仅通过逗号分隔的 `JM_BOOM_CORS_ORIGINS` 显式配置完整 `http(s)` Origin。
- 服务启动后后台探测内置兜底与动态发现端点；探测完成前使用两个兜底故障切换，完成后自动模式选择最优端点。后台每 2 小时刷新，设置页手动探测与定时探测串行执行，并保留手动选择优先级。
- Docker 使用单容器同源部署：Axum 托管 `web/dist` 并提供 SPA fallback，未知 `/api` 路由保持 JSON 404，运行数据统一持久化到 `/app/data`。
- `JM_BOOM_ACCESS_PASSWORD` 只启用轻量前端访问门禁：服务端校验登录请求，前端使用 `sessionStorage` 控制页面进入，业务 API 当前不鉴权。
- 镜像发布到 `ghcr.io/ppxb/jm-boom`；Git tag `v*.*.*` 触发 GitHub Actions 构建 AMD64/ARM64 镜像并发布版本、`latest` 与 `sha-*` 标签。
- 读取或写入中文文件时显式使用 UTF-8。文件编辑使用 `apply_patch`，并保留用户已有的无关改动。

## 当前优化顺序

1. 收敛无语义差异的前端查询重复：复用 ranking/list 的分类和排序解析，只删除与 QueryClient 默认值完全相同的查询选项。
2. 将 `ReadingHistoryService::remove_many` 改为单次批量 SQL，并覆盖空输入、多 ID 和不存在 ID 的行为。
3. 增加下载任务批量删除 API，前端一次提交选中 ID，服务端完成 worker 停止、manifest/目录删除和一次列表返回，避免串行调用单删接口。
4. API 层只抽取纯粹且稳定的数字 ID、页码等校验；保留各模块的错误类型、用户文案和日志上下文，不强行统一所有错误层级。
5. 下载列表 clone 和 1 秒轮询仅在有规模或测量依据时优化；优先精简响应快照或 mutation 返回值，暂不为此引入 SSE。

## 优化边界

- 不重写下载 Manager、generation 模型或 worker 调度，除非存在明确缺陷。
- 不把阅读器职责 hook 合并成巨型 hook，也不为预加载、物化或缓存再增加无必要层级。
- 探索页优先共享 loading、error、empty、grid、pagination 等小部件；不要预先抽出参数繁多的万能列表组件。
- `sqlx::FromRow`、身份字段映射、测试文件外移和 GET `Content-Type` 属于清晰度整理，不作为性能优化优先项。
- 书架/收藏交互和 ranking/list 路由收敛涉及产品语义，不能作为纯代码去重直接改变。

## 验证基线

- 前端在 `web/` 运行：`bun x tsc --noEmit`、`bun run lint`、`bun run fmt:check`、`bun run build`。
- 后端在 `server/` 运行：`cargo test --locked`、`cargo fmt --check`、`cargo clippy --locked -- -D warnings`。
- 提交前运行 `git diff --check`；测试范围随改动风险扩大，不为无关模块制造格式或元数据变更。
