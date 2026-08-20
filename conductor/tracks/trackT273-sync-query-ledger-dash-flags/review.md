# T273 review log — sync query dash-leading Ledgerful flags

**Track:** `conductor/tracks/trackT273-sync-query-ledger-dash-flags`
**Category:** BUGFIX
**BUGFIX TX:** `20892666-f2ed-4710-b05f-371b02200567`
**Date:** 2026-08-20

## Scope

`sync query` always inserts POSIX `--` immediately before the `ledgerful ledger search` QUERY so dash-leading needles (`--limit`, `--days`, `--breaking`, `--json`, `-l`/`-d`/`-b`, `"--"`) are search text, not Ledgerful flags. One helper `ledger_search_argv` covers phrase JSON, token JSON, and human re-run. Operator form: `ai-brains sync query -- --limit` (vault flags **before** `--`). Vault `--limit` is unchanged. No T90 on the ledger argv. No `allow_hyphen_values` on Query. No `sync.rs` / `project.rs` / `recall.rs` / DTO / clap 5 / pin bumps.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC14 / F0–F23 / §13 | PASS |
| R1b | Independent explore (read-only) | **PASS** (no P0–P3 product findings) |
| CX | Codex | **skipped** — F17 BUGFIX optional (no DTO / architecture); R1/R1b PASS; argv units are OS-agnostic strings |

## Findings

### R1 / R1b

No product findings. R1b independently traced Query clap → `probe_ledger_search` → `run_ledger_search` → `ledger_search_argv` and confirmed empty-query never-ran still happens before spawn, no T90, no `allow_hyphen_values` on Query, hotspots not grown.

Execute-time pin correction (not a finding): OpenCode O-1 / F22 guessed `ErrorKind::MissingRequiredArgument` by analogy with T247 `--quick` `requires`. Live clap 4.6.1 reports empty `--limit` (no following token) as `InvalidValue` (clap 4 folded `EmptyValue`). Message still contains `--limit <LIMIT>`; live CLI exit **2**. Spec/plan/unit updated to live kind + message.

Noted out of DoD (not findings): `bridge_search_args` (`ledgerful search` code) still lacks `--` (F7 soft residual); PATH `ai-brains` 0.1.1 until operator `cargo install` (F16).

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 JSON argv `--limit` | met | `ledger_search_argv__json_dash_limit__end_of_options_before_query` |
| AC2 human argv no `--json` | met | `ledger_search_argv__human_dash_limit__no_json_flag` |
| AC3 always-on `--` for plain phrase | met | `ledger_search_argv__plain_phrase__still_emits_double_dash` |
| AC4 needles `--days`/`--breaking`/`--json`/`-l`/`-d`/`-b`/`"--"` | met | seven `ledger_search_argv__json_*` units |
| AC5 T271 forwarder/rescue/miss/classifier | met | T271 unit suite green (18 tests) |
| AC6 `--no-bridge` skips ledger pane | met | `sync_query__no_bridge__skips_ledgerful_section` |
| AC7 layer-1 POSIX `-- --limit` is query | met | `sync_query__posix_end_of_options__limit_is_query` |
| AC8 vault `--limit` stands | met | `sync_query__bare_limit_flag__still_requires_value` (`InvalidValue` + `--limit <LIMIT>`); live EXIT=2 |
| AC9 manual `-- --limit` pane | met | 3 table rows for `'--limit'`; no `--limit <LIMIT>` required |
| AC10 `--no-bridge -- --limit` | met | Recall only; no `Ledgerful Ledger Search`; EXIT=0 |
| AC11 Ledgerful control | met | `ledgerful ledger search --json -- --limit` json_lines=77 |
| AC12 docs + after_help | met | help unit; CAPABILITIES T271 bullet names POSIX `--`; CHANGELOG T273 row; OPERATIONS one-line |
| AC13 T211/T231 hermetics | met | ranking 4 + `sync_query_ux` 7 green |
| AC14 `--quiet -- --limit` prints pane | met | 3 table rows under `--quiet`; EXIT=0 |
| F1/F2 one helper always `--` | met | `run_ledger_search` uses only `ledger_search_argv` |
| F3 no T90 | met | `ledger_forward_query` still strip_ansi.trim |
| F4 rescue/quiet/miss classes | met | T271 units + AC6/AC14 |
| F5/F21 flags before `--` | met | AC7/AC8/AC10; no `allow_hyphen_values` |
| F6 after_help contrast | met | AC12 |
| F7 recall residual not stolen | met | `recall.rs` untouched |
| F11 module | met | helper in `sync_query_ledger.rs`; clap in `main.rs` |
| F12 capture independence | met | argv + docs only |
| F13 no pin bumps | met | clap lock 4.6.1; serde_json 1.0.150 |
| F18 residuals → deferred.md | pending Phase 5 |
| No production unwrap/expect/panic | met | grep empty on production helper; clippy `-D warnings` |
| Hotspots not grown | met | `sync.rs` / `project.rs` not in diff |

## Full gate (observed)

- `.\scripts\dev-check.ps1` **SUCCESS** — nextest **3206** passed, 1 skipped
- `ledgerful verify --scope full` **passed** (fmt 2.2s / clippy 1.8s / nextest 98.1s / deny 2.5s / audit 2.1s)

## Targeted gates (observed)

- Red: `ledger_search_argv` unresolved import (compile fail) before helper existed
- `cargo fmt --check` on touched sources exit 0
- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` exit 0
- `cargo nextest run -p ai-brains-cli --bins -- ledger_search_argv …` 13 passed
- T271 units 18 passed
- smoke `sync_query__no_bridge*` 2 passed
- `sync_query_ranking` 4 passed
- `sync_query_ux` 7 passed

## Manual evidence

```text
AC9  cargo run -p ai-brains-cli -- sync query -- --limit
     --- Ledgerful Ledger Search ---
     3 matching entries for '--limit':
     (T211 / T217 / T273 rows). EXIT=0. No "a value is required for '--limit <LIMIT>'".

AC10 cargo run -p ai-brains-cli -- sync query --no-bridge -- --limit
     --- AI-Brains Recall --- only. No Ledgerful Ledger Search. EXIT=0.

AC11 ledgerful ledger search --json -- --limit
     json_lines=77 (array of hits). EXIT=0.

AC14 cargo run -p ai-brains-cli -- sync query --quiet -- --limit
     Ledger pane printed (3 matching entries). EXIT=0.

AC8  cargo run -p ai-brains-cli -- sync query --limit
     error: a value is required for '--limit <LIMIT>' but none was supplied
     EXIT=2.
```

## Deferred candidates (Phase 5)

- PATH `ai-brains` until `cargo install` (F16)
- `bridge_search_args` dash-query (F7)
- Ledgerful QUERY `allow_hyphen_values` / token-OR (other repo)
- T269 / T270 / T272 peers
