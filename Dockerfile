FROM rust:1.76 AS builder

RUN apt-get update && apt-get install -y lld clang

COPY ./rs ./rs
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --bin logd --release

FROM debian:bookworm-slim
COPY --from=builder /target/release/logd /logd

CMD ["/logd"]
