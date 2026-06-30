# AI-Brains Project Rules

## Engineering Mandates
- **Capture Independence**: The capture path (CLI -> Daemon -> Event Log) MUST remain functional without dependencies on models, embeddings, or graph databases.
- **Canonical Source of Truth**: Every state change MUST be recorded as an immutable event in the SQLCipher-backed append-only event log.
- **CQRS Integrity**: Commands append events; queries read projections. DO NOT mix read/write logic in the same service or transaction.
- **Capture Privacy**: DO NOT store hidden chain-of-thought, model reasoning, or raw tool logs. Capture ONLY the final assistant response and user prompt.
- **Privacy Inheritance**: Derived memories (summaries, clusters) MUST inherit the strictest privacy flag from their source events.
- **Rust Safety**: PROHIBITED use of `unwrap()`, `expect()`, or `panic()` in production code. Explicit error handling (`thiserror`, `anyhow`) and `zeroize` for sensitive key material are mandatory.
- **Provenance**: ALL architectural decisions and track implementations MUST be recorded in the `ledgerful ledger`.
- **No Repository Pollution**: AI-Brains MUST NOT write project-local files by default. Use global user storage (`$env:USERPROFILE\.ai-brains`) unless the user explicitly invokes a repo-write command.

## Technical Invariants
- **Path Normalization**: Normalization for Windows drive-case, UNC prefixes, and WSL mappings is mandatory for all stored paths.
- **Relational Graph**: Implementation MUST use the native SQLite backend (Recursive CTEs) to avoid C++ build friction.
- **Event Sourcing**: Updating or deleting raw events is PROHIBITED. Use compensating events for corrections.
- **Commercial Safety**: Only permissive licenses (MIT, Apache, BSD) are allowed. AGPL/GPL dependencies are PROHIBITED.

## Hardware & Environment
- **Context Constraints**: Enforce a 38,912 token limit for summarization. Use sequential chunking with context carryover for larger sessions.
- **VRAM Management**: High-performance multi-stage RAG (BGE-M3 + Qwen 3.5) MUST use dynamic model switching via `.env` to prevent VRAM overflow.
- **Shell Consistency**: Use PowerShell for all shell commands. PROHIBITED use of `&&`. Use `;` as the statement separator.

## Rust Standards
- **Edition**: 2024.
- **Forbid**: `unwrap()` and `expect()` in production code.
- **Error Handling**: User-facing errors use `thiserror` + `miette::Diagnostic`; internal errors may use `anyhow`.
- **Paths**: Prefer `camino` for UTF-8 paths. All stored paths MUST be normalized for Windows drive-case, UNC prefixes, and WSL mappings.
- **Async**: Core is synchronous; only the daemon feature uses `tokio` (tower-lsp + tokio).
- **Secrets**: Never commit `.env`, credentials, or API keys.
- **Determinism**: Same repo state + config MUST produce the same output. Sort emitted collections, version packet schemas, annotate partial parse/scan failures, never silently suppress failures, and normalize volatile fixture fields (timestamps).

## Workflow & Verification
- **Test-Driven Development**: Behavioral correctness MUST be proven via failing tests before implementation (Two-commit minimum: Red -> Green).
- **CI Gate**: Before every commit, the workspace MUST pass:
  `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit`
- **Track Discipline**: Implementation MUST follow the `conductor/conductor.md` track-by-track.
- **Change Management**:
  - **Before Edits**: Run `ledgerful doctor` to ensure the toolchain is healthy, and `ledgerful scan --impact` for meaningful code/config/policy edits. Inspect any hotspots and high (>70%) temporal coupling.
  - **During Edits**: DO NOT edit `.ledgerful/` state files directly.
  - **After Edits**: Run `ledgerful verify` to validate changes against safety rules. Report verification outcomes, pending transactions, risk levels, and drift.
  - **Ledger Provenance**: Record architectural updates via `ledgerful ledger start` / `commit` or `atomic` commands.
- **AI-Brains Self-Usage**:
  - **Preflight Briefing**: Run `ai-brains preflight --summary` at session start to check active sessions and constraints.
  - **Decision Recall**: Run `ai-brains recall "<query>" --semantic` to search past architectural decisions.
  - **Decisions Pinning**: Run `ai-brains pin "<DECISION/CONSTRAINT: message>"` to persist new decisions or project constraints.

## Test Conventions
- **Naming**: `function_or_feature__condition__expected_result`; drop the `test_` prefix; preserve the `__slow` suffix (tier marker).
- **Tiers**: default (<60s, excludes `compile_fail` + `__slow`), ci (+retries, 60s), slow (`__slow`, 300s, nightly), compile_fail (`trybuild`, separate job), doctests (`cargo test --doc`, PR-time).
- **Never**: `std::env::set_var`/`remove_var` (use `TempEnv` RAII + `#[serial(env)]`), sleep-for-async (use `wait_for_condition`), bare `#[ignore]` (require reason + owner), real network calls (use `httpmock`/loopback), fs writes outside tempdir.
- **Must**: `tempfile::tempdir()` per test, `rstest` `#[case]` for parameterization (never for-loop in one `#[test]`), assert specific values (not just `is_ok`/`is_err`), `DirGuard` for cwd (use `#[serial(cwd)]` if needed), `OnceLock<Arc<SharedState>>` for shared immutable builds (not mutable state, not servers).
- **Fixtures**: Each test spawns its own Axum router on `127.0.0.1:0`. Full details in the onboarding skill.

## Git
- **Forbid**: push to `main`/`master`, force-push without explicit approval, destructive operations without explicit approval, committing secrets/`.env`.
- **Require**: inspect diff before commit, commit only intentional files, keep unrelated fixes separate where practical, clear ledger status before push.
  - **Push Hygiene**: `git fetch --all --prune` before staging; reconcile if `origin/main` moved; stage only intended scope; prune conservatively. The pre-push hook runs `ledgerful verify --scope fast` + `ledgerful ledger status` — treat it as the authoritative publish gate.

## Stop-Before
Halt and ask the user before proceeding with any of:
- Destructive git operation, force-push, push to `main`/`master`.
- Missing secrets or unavailable external service with no documented mock.
- Ambiguous or conflicting specs not resolvable from code + plan.
- Broad unrelated failures (triage first; do not broadly clean up).
- Unsafe required dependency upgrade.
- Scope exceeds current track.

## Unrelated Failures
- **Rule**: Triage; do not broadly clean up.
- **Fix only if**: obvious, low-risk, AND blocking validation.
- **Otherwise**: Document why unrelated, document why fix is necessary, otherwise leave/report.
- **Commit**: Keep separate where practical.

## Contracts
- **Required when**: `ai-brainsd` API payload changed, config gate changed, daemon behavior changed, or `ai-brains-contracts` DTOs changed.
- **Update**: Affected crate docs, `ai-brains-contracts` types, any frontend/CLI consumers of the changed DTO, and `Docs/` references to the contract surface.
- **Missing contract update**: high finding.
- **Template**: E1 empty-state string|null ripple — document the null/empty/missing shape for every new API surface.

## Review & Severity
- **Review Log**: `conductor/<track>/review.md` (the review log is NOT the ledgerful ledger).
- **Critical/High**: MUST be `verified_fixed` before clearance. Regression caused by this work is always high; never deferrable.
- **Medium**: Fix by default. Defer only if not a regression, one-line justification in `review.md`, tracked follow-up, cap ≤3 deferred mediums per track, and appended to `conductor/ISSUES.md`.
- **Low-info**: Defer freely; MUST append to `conductor/ISSUES.md`.
- **Closure**: Code change alone is not closure. Implementer may mark `fixed_pending_verification`; reviewer/cross-model may mark `verified_fixed`. New findings enter the same log; the loop continues until clean.
- **Cross-Model Review**: For high-risk diffs (ARCHITECTURE, FEATURE, SECURITY categories), run a read-only cross-model review before final verification. See the `codex-review` skill.

