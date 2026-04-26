# Track T21 — Model Providers

## Owner
architecture-planner

## Status
In Progress

## Objective
Implement a provider-agnostic model abstraction with initial support for local Ollama and a mock for testing.

## Scope
- Scaffold `ai-brains-models` crate.
- Define `ModelProvider` trait (completion and embedding).
- Implement `OllamaProvider` using `reqwest`.
- Implement `MockProvider` for testing.
- Implement `ProviderRegistry` to handle fallback and privacy-aware routing.

## Out of Scope
- Cloud providers (OpenAI, Anthropic) - focus on local first.
- Complex orchestration (AutoGPT style).

## Files Owned
- `crates/ai-brains-models/**`

## Files Allowed To Touch
- `Cargo.toml`
- `Docs/conductor/conductor.md`
- `Docs/status.md`

## Files Forbidden To Touch
- `crates/ai-brains-core/**`
- `crates/ai-brains-events/**`

## Public Contracts Consumed
- `ai_brains_core::privacy::Privacy`

## Public Contracts Produced
- `ai_brains_models::ModelProvider`
- `ai_brains_models::CompletionRequest`
- `ai_brains_models::EmbeddingRequest`

## Required Tests First
- `tests/ollama_provider_returns_mocked_completion.rs`
- `tests/registry_respects_privacy_gate.rs`

## Implementation Steps
1. [ ] Scaffold `ai-brains-models` crate and add to workspace.
2. [ ] Define core traits and DTOs in `lib.rs`.
3. [ ] Implement `ollama.rs` (local only).
4. [ ] Implement `registry.rs` for provider selection.
5. [ ] Implement `mock.rs` for tests.
6. [ ] Verification and CI gate.

## Failure Modes To Handle
- Provider offline (actionable error).
- Timeout.
- Rate limit (local rate limit logic).

## Security Requirements
- No transmission of `local_only` memory to cloud providers (enforced by registry).
- API keys stored in vault or environment (never in code).

## Acceptance Criteria
- Registry returns an error if a cloud provider is requested for `local_only` data.
- Ollama provider successfully communicates with a local endpoint (mocked in tests).
- CI pass with clippy and nextest.

## Handoff Notes
- Phase 9 start.
