# JM Boom Project Notes

本文件给后续接手的 Codex 使用，目标是快速理解项目约束、当前架构。内容应保持短而准，避免记录已经完成的过程流水账。

## 接手约定

- 项目是 Tauri 2 + React 19 + Vite + TanStack Router/Query + Zustand + shadcn/ui + Rust reqwest。
- 当前优先桌面端体验，小屏适配不是近期重点。
- 不要手动编辑 `src/routeTree.gen.ts`，路由树由 TanStack Router 插件、HMR 或 build 生成。
- 读取或写入包含中文的文件时必须显式使用 UTF-8，避免 Windows 终端默认编码导致乱码，使用git-bash。
- 可以并行读取文件或搜索；文件编辑使用单次 `apply_patch`，不要重复或并行套同一补丁。
- `cargo check`、`vite build`、`tsc` 等会占用输出目录或 build lock 的验证命令不要并行重复触发。
- UI 优先复用 shadcn/ui 已安装组件；需要新的 shadcn 组件时，告诉用户安装，不要自己重造复杂基础组件。
- 项目已有 `floating-nav`，不要新增传统 sidebar。
- Rust 依赖可直接添加并用 `cargo check` 下载；前端依赖不要直接安装，先告诉用户。
- 用户此前表示暂时不要纠结 `lint`；不要因为 `lint` 结果扩大无关改动。
- 每次大的变动后，同步用户相应的 `commit msg`。

## 发布流程

- 发布前必须同步修改三处版本号：`package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`。
- 改完 Rust crate 版本后运行 `cargo check`，更新并验证 `src-tauri/Cargo.lock` 中的 `jm-boom` 版本。
- 每个发布 tag 必须有对应发布日志：`docs/release-notes/<tag>.md`，例如 `docs/release-notes/v0.2.2.md`。
- 发布日志按上一个 tag 到本次发布的 commit 生成，建议先看提交范围：
  - `git describe --tags --abbrev=0`
  - `git log <previous-tag>..HEAD --oneline`
- GitHub Actions 会校验发布日志**必须包含**以下四个板块（即使为空也必须存在）：
  - `## 功能`
  - `## 优化`
  - `## 修复`
  - `## 其他`
- 板块为空时，可以只写标题不写内容，或写”无。”。
- 发布日志不需要带类似 “# JM Boom vx.x.x” 的版本号。
- 每条发布日志记录尽量带对应 commit hash 链接，格式示例：`- 设置页进入时自动检查更新 ([abc1234](https://github.com/ppxb/jm-boom/commit/abc1234))`。
- 发布日志突出重点。
- TAURI_SIGNING_PRIVATE_KEY 用户已完成配置，发布由用户手动通过 Github Workflow 完成。

## 参考项目

- API / 数据源主要参考同级 `Breeze-plugin-JmComic`。
- `Breeze` 更适合作为桌面/跨平台阅读器排版与翻页参考。
