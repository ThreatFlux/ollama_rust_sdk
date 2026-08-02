# Ollama Rust SDK

[![Release](https://img.shields.io/github/v/release/ThreatFlux/ollama_rust_sdk)](https://github.com/ThreatFlux/ollama_rust_sdk/releases/latest)
[![Documentation](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://threatflux.github.io/ollama_rust_sdk/)
[![CI](https://github.com/ThreatFlux/ollama_rust_sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/ThreatFlux/ollama_rust_sdk/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.97.1-blue.svg)](Cargo.toml)
[![License](https://img.shields.io/github/license/ThreatFlux/ollama_rust_sdk)](LICENSE)

An async, type-safe Rust client and CLI for the Ollama API, with builders for generation and
chat, typed embeddings, streaming responses, and model-management operations.

> [!NOTE]
> This is a community-maintained project. It is not affiliated with, endorsed by, or maintained
> by Ollama. The project is pre-1.0 and is currently distributed through GitHub rather than
> crates.io; pin a release tag or commit when reproducible builds matter.

[API documentation](https://threatflux.github.io/ollama_rust_sdk/) · [Guides](docs/) ·
[Examples](examples/) · [API coverage](docs/api-coverage.md) ·
[Configuration](docs/configuration.md) · [Architecture](docs/ARCHITECTURE.md) ·
[Changelog](docs/CHANGELOG.md) · [Security](SECURITY.md)

## Why this SDK?

- **Ergonomic requests:** builder APIs cover common generation, chat, tool, and embedding options.
- **Typed streaming:** chat and generation return `Stream` implementations with typed chunks and
  SDK errors.
- **Local-first configuration:** connect to a local Ollama server by default or configure a trusted
  remote endpoint and headers.
- **Operational APIs:** inspect server health and version, manage models, and work with model blobs.

The Ollama API evolves independently from this crate. Use the dated
[coverage matrix](docs/api-coverage.md) for the exact implemented surface instead of assuming that
every Ollama endpoint is available.

## Quick start

### Requirements

- Rust **1.97.1** or newer for the current default branch (the minimum supported Rust version, or
  MSRV)
- A running [Ollama server](https://docs.ollama.com/quickstart)
- A model available to that server; `ollama ls` shows locally installed models
- Tokio with its macros and multithreaded runtime enabled

This crate is not published to crates.io. For a reproducible dependency, choose
a release tag from the [GitHub Releases page](https://github.com/ThreatFlux/ollama_rust_sdk/releases),
record it in your project, and substitute it below:

```bash
export OLLAMA_RUST_SDK_TAG="replace-with-a-release-tag"
cargo add ollama_rust_sdk --git https://github.com/ThreatFlux/ollama_rust_sdk.git --tag "$OLLAMA_RUST_SDK_TAG"
cargo add tokio --features macros,rt-multi-thread
```

For a dependency pinned to an audited commit rather than a tag, replace the
first `cargo add` command above with:

```bash
export OLLAMA_RUST_SDK_REV="replace-with-a-full-commit-sha"
cargo add ollama_rust_sdk --git https://github.com/ThreatFlux/ollama_rust_sdk.git --rev "$OLLAMA_RUST_SDK_REV"
```

The placeholders are intentional: release automation cannot silently make
this README select a different version for your application.

To test unreleased `main` explicitly:

```bash
cargo add ollama_rust_sdk --git https://github.com/ThreatFlux/ollama_rust_sdk.git --branch main
cargo add tokio --features macros,rt-multi-thread
```

Choose a model already available to your server. The base URL can be omitted for a local server:

```bash
export OLLAMA_MODEL="your-installed-model"
export OLLAMA_BASE_URL="http://127.0.0.1:11434"
```

PowerShell users can use `$Env:OLLAMA_MODEL = "your-installed-model"`.

Create `src/main.rs`:

<!-- BEGIN QUICKSTART -->
```rust
use ollama_rust_sdk::OllamaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::from_env()?;
    let model = std::env::var("OLLAMA_MODEL")?;

    let response = client
        .generate()
        .model(model)
        .prompt("Explain Rust's ownership model in one concise sentence.")
        .send()
        .await?;

    println!("{}", response.response);
    Ok(())
}
```
<!-- END QUICKSTART -->

Run it:

```bash
cargo run
```

The complete source is kept in [`examples/quickstart.rs`](examples/quickstart.rs) and compiled by
documentation CI.

## API coverage

This is an implementation summary, not a claim of complete Ollama API coverage.

| Area | Implemented surface | Start here |
| --- | --- | --- |
| Server | Health and version | `OllamaClient::health`, `OllamaClient::version` |
| Generation | Non-streaming and streaming generation | [`quickstart.rs`](examples/quickstart.rs) |
| Chat and tools | Non-streaming and streaming chat; tool request/response types | [`streaming_chat.rs`](examples/streaming_chat.rs), [`tool_calling.rs`](examples/tool_calling.rs) |
| Embeddings | Single and batch input through `/api/embed` | [`embeddings.rs`](examples/embeddings.rs) |
| Models | List, show, pull, create, copy, delete, and list running models | `OllamaClient` model methods |
| Blobs | Check and upload blobs | `OllamaClient::blob_exists`, `OllamaClient::create_blob` |

See [API coverage](docs/api-coverage.md) for endpoint mappings, streaming details, and known gaps.
Ollama itself offers [partial OpenAI API compatibility](https://docs.ollama.com/api/openai-compatibility),
but this crate does not expose dedicated `/v1` OpenAI-compatible client methods.

## Common examples

| Goal | Example | Command |
| --- | --- | --- |
| Make a minimal generation request | [`quickstart.rs`](examples/quickstart.rs) | `cargo run --example quickstart` |
| Generate text and inspect timings | [`basic_generation.rs`](examples/basic_generation.rs) | `cargo run --example basic_generation` |
| Stream a chat response | [`streaming_chat.rs`](examples/streaming_chat.rs) | `cargo run --example streaming_chat` |
| Create and compare embeddings | [`embeddings.rs`](examples/embeddings.rs) | `cargo run --example embeddings` |
| Complete a tool-calling loop | [`tool_calling.rs`](examples/tool_calling.rs) | `cargo run --example tool_calling` |
| Run tools concurrently | [`parallel_tools.rs`](examples/parallel_tools.rs) | `cargo run --example parallel_tools` |

Examples make live network requests and may require a model with specific capabilities. Read the
example source and update its model name for your Ollama installation before running it.

## CLI

The repository includes the `ollama-cli` binary:

```bash
# List models known to the configured server
cargo run --bin ollama-cli -- models list

# Generate text with a selected model
cargo run --bin ollama-cli -- generate --model "$OLLAMA_MODEL" "Why is the sky blue?"

# Start an interactive chat
cargo run --bin ollama-cli -- chat --model "$OLLAMA_MODEL"
```

Run `cargo run --bin ollama-cli -- --help` for the complete command list. The CLI defaults to
`http://localhost:11434`; override it with the global `--url` option.

## Cargo features

| Feature | Default | What it enables |
| --- | --- | --- |
| `default` | Yes | Enables the `tls` feature |
| `tls` | Yes | Enables Reqwest's Rustls integration and the optional `rustls` dependency |
| `tracing` | No | Makes the optional `tracing` dependency available; the SDK does not yet emit tracing events |

Because Reqwest's default features remain enabled, `tls` is not currently a mutually exclusive TLS
backend selector. Test `--no-default-features` in your own dependency graph before relying on a
reduced feature set.

## Configuration and reliability

- `OllamaClient::from_env()` defaults to `http://127.0.0.1:11434` and a two-minute timeout.
- `OLLAMA_BASE_URL` should be a server origin such as `https://ollama.example.com`, without an
  `/api` suffix. API paths are added by the SDK.
- The default local endpoint uses plaintext HTTP and the client sends no authentication header.
  Only connect to endpoints you trust; add authentication headers when required by your server.
- `OLLAMA_API_HEADERS` can contain credentials. Keep it out of source control and diagnostic logs.
- The client enforces a request timeout, but does **not** currently perform automatic retries.
  `max_retries` and `retry_delay` are stored configuration values only.
- `OllamaError::is_retryable()` classifies selected failures; callers still own backoff, retry
  limits, cancellation, and mutation safety.

See [Configuration and reliability](docs/configuration.md) for all environment variables, custom
configuration, error behavior, streaming caveats, and production guidance.

## Development

The documentation contract requires Python 3.11 or newer.

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo doc --locked --all-features --no-deps
python3 scripts/check_docs.py
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full development workflow.

## Support and security

- Use [GitHub Issues](https://github.com/ThreatFlux/ollama_rust_sdk/issues) for reproducible SDK
  bugs and feature requests.
- Follow [SECURITY.md](SECURITY.md) to report vulnerabilities privately; do not open a public
  security issue.
- Ollama installation, model, account, and service questions belong with the
  [Ollama documentation](https://docs.ollama.com/).

## License

Licensed under the [MIT License](LICENSE).
