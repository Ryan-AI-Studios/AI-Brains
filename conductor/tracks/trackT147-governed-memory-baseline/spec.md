# T147 — Governed Memory Baseline + Edition 2024 + Shadow Fixtures

- **Track ID:** T147-GovernedMemoryBaseline
- **Phase:** P0 (master plan: `.hermes/plans/2026-07-23_204630-memory-control-plane-successor.md`)
- **Execution repo:** `C:\dev\AI-Brains`
- **Status:** Pending
- **Category:** INFRA / FEATURE
- **Depends on:** T146 (docs/architecture Complete)
- **Branch (suggested):** `feature/governed-memory-baseline`
- **Relevant ADRs:** ADR-0010, ADR-0014

## 1. Objective

1. Align the workspace with the documented Rust standard: **edition 2024** on a toolchain **≥ 1.85** (repo currently pins **1.95.0** via `rust-toolchain.toml`; do not downgrade).
2. Prove current vault/event/projection behavior with deterministic synthetic fixtures.
3. Add safe **shadow vault** tooling so dogfood evaluation never mutates the live vault.
4. Record full baseline gate evidence before P1 domain work.

## 2. Context (inspected)

| Fact | Value |
|------|--------|
| `rust-toolchain.toml` | `channel = "1.95.0"` (meets ≥1.85; **not** 1.87 — 1.95 is correct) |
| Workspace edition today | `"2021"` in root `Cargo.toml` + `rustfmt.toml` |
| AGENTS.md claim | Incorrectly already says Edition 2024 |
| `set_var` / `remove_var` | Still safe under 2021; become `unsafe fn` under edition 2024 |
| Known production call sites | `cli/src/main.rs` (`set_var`/`remove_var`), `cli/src/elevation.rs` (`set_var`) |
| Known test call site | `brain/tests/nightly_summarizes_large_session.rs` |
| `TempEnv` / `serial_test` | **Not present** today |
| Migrations | End at `0019_embedding_timestamp.sql` — P0 adds none |
| Dirty unrelated | `.agents/skills/codex-review/SKILL.md` — never stage |
| Replay truncate set | `memory_projection`, `turn_projection`, `session_projection`, `project_projection`, `memory_fts` only |

## 3. In scope

### 3.A Edition 2024 workspace migration (mandatory first slice)

1. Set workspace `edition = "2024"` and `rustfmt.toml` `edition = "2024"`.
2. Keep `rust-toolchain.toml` on **1.95.0** (or higher if the user later pins up). Document **minimum supported** as **1.85.0** (edition 2024 floor). Do **not** pin 1.87 unless that channel is intentionally chosen and verified.
3. Introduce a small test-only (or core-shared) **`TempEnv` RAII** helper that restores prior env values on drop; use it in tests that must mutate env.
4. Fix all production and test `std::env::set_var` / `remove_var` call sites for edition 2024:
   - Production: explicit `unsafe` with a one-line safety comment (single-threaded CLI startup / elevate handoff), **or** a thin internal wrapper documented as the only allowed mutation site.
   - Tests: `TempEnv` (+ `#[serial(env)]` via `serial_test` **only if** a test binary shares a process; nextest process isolation remains the primary isolation story).
5. Align `AGENTS.md` and onboarding skill wording with reality after the migration (edition 2024 true; TempEnv exists).
6. Prove: `cargo check --workspace`, clippy `-D warnings`, and nextest still green.

### 3.B Deterministic fixture infrastructure

1. `fixtures/governed-memory/legacy-v1-events.ndjson` (synthetic, non-sensitive).
2. `fixtures/governed-memory/expected-legacy-projections.json`.
3. Store test helper + replay stability / fresh-vault idempotency tests.

### 3.C Shadow vault tooling

1. Shared path helpers in `ai-brains-path` (export/normalize; no third canonicalize stack).
2. CLI `ai-brains shadow create` with live-vault resolution matching CLI env chain.
3. Safety refusals, dry-run, default turn-content redaction, `shadow-manifest.json`.
4. Thin `scripts/shadow-vault.ps1` wrapper.

### 3.D Baseline gates and hygiene

1. Full CI gate + `ledgerful verify --scope full` recorded in `review.md`.
2. Bump tool minimum pins to installed versions after green gate (nextest 0.9.140, deny 0.20.2, audit 0.22.2).
3. Strike `conductor/deferred.md` item #6 if still present (dev-check parse error — already fixed; re-verify).

## 4. Out of scope

- Domain IDs, new event kinds, migrations `0020+`, briefings, Tauri, sync, erasure (P1+).
- Soft-canonicalize / full TOCTOU handle design (note for P6 connectors).
- T142 / T143 work.
- Staging `.agents/skills/codex-review/SKILL.md`.
- Merge/push to `main` without explicit Ryan approval.
- Downgrading the toolchain below current 1.95.0.

## 5. Requirements

### R-ED1 — Edition and toolchain

| Setting | Required |
|---------|----------|
| `workspace.package.edition` | `"2024"` |
| `rustfmt.toml` edition | `"2024"` |
| `rust-toolchain.toml` channel | stay on `1.95.0` unless intentionally upgraded **upward** |
| Documented floor | Rust **1.85+** (edition 2024 availability) |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` green |

### R-ED2 — Env mutation under edition 2024

- No bare safe `set_var`/`remove_var` after the bump (they will not compile without `unsafe`).
- Production sites use a single documented pattern (prefer `unsafe { std::env::set_var(...) }` with SAFETY comment at CLI startup / elevate handoff).
- Tests that need env mutation use `TempEnv` (restore on drop). Prefer CLI flags / `assert_cmd` env over process env when possible.
- Do **not** wrap edition-2021-safe calls in `unsafe` before the edition bump (that would trip `unused_unsafe`).

### R1 — Synthetic fixture replay is deterministic

Load `legacy-v1-events.ndjson` into a fresh tempfile vault (migrations through 0019), append envelopes **as-is** (fixed `event_id`, `occurred_at`, `payload_hash` — **not** via `EventBuilder`), export selected projection rows, match golden JSON exactly.

Selected tables: `project_projection`, `session_projection`, `turn_projection`, `memory_projection` (stable columns; exclude embedding blobs). FTS: count only.

### R2 — Fresh-vault idempotency

Two independent loads on fresh vaults produce identical snapshots. Second append of same `event_id` into a populated vault must fail cleanly (immutable events).

### R3 — Path safety via `ai-brains-path`

Export/add:

- `resolve_best_effort` (today private in `symlink.rs`)
- `paths_refer_to_same_location(a, b) -> bool`
- `path_is_same_or_inside(candidate, root) -> bool`

Use `dunce`-compatible / `\\?\`-stripped normalization consistent with `BackupService`. No naive string-only `Path::canonicalize` equality.

Shadow refuses when:

1. source and destination are the same location;
2. destination equals resolved **live vault**;
3. destination is inside the live vault’s parent directory (configurable precise rule documented in plan);
4. destination exists as reparse/symlink (reuse `artifact_security` refuse patterns where applicable).

Full handle-based TOCTOU is **out of scope** for P0.

### R3b — Live vault resolution

Same chain as CLI:

1. `AI_BRAINS_VAULT_PATH` after normal env loading;
2. else `~/.ai-brains/.env`;
3. else only enforce same-path(source, dest) and say so on stderr.

### R4 — Dry-run writes nothing

### R5 — Default redact turn content; write `shadow-manifest.json`

### R6 — No live vault mutation in tests

## 6. CLI contract

```text
ai-brains shadow create
  --source <path>
  --destination <path>
  --redact-turn-content / --no-redact-turn-content   # default: redact
  --dry-run
```

Key: `--key` / `AI_BRAINS_KEY` for opening source. Destination is a new vault.

## 7. Files

### Create

| Path | Purpose |
|------|---------|
| `conductor/tracks/trackT147-…/spec.md` | This file |
| `conductor/tracks/trackT147-…/plan.md` | Checklist |
| `conductor/tracks/trackT147-…/review.md` | Evidence |
| `crates/ai-brains-core/src/test_env.rs` **or** test-util module | `TempEnv` (prefer core + `#[cfg(feature)]` / re-export only if production needs it; else `dev-dependency` helper crate-local is acceptable — **prefer** one shared place under `ai-brains-core` with `pub` for tests via `temp_env` module gated by `cfg(any(test, feature = "test-util"))` **or** a tiny `ai-brains-test-util` only if core coupling is painful) |
| `fixtures/governed-memory/*` | NDJSON + golden |
| `crates/ai-brains-store/tests/common/*` | Fixture loader |
| `crates/ai-brains-store/tests/governed_fixture_replay.rs` | R1/R2 |
| `crates/ai-brains-cli/src/commands/shadow.rs` | Shadow CLI |
| `crates/ai-brains-cli/tests/shadow_vault_refuses_live_target.rs` | R3/R4 |
| `scripts/shadow-vault.ps1` | Wrapper |

### Modify

| Path | Change |
|------|--------|
| `Cargo.toml` | `edition = "2024"` |
| `rustfmt.toml` | `edition = "2024"` |
| `rust-toolchain.toml` | Keep `1.95.0`; comment minimum 1.85 if useful |
| `crates/ai-brains-cli/src/main.rs` | Edition-2024-safe env mutation |
| `crates/ai-brains-cli/src/elevation.rs` | Edition-2024-safe env mutation |
| `crates/ai-brains-brain/tests/nightly_summarizes_large_session.rs` | `TempEnv` |
| `crates/ai-brains-path/src/*` | Export path equality helpers |
| `crates/ai-brains-cli/src/commands/mod.rs`, `main.rs` | Shadow wiring |
| `AGENTS.md`, onboarding skill | Match edition 2024 + real TempEnv |
| `Docs/ci-tooling.md`, `scripts/dev-check.ps1` | Pin bump after green gate |
| `conductor/conductor.md` | Register T147 |
| `conductor/deferred.md` | Strike #6 when verified |

### Do not modify

- Migrations `0001`–`0019`
- Unrelated skill file
- Live vault files

## 8. Migration / rollback

- **Edition:** revert `edition` fields + env `unsafe`/`TempEnv` if abandon (single commit slice preferred so rollback is clean).
- **Fixtures/shadow:** additive; delete files to roll back.
- **No SQL migrations.**

## 9. Definition of Done

- [ ] Workspace edition is `2024`; rustfmt edition is `2024`; toolchain remains ≥1.85 (pinned 1.95.0).
- [ ] All `set_var`/`remove_var` sites compile under edition 2024 without `unused_unsafe` failures under 2021-era mistakes.
- [ ] `TempEnv` exists and is used by tests that mutate env.
- [ ] Fixture + shadow RED→GREEN complete.
- [ ] Path helpers used by shadow (not a third canonicalize stack).
- [ ] Full gate + ledgerful verify recorded in `review.md`.
- [ ] Tool pins updated after green gate.
- [ ] deferred #6 cleared or re-opened with new evidence.
- [ ] Diff excludes unrelated skill file.
- [ ] Stop before merge/push to `main`.

## 10. Stop conditions

- Edition 2024 introduces widespread non-env breakage (RPIT, etc.) beyond a small fix set — stop and report before expanding scope.
- Shadow redaction appears to need envelope v2 or SQL migrations.
- Full gate fails outside T147 scope.
- User environment cannot provide Rust ≥1.85 (should not happen: repo pins 1.95.0).
```