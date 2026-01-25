# syntax=docker/dockerfile:1
FROM rust:1.93.0-trixie AS backend-builder

WORKDIR /usr/src/app

ENV SQLX_OFFLINE=true

RUN --mount=type=bind,source=.sqlx,target=.sqlx \
    --mount=type=bind,source=crates,target=crates \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/usr/src/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && cp target/release/app /tmp/app

FROM denoland/deno:alpine-2.6.6 AS frontend-builder

WORKDIR /usr/src/app

COPY . .

RUN deno task build

FROM caddy:2.10.2-alpine

COPY --from=backend-builder /tmp/app /usr/local/bin/app
COPY --from=frontend-builder /usr/src/app/dist /usr/share/caddy
COPY Caddyfile /etc/caddy/Caddyfile

RUN /usr/local/bin/app &

CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile"]
