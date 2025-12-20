FROM rust:1.90-bullseye as build

ARG PROFILE=release
WORKDIR /work

RUN apt-get update && apt-get install -y cmake clang

COPY .git/ .git/
COPY Cargo.toml Cargo.lock ./
COPY consensus consensus
COPY crates crates
COPY rtd-execution rtd-execution
COPY external-crates external-crates

RUN cargo build --profile ${PROFILE} --bin rtd-kv-rpc

FROM gcr.io/distroless/cc-debian12 as deploy

COPY --from=build --chmod=755 /work/target/release/rtd-kv-rpc /opt/rtd/bin/rtd-kv-rpc
