# Real inference behind the Smart Tree context API

Set `ST_INFERENCE_MODELS` before starting `st --ollama-serve` to route requests
to actual local model servers instead of the legacy context-only synthesizer:

```sh
export ST_INFERENCE_MODELS='{"my-model":{"url":"http://127.0.0.1:9000","model":"backend-model-id"}}'
st --ollama-serve
```

The backend must implement non-streaming `POST /v1/chat/completions` and
`GET /health`. Origins must use HTTP with a literal loopback IP. Redirects,
proxy environment variables, and cloud fallback are disabled. Missing models
return 404; backend failures return an error rather than a context template.

The bridge supports `/api/generate`, `/api/chat`, `/v1/chat/completions`,
configured-model listing, and a health-backed `/api/show`. Conversation history
is retained. Smart Tree adds historical project references as reference-only
context. OpenAI request fields pass through; common Ollama generation options
are translated. Output budgets are capped at 8192 tokens, and each backend
must enforce its own context and generation limits.

Streaming is incremental: backend SSE is forwarded as SSE for OpenAI clients
or translated to NDJSON for Ollama clients. Incomplete or malformed backend
streams fail rather than being reported as complete. Model management,
residency inspection, and learned embeddings return 501.
Listings describe configured routes, not measured residency or memory use.

The context code uses integer arithmetic. This does **not** imply integer-only
model inference: llama.cpp and PyTorch backends may use floating-point kernels.
A trained LoRA adapter must be loaded together with its exact base model.
Partial PQM tensor exports are not a complete runnable model. Hash verification,
training loss, model quality, and measured energy savings are separate checks.

Each route can include an `identity` object with a physical weight digest,
byte count, template digest, system prompt, and capability flags, plus a
`defaults` object for decoding settings. The operator must verify these against
the files and backend startup configuration. Generation responses expose the
backend origin, model ID, weight digest and template digest in `X-ST-*` headers.
Replies whose backend model ID does not match the configured route are rejected.
