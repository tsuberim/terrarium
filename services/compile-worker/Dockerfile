# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder
WORKDIR /build
COPY services/compile-worker/Cargo.toml services/compile-worker/Cargo.lock* ./
COPY services/compile-worker/src ./src
RUN cargo build --release

FROM rust:1-bookworm
RUN rustup target add wasm32-unknown-unknown
COPY sdk/rust/terrarium-sdk /opt/terrarium-sdk
COPY services/compile-worker/template /opt/template
COPY --from=builder /build/target/release/compile-worker /usr/local/bin/compile-worker
ENV TEMPLATE_DIR=/opt/template
ENV SDK_PATH=/opt/terrarium-sdk
ENV PORT=8080
EXPOSE 8080
CMD ["compile-worker"]
