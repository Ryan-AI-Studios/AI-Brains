# T217 Review Log — FTS multi-token / natural-phrase rescue

## Round 1 — Internal (read-only)

**Date:** 2026-08-09
**Reviewer:** Grok Build subagent (code review)
**Branch:** `feat/T217-fts-multitoken-rescue`
**Scope:** Spec F1–F22 / D1–D14 / AC1–AC17 vs production + tests (no production edits)

### Verdict: **CLEAN**

No critical/high/medium findings. Hard DoD items (rescue ladder, forget opt-out, raw-query / no double-sanitize, SQL LIMIT, `_` split, negators, core-helper hint, exports/callers, CAPABILITIES+CHANGELOG) are implemented correctly. Residual items below are low test/docs hygiene only.

---

### Findings

| id | severity | description | files | required_fix | status |
|----|----------|-------------|-------|--------------|--------|
| T217-R1-01 | low | **AC15 R2-specific LIMIT proof is thin.** Hermetic LIMIT exercises R0; shared `match_query` path binds LIMIT for R1/R2. Stronger R2-only fixture optional. | `lexical_rescue.rs`; `lexical.rs` | Optional R2 OR fixture. | deferred |
| T217-R1-02 | low | **R1 path not hermetically isolated** in original AC1. | `lexical_rescue.rs` | Added `lexical_rescue__stopword_phrase__hits_via_r1_contentful_and`. | verified_fixed |
| T217-R1-03 | low | **CAPABILITIES Hints row omits T217 fewer-keywords.** | `Docs/CAPABILITIES.md` | Hints row updated with T217 fewer-keywords clause. | verified_fixed |
| T217-R1-04 | low | **Track meta drift** (Planning vs implement). | `spec.md` / conductor | Coordinator updates on PR closeout. | deferred |
| T217-R1-05 | low | **No dedicated AC2 single-token hermetic.** | `lexical_rescue.rs` | Added `lexical_rescue__single_token_known_pin__hits`. | verified_fixed |

---

### Checklist vs review focus

| Focus | Result | Notes |
|-------|--------|-------|
| 1. Incomplete DoD / missing ladder steps | **pass** | R0 AND → (rescue ∧ empty ∧ ≥3) → contentful empty short-circuit → R1 if `c != tokens` → R2 if `\|c\|≥2` + `select_or_tokens` cap 8 → recall then T105. Matches D1/D2/§7.2. |
| 2. forget widening (`rescue` must be false) | **pass** | `LexicalSearchOptions::default()` → `rescue: false`; both forget match + UUID preview paths use default. Hermetic AC14 empty under rescue=false. |
| 3. Double-sanitize breaking OR | **pass** | `recall_full` sanitizes **bridge only**; lexical gets **raw** query; `match_query` does not call `sanitize_fts_query`. F9/F10/AC10. |
| 4. LIMIT missing / after privacy incorrectly | **pass** | Privacy exclusions in SQL `WHERE` before `ORDER BY rank LIMIT ?`. Bound `min(caller, 200)`; recall uses `candidate_depth`. F19/D13. |
| 5. Underscore not split | **pass** | `extract_fts_tokens` splits on `!is_alphanumeric()` (includes `_`); sanitize SOOT; AC16 pure test. |
| 6. Negators dropped | **pass** | §4.1 include set only; `not`/`no`/`never`/… not stopwords; AC6b pure + contentful test. |
| 7. Hint not core / all-stopword false advice | **pass** | CLI uses `ai_brains_core::should_suggest_fewer_keywords`; AC7 + AC7b unit tests. |
| 8. unwrap/expect in production | **pass** | None in `fts.rs` / `lexical.rs` T217 paths; forget uses `unwrap_or` only for line preview. |
| 9. Missing tests | **mostly pass** | Pure AC5/6/6b/16/17; hermetic AC1/3/4/14/15; CLI AC7/7b. Soft gaps: R1 isolation, R2 LIMIT, AC2 local, AC9 multi-token ladder→T105 (pre-existing T105 still green for non-ladder empty). |
| 10. Wrong exports / broken callers | **pass** | Core re-exports helpers + `LEXICAL_MATCH_HARD_CAP`; retrieval exports `LexicalSearchOptions`, `match_limit_bound`, `lexical_search`. All in-repo callers updated. Control-plane/sync keep sanitize-only (no rescue). |
| 11. Docs drift | **pass** (minor low) | CHANGELOG + CAPABILITIES FTS5 + FTS5-catch T217 pointer present. Hints row residual = R1-03. |

---

### F / D / AC audit (summary)

| ID | Result | Evidence |
|----|--------|----------|
| **D1** rescue only when empty R0 ∧ ≥3 ∧ rescue | **met** | `lexical.rs` 70–73 |
| **D2** ladder R0→R1→R2→T105 | **met** | lexical ladder; `recall_full` substring after lexical empty |
| **D3** literal stopwords; negators out; len≥2; dedupe | **met** | `fts.rs` + pure tests |
| **D4** no auto-semantic | **met** | semantic still opt-in only |
| **D5** source=fts; BM25 rank | **met** | `RecallHit::fts`; score from `fts.rank` |
| **D6** SOOT core builders / retrieval MATCH | **met** | split of responsibilities |
| **D7** fewer-keywords via core; contentful≥1 | **met** | `should_suggest_fewer_keywords` |
| **D8** capture independence | **met** | SQLCipher FTS only on rescue path |
| **D9** rescue default false; recall true; forget false | **met** | Default + call sites |
| **D10** no dep bumps / no FTS rebuild | **met** | no migration/tokenchars in track files |
| **D11** privacy/scope every stage | **met** | same `match_query` for R0–R2 |
| **D12** OR select length desc, lexical asc, cap 8 | **met** | `select_or_tokens` + pure tests |
| **D13** SQL LIMIT every MATCH | **met** | `match_query` |
| **D14** `_` split SOOT | **met** | extract + sanitize |
| **F1–F22** | **met** | see focus table; soft test residuals only |
| **AC1** | **met** (via R2) | hermetic rescue true hits + score Some |
| **AC2** | **met** (indirect) | existing single/two-token fixtures |
| **AC3–AC4** | **met** | hermetic |
| **AC5–AC6b, AC16–AC17** | **met** | pure core |
| **AC7–AC7b** | **met** | CLI unit |
| **AC8** | **met** | no embed dep on default lexical |
| **AC9** | **met** (code + pre-T105 hermetic) | ladder empty → substring still called |
| **AC10** | **met** (code review) | raw + rescue true; bridge sanitized |
| **AC11** | **not re-run here** | review is static; focused nextest is plan Phase 5 |
| **AC12** | **met** | CHANGELOG + CAPABILITIES FTS5 |
| **AC13** | **pending** | full gate + this review clean (R1) |
| **AC14–AC15** | **met** | hermetic + unit bound (R1-01 soft) |

---

### Round 2 — Codex cross-model (gpt-5.6-luna high)

**Date:** 2026-08-09  
**Artifact:** `review.codex.md`

| id | severity | disposition | action |
|----|----------|-------------|--------|
| P1-01 | process | **process** — mid-loop; Phase 5 gates run by orchestrator after fixes | Full gate + PR + closeout in progress |
| P1-02 | env | **false_positive** — Codex read-only sandbox could not open ledger/vault DBs; orchestrator already ran doctor/ledger/preflight successfully | N/A |
| P2-01 | process | **validated** | Stage `lexical_rescue.rs` on commit |
| P2-02 | medium | **validated** | Added `lexical_rescue__r2_or_respects_limit_and_excludes_sealed` |
| P2-03 | medium | **validated** | Replaced for-loop fixture setup with explicit pin_extra calls |
| P3-01 | low | **validated** | Trailing whitespace cleaned (`git diff --check`) |

### Architecture notes (affirmations)

- **Dogfood root cause addressed:** NL multi-token fails R0 implicit AND; contentful AND (R1) and/or OR (R2) recover; forget cannot inherit widen.
- **M6 non-stopword 3+:** when `contentful == tokens`, R1 skipped, R2 still runs if `|c|≥2` — correct.
- **Single contentful after stopword strip:** R1 can still fire (`c != tokens`); R2 requires ≥2 contentful — correct.
- **Bridge / control-plane:** intentionally no rescue (D9 / soft residual).

---

### Soft residuals (spec §13 — not findings)

- Bridge multi-round rescue; control-plane evidence FTS rescue; JSON rescue-stage field; locale stopwords; porter/trigram/`tokenchars`/fts5vocab; FTS5 `NEAR`; T218/T224/T231.

---

### Closure recommendation

Safe to proceed to focused nextest + manual dogfood + full gate + cross-model FEATURE review. No production code change required from this R1.
