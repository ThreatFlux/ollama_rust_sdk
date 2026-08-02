# API coverage

Last audited against the public SDK source on **2026-08-02**.

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
| Create model | `POST /api/create` | `create_model`, `create_model_stream` | Streaming and non-streaming | Implemented |
| Copy model | `POST /api/copy` | `OllamaClient::copy_model` | Non-streaming | Implemented |
| Delete model | `DELETE /api/delete` | `OllamaClient::delete_model` | Non-streaming | Implemented |
| Running models | `GET /api/ps` | `OllamaClient::list_running_models` | Non-streaming | Implemented |
| Check blob | `HEAD /api/blobs/{digest}` | `OllamaClient::blob_exists` | Non-streaming | Implemented |
| Upload blob | `PUT /api/blobs/{digest}` | `OllamaClient::create_blob` | Non-streaming | Implemented |

## Request capabilities

| Capability | SDK surface | Notes |
| --- | --- | --- |
| Generation options | `GenerateBuilder`, `Options` | Includes temperature, token limit, top-k, top-p, format, raw mode, keep-alive, images, and lower-level options |
| Chat messages | `ChatBuilder`, `ChatMessage` | System, user, assistant, tool messages, and image-bearing user messages are modeled |
| Tool calling | `ChatBuilder::tools`, `ChatBuilder::tool_choice` | Actual tool support and output depend on the selected model and server |
| Typed streams | `GenerateStream`, `ChatStream` | Streams deserialize newline-delimited JSON chunks into typed responses |
| Batch embeddings | `EmbedRequestBuilder::input` | Accepts a single string or a collection through `EmbedInput` conversions |
| Custom headers | `ClientConfigBuilder::header` or `OLLAMA_API_HEADERS` | Applied to SDK HTTP requests; protect secrets stored in headers |

## Partial and intentionally separate surfaces

- `EmbeddingsApi::embed_legacy` models the deprecated `/api/embeddings` route at the lower-level API
  layer, but `OllamaClient` does not provide a convenience method for it.
- Ollama provides [partial OpenAI API compatibility](https://docs.ollama.com/api/openai-compatibility)
  at `/v1` routes. This SDK targets native `/api` routes and does not expose dedicated `/v1`
  OpenAI-compatible methods.
- Public request structs may not expose every field added by newer Ollama server versions. Open an
  issue with the endpoint, field, server version, and a minimal payload when a type is missing.

## Behavioral limitations

- Streaming parsing operates on each received byte chunk. A JSON record split across transport
  chunks can currently produce `InvalidResponse`; callers should treat streaming compatibility as
  server- and transport-sensitive.
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
