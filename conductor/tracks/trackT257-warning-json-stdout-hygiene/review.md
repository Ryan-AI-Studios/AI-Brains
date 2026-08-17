# T257 review log — Warning + JSON stdout hygiene

**Track:** `conductor/tracks/trackT257-warning-json-stdout-hygiene`
**FEATURE TX:** `d086c5f3-6918-49e6-a1fd-377a743ee7fc`
**Date:** 2026-08-17
**Implementer:** Grok

---

## Scope

Identity SOOT stays human stderr. JSON-effective commands suppress the line so `2>&1` is one object. `scope resolve` JSON injects `project_identity_mismatch env=<uuid> path=<uuid>` into existing `warnings[]`. Remediator skip: `project whoami` / `project adopt-path`. T240 `project list` hermetic still warns.

## AC / DoD matrix (internal)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 skip whoami/adopt-path | Met | `should_skip_identity_mismatch_warn__whoami_and_adopt_path` + project.rs flags test |
| AC2 token | Met | `identity_mismatch_json_token__stable_no_warning_prefix` |
| AC3 scope JSON token + no SOOT | Met | `scope_resolve_json__mismatch__stdout_parses_token_no_soot` |
| AC4 T240 list stderr SOOT | Met | `project_list__identity_mismatch__warn_on_stderr` |
| AC5 whoami JSON no SOOT | Met | `whoami_json__mismatch__no_stderr_soot` |
| AC6 whoami human no SOOT + adopt-path | Met | `whoami_human__mismatch__no_stderr_soot` |
| AC7 nightly JSON no SOOT / no warnings key | Met | `nightly_status_json__mismatch__no_soot_no_warnings_key` |
| AC8 dry-run preview | Met | `nightly_schedule_dry_run__stdout_preview_has_no_soot` |
| AC9 concat parse | Met | `scope_resolve_json__mismatch__concat_streams_parse` |
| AC10 T240 hermetics | Met | `project_identity_convergence` 9/9 |
| AC11 docs | Met | CAPABILITIES + PROTOCOL-COMPAT + CHANGELOG |
| AC12 no DTO / no pins / project.rs shrink | Met | 1514 lines (was ~1549); no contracts/clap/crate edits |
| AC13 npc + global skip | Met | existing T240 npc + `recall_global__mismatch__no_soot` |
| AC14 scope human stderr SOOT | Met | `scope_resolve_human__mismatch__stderr_soot_stdout_clean` |
| AC15 inject idempotent | Met | `inject_identity_mismatch_token__already_present__no_duplicate` |
| AC16 live classify | Pending finalize | source bin; no `.env` write |
| AC17 both scope emit sites | Met | `scope.rs` local `:95` + daemon `:132` |
| F0–F26 | Met (F0 lifted by `/implement-track`) | See spec; no T258/T259 steal |
| T240 AC4 | Met | listed above |
| No live `.env` / no `cargo install` | Met | classify-only |

## Findings

None open.

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| — | — | — | — |

## Completeness

- No TODO/FIXME/placeholder in `identity_warn.rs`.
- `print_json_stdout` in `identity_warn.rs` (not `format_resolve.rs`).
- Nightly status keys unchanged.
- Doctor stays early-route.

## Residual (defer)

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` still noisy until reinstall | F13 — operator `cargo install` |
| Compact JSON sites use `note_machine_stdout` not pretty rewrite | Intentional (T265/T266) |
| T223 env-override can still trail JSON | F17 decline |
| Human warn now **after** the table | F6; T240 asserts presence not order |
| `scope` human still says `next: whoami` (T249) | Keep |
