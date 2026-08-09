# T217 — FTS multi-token / natural-phrase rescue

- **Status:** ✅ **Completed** (2026-08-09; PR #110 `1e22e77`)
- **Source:** Audit 2026-08-05 — `recall "what did we decide about forget list"` → empty; short token / two-token hits same vault (quality **4**)
- **Scores:** usefulness 7 · **output quality 4**
- **Category:** FEATURE / UX
- **Owner:** Grok
- **Depends on:** T90 FTS sanitize; T105 substring fallback; T111 empty hint; T112 scope; T140 bridge sanitize
- **Feeds:** T218 (semantic v2 stays separate); T224 (role-prefix display); T231 (unified search UX)
- **Absorbs:** Natural-language empty trap; stopword + progressive/OR rescue SOOT; empty-hint “try fewer keywords”; double-sanitize lexical fix; AI1 M1–M8 / AI2 fold-ins (2026-08-09)

---

## 1. Objective

Stop long natural-language FTS queries from returning **false empty** when shorter tokenizations of the same query would hit. Keep capture independence (no embeddings required on the default path). Preserve BM25 ranking honesty after rescue. **Do not** widen match sets on destructive commands (`forget`).

---

## 2. Live dogfood freeze (2026-08-09)

Same vault (`test-alias`), `--no-bridge`, default lexical path:

| Query | Result |
|-------|--------|
| `what did we decide about forget list` | **empty** + T111 hint |
| `forget list` | **hits** (T216 pins, `source=fts`) |
| `forget` | **hits** |

Root cause: `sanitize_fts_query` emits space-separated quoted tokens; FTS5 treats whitespace as **implicit AND**. All seven tokens must appear in the document. Pins about “forget list” lack stopwords like `what`/`did`/`we`/`about` → MATCH fails. T105 substring does **not** help: it `LIKE`s the **full raw phrase**, which also does not appear consecutively.

**Also intentional scope (M6):** rescue applies to **any** ≥3-token query when R0 is empty — including all-contentful phrases with no stopwords (e.g. `brittle hotspot fix` → R1 skipped if c==tokens → **R2 OR**). Brand is “natural-phrase” but the gate is token-count, not “must contain stopwords.”

---

## 3. Research freeze (2026-08-09)

| Topic | Finding |
|-------|---------|
| SQLite FTS5 docs | Whitespace between phrases = implicit AND; `OR` / `AND` / `NOT` are explicit boolean ops (NOT > AND > OR); quoted strings are phrase literals (safe against operator injection). |
| unicode61 tokenizer | Default on `memory_fts` (migration 0007). Treats `_` as **separator** — so `ai_brains_core` indexes as three tokens, not one. |
| Current sanitizer | `ai_brains_core::sanitize_fts_query` — alphanumeric runs **keeping `_`** → one quoted phrase. Quoted `"ai_brains_core"` becomes adjacency phrase after tokenize — stricter than AND of three. **T217 aligns extract + sanitize to split `_`** (M4). |
| Double sanitize | `recall_full` sanitizes then `lexical_search` sanitizes again. Idempotent for bare AND today; **breaks** under OR. F9/F10/AC10 correct. |
| LIMIT gap | `lexical_search` has **no SQL LIMIT** today; only `ORDER BY rank`. R2 OR can materialize huge sets into rerank + per-hit graph expansion before `truncate(limit)` (M1). |
| forget `--match` | Single hit + `--force` deletes; empty match deletes nothing. Widening via OR is a **new destructive path** (M2). |
| Dep pins | `rusqlite` **0.39**; `clap` **4.5**. crates.io later (0.40.x / 4.6.x) **not** required. **No bumps.** |
| Prior docs | `Docs/FTS5-catch.md` = syntax safety (T90); T217 = **recall quality**. |

---

## 4. Frozen decisions

| ID | Decision |
|----|----------|
| **D1** | Rescue runs **only when** primary FTS AND returns empty **and** extracted token count **≥ 3** **and** `rescue == true`. |
| **D2** | Ladder (stop at first non-empty): **R0** full AND → **R1** AND of **contentful** tokens (only if contentful differs and non-empty) → **R2** **OR** of contentful tokens (token cap **8**) → then **T105** substring on raw query (recall path only, as today). |
| **D3** | Stopword list: **literal fixed set** in §4.1 (deterministic, no stemmer, no data files). Case-insensitive. **Negators excluded** (never drop `not`/`no`/`never`/…). Contentful tokens: stopword-filtered, **min length 2**, **deduped** preserving first-seen order. If 0 contentful, skip R1/R2. |
| **D4** | Do **not** auto-enable semantic on empty (T202 / T218 separate). |
| **D5** | Rescue hits keep `source = "fts"` and FTS5 `rank` / BM25. Optional `tracing::debug!` for stage. **No** new public `source` value. |
| **D6** | SOOT: pure token/match builders in `ai-brains-core`; MATCH execution in `ai-brains-retrieval`. **`lexical_search(..., rescue: bool, limit: usize)`** (or equivalent options struct). Bridge uses sanitized **primary AND** only (one call). |
| **D7** | Empty hint (CLI, non-semantic): when **contentful token count ≥ 1** and **raw token count ≥ 3** and all paths empty, append **“try fewer keywords”** (plain text, no emoji). Reuse **core** `extract_fts_tokens` / contentful helpers — **no** CLI-local token reimplementation (M5). Never suggest fewer keywords for all-stopword queries. |
| **D8** | Capture independence: pure SQLCipher FTS; no models/embeddings/graph required for rescue. |
| **D9** | **`rescue` opt-in (M2):** default **`false`**. **`recall_full` sets `rescue: true`**. **`forget` stays strict R0** (no ladder). Other callers stay false unless explicitly justified. Control-plane / bridge multi-round out of wire. |
| **D10** | No dep bumps; no FTS migration / re-tokenize / porter / trigram / `tokenchars` (rebuild) this track. |
| **D11** | Privacy filter + project/session scoping identical on every ladder step. |
| **D12** | OR token selection: max **8** contentful; sort length **desc**, then lexical **asc**; take first 8. |
| **D13** | **SQL LIMIT on every MATCH** (M1): `ORDER BY rank LIMIT ?` with bound = `min(caller_limit_cap, LEXICAL_MATCH_HARD_CAP)` where hard cap is **200** and caller from recall uses **`candidate_depth(limit)`** (already 15–50). Applies to R0/R1/R2 so OR cannot flood rerank/graph. |
| **D14** | Token split SOOT (M4): extract (and `sanitize_fts_query`) split on non-alphanumeric **including `_`** so tokens match unicode61 indexing. Update existing sanitizer unit tests. |

### 4.1 Literal English stopword set (D3)

**Include** (case-insensitive; drop from contentful):

```
a an the is are was were be been being have has had do does did
will would could should may might must shall can need to of in for on
with at by from as into through during before after above below between
out off over under again further then once here there when where why how
all each few more most other some such only own same so than too very
just now what which who whom this that these those am i me my we our you
your he she it they them their and or but if because until while about
```

**Explicitly exclude from stopwords (always contentful if length ≥ 2):**

```
not no nor never neither none nobody nothing nowhere
```

Rationale: dropping negators inverts intent (`what not to forget` → `forget`). Contract forms (`n't`) are split by punctuation; resulting `t` is dropped by min-length-2, not by stopword list.

**Also contentful rules:** tokens with `len < 2` are never contentful (drops `t`/`s` noise from contractions). Deduplicate while preserving first-seen order before R1/R2.

---

## 5. Requirements (F-list)

| ID | Requirement |
|----|-------------|
| **F1** | When `rescue` and empty R0 and ≥3 tokens, run D2 ladder before T105 and empty hint. |
| **F2** | Empty hint fewer-keywords via **core** token helpers; gate on raw ≥3 **and** contentful ≥1 (D7/M5). |
| **F3** | No auto-semantic (D4). |
| **F4** | Hermetic: multi-token NL empty under R0 → hits under R1 or R2 when `rescue=true`. |
| **F5** | BM25-honest: scores from FTS rank; `source=fts`. |
| **F6** | Single- and two-token queries: no ladder (gate N/A); behavior equivalent to today aside from `_` split (D14). |
| **F7** | True empty remains empty; no infinite loop. |
| **F8** | Pure tests: extract (incl. `_` split), stopword/negator, dedupe, match_and/match_or, select_or_tokens. |
| **F9** | Low-level MATCH path does not re-run `sanitize_fts_query` on operator expressions. |
| **F10** | `recall_full` passes **raw** query + `rescue=true` + limit to lexical; bridge keeps sanitized primary AND. |
| **F11** | T105 substring still after FTS ladder if empty (≤10k guard intact). |
| **F12** | T90/T140 safety: only quoted tokens + literal ` OR ` / space. |
| **F13** | Debug log when R1/R2 fires (stage + counts). |
| **F14** | CAPABILITIES + CHANGELOG; optional FTS5-catch pointer. |
| **F15** | No unwrap/expect in production path. |
| **F16** | **Forget does not widen match** (M2): `lexical_search` with `rescue=false` (default). Hermetic or unit proof. |
| **F17** | Deterministic OR token selection (D12). |
| **F18** | Scope/privacy identical on R0–R2. |
| **F19** | Every MATCH uses SQL **LIMIT** (D13); hermetic or unit proof that R2 path binds a limit. |
| **F20** | Underscore split SOOT (D14); `ai_brains_core` → tokens `ai`, `brains`, `core`. |
| **F21** | Contentful dedupe + min length 2 (D3). |
| **F22** | Negators preserved as contentful (AC6b). |

---

## 6. Non-goals

- Semantic floor / RRF changes (T218)
- Pretty default / role-prefix strip (T101 / T224)
- ANN / vector index
- FTS schema retokenize (porter, trigram, `tokenchars`, fts5vocab) — rebuild-class (D10)
- Auto-suggest / did-you-mean / emoji in CLI hints
- Multi-round ledgerful bridge rescue
- Control-plane evidence list FTS rescue
- clap 5 / rusqlite 0.40
- Changing forget force/confirm UX beyond not widening match
- FTS5 `NEAR` proximity rescue (soft residual M8)

---

## 7. API / module shape (implementation guide)

### 7.1 Pure helpers (`ai-brains-core` fts module)

```text
extract_fts_tokens(query) -> Vec<String>
  // split on !is_alphanumeric()  — includes '_', no special-case keep-underscore
is_english_stopword(token) -> bool   // §4.1 include set only
contentful_tokens(tokens) -> Vec     // !stopword && len>=2; dedupe order-preserving
match_and(tokens) -> String          // "a" "b" "c"
match_or(tokens) -> String           // "a" OR "b" OR "c"
select_or_tokens(contentful) -> Vec  // D12 cap 8
sanitize_fts_query(query) -> String  // match_and(extract_fts_tokens(query)) — same SOOT
```

### 7.2 Retrieval

```text
LexicalSearchOptions { rescue: bool, limit: usize }  // or explicit params
// limit: from recall = candidate_depth(user_limit); hard cap min(., 200)

lexical_search(conn, raw_query, project, session, opts)
  tokens = extract(raw)
  R0: match_query(match_and(tokens), limit)
  if opts.rescue && empty && tokens.len() >= 3:
    c = contentful(tokens)
    R1: if c non-empty && c != tokens (as sets/seqs): match_and(c)
    R2: if empty && c.len() >= 2: match_or(select_or_tokens(c))
match_query(conn, match_expr, project, session, limit)
  // parameterized MATCH ?; ORDER BY rank LIMIT ?; no sanitize
```

### 7.3 Call sites

| Caller | rescue | limit |
|--------|--------|-------|
| `recall_full` | **true** | `candidate_depth(limit)` |
| `forget --match` / UUID lookup | **false** | small default (e.g. 50) or unlimited AND-only with hard cap still applied for safety |
| other | **false** | hard cap |

### 7.4 Recall pipeline order (frozen)

1. Sanitize for bridge only
2. Bridge (optional)
3. Lexical with rescue ladder (raw query, rescue=true, limited)
4. T105 substring if still empty
5. Semantic if requested
6. Fuse / rank / graph expand on **already-bounded** FTS set
7. CLI empty hint (F2)

---

## 8. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | Hermetic: pin with contentful keywords only; ≥3-token NL query empty under R0 → hits under R1/R2 with `rescue=true`; `source=fts`. |
| **AC2** | Single-token known pin still hits. |
| **AC3** | Multi-token gibberish stays empty. |
| **AC4** | Two-token `forget list` still hits without needing ladder. |
| **AC5** | Pure: `match_or` only quoted tokens + ` OR `; user `foo OR bar` safe. |
| **AC6** | Pure: stopwords drop `what`/`did`/`about`; keep `forget`/`list`. |
| **AC6b** | Pure: `not`/`no`/`never` remain contentful; `what not to forget` contentful includes `not` and `forget`. |
| **AC7** | CLI: multi-token empty + contentful≥1 → fewer-keywords + semantic/global guidance. |
| **AC7b** | CLI: all-stopword multi-token empty → **no** fewer-keywords line (M5). |
| **AC8** | Default lexical path has no embedding dependency. |
| **AC9** | T105 still reachable after ladder empty on small vault. |
| **AC10** | `recall_full` passes raw + rescue; bridge sanitized AND. |
| **AC11** | Focused nextest green (core fts, retrieval, cli hint). |
| **AC12** | CHANGELOG + CAPABILITIES. |
| **AC13** | Full gate green; review clean. |
| **AC14** | **forget / rescue=false:** ≥3-token query that would only hit via R2 does **not** return that hit when `rescue=false` (M2). |
| **AC15** | **LIMIT:** match path SQL includes LIMIT; bound ≤ 200; R2 cannot return unbounded rows (M1). |
| **AC16** | Pure: `extract_fts_tokens("ai_brains_core")` → `["ai","brains","core"]` (M4/D14). |
| **AC17** | Pure: contentful dedupe `["forget","list","forget"]` → `["forget","list"]`. |

---

## 9. Test plan (TDD)

**Red first:**

1. Pure: extract `_` split, stopword/negator, dedupe, match builders, select_or (AC5/6/6b/16/17).
2. Hermetic retrieval: AC1 with `rescue=true`; AC14 with `rescue=false`.
3. AC3 gibberish; AC4 two-token.
4. CLI unit: AC7 + AC7b.
5. AC15: assert SQL bind or result count ≤ hard cap under forced broad OR fixture if practical; at minimum unit that limit is passed into match_query.

**Green:** implement.
**Manual:**

```powershell
ai-brains recall "what did we decide about forget list" --no-bridge --limit 5
ai-brains recall "forget list" --no-bridge --limit 5
ai-brains recall "brittle hotspot fix" --no-bridge --limit 5   # M6 non-stopword 3+
ai-brains recall "zzzz_no_such_token_aaa bbb ccc" --no-bridge
# forget must not OR-widen (dry-run):
ai-brains forget --match "what did we decide about forget list" --dry-run
```

---

## 10. Files (expected)

| Path | Change |
|------|--------|
| `crates/ai-brains-core/src/fts.rs` | extract/stopword/match; `_` split; sanitize SOOT |
| `crates/ai-brains-retrieval/src/lexical.rs` | options, ladder, LIMIT, match_query |
| `crates/ai-brains-retrieval/src/recall.rs` | raw + rescue=true + candidate_depth limit |
| `crates/ai-brains-cli/src/commands/recall.rs` | hint via core helpers (F2) |
| `crates/ai-brains-cli/src/commands/forget.rs` | explicit rescue=false (or default) |
| tests under core/retrieval/cli | AC suite |
| `Docs/CAPABILITIES.md`, `CHANGELOG.md` | user-facing note |

---

## 11. Verification

```powershell
cargo nextest run -p ai-brains-core fts
cargo nextest run -p ai-brains-retrieval
cargo nextest run -p ai-brains-cli -- recall
cargo clippy -p ai-brains-core -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings
# finalize:
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

---

## 12. Risk / review notes

| Risk | Mitigation |
|------|------------|
| OR too broad → noisy / huge sets | Token cap 8 + **SQL LIMIT 200 / candidate_depth** (M1) |
| forget silent wider delete | **rescue default false** (M2) |
| Negator inversion | Exclude negators from stopwords (M3) |
| `_` phrase vs AND mismatch | Split `_` like unicode61 (M4) |
| Hint drift / all-stopword bad advice | Core SOOT + contentful≥1 gate (M5) |
| Operator injection | Quoted tokens only (F12) |
| Contract churn | Keep `source=fts` |

Cross-model review: FEATURE quality path after internal clean.

---

## 13. Deferred / soft residuals

| Residual | Disposition |
|----------|-------------|
| Bridge multi-round rescue | Soft — T231+ |
| Control-plane evidence FTS rescue | Soft |
| JSON rescue-stage field | Soft |
| Locale / configurable stopwords | Soft |
| Porter / trigram / `tokenchars` / fts5vocab (needs rebuild) | Soft (M8) |
| FTS5 `NEAR` as R2.5 proximity | Soft (M8) |
| T224 role-prefix | T224 |
| T218 semantic quality | T218 |
| forget force/confirm UX redesign | Out of scope |

---

## 14. AI fold-in (2026-08-09)

Sources: `C:\dev\AI-review.md` — AI1 (deep findings) + AI2 (architecture summary).

| Item | Source | Disposition |
|------|--------|-------------|
| **M1** R2 OR unbounded → LIMIT before rerank/graph | AI1 High | **Absorbed** D13/F19/AC15 — hard DoD |
| **M2** forget inherits rescue → destructive | AI1 High | **Absorbed** D9/F16/AC14 — `rescue` opt-in; forget strict |
| **M3** negators + literal stopword list | AI1 Medium | **Absorbed** D3/§4.1/F22/AC6b |
| **M4** `_` vs unicode61 | AI1 Medium | **Absorbed** D14/F20/AC16 — split `_` in extract+sanitize |
| **M5** hint reuses core tokens; contentful≥1 | AI1 Medium | **Absorbed** D7/F2/AC7b |
| **M6** non-stopword 3+ also OR | AI1 Low | **Absorbed** §2 note + manual dogfood |
| **M7** forget UUID ladder waste | AI1 Low | **Absorbed free with M2** (rescue=false) |
| **M8** NEAR / tokenchars / fts5vocab | AI1 Low | **Soft residual** §13 |
| Diagnosis / FTS5 research / D5 BM25 | AI1 confirm | Affirmed |
| Ladder diagram R0–R2 + T105 | AI2 | Affirmed (aligned) |
| Deduplicate contentful tokens | AI2 | **Absorbed** D3/F21/AC17 |
| Privacy/scoping per stage | AI2 | Affirmed F18 |
| Emoji “💡” in hint | AI2 | **Rejected** — plain text only (CLI noise) |
| Auto-implement as if coded | AI2 tone | Plan-only until go |

**Not absorbed:** dep bumps; FTS rebuild; auto-semantic; contract `source` change.

---

## 15. Absorbed deferred (prior)

| Source | Item | Handling |
|--------|------|----------|
| deferred.md T217 | FTS natural-phrase empty quality 4 | Hard DoD |
| Series README P1 | empty FTS trap | This track |
| Placeholder F1–F5 | expanded D1–D14 + F/AC | Spec |
| T105/T111 sequencing | ladder before substring + hint | F11 |

**Not absorbed:** T218–T232; T233; closed harness series.

