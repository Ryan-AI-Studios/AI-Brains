# T261 Plan — Recall empty-query latency

**Status:** **Pending** (requirements written; Planned spec; not In Progress)
**Spec:** [spec.md](./spec.md) F0–F19 / AC1–AC18 + §13 fold-in
**Category:** PERFORMANCE / UX / RETRIEVAL
**Ledger TX (planning):** `afe06292-7680-4d1f-b22e-a8a447f0a423` (DOCS)
**Ledger TX (fold-in):** `90a94ca0-7f0f-4989-a7af-443b4df7ff11` (DOCS)
**Ledger TX (implement):** start **FEATURE** on **go** only

---

## AI fold-in (2026-08-17) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. OpenCode **O2** folded as **F19** (contraction per-token). AC5 tightened (`endpoint == None`). FTS5 citation locked to live `fts.rs`. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F19 / AC13:** `"can't"` / `"what's"` contentless; `"i'll"` / `"don't"` contentful.
2. **AC5:** `skipped` + `contentless_query` + `endpoint=None`.
3. **§2.4:** do not cite truncated sqlite.org/fts5.html as SoT.
4. **F7 / AC14:** substring-before-COUNT stays (Agy-O1 already planned).
5. **§2.1:** plan dogfood `1842df0` vs plan commit `20bdd90`.

---

## Preflight (plan time — 2026-08-17)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `1842df0`. Plan commit `20bdd90`. Fold-in docs on that product src. |
| T261 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` (2026-08-17 18:20). Pre-T260 (`--symbols` unknown). **Do not `cargo install`.** |
| Source debug | `target\debug\ai-brains.exe` (2026-08-17 19:34) — newer than PATH. |
| Live hole | `""` **639 ms** no-bridge / **773 ms** default (audit **5730 ms not reproduced**). `"   "` **2145 ms + hits**. `"the the the"` **2027 ms + hits**. `"" --semantic` **1218 ms**. `"   " --global` **497 ms** (10k LIKE skip). |
| Pipeline | `recall_full`: bridge → lexical (empty tokens only) → substring (`is_empty` after COUNT) → semantic → graph → rerank. No contentless gate. |
| SoT | `contentful_tokens` / `extract_fts_tokens` in `ai-brains-core/src/fts.rs`. Add `is_contentless_query`. |
| clap / rusqlite / serde_json | lock clap **4.6.1** / builder **4.6.0** / crates.io **4.6.6**; rusqlite **0.39.0**; serde_json lock **1.0.150** / crates.io **1.0.151**. rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute. |
| Last PR Cursor | #175 comments/reviews/inline **0**. Dependabot only. **N/A.** |
| `deferred.md` | Full scan. Overlap: audit T261 **absorb**; T105/T217/T207/T86 **partial**; T262–T271 / T240 F2 / T255 **decline**. |
| ai-brains | `preflight --summary` ok (3581317d / 2854). Recall: T243 alias; T260 source-only. No contentless pin. |
| ledgerful | doctor ready (hygiene warns). 0 pending at start. Hotspot **#1** `project.rs` — do not touch. `forget.rs` #3 — do not filter. |
| Research | ES empty→match-all footgun (NEST #2179); `match_none` pattern. FTS5 token SoT is live `fts.rs` (sqlite.org/fts5.html fetch was nav-only — OpenCode O1). LIKE `% %` is match-all. clap 4 accepts `""`. |
| `ISSUES.md` | **Does not exist** |
| Live `.env` / leftover / nightly | **Not written** / **not rebound** / **not run** this pass. |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| `recall ""` 5.7 s | T256–T271 audit T261 | **DoD** F1–F7 / AC1–AC18 |
| Whitespace / stopword match-all | live dogfood | **DoD** AC2 / AC3 / AC7 / AC8 |
| `--semantic ""` embed | live 1218 ms | **DoD** F6 / AC5 |
| T86 piped empty error | `read_query_from_stdin` | **Absorb** F2 / AC11 (TTY stays) |
| T105 LIKE for contentful miss | T105 | **Keep** F4; contentless before COUNT F7 |
| T217 `contentful_tokens` | T217 | **Reuse** F1; do not change ladder |
| T207 empty chrome | T207 | **Reuse** F9 |
| T262–T271 / T240 F2 / T255 | deferred / standing | **Decline** F11 / F13 |
| last-PR Cursor | #175 | **N/A** — no leftover to mint |
| Contraction fragments (`can't` / `i'll`) | OpenCode O2 | **Absorb** F19 / AC13 |

---

## Phase 0 — on go (re-verify)

- [ ] Re-read `recall_full` (`recall.rs` ~237): still bridge → lexical → substring → semantic → graph → rerank. Confirm no contentless gate (or note drift and shrink F7).
- [ ] Re-read `contentful_tokens` / stopword list / `len < 2`. Confirm F1 still matches T217. Spot-check F19: `"can't"` contentless, `"i'll"` / `"don't"` contentful.
- [ ] Re-read `substring_fallback`: COUNT still before `query.is_empty()`.
- [ ] Re-read `read_query_from_stdin`: TTY refuse + piped empty `Err` still live.
- [ ] Classify-only dogfood: time `""`, `"   "`, `"the the the"`, `"" --semantic` `--format pretty --no-bridge`. Confirm hole or note drift. Adjust F3 live ceiling if SQLCipher open ≥ 500 ms.
- [ ] Re-check lock clap + crates.io: still no clap 5 (or this track is not that bump).
- [ ] Rescan **entire** `conductor/deferred.md` for new open empty-query / LIKE / stdin rows.
- [ ] Last merged PR + open HEAD PR Cursor comments. Mint placeholder if a leftover fits nowhere.
- [ ] `ledgerful ledger start T261-recall-empty-latency --category FEATURE`

---

## Phase 1 — Red (failing tests first)

- [ ] Core: `is_contentless_query__empty_whitespace_punct_stopword_single_char__true`
- [ ] Core: `is_contentless_query__ok_and_negator_phrase__false`
- [ ] Core: `is_contentless_query__contraction_fragments__per_token__ac13`
- [ ] Retrieval: `recall_full__empty_query__no_hits__ac1`
- [ ] Retrieval: `recall_full__whitespace__no_hits__ac2`
- [ ] Retrieval: `recall_full__all_stopword__no_hits__ac3`
- [ ] Retrieval: `recall_full__semantic_contentless__embedding_skipped__ac5`
- [ ] Retrieval: `recall_full__symbols_contentless__still_empty__ac17`
- [ ] Retrieval: `substring_fallback__whitespace__empty_before_count__ac14`
- [ ] CLI: `recall__empty_pretty__hint_no_hits__ac6`
- [ ] CLI: `recall__whitespace_pretty__no_hit_lines__ac7`
- [ ] CLI: `recall__stopword_pretty__no_hits__ac8`
- [ ] CLI: `recall__empty_json__results_empty__ac9`
- [ ] CLI: `search__empty_pretty__alias__ac10`
- [ ] CLI: `recall_stdin__piped_empty__short_circuit__ac11`
- [ ] Commit red (allowed).

---

## Phase 2 — Green

- [ ] `is_contentless_query` in `fts.rs` + `pub use` in core `lib.rs`.
- [ ] `recall_full` early return (F7) + F6 `skipped` / `contentless_query`.
- [ ] `substring_fallback`: contentless return **before** COUNT.
- [ ] `read_query_from_stdin`: piped trim-empty → `Ok("")`.
- [ ] Do **not** gate `lexical_search` (AC15).
- [ ] Do **not** touch `forget.rs`, `project.rs`, `ranking.rs`, contracts fields.
- [ ] Commit green.

---

## Phase 3 — Docs + targeted gate

- [ ] CAPABILITIES recall table: contentless → T207 empty; no LIKE / bridge / embed; all-stopword honesty.
- [ ] CHANGELOG: whitespace / all-stopword no longer match-all.
- [ ] `cargo nextest run -p ai-brains-core -p ai-brains-retrieval -p ai-brains-cli` (targeted names + T105/T207/T217/T86/T260).
- [ ] `cargo clippy -p ai-brains-core -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings`
- [ ] `cargo fmt --check`

---

## Phase 4 — Finalize (on go only)

- [ ] Full gate: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit`
- [ ] `ledgerful verify --scope full`
- [ ] AC18 live timing note in review.md / plan
- [ ] FEATURE TX commit; conductor **Completed** only after implement-track publish
- [ ] Pin: `DECISION: contentless recall (0 contentful tokens) is T207 empty; no LIKE/bridge/embed/graph.`

---

## Definition of Done

- [ ] AC1–AC18 green (AC18 is a recorded live timing note)
- [ ] F0–F19 honored
- [ ] T105 / T207 / T217 / T86 contentful / T260 suites still green
- [ ] `forget --match` unfiltered
- [ ] No clap 5 / new crates / `.env` write / `cargo install`
- [ ] CAPABILITIES + CHANGELOG updated
- [ ] Medium+ review findings not silently dropped

---

## Stop-before

- Destructive git / force-push / push `main`
- Scope exceeds this track (T262–T271, projection column, ANN, forget filter)
- Ambiguous clap-reject vs short-circuit (frozen: short-circuit)
- Unrelated workspace failures (triage; do not broadly clean up)
