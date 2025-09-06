FROM rust:1.89.0 AS builder

WORKDIR /app

COPY ./src/ /app/src/
COPY ./Cargo.toml /app/Cargo.toml
COPY ./Cargo.lock /app/Cargo.lock

# RUN cargo install --path .
RUN cargo build --release

FROM debian:trixie-slim

WORKDIR /app

COPY --from=builder /app/target/release/page-viewer /app/page-viewer
COPY config.yaml /app/config.yaml
COPY builtin /app/builtin
COPY data /app/data

RUN chmod +x /app/page-viewer

CMD ["/app/page-viewer"]
