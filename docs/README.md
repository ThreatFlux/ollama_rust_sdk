# Documentation guide

Start with the README's [quickstart](../README.md#quick-start), then use the focused guide that
matches your task.

## SDK users

- [API coverage](api-coverage.md) maps public client methods to native Ollama endpoints and records
  known limitations.
- [Configuration and reliability](configuration.md) covers endpoints, authentication headers,
  timeouts, retries, errors, streaming, TLS, and production use.
- [Generated API documentation](https://threatflux.github.io/ollama_rust_sdk/) contains public Rust
  types and method-level details.
- [Examples](../examples/) provide runnable generation, streaming, embedding, and tool-calling
  programs.

## Contributors and maintainers

- [Architecture](ARCHITECTURE.md) describes the crate's main layers and design decisions.
- [Contributing](../CONTRIBUTING.md) defines the development and pull-request workflow.
- [Releasing](RELEASING.md) documents release automation and manual recovery.
- [Changelog](CHANGELOG.md) records notable project changes.
- [Security policy](../SECURITY.md) explains private vulnerability reporting.

## Supplemental API material

- [Raw Ollama API curl examples](ollama-api-curl-reference.md) are a server-oriented companion
  reference. They demonstrate HTTP payloads and model-specific scenarios but are not the primary
  SDK guide or an SDK coverage claim.
