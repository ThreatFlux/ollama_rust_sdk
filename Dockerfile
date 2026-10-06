# ThreatFlux Rust Dockerfile
# Multi-stage build for single-crate or workspace-based applications.
#
# Follows ThreatFlux/rust-cicd-template: a Debian 13 (trixie) Rust builder and
# a distroless Debian 13 runtime with no shell, package manager or coreutils.
# The CLI only needs glibc, libgcc and the CA bundle at runtime (TLS is rustls),
# which gcr.io/distroless/cc-debian13 provides.
#
# Base images are pinned by digest for reproducibility (Scorecard Pinned-Dependencies).
# Refresh with: docker buildx imagetools inspect <image> | awk '/^Digest:/{print $2}'
# Dependabot refreshes the Rust builder; refresh the runtime digest with the
# command above when the template moves.

FROM rust:1.99.0-trixie@sha256:15ad267e7a4cb2dce5905c90c76765adb6714945c5ea6d7c82673897a5e4067b AS rust-base

ARG VERSION=0.0.0
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown
ARG BINARY_NAME=ollama-cli
ARG BINARY_PACKAGE=
ARG SBOM_MANIFEST_PATH=Cargo.toml
ARG OCI_IMAGE_TITLE="Ollama Rust SDK"
ARG OCI_IMAGE_DESCRIPTION="Rust SDK and CLI for the Ollama API"
ARG OCI_IMAGE_VENDOR=ThreatFlux
ARG OCI_IMAGE_SOURCE=https://github.com/ThreatFlux/ollama_rust_sdk

# tini is installed here so the runtime stage can copy it out: distroless ships
# no init, and PID 1 must reap zombies and forward signals. Exact Debian package
# revisions are not pinned: point releases drop superseded revisions from the
# archive, and the digest-pinned base already fixes the Debian release.
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    pkg-config \
    libssl-dev \
    tini \
    && rm -rf /var/lib/apt/lists/*

FROM rust-base AS builder

RUN useradd -m -u 1000 builder
USER builder
WORKDIR /build

ENV CARGO_HOME=/home/builder/.cargo
ENV PATH="/home/builder/.cargo/bin:${PATH}"

COPY --chown=builder:builder . .

RUN rustc --version --verbose && cargo --version && \
    if [ -n "${BINARY_PACKAGE}" ]; then \
      cargo build --locked --release -p "${BINARY_PACKAGE}" --bin "${BINARY_NAME}" --all-features; \
    else \
      cargo build --locked --release --bin "${BINARY_NAME}" --all-features || cargo build --locked --release --all-features; \
    fi

# cargo-cyclonedx writes the SBOM beside the manifest it was handed, which in a
# workspace is not necessarily /build. The find normalizes the output location.
RUN cargo install cargo-cyclonedx --locked --version 0.5.9 && \
    cargo cyclonedx \
      --manifest-path "${SBOM_MANIFEST_PATH}" \
      --all-features \
      --format json \
      --spec-version 1.5 \
      --override-filename "${BINARY_NAME}-sbom" && \
    find /build -name "${BINARY_NAME}-sbom.json" -exec cp {} /build/sbom.cdx.json \; -quit && \
    test -s /build/sbom.cdx.json

# Stage the runtime files: distroless has no shell to copy or rename them with.
RUN mkdir -p /home/builder/out/bin /home/builder/out/doc && \
    cp "target/release/${BINARY_NAME}" /home/builder/out/bin/ollama-cli && \
    cp /build/sbom.cdx.json /home/builder/out/doc/sbom.cdx.json

FROM gcr.io/distroless/cc-debian13:nonroot@sha256:e792ab3d241a468a4fd7519ddbbebe66b49b5f365771716ea688ad40b6c6f1c2 AS runtime

ARG VERSION=0.0.0
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown
ARG OCI_IMAGE_TITLE="Ollama Rust SDK"
ARG OCI_IMAGE_DESCRIPTION="Rust SDK and CLI for the Ollama API"
ARG OCI_IMAGE_VENDOR=ThreatFlux
ARG OCI_IMAGE_SOURCE=https://github.com/ThreatFlux/ollama_rust_sdk
ARG OCI_IMAGE_LICENSES=MIT

LABEL org.opencontainers.image.title="${OCI_IMAGE_TITLE}" \
      org.opencontainers.image.description="${OCI_IMAGE_DESCRIPTION}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.created="${BUILD_DATE}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.vendor="${OCI_IMAGE_VENDOR}" \
      org.opencontainers.image.source="${OCI_IMAGE_SOURCE}" \
      org.opencontainers.image.licenses="${OCI_IMAGE_LICENSES}"

# COPY --from keeps the builder's ownership, so set root explicitly: the binary,
# init and SBOM stay read-only for the runtime user.
COPY --from=builder --chown=0:0 /usr/bin/tini /usr/bin/tini
COPY --from=builder --chown=0:0 /home/builder/out/bin/ollama-cli /usr/local/bin/ollama-cli
COPY --from=builder --chown=0:0 /home/builder/out/doc/sbom.cdx.json /usr/share/doc/app/sbom.cdx.json

# distroless "nonroot" user and its home directory.
USER 65532:65532
WORKDIR /home/nonroot

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/ollama-cli"]
