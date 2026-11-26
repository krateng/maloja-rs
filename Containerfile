FROM rust:alpine3.24 AS builder

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
ENV RUSTFLAGS="-A warnings"

# just build a fake application to build all our dependencies (caching)
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --profile ${CARGO_PROFILE}
RUN cargo clean --profile ${CARGO_PROFILE} && rm -rf src

# build full
COPY askama.toml ./
COPY src/ ./src/
COPY web/ ./web/
RUN cargo build --profile ${CARGO_PROFILE}
# copy result to neutral path so we dont need to care about profile in the final container
RUN cp ${CARGO_TARGET_DIR}/$(if [ "$CARGO_PROFILE" = "release" ]; then echo "release"; else echo "debug"; fi)/maloja-rs ${CARGO_TARGET_DIR}/maloja-rs


FROM alpine:3.24 AS dist

RUN install -o 1000 -d -m 775 /data /config /logs
ENV MALOJA_CONFIG_PATH=/config
ENV MALOJA_DATA_PATH=/data
ENV MALOJA_LOG_PATH=/logs

COPY --from=builder /target/maloja-rs /bin/maloja-rs

USER 1000:1000

EXPOSE 42010

ENTRYPOINT ["/bin/maloja-rs"]