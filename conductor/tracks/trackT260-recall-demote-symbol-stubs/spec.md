# T260 — Recall: demote code-symbol stubs

- **Track ID:** T260-RecallDemoteSymbolStubs
- **Status:** **Completed** (2026-08-17)
- **Category:** FEATURE / UX / RETRIEVAL
- **Owner:** —
- **Source:** Audit 2026-08-16 — recall default **5/4**; `--semantic` **6/6**; `--global` **3/3**; real-project semantic **4/3**; opportunity “demote symbol stubs”
- **Depends on:** T70 / T191 / T233 symbol ingest (`ledgerful:symbol` + `symbol_content` format); T211 `rerank_hits` (F40); T215 ScoreKind / RRF; T217 FTS rescue; T218 dual floor + F11 honesty; T243 `search` alias
- **Blocks / feeds:** Daily vault-first `recall` / `search` answers decisions and session memory. `--global` leftover-project isolation stays **T264**. Empty-query latency stays **T261**. Graph live projection stays **T262**.
- **Absorbs:** `Module sqlite_backend` / `Struct Project` / `Function capture_metadata` beating DECISION pins; “what is this project” returning T70 stubs; `--global` five identical Module rows; T218 F11 lexical fallback implying stubs *are* the answer; T259 closeout “`--global` leftover-first” **ranking half** (symbol monopoly + identical-content dupes)
- **Not absorbed:** Empty-query latency (T261); leftover-project exclusion from `--global` (T264); preflight blender (T264); governed empty store (T263); format maze (T266); T211 F25 vault↔ledger RRF blend; T218 F27 ANN; clap 5 / new crates; `source_tag` projection column (soft)
- **Research date:** 2026-08-17 (plan dogfood HEAD `5119517`; plan commit `1855b5b`; fold-in against current `main`)
- **AI fold-in:** 2026-08-17 `agy-review.md` + `opencode-review.md` (no grok/claude/codex-plan). No Blockers. **Agree hard:** OpenCode **M1** SQL `LIKE` is a looser outer gate than the detector — use **GLOB** (`[0-9]`, case-sensitive); detector is SoT. **Agree:** OpenCode **m1** dedupe **after** `rerank_hits`; OpenCode **m2** penalty is composite-space (re-verify semantic arm); Agy-m1 HEAD note; Agy-O1 `ends_with(')')` fast path; OpenCode **O2** `[symbol]` is chrome not inside the 500-char slice. **Already covered:** Agy-m2 substring SQL (F7). **Already covered:** OpenCode O1 live Scope rebind (Phase 0). Disposition **§13**.
- **Ledger:** planning DOCS TX `0111473b-5c25-4322-87b4-3328e700f1f7`. Fold-in DOCS TX `054e55b2-0ddf-4474-b714-e05923bca846`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** delete ingested symbols. Do **not** add a store migration / `source_tag` column. Do **not** change `forget --match`. Do **not** reopen T211 F40, T218 floors, T240 F2, T255 declines. Do **not** `cargo install`, write live `.env`, or bump clap.

---

## 1. Objective

Vault-first `recall` / `search` must answer **decisions, constraints, and session memory** by default.

T70 code-symbol stubs (`Module` / `Struct` / `Function` / `Enum` / … one-liners from nightly Ledgerful ingest) stay in the vault and stay recallable behind explicit **`--symbols`**. They must not occupy the default top-N, must not fill `candidate_depth` (15 at `--limit 5`), and must not appear as the T218 F11 “showing lexical” answer when cosine is below floor.

This advances the north star: capture independence is useless if the thing an agent recalls first is `Struct Project` instead of the DECISION pin. Ranking only. No models on the default FTS path. No new events. No forgotten symbols.

---

## 2. Live baseline (2026-08-17)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | **Plan dogfood:** `5119517` T257 `#173`. **Plan commit:** `1855b5b` (cherry-pick onto `#174` `45eff95`). **This fold-in:** same product `src/` as `1855b5b` (docs-only). Re-verify SHA at execute. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` (mtime **2026-08-17 18:20**, 24 848 896 bytes). T257 is on PATH. **Do not `cargo install` this pass.** |
| Source debug | `target\debug\ai-brains.exe` (mtime **2026-08-17 07:37**). Older than PATH; plan dogfood used **PATH**. |
| `preflight --summary` | **Plan:** Scope `test-alias` (`441837f6`); 571 pinned. **Fold-in live:** path owner `3581317d` (`C:\dev\ai-brains`); **2854** pinned. T258 rebind happened out of band after the plan. Phase 0 re-dogfood. |
| `recall "what is this project" --no-bridge --limit 5` | test-alias: T257 DECISION/CONSTRAINT; T258 DECISION; **`Struct Project (crates/ai-brains-core/src/project.rs:6)`**; test USER pin; **`Module project (…/commands/mod.rs:14)`**. Hole **still live**. |
| `recall "graph backend sqlite" --global --no-bridge --limit 5` | **Five identical** `Module sqlite_backend (crates/ai-brains-graph/src/lib.rs:7)` score **−19.296**, **five different `memory_id`s** (v5 = `project_id` + qualified_name). F3 fail. |
| `recall "what is the capture independence rule" --semantic --project-id 3581317d --no-bridge` | `Embedding: ok (no semantic hits above threshold; showing lexical)` then chat crumb + **`Function capture_metadata`**, **`Struct ValidationError`**, **`Struct VerificationGateRejection`**, **`Enum CaptureError`**. T218 F11 fired; stubs *are* the shown answer. |
| Last GitHub PR | Plan reviewed [#173](https://github.com/Ryan-AI-Studios/AI-Brains/pull/173). Fold-in last merged is [#174](https://github.com/Ryan-AI-Studios/AI-Brains/pull/174) implement-track chore. Both: comments / reviews / inline **empty**. Open PRs Dependabot only. **last-PR Cursor: N/A.** |
| Ledgerful | `doctor` ready (legacy `.changeguard` / sig-pin / timings / :8081/:8083 unreachable). 0 pending 0 drift at plan start. Hotspot **#1** `project.rs` (3.922) — **do not touch**. `forget.rs` #3 — **do not add symbol filter there**. |
| ai-brains recall | T243 search=recall; T259 leftover memories stay; T260/T264 own `--global` ranking/isolation. No prior “exclude symbol stubs” pin. |

### 2.2 Why this still matters

| Residual | Why it is a product hole / why decline |
|----------|----------------------------------------|
| Short T70 stubs win BM25 | FTS5 BM25 length-normalizes against `avgdl`. Nightly ingest writes thousands of 1-line token-dense stubs (`Module sqlite_backend (path:line)`). That **lowers avgdl** and boosts short docs. Live `--global` BM25 **−19.3** vs decision **−6**. Azure scoring profiles / type filters are the industry answer — we do not have fields. **DoD: exclude from the default candidate set.** |
| Demote-only is not enough | `candidate_depth(5) = 15`. If those 15 MATCH rows are all stubs, `rerank_hits` + `SYMBOL_PENALTY` still returns stubs. **DoD: SQL exclude on lexical + semantic + substring when `include_symbols == false`, plus in-memory retain.** |
| Five identical Module rows | Same `symbol_content`, different `memory_id` (one ingest per project_id). Dedup-by-id misses them. **DoD: dedupe by exact `content` when symbols are present.** |
| T218 F11 + stubs | Honesty line is correct (“showing lexical”) but the lexical *is* stubs. **DoD: default lexical after exclude must not be a stub; remaining empty → T207 hint (F11 does not fire).** |
| `--global` leftover-first | Leftover `7d97a456` still holds historical pins. Symbol monopoly is **this** track. Excluding that project from `--global` is **T264**. |
| `source_tag` not on projection | `memory_projection` has no `source_tag` (latest migration **0028**). Event payload has `ledgerful:symbol` / `changeguard:symbol`. A new column + replay is a store track. **Decline as DoD; detect the live `symbol_content` format.** |
| T70 recallability test | `symbol_ingestion_is_idempotent_and_recallable` uses `RecallOptions { ..Default::default() }` and asserts the Function stub is returned. Default-exclude **must** flip that test to `include_symbols: true`. Ingest idempotency stays. |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Ingest format | `symbol_bridge.rs` `symbol_content` **:595–600** | `"{kind} {qualified_name} ({file_path}:{line_start})"` e.g. `Function crate::routes::get_user (src/routes/user.rs:42)`. |
| Tags | `SOURCE_TAG_SYMBOL` = `ledgerful:symbol`; legacy `changeguard:symbol`. `is_symbol_source_tag` **private** in CLI. Retrieval must **not** depend on CLI. |
| Kinds seen live / tests | `Module`, `Struct`, `Function`, `Enum`, `Fn` (dedupe unit). Closed list in §5.1. |
| `memory_projection` | No `source_tag`. Columns: content, privacy, status, timestamps, level, session_id, project_id, tx_id, embedding*. |
| Lexical SQL | `lexical.rs` `match_query` **:127–150** | `SELECT … rank, updated_at` + project/session. `ORDER BY rank LIMIT ?`. No type filter. |
| Substring | `lexical.rs` `substring_fallback` **:181+** | LIKE on `content`. Skip if project count > 10_000. **`--global` leftover (18k) already skips.** |
| Semantic SQL | `semantic.rs` **:378** | Same projection columns + embedding BLOB. Shares space with decisions (CAPABILITIES). |
| Blend | `recall.rs` `recall_full` **:234–477** | rescue FTS → substring → optional semantic RRF → bridge → graph → **`rerank_hits` → truncate**. |
| Depth | `hybrid.rs` `candidate_depth` | `limit*3` clamp **15..50**. Default limit 5 → **15**. Do **not** raise this track (T261). |
| Rank | `ranking.rs` `rerank_hits` **:245** | T211 F40 single entry. Kind boosts; plan penalty; ScoreKind. **No symbol class today.** 878 lines — new detector lives in **`symbol_stub.rs`**, not more ranking.rs. |
| `RecallHit` | `recall.rs` **:32–50** | `source` is `fts`/`substring`/`hybrid`/`semantic`/`graph`/`bridge` — **not** `source_tag`. |
| `RecallOptions` | `recall.rs` **:14–29** | `#[derive(Default)]`. Add `include_symbols: bool` (default **false**). |
| Construction sites | CLI `recall.rs`; `sync.rs` **two** literals (ndjson + pretty); `ai-brainsd` `lib.rs:271`; `symbol_bridge` test `..Default`; retrieval tests `..Default`. |
| CLI flags | `main.rs` Recall **:480–526** | `semantic`, `min_score`, `graph_boost`, `no_bridge`, `global`, `session_last`. **No `--symbols`.** `search` is `visible_alias`. |
| T70 test | `symbol_bridge.rs` **:1079–1110** | Default options must still prove ingest+recall **with** `include_symbols: true`. |
| Forget | `forget.rs` uses `lexical_search` directly | **No filter.** Changing lexical default would hide symbols from forget. |
| Contracts | `RecallResult` | Additive `staleness` / `score_kind` / `cosine` only. **No new wire key.** |
| PROTOCOL-COMPAT | recall JSON compact; N−1 ignore unknowns | Stay. |
| `project.rs` | Hotspot #1 | **Do not touch.** |

### 2.4 Research (online — snapshot, re-verify at execute)

| Claim | Source | Application |
|-------|--------|-------------|
| BM25 prefers short docs vs `avgdl` | [SQLite FTS5](https://www.sqlite.org/fts5.html) `rank`/`bm25`; ParadeDB / Elasticsearch write-ups (2025–2026): length term `1 − b + b·\|D\|/avgdl` **boosts** docs shorter than average. Adding thousands of 1-line stubs pulls `avgdl` down and makes stubs even stronger. | Do **not** retune FTS5 `bm25(k1,b)` this track. Exclude the class. |
| Type filter / scoring profile | [Azure AI Search BM25 scoring](https://learn.microsoft.com/en-us/azure/search/index-similarity-and-scoring) (updated 2025-08-27): scoring **profiles** boost by content characteristics; filters are un-scored. | We have no typed field. Closest portable pattern: **query-time filter** on a detector, plus a penalty when the operator opts back in. |
| Hybrid still needs a clean lexical arm | arXiv 2604.01733 (2026-04) + T215/T218 in-repo: RRF helps only if the BM25 list is not 100% junk. | Filter **before** `fuse_local_and_semantic`. |
| clap `--symbols` | clap **4** derive `#[arg(long)] symbols: bool`. Workspace pin **4.5** → lock **4.6.1** / builder **4.6.0**; crates.io **4.6.6**. **No clap 5.** | Snapshot — re-verify at execute. |
| serde_json / rusqlite / chrono | lock **1.0.150** / crates.io **1.0.151**; rusqlite **0.39.0**; chrono lock **0.4.44**. rustc **1.95.0**. Workspace **0.1.1**. | No bumps. |
| SQLite `LIKE` vs `GLOB` | [sqlite.org/lang_expr.html](https://www.sqlite.org/lang_expr.html) §5 (page 2026-08-04): `LIKE` is **ASCII case-insensitive** by default (`'a' LIKE 'A'` is TRUE); `GLOB` is Unix-glob and **case-sensitive**, with `[0-9]` classes. `PRAGMA case_sensitive_like` exists but must not be flipped globally. | **F19:** SQL exclude is `GLOB`, not `LIKE`. |
| N/A | No new ranking crate, no regex, no sqlite-vss / ANN. Detector is prefix + trailing ` (path:digits)` — same style as `classify_pin_kind`. | Zero new crates. |

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. No product commits as “planning”. |
| **F1 — default exclude** | Default `recall` / `search` / `sync query` vault arm / daemon recall **exclude** symbol-stub memories from the candidate set. Detect via **`is_symbol_stub_content`** (§5.1). Do **not** add an event kind. Do **not** add a `source_tag` column this track. |
| **F2 — `--symbols`** | CLI long flag **`--symbols`** (name frozen). Sets `RecallOptions.include_symbols = true`. Restores **mix** (stubs + decisions), not symbols-only. `search` inherits (same clap command). No `--symbols-only` DoD. |
| **F3 — `--global` same rule + dedupe** | `--global` uses the same exclude. Five identical `Module sqlite_backend` rows is a fail. When symbols are present (leak **or** `--symbols`), **dedupe by exact `content` after `rerank_hits`**, keep the **first** row in that sort (effective desc → `updated_at` desc → `memory_id` asc). Do **not** dedupe on raw pre-rerank scores. Different `memory_id`s of the same stub collapse to one. |
| **F4 — F11 honesty** | T218 F11 string stays. After default exclude, F11 lexical hits must **not** be stubs. If exclude empties the lexical arm, F11 does **not** fire (T207 empty hint). With `--symbols`, pretty uses the same chrome slot as `[plan/stale?]` — **`[symbol]` sits outside the 500-char content slice** (`format_pretty_hit_line`: T224 strip `:415`, truncate `:416`, badge before `{content}`). |
| **F5 — capture independence** | Ranking / SQL filter only. Default FTS path needs no model. No hidden CoT. No new events. Nightly ingest unchanged. |
| **F6 — do not delete** | Do not forget, rewrite, or skip nightly symbol ingest. Demote / exclude at query time. |
| **F7 — SQL + memory** | When `!include_symbols`: append the shared **GLOB** exclusion (F19) to lexical `match_query`, semantic SELECT, **and** `substring_fallback` SELECT (Agy-m2 — already named; keep). **Also** `retain` after each arm and after graph. SQL may miss a stub the detector would drop; memory retain catches those. Memory retain **cannot** restore a row SQL already dropped — so SQL must be a **subset** of the detector (F19). |
| **F8 — filter before RRF; one final sort** | Drop stubs from `local_hits` and `semantic_hits` **before** `fuse_local_and_semantic`. After graph, retain again, then **`rerank_hits` only** (T211 F40), **then** F3 content-dedupe. Filter/dedupe are not a second sort. |
| **F9 — `--symbols` penalty** | When `include_symbols`, `effective_score` subtracts **`SYMBOL_PENALTY = 16.0` in composite/effective space** (same units as `KIND_*` / after `ScoreKind` conversion — **not** raw cosine). Calibrated to live BM25 −19.3 vs −6. Phase 0 re-verify a `--symbols --semantic` mix (OpenCode m2). If that hermetic AC7 variant fails, scale the penalty per `ScoreKind` then — do not invent a second constant in planning. Stub-only queries still return stubs. |
| **F10 — forget / lexical default** | `lexical_search` / `substring_fallback` **stay unfiltered** unless the caller passes the SQL helper. `forget --match` must still find a stub. |
| **F11 — no DTO field** | Do **not** add `is_symbol` / `kind` to `RecallResult`. JSON `content` stays raw. Pretty `[symbol]` is display-only. PROTOCOL-COMPAT unchanged. |
| **F12 — construction sites** | Add `include_symbols: bool` to `RecallOptions` (Default **false**). Update every explicit literal: CLI, **both** `sync.rs` sites, `ai-brainsd` `lib.rs:271`. Tests that need stubs set `true`. |
| **F13 — T70 test** | Update `symbol_ingestion_is_idempotent_and_recallable` to `include_symbols: true`. Idempotent ingest asserts stay. |
| **F14 — new module** | Detector + **GLOB** fragment + dedupe live in **`crates/ai-brains-retrieval/src/symbol_stub.rs`**. Do not grow `ranking.rs` (878) or `project.rs`. Do not move `SOURCE_TAG_*` this track. |
| **F15 — pins** | clap 4.x lock **4.6.1** / crates.io **4.6.6**; no clap 5; no new crates; rusqlite **0.39.0**. Snapshot — re-verify at execute. |
| **F16 — decline extras** | `source_tag` migration; leftover-project `--global` drop (T264); empty-query short-circuit (T261); raise `candidate_depth`; BM25 k/b retune; ANN; weighted RRF; `--symbols` on `sync query`; symbols-only mode; clap 5; live `.env`; `cargo install`; global `PRAGMA case_sensitive_like`. |
| **F17 — T240 F2 / T255** | Stay closed. |
| **F18 — PATH-behind** | If PATH is older at execute, tests use hermetic/source bin. Do not `cargo install` as the track. |
| **F19 — GLOB ⊆ detector (OpenCode M1)** | `symbol_stub_sql_exclusion` emits `AND NOT (col GLOB 'Module * (*:[0-9]*)' OR … OR col GLOB 'ASSISTANT: Module * (*:[0-9]*)' OR …)` from the same `SYMBOL_KINDS`. **No `LIKE`.** Detector (`is_symbol_stub_content`) is SoT for retain. GLOB is case-sensitive + requires a digit after the last locator colon + whole-string match ending `)`. Residual: `Module x (path:1junk)` may still GLOB-match; not a real pin class. Do **not** set `case_sensitive_like`. |

---

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | `is_symbol_stub_content` is true for live T70 format: `Module sqlite_backend (crates/ai-brains-graph/src/lib.rs:7)`, `Struct Project (crates/ai-brains-core/src/project.rs:6)`, `Function capture_metadata (crates/ai-brains-capture/src/git_capture.rs:4)`, `Enum CaptureError (…:28)`, `Fn` kind, `ASSISTANT: Module foo (a.rs:1)` after one strip. |
| **AC2** | False for `DECISION: …`, `CONSTRAINT: …`, a decision that *mentions* `Module sqlite_backend` mid-body, chat turns, empty, and `Function` without the trailing ` (path:digits)` locator. |
| **AC3** | Hermetic vault: pin `DECISION: we chose foo for the bar path` + stub `Module foo (src/foo.rs:1)`. `recall "what did we decide about foo" --no-bridge` → DECISION in results; stub **absent**. |
| **AC4** | Same vault + `--symbols` → stub **present**. Pretty line starts with `[symbol]` (human). JSON `content` is the raw stub (no `[symbol]` prefix, no new key). |
| **AC5** | `search "…" --symbols` is accepted (alias). |
| **AC6** | Three hits, identical stub `content`, distinct `memory_id`s, `--symbols --limit 5` → **one** stub row. |
| **AC7** | `recall_full` default: stub not in `hits` even when it is the strongest FTS MATCH. `--symbols`: stub present and `effective_score` below a same-query DECISION (penalty). |
| **AC8** | T218 F11 hermetic: semantic requested, `embedding.status=ok`, zero post-threshold semantic, default exclude → remaining lexical is non-stub **or** empty (no F11+stub). |
| **AC9** | `forget --match` (or `lexical_search` default) still returns a hermetic stub. |
| **AC10** | `symbol_ingestion_is_idempotent_and_recallable` green with `include_symbols: true`; still fails (red) if that flag is omitted — documents F1. |
| **AC11** | Existing T211 / T215 / T218 ranking + F11 tests stay green. Dual floors unchanged. |
| **AC12** | `sync.rs` both `RecallOptions` literals compile with `include_symbols: false`. Vault pretty/ndjson default-exclude (no new `sync` flag). |
| **AC13** | CAPABILITIES Recall table: default excludes T70 stubs; `--symbols` restores mix; `[symbol]` pretty; no DTO field. CHANGELOG minor. |
| **AC14** | No `unwrap`/`expect`/`panic` in production. No live vault mutate. No `.env` write. |
| **AC15** | Manual (execute, classify-only): PATH or source `recall "what is this project" --no-bridge` has no `Struct Project` / `Module project` in top 5; `--global "graph backend sqlite"` is not five identical Modules; `--symbols` can still surface a Module. |
| **AC16** | Hermetic `recall_full` default: pin `Module foo (draft: notes)` (kind prefix, **non-digit** locator) + a matching DECISION. Default recall **keeps** that row (SQL GLOB must not drop it; LIKE `'Module % (%:%'` would have). |
| **AC17** | Hermetic `recall_full` default: pin `module foo (src/foo.rs:1)` (lowercase kind + real locator). Default recall **keeps** that row (SQL `LIKE` would have dropped it; GLOB must not). |

---

## 5. Design notes

### 5.1 Detector (`is_symbol_stub_content`)

Source of truth for “is this a T70 stub?” (F19).

1. Fast reject if `!trim.ends_with(')')` (Agy-O1). Then `strip_assistant_prefix` once (T211 helper).
2. `trim`. First token (split on first ASCII space) ∈ closed **`SYMBOL_KINDS`** (case-sensitive):

   `Module`, `Struct`, `Function`, `Fn`, `Enum`, `Trait`, `Type`, `Const`, `Static`, `Impl`, `Macro`, `Field`, `Variant`, `Union`, `Method`, `Interface`, `Class`, `Unknown`

   (`Unknown` is what `symbol_bridge` writes when JSON `kind` is missing.)
3. Remainder contains ` (` and the suffix after the **last** ` (` matches `*:digits)` where `digits` is 1+ ASCII digits (the T70 locator). No `regex` crate.

False-positive guard: DECISION/CONSTRAINT first tokens are not in the list. A pin that *quotes* a stub on a later line stays a decision.

### 5.2 SQL fragment (F19)

`symbol_stub_sql_exclusion(column)` — bind-free list from the same `SYMBOL_KINDS`:

```sql
AND NOT (
  col GLOB 'Module * (*:[0-9]*)' OR
  col GLOB 'ASSISTANT: Module * (*:[0-9]*)' OR
  -- …every kind, both bare and ASSISTANT: …
)
```

**No `LIKE`.** GLOB is whole-string, case-sensitive, and requires a digit after the locator colon. Detector remains SoT on the retain path. Apply to `match_query`, semantic SELECT, and **`substring_fallback`** when `!include_symbols`.

### 5.3 Pipeline

```
lexical (GLOB exclude if !include) → retain
empty → substring (GLOB exclude if !include) → retain
semantic? → GLOB exclude if !include → retain → fuse (F8)
bridge (retain drops stub-shaped Insight)
graph → retain
rerank_hits (F9 penalty when stub)
if include_symbols: dedupe_by_content after sort (F3; stubs only)
truncate(limit)
```

### 5.4 Pretty

`--symbols` + stub → `[symbol]` in the **badge chrome** (same place as `[plan/stale?]`), then T224-stripped / 500-char preview. Badge is **not** counted inside the 500-char slice (OpenCode O2). Default path never prints `[symbol]` because stubs are absent.

### 5.5 Why not a projection column

Correct long-term marker is `source_tag`. It is not on `memory_projection` today. Adding it is a migration + projector + rebuild + T167 preserve path. Content format is what nightly **already writes** and what the audit scored. Soft residual if the heuristic leaks.

---

## 6. Non-goals

- Deleting or re-ingesting symbols; changing nightly `ledgerful symbols` flags
- `source_tag` on `memory_projection`
- Dropping leftover `7d97a456` from `--global` (T264)
- Empty-query short-circuit (T261)
- Raising `candidate_depth` / BM25 parameter retune / ANN / weighted RRF
- `--symbols` on `sync query`; symbols-only mode
- New `RecallResult` keys; clap 5; new crates
- Silent Scope / `.env` rewrite; live leftover rebind; `cargo install`
- Reopening T240 F2, T255 declines, T218 floors, T211 F40

---

## 7. Verification plan (TDD — red names first)

Retrieval units (`symbol_stub.rs` / `ranking.rs`):

- `is_symbol_stub_content__t70_module_format__true`
- `is_symbol_stub_content__assistant_prefix__true`
- `is_symbol_stub_content__fn_kind__true`
- `is_symbol_stub_content__decision_quoting_module__false`
- `is_symbol_stub_content__function_without_locator__false`
- `rerank_hits__included_symbol_below_decision__ac7`
- `dedupe_symbol_stubs__identical_content_distinct_ids__one`

Retrieval integration (temp vault, no HTTP):

- `recall_full__default_excludes_symbol_stub__ac3`
- `recall_full__symbols_includes_stub__ac4`
- `recall_full__duplicate_symbol_content__deduped__ac6`
- `recall_full__kind_prefix_non_locator__survives_default__ac16`
- `recall_full__lowercase_module_locator__survives_default__ac17`
- `lexical_search__default_still_returns_symbol__ac9`

CLI hermetic (`tempdir` + `init_vault` / pin helpers; `#[serial(env)]` + `TempEnv` if keys overlap):

- `recall__hermetic_decision_vs_stub__default_excludes_stub`
- `recall__hermetic_symbols_flag__returns_stub_and_pretty_marker`
- `search__symbols_flag__accepted`

Update in place (not a new red unless it fails): `symbol_ingestion_is_idempotent_and_recallable` → `include_symbols: true`.

Existing T211/T215/T218 suites must stay green.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Heuristic false-positive on a real pin | Closed kind list + locator suffix; AC2; do not match mid-body. |
| Heuristic false-negative (`Unknown` / new tree-sitter kind) | Include `Unknown`; execute re-reads a sample of live stub first lines; soft residual = projection column. |
| `candidate_depth=15` still all-stub if SQL LIKE misses | Dual layer (F7); AC7. |
| T70 test / daemon / sync literals fail compile | F12/F13 listed; `..Default` sites pick up `false`. |
| `--symbols` penalty too weak on −19 BM25 | 16.0 vs live 13-pt gap; AC7 asserts order, not the constant. |
| Forget regression | F10 / AC9; do not change `lexical_search` signature default. |
| Hotspot `project.rs` | Do not touch. |
| Capture independence | Query-time only. |

---

## 9. Deferred absorb / decline

Full `conductor/deferred.md` scan 2026-08-17. `ISSUES.md` does **not** exist.

| Residual | Disposition |
|----------|-------------|
| Recall symbol stubs beat decisions (audit T260 row) | **Absorb** F1–F9 / AC1–AC15 |
| `--global` leftover-first / blender (T259 closeout → T260/T264) | **Partial:** symbol monopoly + identical-content dupes **absorb** (F3). Leftover-**project** exclusion / preflight mix **decline → T264** |
| `recall ""` 5.7 s | **Decline → T261** (latency, not ranking) |
| Graph sparse / 4h pin no node | **Decline → T262** (neighbor stub retain is only the recall_full path) |
| Governed 0 authority | **Decline → T263** |
| `preflight --global` blender | **Decline → T264** |
| Preflight `{text, word_count}` | **Decline → T265** |
| Format maze | **Decline → T266** |
| harness/whoami self-next; list footer leftover-as-AI-Brains | **Decline → T267** |
| `scan-roots` cwd-only | **Decline → T268** |
| Nightly human mixes Router | **Decline → T269** |
| Retention 0 candidates | **Decline → T270** |
| `sync query` ledger pane empty | **Decline → T271** (vault arm inherit exclude via `recall_full`; pane is T271) |
| T211 F25 vault↔ledger RRF blend | **Decline** (soft elsewhere) |
| T215/T218 ANN, weighted RRF, adaptive floor | **Decline** (stay soft) |
| T255 declined bag / T240 F2 | **Stay closed** |
| T257 JSON interleave | **Closed** 2026-08-17 — not ranking |
| T258 `.env` / T259 leftover mutate | **Decline** (out of band) |
| MSI / notarization / R-CI-BRANCH / anyhow allowlist / archive changeguard sweep | **Not related** (one line: not recall ranking) |
| Connector cursors, CE residuals, desktop, sync threat leftovers | **Not related** |
| last-PR Cursor (#173, #174) | **N/A** — both empty comments/reviews/inline; nothing to mint |

---

## 10. Implement order (on go)

1. Phase 0 re-verify live format, `candidate_depth`, construction sites, deferred, last PR.
2. Red: detector + ranking + `recall_full` + CLI hermetics (AC1–AC10 names).
3. Green: `symbol_stub.rs` + **GLOB** helper (F19) + `RecallOptions.include_symbols` + `--symbols` + pretty `[symbol]` chrome + T70 test flip; dedupe after `rerank_hits`.
4. Docs: CAPABILITIES + CHANGELOG.
5. Targeted nextest/clippy on retrieval + cli; then full gate on finalize.
6. FEATURE TX commit; review.md; publish per implement-track (not this skill).

---

## 11. Soft residuals

| Residual | Why not DoD |
|----------|-------------|
| `source_tag` on `memory_projection` + projector | Correct marker; store/replay track |
| `--symbols-only` | Operator has mix; not scored |
| `sync query --symbols` | Human ledger pane; machines use `recall --symbols` |
| Empty-pretty “try `--symbols`” next-step | T207 contract; optional later |
| Raise `candidate_depth` when excluding | SQL exclude should fill 15 with real pins; T261 owns latency |
| Leftover project out of `--global` | **T264** |
| Live `.env` rebind / leftover path mutate / PATH reinstall | Operator out of band |
| Daemon HTTP recall `--symbols` wire | No DTO; default exclude is the product |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-retrieval/src/symbol_stub.rs` | **New.** Detector, kinds, SQL fragment, dedupe. |
| `crates/ai-brains-retrieval/src/lib.rs` | `mod symbol_stub`; export helpers. |
| `crates/ai-brains-retrieval/src/ranking.rs` | `SYMBOL_PENALTY`; apply in `effective_score` / `rerank_hits` when stub. |
| `crates/ai-brains-retrieval/src/recall.rs` | `RecallOptions.include_symbols`; filter before RRF + after graph; dedupe. |
| `crates/ai-brains-retrieval/src/lexical.rs` | Optional SQL exclude from caller / helper — **default path unfiltered**. |
| `crates/ai-brains-retrieval/src/semantic.rs` | Optional SQL exclude when `!include_symbols`. |
| `crates/ai-brains-retrieval/tests/*` | New ranking/recall_full cases. |
| `crates/ai-brains-cli/src/main.rs` | `--symbols` on Recall. |
| `crates/ai-brains-cli/src/commands/recall.rs` | Plumb flag; pretty `[symbol]`. |
| `crates/ai-brains-cli/src/commands/sync.rs` | Two literals + `include_symbols: false`. |
| `crates/ai-brains-cli/src/commands/symbol_bridge.rs` | T70 test sets `include_symbols: true`. |
| `crates/ai-brainsd/src/lib.rs` | Literal + `include_symbols: false`. |
| `crates/ai-brains-cli/tests/recall_symbol_demote.rs` | **New** hermetic CLI ACs. |
| `Docs/CAPABILITIES.md` | Recall table row. |
| `CHANGELOG.md` | Minor. |
| `conductor/conductor.md` / `deferred.md` / README-T256–T271 | Planned pointer. |

**Do not touch:** `project.rs`, `forget.rs` match SQL, migrations, contracts DTO, T218 floor constants, live `.env`.

---

## 13. AI fold-in disposition (2026-08-17)

Sources: `agy-review.md` (Antigravity) and `opencode-review.md` (OpenCode). No `grok-review.md` / `claude-review.md` / `codex-plan-review.md`. No Blockers. Re-verified at fold-in: `symbol_content` still `{kind} {qualified} ({path}:{line})` at `symbol_bridge.rs:595`; `substring_fallback` still unfiltered LIKE at `lexical.rs:208`; `format_pretty_hit_line` strip `:415` / truncate `:416` / badge before `{content}`; SQLite `LIKE` ASCII case-insensitive + `GLOB` case-sensitive ([lang_expr.html](https://www.sqlite.org/lang_expr.html) §5). Review re-confirmed deferred + last-PR Cursor **#173 and #174** empty — **no leftover to mint**. Product `src/` unchanged this pass.

### Antigravity

| ID | Verdict | Action |
|----|---------|--------|
| **m1** spec HEAD `5119517` vs `1855b5b` | **Agree** | §2.1: plan-dogfood SHA vs plan-commit SHA. Product src unchanged. |
| **m2** substring SQL exclude seam | **Already covered** | F7 already names substring SELECT. **Tightened:** F7/F19/§5.2/`substring_fallback` get the same GLOB helper. |
| **O1** `ends_with(')')` fast path | **Agree** | §5.1 step 1. |

### OpenCode

| ID | Verdict | Action |
|----|---------|--------|
| **M1** `LIKE` is a looser outer gate than the detector | **Agree hard** | **F19** + §5.2 GLOB `[0-9]`; detector SoT; **AC16** / **AC17**. No `PRAGMA case_sensitive_like`. |
| **m1** dedupe-before-rerank vs “best `effective_score`” | **Agree** | **F3** / §5.3: dedupe **after** `rerank_hits`; keep first in sort. |
| **m2** `SYMBOL_PENALTY` vs cosine/`RELEVANCE_SCALE` | **Partial** | **F9:** penalty is composite-space. Phase 0 re-verify `--symbols --semantic`. No second constant unless that AC7 variant fails. |
| **O1** live Scope now `3581317d` / 2854 pins | **Already covered** | Phase 0. **Noted** in §2.1 fold-in live row. |
| **O2** `[symbol]` vs 500-char truncate | **Agree** | **F4** / §5.4: badge chrome, not inside the slice. |

### Pins locked by fold-in

1. **F19 / AC16 / AC17:** SQL exclude is **GLOB** ⊆ detector; no `LIKE`.
2. **F3 / §5.3:** content-dedupe **after** `rerank_hits`.
3. **F9:** `16.0` is composite/effective units; semantic-arm re-verify at execute.
4. **F4 / §5.4:** `[symbol]` is badge chrome (with `[plan/stale?]`).
5. **§2.1:** dogfood `5119517` / plan `1855b5b`; live Scope may be path-owner after T258 out-of-band rebind.
6. **F0** until go. No product crate edits this pass.

---

**Planning + fold-in 2026-08-17.** Still **plan-only until go**.
