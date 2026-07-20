# ThreatFlux Rust Dockerfile
# Multi-stage build for single-crate or workspace-based applications.

FROM rust:1.97.0-bookworm AS rust-base

ARG VERSION=0.0.0
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown
ARG BINARY_NAME=ollama-cli
ARG BINARY_PACKAGE=
ARG SBOM_MANIFEST_PATH=Cargo.toml
ARG OCI_IMAGE_TITLE=Ollama Rust SDK
ARG OCI_IMAGE_DESCRIPTION=Rust SDK and CLI for the Ollama API
ARG OCI_IMAGE_VENDOR=ThreatFlux
ARG OCI_IMAGE_SOURCE=https://github.com/ThreatFlux/ollama_rust_sdk

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates=20230311+deb12u1 \
    pkg-config=1.8.1-1 \
    libssl-dev=3.0.20-1~deb12u2 \
    && rm -rf /var/lib/apt/lists/*

FROM rust-base AS builder

RUN useradd -m -u 1000 builder
USER builder
WORKDIR /build

COPY --chown=builder:builder . .

RUN if [ -n "${BINARY_PACKAGE}" ]; then \
      cargo build --release -p "${BINARY_PACKAGE}" --bin "${BINARY_NAME}" --all-features; \
    else \
      cargo build --release --bin "${BINARY_NAME}" --all-features || cargo build --release --all-features; \
    fi

RUN cargo install cargo-cyclonedx --locked --version 0.5.8 && \
    cargo cyclonedx \
      --manifest-path "${SBOM_MANIFEST_PATH}" \
      --all-features \
      --format json \
      --spec-version 1.5 \
      --override-filename "${BINARY_NAME}-sbom"

FROM debian:bookworm-slim AS runtime

ARG VERSION=0.0.0
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown
ARG BINARY_NAME=ollama-cli
ARG OCI_IMAGE_TITLE=Ollama Rust SDK
ARG OCI_IMAGE_DESCRIPTION=Rust SDK and CLI for the Ollama API
ARG OCI_IMAGE_VENDOR=ThreatFlux
ARG OCI_IMAGE_SOURCE=https://github.com/ThreatFlux/ollama_rust_sdk

LABEL org.opencontainers.image.title="${OCI_IMAGE_TITLE}" \
      org.opencontainers.image.description="${OCI_IMAGE_DESCRIPTION}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.created="${BUILD_DATE}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.vendor="${OCI_IMAGE_VENDOR}" \
      org.opencontainers.image.source="${OCI_IMAGE_SOURCE}"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates=20230311+deb12u1 \
    libssl3=3.0.20-1~deb12u2 \
    tini=0.19.0-1+b3 \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /usr/share/doc/app \
    && useradd -m -u 1000 app

COPY --from=builder /build/target/release/${BINARY_NAME} /usr/local/bin/ollama-cli
COPY --from=builder /build/${BINARY_NAME}-sbom.json /usr/share/doc/app/sbom.cdx.json

RUN chown -R app:app /usr/local/bin/ollama-cli /usr/share/doc/app

USER app
WORKDIR /home/app

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/ollama-cli"]
