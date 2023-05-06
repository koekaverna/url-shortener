FROM rust:1.69.0

RUN cargo install cargo-watch

WORKDIR /app

EXPOSE 8000
