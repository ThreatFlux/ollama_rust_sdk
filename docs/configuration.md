# Configuration and reliability

The SDK is local-first, but it can connect to any trusted endpoint that serves compatible native
Ollama `/api` routes. Configuration controls transport behavior; model availability and capabilities
remain properties of the selected server.

## Client construction

Use an explicit base URL when application configuration already owns endpoint selection:

```rust
use ollama_rust_sdk::OllamaClient;

let client = OllamaClient::new("http://127.0.0.1:11434")?;
# Ok::<(), ollama_rust_sdk::OllamaError>(())
```

Use `from_env()` for deploy-time configuration:

```rust
use ollama_rust_sdk::OllamaClient;

let client = OllamaClient::from_env()?;
# Ok::<(), ollama_rust_sdk::OllamaError>(())
```

For full programmatic control, build `ClientConfig`:

```rust
use std::time::Duration;
use ollama_rust_sdk::{ClientConfig, OllamaClient};

let config = ClientConfig::builder()
    .base_url("https://ollama.example.com")
    .timeout(Duration::from_secs(90))
    .follow_redirects(false)
    .header("Authorization", "Bearer replace-me")
    .build()?;
let client = OllamaClient::with_config(config)?;
# Ok::<(), ollama_rust_sdk::OllamaError>(())
```

Avoid hard-coding credentials as shown in the compact example. Read them from a secret manager or
protected environment variable and keep them out of source control and logs.

## Environment variables

`OllamaClient::from_env()` reads these SDK-specific variables:

| Variable | Meaning | Default or behavior |
| --- | --- | --- |
| `OLLAMA_BASE_URL` | Server origin used for native API calls | `http://127.0.0.1:11434` |
| `OLLAMA_TIMEOUT_SECS` | Whole-request timeout in seconds | `120`; invalid integers return `ConfigError` |
| `OLLAMA_USER_AGENT` | HTTP User-Agent override | `ollama-rust-sdk/<crate-version>`; blank values are ignored |
| `OLLAMA_API_HEADERS` | JSON object of additional HTTP headers | No additional headers |

Example custom headers:

```bash
export OLLAMA_API_HEADERS='{"Authorization":"Bearer replace-me","X-Tenant":"example"}'
```

Header values that are not JSON strings are converted to their JSON representation. Prefer strings
for predictable wire values.

`OLLAMA_MODEL` is used by this repository's quickstart example; it is not read by the SDK itself.
The SDK also does not read `OLLAMA_API_KEY` automatically. Consult
[Ollama authentication](https://docs.ollama.com/api/authentication) for the endpoint you use, then
provide any required authorization through a custom header.

## Base URL rules

Pass a server origin, not an API endpoint:

- Use `http://127.0.0.1:11434` for a default local server.
- Use an HTTPS origin such as `https://ollama.example.com` for a protected remote deployment.
- Do not append `/api`; the SDK adds endpoint paths such as `/api/generate`.
- Avoid path-prefixed base URLs. Endpoint paths begin with `/` and replace an existing URL path.

The URL must parse successfully, but construction does not contact the server. Use `health()` or
`version()` for an explicit startup probe.

## Authentication and endpoint trust

The default local URL is plaintext HTTP, and the SDK sends no authentication header unless you add
one. This is appropriate only within a trusted local or private network boundary.

For remote deployments:

- Prefer HTTPS and validate the deployment's certificate and hostname strategy.
- Add the authentication headers required by the server or reverse proxy.
- Treat `OLLAMA_API_HEADERS` as a secret when it contains credentials.
- Disable redirects with `follow_redirects(false)` when redirects are not part of the deployment.
- Do not point a credential-bearing client at user-supplied or untrusted base URLs.

## Timeouts, retries, and cancellation

`ClientConfig::timeout` defaults to two minutes and is passed to Reqwest as a whole-request timeout.
Long model loads, pulls, generation, and streaming sessions may need a larger value. Conversely,
latency-sensitive applications should use a shorter workload-specific deadline.

The SDK does **not** currently retry requests. Although `ClientConfig` contains `max_retries` and
`retry_delay`, the HTTP layer does not consume them. `OllamaError::is_retryable()` is only a
classification helper.

If the application adds retries:

- use bounded exponential backoff with jitter;
- honor the application's overall deadline and cancellation signal;
- distinguish connection failures from deterministic 4xx responses;
- avoid blindly retrying model creation, deletion, blob upload, or other mutations; and
- account for duplicate generation work when a response is lost after the server starts processing.

Dropping a request future or stream is the current cancellation mechanism. The crate does not expose
a server-side cancellation endpoint.

## Errors

Most public methods return `ollama_rust_sdk::Result<T>`, whose error type is `OllamaError`.

| Variant | Typical meaning |
| --- | --- |
| `ConfigError`, `UrlError` | Invalid client or endpoint configuration |
| `NetworkError`, `Timeout` | Transport failure or configured deadline exceeded |
| `ServerError` | Non-success HTTP status with a status code and server message |
| `ModelNotFound`, `ModelLoading` | Model-specific availability condition |
| `InvalidResponse`, `JsonError`, `StreamError` | Response or stream could not be decoded |

Some variants are reserved for richer classification and are not produced by every endpoint. Match a
specific variant only when its source method documents that mapping, and retain a fallback arm for
future variants.

`health()` is intentionally different: it returns `Ok(false)` for transport errors and non-success
statuses. Use `version()` or a normal API request when the original error matters.

## Streaming behavior

Generation and chat streams deserialize newline-delimited JSON into typed chunks. The current parser
processes each transport chunk independently; a JSON record split across chunks can produce
`InvalidResponse`. Applications should:

- handle an error item after a stream was created;
- stop consuming when their own deadline or cancellation signal fires;
- avoid assuming one network chunk equals one semantic response; and
- test streaming against the exact Ollama server and proxy versions used in production.

## Cargo features and TLS

The default `tls` feature enables Reqwest's Rustls integration and the optional direct `rustls`
dependency. Reqwest's own default features are not disabled, so this feature is not currently an
exclusive TLS-backend switch.

The optional `tracing` feature makes the dependency available but the SDK does not currently emit
tracing spans or events. Instrument application call sites until native SDK instrumentation is added.

## Production checklist

- Pin the SDK to a release tag or commit.
- Set an explicit, trusted base URL.
- Use HTTPS and authentication for remote endpoints.
- Choose timeouts per workload and add bounded caller-managed retries only where safe.
- Apply concurrency and resource limits around generation and model-management work.
- Validate streaming through every proxy or load balancer in the request path.
- Record the Ollama server version alongside failures and compatibility reports.
- Keep model names configurable instead of compiling deployment-specific names into the application.
