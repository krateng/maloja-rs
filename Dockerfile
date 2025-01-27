FROM rust:1.83-alpine3.21 AS builder

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
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
RUN cargo clean --release && rm -rf src

# build full
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src
RUN cargo build --release


FROM alpine:3.21 AS dist
ENV CARGO_TERM_COLOR=always

RUN mkdir /data
WORKDIR /data
COPY --from=builder /target/release/maloja-rs /bin

