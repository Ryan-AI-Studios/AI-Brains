# T314 Review Log — clap `--format` / `--dry-run` unify

**Track:** T314-ClapFlagUnify  
**Branch:** `track/T314-clap-flag-unify`  
**FEATURE TX:** `26f296f5-fd76-4d04-afba-6d26e54a1bc5`  
**Category:** FEATURE / UX / CLI honesty  
**Final verdict:** **PASS WITH DEFERRED P3** (engineering DoD clear)

---

## Rounds

| Round | Reviewer | Result |
|-------|----------|--------|
| R1 | Implementer (internal DoD sweep) | PASS — AC1–AC16 wired |
| R2 | Explore subagent (read-only DoD) | PASS — residual lows only |
| CX1 | Codex (`review.codex.md`) | **FAIL** — P1-01 Denied human empty; P2-01 AC7 kind loose; P1-02 process |
| R3 | Implementer fix | P1-01 human Denied fill `Access denied.`; hermetic AC16 Denied; P2-01 exact `UnknownArgument` |
| CX2 | Codex fresh recheck | **PASS WITH DEFERRED P3** (P3-01 test-name drift) |

---

## DoD / AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 bare `--dry-run` progressive | **Met** | `query_progressive__dry_run_bare__parses_true` PASS |
| AC2 false/true/omitted | **Met** | `query_progressive__dry_run_false__parses_false` PASS |
| AC3 briefing bare | **Met** | `briefing_project__…` + `briefing_personal__…` PASS |
| AC4 expand format parse | **Met** | `query_expand__format_json__parses` (json+human+default) PASS |
| AC5 InvalidValue JSON/xml | **Met** | `query_expand__format_JSON__…` + `…xml…` PASS |
| AC6 scan-roots `--dry-run` | **Met** | `scan_roots__dry_run__parses` PASS |
| AC7 progressive no `--format` | **Met** | `query_progressive__format_json__unexpected_argument` exact `UnknownArgument` PASS |
| AC8 T291 freeze | **Met** | `trace_missing_next_step__frozen__exact_string` PASS |
| AC9 expand JSON unknown | **Met** | `query_expand__unknown__preview_nonempty_exit_0` PASS |
| AC10 human Unknown two lines | **Met** | `query_expand__format_human__unknown__two_lines_not_json` PASS |
| AC11 bare dry-run deny | **Met** | `progressive__dry_run_bare_no_grants__exit_3_denied_true` PASS |
| AC12 scan-roots keys + no .env | **Met** | `scan_roots__dry_run_format_json__keys_unchanged_no_env_write` PASS |
| AC13 docs | **Met** | CAPABILITIES split; PROTOCOL-COMPAT expand row; CHANGELOG T314; after_help F25 |
| AC14 manual | **Met** | `cargo run` expand `--format json --project-id` → Unknown JSON; progressive `--dry-run` parses; scan-roots `--dry-run --format json` exit 0 |
| AC15 forbidden empty | **Met** | `git diff` empty on project.rs / project_paths.rs / governed_common.rs / briefing.rs / contracts |
| AC16 human SOOT | **Met** | Unknown + Denied two nonempty lines; Denied human fill `Access denied.` (JSON preview stays empty) |

---

## Findings

| id | severity | status | notes |
|----|----------|--------|-------|
| CX1-P1-01 | high | **verified_fixed** | Denied human empty second line → fill `Access denied.` on human only |
| CX1-P2-01 | medium | **verified_fixed** | AC7 asserts exact `ErrorKind::UnknownArgument` |
| CX1-P1-02 | process | **out_of_scope** | Publish incompleteness — Phase 6 owns |
| CX2-P3-01 | low-info | **deferred** | AC4 unit name drift → `deferred.md` |
| T314-L* | low-info | **deferred** | F18 PATH / F34 / F8 / F10 / not-stolen tracks |

---

## Gates

| Gate | Result |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo nextest run --workspace` (3588, 1 skipped) | PASS via `.\scripts\dev-check.ps1` |
| `cargo deny check` | PASS |
| `cargo audit` | PASS (allowed warnings only) |
| `ledgerful verify --scope full` | PASS |

---

## Residuals → `conductor/deferred.md`

See **T314 implement residuals (2026-08-28)** — PATH, F34, F8 auto, F10 fail_cp, AC4 name drift, not-stolen tracks.
