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

# stress binary doesn't use GIT_REVISION at compile time, so we deliberately
# don't declare ARG/ENV GIT_REVISION in the builder — keeps the cargo-build
# layer cache key stable across commits when source content is unchanged.
RUN cargo build --locked --profile ${PROFILE} --bin stress

# Production Image
#
# Mirrors the docker/rtd-node and docker/rtd-tools pattern: slim base + only
# the runtime tools consumers actually invoke. Previously this stage used
# `rust:1.92-bullseye` plus a `git clone` of the full repo, producing a
# ~1.2 GB compressed image (vs rtd-node's ~114 MB). The oversized export
# made BuildKit's `#28 exporting to client directory` step prone to gRPC
# transport hangs under PVC/network contention.
FROM debian:bookworm-slim AS runtime
# Runtime needs:
#   - rtd-network-aux mounts its own entry.sh and runs `aws s3 cp` to fetch
#     genesis + keystore, with a `curl` fallback.
#   - rtd-antithesis uses the in-image entrypoint below (curl + bash only).
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl awscli \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /rtd
ARG PROFILE=release
COPY --from=builder /rtd/target/${PROFILE}/stress /usr/local/bin/stress
COPY docker/stress/entrypoint.sh /docker/stress/entrypoint.sh
RUN chmod +x /docker/stress/entrypoint.sh

# rtd-benchmark workloads (composite/adversarial/party/slow/max_package) read
# their Move source packages at runtime via BENCHMARK_MOVE_BASE_DIR (or the
# env!("CARGO_MANIFEST_DIR") fallback in crates/rtd-benchmark/src/workloads/
# mod.rs). The previous runtime `git clone` made these visible incidentally;
# now we ship only what those workloads actually publish. Each workload's
# Move.toml has `Rtd = { local = "../../../../../rtd-framework/packages/
# rtd-framework" }`, which (relative to the Move.toml's parent dir) resolves
# to /rtd/crates/rtd-framework/packages/rtd-framework — so that tree plus its
# move-stdlib dep must also be present. Total cost: ~2 MB.
ENV BENCHMARK_MOVE_BASE_DIR=/rtd/crates/rtd-benchmark
COPY crates/rtd-benchmark/src/workloads/data /rtd/crates/rtd-benchmark/src/workloads/data
COPY crates/rtd-framework/packages/rtd-framework /rtd/crates/rtd-framework/packages/rtd-framework
COPY crates/rtd-framework/packages/move-stdlib /rtd/crates/rtd-framework/packages/move-stdlib

# publish_basics_package callers (shared_counter etc.) read examples/move/basics at runtime.
ENV MOVE_EXAMPLES_DIR=/rtd/examples/move
COPY examples/move/basics /rtd/examples/move/basics

ARG BUILD_DATE
ARG GIT_REVISION
LABEL build-date=$BUILD_DATE
LABEL git-revision=$GIT_REVISION

CMD ["/docker/stress/entrypoint.sh"]
