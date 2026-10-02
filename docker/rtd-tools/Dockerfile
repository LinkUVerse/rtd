# Build application
#
# Single-stage builder + minimal runtime. Layer cache short-circuits when the
# COPY inputs are byte-identical to a previous build.
#
FROM rust:1.96.1-bookworm AS builder
RUN apt-get update \
 && apt-get install -y --no-install-recommends cmake clang libpq5 libpq-dev \
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
RUN cargo build --locked --profile ${PROFILE} \
    --bin rtd-faucet \
    --bin rtd-bridge \
    --bin bridge-indexer \
    --bin rtd-bridge-cli \
    --bin rtd \
    --bin rtd-cluster-test \
    --bin rtd-tool \
    --bin move-analyzer \
    --bin rtd-fork \
    --bin rtd-metric-checker

# Production Image
FROM debian:bookworm-slim AS runtime
# rtd-tool needs libpq at runtime and git when fetching Move dependencies.
# openssh-client is explicit — without it `--no-install-recommends` drops the
# git→ssh-client recommendation, breaking git@/ssh:// Move sources.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates git openssh-client libpq5 libpq-dev \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /rtd
# Source path is hardcoded to /release/ because the bench profile inherits
# release's dir-name.
COPY --from=builder /rtd/target/release/rtd-faucet       /usr/local/bin/rtd-faucet
COPY --from=builder /rtd/target/release/rtd-bridge       /usr/local/bin/rtd-bridge
COPY --from=builder /rtd/target/release/bridge-indexer   /usr/local/bin/bridge-indexer
COPY --from=builder /rtd/target/release/rtd-bridge-cli   /usr/local/bin/rtd-bridge-cli
COPY --from=builder /rtd/target/release/rtd              /usr/local/bin/rtd
COPY --from=builder /rtd/target/release/rtd-cluster-test /usr/local/bin/rtd-cluster-test
COPY --from=builder /rtd/target/release/rtd-tool         /usr/local/bin/rtd-tool
COPY --from=builder /rtd/target/release/move-analyzer    /usr/local/bin/move-analyzer
COPY --from=builder /rtd/target/release/rtd-metric-checker  /usr/local/bin/rtd-metric-checker
COPY --from=builder /rtd/target/release/rtd-fork         /usr/local/bin/rtd-fork

ARG BUILD_DATE
ARG GIT_REVISION
LABEL build-date=$BUILD_DATE
LABEL git-revision=$GIT_REVISION
