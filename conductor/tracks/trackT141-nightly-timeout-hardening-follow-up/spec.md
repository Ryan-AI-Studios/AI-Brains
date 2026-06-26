# Track T141: Nightly Timeout Hardening Follow-up

**Status:** Complete
**Owner:** Codex
**Priority:** P1 - prevents repeated nightly hangs and keeps tests deterministic.
**Category:** BUGFIX
**Source:** Worktree triage after T139/T140, 2026-06-26.

## Problem Statement

The worktree contained coherent but untracked nightly/model hardening changes:

- `OllamaProvider` had per-request timeout support like `LlamaCppProvider`.
- `NightlyService` aborted summarization after three consecutive session errors.
- `nightly` treated WAL checkpoint failure as non-fatal.
- Recall smoke tests opted out of the live ChangeGuard bridge with `--no-bridge`.

These changes should not be reverted or mixed into T139/T140. They need a
dedicated track with targeted tests and provenance.

## Acceptance Criteria

- AC1: Ollama completion requests time out with `ModelError::Timeout`.
- AC2: Refused Ollama connections fail quickly as `Timeout` or `Network`.
- AC3: Nightly summarization stops after three consecutive session errors and
  does not attempt a fourth failed session in the same run.
- AC4: WAL checkpoint failures remain non-fatal in the CLI nightly path.
- AC5: Recall scope smoke tests are deterministic and do not depend on the live
  ChangeGuard bridge.

## Implementation Notes

- Add `OllamaProvider::with_timeouts` for hermetic timeout tests.
- Add an `ai-brains-brain` regression test using a failing model provider and
  five completed sessions.
- Preserve the existing nightly/Ollama worktree changes after verifying them.

## Verification

- `cargo nextest run -p ai-brains-models ollama_provider_timeout_expires_returns_timeout_error`
- `cargo nextest run -p ai-brains-models ollama_provider_refused_connection_returns_error_quickly`
- `cargo nextest run -p ai-brains-brain nightly__three_consecutive_summary_errors__aborts_remaining_sessions`
- `cargo nextest run -p ai-brains-cli recall__default_scope__searches_all_project_memories recall__global_flag__searches_all_projects_and_sessions recall__session_flag__scopes_to_specified_session recall__env_session_id__does_not_auto_scope`
- `cargo clippy -p ai-brains-models --all-targets -- -D warnings`
- `cargo clippy -p ai-brains-brain --all-targets -- -D warnings`
- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- `ledgerful verify --scope fast`
