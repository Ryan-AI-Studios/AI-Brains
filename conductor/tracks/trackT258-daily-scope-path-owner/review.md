# T258 review log — Daily Scope = path owner

**Status:** Phase-1 R1/R1b clean; Codex CX1 **product PASS** (process P1-1 = full gate + closeout outstanding at review time)
**HEAD (implement):** `4707919` `feat(cli): T258 project adopt-path (print-only; --write-env --yes)`
**Red:** `a5dd902` `test(T258): red adopt-path ACs (unknown subcommand)`
**FEATURE TX:** `6962a7b8-ff3a-4c0b-90cb-b3167d993335`
**Reviewer:** implementer (Grok)

## Scope

`project adopt-path` print-only remediator + `--write-env --yes` rewrite of cwd `.env` `AI_BRAINS_PROJECT_ID` only. Whoami remediations name the verb. T240 F2 stands. Live repo `.env` not written.

## DoD / AC matrix (R1)

| AC / DoD | Status | Evidence |
|----------|--------|----------|
| AC1 print-only human, no write | **met** | `project_adopt_path__print_only__names_owner_no_write` PASS |
| AC2 `--write-env` sans `--yes` exit 2 | **met** | `…write_env_without_yes__exit_2_no_write` PASS |
| AC3 write rewrites only PROJECT_ID | **met** | `…write_env_yes__rewrites_only_project_id` PASS |
| AC4 missing `.env` create | **met** | `…missing_env__write_creates_project_id_only` PASS |
| AC5 already-bound | **met** | `…already_bound__exit_0_no_rewrite` PASS (human + JSON) |
| AC6 no path owner exit 1 | **met** | `…no_path_owner__exit_1_no_write` PASS |
| AC7 whoami remediations | **met** | `project_whoami__mismatch__remediations_name_adopt_path` PASS |
| AC8 T240 suite | **met** | `project_identity_convergence` 9/9 PASS |
| AC9 `context.rs` untouched | **met** | `git diff` no `context.rs`; `test_context_new_project_rotates_id` PASS |
| AC10 no DTO / pins / events | **met** | `Cargo.lock`/`Cargo.toml` unchanged; `adopt_write` is `fs::write` only; no `context::run` / `ensure_project_and_session_exists` |
| AC11 docs | **met** | CAPABILITIES adopt-path row; WORKFLOWS §0; CHANGELOG T258; no `7d97a456` in WORKFLOWS |
| AC12 `--yes` sans `--write-env` clap 2 | **met** | `…yes_without_write_env__clap_exit_2` PASS |
| AC13 JSON print-only keys | **met** | `…format_json__print_only_keys` PASS |
| AC14 reparse refuse | **met** | `rewrite_project_id_in_env__refuse_reparse` unit PASS |
| AC15 manual print-only | **met** | `cargo run -p ai-brains-cli --quiet -- project adopt-path --format human` names `3581317d-601e-44f7-ab84-fde90aa12d3c`; `.env` SHA256 `0D33E710…7743F` unchanged; mtime 2026-08-12 00:34:11 UTC unchanged |
| AC16 `--no-project-context` file-id | **met** | `…no_project_context__file_project_id_already_bound` PASS |
| F26 `--format human` in hermetic chrome | **met** | all human ACs pass `--format human` |
| Live `.env` not written | **met** | AC15 hash |
| No clap 5 / new crate | **met** | lock clap 4.6.1; crates.io latest 4.6.6; no new deps |
| T257 / T259 not stolen | **met** | warn SOOT unchanged (`identity_mismatch_warn_line` still whoami); leftover alias not touched |

## Targeted gates

| Command | Result |
|---------|--------|
| `cargo fmt --check` | PASS after `cargo fmt` on `project_adopt.rs` |
| `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` | PASS |
| `cargo nextest run -p ai-brains-cli --test project_adopt_path` | **10 passed** |
| `cargo nextest run -p ai-brains-cli --bin ai-brains rewrite_project_id` | **3 passed** |
| `cargo nextest run -p ai-brains-cli --test project_identity_convergence` | **9 passed** |

## Findings

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| R1-P3-1 | low | `absolute_env_path` silent fallback | **verified_fixed** | Helper removed; `env_path` is `current_dir()?.join(".env")` (already absolute) |
| R1-P3-2 | low | No-owner `--format json` never emits frozen object (`to_project_id: null`); generic `COMMAND_FAILED` | **deferred** | AC6 human contract met; existing `handle_cli_result` pattern; T257 owns JSON interleave. Follow-up if scripted error JSON is needed. |
| R1-P3-3 | low | AC10 event-count is inspection-only | **deferred** | Write path is `fs::write` only; no EventStore. Future helper that calls `context` would be a new track. |
| R1-P3-4 | low | OPERATIONS identity SOOT omitted adopt-path | **verified_fixed** | OPERATIONS Identity SOOT now names `project adopt-path` / `--write-env --yes` |

### R1 (implementer) — no additional findings.

### R1b (independent explore, 2026-08-16)

Verdict: **PASS WITH DEFERRED P3**. P0–P2 none. Four P3s as above. AC9 T82 later confirmed: `test_context_new_project_rotates_id` PASS.

## Notes (not findings)

- AC6 no-owner path eprints a human `register-path` line then returns `Err`, so `handle_cli_result` also emits `COMMAND_FAILED` JSON (existing CLI fallback). Spec only requires stderr to mention `register-path`.
- T257 warn still interleaves on live `cargo run` stdout/stderr capture; stdout JSON parse is AC13; warn placement stays T257.
- Soft residuals unchanged: general `project use`, atomic tmp+rename, `export ` prefix, PATH-behind (F24).

## Completeness sweep (R1)

No TODO/FIXME/stub in `project_adopt.rs`. Command is wired: `ProjectCommands::AdoptPath` → `project_adopt::run`. Whoami remediations updated. Docs match.

## Codex CX1

Product **PASS**. Process P1-1 (full CI + closeout + this file missing at review time) addressed by writing `review.codex.md`, running `.\scripts\dev-check.ps1` + `ledgerful verify --scope full`, then conductor/deferred closeout.

## Full gate

| Command | Result |
|---------|--------|
| `.\scripts\dev-check.ps1` | **SUCCESS** — 2997 passed / 1 skipped; deny 0.20.2; audit 0.22.2 |
| `ledgerful verify --scope full` | **Verification passed** (fmt, workspace clippy, workspace nextest, deny, audit) |

## Closeout

T258 **Completed**. Deferred: F14 remainder `project use`; P3-2 no-owner JSON; P3-3 event-count inspect-only; F24 PATH-behind; live operator rebind out of band. No PR (owner did not ask).
