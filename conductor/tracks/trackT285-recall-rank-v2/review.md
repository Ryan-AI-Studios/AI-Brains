# T285 review log — recall rank v2

**Track:** T285-RecallRankV2
**Branch:** `track/T285-recall-rank-v2`
**FEATURE TX:** `ac3da53e-1eea-4539-aaa7-054808fb35a3`
**Reviewers:** implementer (R1) → codex-review (FEATURE)

## Scope

Envelope strip (role + `TAGS:`) in `first_contentful_line` / `classify_pin_kind`; live detector prefixes; pass-1 TAGS-or-authority + in-memory retain + recency retry; `LEADING_QUERY_BONUS` inside `rerank_hits_with_query`; chrome-seed skip from blended content; CLI hermetic AC12/AC13 + `test(graph)` AC17.

**Did not:** `project.rs` / `sync.rs` / CLI `preflight.rs` / `pin.rs` write / `ci.yml` / clap 5 / rusqlite 0.40 / T286 Index / T287 list ORDER / T293 neighbors CLI / leftover F39 / `cargo install`.

## DoD matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | **Met** | `classify_pin_kind__tags_envelope_and_role_trim__ac1` PASS |
| AC2 | **Met** | rstest `is_session_chrome__live_and_closed_prefixes__ac2` 11/11 PASS (new prefixes + T274 closed + `# Heading` false) |
| AC3 | **Met** | `rerank_hits_with_query__onboarding_chrome_loses_to_pin__ac3` PASS |
| AC4 | **Met** | `recall_full__tagged_pin_vs_body_match_review_dumps__hit_one__ac4` PASS (red: dump #1) |
| AC5 | **Met** | untagged pin vs same dumps PASS |
| AC6 | **Met** | `parent_seeds_graph_neighbors__chrome_false_authority_true__ac6` PASS |
| AC7 | **Met** | T274 `recall_pin_rank` CLI+retrieval PASS |
| AC8 | **Met** | graph-on suite 72/72 (T260 stub skip stays in pipeline before T285 skip) |
| AC9 | **Met** | contentless path untouched (T261) |
| AC10 | **Met** | `lexical_search__default_unfiltered__finds_session_chrome` PASS |
| AC11 | **Met** | CLI JSON: raw `TAGS:`+`DECISION:`; no `is_session`/`pin_kind`/`envelope` |
| AC12 | **Met** | CLI `recall` + `search` pretty hit #1 pin, exit 0 (~39s hermetic) |
| AC13 | **Met** | CLI `sync query` vault top pin |
| AC14 | **Met** | `recall_full__semantic_no_blobs__lexical_fallback_pin_in_top3__ac14` PASS |
| AC15 | **Met** | pass-1 SQL has GLOB+TAGS:+LIMIT; recency SQL `updated_at` + `?` only |
| AC16 | **Met** | T216 list ORDER not touched |
| AC17 | **Met** | `recall__graph_on__chrome_parent_does_not_seed_nonmatch_neighbor__ac17` PASS in `-E 'test(graph)'` (72/72) |

## Findings

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| R1-1 | low-info | Live Manual canary `cargo run pin` without forcing cwd `PROJECT_ID` landed in `[test-alias]`; project-scoped recall of the GUID was empty. `--global` recall/search/sync-query of the GUID is hit **#1** tagged DECISION (score −73.364). Hermetic AC12/AC13 remain SoT. | deferred | not a ranking bug; pin env |

No critical/high/medium. R1 product **PASS**.

## Codex CX1 (gpt-5.6-luna)

**Product PASS** — 0 P0 / 0 P2 / 0 P3 product. **P1-1** process (track still In Progress / checkboxes / full gate not yet recorded at review time; `ledgerful doctor` unable to open db in sandbox). **verified_fixed** after `dev-check` **3360** passed / 1 skipped, `ledgerful verify --scope full` exit 0, conductor Completed, Phase 6 publish.

## Full gate

- `.\scripts\dev-check.ps1` **SUCCESS** — nextest **3360** passed (8 slow), 1 skipped; deny; audit 19 allowed warnings
- `ledgerful verify --scope full` exit 0 (fmt/clippy/nextest/deny/audit)

## Manual

```
NEEDLE=T285 rank-v2 unique canary d2067fc4-fb4f-489d-9e0d-f74ebd87e023
pin → Memory 0be0fcbe-8f4a-4c60-8ca8-0ae119e23744
recall --global GUID → [test-alias] TAGS: t285-canary / DECISION: … GUID  (hit #1)
search --global GUID → same
sync query --global GUID → vault top same pin
```

Did not start with `## Objective` / `# Review of Track` / `# AI-Brains Session Onboarding`. Exit 0. Did not `cargo install`. `.env` not rewritten.

## Targeted gates (pre-full)

- `cargo fmt --check` exit 0
- `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings` exit 0
- retrieval units AC1–AC3/AC6/AC15 + hermetic AC4/AC5/AC14 PASS
- CLI AC12/AC13 + T274 `recall_pin_rank` PASS
- `cargo nextest run -p ai-brains-cli --features graph -E "test(graph)"` **72 passed**

## Pins (re-verified at execute)

clap lock 4.6.1; rusqlite 0.39.0; chrono 0.4.44; serde_json 1.0.150; uuid 1.23.1; tokio 1.52.3; rustc 1.95.0; nextest 0.9.140; workspace 0.1.2. **No bumps.** rstest 0.25 already in workspace (retrieval **dev-dep** only).
