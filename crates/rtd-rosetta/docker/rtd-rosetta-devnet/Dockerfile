FROM ubuntu:latest AS chef
WORKDIR rtd
ARG GIT_REVISION
ENV GIT_REVISION=$GIT_REVISION
RUN apt-get update && apt-get install -y build-essential libssl-dev pkg-config curl cmake clang git ca-certificates
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

FROM chef AS builder
RUN git clone https://github.com/LinkUVerse/rtd .
RUN git checkout devnet

RUN curl -sSfL https://raw.githubusercontent.com/coinbase/rosetta-cli/master/scripts/install.sh | sh -s
RUN curl -fLJO https://github.com/LinkUVerse/rtd-genesis/raw/main/devnet/genesis.blob
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
