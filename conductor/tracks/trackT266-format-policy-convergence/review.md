# T266 review log — Format policy convergence

**Track:** `conductor/tracks/trackT266-format-policy-convergence`
**Category:** FEATURE / UX
**FEATURE TX:** `9aec5831-b7a6-4eb6-85b8-49168cf7b07a`
**Date:** 2026-08-18

## Scope

Make `--format` predictable on operator inventory without flipping frozen
script contracts. Five inventory commands join the T248/T249 token set and
call `format_resolve::is_json_output`. Default stays `auto` (TTY table / pipe
JSON). `--format pretty` is a table. Nightly pipes stay human. `graph update`
stays JSON. No `OutputFormat::parse` change, no T180 growth, no clap 5, no
live `.env` / `cargo install`.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC14 / F0–F27 / DoD | PASS |
| R1b | Independent explore | **PASS** (0 findings) |
| CX1 | Codex FEATURE `gpt-5.6-luna` high | Product **PASS**. P1 closeout-pending at review time (`T266-CLOSE-01`) — process, not product |

## Findings

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| CX1-P1 | medium (process) | Phase 4 / publish still pending when Codex ran | `verified_fixed` | Full gate + R1b + this closeout; product ACs already met |

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 resolver units unchanged | met | `resolve_human_json_format__*` 5/5 PASS; helper match unchanged |
| AC2 list-paths + scan-roots clap | met | `pretty` parses; `xml`/`JSON`/`Pretty` InvalidValue (8 clap units) |
| AC3 unfiltered empty pretty | met | `list_paths__format_pretty__human_empty_copy` (no `--project`/`--shared-only`) |
| AC4 named json + human halves | met | `list_paths__format_json__api_version_1` + `list_paths__format_human__table_not_json` |
| AC5 scan-roots pretty + json | met | `scan_roots__format_pretty__not_json` + `scan_roots__format_json__api_version_1` |
| AC6 forks deleted | met | grep `use_json_output` empty; callers use `is_json_output` |
| AC7 whoami/adopt/rebind clap | met | `pretty` parses; `JSON`/`Pretty` InvalidValue (9 clap units) |
| AC8 T254/T259/T258/T240 stay green | met | 67-test keep-green (list-paths/scan-roots/adopt/rebind/whoami) |
| AC9 graph update default JSON | met | no `graph.rs` edit; `graph_update__feature_off` PASS; T246 hermetic untouched |
| AC10 nightly pipes stay human | met | `nightly_status__default_format__human_header_even_if_piped` PASS |
| AC11 docs | met | CAPABILITIES Families A–D + required rows; PROTOCOL-COMPAT list-paths/scan-roots; CHANGELOG T266; after_help + five arg docs name `pretty` |
| AC12 no DTO / no pin bump / parse untouched | met | no contracts crate; lock clap 4.6.1 / serde_json 1.0.150; `OutputFormat::parse` still at `governed_common.rs:307`; retrieval untouched |
| AC13 manual source bin | met | `cargo run -p ai-brains-cli -- --no-project-context`: list-paths `--format human` table; default list-paths JSON `api_version`; retention plan `--format human` T248 matrix; nightly `--status --quick` `=== Nightly Status ===` |
| AC14 non-empty pretty ≡ human | met | `list_paths__format_pretty__table_not_json` |
| F27 `is_json_output` | met | wrapper + `is_json_output__pretty_pipe__false` |
| F8/F9/F10/F11 declines | met | graph update / apply / recall / parse not edited |

## Targeted gates (observed)

- `cargo fmt --all` then `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` exit 0
- clap + resolver units 23/23 PASS
- hermetics + keep-green 67/67 PASS
- `graph_update__feature_off__exit_2_feature_unavailable` PASS

## Full gate (observed)

- `.\scripts\dev-check.ps1` **[SUCCESS] CI Gate passed!** nextest **3126** passed (1 skipped)
- `ledgerful verify --scope full` **passed** (fmt 2.4s / clippy 12.3s / nextest 165.8s / deny 2.4s / audit 3.0s)

## Residual / decline

- T246 F17 TTY-auto `graph update` stays soft (F8)
- T227 F34 `parse_or_fail` stays residual (F11)
- Harness status has no `value_parser` (Family B; F25)
- TTY/`auto` hermetics still force `human`/`json` (T254 F12)
- PATH until operator `cargo install` (F21)
- T267 leftover-as-AI-Brains footer / T268 scan suggestion / T270 0 candidates
