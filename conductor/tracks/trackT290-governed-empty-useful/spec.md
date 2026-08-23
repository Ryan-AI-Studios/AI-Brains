# T290 — Granted-empty governed lists/progressive must be useful

- **Track ID:** T290-GovernedEmptyUseful
- **Status:** **Completed**
- **Category:** FEATURE / UX / HONESTY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — evidence/source/review list **6/8**; `query progressive` **6/8** (`denied: false`, `items`/`results: []`, `next: recall` ellipsis, no pin count). Placeholder minted with T285–T300 (`76c4db9`). T263 ✅ F8 list overlay names recall (**H2 declined**). T243 ✅ F5 progressive `next_step`. T275 ✅ grants (live **3 of 3**). T288 ✅ project briefing stanza (do **not** steal). T289 ✅ Personal deny (do **not** steal).
- **Depends on:** T203 ✅ list DTOs; T221 ✅ deny vs empty exit; T243 ✅ progressive `next_step`; T263 ✅ F8 overlay helper; T214 ✅ `count_pinned_memories`; T275 ✅ live grants
- **Blocks / feeds:** Operators who run `evidence list` / `source list` / `review list` / `query progressive` after grants see that the vault has pins and a **copy-paste** `recall` command. Briefing vault-pin stanza remains **T288**. Personal deny remains **T289**. `query trace` remains **T291**. Neighbors **T293**.
- **Absorbs:** Placeholder problem text + Manual DoD four commands; deferred.md “Lists/progressive pin count”; T263 **F8** parenthetical “or that string + vault pins are not governed evidence”; T263 **F9** progressive empty left as T243 ellipsis — **reopened here** for copy-paste query + `Pinned: N` (not a second SOOT family)
- **Not absorbed (DoD):** T263 H2 pin→Approved; T288 briefing stanza / `vault_pin_*` keys; T289 Personal; T291 `query trace` `null`; T292 `policy check` human; T293 neighbors; T294 leftover upsert; T299 forget-list; T240 F2; clap 5 / rusqlite 0.40; list/progressive DTO new required keys; fabricating evidence/source/review rows
- **Research date:** 2026-08-23 (plan dogfood HEAD `6a8deb3` T289 `#205`; product `src/` = T263 list overlay + T243 progressive hints; PATH **0.1.2** 2026-08-22 19:41 **without** T285–T289 — lists/progressive hole is in **source and PATH**; T288/T289 did not touch these emit paths)
- **AI fold-in:** 2026-08-23 `agy-review.md` + `opencode-review.md` (HEAD `efdfd3d`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy m1 human parity all three nouns (AC3); Agy m2 single-line formatter (F7/AC1); Agy O2 tab sanitize case (AC4); OpenCode m1 `QueryStore` import in four callers not `governed_common.rs` (F12/§5.2); OpenCode m2 AC6 own needle not `progressive_cmd` `"x"` (AC6); OpenCode O1 exact AC1 `assert_eq!` (AC1). **Already:** Agy O1 CLI-EXIT-CODES/OPERATIONS (F25/AC10); OpenCode O2 progressive no `--format` (F10). **No declines of B/M.** Disposition **§13**.
- **Ledger:** planning DOCS TX `c66b1485-a4a7-4ca6-87d2-8b2e2d8b5865`. Fold-in DOCS TX `8875a1cc-fba7-49a3-8026-dff1a033ddd6`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** pin production decisions to the live vault as implement (hermetic needle is SoT; Manual unique canary allowed on go). Do **not** rewrite `.env`. Do **not** live `policy bootstrap` extra grants (live already **3 of 3**). Do **not** `migrate governed`. Do **not** grow hotspot `project.rs` / `sync.rs` / `forget.rs` production / CLI `preflight.rs` / `personal.rs` / `briefing.rs` / `query_store.rs`. Grow `governed_common.rs` **only** for the shared next-step formatter (hotspot **#5** — keep COUNT in callers). Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Granted-empty lists and progressive name pins + a runnable recall.** When `denied: false` and the collection is empty, JSON `next_step` must contain `recall` **and** a copy-paste query (lists: `what did we decide`; progressive: the operator query) **and**, when COUNT succeeds, `(Pinned: N)` from `count_pinned_memories`. Agents must not stop at `ai-brains recall "…"` next to thousands of vault pins.
2. **Arrays stay empty (no H2).** `items: []` / `results: []` stay honest. Do **not** fabricate evidence, sources, review items, or Approved hits. Dual-model: lists/progressive = governed rows; pins via `recall`.
3. **Human empty is not a dead end.** `--format human` empty lines (`evidence: (none)` / `sources: (none)` / `review items: (none)`) print a **second stdout line** with the same next-step string. Progressive stays JSON-only (no `--format`; CAPABILITIES frozen).
4. **North star.** Capture independence: CLI overlay + projection COUNT only. No new events. No hidden CoT. No DTO required-key growth (T180). T263 F8 overlay pattern stands.

This unblocks daily governed discovery: T275 unlocked grants (live **3 of 3**); T263 made granted-empty *honest* (`next_step` names `recall`); the 2026-08-22 audit still scores **6/8** because an ellipsis is not a command and does not show that pins exist.

---

## 2. Live baseline (re-scan 2026-08-23)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `6a8deb3` T289 squash `#205`. Tree **CLEAN**. `origin/main` = HEAD. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. Has T274. **Does not have T285–T289.** Lists/progressive emit is **identical in source** (T288 overlay is `briefing.rs` only; T289 is Personal renderer). Hole is in **source + PATH**. **Do not `cargo install`.** |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **3908** (volatile). In-context **0/0/0**. Word **443**. Grants omitted (live **3 of 3**). |
| `evidence list --format json` | `denied` absent (success envelope), `items: []`, `next_step`: `Ungoverned vault search: ai-brains recall "…"`. **No `Pinned:`**. Exit **0**. |
| `source list --format json` | Same overlay string. `items: []`. |
| `review list --format json` | Same overlay string. `items: []` (no `warnings` / `more_available` keys — DTO smaller). |
| `--format human` (all three) | `evidence: (none)` / `sources: (none)` / `review items: (none)`. **No next line.** Exit **0**. **This is the human hole.** |
| `query progressive "what did we decide about SQLCipher"` | `denied: false`, `results: []`, `next_step` **ellipsis** — **does not contain `SQLCipher`**. Exit **0**. **This is the progressive hole.** |
| `briefing project` | T288 stanza on **source** (`cargo run`); PATH-behind. **Not this track.** |
| Last GitHub PR | [#205](https://github.com/Ryan-AI-Studios/AI-Brains/pull/205) T289 (2026-08-23). `comments` / `reviews` / issue comments / inline `/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, `#58` tower-http, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081). **0 pending / 0 drift.** Hotspot **#1** `project.rs` — **do not touch.** `governed_common.rs` **#5** (2.616) — **grow only the overlay helper**; COUNT stays in callers. `personal.rs` **#7** — **do not touch.** CLI `preflight.rs` **#8** — **do not grow.** |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why granted-empty still trains “stop / empty vault”

| Layer | Truth |
|-------|--------|
| Dual-model is working | Empty `items` / `results` is **correct**. Pins are not governed evidence/conclusions. |
| T263 F8 shipped | JSON overlay sets `next_step` = `PROGRESSIVE_RECALL_FALLBACK` (`governed_common.rs:54–84`). Agents still get an **ellipsis**, not a command. |
| T263 F8 parenthetical | Spec allowed “or that string + vault pins are not governed evidence”. **Not implemented.** This track promotes copy-paste query + `Pinned: N`. |
| T243 F5 shipped | Progressive `next_step` is the **same ellipsis const**. Operator query is discarded. |
| T263 F9 parked progressive | “Leave T243. No second SOOT.” **This track reopens** granted-empty progressive `next_step` **string growth** (same helper; not a second const family). Deny stderr **keeps** the ellipsis const. |
| T275 is not the hole | Live grants **3 of 3**; packets are `denied: false` / exit **0**. |
| T288 is not this surface | Briefing JSON extras `vault_pin_count` stay project-briefing-only. Lists DTOs have **no** `next_step` field — CLI overlay. Progressive DTO **has** optional `next_step` — grow the string, no new keys. |
| Human has no remediator | `emit_list` human empty is a single `(none)` line (`evidence.rs:300–301`, `source.rs:299–300`, `review.rs:179–180`). |
| Default format | clap `default_value = "json"` on list (`main.rs:1955` / `:2035` / `:2160`). Agents see JSON. Progressive has **no** `--format` (always JSON). |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| Overlay helper | `governed_common.rs` `apply_authorized_empty_list_next` `:60` | Sets `next_step` iff empty `items`, not denied, no existing key. Uses `PROGRESSIVE_RECALL_FALLBACK` `:54`. **Change signature** to pass `pin_count: Option<u64>`. |
| Fallback const | `PROGRESSIVE_RECALL_FALLBACK` `:54` | Exact `Ungoverned vault search: ai-brains recall "…"`. **Freeze** for **deny stderr** (`governed_query.rs:133`). Do **not** change the const text. |
| Progressive hints | `governed_query.rs` `apply_progressive_search_hints` `:63` | Denied: append const to `denial_hint`. Empty allowed: `next_step = Some(PROGRESSIVE_RECALL_FALLBACK)`. **Empty arm uses new formatter.** |
| List emit | `evidence.rs` `emit_list` `:289`; `source.rs` `:288`; `review.rs` `:168` | JSON overlay; human `(none)`. **Pass pin_count; human second line.** |
| Local COUNT | `run_list_local` already has `ScopeRef` + `ctx.conn` | evidence **`:202`** (OpenCode cited `:196` — that is **source.rs** `run_list_local`). `ScopeRef::Repository(pid)` → `count_pinned_memories(Some(pid)).ok()`. Personal/Workspace → `None`. |
| QueryStore trait | `store/src/lib.rs:39` trait; **impl** `query_store.rs:135` **for `VaultConnection` only** (not `SqliteEventStore`) | Four callers today `use ai_brains_store::SqliteEventStore` only (`evidence.rs:23`, `source.rs:22`, `review.rs:23`, `governed_query.rs:17`). T288 analog `briefing.rs:21` already imports `QueryStore`. **Must** `use ai_brains_store::QueryStore;` in those four — **not** in `governed_common.rs` (OpenCode m1). `ProjectId` is `Copy` (`ids.rs:7`). |
| Default read path | `choose_read_path` `:372` / default Local `:388–389` | Manual AC12 `cargo run` (no `--daemon`) hits local COUNT. |
| Daemon list | `emit_list(format, &list)` **no ctx** | `pin_count = None` (copy-paste query, no `Pinned:`). CLI local is DoD. |
| Progressive COUNT | `run_progressive` has `project_id` + `ctx.conn` `:89` | `count_pinned_memories(Some(&project_id)).ok()`. |
| COUNT SQL | `query_store.rs:699–715` T214 | `mp.project_id = ?` only. **Reuse; no new store method.** |
| List DTOs | `EvidenceListResponse` `briefings.rs:629`; `SourceListResponse` `sources.rs:28`; `ReviewQueueResponse` `review.rs:25` | **No** `next_step` field. Overlay on `Value`. **Do not add fields.** |
| Progressive DTO | `ProgressiveQueryResponse` `:418` | Optional `next_step` already (`skip_serializing_if`). **Grow string. No new keys.** |
| Hermetic lists | `governed_vault_pin_honesty.rs` AC7 `:249–319` | Asserts `contains("recall")` + empty items. **Stays green** if string grows. Additive ACs for `Pinned:` / copy-paste query. |
| Hermetic progressive | `governed_first_run_deny_exit.rs` `:238–247` | `contains("recall")` only. **Stays green.** Additive needle + `Pinned:`. Helper `progressive_cmd` `:83–93` hardcodes query **`"x"`** — AC6 must **not** reuse it (OpenCode m2). |
| Unit overlay | `governed_common.rs` `:684` / `:704` | Empty sets recall; nonempty/denied omit. **Update call signature.** |
| clap lists | `main.rs` evidence List `:1945`; source `:2028`; review `:2153` | `--format` default **json**. **No new flag.** |
| clap progressive | `GovernedQueryCommands::Progressive` `:1865` | `query: String`; `--project-id`; `--limit` 16; `--dry-run` true. **No `--format`.** |
| evidence search | `main.rs:3987` → `run_list` with `query: Some` | Same `emit_list`. Evidence `--query` is **FTS over evidence summaries** — **not** the recall needle (F32). |
| Hotspots | `project.rs` #1 / `governed_common.rs` #5 / `personal.rs` #7 | Isolation. |

### 2.4 Deps / pins (researched 2026-08-23 — snapshot, re-verify at execute)

| Item | Workspace / lock | crates.io / upstream (this pass) | Decision |
|------|------------------|----------------------------------|----------|
| clap | Cargo.toml `4.5`; lock **4.6.1** | crates.io **4.6.6** (2026-08-06). GitHub clap-rs latest **v4.6.6**. **No clap 5.** | **No bump** |
| rusqlite | **0.39.0** SQLCipher | crates.io **0.40.2**. Dependabot `#61`. | **No bump** |
| serde_json | lock **1.0.150** | crates.io **1.0.151** (prior track snapshot) | **No bump** |
| chrono | lock **0.4.44** | crates.io **0.4.45** (`#62`) | **No bump** |
| tokio | lock **1.52.3** | crates.io **1.53.1** (`#59`) | **No bump** |
| thiserror | lock **2.0.18** | Dependabot `#60` **2.0.20** | **No bump** |
| uuid | lock **1.23.1** | not this track | **No bump** |
| rustc / nextest / workspace | **1.95.0** / **0.9.140** / **0.1.2** | — | Freeze |
| Zero new crates | Required | — | No regex crate; sanitize in std |

### 2.5 Online / product research

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — **Human-first**; “Changing output for humans is usually OK”; scripts use `--json` for stability | **F2** human second line; **F3** JSON `next_step` **string growth** (existing overlay key), not new required keys. |
| clig — “Make it easy to see the current state”; “suggest commands they can run next” | `Pinned: N` is state; copy-paste `ai-brains recall "…"` is the next command. Ellipsis is **not** a suggestion they can run. |
| clig — distinguish no-access vs empty collection | Denied lists stay exit **3** + bootstrap (T221). Granted-empty is **empty governed collection**, not empty vault. |
| T180 `PROTOCOL-COMPAT.md` §3.1 / §4 | Prefer **additive optional** / ignore-unknowns. List DTOs stay unaugmented. Progressive `next_step` already optional. **Do not** add `vault_pin_count` on lists (T288 keys stay briefing-only). |
| T263 F8 | Overlay on serialized `Value`; daemon/HTTP DTOs need not change. **Affirm.** Parenthetical pin-hint **this track**. |
| T243 F5 | Authorized empty sets `next_step`. T221 F14 empty ≠ deny **unchanged**. String contents **grow**. |
| T288 analog | COUNT is `count_pinned_memories` (not GLOB). Fail-open collect. Dual-model: never scrape pins into `items`/`results`. |
| Nielsen / clig empathy | Empty governed + nonempty vault is a third state (T288 labeled it on briefing). Lists/progressive label it in `next_step`. |
| N/A | SQLCipher page encrypt, schtasks, llama.cpp `/health`, FTS5 ranking (COUNT is projection SQL), clap 5 (not released). |

**Could not verify:** exact live `count_pinned_memories(Some(3581317d))` without vault SQL (do not print `AI_BRAINS_KEY`). Preflight Pinned **3908** is the Manual inventory number (volatile; T288 OpenCode m1: do **not** require equality to `memory list --summary`). Hermetic unique needle is SoT for `Pinned:` ≥ 1.

**ledgerful / ai-brains:** `preflight --summary` Pinned **3908**. Dogfood lists/progressive ellipsis + human `(none)`. `ledgerful search "apply_authorized_empty_list_next"` → `governed_common.rs:60` + three emit sites + units. `search "count_pinned_memories"` → `query_store.rs:699` + T288 `briefing.rs:57`. Semantic recall of T263 F8 is plan-audit chrome — live src is SoT.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a **FEATURE** TX. |
| **F1 — Dual-model / no H2** | Never inject pin text, pin ids, or `MemoryPinned` rows into `items[]` / `results[]`. Empty arrays stay. No `classify_legacy` / `decision propose` / auto-approve / live `migrate governed`. T227 F3 / T263 F3 / F11 stand. |
| **F2 — Human second line (lists)** | When format is Human/Markdown **and** `items` empty **and** not a deny emit: after the existing `(none)` line, print `format_authorized_empty_next(pin_count, None)` on stdout. Do **not** change the `(none)` strings. Nonempty lists: no next line. |
| **F3 — JSON `next_step` string growth** | Same gate as T263 F8. `next_step` value becomes `format_authorized_empty_next(pin_count, query)`. **E1:** omit key when overlay off (denied / nonempty / error envelope). Never `null`. No new keys (`vault_pin_count` / `vault_pin_previews` stay T288 briefing-only). |
| **F4 — Count is inventory, fail-open** | `n` = `count_pinned_memories(Some(&project_id))` (T214). **Some(n)** including **0**. `Err` / Personal / Workspace / daemon-without-conn → `None` (copy-paste query, **no** `(Pinned: …)` suffix). **Do not** COUNT GLOB. **Do not** use `count_memories` session-join. AC12 Manual is **nonzero** on live, not equality to 3908. |
| **F5 — Copy-paste query** | Lists / evidence search: needle **`what did we decide`** (`LIST_RECALL_QUERY` const). Progressive granted-empty: operator `options.query` after sanitize. **Do not** use evidence `--query` as the recall needle (wrong corpus). |
| **F6 — Sanitize** | Trim; collapse ASCII whitespace (incl. `\n`/`\r`/`\t`) to single spaces; replace `"` with `'`; char-truncate **80**; empty after trim → `LIST_RECALL_QUERY`. Std only. No panic. Output of `format_authorized_empty_next` is **one line** (`!contains('\n')`) even when the raw query had newlines (Agy m2). |
| **F7 — Exact granted-empty shape** | `Ungoverned vault search: ai-brains recall "{needle}"` + optional ` (Pinned: {n})` when `Some(n)`. Prefix matches T243 family. Suffix space-paren like T288 `Pinned: {n}` for grep. **Single line.** Unit AC1 uses **exact** `assert_eq!` of that shape (OpenCode O1), not only `contains`. |
| **F8 — Fallback const frozen** | Do **not** edit `PROGRESSIVE_RECALL_FALLBACK` text. Denied progressive **stderr** third line stays the ellipsis (`governed_query.rs:133`). Denied `denial_hint` append stays the const (`:74`). |
| **F9 — Denied / nonempty frozen** | Overlay **off**. T221/T275 list deny exit **3** + bootstrap. Progressive deny exit **3**, `next_step` omitted. T263 AC8 stays. |
| **F10 — Progressive JSON-only** | No `--format human` on `query progressive`. CAPABILITIES “json / json / No TTY flip” frozen. T292 is `policy check` human — **not stolen**. |
| **F11 — DTO freeze** | **Do not** add fields to `EvidenceListResponse` / `SourceListResponse` / `ReviewQueueResponse` / `ProgressiveQueryResponse`. Desktop TS `next_step?: string` already optional — string growth is fine. |
| **F12 — File growth** | Formatter + overlay signature in `governed_common.rs` (hotspot #5 — **small**: helper + sanitize + signature; **no** `QueryStore` import). Callers: `evidence.rs` / `source.rs` / `review.rs` `emit_list` + local COUNT; `governed_query.rs` empty arm. **Required:** `use ai_brains_store::QueryStore;` in those **four** files (OpenCode m1; trait is on `VaultConnection` only — `query_store.rs:135`). Units in `governed_common.rs`. Hermetic additive in `governed_vault_pin_honesty.rs` + `governed_first_run_deny_exit.rs`. **Do not** grow `project.rs`, `personal.rs`, `preflight.rs`, `briefing.rs`, `query_store.rs`, `ranking.rs`, `pin.rs` write, `.github/workflows/ci.yml`. |
| **F13 — Helper names** | Required: `LIST_RECALL_QUERY`, `format_authorized_empty_next(pin_count: Option<u64>, recall_query: Option<&str>) -> String`, `sanitize_recall_query(raw: &str) -> String` (`pub(crate)` or `pub` in `governed_common.rs` — CLI-only; **not** control-plane). `apply_authorized_empty_list_next(value, pin_count)`. |
| **F14 — Daemon overlay** | Daemon list path: `pin_count = None` (no local COUNT). Still copy-paste default query (better than ellipsis). Do **not** overlay in `ai-brainsd` / HTTP this track (T243 F24 analog). |
| **F15 — Reuse T214 COUNT** | Call existing `count_pinned_memories`. **No** new `QueryStore` method. **No** `list_authority_memories` samples on lists (that is T288 briefing). |
| **F16 — last-PR Cursor** | #205 empty → **N/A**. Dependabot remotes not this track. **No T301.** |
| **F17 — PATH** | Do not `cargo install` unless the user asks. Tests/manual AC use `cargo run` / hermetic. |
| **F18 — Live vault pin** | Do **not** pin production DECISIONs as implement. Hermetic unique needle is SoT for `Pinned:` ≥ 1. Manual unique canary allowed on go. |
| **F19 — Capture independence** | Overlay + COUNT only. No models, embeddings, graph, ledgerful, new events. Progressive `--dry-run` default true stays. |
| **F20 — Tests** | Naming `function_or_feature__condition__expected_result`. `tempfile::tempdir` per hermetic. **AC1/AC4 required red** units. **AC14 rstest** sanitize/count cases. Hermetic pin: `hermetic_cmd` / `hermetic_cmd_with_ids` (T288 OpenCode m2 analog). |
| **F21 — Cross-model** | FEATURE (operator remediator + JSON string growth). After Phase-1 clean, run read-only `codex-review`. |
| **F22 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F23 — Decline peers** | T288 briefing Completed; T289 Personal Completed; T291 trace; T292 policy-check human; T293 neighbors; T294 leftover; T299 forget-list. |
| **F24 — Decline H2 / F2 / pins** | T263 H2; T240 F2; clap 5; rusqlite 0.40; chrono 0.4.45; no new crates; workspace **0.1.2**. |
| **F25 — Docs** | CAPABILITIES discovery Empty row + progressive granted-empty sentence: copy-paste recall + `Pinned: N`. CHANGELOG T290. list/progressive `after_help` one sentence each. CLI-EXIT-CODES **and** OPERATIONS: authorized-empty lists/progressive **exit 0** + informative `next_step` (recall + pin count) — Agy O1 already this row / AC10. PROTOCOL-COMPAT: CLI overlay **string growth**; daemon/HTTP DTOs **unaugmented**. |
| **F26 — PowerShell** | `;` not `&&`. |
| **F27 — No 140-cap steal** | T263 F29 ≤140 is **briefing** const. List/progressive `next_step` may exceed 140; needle cap **80** keeps the line bounded. Do **not** shorten `PROGRESSIVE_RECALL_FALLBACK`. |
| **F28 — Existing tests stay green** | T263 AC7 `contains("recall")`; T263 AC8 denied no `next_step`; T221/T243 progressive deny omit `next_step`; first-run authorized empty `contains("recall")`. Overlay is additive (`Pinned: 0` / copy-paste query do not break those asserts). |
| **F29 — 0-pin honesty** | Granted-empty **0** pins: still copy-paste query + `(Pinned: 0)` when COUNT Ok. Never fabricate a DECISION preview. |
| **F30 — T266 lists default json** | clap default `json` frozen. Human is opt-in `--format human`. |
| **F31 — Isolation hotspots** | Do not edit `project.rs` / `sync.rs` / `forget.rs` production / `context.rs` / `claude_hook.rs` / `codex_hook.rs` / `briefing.rs`. |
| **F32 — evidence `--query` ≠ recall needle** | `evidence list --query` / `evidence search` FTS is governed evidence summary, not vault recall. Overlay needle stays `LIST_RECALL_QUERY`. |
| **F33 — `apply_progressive_search_hints` signature** | Pass `pin_count: Option<u64>` and `query: &str` into the empty arm. Denied arm **unchanged**. |
| **F34 — Nonempty progressive** | Hits omit `next_step` (T243). Do not append Pinned when `results` nonempty. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit **exact** `assert_eq!` (OpenCode O1): `format_authorized_empty_next(Some(12), None)` **equals** `Ungoverned vault search: ai-brains recall "what did we decide" (Pinned: 12)`; `format_authorized_empty_next(None, None)` **equals** `Ungoverned vault search: ai-brains recall "what did we decide"`. Both `!contains('\n')` (Agy m2) and `!contains('…')` (U+2026). **Required red.** |
| **AC2** | CLI hermetic: discovery grants only (0 pins) → `evidence list --format json --local` exit **0**; `items: []`; `next_step` contains `recall` **and** `what did we decide` **and** `(Pinned: 0)`. Same for `source list` and `review list` (shared helper or three calls). **Required red** (at least evidence). |
| **AC3** | Same 0-pin fixture `--format human` for **all three** nouns (Agy m1): stdout contains `evidence: (none)` / `sources: (none)` / `review items: (none)` **and** a following line with `recall` + `what did we decide` + `Pinned: 0`. Exit **0**. Do **not** change the `(none)` strings. |
| **AC4** | Unit sanitize rstest (AC14 / Agy O2): `"  foo\nbar  "` → `foo bar`; `"foo\tbar"` → `foo bar`; `say "hi"` → `say 'hi'`; empty/`"   "` → `what did we decide`; 81 `'a'` chars → 80 chars; no panic. After sanitize, `format_authorized_empty_next(Some(0), Some(raw_with_newline))` still `!contains('\n')`. **Required red.** |
| **AC5** | Hermetic: grants + `hermetic_cmd` pin `DECISION: {needle}` → `evidence list --format json` `next_step` contains `(Pinned:` with a **nonzero** digit; `items` still `[]`; pin text **not** in `items`. Exit **0**. |
| **AC6** | Hermetic progressive after bootstrap, query `what did we decide about SQLCipher` (or unique needle): `denied: false`, `results: []`, `next_step` contains `recall` **and** `SQLCipher` (or needle) **and** `Pinned:`; does **not** use U+2026 ellipsis as the quoted query. Exit **0**. **Do not** reuse `progressive_cmd` (`governed_first_run_deny_exit.rs:83` hardcodes `"x"` — OpenCode m2). Inline the argv or add `progressive_cmd_query(vault, query)`. |
| **AC7** | T263 `evidence_list__authorized_empty__next_step_names_recall` (and source/review) **stays green** (`contains("recall")`). |
| **AC8** | T263 AC8 denied lists **stay green** (exit **3**, no authorized-empty `next_step`). |
| **AC9** | Unit: `apply_authorized_empty_list_next` nonempty items / `denied: true` / `code: POLICY_DENIED` still omit `next_step`. Denied progressive stderr still prints exact `PROGRESSIVE_RECALL_FALLBACK` (ellipsis). |
| **AC10** | Docs: CAPABILITIES Empty + progressive granted-empty sentences + list/progressive after_help one sentence + CLI-EXIT-CODES **and** OPERATIONS authorized-empty **exit 0** + `next_step` recall/Pinned (Agy O1) + PROTOCOL-COMPAT overlay string growth + CHANGELOG T290. |
| **AC11** | No new crate. No clap 5. No `unwrap`/`expect`/`panic` in production. `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` clean on go. |
| **AC12** | Manual (source/hermetic bin, not PATH): the four placeholder commands — `evidence list --format json`; `source list --format json`; `review list --format json`; `query progressive "what did we decide about SQLCipher"`. Each: `denied` false / absent, arrays `[]`, `next_step` contains `recall` **and** (`Pinned:` with **nonzero** **or** the progressive needle). Human lists `--format human` show the same next line. Exit **0**. Live `N` is `count_pinned_memories(Some)` (volatile; plan-time ~3908) — **not** required equal to preflight 3908. |
| **AC13** | `serde_json::to_value` of default `EvidenceListResponse` / `SourceListResponse` / `ReviewQueueResponse` / `ProgressiveQueryResponse::new` has **no** `vault_pin_count` key and **no** new required fields. |
| **AC14** | rstest `#[case]` for sanitize (AC4) **and** overlay-gate: denied skip / nonempty skip / empty apply. **Required.** |
| **AC15** | Unit: `PROGRESSIVE_RECALL_FALLBACK` **exact** ellipsis string **unchanged** (T243 deny stderr lock). |
| **AC16** | Progressive with nonempty `results` fixture (if one exists) or unit: `next_step` omitted. Do not append Pinned on hits. |
| **AC17** | Contracts golden `progressive_query_response.json` still parses (field omitted when None). |

---

## 5. Design notes

### 5.1 Why `next_step` string, not T288 keys

T288 froze briefing `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` (≤140) so COUNT had to be **extra JSON keys**. List `next_step` is already a CLI overlay string (not a DTO field). Progressive `next_step` is already an optional DTO string. T180 + placeholder: **grow the string**. Do not mint `vault_pin_count` on lists.

### 5.2 Shared formatter (hotspot #5)

```text
pub const LIST_RECALL_QUERY: &str = "what did we decide";

pub fn format_authorized_empty_next(
    pin_count: Option<u64>,
    recall_query: Option<&str>,
) -> String

pub fn apply_authorized_empty_list_next(
    value: &mut serde_json::Value,
    pin_count: Option<u64>,
)
```

COUNT stays in `run_list_local` / `run_progressive` (`ctx.conn.count_pinned_memories`). Do **not** import `QueryStore` into `governed_common.rs`. The four callers **must** add `use ai_brains_store::QueryStore;` (OpenCode m1) — `count_pinned_memories` is a trait method on `VaultConnection` only (`query_store.rs:135`); today they import `SqliteEventStore` alone. Analog: T288 `briefing.rs:21`.

### 5.3 Daemon

Daemon `EvidenceList` / `SourceList` / `ReviewList` responses have no pin count. CLI overlay still replaces ellipsis with copy-paste `what did we decide`. Local path is Manual/hermetic DoD.

---

## 6. Non-goals

- Pin → Approved (H2)
- Fabricate evidence / source / review / progressive hits
- T288 `vault_pin_*` keys on lists
- T289 Personal markdown
- `query trace` wrap (`null` stays T291)
- `policy check` human (T292)
- Graph neighbors (T293)
- Leftover upsert (T294)
- Forget-list empty (T299)
- Progressive `--format human`
- New clap flags / clap 5 / rusqlite 0.40
- DTO new fields
- `cargo install` / `.env` write / live extra bootstrap
- Growing `briefing.rs` / `personal.rs` / `preflight.rs` / `query_store.rs`

---

## 7. Verification plan (TDD)

**Red first:**

1. `format_authorized_empty_next__with_count__includes_pinned_and_copy_paste` (AC1)
2. `sanitize_recall_query__cases__expected_needle` (AC4 / AC14)
3. `evidence_list__authorized_empty__next_step_names_pinned_and_query` (AC2)
4. `query_progressive__authorized_empty__next_step_contains_query_and_pinned` (AC6)

Then human AC3; pin AC5; stay-green AC7/AC8/AC9/AC15/AC17; docs AC10; Manual AC12.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Agents parse exact ellipsis | AC1/AC6 require copy-paste; CAPABILITIES sentence |
| `Pinned: 0` looks like a lie on live | COUNT fail-open; Manual AC12 nonzero on this vault; hermetic AC2 is 0-pin honesty |
| Growing hotspot `governed_common.rs` | F12: formatter only; no store import; COUNT in callers |
| Quote injection in next_step | F6 replace `"` |
| T288 key creep | F3 / AC13 no `vault_pin_count` on lists |
| Deny stderr drift | F8 / AC15 const exact |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit lists/progressive granted-empty U=6 | **Absorb** F1–F7 / AC1–AC6 / AC12 |
| Placeholder Manual four commands | **Absorb** AC12 |
| T263 F8 parenthetical pin hint | **Absorb / promote** F3 / F7 |
| T263 F9 leave T243 ellipsis | **Partial reopen** F5/F7 granted-empty string; **affirm** F8 deny const |
| T243 F5 `next_step` overlay | **Affirm gate**; **grow** contents |
| T214 `count_pinned_memories` | **Reuse** F4 / F15 |
| T288 briefing stanza / overlay keys | **Decline** F3 / F23 — Completed project-only |
| T289 Personal `_None_` | **Decline** — Completed |
| T291 `query trace` `null` | **Decline → T291** |
| T292 `policy check` human | **Decline → T292** |
| T293 neighbors dump sessions | **Decline → T293** |
| T294 leftover dest-missing | **Decline → T294** |
| T299 forget-list empty | **Decline → T299** |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F24 |
| last-PR Cursor #205 | **N/A** empty — **no T301** |
| Identity leftover `7d97a456` | **Not this track** — T258 / T294 |
| Open T291–T300 except this | **Not related** except named declines |
| Closed T274–T289 | **Stay closed** |

---

## 10. Implement order (on go)

1. Phase 0 re-verify HEAD / deferred / #205 still empty / live lists+progressive read-only
2. FEATURE TX
3. Red AC1/AC4
4. Green: formatter + sanitize + `apply_authorized_empty_list_next` signature in `governed_common.rs`
5. Red/green AC2/AC3/AC5 list emit + COUNT; AC6 progressive
6. Stay-green AC7–AC9 / AC15–AC17
7. Docs AC10
8. Clippy + nextest + deny/audit
9. Manual AC12
10. Phase-1 review → codex-review
11. Publish: push `track/T290-*` → PR → watch GHA `CI` green → squash-merge → prune

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| PATH until `cargo install` | F17 |
| Daemon list no `Pinned:` | F14 |
| Personal/Workspace list COUNT skipped | F4 |
| Live GLOB 0 samples | T288 residual; this track COUNT not GLOB |
| `query trace` `null` | T291 |
| T288 PATH-behind stanza | Not this track |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/governed_common.rs` | `LIST_RECALL_QUERY`; `sanitize_recall_query`; `format_authorized_empty_next`; overlay signature + units AC1/AC4/AC9/AC14/AC15 |
| `crates/ai-brains-cli/src/commands/evidence.rs` | `use ai_brains_store::QueryStore;` (OpenCode m1); local COUNT; `emit_list(..., pin_count)`; human second line. `run_list_local` **`:202`**. |
| `crates/ai-brains-cli/src/commands/source.rs` | Same (`run_list_local` `:196`) |
| `crates/ai-brains-cli/src/commands/review.rs` | Same; human empty stays `review items: (none)` then next line (Agy m1 / AC3) |
| `crates/ai-brains-cli/src/commands/governed_query.rs` | `QueryStore` import; empty-arm formatter + COUNT; deny stderr const frozen |
| `crates/ai-brains-cli/tests/governed_vault_pin_honesty.rs` | Additive AC2/AC3/AC5 |
| `crates/ai-brains-cli/tests/governed_first_run_deny_exit.rs` | Additive AC6 |
| `crates/ai-brains-cli/src/main.rs` | after_help one sentence on list + progressive |
| `Docs/CAPABILITIES.md` | Empty + progressive granted-empty |
| `Docs/CLI-EXIT-CODES.md` | Authorized-empty progressive sentence |
| `Docs/OPERATIONS.md` | empty-vs-deny one-liner |
| `Docs/PROTOCOL-COMPAT.md` | CLI overlay string growth; DTOs unaugmented |
| `CHANGELOG.md` | T290 (on go) |

**Do not touch:** `briefing.rs`, `personal.rs`, `project.rs`, CLI `preflight.rs`, `query_store.rs`, contracts struct fields, `ci.yml`.

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` + `opencode-review.md` at HEAD `efdfd3d`. Live verify: `apply_authorized_empty_list_next` `:60`; `PROGRESSIVE_RECALL_FALLBACK` `:54`; `choose_read_path` default Local `:388–389`; evidence `run_list_local` **`:202`** (OpenCode `:196` is source.rs); review human `review items: (none)` `:180`; four callers `SqliteEventStore` only; `QueryStore` impl `VaultConnection` `query_store.rs:135`; `progressive_cmd` `:83` query `"x"`; `hermetic_cmd` `:165`; clap progressive no `--format` `:1865`. Pins **snapshot — re-verify at execute** (clap lock 4.6.1 / crates.io 4.6.6; rusqlite 0.39.0; no clap 5).

### Pins locked by fold-in

1. **F12 / §5.2 (OpenCode m1):** `use ai_brains_store::QueryStore;` in `evidence.rs` / `source.rs` / `review.rs` / `governed_query.rs` — **not** `governed_common.rs`. Trait is on `VaultConnection` only.
2. **AC6 (OpenCode m2):** do **not** reuse `progressive_cmd` (`:83` hardcodes `"x"`); inline or `progressive_cmd_query`.
3. **AC1 (OpenCode O1 + Agy m2):** exact `assert_eq!` F7 shape; `!contains('\n')`; `!contains('…')`.
4. **AC3 (Agy m1):** human empty parity for evidence **and** source **and** review (`review items: (none)`).
5. **AC4 (Agy O2):** tab case + newline round-trip still single-line formatter.
6. **Already:** Agy O1 CLI-EXIT-CODES/OPERATIONS (F25/AC10); OpenCode O2 progressive JSON-only (F10).

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** review human `(none)` + next line parity | **Folded** AC3 / F2 |
| Agy | **m2** formatter single-line even with newline query | **Folded** F6 / F7 / AC1 / AC4 |
| Agy | **O1** CLI-EXIT-CODES + OPERATIONS exit 0 + next_step | **Already** F25 / AC10; **tightened** AC10 wording |
| Agy | **O2** sanitize rstest multi-line / tab / quotes / empty / 80 | **Already** AC4/AC14; **folded** tab + formatter `!contains('\n')` |
| OpenCode | B / M | None filed |
| OpenCode | **m1** `QueryStore` import in four callers | **Folded** F12 / §2.3 / §5.2 / touch map |
| OpenCode | **m2** AC6 not `progressive_cmd` `"x"` | **Folded** AC6 / §2.3 |
| OpenCode | **O1** exact `next_step` `assert_eq!` | **Folded** AC1 (unit exact; hermetic AC2 stays 0-pin contains/`Pinned: 0`) |
| OpenCode | **O2** progressive no `--format` | **Already** F10 |
| both | last-PR #205 Cursor | **Affirm N/A** — no T301 |
| both | deferred T288/T289/T291–T299 / H2 | **Affirm** |

No Blockers. No Majors. No new placeholder minted. Do **not** edit `*-review.md`.
