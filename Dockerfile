# syntax=docker/dockerfile:1
FROM rust:1.93.0-trixie AS builder

WORKDIR /usr/src/app

ENV SQLX_OFFLINE=true

RUN --mount=type=bind,source=.sqlx,target=.sqlx \
    --mount=type=bind,source=crates,target=crates \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release

FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /usr/src/app/target/release/app /usr/local/bin/app

EXPOSE 8080

CMD ["/usr/local/bin/app"]
