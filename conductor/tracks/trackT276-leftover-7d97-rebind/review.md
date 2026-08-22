# T276 review log — Leftover `7d97a456` must not starve `--global`

**Track:** T276-Leftover7d97Rebind  
**Status:** Completed  
**FEATURE TX:** `6846ad81-4892-41fd-935c-82030dcaf0ac`  
**HEAD (implement):** `track/T276-leftover-7d97-rebind`

## Reviewers / rounds

| Round | Reviewer | Result |
|-------|----------|--------|
| R1 | Implementer (Grok) vs spec AC1–AC16 / DoD | **PASS** — AC2 red then green; AC4/AC5/AC9/AC15 hermetic green; F6/F9/F11/F14 held |
| CX1 | Codex (FEATURE) | **FAIL** — P1-1 partition, P1-2 process gate, P2-1/2/3 |
| CX2 | Codex (FEATURE) re-review `review.codex.md` | **PASS WITH P3-1** — product CX1 findings `verified_fixed`; P3-1 review-log stale notes (this file) |

## Finding fields

id, severity, description, source, files, required_fix, status, evidence.

## Findings

### CX1-P1-1 — Post-rerank preferred partition

- **severity:** high
- **source:** Codex CX1
- **status:** `verified_fixed`
- **files:** `crates/ai-brains-retrieval/src/recall.rs`
- **required_fix:** Remove blanket post-`rerank_hits` sort; F1 merge then existing rerank only
- **evidence:** Partition removed in `772850d`. Leftover-authority starve is proven at merge (`merge_preferred_then_global__leftover_authority__owner_first`); AC2 chrome fixture restored.

### CX1-P1-2 — Full gate / provenance closeout

- **severity:** high (process)
- **source:** Codex CX1
- **status:** `verified_fixed`
- **evidence:** `.\scripts\dev-check.ps1` **SUCCESS** (nextest **3266 passed / 1 skipped** + deny + audit 19 allowed). `ledgerful verify --scope full` exit 0.

### CX1-P2-1 — Preferred-full still ran unscoped MATCH

- **severity:** medium
- **source:** Codex CX1
- **status:** `verified_fixed`
- **files:** `crates/ai-brains-retrieval/src/recall.rs`
- **required_fix:** Lazy unscoped `lexical_search` only when `scoped.len() < depth` (F39)
- **evidence:** `772850d` skips the second MATCH when preferred fills depth.

### CX1-P2-2 — Substring fallback `project_id: None`

- **severity:** medium
- **source:** Codex CX1
- **status:** `verified_fixed`
- **files:** `crates/ai-brains-retrieval/src/lexical.rs`
- **required_fix:** COALESCE project_id on LIKE SELECT (F15 analog; F34 path unchanged)
- **evidence:** `substring_fallback__maps_coalesce_project_id` green

### CX1-P2-3 — AC2/AC3 used leftover authority instead of chrome

- **severity:** medium
- **source:** Codex CX1
- **status:** `verified_fixed`
- **files:** `crates/ai-brains-retrieval/tests/recall_global_prefer.rs`
- **required_fix:** Restore 15 leftover chrome rows for AC2/AC3; keep leftover-authority as a separately named merge test
- **evidence:** AC2/AC3 chrome; `merge_preferred_then_global__leftover_authority__owner_first` extra

### CX2-P3-1 — Review log retained pre-CX1 evidence

- **severity:** low
- **source:** Codex CX2
- **status:** `verified_fixed`
- **required_fix:** AC2 evidence = chrome fixture; notes must not claim post-rerank partition remains
- **evidence:** this file updated after CX2 (AC2 row + Notes)

## DoD matrix (AC1–AC16)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | `prefer_project.rs` units: preferred first, overlap once (`HashSet`), preferred-full skip, empty-preferred identity |
| AC2 | Met | `recall_full__global_prefer__owner_pin_beats_leftover_chrome` — 15 leftover **chrome** (`## Objective`) + owner DECISION; hit #1 is the owner pin. Leftover **authority** starve is a separately named merge test |
| AC3 | Met | `recall_full__global_prefer__leftover_still_in_candidates` — leftover still in recall hits (F41: pre-rerank merge is the contract; post-rerank may still include leftover when remainder > 0) |
| AC4 | Met | `recall__global_pretty__tags_project` — leading `[`+tag+`]` then space then `[score=` / `[rank=#` |
| AC5 | Met | `recall__global_json__no_project_id_key` — `results[]` has no `project_id` |
| AC6 | Met | `recall_full__preferred_none__no_fill_panic` + T274 chrome monopoly still green |
| AC7 | Met | `recall_full__chrome_monopoly__authority_pin_is_hit_one` green |
| AC8 | Met | `project_rebind_path` + `preflight_global_isolation` 28-test targeted run green |
| AC9 | Met | `recall__scoped_pretty__no_global_tag` |
| AC10 | Met | CAPABILITIES Scope + T264 rows; OPERATIONS rebind sentence; CHANGELOG T276 |
| AC11 | Met | no production `unwrap`/`expect`/`panic` in new modules; clap lock 4.6.1; rusqlite 0.39.0; no DTO keys |
| AC12 | Met | `list-paths --shared-only` print-only (11 leftover roots); no `--write --yes` |
| AC13 | Met | `POLICY_DENIED_HINT` / T275 grant-wall / T274 GLOB not edited |
| AC14 | Met | SQL + pretty only; recall without grants still works |
| AC15 | Met | `sync_query__global_pretty__tags_project` shares `print_pretty_hits_with_tags` |
| AC16 | Met | new T276 files do not recommend `set-alias` + leftover UUID + `AI-Brains` |

## Targeted gates (R1)

```text
cargo nextest run -p ai-brains-retrieval --test recall_global_prefer --test recall_pin_rank
  7 passed (2026-08-22)

cargo nextest run -p ai-brains-cli --test recall_global_prefer --test recall_pin_rank --test project_rebind_path --test preflight_global_isolation
  28 passed (5 slow)

cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings
  exit 0

.\scripts\dev-check.ps1
  SUCCESS; nextest 3266 passed / 1 skipped; deny + audit 19 allowed

ledgerful verify --scope full
  exit 0
```

## Manual (AC12)

```text
ai-brains project list-paths --shared-only --format json
  11 leftover C:\dev\* roots, all exists: true, project_id 7d97a456-…
  Did not run rebind-path --write --yes (F9; owner did not confirm)
```

## Notes

- `ranking.rs` product logic untouched; test helper gained `project_id: None` for the new `RecallHit` field (compile fill).
- F1 pipeline is merge then existing `rerank_hits` (no post-rerank preferred partition). F39 skips the unscoped MATCH when preferred fills depth.
- F18 PATH: no `cargo install`.
- F9: live leftover 11 roots unchanged.
