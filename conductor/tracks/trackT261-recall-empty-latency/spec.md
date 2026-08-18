# T261 — Recall empty-query latency

- **Track ID:** T261-RecallEmptyLatency
- **Status:** **Planned** (plan-only until go; registry stays **Pending**)
- **Category:** PERFORMANCE / UX / RETRIEVAL
- **Owner:** —
- **Source:** Audit 2026-08-16 — friction: `recall '' --format pretty` **5730 ms**. Live re-dogfood 2026-08-17: `""` is already ~640 ms; **whitespace and all-stopword take ~2 s and return hits**.
- **Depends on:** T105 substring fallback; T111/T207 empty hint + Scope; T217 `contentful_tokens` / rescue ladder; T86 `recall -` stdin; T243 `search` alias; T260 `include_symbols` (inherit, do not reopen)
- **Blocks / feeds:** Daily `recall` / `search` / `sync query` vault arm stop burning seconds (and writing `MemoryPinned`) on queries with nothing to search. Graph live projection stays **T262**. Leftover-project `--global` stays **T264**. Ledger pane stays **T271**.
- **Absorbs:** Empty / whitespace / punctuation-only / all-stopword / single-char-only queries running T105 LIKE, T217 R0 MATCH, Ledgerful `--auto-index` bridge, `--semantic` embed, graph expansion, and `MemoryPinned` appends
- **Not absorbed:** Ranking quality (T260 Completed); FTS rescue correctness (T217 closed); leftover-project isolation (T264); graph projection (T262); format maze (T266)
- **Research date:** 2026-08-17 (HEAD `1842df0` T260 `#175`)
- **AI fold-in:** none yet
- **Ledger:** planning DOCS TX `afe06292-7680-4d1f-b22e-a8a447f0a423`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** change `forget --match`. Do **not** reopen T217 rescue, T105 10k guard (contentful miss), T207 hint copy, T240 F2, T255 declines, T260 GLOB. Do **not** `cargo install`, write live `.env`, bump clap, or add crates.

---

## 1. Objective

A query with **zero contentful tokens** must become the existing T207 empty response **without** opening FTS rescue, substring LIKE, Ledgerful bridge, semantic embed, or graph expansion.

That advances the north star: capture independence includes **not writing** `MemoryPinned` events for `"   "` or `"the the the"`. Today those queries LIKE/MATCH almost every pin, then the CLI appends a pin event per hit. Agents that fat-finger an empty or stopword query should get the empty hint in well under a second, not a 2 s dump of random session crumbs.

No models on the default path. No new events. No DTO keys. No forgotten-match change.

---

## 2. Live baseline (2026-08-17)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `1842df0` — T260 `#175` squash-merged. Tree **CLEAN**. `main` == `origin/main` (`00`). |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` (mtime **2026-08-17 18:20**, 24 848 896 bytes). **Pre-T260** (`--symbols` unknown). T257 on PATH. **Do not `cargo install`.** |
| Source debug | `target\debug\ai-brains.exe` (mtime **2026-08-17 19:34**, 40 980 480 bytes). Newer than PATH. |
| `preflight --summary` | Scope path owner `3581317d` (`C:\dev\ai-brains`); **2854** pinned. Discovery grants empty (T263). |
| `recall "" --format pretty --no-bridge` | **639 ms.** T207 hint. No hits. |
| `recall "" --format pretty` (bridge on) | **773 ms.** Same hint. Bridge extra ~130 ms (warm index). **Audit 5730 ms not reproduced** (warm index + 2854-pin project; 2026-08-16 likely cold `--auto-index` and/or leftover 18k / whitespace). |
| `recall "zzzznonexistentqueryxyz" --no-bridge` | **758 ms.** Same empty hint. Fair miss. |
| `recall "   " --format pretty --no-bridge` | **2145 ms. RETURNS HITS** (LIKE `%   %` matches almost every pin). Pretty prints Session + content. CLI will `MemoryPinned` each hit. |
| `recall "the the the" --format pretty --no-bridge` | **2027 ms. RETURNS HITS** (FTS R0 AND of stopwords). Same pin pollution. |
| `recall "" --global --no-bridge` | **485 ms.** Empty. Vault > 10 000 so T105 skips LIKE even for whitespace (**497 ms** for `"   " --global`). Project 2854 is **under** the guard — that is why project whitespace is the slow path. |
| `recall "" --semantic --no-bridge` | **1218 ms.** Embed HTTP of empty string + score stored BLOBs. Hint is the semantic empty copy. |
| Last GitHub PR | [#175](https://github.com/Ryan-AI-Studios/AI-Brains/pull/175) T260. `gh pr view --comments`, `/reviews`, `/comments`, issue comments all **empty**. HEAD is `main` (no open product PR). Open PRs are Dependabot only. **last-PR Cursor: N/A.** |
| Ledgerful | `doctor` ready (legacy `.changeguard` / sig-pin / timings / :8081 unreachable; :8083 ok). 0 pending 0 drift at plan start. Hotspot **#1** `project.rs` (3.903) — **do not touch.** `forget.rs` #3 — **do not add a contentless filter there.** `sync.rs` #2 inherits via `recall_full`. |
| ai-brains recall | T243 `search` = `recall`. T260 default-exclude lives in source, not PATH. No prior “contentless short-circuit” pin. |

### 2.2 Why this still matters

| Residual | Why it is a product hole / why decline |
|----------|----------------------------------------|
| Audit `recall ""` 5.7 s | Not reproduced today on this Scope (639–773 ms). Still a real class: `recall_full` always runs bridge (`ledgerful search --auto-index --json ""`) unless `--no-bridge`. Cold `--auto-index` is the plausible 5 s. **DoD: never call the bridge on a contentless query.** |
| Whitespace LIKE `%   %` | `substring_fallback` only skips `query.is_empty()`. `"   "` is not empty. COUNT then table scan. Project 2854 < 10k → **match-all**. **DoD: treat 0-contentful as empty before COUNT.** |
| All-stopword FTS | `lexical_search` returns early only when **extracted tokens** are empty. `"the the the"` has tokens → R0 MATCH of stopwords → hits. T217 already skips R1/R2 when contentful is empty; R0 still runs. **DoD: 0-contentful short-circuit in `recall_full` (not in `lexical_search`, so forget stays strict).** |
| `--semantic ""` | `fetch_embedding("")` + load every stored BLOB. Closed set already has `skipped`. **DoD: skip embed; status `skipped`, detail `contentless_query`.** |
| `MemoryPinned` on garbage | CLI pins every hit. Whitespace/stopword recall writes events. Capture independence forbids hidden CoT; it also should not invent “the user recalled this” from a blank query. **DoD: 0 hits ⇒ 0 pin events.** |
| Live `< 500 ms` (placeholder F3) | Process + SQLCipher open alone is ~600 ms today for `""`. Hermetic `recall_full` is the ms proof. Live bar: whitespace/stopword must join the `""` band; stretch `< 500 ms` if vault-open allows. Phase 0 may retarget the live number, not the short-circuit. |
| T105 10k LIKE skip | Still required for **contentful** misses. Do not remove. |
| T217 rescue | Still required for natural phrases with contentful tokens. Do not reopen. |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Token SoT | `ai-brains-core/src/fts.rs` | `extract_fts_tokens` (unicode61, split `_`); `contentful_tokens` (drop stopwords + `len < 2`, case-insensitive dedupe); `should_suggest_fewer_keywords`. **No `is_contentless_query` today.** |
| Export | `ai-brains-core/src/lib.rs` `:30–33` | Add the new helper to the existing `pub use fts::{…}` list. |
| `lexical_search` | `lexical.rs` `:60–63` | Empty **extracted tokens** → `Ok([])`. All-stopword still runs R0. `rescue` default **false**. |
| `substring_fallback` | `lexical.rs` `:211–235` | **COUNT first**, then `if query.is_empty()`. Whitespace never hits that return. Skip if scoped count > 10 000. |
| `recall_full` | `recall.rs` `:237–506` | Sanitize (bridge only) → **bridge** → lexical rescue → substring → optional semantic → blend → T260 retain → graph (default `graph_hop_depth=1`) → `rerank_hits` → stub dedupe → truncate. **No contentless gate.** |
| Bridge | `query_ledgerful_bridge` `:523` | `ledgerful search --auto-index --json <sanitized>`. `sanitize_fts_query("")` is `""`. |
| Semantic | `semantic.rs` `semantic_search` `:236` | Always `fetch_embedding(query)` then score BLOBs. |
| Embedding closed set | `ai-brains-contracts/src/recall.rs` `:40` | `ok` \| `unreachable` \| `error` \| `no_stored_embeddings` \| **`skipped`**. Retrieval never emits `skipped` today. |
| CLI query | `main.rs` Recall `:481` `query: String` (required). T86 `:3775–3779` `"-"` → `read_query_from_stdin`. |
| Stdin | `main.rs` `:2404–2420` | TTY → error (no hang). Piped trim-empty → **`Err("Query read from stdin is empty.")`**. |
| Pretty empty | `commands/recall.rs` T207 | Scope → optional Session (omit generated) → Embedding ≠ ok → hint. `build_recall_hint` still COUNTs project memories when `< 10` (keep). |
| `search` | `visible_alias` of Recall | Same path. |
| `sync query` | `sync.rs` `:487–503` | `recall_full` + `no_bridge: true` on pretty vault arm. Inherits F1. Ledger section is **T271**. |
| Forget | `forget.rs` `lexical_search` `rescue: false` | **Do not** apply the contentless gate. `"the"` still MATCHes. |
| Construction | `RecallOptions` | **No new field.** Short-circuit is a query-shape check. Sites: CLI, `sync.rs` ×2, `ai-brainsd` `:271`. |
| clap / rusqlite / serde_json | workspace clap **4.5** / lock **4.6.1** / builder **4.6.0** / crates.io **4.6.6**; rusqlite **0.39.0**; serde_json lock **1.0.150** / crates.io **1.0.151**; rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute. |
| `project.rs` | Hotspot #1 | **Do not touch.** |

### 2.4 Research (current — not memory)

| Claim | What was checked | Fit |
|-------|------------------|-----|
| clap 4 required positional `""` | crates.io clap **4.6.6** (2026-08-06). clap 5 **not** current. A required `String` accepts empty; it is not “missing.” Do not add a value parser that rejects `""` (that would turn F1 into clap exit 2). | Keep required `<QUERY>`; empty is a valid value. |
| Empty search = match-all is a known footgun | NEST / elasticsearch-net **#2179**: empty query rewrite became match-all. Elastic `match_none` exists specifically to mean “run the request, return nothing.” | F1 is `match_none`, not match-all. Do **not** skip the command (exit 2) — return the T207 empty envelope. |
| SQLite FTS5 MATCH | sqlite.org/fts5.html — MATCH is token-based. Empty MATCH is not a documented “no documents” shortcut; LIKE `% %` **is** match-almost-all. | We already skip MATCH on empty tokens. The live hole is LIKE + stopword R0, not MATCH `''`. |
| T105 LIKE | Live src + sqlite LIKE ASCII case-fold. No API change. | Keep for contentful misses. Gate contentless **before** COUNT. |
| N/A | Windows schtasks / SQLCipher page encrypt / new crates | This track does not schedule, encrypt, or add deps. |

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until the owner says go / implement / execute. |
| **F1** | `is_contentless_query(q)` ≡ `contentful_tokens(&extract_fts_tokens(q)).is_empty()`. Covers `""`, whitespace, punctuation-only, all-stopwords, single-char-only (`"a"`, `"i"`). **SoT in `ai-brains-core`.** |
| **F2** | Clap still requires `<QUERY>`. TTY `recall -` stays T86 refuse (do not hang). Piped `-` with trim-empty stdin becomes query `""` (then F1) — **not** the T86 `Err("Query read from stdin is empty.")`. Contentful `-` unchanged. |
| **F3** | Hermetic `recall_full` contentless: milliseconds, **no** SQL MATCH/LIKE/embed/bridge. Live pretty `--no-bridge` contentless: whitespace/stopword must sit in the same band as today’s `""` (not ~2 s). Stretch **&lt; 500 ms** if vault-open allows; Phase 0 may keep the live ceiling at **≤ current `""` + 100 ms** if SQLCipher open alone is ≥ 500 ms. |
| **F4** | Contentful tokens still run T217 rescue + T105 LIKE. `"llo worl"` / `"forget list"` / `"what not to forget"` stay as today. |
| **F5** | Capture independence: query-time only. 0 hits ⇒ 0 `MemoryPinned`. No CoT. No new events. |
| **F6** | `--semantic` + contentless: **do not** `fetch_embedding`. `embedding.status = "skipped"`, `detail = Some("contentless_query")`, `endpoint = None`. `semantic_post_threshold_count = Some(0)`. T218 F11 honesty does **not** fire (`status != ok`). No new DTO key. |
| **F7** | Gate at the **top of `recall_full`**, before bridge / lexical / substring / semantic / graph / `rerank_hits`. Defense-in-depth: `substring_fallback` returns `Ok([])` on contentless **before** `COUNT(*)`. |
| **F8** | `search` and `sync query` vault arm inherit F1 via `recall_full`. `forget --match` uses `lexical_search` and stays **unfiltered** (all-stopword still MATCH). |
| **F9** | T207 / T228 / T231 empty chrome stays: Scope, omit generated Session, hint quotes the **typed** query (spaces preserved), recall pretty still appends the ledger next-step. Do not invent a new “empty query” sentence. |
| **F10** | No clap 5. No new crates. No pin bump. |
| **F11** | T240 F2 and T255 declines stay closed. |
| **F12** | PATH-behind is operator / `cargo install`. Tests use hermetic/source bin. |
| **F13** | Decline extras: leftover-project `--global` (T264); graph projection (T262); T271 ledger pane; `source_tag` column; raise `candidate_depth`; BM25 retune; ANN; clap reject `""`; `--no-empty-short-circuit` flag; doctor change. |
| **F14** | No new `RecallOptions` field. Construction sites compile unchanged. |
| **F15** | `include_symbols` / `--symbols` does **not** override F1. Contentless is empty even when mixing stubs. |
| **F16** | Default `graph_hop_depth=1` stays for contentful queries. Contentless never reaches graph. |
| **F17** | JSON empty: `results: []`, existing `hint` with `No results`, optional `embedding` only when `--semantic` (F6). T180 compact / N−1 ignore unknowns. |
| **F18** | All-stopword / single-char becoming empty is an **intentional behavior change** (today they match-all). Document in CAPABILITIES + CHANGELOG. Negators stay contentful (`not` / `no` / `never` — T217 F22). |

---

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | `recall_full("", …)` → `hits` empty; `embedding` None when `semantic: false`. |
| **AC2** | `recall_full("   ", …)` → empty hits (not substring match-all). |
| **AC3** | `recall_full("the the the", …)` and `recall_full("what is the", …)` → empty hits. |
| **AC4** | `recall_full("ok", …)` / `"forget list"` / `"what not to forget"` still search (contentful). Existing T105 `"llo worl"` still finds via substring. |
| **AC5** | `recall_full("", …, semantic: true)` → empty hits; `embedding.status == "skipped"`; `detail == Some("contentless_query")`; hermetic must not call the embed provider. |
| **AC6** | Hermetic CLI `recall "" --format pretty` → exit 0, `Scope:`, `No results for ''`. |
| **AC7** | Hermetic CLI `recall "   " --format pretty` → **No results**, **no** hit lines (regression vs live 2026-08-17). |
| **AC8** | Hermetic CLI `recall "the the the" --format pretty` → No results (not a stopword dump). |
| **AC9** | Hermetic CLI `recall "" --format json` → `results == []`, `hint` contains `No results`, exit 0. |
| **AC10** | Hermetic `search "" --format pretty` same empty chrome as recall (T243 alias). |
| **AC11** | Piped `recall -` with empty/whitespace stdin → F1 empty envelope, exit **0** (not T86 empty-stdin error). |
| **AC12** | TTY `recall -` still errors (T86 hang guard). Keep the existing hermetic if present; do not weaken. |
| **AC13** | `is_contentless_query` units: `""` / `" \t\n"` / `"..."` / `"the"` / `"a"` → true; `"ok"` / `"not ok"` / `"forget"` → false. |
| **AC14** | `substring_fallback("   ", …)` returns `[]` **without** requiring a 10k-row vault (early contentless return). Existing T105 10k skip test stays green. |
| **AC15** | `lexical_search("the the the", rescue: false)` still **runs** MATCH (forget path). Do not add F1 to `lexical_search`. |
| **AC16** | Contentless `recall_full` does not invoke bridge / semantic fetch / graph neighbor (unit: empty outcome + no error; CLI: no `MemoryPinned` because 0 hits). |
| **AC17** | `--symbols` + contentless still empty (F15). |
| **AC18** | Live dogfood note on go: time `""`, `"   "`, `"the the the"` `--no-bridge --format pretty`. Whitespace/stopword in the `""` band. Record ms. |

---

## 5. Design notes

### 5.1 Helper

```rust
pub fn is_contentless_query(query: &str) -> bool {
    contentful_tokens(&extract_fts_tokens(query)).is_empty()
}
```

Do not invent a second stopword list. Do not trim-only (`"the"` is contentless; `"  ok  "` is not).

### 5.2 `recall_full` early return

Immediately after reading `project_id` / `session_id` (or before sanitize — sanitize is unused if we skip bridge):

```text
if is_contentless_query(query) {
    return Ok(RecallOutcome {
        hits: vec![],
        embedding: if options.semantic { Some(skipped/contentless_query) } else { None },
        semantic_post_threshold_count: if options.semantic { Some(0) } else { None },
    });
}
```

Then the existing pipeline. No `RecallOptions` flag.

### 5.3 `substring_fallback`

Move/replace the `query.is_empty()` check with `is_contentless_query(query)` **above** `project_memory_count`. Whitespace must not COUNT+LIKE. Empty string remains a subset.

### 5.4 Stdin

`read_query_from_stdin`: TTY refuse stays. After `trim`, **return** `""` instead of `Err`. `recall_full` applies F1. Pretty/JSON empty contracts apply. Do not hang.

### 5.5 Why not clap-reject `""`

Rejecting empty at clap is exit **2** (`fail_usage`) with no T207 hint. Agents and `recall -` pipes would break. Elasticsearch-class lesson: empty query should be `match_none` + a normal response, not a parser error.

### 5.6 Why not put F1 in `lexical_search`

Forget `--match "the"` would silently match nothing. T260 F10 already froze forget as unfiltered. Same isolation.

---

## 6. Non-goals

- Changing T217 rescue / stopword list / negators
- Removing T105 LIKE for contentful misses or changing the 10k guard
- T207 hint wording / Scope / T231 ledger next-step
- Graph projector / T69 live nodes (T262)
- Leftover-project `--global` (T264)
- `sync query` ledger pane (T271)
- clap 5 / new crates / pin bumps
- `cargo install` / live `.env` / leftover rebind
- Reopening T240 F2, T255 declines, T260 GLOB
- New `RecallResult` / `RecallOptions` fields
- Doctor / nightly / schedule

---

## 7. Verification plan (TDD — red names first)

Core (`fts.rs`):

- `is_contentless_query__empty_whitespace_punct_stopword_single_char__true`
- `is_contentless_query__ok_and_negator_phrase__false`

Retrieval (`tests/recall_empty_latency.rs` + `lexical.rs` unit):

- `recall_full__empty_query__no_hits__ac1`
- `recall_full__whitespace__no_hits__ac2`
- `recall_full__all_stopword__no_hits__ac3`
- `recall_full__contentful_still_searches__ac4` (or rely on existing T105)
- `recall_full__semantic_contentless__embedding_skipped__ac5`
- `recall_full__symbols_contentless__still_empty__ac17`
- `substring_fallback__whitespace__empty_before_count__ac14`

CLI hermetic (`tempdir` + init/pin; force `--format pretty` / `json`):

- `recall__empty_pretty__hint_no_hits__ac6`
- `recall__whitespace_pretty__no_hit_lines__ac7`
- `recall__stopword_pretty__no_hits__ac8`
- `recall__empty_json__results_empty__ac9`
- `search__empty_pretty__alias__ac10`
- `recall_stdin__piped_empty__short_circuit__ac11`

Keep green: T86 contentful `recall -`; T105 substring + 10k skip; T207 empty pretty/JSON; T217 rescue suite; T260 symbol demote.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| All-stopword users lose accidental hits | Intentional (F18). Those hits were match-all noise + pin pollution. CAPABILITIES + CHANGELOG. |
| `"a"` / `"i"` become empty | Same as T217 `len < 2` + stopword. AC13. |
| F3 `< 500 ms` misses on SQLCipher open | Phase 0 honesty; hermetic is the proof; live bar is “join the `""` band.” |
| Forget `--match "the"` broken | F8 / AC15 — do not gate `lexical_search`. |
| T86 empty-stdin scripts expecting exit 1 | Rare; F2 is the product. AC11. Contentful `-` stays. |
| `--semantic` JSON consumers unknown `skipped` | Already in T202 closed set / PROTOCOL-COMPAT / CAPABILITIES. |
| Hotspot `project.rs` | Do not touch. |
| Capture independence | 0 hits / 0 pins. |

---

## 9. Deferred absorb / decline

Full `conductor/deferred.md` scan 2026-08-17. `ISSUES.md` does **not** exist.

| Residual | Disposition |
|----------|-------------|
| `recall ""` 5.7 s (audit T261 row) | **Absorb** F1–F7 / AC1–AC18. Live shape is whitespace/stopword match-all + cold-bridge class. |
| T105 substring fallback | **Partial:** keep for contentful miss (F4). Contentless must not LIKE (F7 / AC14). |
| T217 rescue / stopwords | **Partial:** reuse `contentful_tokens` (F1). Do not change the ladder. |
| T207 / T228 / T231 empty chrome | **Partial:** reuse (F9). Do not rewrite copy. |
| T86 empty stdin error | **Absorb** F2 / AC11–AC12 (TTY refuse stays; piped empty → F1). |
| T260 closeout leftover-project `--global` | **Decline → T264** |
| T260 `source_tag` column | **Decline** (soft elsewhere) |
| Graph sparse / 4h pin no node | **Decline → T262** (this track only stops pinning contentless hits) |
| Governed 0 authority | **Decline → T263** |
| `preflight --global` blender | **Decline → T264** |
| Preflight `{text, word_count}` | **Decline → T265** |
| Format maze | **Decline → T266** |
| harness/whoami self-next; list footer | **Decline → T267** |
| `scan-roots` cwd-only | **Decline → T268** |
| Nightly human mixes Router | **Decline → T269** |
| Retention 0 candidates | **Decline → T270** |
| `sync query` ledger pane empty | **Decline → T271** (vault arm inherits F1; pane is T271) |
| T211 F25 / T218 ANN | **Decline** (soft) |
| T255 declined bag / T240 F2 | **Stay closed** F11 |
| T256–T260 PATH `cargo install` | **Decline** F12 / F13 |
| MSI / notarization / R-CI-BRANCH / anyhow allowlist / archive changeguard sweep | **Not related** (not recall latency) |
| Connector cursors, CE residuals, desktop, sync threat leftovers | **Not related** |
| last-PR Cursor (#175) | **N/A** — comments/reviews/inline empty; Dependabot only; nothing to mint |

---

## 10. Implement order (on go)

1. Phase 0 re-verify `recall_full` order, `contentful_tokens`, stdin, deferred, last PR, pins, live timings.
2. Red: core helper + `recall_full` / substring / CLI hermetics (AC names above).
3. Green: helper + early return + substring gate + stdin empty → `""`.
4. Docs: CAPABILITIES recall table + CHANGELOG (all-stopword behavior change).
5. Targeted nextest/clippy on `ai-brains-core` + `ai-brains-retrieval` + `ai-brains-cli`; full gate on finalize.
6. FEATURE TX commit; review.md; publish per implement-track (not this skill).

---

## 11. Soft residuals

| Residual | Why not DoD |
|----------|-------------|
| Skip CLI graph-vault open on contentless | Micro-opt after F7; SQLCipher open dominates |
| Skip T207 `< 10` memory COUNT on contentless | One indexed COUNT; keep the small-vault sentence |
| `lexical_search` 0-contentful early return | Would change forget (F8) |
| clap `value_parser` reject empty | Exit 2; declined F13 |
| Embed `skipped` endpoint label | `None` is enough |
| Live leftover 18k / PATH reinstall | Operator out of band |
| Daemon/HTTP recall contentless | Inherits `recall_full`; no extra wire |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-core/src/fts.rs` | Add `is_contentless_query` + units. |
| `crates/ai-brains-core/src/lib.rs` | Re-export. |
| `crates/ai-brains-retrieval/src/recall.rs` | F7 early return + F6 skipped embedding. |
| `crates/ai-brains-retrieval/src/lexical.rs` | Contentless return **before** COUNT in `substring_fallback`. |
| `crates/ai-brains-retrieval/tests/recall_empty_latency.rs` | **New** AC1–AC5 / AC14 / AC17. |
| `crates/ai-brains-cli/src/main.rs` | Piped empty `-` → `""` (F2). |
| `crates/ai-brains-cli/tests/recall_empty_latency.rs` | **New** AC6–AC11. Or extend `recall_empty_pretty_scope.rs` if cheaper. |
| `Docs/CAPABILITIES.md` | Recall table: contentless = T207 empty, no LIKE/bridge/embed. |
| `CHANGELOG.md` | Behavior: all-stopword / whitespace no longer match-all. |
| `conductor/conductor.md` / `deferred.md` / README-T256–T271 | Planned note. **Pending** until go. |

Do **not** touch: `project.rs`, `forget.rs` match path, `ranking.rs`, `symbol_stub.rs`, contracts DTO fields, migrations.
