---
name: onboarding
description: "Load at session start, before Rust edits, before conductor work, and before any implementation task. Establishes AI-Brains repo context, workspace layout, the implement loop, tooling, and research precedence. Rules live in AGENTS.md; this skill does not repeat them."
---

# AI-Brains Onboarding

**AI-Brains** — Windows-first, local-first memory system for AI coding harnesses. Captures clean conversation history without tool logs or hidden thinking.

> **Rules live in `AGENTS.md`** (Engineering Mandates, Technical Invariants, Rust Standards, Test Conventions, Git, Stop-Before, Unrelated Failures, Contracts, Review & Severity). This skill provides context, commands, and workflow — it does not repeat rules.

## Architecture: Rust Workspace

| Crate | Responsibility |
|-------|----------------|
| `ai-brains-core` | Pure domain model (ids, privacy, session, memory). No external IO. |
| `ai-brains-events` | Immutable event definitions and the event envelope (hashing, signing). |
| `ai-brains-contracts` | Shared JSON DTOs for CLI <-> Daemon communication. |
| `ai-brains-store` | SQLCipher event log, migrations, and read projections. |
| `ai-brains-crypto` | Key material management, DPAPI wrappers, and recovery kit logic. |
| `ai-brains-path` | Windows/WSL/UNC path normalization. |
| `ai-brains-capture` | Converts harness-specific IO into normalized domain events. MUST NOT depend on `ai-brains-models` or `ai-brains-graph`. |
| `ai-brains-retrieval` | FTS5 + semantic search + graph-augmented recall. |
| `ai-brains-graph` | GraphProjector, SqliteGraphBackend, CozoProxy bridge (feature-gated `--features graph`). Failures non-fatal. |
| `ai-brains-models` | Local AI provider routing (Ollama, etc.). |
| `ai-brains-brain` | NightlyService, MemorySynthesizer, EmbeddingService. |
| `ai-brains-scheduler` | Windows Task Scheduler integration. |
| `ai-brainsd` | Daemon: single-writer queue, vault unlock. |
| `ai-brains-cli` | Main CLI binary (`ai-brains` command) + LiveGraphHook (T69). |

## Authority Order

1. User/run prompt
2. `conductor/conductor.md`
3. `conductor/<track>/spec.md`
4. `conductor/<track>/plan.md`
5. This onboarding skill
6. `AGENTS.md`
7. Docs and ledger history
8. External docs (context7, web)

## Current State

- Tracks T61–T71 all complete. See `conductor/conductor.md` for the registry.
- Graph feature-gated (`--features graph`). Full CI gate reproducible on Windows (T71).
- Deviations documented in `Docs/Deviations.md`.

## Session Start

```powershell
ai-brains preflight --summary
ledgerful doctor
ledgerful ledger status --compact
ledgerful index --incremental
ai-brains recall "what is this project" --semantic
```

Then read `conductor/conductor.md` (track registry) and `conductor/ISSUES.md` (unresolved debt). Map relevant ISSUES items into the current track's plan if scopes overlap. Reconcile dirty ledger/drift before edits unless user says otherwise.

## Recall & Graph Commands

```bash
# Recall (session memory, decisions, synthesized knowledge, code symbols)
ai-brains recall "GPU driver fix" --limit 5                      # FTS5 keyword
ai-brains recall "auth flow" --semantic --limit 5               # semantic
ai-brains recall "login" --semantic --graph-boost 0.1 --limit 5 # graph-boosted
ai-brains recall "query" --project-id <id>                       # project-scoped

# Graph (requires --features graph build)
ai-brains graph neighbors <memory_id>    # 1-hop neighbors
ai-brains graph hierarchy <memory_id>    # synthesis chain
ai-brains graph session <session_id>     # all memories in a session
ai-brains graph update                   # health check
ai-brains graph rebuild                  # full resync (recovery only)

# ledgerful (live code symbols, routes, call graph)
ledgerful search "handleGetUser"        # find by name
ledgerful ask "what calls validateToken" # natural language
ledgerful scan --impact                  # blast radius before editing
```

| Question | Use |
|----------|-----|
| "What did we decide about X?" | `ai-brains recall` |
| "What does function X do / which endpoints exist?" | `ai-brains recall --semantic` (symbols indexed since T70) |
| "Live code query (not yet in nightly)" | `ledgerful search` / `ledgerful ask` |
| "What calls this function?" | `ledgerful ask "what calls <fn>"` |
| "What was synthesized from this session?" | `ai-brains graph hierarchy <id>` |

> **T70:** `ai-brains recall` also returns code symbols ingested from ledgerful during nightly — a single recall query suffices for most questions about decisions and code structure.

## CI Gate

`AGENTS.md` defines the gate. Use the helper script or run it directly:

```powershell
.\scripts\dev-check.ps1              # checks tools + runs gate
.\scripts\dev-check.ps1 --check-only # tool presence + versions only
```

Required tool versions (install commands in `Docs/ci-tooling.md`):

| Tool | Min Version |
|------|-------------|
| `cargo-nextest` | 0.9.137 |
| `cargo-deny` | 0.19.4 |
| `cargo-audit` | 0.22.1 |

### Targeted vs Full Verification

- **During work**: `ledgerful verify --scope fast` (scoped to changed files via `test_mapping`).
- **Before finalizing**: `ledgerful verify --scope full`.
- **Targeted**: `cargo nextest run --lib --bins -p <crate>` + `cargo clippy -p <crate> --all-targets -- -D warnings`.
- **Integration**: `--test-threads=1` if tests share state.
- **Doctests**: `cargo test --doc -p <crate>` if examples added.
- **Never**: `--no-verify` unless user explicitly requests.

## Anti-Overengineering

Do NOT:
- Build lock managers before a real race exists.
- Build generalized plugin systems with one implementation.
- Force SQLite when flat-file state is enough.
- Create an abstraction layer with a single implementation.
- Build repo-wide call graphs when targeted queries suffice.

## Test Conventions (Reference)

`AGENTS.md` has the compact forget-prone rules. Full detail:

- **Naming**: `function_or_feature__condition__expected_result`; drop `test_` prefix; preserve `__slow` suffix (tier marker).
- **Tiers**:
  - **default**: fast unit + integration, <60s target, excludes `compile_fail` + `__slow`.
  - **ci**: default + retries, 60s slow-timeout.
  - **slow**: heavy integration tests suffixed `__slow`, 300s timeout, nightly.
  - **compile_fail**: `trybuild`, separate CI job.
  - **doctests**: `cargo test --doc`, PR-time.
- **Environment Variables**: NEVER call `std::env::set_var`/`remove_var` directly in tests — use a `TempEnv` RAII guard for cleanup + `#[serial(env)]` for thread safety. Both required. Edition 2024 made these unsafe.
- **Working Directory**: Per-test `DirGuard` restores `set_current_dir` on drop; use `#[serial(cwd)]` if serialization needed. No global cwd_lock mutex.
- **Parameterization**: Use `rstest` `#[case]` (generates independent `#[test]` per case, granular failures). NEVER for-loop inside a single `#[test]` (aborts on first failure, hides rest).
- **Fixtures**: Per-test `tempfile::tempdir()` mandated for SQLite/hermetic isolation. Share immutable expensive builds via `OnceLock<Arc<SharedState>>` — NOT mutable state, NOT servers. Each test spawns its own Axum router on `127.0.0.1:0`.
- **Sleeps**: NEVER sleep-for-async. Use a `wait_for_condition` helper — bounded poll, 2s timeout, 50ms interval. Production retry backoffs are exempt.
- **Assertions**: Assert specific values, not just `is_ok()`/`is_err()`/`is_empty()`. Every test MUST fail for a meaningful reason if the protected behavior breaks.
- **Ignored**: `#[ignore = "reason"]` with explicit reason + owner. No bare `#[ignore]`.
- **No Secrets**: No real network calls in tests — use `httpmock` or loopback to a spawned server. No fs writes outside tempdir.

## The Conductor / Track System

Each track is a bounded unit of work: `spec.md` (specification) + `plan.md` (task checklist). Status in `conductor/conductor.md`. Debt in `conductor/ISSUES.md`.

| File | Purpose |
|------|---------|
| `conductor/conductor.md` | Track registry |
| `conductor/ISSUES.md` | Unresolved debt (deferred mediums/lows) |
| `conductor/<track>/spec.md` | Track spec (objective, API contracts, verification plan) |
| `conductor/<track>/plan.md` | Task checklist (`- [ ]`) |
| `conductor/<track>/review.md` | Review log (NOT the ledgerful ledger) |

**Backlog routing**: When planning a track, absorb related `ISSUES.md` items into `plan.md` and remove them from `ISSUES.md`.

## Implement Loop

```
plan -> start_tx -> implement_tdd -> targeted_checks -> review_convergence
      -> manual_test -> sync_contracts -> full_gate -> finalize
```

### 1. Plan
- Read `conductor/conductor.md`, `conductor/<track>/spec.md`, `conductor/<track>/plan.md`.
- Run `ledgerful ledger status --compact` + `ledgerful scan --impact` + `ledgerful hotspots`.
- Output: affected crates/files, expected behavior, proof tests, dependency research, contract surfaces, likely conflicts.
- **Spec vs reality**: Follow actual code layout over aspirational spec paths. Note drift in `plan.md`; don't create fake modules to match spec.
- **Missing spec/plan**: Create objective, requirements, API contracts, testing strategy, phased checklist. Set conductor status to `Planning`.

### 2. Start Transaction
```bash
ledgerful ledger start T<ID>-<name> --category <CAT> --message "Intent"
```

### 3. Implement (TDD)
- **Red**: failing tests asserting desired behavior; commit allowed.
- **Green**: production code; commit allowed.
- After: `ledgerful scan --impact`. Intermediate commits allowed. Clean gate applies only to finalizing ledger commit.

### 4. Targeted Checks
```bash
cargo nextest run --lib --bins -p <crate>
cargo clippy -p <crate> --all-targets -- -D warnings
```

### 5. Review Convergence
Review log: `conductor/<track>/review.md`. Severity rules in `AGENTS.md` (Review & Severity).

**Phase 1 — Primary:**
1. Subagent reviews code vs `spec.md` (completeness, regressions, placeholders).
2. Subagent resolves findings.
3. Repeat review → resolve until clean (including mediums unless strictly justified).

**Phase 2 — Cross-Model (only when phase 1 clean):**
1. Read-only cross-model review (see `codex-review` skill).
2. If findings: resolve → verify → re-run.
3. Repeat until cross-model output clean.

**Review for:** correctness, Rust idioms, architecture fit, missing tests, edge cases, regressions, contract drift, placeholders/stubs.

**Finding fields:** id, severity, description, source, files, required_fix, status, evidence.
**Statuses:** `open` | `fixed_pending_verification` | `verified_fixed` | `deferred` | `out_of_scope`.
**Closure:** Implementer marks `fixed_pending_verification`; reviewer/cross-model marks `verified_fixed`. Code change alone is not closure.

### 6. Manual Test
Required at every gate. Record in `plan.md` or final report:
- Visible behavior (happy path), relevant error path, any prior regression path.
- Exact command/input/output/result.
- **Determinism** (impact/verify/ledger): run twice on same repo state, diff, require byte-identical output.
- **Daemon/API tracks**: start `ai-brainsd`, hit live endpoint, confirm empty-state JSON shape matches contract.

### 7. Sync Contracts
Per `AGENTS.md` (Contracts). Required when `ai-brainsd` API payload, config gate, daemon behavior, or `ai-brains-contracts` DTOs change.

### 8. Full Gate
```powershell
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit ; ledgerful verify --scope full
```

### 9. Finalize
Gate clears only if: no open critical/high, mediums fixed or justified-deferred (cap ≤3), no unresolved regression, full gate green, manual evidence recorded, contracts synced, conductor updated, ledger clean after commit.

Then:
- Mark `plan.md` tasks complete. Set `conductor/conductor.md` entry to `Completed`.
- APPEND deferred medium/low_info findings from `review.md` to `conductor/ISSUES.md` (with track name).
- Add one-line evidence note.
- Pin non-obvious decisions: `ai-brains pin "DECISION: <what + why>" --tx-id <tx-id>`.

## ChangeGuard Integration

| Phase | Command | Purpose |
|-------|---------|---------|
| Session start | `ledgerful ledger status` | Detect untracked drift |
| Before implementation | `ledgerful ledger start` | Begin track transaction |
| Before edits | `ledgerful scan --impact` + `hotspots` | Blast radius, brittle files |
| After edits | `ledgerful verify --scope fast\|full` | Scoped or full CI gate |
| On commit | `ledgerful ledger commit` | Close transaction |

**Ledger categories**: `ARCHITECTURE`, `SECURITY`, `FEATURE`, `INFRA`, `REFACTOR`, `BUGFIX`, `DOCS`, `CHORE`.

**Skip ChangeGuard for**: format-only, scratch files, binary/media-only, lockfile-only churn, explicit user bypass.

**If commands fail**: continue with native checks if unavailable; reconcile drift if ledger dirty; report if `scan --impact` can't complete; use `--auto-index` if index is `[STALE]`; never edit `.changeguard/` state files directly.

## Tooling

- **ChangeGuard**: `ledgerful scan --impact`, `ledgerful search`, `ledgerful ask`, `ledgerful ledger start/commit/atomic`, `ledgerful verify --scope fast|full`, `ledgerful dead-code --threshold 0.75`. See the `changeguard` skill for full reference.
- **GitHub CLI**: `gh run list` (CI), `gh run view` (details), `gh pr diff` (review), `gh pr status`.
- **Dependency alerts**: `cargo tree -i <crate>@<version>` to find direct vs transitive. If transitive, upgrade the direct dep. If via git dep, verify upstream fix visibility. Record external handoffs in a conductor track. Run focused checks + `ledgerful verify` after dep changes.

## Research Precedence

1. Active file/spec (current context).
2. Conductor track (`spec.md` + `plan.md`).
3. `ledgerful ledger search` (provenance history).
4. `.agents/rules/*.md` (local rules).
5. `Docs/` (`PRD.md`, `Implementation-Plan.md`, `Engineering.md`).
6. `context7` for crate/library docs (Tokio, Axum, SQLCipher, Rusqlite).
7. `exa` for web search on ecosystem/current questions.

External research needed when: Rust 2024 behavior matters, recent crate API matters, CLI/framework docs matter, or version pin matters. Prefer `context7` → official docs → release notes → upstream changelog.

## Parallel Agents

Use when ≥2 implementation agents edit code concurrently. Skip for serial runs.

- **Max concurrent**: 3. Each agent needs own branch + git worktree.
- **Allowed only if**: non-intersecting scopes, no same files/modules, no conflicting shared API changes.
- **MUST NOT touch**: `conductor.md`, `Cargo.toml`, `Cargo.lock`, shared config, migrations, generated files, CI config.
- **Coordinator owns**: integration, conductor updates, dep/config/lockfile changes, ledger finalization, commits/pushes.
- **Command**: `git worktree add ..\ai-brains-<track> -b agent/<track>`.
- **Integration**: merge/cherry-pick one branch at a time; smallest clean diff; full gate after each; track not clear until integrated, reviewed, manually tested, recorded. Abandon/delete rejected or integrated worktrees.

## Final Report

After completing a track: tracks completed, files changed, checks/tests/manual evidence with exact commands, review-log summary by severity, deferred mediums/lows with justifications, confirmation deferred items appended to `conductor/ISSUES.md`, contracts synced, dependency docs/versions consulted, commits and push status, residual risks/follow-ups.

## Key Reference Documents

| Document | Purpose |
|----------|---------|
| `Docs/PRD.md` | Product vision and core requirements |
| `Docs/Implementation-Plan.md` | Master execution plan (Tracks) |
| `Docs/Deviations.md` | Architectural departures |
| `Docs/ci-tooling.md` | CI tool install commands |
| `AGENTS.md` | Unified project rules and mandates |
| `conductor/conductor.md` | Track registry |
| `conductor/ISSUES.md` | Unresolved debt |