# JM Boom Project Notes

本文件记录当前项目已经实现的关键工作和后续接手约定，避免下次继续开发时重新摸索。

## 基本约定

- 这是 Tauri 2 + React 19 + Vite + TanStack Router/Query + shadcn/ui + Rust reqwest 项目。
- 当前优先桌面端体验，暂时不重点处理小屏适配。
- 用户明确说过：代码已格式化，不用优先管 lint。常规验证优先跑 `bun run build`；如果当前 Windows 环境没有 `bun`，可直接跑本地 `node_modules/.bin/tsc.exe --noEmit` 和 `node_modules/.bin/vite.exe build`；改 Rust 后再跑 `cargo check`。
- 不要手动编辑 `src/routeTree.gen.ts`。路由变更后由 TanStack Router 插件、HMR 或 build 自动生成。
- 读取或写入包含中文的文件时必须显式使用 UTF-8 编码，避免 Windows 终端默认编码导致乱码读写。
- 前端和 Rust 通信用 Tauri `invoke`，项目不再保留 axios；新接口不要新增 axios 调用。
- 全局用户状态使用 Zustand：`src/stores/user-store.ts`，目前只在内存中保存登录用户和 endpoint。
- UI 优先复用 shadcn/ui 已安装组件。需要新 shadcn 组件时，告诉用户安装，不要自己重造复杂基础组件。
- 项目已有 `floating-nav`，不要再加传统 sidebar。
- 需要安装外部依赖，如果该依赖是 Rust 需要的，则直接添加，后续使用 `cargo check` 下载，如果是前端项目，则直接告诉用户，由用户来安装相关依赖。
- 项目接口实现或数据来源参考同级项目的 `jm-mobile` 项目。
- 漫画相关的渲染、预览、接口实现可以参考同目录层级的 `jm-mobile`、`Breeze-plugin-JmComic` 和 `Breeze` 三个项目。

## 已实现页面

- 首页：`src/routes/_app/index.tsx`
  - 通过 `getHomeFeed` 获取信息流。
  - 从上到下依次渲染各个信息流大项，不使用 tabs。
  - 右侧有轻量“导航”目录，可平滑跳转并随滚动高亮当前分组。
  - 有独立 BackTop 组件，滚动到一定距离后显示。

- 每周推荐：`src/routes/_app/weekly.tsx`
  - 已使用 shadcn `Tabs` 和 `Select`。
  - filters 使用较长 stale time，切换 tabs/select 时只刷新下方 items 区域。
  - items 使用 `ComicGrid`，骨架屏尺寸和真实 card 保持一致。

- 详情页：`src/routes/_app/comic/$comicId.tsx`
  - 展示封面、标题、作者、stats、简介、标签、章节、相关推荐。
  - 评论通过 stats 中的评论入口打开右侧 Drawer。
  - 评论 Drawer 不显示头像，滚动到底部自动加载更多；无更多时显示“暂无更多评论”。
  - 评论时间使用原生 `Intl.DateTimeFormat('zh-CN')` 格式化为中文日期/时间，不再使用 `date-fns`，也不显示相对时间。
  - 章节使用 shadcn `Pagination`，每页 10 个。
  - 章节标题兜底规则：有 `chapter.title` 用标题；否则有 `sort` 显示 `第 N 章`；再否则显示 `章节 N`。
  - 章节进入阅读页时会传 `title`、`chapter`、`albumId`、`pageIndex: '0'`、`nextId`、`nextChapter`。

- 历史观看：`src/routes/_app/history.tsx`
  - 使用 `useReadingHistoryStore` 读取本地阅读历史，并按 `updatedAt` 倒序展示 4 列卡片。
  - 历史记录以漫画/作品 `comicId` 为唯一键覆盖，同一作品只保留最近一次阅读进度；当前章节单独保存为 `chapterId`。
  - 历史卡片点击时使用 `chapterId` 进入 `/reader/$comicId`，同时传入 `title`、`chapterTitle`、`albumId` 和保存的 `pageIndex`，阅读器会直接 seek 到保存页。
  - 卡片展示封面、JM id、进度条、当前章节、作者和更新时间；封面隐私模式复用 `settings-store.hideCovers`。
  - 清除记录使用 shadcn `AlertDialog` 二次确认，样式参考启动时的 NSFW 警告弹窗；确认后清空本地历史并 toast 提示。
  - 不需要兼容旧历史数据结构；结构变更后用户可以通过“清除记录”重建最新数据。

- 阅读页：`src/routes/reader/$comicId.tsx`
  - 独立于 `_app`，所以没有 floating-nav。
  - 路由参数 `$comicId` 在阅读页语义上是 read/chapter id；作品 id 通过 search 中的 `albumId` 传入。
  - 顶部 toolbar 显示返回、作品标题、章节名、刷新按钮；不显示页码。
  - toolbar 在鼠标移动时出现，短暂后隐藏；键盘操作时隐藏。
  - 左右方向键翻页，`Esc` 返回；页面左右透明点击区也可翻页。
  - 图片使用 `h-screen w-screen object-contain`，尽量占满视口但保留原比例，不裁切。
  - 底部 footer 是悬浮进度组件：`absolute bottom-10 left-1/2 w-80`，圆角背景，内部上方进度条、下方页码。
  - 到当前章节最后一页且存在下一章时，右下角单独显示“下一章”按钮。
  - 阅读页会优先使用 search 中的章节名；如果为空，会根据 `albumId` 获取详情并按同一兜底规则生成章节名。
  - `pageIndex` search 会作为初始页，`useReaderPages` 会规范化并在 manifest 返回页数后 clamp 到合法范围；历史记录恢复依赖这个 seek 行为。
  - 阅读进度只在已有章节标题且 `pageCount > 0` 时写入历史，避免保存无效页数。

- 个人中心：`src/routes/_app/me.tsx`
  - 使用 Zustand 的 `useUserStore` 读取登录用户、endpoint 和 logout。
  - 未登录时显示登录入口；已登录后展示头像、UID、邮箱、经验、等级、金币、收藏容量、等级进度和常用入口占位。
  - 签到数据通过 `getSignInData` 获取，签到动作通过 `signIn` 触发，成功后 toast 提示并 refetch。
  - 签到图为 GitHub 风格 6 个月视图：前 5 个月 + 本月完整日期，并补齐到周网格；尽量不出现横向滚动条。
  - 签到格子小尺寸显示：已签到为绿色，额外奖励日只加黄色边框；不再显示图例和月份标签。
  - 单个格子 hover 使用 Tooltip 显示中文日期、签到状态和额外奖励提示。
  - 当前接口主要返回本月记录，前端优先用 `record.date` 匹配完整日期，无法解析时 fallback 到本月 `day` 序号；后续若用 app storage 保存历史月份，可直接按完整日期回填。

- 设置页：`src/routes/_app/settings.tsx`
  - 使用单个主 Card 展示全部配置大项，不保留右侧当前配置面板。
  - API 接口不再使用静态白名单；设置页通过 Tauri command `discover_api_endpoints` 拉取 `newsvr-2025.txt`，合并本地 fallback 后统一测活测速。
  - 本地 fallback API 端点为 `https://www.cdnhth.club` 和 `https://www.cdnhjk.net`；`newsvr-2025.txt` 仅作为动态候选来源，不直接当作业务 API。
  - API 下拉展示连通状态和延迟，重测按钮会重新请求配置源并测速；探测完成后会自动优选可用接口中延迟最低的 endpoint。
  - 代理是独立配置大项，支持关闭、HTTP、SOCKS5，本地默认值为 `127.0.0.1:7890`。
  - 主题使用图标 Tabs；NSFW 保护里的封面隐私模式使用项目 `Switch`；阅读图片线路、预载数量、缓存上限使用 Select。
  - 缓存区展示当前缓存大小、缓存上限、disabled 样式缓存路径、打开缓存目录按钮和 destructive 清理缓存按钮。

- 应用壳：`src/routes/_app.tsx`
  - `floating-nav` 不再承载主题切换，主题切换统一放在设置页。
  - `floating-nav` 的 `me` 路由未登录时会拦截跳转并打开 `LoginDialog`。
  - 登录成功后自动跳转到 `/me`，个人页直接渲染 Zustand 中的用户数据。
  - 应用启动后会把持久化的代理配置同步给 Rust 侧 HTTP client。

## 组件与视觉状态

- `src/features/user/login-dialog.tsx`
  - 使用 shadcn `Dialog`、`Input`、`Button` 和项目 `Field` 组件。
  - 登录失败用 `sonner` toast 展示错误，不把长错误文本渲染进表单布局，避免 input 区域被撑开。
  - 登录按钮在用户名/密码为空或正在登录时禁用；成功后关闭对话框并触发可选 `onLoginSuccess`。

- `src/stores/user-store.ts`
  - 使用 Zustand 管理 `user`、`endpoint`、`isLoggingIn`、`login`、`logout`。
  - `login` 调用 Tauri `login` command，保存返回的 user 和 endpoint。
  - `logout` 调用 `clear_session` command 后清空本地 store。

- `src/stores/settings-store.ts`
  - 使用 Zustand persist 保存 `api`、阅读图片线路、阅读预载数量、阅读缓存上限、代理配置和封面隐私模式。
  - `api` 是运行时字符串，不是固定 union；写入时只做 URL 规范化，允许选择动态发现出的 API endpoint。
  - 代理配置包含 `proxyMode`、`proxyHost`、`proxyPort`；二值设置优先使用项目 `Switch` 组件。

- `src/stores/reading-history-store.ts`
  - 使用 Zustand persist 保存本地阅读历史，存储名为 `jm-boom-reading-history`。
  - `ReadingHistoryItem` 当前字段为：`comicId`、`albumId`、`title`、`author`、`coverUrl`、`chapterId`、`chapterTitle`、`pageIndex`、`pageCount`、`updatedAt`。
  - `comicId` 是作品/漫画 id，也是历史记录唯一键；`chapterId` 是阅读接口使用的章节/read id。
  - `upsert` 会把新记录放到最前面，并删除同 `comicId` 的旧记录；不做旧数据结构迁移。

- `src/components/comic-feed.tsx`
  - `ComicGrid` 为 4 列网格。
  - Comic card 使用 shadcn `Card`。
  - 封面是 `aspect-square`，只展示标题和作者。
  - 标题单行省略，hover 使用项目 Tooltip。
  - 封面左上角半透明圆角容器显示 `JM id`。
  - 封面隐私模式读取 `settings-store.hideCovers`，开启时用统一蒙版遮住真实封面。

## Rust / Tauri API

- Rust API 已从 `lib.rs` 拆出：
  - `src-tauri/src/api.rs`：远程设置、搜索、首页、每周推荐、详情、评论等普通接口。
  - `src-tauri/src/reader.rs`：阅读 manifest、图片下载、解码、缓存、预取。
  - `src-tauri/src/lib.rs`：只保留 Tauri command 包装和 invoke handler 注册。

- 已注册 Tauri commands：
  - `get_remote_setting`
  - `discover_api_endpoints`
  - `search_comics`
  - `get_home_feed`
  - `get_week_filters`
  - `get_week_items`
  - `get_comic_detail`
  - `get_comic_comments`
  - `login`
  - `get_sign_in_data`
  - `sign_in`
  - `clear_session`
  - `configure_network_proxy`
  - `get_reader_cache_stats`
  - `clear_reader_cache`
  - `open_reader_cache_dir`
  - `get_comic_read_manifest`
  - `get_comic_read_page`
  - `prefetch_comic_read_pages`

- API / 数据源 Rust 侧要点：
  - 当前移动端 API `tokenparam` 版本对齐 Breeze 插件为 `2.0.20`。
  - `discover_api_endpoints` 会从两个 `newsvr-2025.txt` 配置源读取并 AES-ECB 解密 host 池，合并本地 fallback 后逐个请求 `/setting?app_img_shunt=1&t=...` 测活测速。
  - 业务请求端不做编译期白名单限制，只规范化 http/https endpoint；空 endpoint 回退到 `https://www.cdnhth.club`。
  - Rust 侧支持全局网络代理配置，`configure_network_proxy` 变更配置后会清掉共享 reqwest client，让后续 API、阅读 manifest 和图片下载请求使用新代理；HTTP 和 SOCKS5 代理依赖 `reqwest` 的 `socks` feature。
  - 首页 `promote` 数据过滤 `禁漫小说` 和 `禁漫书库` 分组；当前项目只实现漫画图片阅读链路，小说类型暂不进入详情/阅读。

- 用户 / 签到 Rust 侧要点：
  - `login` 使用 multipart 请求 `/login`，登录响应里的 `uid` 等数字字段可能是字符串，反序列化需兼容 string/number。
  - Rust reqwest client 现在通过共享 client + cookie jar 维持会话；重新登录前和退出登录时调用 `clear_session` 清掉会话。
  - `get_sign_in_data` 请求 `/daily`，返回 `dailyId`、奖励配置、event name、current progress、record 列表。
  - `sign_in` 请求 `/daily_chk`，需要 `user_id` 和 `daily_id`。
  - user avatar 通过远程 `img_host` 拼接 `/media/users/{photo}`，如果服务端直接返回 http(s) 地址则原样使用。

- 阅读器 Rust 侧要点：
  - 读取 `chapter_view_template`。
  - 使用 `json5` 解析源端宽松 JS/JSON 对象。
  - 使用 `image` crate 解码被切割/扰乱的图片。
  - 解码后页面缓存到 app cache 的 `reader` 目录。
  - 缓存上限约 512MB，修剪目标约 420MB。
  - 当前页会预取前后约 3 页。
  - `tauri.conf.json` 已开启 asset protocol，并允许 `$APPCACHE/reader/**`。

## 阅读器性能优化实验计划

当前体感问题集中在章节图片首次准备和翻页显示偏慢。后续优化按小步实验推进，每次只落一个策略，跑 `cargo check` 和前端构建后，再用同一章节做冷缓存/热缓存对比。记录时建议至少观察：首张图可见耗时、连续翻页等待、CPU 峰值、缓存目录增长、失败重试情况。

| 序号 | 策略 | 核心改动 | 预估收益 | 风险 / 代价 | 测试重点 |
| --- | --- | --- | --- | --- | --- |
| 1 | 非扰图快路径 | 对不需要解扰的图片，避免 `image::load_from_memory` 整图解码；直接缓存原始 bytes 并返回路径，宽高/aspectRatio 可先返回 0 或按需懒加载 | 非扰图首次准备可能减少 20%-50%；CPU 明显下降 | 前端若依赖宽高需要兜底；缓存损坏检测会少一次图片解析 | GIF、老章节、`speed == "1"` 章节能否直接显示；错误图是否可重试 |
| 2 | 扰图 raw buffer 重排 | 参考 Breeze Rust 实现，用 RGB/RGBA raw row copy 替代 `crop_imm(...).to_image()` + `replace()` | 扰图解码阶段可能减少 10%-30%；内存临时分配下降 | 需要严格对齐当前算法，避免边界行错位 | 大图、余数行、不同 seed、GIF 跳过逻辑 |
| 3 | WebP 编码策略调整 | 扰图缓存仍存 WebP，但降低质量或使用更快编码；或评估 PNG/JPEG 直存 | 扰图总耗时可能减少 10%-40%，取决于编码占比 | 质量、体积、透明通道处理要权衡 | 视觉质量、文件大小、首次解码耗时 |
| 4 | 预取有限并发 | `prefetchCount` 表示当前页前后各准备 N 张；预取并发直接跟随当前设置值，避免现在偏串行 | 翻页等待可能明显下降；热路径更顺 | 网络和 CPU 峰值会上升；过高并发会抢当前页资源 | 快速连续翻页、低端 CPU、代理慢网环境 |
| 5 | 当前页优先队列 | 当前页下载/解扰永远优先于预取；预取任务发现当前页切换后可跳过过期任务 | 体感响应更稳定，减少后台任务拖慢当前页 | 需要任务取消或过期检查，状态复杂度上升 | 快速左右切页时是否还会卡住旧页 |
| 6 | Manifest 返回图片元信息 | manifest 直接返回 page 列表、source_url、是否需要解扰；前端/预取可更早决策 | 减少重复判断，便于后续直接图源实验 | 暴露源 URL 要注意 referer、权限和隐私 | 返回结构兼容、动态 shunt、缓存命中 |
| 7 | 非扰图直取显示实验 | 对非扰图不落本地缓存，直接让前端显示远程 URL；必要时 Rust 提供代理 URL 或 blob | 冷启动可能最快，省磁盘 IO | WebView 跨域、referer、鉴权、离线缓存都会更复杂 | Tauri WebView 是否能稳定加载、是否被 CDN 拒绝 |

预取设置约定：设置页里的“预载数量”应表示当前页两侧的预取半径，例如 `3` 代表最多准备前 3 张和后 3 张。预取并发数直接跟随当前设置值，例如 `prefetchCount = 3` 时最多并发准备 3 张预取页。当前页加载不计入预取并发，并且必须优先于所有预取任务。

## 验证建议

- 仅前端改动：跑 `bun run build`。
- 如果 `bun` 不在 PATH，可退一步跑 `node_modules/.bin/tsc.exe --noEmit` 和 `node_modules/.bin/vite.exe build`。
- 改 `src-tauri` Rust 代码：跑 `cargo check`，工作目录为 `src-tauri`。
- 用户此前表示暂时不要纠结 lint；不要因为 lint 结果扩大无关改动。

