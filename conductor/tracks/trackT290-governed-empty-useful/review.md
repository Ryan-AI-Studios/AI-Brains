# T290 review log — granted-empty lists/progressive copy-paste recall + Pinned: N

**Track:** T290-GovernedEmptyUseful
**Branch:** `track/T290-governed-empty-useful`
**FEATURE TX:** `64da7141-6bb0-4e9c-8895-3aaba3bdb5d2`
**Reviewers:** implementer (R1) → codex-review (FEATURE)

## Scope

Granted-empty `evidence` / `source` / `review` list and `query progressive` JSON `next_step` is copy-paste `ai-brains recall "…"` (lists: `what did we decide`; progressive: sanitized operator query) plus `(Pinned: N)` when `count_pinned_memories` succeeds. Human lists print that line after the frozen `(none)` strings. `items[]` / `results[]` stay empty (no H2). List DTOs unaugmented (CLI `Value` overlay). Progressive `next_step` string growth only. T243 `PROGRESSIVE_RECALL_FALLBACK` ellipsis frozen for deny stderr.

**Did not:** H2 pin→Approved; T288 `vault_pin_*` on lists; T289 Personal; T291 `query trace`; T292 `policy check` human; T293 neighbors; T294 leftover; DTO new fields; clap 5 / rusqlite 0.40; `cargo install`; `.env` write; extra live `policy bootstrap`; grow `briefing.rs` / `personal.rs` / `project.rs` / CLI `preflight.rs` / `query_store.rs`. `QueryStore` imported in the four callers, not `governed_common.rs`.

## DoD matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | **Met** | `format_authorized_empty_next__with_count__includes_pinned_and_copy_paste` PASS (red: `T290_RED_STUB`; green: exact F7 strings; `!contains('\n')`; `!contains('…')`) |
| AC2 | **Met** | `list__authorized_empty__next_step_names_pinned_and_query` rstest evidence/source/review PASS (`items: []`, `recall` + `what did we decide` + `(Pinned: 0)`) |
| AC3 | **Met** | `list__authorized_empty_human__none_then_next_line` rstest all three nouns PASS (`evidence: (none)` / `sources: (none)` / `review items: (none)` + next line) |
| AC4 | **Met** | `sanitize_recall_query__cases__expected_needle` + 80-cap + newline formatter PASS |
| AC5 | **Met** | `evidence_list__authorized_empty_with_pin__next_step_nonzero_items_empty` PASS (`(Pinned:` not `0`; items `[]`; pin text not in items) |
| AC6 | **Met** | `query_progressive__authorized_empty__next_step_contains_query_and_pinned` uses `progressive_cmd_query` (not `"x"`) PASS (`SQLCipher` + `Pinned:` + no U+2026) |
| AC7 | **Met** | T263 `*_list__authorized_empty__next_step_names_recall` stay green |
| AC8 | **Met** | T263 denied list AC8 stay green (exit 3, no authorized-empty `next_step`) |
| AC9 | **Met** | overlay-gate rstest + `denied: true` omit; deny stderr still `PROGRESSIVE_RECALL_FALLBACK` |
| AC10 | **Met** | CAPABILITIES Empty + progressive sentences; list/progressive after_help; CLI-EXIT-CODES + OPERATIONS; PROTOCOL-COMPAT overlay string growth; CHANGELOG T290; help units PASS |
| AC11 | **Met** | `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` PASS; no new crate; clap lock 4.6.1; rusqlite 0.39.0 |
| AC12 | **Met** | Manual `cargo run` four commands: JSON `items`/`results` `[]`, `next_step` `recall` + `(Pinned: 3975)` (lists) / `SQLCipher` (progressive); human evidence `(none)` then next line. Exit 0. Live N not required equal to preflight. |
| AC13 | **Met** | `list_and_progressive_dtos__serde__no_vault_pin_count` PASS |
| AC14 | **Met** | sanitize rstest + overlay-gate rstest |
| AC15 | **Met** | `progressive_recall_fallback__exact__ellipsis_unchanged` PASS |
| AC16 | **Met** | `apply_progressive_search_hints__authorized_nonempty__omits_next_step` PASS |
| AC17 | **Met** | `progressive_query_response__golden_fixture_parses` PASS |

## Findings

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| R1-1 | low-info | PATH `ai-brains` still T281-era until `cargo install`. Source/hermetic SoT. | deferred | F17 |
| R1-2 | low-info | Daemon list overlay has copy-paste query but `pin_count = None` (no `(Pinned: N)`). | deferred | F14 |

No critical / high / medium. Internal R1 **PASS**.

## Targeted gates (pre-full)

- `cargo fmt` PASS
- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` PASS
- Red was assertion-fail (not compile-error-only): AC1 stub `T290_RED_STUB`; AC4 raw passthrough
- CLI units (formatter/sanitize/overlay/help) **21 PASS**
- Hermetic `governed_vault_pin_honesty` + `governed_first_run_deny_exit` **32 PASS**
- Contracts golden AC17 PASS

## Manual

```
cargo run -p ai-brains-cli --quiet -- evidence list --format json --local
cargo run -p ai-brains-cli --quiet -- evidence list --format human --local
cargo run -p ai-brains-cli --quiet -- source list --format json --local
cargo run -p ai-brains-cli --quiet -- review list --format json --local
cargo run -p ai-brains-cli --quiet -- query progressive "what did we decide about SQLCipher"
```

JSON lists: `items: []`, `next_step` = `Ungoverned vault search: ai-brains recall "what did we decide" (Pinned: 3975)`. Human evidence: `evidence: (none)` then that next line. Progressive: `denied: false`, `results: []`, `next_step` contains `SQLCipher` + `(Pinned: 3975)`, no U+2026. Exit 0. PATH not reinstalled (F17). Did not write `.env`. Did not extra `policy bootstrap`.

## Codex CX1 (gpt-5.6-luna, read-only)

Product **PASS** after P1 fix. No remaining product P0–P2.

| id | severity | disposition |
|----|----------|-------------|
| P0 | process | **verified_fixed** after closeout + Phase 6 (same class as T289 P1-1 — local Completed is not published) |
| P1 | product | **verified_fixed** — sanitize drops `$` and backtick; rstest `echo $(hi)` / `` say `whoami` `` + formatter unit `format_authorized_empty_next__powershell_interpolators__stripped` PASS |

## Full gate

- `.\scripts\dev-check.ps1` **SUCCESS** nextest **3429** passed / 1 skipped (10 slow)
- `ledgerful verify --scope full` exit 0 (`fmt` / workspace clippy / nextest / deny / audit)

Did **not** `cargo install`. Did **not** write `.env`. Did **not** extra `policy bootstrap`. Daemon left **Stopped**.
