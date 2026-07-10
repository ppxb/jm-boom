# 多阶段构建
FROM rust:1.75-slim as server-builder

WORKDIR /build
COPY server /build/server
RUN cd server && cargo build --release

FROM node:20-slim as web-builder

WORKDIR /build
COPY web /build/web
RUN cd web && npm install && npm run build

# 运行时镜像
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制编译产物
COPY --from=server-builder /build/server/target/release/jm-boom-server /app/
COPY --from=web-builder /build/web/dist /app/static

# 创建数据目录
RUN mkdir -p /app/data

VOLUME ["/app/data"]
EXPOSE 3000

ENV RUST_LOG=info

CMD ["/app/jm-boom-server"]
