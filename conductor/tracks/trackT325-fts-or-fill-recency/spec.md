# T325 — FTS authority-OR fill must recency-retry (T312 F8 leftover)

- **Track ID:** T325-FtsOrFillRecency
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T325`.
- **Category:** BUGFIX / RETRIEVAL
- **Owner:** Grok
- **Source:** Last-PR Cursor Bugbot on [#230](https://github.com/Ryan-AI-Studios/AI-Brains/pull/230) (T312, `mergedAt` **2026-08-28T02:35:31Z**). Medium: F8 authority-OR path runs only BM25 `Prefer` MATCH + in-memory retain; when that retain is empty it does **not** recency-retry, unlike the AND pass.
- **Depends on:** T312 ✅ F8 authority-OR fill (`lexical.rs` `match_query`)
- **Blocks / feeds:** Two-token AND-miss pins whose TAGS/OR hits lose the Prefer `LIMIT` window to newer `TAGS:` rows.
- **Absorbs:** `#230` Cursor comment `pulls/comments/3877408710` (still true on HEAD `44520d8`)
- **Not absorbed (DoD):** T315 summary next-step; T217 ≥3 gate; T218 floors; clap 5
- **Research date:** 2026-08-28. Live `lexical.rs`: AND empty → `AuthorityFilter::PreferRecency` (`:197–213`); F8 OR fill → `AuthorityFilter::Prefer` only (`:231–250`). `PreferRecency` is `ORDER BY mp.updated_at DESC` (`:390–392`); Prefer is `ORDER BY rank`. Snapshot — re-verify at execute.
- **Ledger:** minted with T315 planning DOCS TX `ca5b1614-6849-416d-ad27-1d44a23198d7`. Implement **BUGFIX** TX on go.
- **Isolation:** Do **not** implement until go / `/plan-track T325` then **go**. Do **not** steal into T315. Do **not** `cargo install`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **OR-fill has the same recency retry as AND.** When F8 Prefer-OR retain is empty, one `PreferRecency` MATCH on the **OR** expr (same bind/`?` rules as T274 F35) before pass-2.
2. **Do not weaken F8.** Still fire only when AND+recency retain is empty; ≥2 contentful tokens; authority-only retain.
3. **North star.** Capture independence: FTS two-pass only. No new events.

---

## 2. Live baseline (mint 2026-08-28)

| Signal | Observation |
|--------|-------------|
| HEAD | `44520d8` T312 `#230` |
| AND path | Prefer → in-memory retain → if empty **PreferRecency** (`:197–213`) |
| F8 OR path | Prefer-OR → retain; **no** PreferRecency (`:231–250`) |
| Cursor | Medium, still true — TAGS rows matching either token can fill `LIMIT` and drop the OR-only pin |
| T315 | Preflight summary — **not** this hole |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- T312 F8/F40/F41/F42 needle grammar stays until `/plan-track T325` says otherwise.

---

## 6. Non-goals

Changing T217 R0/≥3. Raising `candidate_depth`. T315 summary. KIND bump.

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| `#230` Cursor Bugbot F8 recency | **Absorb** (this placeholder) |
| T315 0/0/0 | **Not stolen** |
| T307 | **Not stolen** |

---

## 12. Touch map (sketch)

`crates/ai-brains-retrieval/src/lexical.rs` `match_query` F8 arm + existing PreferRecency helper. Hermetic in `recall_rank_v3.rs` / lexical units.
