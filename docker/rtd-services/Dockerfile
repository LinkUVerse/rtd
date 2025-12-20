FROM rust:1.90-bullseye AS chef
WORKDIR rtd
ARG GIT_REVISION
ENV GIT_REVISION=$GIT_REVISION
RUN apt-get update && apt-get install -y cmake clang libpq5 ca-certificates libpq-dev postgresql

# Build application
FROM chef AS builder 

COPY Cargo.toml Cargo.lock ./
COPY consensus consensus
COPY crates crates
COPY rtd-execution rtd-execution
COPY external-crates external-crates
RUN cargo build --release \
    --bin rtd-oracle \
    --bin rtdns-indexer
RUN mkdir /rtd/bin/
RUN find /rtd/target/release/ -maxdepth 1 -type f -executable -print 
RUN find /rtd/target/release/ -maxdepth 1 -type f -executable -print | xargs cp -t /rtd/bin/

# Production Image
FROM debian:bullseye-slim AS runtime
WORKDIR rtd
COPY --from=builder /rtd/bin /tmp/rtd-bin
RUN find /tmp/rtd-bin -maxdepth 1 -type f -executable -print | xargs cp -t /usr/local/bin/
RUN rm -rf /tmp/rtd-bin
RUN apt update && apt install -y libpq5 libpq-dev postgresql

ARG BUILD_DATE
ARG GIT_REVISION
LABEL build-date=$BUILD_DATE
LABEL git-revision=$GIT_REVISION

