# Build application
#
# Copy in all crates, Cargo.toml and Cargo.lock unmodified,
# and build the application.
ARG PROFILE=release-unwind
FROM rust:1.96.1-bookworm AS builder
ARG PROFILE
ARG FEATURES=""
WORKDIR "$WORKDIR/rtd"

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates cmake clang \
 && rm -rf /var/lib/apt/lists/*

# COPYs ordered least- to most-frequently-changing for deeper layer cache hits.
COPY rust-toolchain.toml ./
COPY .cargo .cargo
COPY Cargo.toml Cargo.lock ./
COPY external-crates external-crates
COPY rtd-execution rtd-execution
COPY consensus consensus
COPY crates crates

# GIT_REVISION declared late so SHA changes don't invalidate the COPYs above.
ARG GIT_REVISION
ENV GIT_REVISION=$GIT_REVISION
RUN if [ -n "$FEATURES" ]; then \
        cargo build --locked --profile ${PROFILE} --bin rtd-indexer-alt-graphql --features "$FEATURES"; \
    else \
        cargo build --locked --profile ${PROFILE} --bin rtd-indexer-alt-graphql; \
    fi

# Production Image
FROM debian:bookworm-slim AS runtime
ARG PROFILE
# Use jemalloc as memory allocator
RUN apt-get update \
 && apt-get install -y --no-install-recommends libjemalloc-dev ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*
ENV LD_PRELOAD /usr/lib/x86_64-linux-gnu/libjemalloc.so
WORKDIR "$WORKDIR/rtd"
COPY --from=builder /rtd/target/${PROFILE}/rtd-indexer-alt-graphql /usr/local/bin

ARG BUILD_DATE
ARG GIT_REVISION
LABEL build-date=$BUILD_DATE
LABEL git-revision=$GIT_REVISION
ENV GIT_REVISION=$GIT_REVISION
