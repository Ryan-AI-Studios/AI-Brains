# T287 — `memory list` first page must include pins, not only just-now ingest

- **Track ID:** T287-MemoryListAuthority
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE / UX
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `memory list --limit 5` **8/6** (onboarding / review-track “just now”). Placeholder minted with T285–T300 (`76c4db9`). T274 F13 froze recency — **reopen as this track**. T285/T286 Completed (`#201`/`#202`) and **did not steal** list ORDER.
- **Depends on:** T216 ✅ recency ORDER + inventory skim (lift, do not silently drop recency); T274 ✅ F13 declined mix — **this track is the reopen**; T283 ✅ human-only permute / JSON freeze analog; T285 ✅ `classify_pin_kind` / `first_contentful_line`; T286 ✅ Index TAGS-or-GLOB (do **not** reopen Index)
- **Blocks / feeds:** Operators using `memory list` as inventory see `DECISION:` pins on the first human page. Briefing granted-empty remains **T288**. `graph neighbors` CLI **T293**. Empty `forget --list-forgotten` next **T299**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “`memory list` just-now ingest”; T274 F13 / T285 F14 / T286 F15 / T286 AC15 “list ORDER is T287”; live hole: default page and `--limit 5` are harness ingest (`## Objective`, review-track)
- **Not absorbed (DoD):** T288 briefing stanza; T293 neighbors CLI; T299 forgotten-empty next; T294 leftover upsert; T216 `--status` / limit 50 / JSON keys; T274 `forget --match` unfiltered; T263 H2 pin→Approved; T240 F2; clap 5 / rusqlite 0.40
- **Research date:** 2026-08-23 (plan dogfood HEAD `360139d` T286 `#202`; product `src/` = T286 Index + T216 list recency; PATH **0.1.2** 2026-08-22 19:41 **without** T285/T286 — list hole is in **source and PATH**)
- **AI fold-in:** none yet (plan pass). Disposition after `/fold-in 287` → **§13**.
- **Ledger:** planning DOCS TX `673e7322-b68f-40dd-bd34-6a91a83e7412`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** pin production decisions to the live vault as implement (hermetic needle is SoT; Manual DoD unique canary is allowed on go). Do **not** rewrite `.env`. Do **not** grow hotspot `project.rs` / `sync.rs` / `forget.rs` **production** / CLI `preflight.rs` / `session_chrome.rs` / `ranking.rs` / `pin.rs` write. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Human first page includes pins.** `ai-brains memory list` (default `--status pinned`, including `--limit 5`) prefer-fills leading-line authority pins (`DECISION:` / `CONSTRAINT:` / `INVARIANT:` / `HOTSPOT:` after the T285 envelope) when the scoped vault has any, then recency-fills the rest. It must not be five just-now `## Objective` / review-track ingest rows when thousands of pins exist.
2. **JSON and store recency stay honest for scripts.** `--format json` `items[0]` stays newest-`updated_at` (T216 F7 / T283 analog). `QueryStore::list_memories` `ORDER BY mp.updated_at DESC, mp.memory_id ASC` stays. `--summary` counts stay T216.
3. **Keep T216 contracts.** Default limit **50** / max **200**. `--status forgotten` / `forget --list-forgotten` stay recency. Exit **2** without project and without `--global`. JSON keys frozen. Tag two-stage stays.
4. **North star.** Capture independence: inventory selection + display previews only. No new events. No hidden CoT. Operators who run `memory list --limit 5` must not conclude this repo has no decisions because ingest recency buried every pin.

This unblocks daily inventory: T216 shipped recency skim; T274/T285/T286 ranked recall and Index. `memory list` is still “what just landed,” not “what we pinned.” `--summary` Pinned **3751** is honest; the table is not.

---

## 2. Live baseline (re-scan 2026-08-23)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `360139d` T286 squash `#202`. Tree **CLEAN**. `origin/main` = HEAD. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. Has T274. **Does not have T285/T286.** List hole is in **source + PATH** (T286 did not touch list). **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `memory list --limit 5` | Scope `3581317d`. Five recency rows: `## Objective` / review-file ingest. **0** `DECISION:` previews. Footer `Showing 5 of 3751`. |
| `memory list` (default limit 50) | Fifty recency rows: onboarding, `/review-track`, `## Objective`, agent “Now let me verify…”. **0** `DECISION:` in the first page. |
| `memory list --summary` | `Pinned: 3751` / `Forgotten: 0`. Counts honest. Table is the hole. |
| `memory list --format json --limit 1` | `api_version=1`, `status=pinned`, `total=3751`, `more_available=true`. `items[0].preview` is recency chrome (`insert_event_row` verify prose) — **not** a pin. Keys match T216 F10. |
| `--help` | Default **`--limit` 50** (max 200), `--status pinned`. Placeholder text “Default `--limit 5`” is **false** vs live clap — F11 freeze 50; DoD uses `--limit 5`. |
| Last GitHub PR | [#202](https://github.com/Ryan-AI-Studios/AI-Brains/pull/202) T286 (2026-08-23). `gh pr view --json comments,reviews` **empty**; issue comments **0**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081 unreachable). **0 pending / 0 drift** at scan. Hotspot **#1** `project.rs` (3.986) — **do not touch.** `sync.rs` #2. `forget.rs` **#3** — **do not grow production** (list-forgotten already delegates to `memory.rs`). `session_chrome.rs` **#6** — **do not edit** (T286 freeze). CLI `preflight.rs` **#8** — **do not touch.** `memory.rs` / `query_store.rs` **not** top-10 — **extend here.** |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why the first page is ingest

| Layer | Truth |
|-------|--------|
| T216 list is recency | `query_store.rs` `:235–241`: `ORDER BY mp.updated_at DESC, mp.memory_id ASC LIMIT ?`. Status `pinned` includes **every** projection row (harness ingest is pinned, not forgotten). |
| Capture pins turns | Live first-50 previews are session dumps (`## Objective`, `# AI-Brains Onboarding`, review-track). Vault `status=pinned` ≠ leading-line `DECISION:`. |
| T274 F13 froze mix | “Default `memory list` stays `updated_at DESC`. **Not DoD** to pin-first.” T285 AC16 / T286 AC15 regression-locked that store ORDER. **This track lifts F13 for human pinned only.** |
| Preview can title `TAGS:` | `preview_line` (`memory.rs:32–39`) is first non-empty line + role strip. `pin --tag` stores `ASSISTANT: TAGS: …\nDECISION:` — same envelope hole T286 fixed for Index titles. Even a pin that leaked into the recency page would preview as `TAGS:`. |
| Over-fetching recency cannot find buried pins | 3751 rows; default page 50. Prefer-fill **must** query authority separately (T274 two-pass lesson). Partitioning the first recency page is **not** enough. |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| CLI run | `crates/ai-brains-cli/src/commands/memory.rs` `run_inventory` `:136` | One `list_memories` call; tag retain; JSON vs human emit. **Mix here** for human + `status=pinned`. |
| Preview | `preview_line` `:32` | First non-empty + `strip_role_prefix`. Lift to `first_contentful_line` (retrieval already a CLI dep). |
| JSON DTO | `MemoryListJson` `:73–84` CLI-local (T216 F22). Keys: `api_version`, `scope`, `project_id`, `status`, `items[]`, `returned`, `more_available`, `limit`, `total`. Item: `memory_id`, `preview`, `updated_at`, `project_id`. **No contracts freeze.** |
| clap | `main.rs` `MemoryCommands::List` `:2608–2630` | `--status` / `-l` / `--global` / `--format` / `--summary` / `--tag` / `--project-id`. **No new flag.** |
| Store list | `query_store.rs` `list_memories` `:228`; `memory_list_from_where` `:45` | Parameterized. Tag LIKE includes USER/SYSTEM `TAGS:` (T216). ORDER F7. |
| Filter | `MemoryListFilter` `store/src/lib.rs:185` | `status`, `project_id`, `tag`, `limit`. **Do not add fields** (every test literal breaks). New trait method instead. |
| Trait | `QueryStore::list_memories` only `VaultConnection` impl | Add `list_authority_memories`. |
| Classifier | `ranking.rs` `first_contentful_line` `:102`, `classify_pin_kind` `:122`, `PinKind` `:65`. `lib.rs` already `pub use`. | Shared classifier (stub isolation). **Do not edit ranking.rs.** |
| Index GLOB | `session_chrome.rs` `index_pass1_glob_sql` `:108` | T286 F2. **Do not edit.** Duplicate bind-free GLOB in store for `mp.content` (F27). |
| T216 ORDER unit | `store/tests/memory_list_inventory.rs` `list_memories__limit_plus_one__returns_extra_row_for_more_available` | Must **stay green**. |
| CLI hermetic | `cli/tests/memory_list_inventory.rs` | Pins `DECISION:` and asserts presence/keys — **does not** assert chrome-vs-pin order. Additive ACs. |
| `forget --list-forgotten` | `forget.rs` `:48` → `run_inventory` status forgotten | Mix **off**. Do not grow `forget.rs`. |
| Hotspots | `project.rs` #1 / `sync.rs` #2 / `forget.rs` #3 | Isolation. |

### 2.4 Deps / pins (researched 2026-08-23 — snapshot, re-verify at execute)

| Item | Workspace / lock | crates.io / upstream (this pass) | Decision |
|------|------------------|----------------------------------|----------|
| clap | Cargo.toml `4.5`; lock **4.6.1** | crates.io **4.6.6** (2026-08-06). GitHub clap-rs latest **v4.6.6**. **No clap 5.** | **No bump** |
| rusqlite | **0.39.0** SQLCipher | crates.io **0.40.2** (2026-08-08). Dependabot `#61`. | **No bump** |
| serde_json | lock **1.0.150** | crates.io **1.0.151** | **No bump** |
| chrono | lock **0.4.44** | crates.io **0.4.45** (`#62`) | **No bump** |
| uuid | lock **1.23.1** | not this track | **No bump** |
| rustc / nextest / workspace | **1.95.0** / **0.9.140** / **0.1.2** | — | Freeze |
| Zero new crates | Required | — | No comfy-table / regex |

### 2.5 Online / product research

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — **Human-first**; “Changing output for humans is usually OK”; scripts should use `--json` for stability | **F1** human mix; **F2** JSON recency freeze (T283 analog). Dual-truth documented in after_help, not a new flag. |
| clig — `--json` when tables break scripts; humans first | JSON keys T216 F10 frozen; preview string of a tagged pin may skip `TAGS:` (display honesty, not key change). |
| [SQLite GLOB](https://sqlite.org/lang_expr.html) — case-sensitive Unix glob; `*` any sequence | Store pass-1 uses **GLOB** literals (`DECISION:*`, `ASSISTANT: TAGS:*`, …) matching T274/T286. LIKE stays for T216 `--tag` only. |
| T283 human cwd-first / JSON size-desc | Precedent for human permute + JSON freeze. Copy the after_help sentence pattern. |
| T274/T286 two-pass prefer-fill | Pass-1 authority query, pass-2 recency fill, `classify_pin_kind != Other` retain. **Not** Elastic `function_score`. **Not** hard-exclude transcripts. |
| T285 F7 envelope GLOB | `TAGS:*` OR `ASSISTANT: TAGS:*` only. **Do not** add `USER:`/`SYSTEM:` TAGS GLOB (OpenCode L1 declined on T286). |
| N/A | SQLCipher page encrypt, schtasks, llama.cpp `/health`, FTS5 `bm25(title,body)` (list is projection SQL, not FTS), clap 5 (not released). |

**Could not verify:** exact COUNT of post-envelope leading-marker rows in `3581317d` without vault SQL (do not print `AI_BRAINS_KEY`). Hermetic unique needle + live Manual `--limit 5` are the proof.

**ledgerful / ai-brains:** `preflight --summary` Pinned **3750** / in-context 0/0/0 / word **237**; `memory list --limit 5` all chrome; `ledgerful ledger status --compact` 0 pending / 0 drift; `search "list_memories"` → `query_store.rs` + `memory.rs:177` + store tests; `scan --impact` CLEAN at `360139d`; hotspots `project.rs` #1 / `forget.rs` #3. Semantic recall of “memory list ORDER” still returns onboarding/review chrome (PATH 0.1.2 / ranking evidence) — not SoT for list mix.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `673e7322`. Implement starts a **FEATURE** TX. |
| **F1 — Human pinned prefer-fill** | When `--status pinned` and **not** `--format json` and **not** `--summary`: pass-1 authority rows (newest first), then pass-2 recency fill excluding pass-1 ids, cap `page_limit`. First human data row is an authority pin when the scoped vault has ≥1 injectable leading-marker pin. Chrome dumps **may** fill later slots. |
| **F2 — JSON recency freeze** | `--format json` uses **only** `list_memories` recency. `items[0]` is newest `updated_at` (tie `memory_id ASC`). **No** mix. **No** new keys (`mix`, `authority`, `sort`). T283 F2 analog. |
| **F3 — Store `list_memories` ORDER freeze** | `ORDER BY mp.updated_at DESC, mp.memory_id ASC` stays (T216 F7 / T274 F13 store / T285 AC16 / T286 AC15). **Do not** change that SQL. Mix is CLI (human) + a **new** store method (pass-1). |
| **F4 — `list_authority_memories`** | New `QueryStore` method. Same `memory_list_from_where` (status / project / tag LIKE) **plus** bind-free GLOB extra on **`mp.content` only** (no column arg — no ident injection). GLOB inner = T286 `index_pass1_glob_sql("mp.content")` shape: marker+HOTSPOT **OR** `TAGS:*` **OR** `ASSISTANT: TAGS:*`. Single `AND (` group. **Do not** stack two `AND (` requiring both. **Do not** add `USER: TAGS:*` / `SYSTEM: TAGS:*` (F29). Same `ORDER BY` + `LIMIT ?`. **Do not** add fields to `MemoryListFilter`. |
| **F5 — Retain `classify_pin_kind != Other`** | After pass-1 fetch, drop rows where `classify_pin_kind` is `Other` (TAGS-only / body without marker). **Hotspot stays.** **Forbidden** to switch to `is_authority_pin_content` (drops Hotspot). Import `ai_brains_retrieval::{classify_pin_kind, PinKind, first_contentful_line}` — CLI already depends on retrieval. **Do not edit** `ranking.rs`. |
| **F6 — Envelope preview** | `preview_line` uses `first_contentful_line` then existing char-safe truncate (80). Empty contentful → **fallback** to today’s first non-empty line after role strip (may be `TAGS:`). **Do not** invent `Untitled Memory` on list (Index-only T286 F4). Human **and** JSON previews share this helper (display honesty). No `unwrap`/`expect`/`panic`. |
| **F7 — Forgotten recency** | `--status forgotten` and `forget --list-forgotten` **do not** mix. Recency only. T299 owns empty next-step. |
| **F8 — Summary freeze** | `--summary` still COUNT only (T216 F11/F46). Ignores `--status`/`--limit`. **Do not** list pins in summary. |
| **F9 — No new clap flag** | No `--authority` / `--kind` / `--recency` / `--sort` / `--pins-only`. Silent human mix (T274 F15 / T283 F5 / T286 F17). Placeholder “or `--authority` default-on” **declined**. |
| **F10 — JSON keys freeze** | T216 F10 shape unchanged. Preview **values** may skip `TAGS:`. PROTOCOL-COMPAT: N/A (CLI-local; T216 F22). |
| **F11 — Limit freeze** | Default **50**, max **200** (`clamp_list_limit`). Placeholder “default limit 5” is **wrong** vs clap. Hermetic + Manual DoD use `--limit 5`. |
| **F12 — Tag two-stage** | `--tag` still SQL LIKE + Rust token match on **both** passes. Mix among tag-matching rows only. |
| **F13 — Scope / exit 2 freeze** | T216 F3/F44. Missing project without `--global` → `fail_usage` exit **2**. |
| **F14 — `forget --match` unfiltered** | T274 F18 stands. Do not two-pass mutation search. |
| **F15 — Decline peers** | T288 briefing; T293 neighbors; T299 forgotten-empty next; T294 leftover upsert. |
| **F16 — Decline H2 / F2 / pins** | T263 H2; T240 F2; clap 5; rusqlite 0.40; chrono 0.4.45; no new crates; workspace **0.1.2**. |
| **F17 — PATH** | Do not `cargo install` unless the user asks. |
| **F18 — Live vault pin** | Do **not** pin production DECISIONs as implement. Hermetic unique needle is SoT. Manual unique canary allowed on go. |
| **F19 — last-PR Cursor** | #202 empty → **N/A**. Dependabot remotes not this track. **No T301.** |
| **F20 — Capture independence** | SQL + pure formatters + existing classifier. No models, embeddings, graph, ledgerful, new events. **Do not rewrite** `pin.rs` stored shape. |
| **F21 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. `tempfile::tempdir` per hermetic. rstest optional for GLOB cases. |
| **F22 — Cross-model** | Inventory mix is FEATURE. After Phase-1 clean, run read-only `codex-review`. |
| **F23 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F24 — File growth** | Mix helper + `preview_line` in `memory.rs`. Optional tiny `prefer_fill_authority` unit in the same file (or `memory_list_order.rs` if `memory.rs` would grow ≥80 net lines — prefer **same file** first). Store GLOB + `list_authority_memories` in `query_store.rs` + trait in `lib.rs`. Hermetic in existing `memory_list_inventory.rs` (CLI + store). **Do not** grow `project.rs`, `sync.rs`, `forget.rs` production, `session_chrome.rs`, `ranking.rs`, `lexical.rs`, `pin.rs` write, CLI `preflight.rs`, `.github/workflows/ci.yml`. |
| **F25 — Docs** | CAPABILITIES Memory inventory: human pinned prefer-fills authority; JSON recency frozen. CHANGELOG T287. `memory list` after_help one sentence (T283 pattern). OPERATIONS one-liner. |
| **F26 — PowerShell** | `;` not `&&`. |
| **F27 — No shared GLOB helper** | Do **not** import `index_pass1_glob_sql` into store (store must not depend on retrieval). Do **not** extract a shared crate helper. Duplicate the bind-free `mp.content` GLOB extra in `query_store.rs`. Store unit locks the literal needles (`DECISION:*`, `TAGS:*`, `ASSISTANT: TAGS:*`, `HOTSPOT:*`). |
| **F28 — Existing tests stay green** | T216 CLI inventory suite; store `list_memories` recency unit; T216 JSON schema keys; empty `No pinned memories.`; forgotten share-backend; summary counts; tag exact-token; exit 2. |
| **F29 — USER/SYSTEM TAGS GLOB decline** | T285 F7 / T286 OpenCode L1. Default `pin` role is assistant. Re-trigger: live `--role user\|system` tagged pins miss **human** first page after ship. |
| **F30 — Dual-truth after_help** | Human pinned prefer-fills authority; JSON order unchanged (recency). Same class as T283 “human table puts the cwd path-owner first; JSON order unchanged”. |
| **F31 — Pass-2 fill stands** | Recency chrome **may** appear as rows 2+. Do **not** hard-exclude transcripts (T260 analog). |
| **F32 — Chrome-only vault** | Pass-1 empty after retain → today’s recency (no T207 lie). Pin-only vault → first page all pins. |
| **F33 — `more_available`** | After merge, truncate to `page_limit`. `more_available` if leftover merge candidates **or** `total > returned` (T216 footer `Showing N of T` stays). Pass-1 `LIMIT page_limit`; pass-2 `LIMIT page_limit+1`. |
| **F34 — T266 Family B** | Default stays human (including pipes). Mix applies to default + `--format human` only. |
| **F35 — Mix helper uniqueness** | `prefer_fill_authority(pass1, pass2, limit)`: pass-1 order preserved; pass-2 appended if `memory_id` not already present; `len <= limit`; no duplicate ids. |
| **F36 — Pass-2 SQL NOT IN optional** | CLI in-memory id filter is enough (page is tiny ≤200). **Do not** require store `NOT IN` this track (T274 F35 is Index/recall). |
| **F37 — `--global` mix** | Same two-pass with no project predicate. Project column stays T216 F8. |
| **F38 — Injectable / privacy** | List already shows operator-vault previews (T216 F34). Do **not** add Index `is_injectable_privacy` filter as DoD. NeverInject rows may still list (inventory honesty). |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | CLI hermetic: older `pin --tag t287 -- "DECISION: {needle}"` + four **newer** `## Objective` dumps → `memory list --limit 5` stdout (human) contains `DECISION:` and `{needle}`; the first **data** row preview does **not** start with `## Objective`. EXIT **0**. **Required red.** |
| **AC2** | Same fixture: `memory list --format json --limit 5` `items[0].preview` contains `## Objective` or the newest dump body (recency); does **not** require `items[0]` to be the pin. Keys T216 still present. EXIT **0**. **Required red** (JSON freeze). |
| **AC3** | Unit: `preview_line("ASSISTANT: TAGS: t287\nDECISION: needle", 80)` contains `DECISION:` and does **not** start with `TAGS:`. Existing `preview_line__role_prefix_stripped_always` stays green. |
| **AC4** | Store unit `list_memories__limit_plus_one__returns_extra_row_for_more_available` **stays green** (recency ORDER). |
| **AC5** | Store: `list_authority_memories` on older tagged `DECISION:` + newer `## Objective` returns the pin (content GLOB/retain). SQL extra contains `GLOB 'TAGS:*'`, `GLOB 'ASSISTANT: TAGS:*'`, `GLOB 'DECISION:*'`, `GLOB 'HOTSPOT:*'`, and a single `AND (` with `OR` (not two stacked `AND (`). |
| **AC6** | T216 `memory_list__format_json__schema_keys` **stays green**. |
| **AC7** | T216 `memory_list__summary__pinned_and_forgotten_counts` **stays green**. Hermetic mix fixture `--summary` `Pinned:` still counts dumps+pin (status COUNT, not authority COUNT). |
| **AC8** | Forgotten: pin a dump, forget it, `memory list --status forgotten --limit 5` and `forget --list-forgotten --limit 5` still recency; **no** authority promote of remaining pinned DECISION into forgotten list. |
| **AC9** | Untagged `DECISION: {needle}` (no `--tag`) vs newer Objective → human `--limit 5` still prefer-fills the pin (T274 untagged analog). |
| **AC10** | Chrome-only vault (four Objective dumps, no marker pin) → human `--limit 5` first row **is** `## Objective` (F32). EXIT **0**. |
| **AC11** | T216 `memory_list__missing_scope__exit_2_fail_usage` **stays green**. |
| **AC12** | JSON serde of mix-fixture run has the T216 field set only (no `authority` / `mix` key). |
| **AC13** | Displayed human preview lines still do not begin with `ASSISTANT:` (T216/T219). |
| **AC14** | `forget --list-forgotten` share-backend test **stays green**. |
| **AC15** | Manual PATH-or-`cargo run`: `memory list --limit 5` among the 5 human rows **≥1** preview starts with `DECISION:` / `CONSTRAINT:` / `INVARIANT:` / `HOTSPOT:` when such pins exist in scope; `memory list --summary` Pinned still matches. EXIT **0**. Hermetic AC1 covers if live page is PATH-behind. |
| **AC16** | Unit `prefer_fill_authority`: pass-1 `[pin]`, pass-2 `[dump, pin]` → `[pin, dump]`; `len==2`; pin once. |
| **AC17** | `memory list --help` / after_help mentions that human pinned prefer-fills authority and JSON stays recency (substring). |
| **AC18** | CAPABILITIES Memory inventory row updated; CHANGELOG T287. |

---

## 5. Design notes

### 5.1 Human mix (`run_inventory`)

```text
if summary → run_summary (unchanged)
status = parse_status
page = clamp_list_limit
if json || status != Pinned:
    rows = list_memories(recency limit+1)  # today
else:
    pass1 = list_authority_memories(limit = page)
    pass1.retain(|r| classify_pin_kind(&r.content) != PinKind::Other)
    if tag { pass1.retain(content_has_tag) }  # store already LIKE; token stage still
    pass2 = list_memories(limit = page+1)
    if tag { pass2.retain(content_has_tag) }
    rows = prefer_fill_authority(pass1, pass2, page+1)  # +1 for more_available probe
more_available = rows.len() > page
rows.truncate(page)
```

`--tag` over-fetch (T216 F43) still applies to **each** store call when tag is set.

### 5.2 Store GLOB extra (F4 / F27)

Hardcoded on `mp.content` (no `format!` of ids or column names):

```sql
AND (
  mp.content GLOB 'DECISION:*' OR mp.content GLOB 'ASSISTANT: DECISION:*' OR
  mp.content GLOB 'CONSTRAINT:*' OR mp.content GLOB 'ASSISTANT: CONSTRAINT:*' OR
  mp.content GLOB 'INVARIANT:*' OR mp.content GLOB 'ASSISTANT: INVARIANT:*' OR
  mp.content GLOB 'HOTSPOT:*' OR mp.content GLOB 'ASSISTANT: HOTSPOT:*' OR
  mp.content GLOB 'TAGS:*' OR mp.content GLOB 'ASSISTANT: TAGS:*'
)
```

Classifier is SoT for lowercase `decision:` (GLOB miss → pass-2 only). Document F8 analog: GLOB is a subset.

### 5.3 Preview (F6)

```rust
pub(crate) fn preview_line(content: &str, max_chars: usize) -> String {
    let contentful = first_contentful_line(content);
    let line = if contentful.is_empty() {
        let raw = content.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
        super::display_text::strip_role_prefix(raw)
    } else {
        contentful
    };
    super::display_text::truncate_preview_chars(line, max_chars)
}
```

`first_contentful_line` already skips `tags:` after role strip — **do not** double-strip role on the contentful path.

### 5.4 Dual-truth (F2 / F30)

Scripts that want “newest ingest” keep `--format json`. Humans who skim inventory get pins first. Same product decision as T283 cwd-first. Not a clap maze (F9).

---

## 6. Non-goals

- Changing `list_memories` ORDER or JSON item order
- `--authority` / `--kind` / `--sort` / `--offset` (T216 F24)
- Tag schema / pin rewrite / T263 H2
- `forget --match` two-pass
- Briefing stanza (T288), neighbors (T293), forgotten-empty next (T299)
- Index/recall ranking (T285/T286 Completed)
- Growing `project.rs` / `sync.rs` / `forget.rs` / `session_chrome.rs` / `preflight.rs`
- `cargo install`, live `.env` write, live `retention apply --confirm`, live `graph rebuild`
- clap 5, rusqlite 0.40, new required DTO keys

---

## 7. Verification plan (TDD)

**Red first (required):** AC1, AC2, AC3, AC5, AC16.

Then green: F4 method + F1 mix + F6 preview.

Regression: AC4, AC6–AC14, AC17–AC18.

Manual: AC15 via `cargo run -p ai-brains-cli -- memory list --limit 5` (not PATH until install).

No full workspace nextest as a plan gate. On go: targeted CLI+store nextest then full gate before publish.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Dual-truth agents vs scripts | F30 after_help + CAPABILITIES; JSON freeze AC2 |
| GLOB drift vs T286 `index_pass1_glob_sql` | F27 store unit needles; classifier retain F5 |
| `MemoryListFilter` field add breaks literals | F4/F37 new method, no field |
| `forget.rs` hotspot growth | Mix gated in `memory.rs`; forgotten path unchanged |
| Prefer-fill first page all pins (limit 50) | Intentional inventory; dumps still in JSON recency / raise `--limit`; F31 later slots if pass-1 &lt; limit |
| USER/SYSTEM tagged pins miss pass-1 | F29 residual; default assistant |
| PATH 0.1.2 still chrome | F17; Manual `cargo run`; AC1 hermetic SoT |

---

## 9. Deferred absorb / decline

### 9.1 Entire `deferred.md` scan (open overlapping)

| Item | Disposition |
|------|-------------|
| `memory list` just-now ingest (T285–T300 mint + T274/T285/T286 decline rows) | **Absorb** F1–F6 / AC1–AC3 / AC15 |
| T274 F13 recency freeze / T285 F14 / T286 F15 / T286 AC15 list ORDER | **Lift F13 for human pinned**; **affirm store+JSON** F2/F3/AC4 |
| T216 closeout `--offset` / tag histogram | **Decline** F11 / T216 F24 |
| T286 closeout Index residual `## Objective` without fitting pin | **Not this track** — T286 Completed residual |
| T288 briefing granted-empty | **Decline → T288** |
| T289 personal briefing | **Decline → T289** |
| T290 governed empty | **Decline → T290** |
| T291 `query trace` | **Decline → T291** |
| T292 `policy check` human | **Decline → T292** |
| T293 graph neighbors dumps | **Decline → T293** |
| T294 leftover dest / context upsert | **Decline → T294** |
| T295 usable backup | **Decline → T295** |
| T296–T298 nightly/daemon/device | **Decline** peers |
| T299 forget-list empty next | **Decline → T299** (do **not** steal empty next) |
| T300 graph sparse rebuild | **Decline → T300** |
| T240 F2 / T263 H2 / 750 ms / clap 5 / density floors | **Decline** F16 |
| Identity leftover `7d97a456` | **Not this track** — T258 / T294 |
| last-PR Cursor #202 | **N/A** empty — **no T301** |

Closed/strikethrough rows (T216–T286 Completed, T274–T284, T255, …) stay closed.

### 9.2 last-PR Cursor

| Source | Finding | Disposition |
|--------|---------|-------------|
| [#202](https://github.com/Ryan-AI-Studios/AI-Brains/pull/202) T286 | `comments` / `reviews` / issue comments **empty** | **N/A** — no absorb, no mint |
| Open HEAD PR | none (on `main`) | N/A |
| Dependabot `#58–#72` | rusqlite 0.40.2 / chrono / tokio / actions | **Not this track** — standing pin freeze |

---

## 10. Implement order (on go)

1. Phase 0 re-verify pins, deferred, `#202` still empty, dogfood list `--limit 5` read-only.
2. FEATURE TX.
3. **Red:** AC1/AC2/AC3/AC5/AC16.
4. **Green:** `list_authority_memories` + `prefer_fill_authority` + `preview_line` + `run_inventory` branch.
5. Docs: CAPABILITIES / CHANGELOG / after_help.
6. Targeted nextest `-p ai-brains-cli` `-p ai-brains-store` + clippy those packages.
7. Review log + codex-review (F22).
8. Full gate; conductor Completed; deferred closeout; publish (implement-track Phase 6).

---

## 11. Soft residuals

| Residual | Note |
|----------|------|
| PATH until `cargo install` | F17 |
| Live first page may be 50 pins (no ingest) | F1 success for inventory; JSON recency still shows ingest |
| Lowercase `decision:` GLOB miss | Classifier SoT; GLOB subset (T274 F8 analog) |
| USER/SYSTEM TAGS GLOB | F29 |
| Duplicate GLOB vs `index_pass1_glob_sql` | F27 |
| T299 empty forgotten next | Not stolen |
| `--offset` cursor | T216 F24 |
| Index live residual without fitting pin | T286 closeout; not list |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-store/src/lib.rs` | Trait method `list_authority_memories` |
| `crates/ai-brains-store/src/query_store.rs` | GLOB extra + method impl |
| `crates/ai-brains-store/tests/memory_list_inventory.rs` | AC5 (+ AC4 stays) |
| `crates/ai-brains-cli/src/commands/memory.rs` | Mix branch, `preview_line`, `prefer_fill_authority` |
| `crates/ai-brains-cli/src/main.rs` | after_help sentence only (≤5 lines) |
| `crates/ai-brains-cli/tests/memory_list_inventory.rs` | AC1/AC2/AC7/AC9/AC10 |
| `Docs/CAPABILITIES.md` | Memory inventory row |
| `CHANGELOG.md` | T287 |
| `Docs/OPERATIONS.md` | One-liner if the list example still says recency-only |
| `conductor/conductor.md` / `deferred.md` / this spec+plan | Registry |

**Do not touch:** `project.rs`, `sync.rs`, `forget.rs` production, `session_chrome.rs`, `ranking.rs`, `lexical.rs`, `pin.rs` write, CLI `preflight.rs` production, `ci.yml`, `Cargo.toml` pins.

---

## 13. AI fold-in

Reserved. `/fold-in 287` after review-track. Do **not** edit `*-review.md` here.
