# JM Boom Project Notes

本文档记录 `feat/docker` 分支放弃 Tauri、迁移为 React Web 前端与 Axum 后端后的当前架构约束、已确认问题和后续工作顺序。内容应保持短而准；问题修复后及时删除或更新对应条目，不记录过程流水账。

## 当前架构与约定

- 项目已完全放弃 Tauri，当前结构为 `web/` React 前端与 `server/` Rust/Axum 后端。
- 前端使用 React 19、Vite、TanStack Router/Query、Zustand、shadcn/ui。
- 后端负责匿名 JM API 请求、端点探测、阅读图片解扰、缓存和下载任务。
- 下载使用服务端有界队列：同时最多运行 2 个漫画任务，每个任务最多 5 个页面 job，全局最多 8 个下载页面 job；暂停和取消通过 worker 通知退出，恢复使用 generation 隔离旧 worker。
- 下载文件独立保存在 `data/downloads/<task_id>/`，章节 manifest 持久化到 SQLite；阅读器在上游不可用时回退到离线 manifest 和页面文件，删除任务时同步删除对应 manifest 与文件目录。
- 阅读页仅在确需解扰时重新编码为 WebP；无需解扰的 JPEG、PNG、WebP 和 GIF 原样缓存，阅读 URL 包含缓存算法版本。
- 漫画封面通过同源 `/api/covers/:id` 代理并原样写入服务端图片缓存；详情页主封面加载完成后才开始阅读图片预热，列表封面继续懒加载。
- 前端依赖、类型检查、构建、lint 和格式验证统一使用 Bun，不使用 pnpm；不要用 npm 改写依赖锁。
- 不要手动编辑 `web/src/routeTree.gen.ts`，路由树由 TanStack Router 插件生成。
- 读取或写入中文文件时显式使用 UTF-8。
- 文件编辑使用 `apply_patch`，保留用户已有的无关改动。
- 服务端默认不启用 CORS，生产与 Vite 开发代理均按同源访问；确需跨域时使用逗号分隔的 `JM_BOOM_CORS_ORIGINS` 显式配置完整 `http(s)` Origin。
- Docker 使用单容器同源部署：Axum 托管 `web/dist` 并提供 SPA fallback，`/api` 未知路由保持 JSON 404；运行数据统一持久化到 `/app/data`。
- Docker 通过 `JM_BOOM_ACCESS_PASSWORD` 启用轻量前端访问门禁；服务端仅校验登录请求，前端使用 `sessionStorage` 控制页面进入，业务 API 暂不鉴权，后续再升级为正式 Session/JWT。
- 公开镜像发布到 `ghcr.io/ppxb/jm-boom`，首个容器版本为 `0.5.0`；Git tag `v*.*.*` 触发 GitHub Actions 构建 AMD64/ARM64 镜像并发布版本、`latest` 与 `sha-*` 标签。
- 产品界面采用移动端优先、桌面端完整适配的响应式方案；主导航在所有尺寸下都使用底部胶囊形态，不恢复桌面左侧导航。
- 书架、探索、收藏、下载和设置统一使用 `AppPage` 页面外壳；漫画详情与阅读器保留独立布局。
- 主导航固定为书架、探索、收藏、下载、设置。书架使用 `ShellIcon`；收藏、下载、设置沿用现有图标。
- 探索负责推荐、每周、排行、搜索和首页分区列表；漫画详情与阅读器隐藏底部导航。
- 收藏只保存到当前浏览器，不依赖登录或上游云收藏。
- JM 账号登录、退出、签到和上游云收藏已明确放弃，相关 API、JWT、会话状态和账号字段不再保留。

## 已保留的非 Tauri 功能

- 首页、首页分区列表、每周推荐、排行榜和搜索。
- 漫画详情、章节、相关推荐、评论和本地收藏。
- 单页、双页、条漫、键盘/点击翻页和自动阅读。
- 本地阅读历史。
- 服务端上游接口发现、测速、自动选择和故障切换。
- 服务端阅读缓存和批量章节缓存任务。

以下能力属于 Tauri/桌面运行时能力，放弃是预期行为，不作为缺失：应用内更新安装、系统目录打开、桌面日志目录、Tauri IPC、桌面窗口/托盘和本地文件 URL 转换。

JM 登录、签到和上游云收藏同样属于已明确放弃的产品能力，不作为功能缺失。

## 当前优先问题

### P2：搜索功能存在回退缺失和遗留协议

- 后端保留上游 `redirect_aid`，前端声明但不处理。部分搜索结果只有重定向漫画 ID 时会错误显示为空。
- `search.ts` 仍构造旧插件协议的 `source/scheme/data/extern`，当前页面实际只使用 `items` 和 `paging`。
- 应在后端完成 redirect fallback，并让 HTTP API 返回前端真正需要的简洁 DTO。
- 当前搜索结果把 `updatedAt` 写成请求时的 `Date.now()`，不应伪造更新时间；没有真实值时使用 `null`。

### P2：上游响应反序列化健壮性较 master 退化

- 漫画详情中的 `author/tags/actors/works` 直接使用 `Vec<String>`。上游若返回单个标量，整个接口可能反序列化失败。
- `master` 已存在数组/标量混合字段的宽松解析逻辑；迁移时应复用或集中实现，不能在 `comics.rs`、`home.rs`、`models.rs` 中复制多套近似 helper。
- 为真实上游响应样本增加反序列化回归测试。

### P2：阅读器与下载并发优先级

- 下载任务和下载页面 job 已有独立并发上限；阅读器后台预热与图片解码仍未和下载共享前台优先的并发预算，可能出现预热先持有页面物化锁的优先级反转。

## 设置页尚未对齐的非 Tauri 能力

- 网络代理：代理作用对象现在是服务端 HTTP Client，应改为服务端环境变量或受保护的管理配置，而不是浏览器本地设置。
- Web/Server 构建版本已恢复展示；检查更新和安装更新仍属于后续独立能力，其中安装更新可继续放弃。

## 冗余和待清理代码

- `web/src/components/theme-toggle.tsx` 当前没有引用。
- `/api/comics/:id/chapters` 当前没有前端调用，详情接口已经返回章节列表。
- `server/Cargo.toml` 中直接依赖的 `tower`、`cbc`、Redis feature 当前没有实现用途；Axum 的 multipart/ws 也未使用。清理前先确认近期规划。
- `readerFileSrc()` 目前只是返回同源图片 URL，可在后续收口阅读 DTO 时一并内联删除。
- 漫画详情前端人为补出的 `seriesId: ''`、`price: 0`、`purchased: true` 应删除或由真实 DTO 表达。

## 测试和验证基线

当前已验证：

- `bun x tsc --noEmit`
- `bun run lint`
- `cargo test --locked`
- `cargo fmt --check`
- `git diff --check master...HEAD`

当前 Rust 工具链下，严格 Clippy 仍有 `download.rs` 和 `crypto.rs` 中 2 个既有警告，需单独清理后恢复为验证基线。

服务端目前只有首页分区过滤、CORS Origin 解析和图片分段算法等少量单测。后续至少补充：

- API JSON 合约测试，包括空响应和错误响应。
- 下载暂停、恢复、取消、删除和重启恢复状态机测试。
- manifest 与页面缓存的离线读取测试。
- 缓存容量、LRU 淘汰、孤儿文件和并发物化测试。
- 上游混合字段反序列化样本测试。

## 建议实施顺序

1. 收口阅读器前台优先的共享并发预算。
2. 完成搜索 fallback、上游宽松解析和阅读器真实预取。
3. 清理兼容层、未使用代码和重复 helper。
4. 补齐 API/状态机/缓存集成测试与新的 Web 架构 README。
