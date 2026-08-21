# T274 review log — Pins vs harness ingest ranking

**Track:** T274-PinRankVsIngest  
**Status:** In Progress  
**FEATURE TX:** `a5e94797-f17d-45bc-b591-a2399fa42da5`  
**HEAD (implement):** `track/T274-pin-rank-vs-ingest`

## Reviewers / rounds

| Round | Reviewer | Result |
|-------|----------|--------|
| R1 | Implementer (Grok) vs spec AC1–AC17 / DoD | Findings below; I1 test fix re-run green |
| R1b | Internal explore subagent vs spec | **PASS** — no P0/P1/P2. P3 residuals below. |
| CX1 | Codex (FEATURE) `review.codex.md` | Mid-implement **Not complete**. P1 process (gates) now met. P2-1 F8 documented. **P2-2 Recent recency `verified_fixed`**. P2-3 test honesty deferred low. |

## Finding fields

id, severity, description, source, files, required_fix, status, evidence.

## Findings

### I1 — CLI pin stores `ASSISTANT: DECISION:` so JSON `starts_with("DECISION:")` was false

- **severity:** medium (test honesty, not product)
- **source:** implementer / AC14 hermetic CLI
- **files:** `crates/ai-brains-cli/tests/recall_pin_rank.rs`
- **required_fix:** Assert leading DECISION: after one `ASSISTANT: ` strip (JSON stays raw; pretty already strips).
- **status:** `verified_fixed`
- **evidence:** `cargo nextest run -p ai-brains-cli --test recall_pin_rank` — 2 passed (2026-08-21). JSON hit #1 is `ASSISTANT: DECISION: …`; chrome is not #1.

### R1b-P3-1 — detector tests are sequential `#[test]`, not `rstest #[case]` (F27 style)

- **severity:** low-info
- **source:** internal R1b
- **files:** `crates/ai-brains-retrieval/src/session_chrome.rs`
- **required_fix:** none for DoD — closed-list cases are covered
- **status:** `deferred`
- **evidence:** AC2 true/false tests exist; F20 avoided adding rstest to retrieval

### R1b-P3-2 — AC6 dump uses buried `decision:` not buried `CONSTRAINT:`

- **severity:** low-info
- **source:** internal R1b
- **files:** `crates/ai-brains-retrieval/tests/preflight_index_pin_rank.rs`
- **required_fix:** none — buried CONSTRAINT: would match Safety LIKE and steal the dump from Index (T279). Buried `decision:` still proves F2 Other + Index two-pass.
- **status:** `deferred`

### R1b-P3-3 — SQL GLOB misses lowercase `decision:` (F8 documented)

- **severity:** low-info
- **source:** internal R1b / spec §11
- **files:** `session_chrome.rs` `authority_glob_sql`
- **required_fix:** none — detector + in-memory prefer-fill/penalty are SoT
- **status:** `deferred`

### CX1-P1-1 — DoD/gates incomplete at Codex snapshot

- **severity:** high (process)
- **source:** Codex CX1
- **status:** `verified_fixed`
- **evidence:** `cargo fmt --check` + workspace clippy `-D warnings` + `cargo nextest run --workspace` **3247 passed / 1 skipped** (daemon Stopped) + `cargo deny check` + `cargo audit` (19 allowed warnings). Restore drill false-fail with daemon Running is unrelated (T188).

### CX1-P2-1 — lowercase GLOB miss vs case-insensitive classify

- **severity:** medium (claimed)
- **source:** Codex CX1
- **disposition:** **False positive / spec F8**. GLOB is a case-sensitive subset; detector + in-memory prefer/penalty are SoT. Same residual as R1b-P3-3 / spec §11.
- **status:** `out_of_scope`

### CX1-P2-2 — Most Recent Memories used authority-first Index collection

- **severity:** medium
- **source:** Codex CX1
- **files:** `crates/ai-brains-retrieval/src/preflight.rs`
- **required_fix:** Recency-ordered drain for Recent; Index stays two-pass.
- **status:** `verified_fixed`
- **evidence:** separate `recent_raw` recency scan; AC6 still lists DECISION pin in Index.

### CX1-P2-3 — AC16 helper vs semantic caller; AC14 JSON prefix

- **severity:** low-info
- **source:** Codex CX1
- **disposition:** AC16 spec is unit-only no HTTP (`prefer_authority_hits`). AC14 JSON now asserts `ASSISTANT: DECISION:` or `DECISION:`.
- **status:** `deferred`

## AC / DoD matrix (R1)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 buried / INVARIANT | Met | `classify_pin_kind__buried_decision__other` |
| AC2 detector | Met | `is_session_chrome__closed_prefixes__true` / `__authority_and_chat__false` |
| AC3 chrome BM25 −12 | Met | `rerank_hits__leading_decision_outranks_session_chrome` |
| AC4 chrome monopoly | Met | `recall_full__chrome_monopoly__authority_pin_is_hit_one` |
| AC5 chrome first-line collapse | Met | `dedupe_session_chrome__identical_first_line__one_chrome_two_pins` |
| AC6 Index pin present | Met | `preflight__index_prefers_leading_decision_over_objective_dump` |
| AC7 summary DECISION: count | Met | same fixture `matches("DECISION:") >= 1` |
| AC8 T211 plan/shipped | Met | `rerank_hits__plan_below_shipped_same_track__ac1` still green |
| AC9 T260 exclude | Met | `recall_full__default_excludes_symbol_stub__ac3` |
| AC10 contentless | Met | existing T261 units still in `--lib` |
| AC11 memory list ORDER | Met | `memory_list_inventory` 7 passed; store untouched |
| AC12 forget unfiltered | Met | `lexical_search__default_unfiltered__finds_session_chrome` |
| AC13 no new JSON keys | Met | CLI JSON asserts no `is_session` / `pin_kind` |
| AC14 hermetic CLI pretty+JSON | Met | `recall__unique_pin_needle__hit_one` |
| AC15 sync query vault | Met | `sync_query__unique_pin_needle__vault_top_is_pin` |
| AC16 semantic prefer-fill unit | Met | `prefer_authority_hits__authority_first_then_cap` (no HTTP) |
| AC17 bound NOT IN + GLOB LIMIT | Met | `match_sql__pass1_glob_limit__pass2_bound_not_in` |
| Safety SQL unchanged | Met | `safety_sql` string untouched |
| `project.rs` / CLI `preflight.rs` / `sync.rs` | Met | git diff does not include them |
| clap 5 / rusqlite 0.40 / DTO keys | Met | no Cargo.toml pin bumps |

## Targeted gates (R1)

- `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings` — exit 0
- `cargo nextest run -p ai-brains-retrieval --lib` — 120 passed
- `cargo nextest run -p ai-brains-retrieval --test recall_pin_rank --test preflight_index_pin_rank` — 3 passed
- `cargo nextest run -p ai-brains-retrieval --test recall_symbol_demote --test lexical_rescue` — 16 passed
- CLI `recall_pin_rank` — 2 passed after I1
- Full workspace gate: `cargo nextest run --workspace` **3247 passed / 1 skipped** (after P2-2; daemon Stopped). `cargo deny check` + `cargo audit` exit 0 (19 allowed warnings).
