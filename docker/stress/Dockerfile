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

RUN cargo build --profile ${PROFILE} --bin stress

FROM rust:1.90-bullseye
ARG PROFILE=release
ARG GIT_REVISION
COPY --from=builder /rtd/target/${PROFILE}/stress /usr/local/bin
WORKDIR "$WORKDIR/rtd"

RUN apt-get update && apt-get -y --no-install-recommends install wget \
        iputils-ping netcat procps bind9-host bind9-dnsutils curl iproute2 git ca-certificates awscli

RUN git clone https://github.com/LinkUVerse/rtd.git ; \
        cd rtd ; \
        git checkout $GIT_REVISION ; \
        cd .. ; \
        mv rtd/* .
        
RUN mkdir -p /docker/stress
RUN cp docker/stress/entrypoint.sh /docker/stress/entrypoint.sh
RUN chmod +x /docker/stress/entrypoint.sh
RUN echo ${GIT_REVISION:-$GIT_REVISION} > /var/run/rtd_commit

CMD ["/docker/stress/entrypoint.sh"]
