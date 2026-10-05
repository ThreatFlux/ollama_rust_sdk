# API coverage

Last audited against public SDK main `f9fab82` on **2026-10-04** and compared with stable Ollama
v0.35.1. See the [verified feature audit and follow-up plan](sdk-feature-audit.md) for provider
evidence, compatibility implications and proposed tests. This modernization change records the
gaps; it does not implement the proposed API additions or fixes.

This matrix records what the crate implements. It is deliberately not a completeness claim about
the independently evolving [Ollama API](https://docs.ollama.com/api/introduction). A row marked
“implemented” means the listed SDK method constructs the endpoint request and has typed success or
error handling; it does not guarantee that every request field or response variant from every
Ollama server version is modeled.

## High-level client

| Ollama area | HTTP endpoint | Public SDK entry point | Mode | Status |
| --- | --- | --- | --- | --- |
| Health | `GET /` | `OllamaClient::health` | Non-streaming | Implemented |
| Version | `GET /api/version` | `OllamaClient::version` | Non-streaming | Implemented |
| Generate | `POST /api/generate` | `OllamaClient::generate` | Streaming and non-streaming | Implemented |
| Chat | `POST /api/chat` | `OllamaClient::chat` | Streaming and non-streaming | Implemented |
| Embeddings | `POST /api/embed` | `OllamaClient::embed` | Non-streaming; single or batch input | Implemented |
| List models | `GET /api/tags` | `OllamaClient::list_models` | Non-streaming | Implemented |
| Show model | `POST /api/show` | `OllamaClient::show_model` | Non-streaming | Implemented |
| Pull model | `POST /api/pull` | `pull_model`, `pull_model_stream` | Streaming and non-streaming | Implemented |
| Create model | `POST /api/create` | `create_model`, `create_model_stream` | Streaming and non-streaming | Legacy Modelfile payload; incompatible with current creation |
| Copy model | `POST /api/copy` | `OllamaClient::copy_model` | Non-streaming | Implemented |
| Delete model | `DELETE /api/delete` | `OllamaClient::delete_model` | Non-streaming | Implemented |
| Running models | `GET /api/ps` | `OllamaClient::list_running_models` | Non-streaming | Implemented |
| Check blob | `HEAD /api/blobs/{digest}` | `OllamaClient::blob_exists` | Non-streaming | Implemented |
| Upload blob | `PUT /api/blobs/{digest}` | `OllamaClient::create_blob` | Non-streaming | SDK uses PUT; current server requires POST |

## Request capabilities

| Capability | SDK surface | Notes |
| --- | --- | --- |
| Generation options | `GenerateBuilder`, `Options` | Includes temperature, token limit, top-k, top-p, format, raw mode, keep-alive, images, and lower-level options |
| Chat messages | `ChatBuilder`, `ChatMessage` | System, user, assistant, tool messages, and image-bearing user messages are modeled |
| Tool calling | `ChatBuilder::tools`, `ChatBuilder::tool_choice` | Tools depend on the model; `tool_choice` has no verified native enforcement |
| Typed streams | `GenerateStream`, `ChatStream` | Streams deserialize newline-delimited JSON chunks into typed responses |
| Batch embeddings | `EmbedRequestBuilder::input` | Accepts a single string or a collection through `EmbedInput` conversions |
| Custom headers | `ClientConfigBuilder::header` or `OLLAMA_API_HEADERS` | Applied to SDK HTTP requests; protect secrets stored in headers |

## Partial and intentionally separate surfaces

- `EmbeddingsApi::embed_legacy` models the deprecated `/api/embeddings` route at the lower-level API
  layer, but `OllamaClient` does not provide a convenience method for it. Its response type currently
  requires a model field omitted by the stable server's legacy response.
- Ollama provides [partial OpenAI API compatibility](https://docs.ollama.com/api/openai-compatibility)
  at `/v1` routes. This SDK targets native `/api` routes and does not expose dedicated `/v1`
  OpenAI-compatible methods.
- Public request structs may not expose every field added by newer Ollama server versions. Open an
  issue with the endpoint, field, server version, and a minimal payload when a type is missing.
- Current structured create/import, schema-based output, thinking controls/output, embedding
  dimensions, native push and current model metadata have verified gaps described in the audit.
- Local System One, hosted web search/fetch and Anthropic-compatible methods are separate provider
  surfaces without dedicated SDK APIs. Experimental operations are not included in a native
  completeness claim.

## Behavioral limitations

- Blob upload currently sends PUT and adds a JSON Content-Type alongside its binary header. Stable
  Ollama registers POST for uploads. Creation sends an obsolete `modelfile` payload instead of the
  current structured source/settings fields. These are current-server incompatibilities, not just
  missing convenience methods.
- Streaming parsing operates on each received byte chunk. Split JSON can produce `InvalidResponse`,
  split UTF-8 can corrupt content, and multiple records in one chunk lose later records. The chat
  collector also loses tool calls emitted before the terminal message.
- `tool_choice` is serialized by the SDK but is not part of the verified native request contract;
  callers must not rely on it to force or prohibit tool selection.
- `KeepAlive::Never` serializes as null, which current request decoding treats as the server default;
  it does not disable keep-alive as documented. Use `Seconds(0)` to request unloading. Numeric seconds
  cannot be negative, although a negative duration string such as `"-1s"` can request retention.
- `version()` does not check HTTP status before decoding JSON, so a JSON error body may be returned
  as a successful value.
- Configuration/client Debug output can include custom Authorization header values. Credential
  headers are not explicitly marked sensitive; avoid logging configuration objects.
- `ClientConfig::max_retries` and `retry_delay` are retained configuration fields but are not
  consumed by the HTTP client. Requests are attempted once.
- The optional `tracing` feature adds the dependency but the SDK does not currently emit tracing
  spans or events.
- `health()` returns `false` for connection failures and non-success statuses instead of preserving
  the underlying error.

See [Configuration and reliability](configuration.md) for production guidance around these
behaviors.

## Updating this matrix

When adding or changing an endpoint:

1. Update the corresponding row and its mode.
2. Add or update a compile-tested example when the workflow is user-facing.
3. Add mock-server tests for request paths, serialization, statuses, and response decoding.
4. Update configuration or reliability notes when transport behavior changes.
5. Refresh the audit date only after comparing the matrix with the public client and API modules.
