FROM rust:1.96.1-bookworm AS builder
RUN apt-get update \
 && apt-get install -y --no-install-recommends cmake clang \
 && rm -rf /var/lib/apt/lists/*
ARG PROFILE=release
WORKDIR /work

# COPYs ordered least- to most-frequently-changing for deeper layer cache hits.
COPY rust-toolchain.toml ./
COPY .cargo .cargo
COPY Cargo.toml Cargo.lock ./
COPY external-crates external-crates
COPY rtd-execution rtd-execution
COPY consensus consensus
COPY crates crates

# rtd-checkpoint-blob-indexer doesn't use GIT_REVISION at compile time, so we
# deliberately don't declare ARG/ENV GIT_REVISION in the builder — keeps the
# cargo-build layer cache key stable across commits with identical source.
RUN cargo build --locked --profile ${PROFILE} --bin rtd-checkpoint-blob-indexer

FROM debian:12-slim AS deploy
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates libjemalloc-dev \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder --chmod=755 /work/target/release/rtd-checkpoint-blob-indexer /opt/rtd/bin/rtd-checkpoint-blob-indexer
