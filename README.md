# JM Boom

当前分支提供通过 `Docker` 部署的前后端分离项目。

## Docker 部署

公开镜像：

```text
ghcr.io/ppxb/jm-boom
```

使用仓库中的 Compose 部署：

```bash
# 先编辑 docker-compose.yml 中的 JM_BOOM_ACCESS_PASSWORD
docker compose pull
docker compose up -d
```

默认访问地址：<http://localhost:3000>。

查看状态和日志：

```bash
docker compose ps
docker compose logs -f jm-boom
```

默认使用 Compose 文件同级的 `./jm-boom/data` 目录持久化 SQLite、图片缓存和离线下载。升级或重建容器不会删除该目录。

镜像使用 UID `10001` 的非 root 用户运行。Linux/NAS 如果提示数据目录没有写入权限，可执行：

```bash
mkdir -p ./jm-boom/data
sudo chown -R 10001:10001 ./jm-boom/data
```

升级：

```bash
docker compose pull
docker compose up -d
```

## 运行时配置

- `RUST_LOG`：服务端日志过滤，默认 `info,jm_boom_server=info`。
- `JM_BOOM_ACCESS_PASSWORD`：进入 Web 界面前使用的轻量访问密码；部署前必须修改 Compose 中的默认值。
- `JM_BOOM_STATIC_DIR`：前端静态资源目录，镜像内默认为 `/app/static`。
- `JM_BOOM_CORS_ORIGINS`：可选的逗号分隔跨域 Origin；同源部署不需要设置。

## NSFW 警告

本软件可能存在裸露、暴力、色情或冒犯等不适宜公众场合的内容，请勿在公共场合使用本软件，避免不必要的纷争。

## 致谢

本项目参考了以下项目的部分实现，在此表示衷心的感谢！

- [Breeze](https://github.com/deretame/Breeze)
- [jmcomic-next](https://github.com/HongShi2333/jmcomic-next)

同时感谢社区 [LinuxDO](https://linux.do) 的帮助。

## 免责声明

本项目仅供学习、研究和技术交流使用。项目作者与任何第三方服务、原始应用或内容提供方无关。
使用者应自行遵守当地法律法规以及相关服务条款。因使用本项目产生的任何法律、版权、账号、数据或财务风险均由使用者自行承担。

## License

遵循 [MIT](./LICENSE) 协议。
