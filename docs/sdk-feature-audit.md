# SDK feature audit and implementation plan

Audited **2026-10-04** against SDK main
[`f9fab82d3c0f6a6e53dbbdf125fa8c04062e9b7f`](https://github.com/ThreatFlux/ollama_rust_sdk/tree/f9fab82d3c0f6a6e53dbbdf125fa8c04062e9b7f)
and the latest stable Ollama release,
[v0.35.1](https://github.com/ollama/ollama/releases/tag/v0.35.1), published 2026-09-29.

This modernization change records verified API gaps and a proposed follow-up plan. It does not
implement the feature additions or behavior fixes below. The [coverage matrix](api-coverage.md)
continues to describe the SDK's actual public surface.

## Evidence and scope

The audit compared the public client, builders, request/response models, HTTP transport and stream
collectors with the official [documentation index](https://docs.ollama.com/llms.txt),
[OpenAPI schema](https://docs.ollama.com/openapi.yaml) and stable server/client source. The OpenAPI
schema was fetched and parsed successfully; it contains 14 paths, including local System One.
Release source was also necessary because the schema does not include every native replay field.
This is source and schema evidence, not a live-server compatibility test. No inference, upload,
model creation or publication was performed for the audit.

The most urgent findings affect existing operations: blob upload uses the wrong HTTP method,
creation sends an obsolete payload, and streaming depends on transport chunk boundaries. An
implemented endpoint row should not be read as proof of current server compatibility.

## Verified gaps

Priorities indicate the recommended implementation order. P0 covers broken existing operations;
P1 covers current native capabilities; P2 covers compatibility and convenience improvements.

| Priority / kind | Current SDK evidence | Provider evidence and resulting gap | Proposed follow-up and verification |
| --- | --- | --- | --- |
| P0 / blob upload bug | [`BlobsApi::create_blob`](../src/api/blobs.rs) sends PUT. [`HttpClient::send_request`](../src/utils/http.rs) adds JSON Content-Type to every request, including the binary upload. | Stable [routes](https://github.com/ollama/ollama/blob/v0.35.1/server/routes.go#L2061), [official client](https://github.com/ollama/ollama/blob/v0.35.1/api/client.go#L447) and OpenAPI require POST with a binary body. Current upload cannot reach the registered method. | Keep the public method signature; send POST with one octet-stream Content-Type. Mock exact bytes, existing/created responses and failures. Validate the digest or encode it as one path segment. |
| P0 / create incompatibility | [`CreateRequest`](../src/models/model_info.rs) sends `name` and `modelfile`; [`create_model`](../src/client.rs) only accepts a Modelfile string. | [Current create](https://docs.ollama.com/api/create) uses a structured request. Stable [handler](https://github.com/ollama/ollama/blob/v0.35.1/server/create.go#L230) rejects requests without `from` or `files`; its request type has no `modelfile`. | Add a current structured create API and typed progress. Preserve and label legacy methods; do not silently translate arbitrary Modelfiles. Mock source models, blob files, import settings, licenses and server errors. |
| P0 / NDJSON framing bug | [`GenerateApi`](../src/api/generate.rs), [`ChatApi`](../src/api/chat.rs) and both [model progress parsers](../src/api/models.rs) decode each byte chunk and return its first nonempty line. | [Streaming](https://docs.ollama.com/api/streaming) is newline-delimited JSON, not one record per network chunk. Split JSON fails, split UTF-8 can corrupt text, and later records in one chunk are lost. [Midstream errors](https://docs.ollama.com/api/errors) arrive as error objects under the original HTTP status. | Share a bounded byte-buffered decoder. Test arbitrary splits, multiple records, CRLF, blank lines, a final record without newline, malformed records, error objects, limits and incomplete EOF. Preserve public stream return types. |
| P0 / collected tool-call loss | [`ChatStream::collect_response`](../src/streaming/stream.rs) combines content but takes tool calls only from the terminal message. | The [tool-calling guide](https://docs.ollama.com/capabilities/tool-calling) requires accumulating streamed calls before replay. Calls emitted earlier disappear from the collected response. | Accumulate calls in order. Test intermediate calls followed by an empty final message, multiple calls, repeated identical calls and interrupted streams. Never automatically execute callbacks. |
| P1 / structured output gap | [`ResponseFormat`](../src/models/common.rs) only represents `Text` and `Json`. | [Structured outputs](https://docs.ollama.com/capabilities/structured-outputs) accept a JSON schema object. Native `"text"` is not a documented format setting. | Add a schema-capable current format API; use omission for ordinary text. Test exact request serialization and retain existing JSON behavior. |
| P1 / thinking gap | [Generate](../src/models/generation.rs) and [chat](../src/models/chat.rs) lack `think` and discard thinking output; [model info](../src/models/model_info.rs) discards advertised controls. | [Thinking](https://docs.ollama.com/capabilities/thinking) uses booleans or model-defined names, with omission/null selecting the default. Missing metadata does not prove lack of support. | Add current request/response types and a current collector. Test false/true/named/default values, numeric rejection, absent metadata and `[false]`. Discover exact supported names rather than infer them from model families. |
| P1 / tool replay gap | [`ChatMessage`](../src/models/chat.rs) has `tool_call_id` but lacks `tool_name` and thinking; [`FunctionCall`](../src/models/common.rs) lacks native `index`. | Native [tool replay](https://docs.ollama.com/capabilities/tool-calling) includes function names and indexed calls. | Add current replay models retaining names, indices, IDs and thinking. Round-trip assistant and tool messages, including index zero and multiple calls. Do not reinterpret a call ID as a function name. |
| P1 / unsupported native selection claim | [`ChatBuilder::tool_choice`](../src/builders/chat_builder.rs) serializes a selection strategy. | Neither the native OpenAPI ChatRequest nor stable [native ChatRequest](https://github.com/ollama/ollama/blob/v0.35.1/api/types.go#L133) defines `tool_choice`. The native server provides no verified selection guarantee for this field. | Retain the legacy surface if needed, but document its lack of native enforcement. Do not promise forced-tool behavior or invent client-side execution to simulate it. |
| P1 / generation and metrics gaps | [Generate request/response](../src/models/generation.rs), [chat request/response](../src/models/chat.rs) and their builders lack current controls and metadata. | [Generate](https://docs.ollama.com/api/generate) and [chat](https://docs.ollama.com/api/chat) document suffix where applicable, logprobs/top-logprobs, creation time, stop reason and cached prompt counts. Stable native request types also contain truncate/shift controls. | Add current controls and probability models; retain streamed metadata. Test false/zero settings, token bytes, alternatives and top-logprobs bounds. Keep debug-only fields outside ordinary coverage promises. |
| P1 / embedding dimensions gap | [`EmbedRequest`](../src/models/embedding.rs) and [`EmbedRequestBuilder`](../src/client.rs) omit `dimensions`. | [Embedding API](https://docs.ollama.com/api/embed) documents dimensionality control. | Add a compatible current request/builder. Test single/batch inputs, positive dimensions, truncation and unsupported-model errors; do not guess a model's supported sizes. |
| P1 / keep-alive semantics and numeric gap | [`KeepAlive::Never`](../src/models/common.rs) is documented to disable keep-alive but serializes as null; numeric seconds are unsigned. | Stable [requests](https://github.com/ollama/ollama/blob/v0.35.1/api/types.go#L92) use `*Duration`. [Go JSON decoding](https://pkg.go.dev/encoding/json#Unmarshal) maps null to a nil pointer, selecting the [server default](https://github.com/ollama/ollama/blob/v0.35.1/server/sched.go#L517), rather than zero/unload. Negative duration strings already allow retention; only negative numeric seconds are unrepresentable. | Document the existing default behavior and introduce explicit unload/default/retention semantics without silently changing legacy behavior. Test omitted/null defaults, zero/unload, positive and negative seconds, and negative duration strings. |
| P1 / missing native endpoint | No push entry point exists in [`OllamaClient`](../src/client.rs) or [`ModelsApi`](../src/api/models.rs). | [Push](https://docs.ollama.com/api/push) is a documented native operation supported by the [stable client](https://github.com/ollama/ollama/blob/v0.35.1/api/client.go#L335). | Add non-streaming and typed progress APIs using the shared decoder. Mock publication requests, errors, success and cancellation; do not publish real models during tests. |
| P2 / model metadata and show controls | [`show_model`](../src/api/models.rs) forces `verbose:false`; [`ModelInfo` and `RunningModel`](../src/models/model_info.rs) discard current metadata. | [Show](https://docs.ollama.com/api-reference/show-model-details) exposes capabilities, thinking and model metadata; [running models](https://docs.ollama.com/api/ps) expose context length. | Add configurable show and current metadata types retaining extension fields. Test absent/null values, unknown keys and verbose requests; use advertised metadata for capability checks. |
| P2 / legacy response incompatibility | [`LegacyEmbeddingResponse`](../src/models/embedding.rs) requires `model`. | Stable [legacy embedding response](https://github.com/ollama/ollama/blob/v0.35.1/api/types.go#L646) contains only `embedding`. | A serde default can preserve the existing Rust field type while accepting the response. Test an embedding-only payload; document the absent model and prefer `/api/embed`. |
| P2 / version status bug | [`OllamaClient::version`](../src/client.rs) decodes JSON without checking HTTP status. | The [error contract](https://docs.ollama.com/api/errors) uses JSON error bodies. A non-success JSON body can currently be returned as `Ok(Value)`. | Check status while retaining the method signature. Mock valid versions, JSON errors and malformed JSON; add a typed convenience method separately if useful. |

## Transport and configuration findings

[`ClientConfig` and its builder](../src/config.rs), [`HttpClient`](../src/utils/http.rs) and
[`OllamaClient`](../src/client.rs) derive `Debug`. Configuration headers can therefore expose a
custom Authorization value in debug output. Transport header values are not explicitly marked
sensitive. A follow-up should redact credentials in configuration/client diagnostics, validate
headers at construction without echoing values, and mark credential headers sensitive. Test a
fresh runtime sentinel through Debug and error paths and through each request format.

`max_retries` and `retry_delay` remain inactive; requests are attempted once. The optional `tracing`
feature adds a dependency but does not emit spans/events. Keep these limitations documented until
implemented. A retry design must define eligible operations and failures, respect cancellation and
deadlines, and avoid replaying model mutations or a stream after output has reached the caller.
Tracing should omit credentials, prompts, generated content and tool arguments by default.

Direct cloud access is already possible with origin `https://ollama.com` and a custom Authorization
header. Dedicated `OLLAMA_API_KEY` configuration is a convenience gap, not evidence that cloud
inference is unsupported. The [authentication contract](https://docs.ollama.com/api/authentication)
requires a Bearer token for direct hosted access. Client endpoint paths already include `/api`;
the SDK's configured URL should be the origin, rather than a copied API-prefixed URL.

`health()` intentionally returns false for failures, as documented. An error-preserving diagnostic
method would be an additive convenience. It should not silently change the existing method's
meaning.

## Implementation plan

### 1. Repair existing operations

Assign shared HTTP/header changes and blob upload to one transport owner. Assign the NDJSON decoder
and collectors to one streaming owner so all four parsing paths use the same framing rules.
Repair version status handling and legacy embedding decoding in a small compatibility slice.
Add deterministic mock regressions before removing any limitation from the coverage document.
Current create support belongs in the next slice because its public request shape needs a migration.

The decoder should buffer bytes until a record boundary, decode UTF-8 only for a complete record,
emit all records in order, and fail explicitly at a positive frame limit. Generate/chat collectors
must distinguish terminal completion from premature EOF and return when the terminal response
arrives. Model progress must distinguish terminal success from intermediate download completion.
Dropping a stream must release the HTTP body without a detached producer continuing work.

### 2. Add current native models and builders

Introduce additive current request/response types for formats, thinking, replay, probabilities,
embedding dimensions, keep-alive and model metadata. Preserve useful legacy entry points, and
provide a compile-tested current workflow example. Keep unknown response metadata where practical
while rejecting malformed recognized data. Do not hide reasoning/tool-call loss behind a legacy
conversion; document any lossy projection.

For keep-alive, direct `Duration.UnmarshalJSON` rejects null, but that decoder is not invoked when
JSON null is assigned to the request's duration pointer. The endpoint therefore accepts null as
unset. `Never` does not disable keep-alive or explicitly request indefinite retention: it selects
the configured server default. `Seconds(0)` requests unloading, and a negative duration string such
as `Duration("-1s")` can already request retention. Adding signed numeric seconds is a separate
representation improvement.

Adding fields to public structs breaks existing struct literals. Adding variants to public enums
breaks exhaustive matches; changing a field's type also affects consumers. A defaulted serde field
can fix decoding without changing the Rust shape, but it does not make every DTO expansion
compatible. Use separate current types unless a deliberate breaking release is selected. Compile
representative old literals/matches and run a semver comparison before publication.

### 3. Complete native model management

Add a structured create builder and request API supporting source models or uploaded files,
license strings/lists, parameters, messages and current import settings. Keep deprecated adapters
out of the advertised supported path. Add configurable show and push operations, sharing typed
status/progress decoding and transport errors. Provide a migration from the legacy Modelfile
method; do not implement a partial Modelfile parser without an explicit parsing contract.

An independent reviewer should compare fixtures to stable source and examine compatibility before
the integrator updates public exports, coverage and release notes. Use distinct file ownership for
transport, wire models and model management; coordinate shared models, collectors and Cargo runs
through one integrator.

### 4. Select separate surfaces deliberately

The following are verified provider surfaces but are outside the initial native repair plan:

- [System One](https://docs.ollama.com/api/systemone) is a local-only `/v1/systemone` operation,
  available in Ollama v0.35.0 or later. It is separate from OpenAI compatibility. It needs tagged
  question/answer types, provider input bounds and scoring-capable models; it does not support
  streaming, tools or ordinary generation controls.
- [Hosted web search/fetch](https://docs.ollama.com/capabilities/web-search) use authenticated
  `/api/web_search` and `/api/web_fetch`. Local stable-server proxies have explicitly experimental
  paths. Keep hosted APIs and experimental local wrappers distinct.
- [OpenAI compatibility](https://docs.ollama.com/api/openai-compatibility) and
  [Anthropic compatibility](https://docs.ollama.com/api/anthropic-compatibility) expose subsets with
  local/cloud differences. Dedicated compatibility clients would be separate SDK scope.
- Experimental recommendations/cloud status, account administration and debug rendering should
  stay outside ordinary native completeness promises.

## Follow-up acceptance criteria

Each delivered slice must update the coverage matrix, configuration notes where relevant, examples
and changelog. Run `make ci-local`, the documentation contract, required feature configurations and
the actual documented MSRV. Check semver and package contents when public models or modules change;
inspect hosted results on the pushed commit. Keep publication as a dry run unless separately
authorized.

Use mocked requests and controlled byte streams for ordinary tests. Existing integration and
performance tests can contact localhost and perform inference when a server is present; a passing
local gate is not proof of provider completeness. A separately authorized live compatibility run
must record the server version and model and distinguish skipped cases from successful coverage.
