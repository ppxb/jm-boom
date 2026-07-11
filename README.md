# JM Boom

JM Boom 当前由 React Web 前端和 Rust/Axum 服务端组成。生产镜像由 Axum 同源托管前端静态资源与 `/api`，运行数据统一保存在 `/app/data`。

## Docker 部署

公开镜像：

```text
ghcr.io/ppxb/jm-boom
```

使用仓库中的 Compose 启动 0.5.0：

```bash
cp .env.example .env
# 编辑 .env 并设置 JM_BOOM_ACCESS_PASSWORD
docker compose pull
docker compose up -d
```

默认访问地址：<http://localhost:3000>。

查看状态和日志：

```bash
docker compose ps
docker compose logs -f jm-boom
```

默认使用名为 `jm-boom-data` 的 Docker volume 持久化 SQLite、图片缓存和离线下载。升级或重建容器不会删除该 volume。

可以通过环境变量覆盖镜像版本和宿主机端口：

```bash
JM_BOOM_VERSION=0.5.0 JM_BOOM_PORT=8080 docker compose up -d
```

升级：

```bash
docker compose pull
docker compose up -d
```

回滚时将 `JM_BOOM_VERSION` 改为旧版本后重新启动。

## 本地构建镜像

```bash
docker build \
  --build-arg VERSION=0.5.0 \
  --build-arg REVISION=local \
  -t ghcr.io/ppxb/jm-boom:0.5.0 .
```

## 发布镜像

推送符合 `v*.*.*` 的 Git tag 后，GitHub Actions 会构建 `linux/amd64` 和 `linux/arm64` 镜像并发布到 GHCR：

```bash
git tag v0.5.0
git push origin v0.5.0
```

0.5.0 会发布以下标签：

```text
ghcr.io/ppxb/jm-boom:0.5.0
ghcr.io/ppxb/jm-boom:latest
ghcr.io/ppxb/jm-boom:sha-<commit>
```

首次发布后，需要在 GitHub Packages 的 `jm-boom` Package settings 中将可见性改为 Public。

## 运行时配置

- `RUST_LOG`：服务端日志过滤，默认 `info,jm_boom_server=info`。
- `JM_BOOM_ACCESS_PASSWORD`：进入 Web 界面前使用的轻量访问密码；Compose 部署时必须配置。
- `JM_BOOM_STATIC_DIR`：前端静态资源目录，镜像内默认为 `/app/static`。
- `JM_BOOM_CORS_ORIGINS`：可选的逗号分隔跨域 Origin；同源部署不需要设置。

健康检查地址：`/health`。

轻量门禁只控制前端页面进入，不保护业务 API。登录成功状态保存在当前浏览器标签页的 `sessionStorage` 中，关闭标签页后需要重新登录。

本地开发时可在 PowerShell 中临时启用：

```powershell
$env:JM_BOOM_ACCESS_PASSWORD = "dev-password"
cargo run --manifest-path server/Cargo.toml
```

未设置该环境变量时，本地开发会自动跳过登录页。
