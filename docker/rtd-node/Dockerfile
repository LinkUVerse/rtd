# Build application
#
# Copy in all crates, Cargo.toml and Cargo.lock unmodified,
# and build the application.
FROM rust:1.90-bullseye AS builder
ARG PROFILE=release
ARG GIT_REVISION
ENV GIT_REVISION=$GIT_REVISION
WORKDIR "$WORKDIR/rtd"
RUN apt-get update && apt-get install -y cmake clang

COPY Cargo.toml Cargo.lock ./
COPY consensus consensus
COPY crates crates
COPY rtd-execution rtd-execution
COPY external-crates external-crates

RUN cargo build --profile ${PROFILE} --bin rtd-node

# Production Image
FROM debian:bullseye-slim AS runtime
# Use jemalloc as memory allocator
RUN apt-get update && apt-get install -y ca-certificates curl
ARG PROFILE=release
WORKDIR "$WORKDIR/rtd"
# Both bench and release profiles copy from release dir
COPY --from=builder /rtd/target/release/rtd-node /opt/rtd/bin/rtd-node
# Support legacy usages of /usr/local/bin/rtd-node
COPY --from=builder /rtd/target/release/rtd-node /usr/local/bin

ARG BUILD_DATE
ARG GIT_REVISION
LABEL build-date=$BUILD_DATE
LABEL git-revision=$GIT_REVISION
