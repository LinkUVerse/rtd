# Build application
#
# Copy in all crates, Cargo.toml and Cargo.lock unmodified,
# and build the application.
FROM rust:1.90-bullseye AS builder
ARG PROFILE=release
ENV PROFILE=$PROFILE
ARG GIT_REVISION
ENV GIT_REVISION=$GIT_REVISION
WORKDIR "$WORKDIR/rtd"
RUN apt-get update && apt-get install -y cmake clang libpq-dev

COPY Cargo.toml Cargo.lock ./
COPY consensus consensus
COPY crates crates
COPY rtd-execution rtd-execution
COPY external-crates external-crates

RUN cargo build --profile ${PROFILE} --bin rtd-graphql-rpc --features staging

# Production Image
FROM debian:bullseye-slim AS runtime
WORKDIR "$WORKDIR/rtd"
# Both bench and release profiles copy from release dir
RUN mkdir -p /opt/rtd/bin
COPY --from=builder /rtd/target/release/rtd-graphql-rpc /opt/rtd/bin
RUN apt-get update && apt-get install -y libpq5 ca-certificates libpq-dev postgresql

ARG BUILD_DATE
ARG GIT_REVISION
LABEL build-date=$BUILD_DATE
LABEL git-revision=$GIT_REVISION
