# Track T100: Add LLM Request Timeouts to Prevent Nightly Hangs

**Status:** Complete
**Started:** 2026-06-22
**Owner:** GLM-5.2
**Priority:** P1 — `nightly` hangs indefinitely when the LLM is slow or context exceeds limits.
**Source:** Non-destructive command test 2026-06-22; root cause confirmed by code audit.

---

## Problem Statement

`ai-brains nightly` hangs during session summarization. Root cause: `LlamaCppProvider::complete()` in `crates/ai-brains-models/src/llama_cpp.rs:48` creates `reqwest::Client::new()` with **no timeout**. When the LLM backend takes a long time (large context, slow generation, or context-exceeds-limit errors), the request hangs forever. The nightly service calls `complete()` for each session, so one slow session blocks the entire pipeline.

The same issue affects `embed()` (line 106) and `tokenize()` (line 149) — all use `reqwest::Client::new()` with no timeout.

## Acceptance Criteria

**AC1:** `LlamaCppProvider` creates its `reqwest::Client` with a configurable timeout. Default: 120 seconds for completions, 30 seconds for embeddings, 10 seconds for tokenize.

**AC2:** The timeout is configurable via env vars: `AI_BRAINS_LLM_TIMEOUT_SECS` (default 120), `AI_BRAINS_EMBEDDING_TIMEOUT_SECS` (default 30).

**AC3:** When a request times out, the error is propagated gracefully (not a panic). The nightly service catches the error, logs it, and skips that session — it does not abort the entire nightly run.

**AC4:** `nightly` completes in bounded time even when the LLM is unreachable. Sessions that time out are logged as errors and skipped.

**AC5:** No regression in existing tests. The `reqwest::Client` with timeout is used in all three methods (`complete`, `embed`, `tokenize`).

## Design Notes

- Use `reqwest::Client::builder().build()` (no global timeout) and apply per-request timeouts via `client.post(...).timeout(Duration::from_secs(...)).send()`. This is critical because completions need 120s, embeddings 30s, and tokenize 10s — a single client-level timeout cannot serve all three.
- Store the `reqwest::Client` as a field on `LlamaCppProvider` (currently `Client::new()` is called on every request — this is also inefficient). Reuse the client across requests.
- Read timeouts from env vars at construction time, store as fields on the struct.
- The nightly service at `crates/ai-brains-brain/src/lib.rs:110-130` already catches errors per session (`tracing::error!` + `errors.push(...)`), so the graceful degradation is already in place — it just never fires because the request hangs instead of erroring.
- **Do NOT change the LLM model, the router, or the dynamic router config.** The LLM infrastructure (gemma-4-E4B-it on port 8081 via the dynamic router) is working correctly. The fix is purely on the AI-Brains client side: add a timeout so requests fail fast instead of hanging.

## Files

- `crates/ai-brains-models/src/llama_cpp.rs` — add timeout to `reqwest::Client`, store `Client` as a field, read timeout from env.
- `crates/ai-brains-models/src/lib.rs` — if there's a provider factory, update it to pass timeout config.
- `crates/ai-brains-models/tests/` — add a test that verifies the client has a timeout set (or that a request to a dead server fails within the timeout).

## Tests (TDD)

**Red:** `llama_cpp_provider__timeout_expires__returns_network_error` — create a `LlamaCppProvider` pointing to a port with no server, call `complete()` with a 2-second timeout, assert it returns an `Err` within 3 seconds (not hanging).

**Green:** Add timeout to the `reqwest::Client` builder. Test passes.

## Verification

- `cargo nextest run -p ai-brains-models`
- Manual: `ai-brains nightly --skip-import` — completes in bounded time (sessions that exceed timeout are skipped with an error message, not hung).
- Manual: Verify that normal completions still work (no regression): `ai-brains pin "test"` should still succeed.

## Out of Scope

- Changing the LLM model or router configuration.
- Streaming responses (currently `stream: false`).
- Retry logic (separate track if needed).
- Context-size validation (the nightly already has chunking for large sessions; the timeout is the safety net).