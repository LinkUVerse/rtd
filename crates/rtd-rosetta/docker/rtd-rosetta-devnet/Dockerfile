FROM ubuntu:latest AS chef
WORKDIR rtd
ARG GIT_REVISION
ENV GIT_REVISION=$GIT_REVISION
RUN apt-get update && apt-get install -y build-essential libssl-dev pkg-config curl cmake clang git ca-certificates
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

FROM chef AS builder
COPY Cargo.toml Cargo.lock ./
COPY consensus consensus
COPY crates crates
COPY rtd-execution rtd-execution
COPY external-crates external-crates

RUN curl -sSfL https://raw.githubusercontent.com/coinbase/rosetta-cli/master/scripts/install.sh | sh -s
ARG RTD_GENESIS_BLOB_URL
ARG RTD_GENESIS_BLOB_SHA256
RUN test -n "$RTD_GENESIS_BLOB_URL" || (echo "Set RTD_GENESIS_BLOB_URL to a verified RTD genesis blob URL" >&2; exit 1)
RUN test -n "$RTD_GENESIS_BLOB_SHA256" || (echo "Set RTD_GENESIS_BLOB_SHA256 to the verified genesis blob digest" >&2; exit 1)
RUN curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL "$RTD_GENESIS_BLOB_URL" -o genesis.blob
RUN echo "$RTD_GENESIS_BLOB_SHA256  genesis.blob" | sha256sum -c -
RUN cargo build --release --bin rtd --bin rtd-rosetta --bin rtd-node

# Production Image
FROM ubuntu:latest AS runtime
WORKDIR rtd
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /rtd/target/release/rtd /usr/local/bin
COPY --from=builder /rtd/target/release/rtd-node /usr/local/bin
COPY --from=builder /rtd/target/release/rtd-rosetta /usr/local/bin
COPY --from=builder /rtd/bin/rosetta-cli /usr/local/bin
COPY --from=builder /rtd/crates/rtd-config/data/fullnode-template.yaml /rtd/devnet/fullnode.yaml
COPY --from=builder /rtd/genesis.blob /rtd/devnet/genesis.blob
RUN /usr/local/bin/rtd genesis

ARG BUILD_DATE
LABEL build-date=$BUILD_DATE
LABEL git-revision=$GIT_REVISION
