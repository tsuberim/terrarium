# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml rust-toolchain.toml ./
COPY crates ./crates
# Registry + target caches persist across builds (CI GHA cache + local BuildKit).
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin terrarium-server && \
    cp /app/target/release/terrarium-server /app/terrarium-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/terrarium-server /app/terrarium-server
RUN mkdir -p /app/data
ENV LISTEN_ADDR=0.0.0.0:8080
ENV DATABASE_URL=sqlite:///app/data/terrarium.db?mode=rwc
EXPOSE 8080
CMD ["/app/terrarium-server"]
