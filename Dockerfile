FROM rust:1.96-slim-bookworm AS server-builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential cmake pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build/server
COPY server/Cargo.toml server/Cargo.lock ./
COPY server/migrations ./migrations
COPY server/src ./src
RUN cargo build --release --locked

FROM oven/bun:1.3.14 AS web-builder

WORKDIR /build/web
COPY web/package.json web/bun.lock ./
RUN bun install --frozen-lockfile
COPY web/ ./
RUN bun run build

FROM debian:bookworm-slim AS runtime

ARG VERSION=0.5.1
ARG REVISION=unknown

LABEL org.opencontainers.image.title="JM Boom" \
      org.opencontainers.image.description="JM Boom React Web frontend and Axum backend" \
      org.opencontainers.image.source="https://github.com/ppxb/jm-boom" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${REVISION}"

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 jm-boom \
    && useradd --uid 10001 --gid jm-boom --create-home --home-dir /app jm-boom

WORKDIR /app

COPY --from=server-builder --chown=jm-boom:jm-boom \
    /build/server/target/release/jm-boom-server /app/jm-boom-server
COPY --from=web-builder --chown=jm-boom:jm-boom /build/web/dist /app/static

RUN mkdir -p /app/data && chown -R jm-boom:jm-boom /app

USER jm-boom

ENV RUST_LOG="info,jm_boom_server=info" \
    JM_BOOM_STATIC_DIR="/app/static"

VOLUME ["/app/data"]
EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl --fail --silent http://127.0.0.1:3000/health || exit 1

ENTRYPOINT ["/app/jm-boom-server"]
