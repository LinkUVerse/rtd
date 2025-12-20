# Build application
#
# Copy in all crates, Cargo.toml and Cargo.lock unmodified,
# and build the application.
FROM rust:1.90-bullseye AS builder
ARG PROFILE=release
ARG GIT_REVISION
ENV GIT_REVISION=$GIT_REVISION
WORKDIR "$WORKDIR/rtd"
RUN apt-get update && apt-get install -y cmake clang libpq5 libpq-dev

COPY Cargo.toml Cargo.lock ./
COPY consensus consensus
COPY crates crates
COPY rtd-execution rtd-execution
COPY external-crates external-crates

RUN cargo build --profile ${PROFILE} \
    --bin rtd-faucet \
    --bin rtd-bridge \
    --bin bridge-indexer \
    --bin deepbook-indexer \
    --bin rtd-bridge-cli \
    --bin rtd \
    --bin rtd-cluster-test \
    --bin rtd-tool

# Production Image
FROM debian:bullseye-slim AS runtime
WORKDIR "$WORKDIR/rtd"

# rtd-tool needs libpq at runtime and uses git when fetching move dependencies during build
RUN apt-get update && apt-get install -y libpq5 libpq-dev ca-certificates git

COPY --from=builder /rtd/target/release/rtd-faucet /usr/local/bin
COPY --from=builder /rtd/target/release/rtd-bridge /usr/local/bin
COPY --from=builder /rtd/target/release/bridge-indexer /usr/local/bin
COPY --from=builder /rtd/target/release/deepbook-indexer /usr/local/bin
COPY --from=builder /rtd/target/release/rtd-bridge-cli /usr/local/bin
COPY --from=builder /rtd/target/release/rtd /usr/local/bin
COPY --from=builder /rtd/target/release/rtd-cluster-test /usr/local/bin
COPY --from=builder /rtd/target/release/rtd-tool /usr/local/bin

ARG BUILD_DATE
ARG GIT_REVISION
LABEL build-date=$BUILD_DATE
LABEL git-revision=$GIT_REVISION
