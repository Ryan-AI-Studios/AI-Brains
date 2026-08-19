# T271 review log — sync query ledger pane

**Track:** `conductor/tracks/trackT271-sync-query-ledger-pane`
**Category:** BUGFIX / UX (FEATURE TX)
**FEATURE TX:** `67ed4a3e-d354-4d7c-abf2-36792d46d0b8`
**Date:** 2026-08-19

## Scope

Stop FTS-quoting `ledgerful ledger search` (Ledgerful already phrase-wraps).
First-seen contentful token rescue (cap 3) with F7 banner. Named misses:
never-ran / failed / ran-empty (user query, never `'"tok" "tok"'`).
`--no-bridge` unchanged. Vault MATCH keeps T90. No Ledgerful source edits,
no retrieval `preflight.rs` (T272), no `project.rs`, no clap 5, no lock bumps,
no live `.env` / `cargo install`.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC19 / F0–F23 / DoD | PASS |
| R1b | Independent explore | PASS WITH DEFERRED P3 → easy P3s fixed |
| CX1 | Codex FEATURE `gpt-5.6-luna` high | FAIL — P1-1 FP (§5.2); P1-2 process; P2-1/P2-2 fixed |
| CX2 | Codex FEATURE `gpt-5.6-luna` high | **PASS WITH DEFERRED P3** |

## Findings

### R1b (explore)

| ID | Sev | Disposition |
|----|-----|-------------|
| P3-1 CAPABILITIES Ledger-first “vault-only” | P3 | **fixed** — miss/fail no reorder; named miss line |
| P3-2 OPERATIONS `--quiet` = pane-off | P3 | **fixed** — `--no-bridge` skips pane; quiet omits never-ran/failed |
| P3-3 AC5 picker `cfg(test)` vs probe loop | P3 | **deferred** — sequential IO cannot collect-then-pick without extra procs |
| P3-4 F8 quiet untested | P3 | **fixed** — `ledger_quiet_omits_pane` + unit |
| P3-5 F2 copy not asserted | P3 | **fixed** — `SYSTEM32_NEVER_RAN` locked in AC7 unit |
| P3-6 F2 not `cfg(windows)` | P3 | **deferred** — path suffix false on Unix; keeps the helper live for clippy |
| P3-7 invented empty-line never-ran reason | P3 | **fixed** — empty first line → `Ledger search failed.` |
| P3-8 human re-run JSON fallback | P3 | **deferred** — T211 carry-over; F17 still re-runs human on hits |
| P3-9 `len >= 2` untested | P3 | **fixed** — `ledger_rescue_pick__single_token_hit__does_not_rescue` |

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 forward no FTS quotes; empty → `""` | met | `ledger_forward_query__user_phrase__not_fts_quoted` + `…empty__returns_empty` PASS |
| AC2 ANSI strip | met | `ledger_forward_query__ansi_stripped` PASS |
| AC3 forwarded ≠ `"capture" "independence"` | met | same AC1 assert |
| AC4 first-seen tokens | met | `ledger_rescue_tokens__capture_independence__first_seen_capture` PASS |
| AC5 pick second token | met | `ledger_rescue_pick__first_token_empty_second_hits__selects_second` PASS |
| AC6 ran-empty uses user query | met | `ledger_miss_copy__ran_empty__uses_user_query_not_quotes` PASS |
| AC7 System32 / SysWOW64 | met | `is_windows_system_cwd__system32_and_syswow64__true` PASS |
| AC8 never-ran `did not run` | met | `ledger_miss_copy__never_ran__did_not_run` PASS |
| AC9 F7 banner exact | met | `ledger_rescue_banner__phrase_empty_token_hit__locked_sentence` PASS |
| AC10 json_non_empty moved | met | four `ledger_json_non_empty__*` PASS in sibling |
| AC11 `--no-bridge` | met | `sync_query__no_bridge__skips_ledgerful_section` PASS |
| AC12 T211 + T231 hermetics | met | ranking 4 + UX 6 PASS |
| AC13 live dogfood | met | `cargo run -p ai-brains-cli -- sync query "capture independence" --quiet`: F7 banner + 9 capture rows; no T90 quotes |
| AC14 `--no-bridge` live | met | same query `--no-bridge --quiet`: Recall only, no `Ledgerful Ledger Search` |
| AC15 ledger control | met | `ledgerful ledger search capture` ≥1 (9) |
| AC16 docs | met | CAPABILITIES ledger pane row + root CHANGELOG T271 |
| AC17 empty query never-ran | met | `ledger_miss_copy__empty_query__did_not_run` PASS |
| AC18 git stderr → never-ran | met | `ledger_classify_outcome__nonzero_git_stderr__never_ran` PASS |
| AC19 other stderr → failed + 140 cap | met | `ledger_classify_outcome__nonzero_other_stderr__failed` PASS |
| F5 / F6 / F10 / F19 / F22 | met | no sanitize on argv; first-seen cap 3; `pub mod`; local 140 cap; probe sequential |

## Targeted gates (observed)

- `cargo fmt --all` then clippy `-p ai-brains-cli --all-targets -- -D warnings` exit 0
- Bin units 23 PASS (T271 + resolve_sync_project_id)
- Hermetics 11 PASS (no-bridge, ranking, UX)

## Full gate (observed)

- `ai-brains daemon stop` first (restore hermetics).
- `.\scripts\dev-check.ps1` **[SUCCESS]** nextest **3167** passed (1 skipped)
- `ledgerful verify --scope full` **passed** (fmt 2.4s / clippy 13.3s / nextest 186.3s / deny 3.0s / audit 2.7s)

## Residual / decline

- PATH until operator `cargo install` (F16)
- Ledgerful token-OR / stop phrase-wrapping (F23 / other repo)
- T211 F25 blend / double shell (declined F11)
- T268–T270 / T272 / T240 F2 / T255 bag (declined F12)
- Rescue scoring / merge all token tables (soft §11)
