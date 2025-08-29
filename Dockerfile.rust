FROM rust:1.89.0 as builder

WORKDIR /app

COPY . .

RUN cargo install --path .
RUN cargo build

FROM debian:trixie-slim

WORKDIR /app

COPY --from=builder /app/target/release/page-viewer /app/page-viewer
COPY config.yaml /app/config.yaml
COPY builtin /app/builtin
COPY data /app/data

RUN chmod +x /app/page-viewer

CMD ["/app/page-viewer"]
