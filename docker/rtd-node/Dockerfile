# Build application
#
# Single-stage builder + minimal runtime. Layer cache short-circuits when the
# COPY inputs are byte-identical to a previous build.
#
FROM rust:1.96.1-bookworm AS builder
RUN apt-get update \
 && apt-get install -y --no-install-recommends cmake clang \
 && rm -rf /var/lib/apt/lists/*
ARG PROFILE=release
WORKDIR /rtd

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
RUN cargo build --locked --profile ${PROFILE} --bin rtd-node

# Production Image
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /rtd
ARG PROFILE=release
COPY --from=builder /rtd/target/${PROFILE}/rtd-node /opt/rtd/bin/rtd-node
COPY --from=builder /rtd/target/${PROFILE}/rtd-node /usr/local/bin/rtd-node

ARG BUILD_DATE
ARG GIT_REVISION
LABEL build-date=$BUILD_DATE
LABEL git-revision=$GIT_REVISION
