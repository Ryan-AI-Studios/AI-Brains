# T211 Internal Review — Sync query ranking + stale DECISION demotion

- **Reviewer:** internal re-review (read-only)
- **Date:** 2026-08-04 (re-verify pass)
- **Branch:** `track/T211-sync-query-ranking-staleness`
- **Scope:** R1–R3 re-verify + fresh F1–F42 / AC1–AC12 sweep (diff + file read)
- **Sources:** `spec.md`, `plan.md`, `ranking.rs`, `sync.rs`, `recall.rs`, `recall` CLI pretty helpers, contracts, hermetic tests, CAPABILITIES/CHANGELOG
- **Prior status:** R1–R3 `fixed_pending_verification`

## Verdict: CLEAN

R1–R3 fixes are present and correct against hard ACs / F12 / F33. Fresh sweep found **no new medium+** findings. Soft residuals (R4) remain non-blocking.

---

## Re-verify: prior findings

### T211-R1 — FTS BM25 polarity inverted under composite DESC sort

| Field | Value |
|-------|--------|
| **id** | T211-R1 |
| **severity** | high |
| **status** | **verified_fixed** |
| **source** | internal |
| **files** | `crates/ai-brains-retrieval/src/ranking.rs` (`effective_score`, `rerank_hits`) |
| **description** | FTS5 `rank` / bm25 is **more negative = better match**. Raw rank as base + DESC sort inverted same-kind order. |
| **required_fix** | `base = -score` (None→0.0); AC1b revalidated; unit `rerank_hits__same_kind_better_bm25_first__f33`. |
| **verification** | **PASS.** `effective_score` (lines ~190–194): `Some(s) => -s`, `None => 0.0`. Module docs state BM25-preserving higher-is-better base for DESC sort. AC1b asserts magnitudes **3.5 > 2.0** (shipped base 0.5+2+1; plan base 3+2−3). F33 same-kind unit: score −10.0 ranks above −0.5 for plain Other. Known gate: ranking 13 pass. |
| **note** | Spec F8 literal `base = score.unwrap_or(0.0)` is intentionally adapted for F33 under DESC sort; docs + AC1b lock the BM25-preserving convention. Semantic higher-is-better polarity is out of T211 hard scope (F2 / T215). |

### T211-R2 — F12 secondary ledger-first condition incomplete

| Field | Value |
|-------|--------|
| **id** | T211-R2 |
| **severity** | medium |
| **status** | **verified_fixed** |
| **source** | internal |
| **files** | `crates/ai-brains-cli/src/commands/sync.rs` (`run_query`) |
| **description** | F12 secondary branch “every vault Decision is Plan” was missing. |
| **required_fix** | `top_is_plan \|\| every_decision_is_plan` via `classify_pin_kind` + `is_plan_demoted`. |
| **verification** | **PASS.** `run_query` (~497–510): `top_is_plan = hits.first().is_some_and(|h| h.is_plan_demoted)`; `every_decision_is_plan` filters Decision kind then `all(is_plan_demoted)` with non-empty guard; `ledger_first = ledger_non_empty && (top_is_plan \|\| every_decision_is_plan)`. Banner + section reorder on true; fail-open vault-only when probe empty/missing. |

### T211-R3 — No same-kind FTS order regression unit (coverage gap for R1)

| Field | Value |
|-------|--------|
| **id** | T211-R3 |
| **severity** | medium |
| **status** | **verified_fixed** |
| **source** | internal |
| **files** | `crates/ai-brains-retrieval/src/ranking.rs` tests |
| **description** | Missing same-kind FTS order unit. |
| **required_fix** | Add pure unit covering R1. |
| **verification** | **PASS.** `rerank_hits__same_kind_better_bm25_first__f33` present: weak `Some(-0.5)` vs strong `Some(-10.0)`, asserts `strong` first. |

### T211-R4 — Soft residuals (non-blocking)

| Field | Value |
|-------|--------|
| **id** | T211-R4 |
| **severity** | low |
| **status** | open |
| **source** | internal |
| **files** | plan soft items; CAPABILITIES ledger-first wording; `deferred.md` |
| **description** | Soft not required for hard DoD: AC10 ledger-first hermetic/manual evidence; AC12 `--limit 1` hermetic; F25 full vault↔ledger blend; OPERATIONS/skill one-liner; deferred.md strike T211 + F25 note. F26 `staleness` **shipped** (OK). AC9 full CI gate not executed in this re-review session. CAPABILITIES ledger-first blurb mentions top-hit only (not secondary “every Decision is Plan”) — soft docs completeness. Semantic score polarity under `base = -score` is a T215 / `--semantic` interaction, not T211 hard surface (`sync` uses `semantic: false`). |
| **required_fix** | Optional before ship: manual `sync query "path TOCTOU"`; soft tests if free; deferred.md on finalize; optional CAPABILITIES secondary F12 clause. |

---

## Fresh regression sweep (hard ACs + F1–F42)

No new **medium+** findings. Hard path remains coherent.

| Area | Result |
|------|--------|
| BM25 / F33 | `base = -score`; same-kind unit; AC1b magnitudes with negation |
| F12 ledger-first | Primary + secondary branches; banner exact string; probe fail-open |
| F37 sync pretty | `recall_full` direct; `semantic: false`; `no_bridge: true` on vault arm; shared pretty helpers |
| F27 `--limit` | `default_value_t = 5` on `SyncCommands::Query` |
| F11 badge | Single-site `format_pretty_hit_line` / `print_pretty_hits`; `[plan/stale?]` |
| F16/F38 `updated_at` | FTS/substring/graph constructors; bridge None |
| F6/F18 | Manual track tokens; no `regex` in retrieval Cargo.toml |
| F10 / F19 | Rank/presentation only; no models on sync pretty vault arm |
| Production unwrap | No `unwrap`/`expect`/`panic!` in ranking production body or sync F12 path |
| Hermetic | `sync_query_ranking.rs` AC1/AC3/AC6/AC2; `sync_query_isolation.rs` intact |
| Docs | CAPABILITIES re-rank + ledger-first; CHANGELOG T211 entry |
| Soft F26 | `RecallResult.staleness` additive `skip_serializing_if` |

---

## Checklist (audit)

| Check | Result |
|-------|--------|
| 1. Hard AC1–AC9, AC11 evidence | **AC1,1b,2,3,4,5,6,8,11 met in code/tests; AC7 isolation file intact; AC9 not run this re-review** |
| 2. No placeholders / false-green | **Met** — real ACs + F33 unit |
| 3. F37 sync pretty → `recall_full` not `recall::run` | **Met** |
| 4. F12 `--json` probe + ledger-first + banner | **Met** (primary + secondary OR) |
| 5. F27 `--limit` default 5 | **Met** |
| 6. F8 single composite; F9 magnitudes; AC1b unit | **Met** (BM25-preserving base; AC1b 3.5>2.0) |
| 7. F6 manual track tokens; no regex in retrieval | **Met** |
| 8. F16/F38 `updated_at` all constructors; graph SQL; bridge None | **Met** |
| 9. F11 badge single-site shared helper | **Met** |
| 10. T207 empty pretty preserved | **Met** (`print_pretty_empty_sync`) |
| 11. No production unwrap/expect/panic | **Met** on T211 ranking/sync pretty paths |
| 12. Capture independence | **Met** |
| 13. Soft F25 not required; F26 optional | **F25 residual; F26 shipped** |

---

## DoD matrix

| Requirement | Status | Evidence |
|-------------|--------|----------|
| **AC1** Plan below Shipped same track | **met** | unit + hermetic |
| **AC1b / AC11** score-gap magnitudes | **met** | unit 3.5>2.0 with BM25 negation |
| **AC2** CONSTRAINT > Other | **met** | unit + hermetic |
| **AC3** pretty `plan/stale?` | **met** | hermetic + badge helper |
| **AC4** sibling demotion | **met** | unit |
| **AC5** memory_id asc ties | **met** | unit |
| **AC6** `--no-bridge` re-ranks | **met** | hermetic uses `--no-bridge` |
| **AC7** isolation regression | **met** (review) | `tests/sync_query_isolation.rs` present; ranking doesn’t scope-break |
| **AC8** CAPABILITIES + CHANGELOG | **met** | re-rank / ledger-first / T211 entry |
| **AC9** CI gate / no prod panic | **partial** | no prod unwrap in new paths; full gate **not run** this re-review |
| **AC10** ledger-first (soft) | **partial** | code path + banner; no hermetic/manual log |
| **AC12** `--limit 1` (soft) | **unmet** (soft) | clap default 5 only; no limit=1 test |
| **F1** surface sync + shared recall | **met** | |
| **F2** no semantic on sync | **met** | `semantic: false` |
| **F3** no progressive rewrite | **met** | untouched |
| **F4 / F42** pin kind + ASSISTANT strip | **met** | |
| **F5** staleness classes | **met** | Plan precedence |
| **F6 / F18** manual tokens; no regex dep | **met** | |
| **F7** recency tie-break | **met** | |
| **F8** single composite None→0 | **met** | None→0.0; FTS base negated for DESC (F33) |
| **F9** boost consts | **met** | |
| **F10** no pin delete | **met** | |
| **F11** badge | **met** | |
| **F12** ledger preference | **met** | top Plan **or** every Decision Plan |
| **F13** section model | **met** | reorder only |
| **F14** limit 5 after re-rank | **met** | |
| **F15 / F26** pretty hard; JSON soft | **met** | F26 shipped |
| **F16 / F38** updated_at map | **met** | |
| **F17** graph/bridge scores as base | **met** | same `effective_score` path |
| **F19** capture independence | **met** | |
| **F27** `--limit` DoD | **met** | |
| **F30** determinism memory_id | **met** | |
| **F33** do not invert BM25 | **met** | `base = -score` + F33 unit |
| **F34** badge honesty | **met** | |
| **F37** direct recall_full + shared pretty | **met** | |
| **F39** AC1b | **met** | |
| **F40** single entry `rerank_hits` | **met** | |
| **F25** full blend | **soft residual** | not required |

---

## Completeness notes

### What remains solid after re-verify

- Pure `ranking.rs` with F9 consts, classifiers, F40 entry-point docs, 13 pure tests including F33 same-kind BM25.
- **F37** pretty `sync query` → `recall_full`; shared badge helper; T207 empty via `print_pretty_empty_sync`.
- **F12** primary + secondary ledger-first; `ledger_json_non_empty` units; fail-open.
- **F27** default limit 5; truncate after re-rank.
- Soft **F26** JSON `staleness`.
- Capture independence and no production panic on T211 paths.

### Blocking vs soft (post re-verify)

| Severity | Count | Action |
|----------|-------|--------|
| critical | 0 | — |
| high | 0 (R1 **verified_fixed**) | — |
| medium | 0 (R2, R3 **verified_fixed**) | — |
| low | 1 (R4) | soft / finalize hygiene only |

### Explicit non-findings (re-confirm)

- F37 does not call `recall::run` for pretty.
- Capture independence preserved on sync pretty vault arm.
- No auto-forget / semantic default / progressive rewrite.
- F25 full blend correctly not implemented.
- No new unwrap/expect/panic in T211 ranking/sync production paths.
- No new medium+ regressions vs hard ACs / F1–F42 in this read-only pass.

---

## Cross-model (Claude) — final gate

- **Date:** 2026-08-05
- **Reviewer:** Claude Sonnet (Codex rate-limited)
- **Artifact:** `review.claude.md`
- **Verdict:** **PASS WITH DEFERRED P3**
- **P0–P2:** none
- **P3 deferred:** double ledger shell (F12 allows); re-classify cost; soft AC10 manual; deferred.md ship hygiene; CAPABILITIES F40 wording (addressed lightly)
- **Soft AC12:** hermetic `--limit 1` added after review

## Final completion decision

Engineering DoD met. Internal CLEAN (R1–R3 verified_fixed). Cross-model PASS WITH DEFERRED P3. Full local gate: fmt, clippy workspace, nextest 2110 pass, deny ok, audit pre-existing warnings only, ledgerful verify fast pass. Manual path TOCTOU: shipped first + `[plan/stale?]` on plan.
