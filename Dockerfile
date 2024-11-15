FROM rust:1.82 AS builder

RUN mkdir -p /var/log/pods

RUN apt-get update && apt-get install -y lld clang

COPY ./rs ./rs
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release

FROM debian:bookworm-slim AS logd
COPY --from=builder /target/release/logd /logd
CMD ["/logd"]

FROM debian:bookworm-slim AS watchd
COPY --from=builder /target/release/watchd /watchd
CMD ["/watchd"]
