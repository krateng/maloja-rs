FROM rust:1.83-alpine3.21 AS builder

ARG CARGO_PROFILE=release

WORKDIR /source

# system requirements
RUN apk add --no-cache \
    build-base \
    curl \
    libressl-dev \
    pkgconfig \
    git \
    bash

ENV CARGO_TARGET_DIR=/target

# just build a fake application to build all our dependencies (caching)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --profile ${CARGO_PROFILE}
RUN cargo clean --profile ${CARGO_PROFILE} && rm -rf src

# build full
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src
RUN cargo build --profile ${CARGO_PROFILE}
# copy result to neutral path so we dont need to care about profile in the final container
RUN cp ${CARGO_TARGET_DIR}/$(if [ "$CARGO_PROFILE" = "release" ]; then echo "release"; else echo "debug"; fi)/maloja-rs ${CARGO_TARGET_DIR}/maloja-rs


FROM alpine:3.21 AS dist
ENV CARGO_TERM_COLOR=always

RUN mkdir /data /config /logs
WORKDIR /data
ENV MALOJA_CONFIG_PATH=/config
ENV MALOJA_DATA_PATH=/data
ENV MALOJA_LOG_PATH=/logs

COPY --from=builder /target/maloja-rs /bin

EXPOSE 42010

ENTRYPOINT ["/bin/maloja-rs"]