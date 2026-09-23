FROM ubuntu:latest AS chef
WORKDIR rtd
ARG GIT_REVISION
ENV GIT_REVISION=$GIT_REVISION
RUN apt-get update && apt-get install -y build-essential libssl-dev pkg-config curl cmake clang ca-certificates
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Build application
FROM chef AS builder

RUN curl -sSfL https://raw.githubusercontent.com/coinbase/rosetta-cli/master/scripts/install.sh | sh -s

COPY Cargo.toml Cargo.lock ./
COPY consensus consensus
COPY crates crates
COPY rtd-execution rtd-execution
COPY external-crates external-crates
RUN cargo build --release --bin rtd --bin rtd-rosetta

# Production Image
FROM ubuntu:latest AS runtime
WORKDIR rtd
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /rtd/target/release/rtd /usr/local/bin
COPY --from=builder /rtd/target/release/rtd-rosetta /usr/local/bin
COPY --from=builder /rtd/bin/rosetta-cli /usr/local/bin
COPY --from=builder /rtd/crates/rtd-config/data/fullnode-template.yaml /rtd/devnet/fullnode.yaml
RUN /usr/local/bin/rtd genesis

ARG BUILD_DATE
LABEL build-date=$BUILD_DATE
LABEL git-revision=$GIT_REVISION
