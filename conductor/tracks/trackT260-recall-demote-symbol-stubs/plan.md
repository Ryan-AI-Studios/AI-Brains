# T260 Plan — Demote code-symbol stubs

**Status:** **Pending** (Planned + fold-in; F0 until go)
**Spec:** [spec.md](./spec.md) F0–F19 / AC1–AC17 + §13 fold-in
**Category:** FEATURE / UX / RETRIEVAL
**Ledger TX (planning):** `0111473b-5c25-4322-87b4-3328e700f1f7` (DOCS)
**Ledger TX (fold-in):** `054e55b2-0ddf-4474-b714-e05923bca846` (DOCS)
**Ledger TX (implement):** start **FEATURE** on **go** only

---

## AI fold-in (2026-08-17) — `agy-review.md` + `opencode-review.md`

No Blockers. OpenCode **M1** folded hard (GLOB ⊆ detector). Dedupe-after-rerank and composite-space penalty locked. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F19 / AC16 / AC17:** SQL exclude is `GLOB 'Kind * (*:[0-9]*)'` (+ `ASSISTANT:`), not `LIKE`.
2. **F3 / §5.3:** content-dedupe **after** `rerank_hits`.
3. **F9:** `SYMBOL_PENALTY = 16.0` is composite/effective units; Phase 0 re-verify `--symbols --semantic`.
4. **F4:** `[symbol]` is badge chrome, not inside the 500-char slice.
5. **§2.1:** plan dogfood `5119517` vs plan commit `1855b5b`; live Scope may already be `3581317d`.

---

## Preflight (plan time — 2026-08-17)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `5119517`. Plan commit `1855b5b`. Fold-in docs on that product src. |
| T260 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` (2026-08-17 18:20). T257 on PATH. Default recall still shows **Struct Project** / **Module project**. `--global` **five** `Module sqlite_backend` (−19.296, distinct ids). Semantic F11 → capture stubs. |
| Source debug | `target\debug\ai-brains.exe` (2026-08-17 07:37) — older than PATH. Dogfood used PATH. |
| Detector baseline | `symbol_content` = `{kind} {qualified} ({path}:{line})`. `memory_projection` has **no** `source_tag`. Latest migration **0028**. |
| Pipeline | `candidate_depth(5)=15` → blend → `rerank_hits` → truncate. No `--symbols`. `forget` uses unfiltered `lexical_search`. |
| Construction | `RecallOptions` literals: CLI, `sync.rs` ×2, `ai-brainsd` `lib.rs:271`. T70 test `..Default` asserts stub recallable. |
| clap / rusqlite / serde_json | lock clap **4.6.1** / builder **4.6.0** / crates.io **4.6.6**; rusqlite **0.39.0**; serde_json lock **1.0.150** / crates.io **1.0.151**. rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute. |
| Last PR Cursor | #173 and #174 comments/reviews/inline **0**. Dependabot only. **N/A.** |
| `deferred.md` | Full scan. Overlap: audit T260 **absorb**; T259 `--global` ranking half **absorb**; leftover-project / preflight **T264**; T261–T271 / T255 / T240 F2 **decline**. |
| ai-brains | `preflight --summary` ok (wrong Scope — T258 out of band). Recall: T243 alias; T259 leftover stays; no exclude-stubs pin. |
| ledgerful | doctor ready (hygiene warns). 0 pending at start. Hotspot **#1** `project.rs` — do not touch. `forget.rs` #3 — do not filter. New file `symbol_stub.rs`. |
| Research | FTS5 BM25 short-doc vs `avgdl` (sqlite.org + ParadeDB/ES). Azure scoring **profiles** / type filter — we filter by content format. RRF needs a clean lexical arm. |
| `ISSUES.md` | **Does not exist** |
| Live `.env` / leftover / nightly | **Not written** / **not rebound** / **not run** this pass. |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Symbol stubs beat decisions | T256–T271 audit T260 | **DoD** F1–F9 / AC1–AC15 |
| `--global` five identical Modules | same + live dogfood | **DoD** F3 / AC6 |
| T218 F11 shows stubs as the answer | live `--semantic` 3581317d | **DoD** F4 / AC8 |
| T259 `--global` leftover-first (ranking) | T259 closeout | **Partial absorb** F3; project exclusion **T264** |
| T70 recallable test | `symbol_bridge.rs:1079` | **F13** / AC10 — opt in `--symbols` |
| T261 empty latency | deferred | **Not absorbed** F16 |
| T264 preflight / leftover project | deferred | **Not absorbed** F16 |
| T211 F25 / T218 ANN | soft | **Not absorbed** |
| T240 F2 / T255 declines | standing | **Stay closed** F17 |
| last-PR Cursor | #173 | **N/A** — no leftover to mint |
| `source_tag` column | honesty | **Soft** §11 |

---

## Phase 0 — on go (re-verify)

- [ ] Re-read `symbol_content` (`symbol_bridge.rs` ~595) and a live stub first line. Confirm format still `{kind} {qualified} ({path}:{line})` (or note drift and fix F1).
- [ ] Re-read `recall_full` blend order and `candidate_depth`. Confirm still 15 at limit 5; filter must be SQL+memory (F7) not demote-only.
- [ ] Re-read `RecallOptions` + every explicit literal (CLI, `sync.rs` ×2, `ai-brainsd`). Confirm T70 test still uses `Default`.
- [ ] Re-read `lexical_search` / `forget.rs` / `substring_fallback` — still unfiltered default; F19 GLOB attaches only when `recall_full` asks.
- [ ] Confirm SQLite `GLOB` + `[0-9]` still the exclude (F19); do **not** ship `LIKE` or `PRAGMA case_sensitive_like`.
- [ ] Classify-only dogfood: `recall "what is this project" --no-bridge`; `--global "graph backend sqlite"`; `--semantic` capture-independence on path-owner. Confirm hole or note drift.
- [ ] `--symbols --semantic` mix: if a hermetic decision+stub pair inverts, scale `SYMBOL_PENALTY` per `ScoreKind` (F9).
- [ ] Re-check lock clap + crates.io: still no clap 5 (or this track is not that bump).
- [ ] Rescan **entire** `conductor/deferred.md` for new open ranking / symbol-stub rows.
- [ ] Last merged PR + open HEAD PR Cursor comments. Mint placeholder if a leftover fits nowhere.
- [ ] `ledgerful ledger start T260-recall-demote-symbol-stubs --category FEATURE`

---

## Phase 1 — Red (failing tests first)

- [ ] Add retrieval units in `symbol_stub.rs` (or `ranking.rs` tests): AC1/AC2 detector names in spec §7.
- [ ] Add `rerank_hits__included_symbol_below_decision__ac7` and `dedupe_symbol_stubs__identical_content_distinct_ids__one`.
- [ ] Add `recall_full__default_excludes_symbol_stub__ac3` + `__symbols_includes_stub__ac4` + `__duplicate_symbol_content__deduped__ac6` + `__kind_prefix_non_locator__survives_default__ac16` + `__lowercase_module_locator__survives_default__ac17` (temp vault; pin DECISION + T70-format stub; no HTTP).
- [ ] Add `lexical_search__default_still_returns_symbol__ac9`.
- [ ] Add CLI hermetic `crates/ai-brains-cli/tests/recall_symbol_demote.rs` (tempdir + existing pin/init helpers; `TempEnv` + `#[serial(env)]` if overlapping keys).
- [ ] Confirm Phase 1 fails for the right reason (stubs still win / flag missing). Commit allowed (red).

---

## Phase 2 — Green (detector + filter + flag)

- [ ] New `crates/ai-brains-retrieval/src/symbol_stub.rs`: `SYMBOL_KINDS`, `is_symbol_stub_content` (fast `ends_with(')')`), **GLOB** `symbol_stub_sql_exclusion`, `dedupe_symbol_stubs`.
- [ ] `ranking.rs`: `SYMBOL_PENALTY = 16.0` composite-space; apply when stub (F9). No second sort.
- [ ] `RecallOptions.include_symbols` default false. Update every literal (F12).
- [ ] `recall_full`: GLOB exclude + retain before RRF + after graph; **`rerank_hits` then F3 dedupe**; truncate.
- [ ] Lexical/semantic/**substring**: apply GLOB helper **only** when caller asks (`include_symbols == false` from `recall_full`). Default `lexical_search` unfiltered (F10).
- [ ] CLI: `--symbols` on Recall; plumb to options; pretty `[symbol]` (F4).
- [ ] Flip T70 test to `include_symbols: true` (F13 / AC10).
- [ ] `ai-brainsd` + `sync.rs` ×2 compile with `include_symbols: false`.
- [ ] Targeted: `cargo nextest run -p ai-brains-retrieval -p ai-brains-cli --lib --bins` plus the new test file; `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings`.
- [ ] Commit allowed (green).

---

## Phase 3 — Docs + honesty

- [ ] CAPABILITIES Recall table: default exclude; `--symbols` mix; `[symbol]` pretty; no DTO (AC13).
- [ ] CHANGELOG minor.
- [ ] conductor row stays **Pending** until implement-track closeout; review.md on execute only.

---

## Phase 4 — Execute closeout (implement-track, not this pass)

- [ ] Manual AC15 classify-only (no live mutate).
- [ ] Full gate: `cargo fmt --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run --workspace` ; `cargo deny check` ; `cargo audit` ; `ledgerful verify --scope full`
- [ ] FEATURE TX commit; pin; review.md; FEATURE + codex-review; publish (PR + wait GHA + squash-merge + prune). **Never** `git push origin main`.

---

## Definition of done

- [ ] AC1–AC17 checkable and green (AC15 recorded with exact commands)
- [ ] F0–F19 honored; T240 F2 / T255 declines untouched
- [ ] `forget --match` still finds stubs (AC9)
- [ ] No `source_tag` migration; no clap 5; no new crates; no live `.env`
- [ ] Medium+ review findings not silently dropped
- [ ] CAPABILITIES + CHANGELOG updated
- [ ] Ledger FEATURE TX committed; 0 pending 0 drift at close

---

## Stop-before (execute)

- Live `.env` rewrite; leftover `rebind-path --write`; `cargo install`; `nightly` without `--status`; mutating scheduled tasks
- Reopening T240 F2 / T255 / T218 floors / T211 F40
- Push to `main` / force-push
- Scope exceeds this track (T261–T271, projection column, ANN)
