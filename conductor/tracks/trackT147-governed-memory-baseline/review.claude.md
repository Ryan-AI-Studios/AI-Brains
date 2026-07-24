I now have all the information needed for the complete audit. Let me compile the formal report.

---

# Track Completion Audit ΓÇö T147

## Verdict: PASS WITH DEFERRED P3

---

## Scope Reviewed

- **Branch:** `feature/governed-memory-baseline`
- **Working tree vs HEAD:** 81 files modified, all unstaged (nothing in index); `.agents/skills/codex-review/SKILL.md` is ` M` (working-tree only, excluded from scope per spec).
- **New files audited:** `temp_env.rs`, `location.rs`, `legacy-v1-events.ndjson`, `expected-legacy-projections.json`, `common/mod.rs`, `common/governed_fixture.rs`, `governed_fixture_replay.rs`, `shadow.rs`, `shadow_vault_refuses_live_target.rs`, `shadow-vault.ps1`.
- **Modified files audited:** `Cargo.toml`, `rustfmt.toml`, `rust-toolchain.toml`, `ai-brains-core/src/lib.rs`, `ai-brains-path/src/lib.rs`, `ai-brains-cli/src/main.rs`, `elevation.rs`, `commands/mod.rs`, `nightly_summarizes_large_session.rs`, `AGENTS.md`, `onboarding/SKILL.md`, `ci-tooling.md`, `dev-check.ps1`, `conductor.md`, `deferred.md`, `review.md`.

---

## Requirement and DoD Matrix

| Req | Description | Status | Evidence |
|-----|-------------|--------|----------|
| R-ED1 | `workspace.package.edition = "2024"` | Γ£à | `Cargo.toml:28` |
| R-ED1 | `rustfmt.toml edition = "2024"` | Γ£à | `rustfmt.toml:1` |
| R-ED1 | `rust-toolchain.toml` stays `1.95.0` | Γ£à | `rust-toolchain.toml:2` |
| R-ED1 | Documented floor Rust 1.85+ | Γ£à | Comment in `rust-toolchain.toml:1` |
| R-ED1 | Clippy `-D warnings` green | Γ£à | Gate evidence 2026-07-24 |
| R-ED2 | No bare `set_var`/`remove_var` | Γ£à | All 9 call sites are inside `unsafe {}` blocks with SAFETY comments |
| R-ED2 | Production sites use single documented `unsafe` pattern | Γ£à | `main.rs:644-647, 680-684`; `elevation.rs:84-87` |
| R-ED2 | Tests use `TempEnv` | Γ£à | `nightly_summarizes_large_session.rs:5,33` |
| R-ED2 | No `unused_unsafe` trip | Γ£à | Clippy GREEN (edition 2024 makes these required) |
| R1 | Fixture replay is deterministic; matches golden exactly | Γ£à | `governed_fixture_replay__synthetic_events__stable_selected_projections`; golden committed |
| R1 | Stable columns only; FTS as count | Γ£à | `governed_fixture.rs:109-221` excludes embeddings, `memory_id` |
| R2 | Two independent loads produce identical snapshots | Γ£à | `governed_fixture_replay__load_twice_on_fresh_vaults__identical_snapshots` |
| R2 | Duplicate `event_id` append fails cleanly | Γ£à | Same test; asserts error contains `UNIQUE`/`immutable`/`Failed to append` |
| R3 | `resolve_best_effort` exported | Γ£à | `ai-brains-path/src/lib.rs:21` |
| R3 | `paths_refer_to_same_location` exported | Γ£à | `lib.rs:18` |
| R3 | `path_is_same_or_inside` exported | Γ£à | `lib.rs:18` |
| R3 | No third canonicalize stack | Γ£à | All use `dunce`-compatible `strip_extended_length_prefix` + drive normalize; no raw `Path::canonicalize` equality |
| R3 | Refuse same source/dest | Γ£à | `shadow.rs:120-124`; test `shadow_create__same_source_and_destination__refuses` |
| R3 | Refuse dest == live vault | Γ£à | `shadow.rs:127-131`; test `shadow_create__destination_equals_live_vault__refuses` |
| R3 | Refuse dest inside live vault parent | Γ£à | `shadow.rs:132-140`; test `shadow_create__destination_inside_live_vault_parent__refuses` |
| R3 | Refuse dest as reparse/symlink | Γ£à | `shadow.rs:144-154`; test `shadow_create__destination_reparse_or_symlink__refuses` |
| R3b | Live vault resolution: env ΓåÆ `~/.ai-brains/.env` ΓåÆ None | Γ£à | `shadow.rs:42-68`; correctly mirrors main.rs env chain |
| R4 | Dry-run writes nothing | Γ£à | `shadow.rs:219-232`; test asserts dest absent, manifest absent, source length unchanged |
| R5 | Default redact turn content | Γ£à | `main.rs:809` `let redact = !*no_redact_turn_content`; `shadow.rs:99-112` |
| R5 | `shadow-manifest.json` written | Γ£à | `shadow.rs:267-279`; test verifies all fields |
| R5 | `payload_hash` recomputed after redact | Γ£à | `shadow.rs:109-111` via `compute_payload_hash` |
| R6 | No live vault mutation in tests | Γ£à | Tests use `Command::env(...)` (not process env); `--no-project-context`; tempdir vaults only |
| 3.C.4 | `scripts/shadow-vault.ps1` thin wrapper | Γ£à | Present; `Requires -Version 5.1`; delegates to `ai-brains shadow @RemainingArgs` |
| 3.D.1 | Full gate recorded in `review.md` | Γ£à | 7-command table; fmt/clippy/nextest 426/deny/audit/ledgerful/dev-check.ps1 all GREEN |
| 3.D.2 | Tool pins bumped: nextest 0.9.140 / deny 0.20.2 / audit 0.22.2 | Γ£à | `ci-tooling.md:7-11`; `dev-check.ps1:14-16` |
| 3.D.3 | `deferred.md` #6 struck | Γ£à | `deferred.md:39-42` struck-through with dated evidence |
| DoD | Workspace edition 2024; toolchain ΓëÑ1.85 pinned 1.95.0 | Γ£à | `Cargo.toml:28`, `rust-toolchain.toml` |
| DoD | All env mutation sites compile without `unused_unsafe` | Γ£à | Gate GREEN |
| DoD | `TempEnv` exists and used by tests that mutate env | Γ£à | `ai-brains-core::temp_env`; used in brain test |
| DoD | Fixture + shadow REDΓåÆGREEN complete | Γ£à | `review.md` internal review rounds |
| DoD | Path helpers used by shadow, not a third stack | Γ£à | `shadow.rs:13` imports all three helpers from `ai-brains-path` |
| DoD | Gate + ledgerful verify recorded | Γ£à | `review.md` |
| DoD | Tool pins updated | Γ£à | `ci-tooling.md`, `dev-check.ps1` |
| DoD | deferred #6 cleared | Γ£à | `deferred.md:39-42` |
| DoD | Diff excludes unrelated skill file | Γ£à | `git diff --cached` returns empty; `codex-review/SKILL.md` is ` M` only |
| DoD | Stop before merge/push | Γ£à | `conductor.md` shows "In Progress"; ledger not committed |

---

## Findings (P0ΓÇôP3)

### T147-AUD-F1 ΓÇö P3 (Low) | DEFERRED

**Confidence:** High  
**Location:** `.agents/skills/onboarding/SKILL.md:105-107`  
**Problem:** Version table in onboarding skill shows pre-T147 pins (`cargo-nextest 0.9.137`, `cargo-deny 0.19.4`, `cargo-audit 0.22.1`). The spec's pin-bump scope (`Docs/ci-tooling.md`, `scripts/dev-check.ps1`) is satisfied; the SKILL.md table was updated for TempEnv/edition wording but the pin rows were not synchronized.  
**Evidence:** `SKILL.md:105-107` shows old pins; `ci-tooling.md:7-11` and `dev-check.ps1:14-16` correctly show 0.9.140 / 0.20.2 / 0.22.2.  
**Correction:** Update the three version rows in `SKILL.md` to match `ci-tooling.md`.  
**Verification:** `grep 0.9.140 .agents/skills/onboarding/SKILL.md`  
**Deferrable:** Yes ΓÇö authoritative sources are correct; stale convenience table is non-blocking.

---

### T147-AUD-F2 ΓÇö P3 (Low) | DEFERRED

**Confidence:** High  
**Location:** `crates/ai-brains-store/tests/common/governed_fixture.rs:23-26`  
**Problem:** `fixture_sql_key()` generates a random `DataKey` and is `pub`, but is never called anywhere in the fixture infrastructure ΓÇö `load_from_path` consistently uses `fixture_zero_sql_key()`. The `#![allow(dead_code)]` in `common/mod.rs:1` silences the warning, so it compiles cleanly. The function's intent is not documented.  
**Evidence:** `governed_fixture.rs:23-26` defines the function; no call site exists in the test tree; `load_from_path:77` calls `fixture_zero_sql_key()`.  
**Correction:** Either remove `fixture_sql_key()` or add a doc comment explaining the intended use.  
**Verification:** `cargo clippy -p ai-brains-store` with `#![allow(dead_code)]` removed from `common/mod.rs` would surface the warning.  
**Deferrable:** Yes ΓÇö `#![allow(dead_code)]` is intentional on the common module; no correctness impact.

---

## Completeness Sweep

| Area | Complete? | Notes |
|------|-----------|-------|
| Edition 2024 migration | Γ£à | `Cargo.toml`, `rustfmt.toml`, let-chain collapses across workspace |
| TempEnv RAII helper | Γ£à | Full implementation with nested-guard support; self-tests |
| Production unsafe env | Γ£à | 3 sites (main.rs ├ù2 blocks, elevation.rs); SAFETY comments present |
| Test env (TempEnv) | Γ£à | `nightly_summarizes_large_session.rs` |
| Path helpers exported | Γ£à | All 3 from `ai-brains-path/src/lib.rs` |
| Location helpers unit-tested | Γ£à | 8 tests in `location.rs` cover `\\?\` strip, drive-case, inside-parent, sibling-prefix |
| Fixture NDJSON | Γ£à | 5 fixed envelopes; valid v4 UUIDs; correct `occurred_at` ordering |
| Golden JSON committed | Γ£à | All 5 projection tables; `memory_id` omitted for determinism |
| Golden regen gate | Γ£à | `UPDATE_GOVERNED_GOLDEN=1` required; panics with instructions otherwise |
| Duplicate event_id test | Γ£à | Asserts `is_err()` and error message pattern |
| Shadow CLI wiring | Γ£à | `Commands::Shadow` handled pre-`AppContext` in `run()`; no vault path required |
| Shadow refusals (4 rules) | Γ£à | All tested; refusals fire before dry-run check |
| Shadow redaction | Γ£à | `UserPromptRecorded` + `AssistantFinalRecorded` content ΓåÆ `[REDACTED]`; hash recomputed |
| Source not migrated | Γ£à | `VaultConnection::open` only; no `.migrate()` call on source |
| Shadow manifest fields | Γ£à | `version`, `created_at`, `source_path`, `destination_path`, `source_fingerprint`, `redaction_policy`, `event_count`, `dry_run` |
| `--no-redact-turn-content` flag | Γ£à | Tested; manifest reflects policy; content preserved |
| `shadow-vault.ps1` | Γ£à | Requires PS 5.1; delegates via `ai-brains shadow @RemainingArgs` |
| AGENTS.md / onboarding wording | Γ£à | Edition 2024 and TempEnv patterns documented accurately |
| ci-tooling.md pin bump | Γ£à | Correct values |
| dev-check.ps1 `$Required` | Γ£à | Correct values |
| deferred.md #6 | Γ£à | Struck with dated PS 5.1 evidence |
| conductor.md registered | Γ£à | T147 row present; "In Progress" (correct pre-close state) |
| Unrelated skill excluded | Γ£à | `codex-review/SKILL.md` ` M` only; git index is empty |

---

## Wiring and Regression Review

**Edition 2024 let-chain collapses:** The implementation notes describe widespread `collapsible_if` fixes across the workspace. These are behavior-preserving rewrites (clippy `--fix` applied). Gate is GREEN on all 426 tests, confirming no regressions were introduced.

**Shadow pre-AppContext routing:** `run()` in `main.rs:798-820` intercepts `Commands::Shadow` before building `AppContext`, so shadow create never needs a global vault path configured. The resolution chain in `resolve_live_vault_path()` is independent and mirrors what `main()` loads at startup. Correct.

**Refusal ordering:** `refuse_unsafe_destination` fires at line 187, before source existence check (line 189) and before dry-run check (line 219). This means refusals always execute regardless of dry-run. Tests that use `--dry-run` to exercise refusals exercise the correct production code path.

**Source mutation guard:** The dry-run test verifies `source_meta_after.len() == source_len_before`. `VaultConnection::open` without `.migrate()` does not alter the main database file (WAL siblings are separate files not checked). Test passes; F5 fix verified in production code.

**TempEnv nesting:** The unit test at `temp_env.rs:94-100` exercises a nested drop (`_g` containing `_inner`) and verifies LIFO restore behavior. Γ£ô

**R3 path helper boundary case (sibling prefix):** `path_is_same_or_inside__sibling_prefix__false` verifies `C:\foobar\x` is not inside `C:\foo` (separator boundary enforced by appending `\` before `starts_with`). Γ£ô

**Golden immutability:** If `UPDATE_GOVERNED_GOLDEN` is not set to `"1"`, the test panics with clear instructions rather than silently overwriting. A future schema migration that changes projection output will cause a clear failure, not a silent golden corruption. Γ£ô

---

## Verification Evidence

From `review.md` (gate 2026-07-24):

| Command | Result |
|---------|--------|
| `cargo fmt --check` | GREEN |
| `cargo clippy --workspace --all-targets -- -D warnings` | GREEN |
| `cargo nextest run --workspace` | 426 passed, 0 skipped |
| `cargo deny check` | GREEN |
| `cargo audit` | GREEN (RUSTSEC-2026-0190 allowed per `deny.toml`) |
| `ledgerful verify --scope full` | GREEN ΓÇö all 5 steps SUCCESS |
| `powershell.exe -NoProfile -File scripts\dev-check.ps1 -CheckOnly` | GREEN ΓÇö pins 0.9.140/0.20.2/0.22.2 |

The gate validates that all R-ED1/R-ED2/R1ΓÇôR6 behaviors compile and pass under the full test suite in a real environment.

---

## Deferred Candidates

| ID | Severity | Description | Rationale |
|----|----------|-------------|-----------|
| T147-AUD-F1 | P3 Low | Onboarding SKILL.md version table stale (0.9.137 vs 0.9.140 etc.) | Authoritative sources correct; convenience reference only |
| T147-AUD-F2 | P3 Low | `fixture_sql_key()` dead code in governed_fixture.rs | Suppressed by `#![allow(dead_code)]`; no correctness impact |

No P0/P1/P2 findings. The accepted residuals from the internal review cycle (T147-F4: non-deterministic `memory_id`, T147-F7: `TempEnv` public) are properly documented and are not implementation gaps.

**Out-of-scope items confirmed out of scope:**
- Source vault not migrated (production assumption; no migration `0020+` added)
- Shadow TOCTOU / full handle-based path design (P6)
- `conductor.md` still shows "In Progress" ΓÇö correct pre-merge state per DoD stop condition
- Phase 5 actions (ledger commit, mark Complete) ΓÇö orchestrator-owned post-review

---

## Completion Decision

**PASS WITH DEFERRED P3**

All mandatory requirements (R-ED1, R-ED2, R1ΓÇôR6, 3.C, 3.D) are implemented, wired end-to-end, and verified by the full CI gate. Tests prove required behavior and would catch regressions. No placeholders, stubs, silent fallbacks, or skipped test cases were found. The two P3 findings are cosmetic documentation gaps that do not affect correctness, safety, or the gate.

**Pre-merge actions for orchestrator:**
1. Fix T147-AUD-F1 (update `onboarding/SKILL.md` version pins) ΓÇö can be done in the finalization commit.
2. Inspect staged files and exclude `codex-review/SKILL.md` (working tree is clean from staging perspective; just don't `git add` it).
3. Mark `conductor.md` T147 ΓåÆ **Complete**.
4. Commit ledger transaction `46bcef1f-4a50-49f0-b175-b2d76d07da77`.
5. Stop before push to `main` ΓÇö explicit Ryan approval required.
