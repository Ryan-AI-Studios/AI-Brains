## Verdict: PASS WITH DEFERRED P3

## Scope Reviewed

Full working-tree diff on `feat/t205-global-dotenv-key-gapfill` against spec.md/plan.md F1ΓÇôF36, AC1ΓÇôAC12: `main.rs` always-merge loader + `resolve_user_home_for_dotenv`; `common/mod.rs` empty-home isolation (`isolate_empty_home`, `hermetic_bin_no_key`); new `tests/global_dotenv_key_gapfill.rs` (AC1ΓÇôAC4); `vault_key_bootstrap.rs` / `daemon_status_vault_independence.rs` re-green under F11; `shadow.rs`/`migrate.rs` comment freshness (M2); docs (INSTALL/CAPABILITIES/OPERATIONS/CHANGELOG); `.claude/skills/ai-brains/SKILL.md`; `.gitignore`. Read-only; no files modified by this review.

## Requirement and DoD Matrix

| AC/F | Status | Evidence |
|---|---|---|
| AC1 always-merge, path set, KEY only global | **Met** | `main.rs:1771ΓÇô1794`; `global_dotenv_key_gapfill.rs:46-104` |
| AC2 shell KEY wins | **Met** | dotenvy non-override (`from_path`, never `_override`); test `ac2__shell_key_wins_over_global_key` |
| AC3 project KEY wins over global | **Met** | test `ac3__project_env_key_wins_over_global` |
| AC4 `--no-project-context` still gap-fills | **Met** | main.rs loader runs unconditionally after the `if !no_project_context` block (main.rs:1756-1794); test `ac4__ΓÇª` |
| AC5 T113 regression | **Met** | `smoke.rs` ~2498 `env_var_precedence__project_env_overrides_global_env` untouched (no diff) |
| AC6/F11 no-key tests immune to real dev global KEY | **Met** | `isolate_empty_home` sets both `USERPROFILE`+`HOME` to a process-lifetime tempdir (`common/mod.rs:88-135`); all F31 targets (`doctor__missing_key__vault_open_skipped`, `assert_appcontext_missing_key_family`, `init__missing_key__generates_and_prints_bootstrap`, `init__generated_key__opens_doctor`) migrated to `hermetic_bin_no_key()` |
| AC7 INSTALL | **Met** | quoted-value guidance (F23) + always-merge order documented |
| AC8 CAPABILITIES one-liner | **Met** | exact F29 wording present at both the summary table and ┬º14 |
| AC9 skill honesty | **Met** (see Finding P3-1) | key-home section accurate but rewrite scope exceeds F18/L5 |
| AC10 CHANGELOG | **Met** | correct `[Unreleased]/### Changed` entry |
| AC11 full gate green | **Unverified this session** ΓÇö see Verification Evidence |
| AC12 helper doc matches reality | **Met** | `hermetic_bin_no_key` doc explicitly states global dotenv still merges; isolation is via empty home |
| F2 elevation asymmetry documented | **Met** | `elevation.rs` untouched; CAPABILITIES/CHANGELOG state the override-only-for-elevated-child asymmetry |
| F22 dual home env | **Met** | every isolation site sets both `USERPROFILE` and `HOME` |
| F25/F33 soft parse warn | **Met** | `tracing::warn!` on `from_path` Err, non-fatal |
| F28/M2 comment freshness | **Met** | `shadow.rs`/`migrate.rs` updated, no longer imply global is "else unset" only |
| F32/L4 gitignore | **Met** | `vault.db.plain-*` added; confirmed an actual untracked `vault.db.plain-20260804104801` in the tree is ignored, not staged |

## Findings (P0ΓÇôP3)

**P3-1 ΓÇö Skill rewrite exceeds F18/L5 trim scope.** `.claude/skills/ai-brains/SKILL.md` diff (94/91 lines) adds substantial content beyond key/dotenv honesty ΓÇö new "Multi-repo model," "Phase 0: Health," "Governed discovery," restructured command-summary sections. Spec explicitly calls this out as a pre-existing seed risk ("Skill: diff-trim if rewrite beyond key/dotenv (L5)"). Content spot-checked (`--global`, `--semantic`, `--project-id` flags) and is factually accurate against current `main.rs`, so this is not a correctness bug, just an un-trimmed scope-creep item the spec asked to flag. No action required to ship; worth a follow-up trim if the team wants SKILL.md diffs to stay narrowly scoped to the track that touches them.

No P0/P1/P2 found.

## Completeness Sweep

- Changed-file list matches the spec's touch map exactly (main.rs, common/mod.rs, new test file, vault_key_bootstrap.rs, daemon_status_vault_independence.rs, shadow.rs, migrate.rs, INSTALL/CAPABILITIES/OPERATIONS/CHANGELOG, skill, gitignore) ΓÇö no undeclared surfaces touched.
- `dirs::home_dir()` call sites audited repo-wide: `backup.rs`, `antigravity.rs`, `ai-brains-api-server/token.rs`, `ai-brainsd/*` remain plain `dirs::home_dir()` ΓÇö all correctly out of T205 DoD (F14 daemon out-of-scope; backup.rs/antigravity.rs not in touch map, unaffected by this change).
- `elevation.rs` confirmed untouched (F2/F26 ΓÇö override semantics for elevated child unchanged).
- No stale "fallback"/"unset" dotenv phrasing left in `Docs/` grep sweep.
- Conductor tracking files (`conductor.md`, `deferred.md`) still show T205 as "≡ƒôï Planning" ΓÇö correct at this stage; final Completed/strike edits are D2 closeout, not part of this review gate.

## Wiring and Regression Review

- Load order in `main_inner` verified line-by-line: project block (`!no_project_context`) unchanged in placement; global merge block moved outside/after it and now unconditional ΓÇö matches F1ΓÇôF4 exactly.
- `resolve_user_home_for_dotenv` (USERPROFILEΓåÆHOMEΓåÆdirs::home_dir, trimmed non-empty) is a new named helper, used consistently by the loader; `shadow.rs::resolve_live_vault_path` independently applies the same USERPROFILE/HOME-first order for consistency with the main loader (defensive re-read, not a second override loader ΓÇö matches F35).
- Let-chain syntax (`if let ... && let Err(...) = ...`) has multiple existing precedents in this codebase (`antigravity.rs`, `backup.rs`, `control-plane/*`) and toolchain is pinned to 1.95.0 (edition 2024) ΓÇö compiles.
- `tempfile::TempDir::keep()` used in `common/mod.rs` ΓÇö valid API at pinned `tempfile 3.27.0` (Cargo.lock verified).
- Test helper `hermetic_bin_no_key()` correctly composes: ambient strip ΓåÆ empty-home redirect ΓåÆ explicit KEY/ALLOW_ZERO_KEY removal ΓåÆ `--no-project-context`; no duplicate-flag or ordering issues found across all call sites.
- `smoke.rs` T113 precedence test (AC5 lock) has zero diff ΓÇö regression surface confirmed stable.

## Verification Evidence

**Could not execute `cargo build`/`cargo nextest`/any compiled `.exe` this session** ΓÇö the Bash tool auto-denies cargo invocations and direct binary execution under the current "don't ask" permission mode (only `git`/`ls`-class commands were permitted). This is a session/tooling limitation, not a code concern; I was unable to independently confirm the "Full nextest 2024 pass" claim in the task brief.

Circumstantial evidence gathered in place of execution:
- A previously-built `global_dotenv_key_gapfill-*.exe` in `target/debug/deps/` is timestamped **after** the current `global_dotenv_key_gapfill.rs` source file, indicating a successful build post-edit occurred earlier in this branch's history.
- Static line-by-line trace of `main_inner()`, the new test file, and all three edited test files shows internally consistent logic with no obvious compile or logic errors.
- `.gitignore` change independently confirmed effective against a real untracked `vault.db.plain-20260804104801` file sitting in the working tree (git reports it `!!` ignored).

**Recommend the user re-run `cargo nextest run` (or trigger CI) before merge**, since this review's tooling could not do so.

## Deferred Candidates

- P3-1 (skill trim) ΓÇö soft, no action required to ship per F18 disposition ("Accept" with trim-if-broad noted, not a blocker).
- Everything else in spec's own "Residual after ship" (daemon VAULT_KEY silent-zero, monorepo dotenv parent search) is explicitly out of T205's DoD and correctly left untouched.

## Completion Decision

Code and docs changes are complete, internally consistent, correctly scoped to the touch map, and match every frozen decision (F1ΓÇôF36) and AC1ΓÇôAC12 by static/code review. The only deviation is the SKILL.md breadth (P3, non-blocking, spec pre-acknowledged). **The one gap is that I could not execute the build/test suite in this session** due to tool permission restrictions ΓÇö this is a verification-evidence gap, not a defect found in the code. Given the static review is clean and the spec states the full gate already passed, I'm issuing **PASS WITH DEFERRED P3**, conditioned on the user (or CI) confirming `cargo nextest run` is green before merge.
