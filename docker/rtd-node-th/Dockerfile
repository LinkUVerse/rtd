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
ENV USE_TIDEHUNTER=1
RUN cargo build --locked --profile ${PROFILE} \
    --features typed-store/tidehunter,rtd-tool/tideconsole \
    --bin rtd-node --bin rtd --bin rtd-tool --bin stress

# Production Image
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /rtd
# Tidehunter-feature binaries ship at suffixed paths only — no canonical
# aliases, since the storage backend is incompatible with rocksdb and a
# silent swap at /opt/rtd/bin/rtd-node could corrupt unrelated databases.
# Source path is hardcoded to /release/ because the release and bench
# profiles both emit there (bench inherits release's dir-name).
COPY --from=builder /rtd/target/release/rtd-node /opt/rtd/bin/rtd-node-th
COPY --from=builder /rtd/target/release/rtd      /opt/rtd/bin/rtd-th
COPY --from=builder /rtd/target/release/rtd-tool /opt/rtd/bin/rtd-tool-th
COPY --from=builder /rtd/target/release/stress   /opt/rtd/bin/stress-th

ARG BUILD_DATE
ARG GIT_REVISION
LABEL build-date=$BUILD_DATE
LABEL git-revision=$GIT_REVISION
