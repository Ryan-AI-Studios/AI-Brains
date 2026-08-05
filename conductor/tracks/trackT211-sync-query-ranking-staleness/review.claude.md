Now I have enough evidence to render the verdict. Let me complete the review.

---

# T211 Independent Completion Review ΓÇö Sync Query Ranking + Stale DECISION Demotion

**Reviewer:** Claude Sonnet 4.6 (independent, read-only)
**Date:** 2026-08-04
**Branch:** `track/T211-sync-query-ranking-staleness` vs `origin/main`
**Scope:** spec.md ┬º4 (F1ΓÇôF42), ┬º6 (AC1ΓÇôAC12), plan.md DoD checklist; all modified and new files

---

## Evidence Map

| File | Verified |
|------|---------|
| `ai-brains-retrieval/src/ranking.rs` (new) | Full read |
| `ai-brains-retrieval/src/lexical.rs` | Full read |
| `ai-brains-retrieval/src/recall.rs` | Full read |
| `ai-brains-retrieval/src/lib.rs` | Full read |
| `ai-brains-retrieval/src/semantic.rs` | Partial (updated_at plumbing) |
| `ai-brains-cli/src/commands/sync.rs` | Full read |
| `ai-brains-cli/src/commands/recall.rs` | Full read |
| `ai-brains-cli/src/main.rs` | SyncCommands::Query + dispatch |
| `ai-brains-contracts/src/recall.rs` | Full read |
| `ai-brains-cli/tests/sync_query_ranking.rs` (new) | Full read |
| `ai-brains-cli/tests/sync_query_isolation.rs` | Full read |
| `ai-brains-cli/tests/common/mod.rs` | Full read |
| `ai-brains-retrieval/Cargo.toml` | Full read (no regex) |
| `Docs/CAPABILITIES.md` ┬º7 recall + ┬º7 sync query | Verified |
| `CHANGELOG.md` line 20 | Verified |
| `conductor/deferred.md` | Verified |
| `conductor/tracks/trackT211-*/review.md` | Full read |

---

## Requirement-by-Requirement Audit

### F33 ΓÇö FTS BM25 polarity (prior R1, flagged high)

**Verdict: VERIFIED FIXED.** `effective_score` at `ranking.rs:191` uses `base_v = -s` for `Some(s)` ΓÇö negates the raw FTS rank so that more-negative BM25 (better match) becomes a larger positive base in the DESC composite sort. `None ΓåÆ 0.0` correctly prevents unscored hits from leapfrogging strong FTS. AC1b magnitudes: `shipped_eff = 0.5 + 2.0 + 1.0 = 3.5`; `plan_eff = 3.0 + 2.0 ΓêÆ 3.0 = 2.0`; `3.5 > 2.0` Γ£ô. The same-kind BM25 order unit (`rerank_hits__same_kind_better_bm25_first__f33`) asserts `Some(-10.0)` ranks above `Some(-0.5)` for plain Other Γ£ô. "Do not invert BM25" = do not reverse the relative preference order; this implementation preserves it.

### F37 ΓÇö sync pretty ΓåÆ `recall_full` directly, not `recall::run` (prior R2, medium)

**Verdict: VERIFIED FIXED.** `sync.rs:470` calls `ai_brains_retrieval::recall_full(...)` on the pretty path. The vault arm sets `no_bridge: true` explicitly (line 483); the ledger is a separate section via `probe_ledger_search`. `recall::run` is not called. Shared pretty helpers (`print_pretty_hits`, `format_pretty_hit_line`) are `pub` in `recall.rs` and used via `crate::commands::recall::print_pretty_hits(&hits)`. T207 empty path preserved via `print_pretty_empty_sync` Γ£ô.

### F12 ΓÇö Ledger-first: primary + secondary condition (prior R2)

**Verdict: VERIFIED FIXED.** `run_query` computes both conditions:  
- `top_is_plan = hits.first().is_some_and(|h| h.is_plan_demoted)` ΓÇö top vault hit is Plan-class Decision.  
- `every_decision_is_plan` ΓÇö re-classifies with `classify_pin_kind`, filters to Decisions, checks all `is_plan_demoted` with non-empty guard.  
- `ledger_first = ledger_non_empty && (top_is_plan || every_decision_is_plan)`.  
Banner exact string `"Note: vault top hit is plan/stale; ledger results shown first."` matches spec F12/L5. Fail-open: missing/failed `ledgerful` ΓåÆ `probe_ledger_search` returns `None` ΓåÆ vault-only, no panic Γ£ô.

### F8 / F9 ΓÇö Single composite, F9 magnitudes, NoneΓåÆ0.0

**Verified.** Consts: `KIND_CONSTRAINT = 4.0`, `KIND_DECISION = 2.0`, `KIND_HOTSPOT = 0.5`, `SHIPPED_BOOST = 1.0`, `PLAN_PENALTY = 3.0`, `SIBLING_PLAN_PENALTY = 2.0`, `RECENCY_SCALE = 1.0`. Single `rerank_hits` replaces old None-last bucket sort. Sort: effective desc ΓåÆ `updated_at` desc (missing last) ΓåÆ `memory_id` asc (AC5) Γ£ô.

### F4/F42 ΓÇö ASSISTANT: strip; case-insensitive leftmost marker

**Verified.** `strip_assistant_prefix` calls `.strip_prefix("ASSISTANT: ").unwrap_or(content)` ΓÇö safe, single-call. `classify_pin_kind` finds leftmost occurrence among all three markers (not first-checked-wins). Unit tests cover lowercase, mixed-case, strip Γ£ô.

### F5 ΓÇö Staleness classes; Plan-first precedence

**Verified.** Plan markers checked before Shipped (demotion-honest). Markers exactly match spec: `["plan-only","placeholder","expanded","until go","not dod","planning"]` / `["shipped","complete","closed by","pr #","squash-merged","verified_fixed"]`. Non-Decision kind always returns Unknown Γ£ô.

### F6/F18 ΓÇö Manual track tokens; no regex in retrieval

**Verified.** `extract_track_tokens` is a byte-scan loop with no `regex` dep. `ai-brains-retrieval/Cargo.toml` has no `regex` dependency Γ£ô. Word-boundary check: `i == 0 || !bytes[i-1].is_ascii_alphanumeric()`. Tokens normalized to `T{digits}`, sorted, deduped. Test confirms `"XT999"` (no preceding boundary) yields empty result Γ£ô.

### F16/F38 ΓÇö `updated_at` plumbed through all constructors

**Verified.**  
- FTS: `SELECT mp.memory_id, mp.content, mp.privacy, mp.session_id, fts.rank, mp.updated_at` Γ£ô  
- Substring: `SELECT memory_id, content, privacy, session_id, updated_at FROM memory_projection` Γ£ô  
- Graph: `SELECT content, updated_at FROM memory_projection WHERE memory_id = ?1` (second column) Γ£ô  
- Bridge: `updated_at: None` explicitly, with doc comment "F16" Γ£ô  
- Semantic: `updated_at: None` with `// Semantic arm does not SELECT memory updated_at (F16: None OK)` Γ£ô  

### F27 / F14 ΓÇö `--limit` default 5, truncate after re-rank

**Verified.** `SyncCommands::Query` at `main.rs:1603`: `#[arg(short = 'l', long, default_value_t = 5)] limit: usize`. Truncation in `recall_full` after `rerank_hits`: `if blended.len() > limit { blended.truncate(limit); }` Γ£ô.

### F11 / F26 ΓÇö Pretty badge; soft JSON staleness

**Verified.** `format_pretty_hit_line` sets `badge = "[plan/stale?] "` when `is_plan_demoted`, placed after `memory_id: ` before content (spec: "before content") Γ£ô. `RecallResult.staleness` in contracts has `#[serde(default, skip_serializing_if = "Option::is_none")]`; set to `Some("plan".to_string())` in `recall::run` when `is_plan_demoted` Γ£ô.

### F40 ΓÇö Single post-blend entry point (T215 forward-compat)

**Verified.** Module docstring (lines 1ΓÇô7) and `rerank_hits` docstring both state T215 must extend this function, not add a second final sort. CAPABILITIES ┬º7 recall table (line 193): "the only post-blend ranking entry point; T215 semantic/RRF extends it, does not add a second final sort" Γ£ô. Sync query table: "Same `recall_full` + `rerank_hits` path as `recall`" Γ£ô.

### F19 ΓÇö Capture independence

**Verified.** Sync pretty vault arm: `semantic: false`, `no_bridge: true`. No models, no embeddings, no graph DB required. `probe_ledger_search` is an optional shell-out; fail-open Γ£ô.

### AC7 / F1 ΓÇö Isolation regression; shared ranking path

**Verified.** `sync_query_isolation.rs` has 3 tests: scoped default, global cross-project, NDJSON scoped. Ranking reorders hits but does not change the scoped query predicate; these tests remain correct Γ£ô.

### AC8 ΓÇö CAPABILITIES + CHANGELOG

**Verified.** `CHANGELOG.md:20` has the T211 entry covering re-rank, plan demotion, sibling track demotion, no regex dep, badge, `recall_full` direct, `--limit` default 5, ledger `--json` probe + ledger-first, soft staleness JSON, hermetic. CAPABILITIES ┬º7 has Vault re-rank, Plan badge, Ledger-first, and Honesty rows Γ£ô.

### AC9 ΓÇö No production unwrap/expect

**Verified.** In `ranking.rs` production paths: only `.unwrap_or(content)` (safe). In `sync.rs` new code: all error paths use `?`, `.ok()`, or `match`; no bare `unwrap()/expect()`. In `recall.rs` retrieval: graph `conn.lock().ok()` + `.and_then(...)` Γ£ô. Test files use `#[allow(clippy::disallowed_methods)]` correctly confined to tests.

---

## Hermetic Test Coverage

| AC | Test | Location |
|----|------|----------|
| AC1 + AC6 | `sync_query_ranking__shipped_above_plan_no_bridge__ac1_ac6` | `tests/sync_query_ranking.rs` |
| AC1b / AC11 | `rerank_hits__shipped_worse_fts_beats_plan_better_fts__ac1b` | `ranking.rs` unit |
| AC2 | `sync_query_ranking__constraint_above_other` + `rerank_hits__constraint_outranks_other__ac2` | hermetic + unit |
| AC3 | `sync_query_ranking__plan_badge__ac3` | hermetic |
| AC4 | `rerank_hits__sibling_plan_demotion__ac4` | unit |
| AC5 | `rerank_hits__equal_scores_memory_id_asc__ac5` | unit |
| AC7 | `sync_query_isolation.rs` (3 tests) | hermetic regression |
| F33 | `rerank_hits__same_kind_better_bm25_first__f33` | unit |
| AC10 | soft/manual | (not automated ΓÇö spec-permitted) |
| AC12 | soft/not present | (not required) |

---

## P3 Findings (non-blocking, evidence-based)

### P3-1 ΓÇö Double shell invocation in `probe_ledger_search`

`probe_ledger_search` (sync.rs:558ΓÇô630) makes two process spawns: `ledgerful ledger search --json <q>` for non-empty detection, then `ledgerful ledger search <q>` for human display. Spec F12 says "single call preferred if free." The double invocation introduces a race window where the probe call and the display call can return different result sets (ledger state could change between the two ms-apart calls), and adds unnecessary process-spawn latency. The spec permits it ("may re-run without --json") but the preference was noted. Not blocking; residual for a cleanup pass.

### P3-2 ΓÇö `every_decision_is_plan` redundant re-classify

`run_query` (sync.rs:501ΓÇô508) re-calls `classify_pin_kind` on each hit after `rerank_hits` has already classified content. A `pin_kind: PinKind` field on `RecallHit` would eliminate the double scan; instead the current code re-classifies on every `sync query` call. Correct but slightly redundant. Not blocking.

### P3-3 ΓÇö Soft ACs AC10 / AC12 not covered by automated tests

Per spec, AC10 (ledger-first hermetic) is "Soft / manual" and AC12 (`--limit 1` hermetic) is "AC12 Soft." Both correctly unimplemented per spec direction. Noted for completeness; no action required before merge.

### P3-4 ΓÇö `deferred.md` T211 not yet struck; F25 note absent

`plan.md` checklist has `[ ] deferred.md: strike T211 on ship; note F25 blend residual if not shipped`. `deferred.md:36` still shows T211 as "Planning", not struck. This is a pre-ship hygiene task; code itself is correct.

### P3-5 ΓÇö CAPABILITIES sync query table omits `rerank_hits` single-entry-point wording

The sync query `Honesty` row says "Semantic embedding relevance is T215" without explicitly stating `rerank_hits` is the single entry point. F40 coverage exists in the recall table (line 193) and in `ranking.rs` module/function docs. Asymmetry is mild; cross-reference is sufficient.

---

## Verdict

### **PASS WITH DEFERRED P3**

All hard acceptance criteria (AC1ΓÇôAC9, AC11) are met by evidence in code and tests. No P0, P1, or P2 findings. Prior internal R1 (BM25 polarity, high) and R2 (every-Decision-is-Plan secondary condition, medium) are verified fixed with correct unit coverage. F18 zero-new-crates for retrieval (no `regex` dep), F37 direct `recall_full` (not `recall::run`), F40 single post-blend entry point for T215, and F38 `updated_at` plumbing through all four constructors are all confirmed. Five P3 items are deferred: double shell probe call, redundant re-classify, soft AC10/AC12 tests absent per spec, deferred.md pre-ship hygiene, and minor CAPABILITIES wording asymmetry on F40.
