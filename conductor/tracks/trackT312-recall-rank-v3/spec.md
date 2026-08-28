# T312 — Recall rank v3: pins and synth over review dumps

- **Track ID:** T312-RecallRankV3
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T312`.
- **Category:** FEATURE / UX / RETRIEVAL
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — finding #2 (“ranking is the product”); `recall` FTS 9/8 but dump-first; `--semantic` 9/**7**. Series README `README-T312-T324-CLI-DOGFOOD.md`.
- **Depends on:** T285 ✅ rank v2 (envelope + chrome prefixes + TAGS-or-authority + chrome-seed skip); T274 ✅ −16 chrome; T211 ✅ `rerank_hits` F40; T218 ✅ dual floor (do **not** retune); T260 ✅ stub skip
- **Blocks / feeds:** Daily `recall` / `search` / `--semantic` lexical fallback / `sync query` vault half. Preflight Index remains **T286**. Neighbors **T317**.
- **Absorbs:** Audit ranking gap; opportunity (a) prefer `DECISION:` / `CONSTRAINT:` / structured synth; T285 closeout “live vault still dumps” **reopened** (PATH **is** 0.1.3 / T285 and still dump-first on `graph backend`)
- **Not absorbed (DoD):** T315 summary 0/0/0; T313 ledger pane; T317 RECALLS spam; T218 floors / ANN; raise `candidate_depth`; clap 5; T263 H2; T240 F2; FTS title/body schema split (T285 declined)
- **Research date:** 2026-08-27. SQLite FTS5 still one-column `bm25` (https://www.sqlite.org/fts5.html) — no title weight without a schema split. clap lock **4.6.1**. Snapshot — re-verify at execute.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install`. Do **not** pin production decisions to the live vault as implement. Do **not** rewrite `.env`. Do **not** grow hotspot `project.rs` / `sync.rs` / CLI `preflight.rs`. Touch `ranking.rs` + `session_chrome.rs` (#6) only as the full plan names. Do **not** print or commit `AI_BRAINS_KEY`. Frozen until full plan: `candidate_depth`, T218 floors, `KIND_*` unless the plan explicitly unfreezes.

---

## 1. Objective

1. **Pins and structured synth beat raw review dumps** for topical queries (`graph backend`, `GPU driver fix`) when a matching `DECISION:` / `CONSTRAINT:` / `INVARIANT:` pin or nightly synth exists in-scope.
2. **T285 is not enough on this vault.** Envelope + closed-list chrome + KIND_DECISION **+2** still lose to long BM25 dumps that are **not** in the chrome prefix list (live #1 was the audit dump itself, score **−4.06**; #3 `# Review of Track 253`).
3. **Keep dumps recallable.** Do not hard-exclude transcripts. Prefer-fill + detector extension + dump-length / review-heading penalty are the expected levers (full plan decides).
4. **North star.** Capture independence: ranking/retrieval only. No new events. No hidden CoT. Agents can `recall "what did we decide"` and get the pin.

---

## 2. Live baseline (mint 2026-08-27)

| Signal | Observation |
|--------|-------------|
| HEAD | `a1d4081` T311 `#229`. Product `src/` has T285. |
| PATH | `0.1.3` graph-on; mtime **05:52** (pre-T311 merge). Ranking hole is **source + PATH**. |
| `recall "graph backend" --no-bridge --limit 3` | #1 audit dump −4.06; #2 T309 `## Objective`; #3 Track 253 review. |
| `KIND_*` | `ranking.rs`: CONSTRAINT 4.0, DECISION **2.0**, chrome −16. Unclassified dumps skip −16. |
| Hotspots | `session_chrome.rs` **#6**. Do not grow `project.rs` #1. |
| last-PR `#229` Cursor | **N/A empty** |

---

## 3. Frozen until `/plan-track T312`

- **F0** plan-only until go.
- Do not bump `KIND_*` / `SESSION_CHROME_PENALTY` in the placeholder.
- Do not raise `candidate_depth` (T261).
- Do not retune T218 semantic floors.

---

## 6. Non-goals

Hard-delete dumps from FTS. Pin→Approved (H2). New FTS columns. `--semantic` HTTP in hermetic tests.

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| T285 live dumps residual | **Absorb** this track |
| T311 R1–R7 | **Not this DoD** (T322–T324 / declined) |
| T307 Blocked | **Not stolen** |
| last-PR `#229` Cursor | **N/A empty** |

---

## 12. Touch map (sketch)

`crates/ai-brains-retrieval/src/ranking.rs`, `session_chrome.rs`, recall pipeline tests. Not `sync.rs` (vault arm follows automatically).
