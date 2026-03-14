FROM rust:1.94-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/rate-limiting /usr/local/bin/rate-limiting
CMD ["rate-limiting"]
