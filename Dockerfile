FROM docker.io/threatflux/rust-cicd-template:base-rust-latest AS builder

ARG RUST_TOOLCHAIN=stable
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:$PATH
USER root
WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev curl && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain ${RUST_TOOLCHAIN} --profile minimal

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release && rm -rf src

COPY . .
RUN touch src/main.rs src/lib.rs && cargo build --release --bin ollama-cli

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ollama-cli /usr/local/bin/ollama-cli

ENTRYPOINT ["ollama-cli"]
