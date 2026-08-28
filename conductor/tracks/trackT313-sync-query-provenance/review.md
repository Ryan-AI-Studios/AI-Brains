# T313 Review Log — `sync query` rescued heading

**Track:** T313-SyncQueryProvenance  
**Category:** FEATURE / UX honesty  
**Branch:** `track/T313-sync-query-provenance`  
**FEATURE TX:** `a58ee509-ed84-420b-9fd0-c4112782289d`  
**Date:** 2026-08-28

## Scope

When T271 F6 token rescue produces ledger hits, the ledger pane heading becomes
`--- Ledgerful Ledger Search (rescued token: '<tok>') ---`. Phrase hits / misses
keep the generic heading. F7 banner sentence unchanged. T231 ndjson stays vault-only.

## DoD matrix

| Item | Status | Evidence |
|------|--------|----------|
| AC1 rescued heading unit | Met | `ledger_section_heading__rescued_token__names_token` PASS |
| AC2 generic heading unit | Met | `ledger_section_heading__phrase_hit__generic` PASS |
| AC3 empty/whitespace → generic | Met | `ledger_section_heading__empty_token__generic` PASS |
| AC4 F7 banner exact | Met | `ledger_rescue_banner__phrase_empty_token_hit__locked_sentence` PASS |
| AC5 T273 argv | Met | `ledger_search_argv__*` PASS |
| AC6 `--no-bridge` | Met | `sync_query__no_bridge__skips_ledgerful_section` PASS |
| AC7 vault header | Met | `sync_query_pretty_*` isolation PASS |
| AC8 lines helper order | Met | `format_ledger_section_lines__rescued__heading_then_banner` PASS |
| AC9 `rescued_token` field | Met | F6 arm `Some(token.clone())`; other arms `None` |
| AC10 docs | Met | CAPABILITIES / OPERATIONS / WORKFLOWS / CHANGELOG |
| AC11 manual rescue | Met | `cargo run … "graph backend"` → rescued heading + F7 + matching entries for 'graph' |
| AC12 manual phrase | Met | `cargo run … "T314"` → generic heading; no F7 banner |
| AC13 crates allow-list | Met | `git diff --name-only -- crates/` → sync.rs / sync_query_ledger.rs / smoke.rs only |
| AC14 ndjson no heading | Met | `sync_query__format_ndjson__no_ledger_heading` PASS (no `--no-bridge`) |
| sync.rs shrink | Met | main 587 lines → current 578 |

## Findings

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| CX-P0-1 | process | Full gate incomplete at Codex time | `verified_fixed` | `ledgerful verify --scope full` PASS |
| CX-P0-2 | process | Conductor / commit / PR incomplete at Codex time | `verified_fixed` | Conductor Completed; FEATURE commit + Phase 6 |

No product critical/high/medium. Soft residuals → `deferred.md` T313 implement residuals.

## Reviewers

| Round | Reviewer | Verdict |
|-------|----------|---------|
| R1 | Internal subagent | **PASS** (no product findings) |
| CX1 | Codex (`review.codex.md`) | Product **PASS**; process findings closed after full gate |

## Gates

- `cargo fmt --check` PASS
- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` PASS
- Targeted nextest (heading/banner/argv/json/smoke AC6+AC14) **21 PASS**
- Isolation AC7 **2 PASS**
- Manual AC11/AC12 PASS
- Workspace nextest **3593 passed** (1 skipped; unrelated flakes recovered on retry)
- `ledgerful verify --scope full` PASS (fmt/clippy/nextest/deny/audit)

## Completion decision

Engineering DoD met. Publish via implement-track Phase 6.
