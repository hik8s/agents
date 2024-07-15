FROM rust:1.76 AS builder

COPY ./rs ./rs
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --bin logd

CMD ["./logd"]
