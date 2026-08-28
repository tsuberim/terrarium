# Multi-stage build for the native Terrarium host (Cloud Run).
# Cheap: one small always-on service. Skin is a client; host owns World.

FROM rust:1.83-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p terrarium-host

FROM debian:bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/terrarium-host /usr/local/bin/terrarium-host
COPY apps/skin /skin
RUN rm -rf /skin/pkg
ENV SKIN_DIR=/skin
ENV PORT=8080
ENV TERRARIUM_ENV=staging
EXPOSE 8080
USER nobody
CMD ["terrarium-host"]
