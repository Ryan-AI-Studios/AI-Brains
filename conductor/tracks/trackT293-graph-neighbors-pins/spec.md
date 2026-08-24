# T293 — `graph neighbors` of a pin must not be dump-session soup

- **Track ID:** T293-GraphNeighborsPins
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / FEATURE
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `graph neighbors` **7/8**; PREVIEW filled (T278) but neighbors are dump sessions (`# Review of Track 254`, `## Objective`, ````json`). Placeholder minted with T285–T300 (`76c4db9`). T246 ✅ pretty table + JSON keys/sort. T262 ✅ pin = memory node + `RECALLS`. T278 ✅ session PREVIEW `{n} memories · first line`. T287 ✅ human prefer-fill / JSON freeze analog. **This track is human-only 1-hop reorder**, not 2-hop, not rebuild.
- **Depends on:** T246 ✅ pretty + JSON freeze + F9 direction→label→id; T262 ✅ live projection; T278 ✅ session PREVIEW (do **not** restyle); T274/T285 ✅ `classify_pin_kind` / `first_contentful_line`; T283/T287 ✅ human permute + JSON freeze
- **Blocks / feeds:** Operators who paste a pin id into `graph neighbors --format human` see an authority neighbor (leading `DECISION:` / `CONSTRAINT:` / `INVARIANT:` / `HOTSPOT:` memory, or a session whose caption first-line is that marker) on the **first data row** when such a 1-hop exists. Sparse rebuild remains **T300**. Leftover dest upsert **T294**. Forget-list **T299**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “graph neighbors dump sessions” (every T285–T292 decline pointer)
- **Not absorbed (DoD):** Live `graph rebuild` (T300); T213 floor retune; Cargo default-on graph; projector rewrite / fake edges / 2-hop pretty rows (T278 F18); hierarchy/session/update restyle; JSON `kind`/`preview` keys; `--authority` / `--sort`; T285 chrome-seed skip (recall `--graph-boost`); T240 F2; T263 H2; clap 5 / rusqlite 0.40
- **Research date:** 2026-08-23 (plan dogfood HEAD `80a10d9` T292 `#208`; plan commit `fe9fb89`. Product `src/` = T278 session PREVIEW + T246 JSON sort incoming-first. PATH **0.1.2** 2026-08-22 19:41 **without** T285–T292 — neighbor-order hole is in **source and PATH**)
- **AI fold-in:** 2026-08-23 `agy-review.md` (`fe9fb89`) + `opencode-review.md` (`fe9fb89`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy/OpenCode m2 `split_once(" · ")`; Agy m2 `sort_by_key` + original index; OpenCode m1 new `memory_projection` INSERT helper (not T278 DROP COLUMN); Agy/OpenCode O2 four-tier rstest; OpenCode O3 exact AC3 first-row identity; Agy/OpenCode O1 PROTOCOL-COMPAT `:95` pretty note. **Already:** F1 stable; F4/AC13 strip; F14/AC11 array-order; AC4 exact dump UUID. **Decline:** OpenCode “third neighbors row `:103`” — live `:103` is `project scan-roots`. Word/pin/hotspot snapshot only. Disposition **§13**.
- **Ledger:** planning DOCS TX `83553530-cc14-4e4a-ad5e-cf366cf11a03`. Fold-in DOCS TX `13843d9e-33be-4288-8979-534f1593d3ed`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** pin production decisions to the live vault as implement (hermetic needle is SoT; Manual unique canary allowed on go). Do **not** rewrite `.env`. Do **not** live `graph rebuild`. Do **not** grow hotspot `project.rs` / `sync.rs` / `forget.rs` production / CLI `preflight.rs` / `session_chrome.rs` / `ranking.rs` (import `classify_pin_kind` only) / `projector.rs` / `queries.rs` `get_neighbors`. Helpers live in `graph.rs` (not top-10; **1073** lines — same file first; split `graph_neighbors_order.rs` only if production net ≥80). Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Pretty first page prefers authority 1-hop.** `ai-brains graph neighbors <id> --format human` (and `pretty` / `text` / TTY `auto`) reorders **existing** 1-hop rows so authority neighbors land first: memory-kind whose preview classifies as Decision/Constraint/Hotspot, then sessions whose T278 caption after ` · ` classifies the same way. Dump sessions (`## Objective`, `# Review of Track`, ````json`) stay on the page, later.
2. **JSON 1-hop stays a wire contract.** `--format json` / pipe `auto` keep T246 keys `{memory_id, neighbors:[{external_id,label,direction}]}` and T246 F9 array order **direction → label → id**. No `kind` / `preview`. No pretty permute.
3. **Do not invent graph.** 1-hop only (`get_neighbors` unchanged). No 2-hop sibling memories as extra pretty rows (T278 F18). No projector edges. No floor retune. No live rebuild.
4. **North star.** Capture independence: pretty presentation over existing `graph_node` / `graph_edge` / `memory_projection`. No models. No new events. Operators who open neighbors of a pin must not conclude the vault’s graph is only review-track sessions because UUID-sort put dumps first.

This unblocks the unused caption: T278 filled PREVIEW; the first row is still dump-session soup because T246 sorts `incoming` before `outgoing` and dump session UUIDs win. Live `b189ad20` (T278 sample pin) is three `## Objective` sessions. Live recency dump `d9183790` is twelve `in RECALLS` sessions (`# Review of Track 254` / ````json`).

---

## 2. Live baseline (re-scan 2026-08-23)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `80a10d9` T292 squash `#208`. Tree **CLEAN**. `origin/main` = HEAD (`00`). |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. Has T274 + T278 PREVIEW. **Does not have T285–T292.** Neighbor-order hole is in **source + PATH** (T246-era sort). **Do not `cargo install`.** Tests/manual AC use `cargo run --features graph` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4042** (volatile). In-context **0/0/0**. Word **280**. Grants omitted (live 3 of 3). |
| `graph update --format human` | `status: sparse` `density: warn` `nodes: 30524` `edges: 3746` `pinned_memories: 48087` `memory_nodes: 28141` `edge_node_ratio: 0.123` remediator `ai-brains graph rebuild`. Honesty is **T300**, not this track. |
| Recency dump `d9183790-…` (`# AI-Brains Session Onboarding Complete`) | Pretty neighbors **(12)** all `in RECALLS` / `KIND session`. PREVIEW filled: `# Review of Track 254`, `# Review of Track 255`, ````json`. **T278 captions work. First page is dump sessions.** |
| T278 sample pin `b189ad20-ba63-4cfa-a282-333c49996103` | Pretty **(3)** all `in RECALLS` session / PREVIEW `## Objective`. JSON: three `{external_id,label,direction}` incoming `RECALLS` sorted by UUID (`13d5625b` then `9c866cec` then `fd6035c8`). **This is the 7/8 hole.** |
| `graph neighbors --help` | `--format` default **`auto`**, tokens `auto\|pretty\|human\|text\|json`. `--limit`. Parent after_help: TTY table; JSON compact; session PREVIEW sentence. **No** dual-truth “human prefers authority; JSON order unchanged.” |
| Last GitHub PR | [#208](https://github.com/Ryan-AI-Studios/AI-Brains/pull/208) T292 (merged 2026-08-24). `gh pr view --comments`, `/reviews`, `/comments`, `issues/208/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, `#58` tower-http, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / doctor | ledgerful doctor **4** warn (legacy `.changeguard` / sig-pin / sig-version / timings). Optional :8081 unreachable; :8083 ok. **0 pending / 0 drift.** Hotspot **#1** `project.rs` (**3.932**) — **do not touch.** `sync.rs` **#2** (3.619). `governed_common.rs` **#3** (3.604). `session_chrome.rs` **#6**. CLI `preflight.rs` **#8**. `graph.rs` **1073** / `projector.rs` / `queries.rs` **not** top-10 — **extend `graph.rs`.** |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why the first pretty row is dump-session soup

| Layer | Truth |
|-------|--------|
| 1-hop of a pin is sessions | `MemoryPinned` projector (`projector.rs:63–84`) adds memory + session + session→memory `RECALLS`. Live 1-hop is almost always `incoming` / `RECALLS` / `kind=session`. |
| T246 F9 sorts incoming first | `sort_neighbor_hits` (`graph.rs:140`) direction→label→id. Dump session UUIDs that sort before the pin’s own session win the first row. JSON **and** pretty share this sort today (`neighbors` `:386` then pretty rows in that order). |
| T278 filled PREVIEW | `pretty_neighbor_rows` `:316–322` session arm uses `session_neighbor_caption`. Live cells are scannable **and** they say `# Review of Track 254`. Captions are not the hole; **order** is. |
| T278 F18 declined 2-hop | Extra pretty rows of sibling memories would diverge pretty vs JSON 1-hop. Operator already has `graph session <id>`. **Affirm.** |
| Chrome-only 1-hop is honest | T278 pin `b189ad20` has **zero** authority-captioned 1-hop. Prefer-fill cannot invent a `DECISION:` neighbor. Residual F32 (T287 analog): first row stays dump. Hermetic mixed fixture is SoT. |
| Recall `--graph-boost` | T285 AC17 chrome-seed skip is **recall** ranking (`recall_rank_v2_graph.rs`). `get_neighbors` API frozen (T246 F10). **Do not steal.** |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| CLI neighbors | `graph.rs` `neighbors` **`:361–407`** | `sort_neighbor_hits` then pretty vs JSON. **Pretty: reorder after `pretty_neighbor_rows`; JSON: leave F9 sort.** |
| Pretty rows | `pretty_neighbor_rows` **`:308–331`** | kind + preview. Session T278 caption. Memory `preview_line` 80 (T287 envelope inherit). |
| JSON emit | `format_neighbors_json` **`:185–193`** | `NeighborsOutput` `{memory_id, neighbors}`. `NeighborHit` three keys. Compact `to_string`. |
| JSON sort | `sort_neighbor_hits` **`:140–147`** | Unit `:1029`. **Stay green. Pretty must not call this as the last sort.** |
| Session caption | `format_session_neighbor_preview` **`:253–259`** | `{n} memories` + optional ` · first`. Cap 80. **Freeze T278 F2.** Ranker strips ` · ` once. |
| Classifier | `ranking.rs` `classify_pin_kind` **`:122`**, `PinKind` **`:65`**, `first_contentful_line` **`:102`**. `lib.rs` `pub use`. | Import in `graph.rs`. **Do not edit `ranking.rs`.** Hotspot stays (T287 F5). |
| `get_neighbors` | `queries.rs` **`:62`** | UNION ALL in+out; no ORDER BY; no kind. Recall `--graph-boost` uses `external_id` only. **Do not change signature or SQL.** |
| Projector pin | `projector.rs` `MemoryPinned` **`:63–84`** | **Do not rewrite.** |
| clap | `main.rs` `GraphCommands::Neighbors` **`:2846–2854`** | default `auto`; `value_parser` five tokens; `--limit`. **No new flags.** after_help on **enum** `:2841`. |
| Limit | `clamp_list_limit` / `format_neighbors_pretty` `take(shown)` | Pretty always clamps (default 50). JSON unlimited unless `--limit`. Reorder **full** 1-hop then apply pretty limit (already builds all rows). |
| Hermetic | `tests/graph_human_cli.rs` JSON keys `:213`; `graph_live_projection.rs` T262 AC6/AC7 + T278 AC3 PREVIEW | Additive ACs. Stay-green those. Feature-off `:17`. |
| T285 graph test | `tests/recall_rank_v2_graph.rs` | Recall chrome-seed. **Do not edit as DoD.** |
| PROTOCOL-COMPAT | `Docs/PROTOCOL-COMPAT.md` **`:94–95`** | Keys unchanged. Array order direction→label→id. **Add pretty human-only prefer note on the array-order row.** |
| CAPABILITIES | **`:103`** / command **`:462`** | T278 PREVIEW sentence. Add human prefer-fill / JSON freeze. |
| OPERATIONS | graph **update/rebuild** `:893+`; neighbors paragraph **`:948`** (T246/T262/T278) | **Extend `:948`** — do not add a second block. |
| Hotspots | `project.rs` #1 3.932 | **Do not touch.** Helpers in `graph.rs`. |

### 2.4 Dependency / standards research (2026-08-23) — snapshot; re-verify at execute

| Pin / source | Workspace / live | Action |
|--------------|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** / GitHub **v4.6.6** (2026-08-06) / **no clap 5** | **No bump.** No new flags. Additive after_help only. |
| `serde_json` | lock **1.0.150** / crates.io **1.0.151** | **No bump.** Compact JSON keys frozen. |
| `chrono` | lock **0.4.44** / crates.io **0.4.45** (`#62`) | **No bump.** |
| `rusqlite` | lock **0.39.0** / crates.io **0.40.2** (`#61`) | **No bump.** No extra SQL. |
| `uuid` | lock **1.23.1** | **No bump.** |
| rustc / edition / nextest | **1.95.0** / **2024** / **0.9.140** | Unchanged. |
| workspace version | **0.1.2** | **No bump.** |
| New crates | — | **Zero.** No comfy-table / petgraph / regex. |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Human CLI may change; JSON stays stable | [clig.dev Output](https://clig.dev/#output) (fetched 2026-08-23): humans first; TTY heuristic; `--json` for structure; **changing human output is usually OK** | Pretty reorder is human-only. JSON F9 order is the script interface. |
| Future-proof additive | [clig.dev Future-proofing](https://clig.dev/#future-proofing) | Do not add required JSON keys. Dual-truth in after_help (T283/T287 pattern). |
| Neighbor lists need ranked captions | [Neo4j Browser styling](https://neo4j.com/docs/browser/operations/browser-styling/) (T278 cite): nodes get **captions** from properties, not raw ids. Wikidata [Help:Ranking](https://www.wikidata.org/wiki/Help:Ranking): **preferred** statements surface first; deprecated stay listed | Prefer authority 1-hop; do **not** drop dump sessions. |
| Typed graphs; do not stuff edges | T213 / T278: TRACE-KG 2026 compact typed graphs; raising live E/N is rebuild/projector | **No** 2-hop pretty rows; **no** fake `RELATED` edges. Density stays T300. |
| Grounding prefers typed evidence, never strips | [knowgraph USER_GUIDE §10](https://github.com/yunusgungor/knowgraph/blob/main/docs/USER_GUIDE.md) (crawled 2026-08-22): evidence-backed ranking; **nothing is ever stripped** | Demote dumps; do not filter them out of the 1-hop list. |
| T180 P-CLI | Additive extras OK; type/default change needs documented lift; compact↔pretty without a flag is breaking | Pretty **row order** is not a key change. JSON array order **frozen** (T246 F9 PROTOCOL-COMPAT behavioral note). Document the human-only lift on that same row. |
| SQLite / SQLCipher / schtasks | N/A — no new SQL, no tasks | N/A (written). |

**Could not verify:** live 1-hop of an in-scope `DECISION:` pin that also has a dump session (PATH recall still ranks chrome; no live canary). Hermetic mixed fixture is SoT. Live Manual is pass-with-observed-data if chrome-only (F32).

**ledgerful / ai-brains:** `preflight --summary` Pinned **4042** / 0/0/0 / word **280**. `graph update` sparse 0.123. `graph neighbors` dump soup on `d9183790` and `b189ad20`. `ledgerful doctor` 4 warn; 0 pending / 0 drift; `index --incremental`; `search "pretty_neighbor_rows"` → `graph.rs:308`; `search "sort_neighbor_hits"` → `:140` / `:386`; `scan --impact` CLEAN at `80a10d9`; hotspots `project.rs` #1. Semantic recall of “graph neighbors pin first” still returns review-track dumps (PATH 0.1.2 / T285 not installed) — not SoT.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `83553530`. Implement starts a **FEATURE** TX. |
| **F1 — Human-only prefer-authority** | After T246 F9 sort + `pretty_neighbor_rows`, pretty path calls `prefer_authority_neighbor_rows`. Rank: **0** memory + `classify_pin_kind != Other`; **1** session + caption body `!= Other`; **2** other memory; **3** other session / other kinds. Implement with `Vec::sort_by_key` (stable; do **not** `sort_unstable_by_key`) on `(rank, original_index)` so within-tier order stays F9 direction→label→id (Agy m2). Same length; **no drops**. Applies to `pretty`/`human`/`text` and TTY `auto`. |
| **F2 — JSON freeze** | `sort_neighbor_hits` remains the JSON order (T246 F9). Keys T246 F5. Compact. **No** pretty permute on JSON. Pipe `auto` is JSON (already). |
| **F3 — 1-hop only** | Do **not** add 2-hop sibling memories as pretty rows (T278 F18). Do **not** change `get_neighbors` SQL/signature. Do **not** walk `get_session_memories` to emit extra rows. |
| **F4 — Classifier from displayed preview** | Memory: `classify_pin_kind(&row.preview)`. Session: `str::split_once(" · ")` (**exact** space-dot-space; T278 `format_session_neighbor_preview` `:256` — Agy m1 / OpenCode m2). Classify the remainder (no separator / empty remainder → Other). Do **not** split on `.` or bare `·`. Import `ai_brains_retrieval::{PinKind, classify_pin_kind}`. **Do not edit** `ranking.rs` / `session_chrome.rs`. Hotspot **stays** (INVARIANT→Constraint). |
| **F5 — Dumps stay** | Prefer-fill, not hard-exclude (T260/T274 analog). Chrome 1-hop may occupy rows 2+ and the whole page when no authority 1-hop exists (**F25**). |
| **F6 — T278 PREVIEW freeze** | Session cell remains `{n} memories · first line` (fail-open, 80-cap, `pick_first_nonempty`). Do **not** retitle PREVIEW to `DECISION:` when the first memory is chrome. Ranker reads the caption; it does not rewrite it. |
| **F7 — Projector / density / rebuild freeze** | No `projector.rs` rewrite. No T213 floor change. No live `graph rebuild` (T300). `graph update` JSON/human unchanged. |
| **F8 — No new clap flag** | No `--authority` / `--sort` / `--pins-only`. Silent human mix (T287 F9). Format tokens / default `auto` frozen (T246 F1). |
| **F9 — Limit after pretty reorder** | Reorder the full pretty row vec, then `format_neighbors_pretty` `take(limit)`. JSON truncate still after F9 sort only. |
| **F10 — NeighborHit / recall freeze** | Do not add serde fields. Do not change `get_neighbors`. T285 `recall_rank_v2_graph.rs` stays green without edits. |
| **F11 — Hierarchy / session / update freeze** | Do **not** prefer-fill those commands. Neighbors only. |
| **F12 — Feature-off freeze** | Exit **2** + `FEATURE_UNAVAILABLE` stays. |
| **F13 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. `tempfile::tempdir` hermetic. **AC2 required rstest `#[case]`** for the rank helper. |
| **F14 — Docs** | CAPABILITIES graph row: human prefer-fills authority 1-hop; JSON order unchanged. PROTOCOL-COMPAT §5 **array-order row `:95`**: pretty human-only prefer-authority; JSON stays direction→label→id (Agy O1 / OpenCode O1). GraphCommands `after_help` one dual-truth sentence (T283/T287). OPERATIONS **extend** the existing neighbors paragraph **`:948`** (do **not** add a second graph block). CHANGELOG on implement. |
| **F15 — PATH** | Soft. Source/hermetic SoT. Do not `cargo install` as implement. |
| **F16 — Capture independence** | Pretty reorder + existing SQL. No models, embeddings, new events, ledgerful writes. |
| **F17 — Isolation hotspots** | Do not grow `project.rs` / `sync.rs` / `forget.rs` production / CLI `preflight.rs` / `personal.rs` / `briefing.rs` / `session_chrome.rs` / `ranking.rs` (import only) / `projector.rs` / `queries.rs` / `doctor.rs` / `.github/workflows/ci.yml`. |
| **F18 — File growth** | `prefer_authority_neighbor_rows` + `neighbor_authority_rank` + `session_caption_body` are `pub(crate)` in `graph.rs`. **Not** `pub`. **Not** re-exported from `commands/mod.rs`. Split `graph_neighbors_order.rs` only if production net ≥80 lines. |
| **F19 — last-PR Cursor** | #208 empty → **N/A**. Dependabot remotes not this track. **No T301.** |
| **F20 — Decline peers** | T294 leftover upsert; T295 backup; T296 nightly Router; T297 daemon vs LLM; T298 device; T299 forget-list; T300 graph sparse. T292 Completed `#208` — not stolen. T285 chrome-seed skip — not stolen. |
| **F21 — Standing declines** | T240 F2; T263 H2; 750 ms; clap 5; rusqlite 0.40; DTO new required keys; Cargo default-on graph; floor retune. |
| **F22 — ISSUES.md** | Does not exist. Debt is `deferred.md`. |
| **F23 — Cross-model** | FEATURE (operator pretty contract). After Phase-1 review clean, run read-only `codex-review`. |
| **F24 — Stop-before** | Even after go: no live `graph rebuild`; no `.env` write; no extra `policy bootstrap`; no `retention apply --confirm`; no schtasks mutate; no `cargo install`. |
| **F25 — Chrome-only 1-hop** | Pass-1 empty (all ranks 2–3) → today’s F9 pretty order (no lie). Authority-only 1-hop → first page all authority. |
| **F26 — Dual-truth after_help** | Human table prefer-fills authority 1-hop; JSON order unchanged (direction→label→id). Same class as T283/T287. |
| **F27 — Stay-green** | T246 `graph_neighbors__json_and_pretty__frozen_keys_and_dir`; T262 AC6 JSON incoming RECALLS + AC7 pretty `in`+`RECALLS`; T278 AC3 session PREVIEW contains `1 memories` + `DECISION`; T278 AC5 fail-open; `sort_neighbor_hits` unit; feature-off exit 2; `format_neighbors_json` key units. T278 AC3 does **not** assert first-row identity — stay green without rewrite. |
| **F28 — PowerShell** | `;` not `&&`. |
| **F29 — Identity stdout** | JSON still `note_machine_stdout` (T257). Pretty does not. |
| **F30 — No shared list helper** | Do **not** reuse `memory::prefer_fill_authority` (different row type). Graph helper is local. |
| **F31 — AC3 dump-memory INSERT (OpenCode m1)** | `graph_human_cli.rs` has `seed_node` `:104` / `seed_edge` `:119` / `open_zero_vault` `:91`. **No** `memory_projection` insert helper exists. T278 DROP COLUMN (`graph_live_projection.rs:187`) is the fail-open AC5 fixture — **do not reuse it** to seed content. Write a **new** `seed_memory_projection` (file-local) with columns `memory_id, session_id, project_id, content, privacy, status, level, created_at, updated_at` (CLI test shape `cross_repo_bridge_smoke.rs:101`; `tx_id` optional). `privacy` JSON string `"LocalOnly"`. Required NOT NULL from `0006_memory_projection.sql`: `memory_id, content, privacy, status, created_at, updated_at`. |

---

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | Unit: `prefer_authority_neighbor_rows` on `[dump-session Objective, authority-memory DECISION:]` (F9 incoming-first order) → first row is the memory; len unchanged; dump still present second. **Required red.** |
| **AC2** | rstest `#[case]` on the helper: (1) overlap/stable — two dump sessions + one Decision memory → memory first, dumps keep relative order; (2) session-authority — dump session Objective + session caption `1 memories · DECISION: x` → Decision session first; (3) chrome-only — two Objective sessions → original order (F25); (4) Hotspot memory ranks with Decision (not Other); (5) `INVARIANT:` session caption ranks as authority; **(6) four-tier mixed** (Agy O2 / OpenCode O2) — one of each rank 0–3 in F9 dump-first order → exact `[authority-memory, authority-session, other-memory, other-session]`. |
| **AC3** | CLI hermetic (`test(graph)`): pin `DECISION: {needle}` + extra incoming dump session (UUID `00000000-0000-4000-8000-000000000001`) whose session memories first-line is `## Objective` (F31 INSERT, not T278 DROP COLUMN) → `graph neighbors <pin> --format pretty --limit 8` first **data** row after the header contains `DECISION` or `{needle}` **and** does **not** contain `## Objective` / `# Review of Track`; that row’s ID is **not** `00000000-…0001` (OpenCode O3: pin’s own session / authority neighbor, not the dump UUID); dump session UUID still appears in a later data row. Exit **0**. **Required red.** |
| **AC4** | Same fixture: `--format json` `neighbors[0].external_id` is the dump session `00000000-…0001` (incoming UUID-first); keys still exactly `{external_id,label,direction}`; root keys `{memory_id,neighbors}`. Exit **0**. **Required red** (JSON freeze). |
| **AC5** | T278 `pin__graph_on__neighbors_pretty__session_preview_nonblank` **stays green**. |
| **AC6** | T246 `graph_neighbors__json_and_pretty__frozen_keys_and_dir` **stays green**. |
| **AC7** | T262 `pin__graph_on__printed_id_neighbors_json_without_rebuild__ac6` and pretty AC7 **stay green**. |
| **AC8** | Feature-off `graph_neighbors__format_pretty__feature_off_exit_2` **stays green**. |
| **AC9** | `sort_neighbor_hits__incoming_before_outgoing_then_label_then_id` **stays green**. |
| **AC10** | `graph neighbors --help` / GraphCommands after_help mentions human prefer-fills authority and JSON order unchanged (substring). Catalog/examples still include `--format json` and session PREVIEW sentence. |
| **AC11** | CAPABILITIES graph row + PROTOCOL-COMPAT §5 **array-order row `:95`** (pretty human-only prefer; JSON direction→label→id) + OPERATIONS **`:948` paragraph extended** (not a new section) + CHANGELOG T293. |
| **AC12** | Manual (on go, `cargo run -p ai-brains-cli --features graph --`, no `--daemon`): `graph neighbors <pin-id> --format human --limit 8` and `--format json`. Pass = if the live 1-hop contains an authority neighbor, pretty first data row is that class (not `## Objective` / Track 254 as row 1); JSON `neighbors[0]` still F9 order. If live 1-hop is chrome-only (F25), record observed first PREVIEW and **hermetic AC3 is SoT**. Unique canary pin **allowed**. **Do not** `cargo install`. **Do not** `graph rebuild`. |
| **AC13** | Unit: `session_caption_body("5 memories · DECISION: x")` is `DECISION: x`; `"2 memories"` / `"2 memories ·    "` → empty/Other. `"1 memories · 1.2.3 dump"` remainder is `1.2.3 dump` (does **not** split on `.`). Does **not** classify the `{n} memories` prefix as Decision. |
| **AC14** | Pretty `--limit 1` on the AC3 fixture returns **one** data row and it is the authority neighbor (`… and N more` still correct). JSON `--limit 1` is still the dump session. |

---

## 5. Design notes

### 5.1 Why not 2-hop pretty rows

T278 F18: extra sibling memories diverge pretty vs JSON 1-hop and duplicate `graph session`. Live dump soup is **order among existing 1-hop**, not missing edges. Mixed hermetic (dump session + pin’s own Decision-captioned session) is enough to prove reorder. Adding `SYNTHESIZED_FROM` in the hermetic fixture is optional extra coverage, not required if the pin session already captions `DECISION:`.

### 5.2 Why JSON keeps incoming-first

T246 F9 is a PROTOCOL-COMPAT **behavioral** note (AI2 M2). Scripts that pin `neighbors[0]` as “first incoming UUID” must not break. Pretty is the human conversation (clig). Dual-truth after_help.

### 5.3 Caption strip

T278 `format_session_neighbor_preview` concatenates `"{n} memories"` + `" · "` + first line (`graph.rs:256`). Ranker uses `split_once(" · ")` only (Agy m1). Do **not** regex the count. Do **not** split on `.` or bare `·`. Do **not** classify `5 memories` as authority.

### 5.4 Hermetic dump session (AC3 / F31)

**Reuse:** `graph_human_cli.rs` `open_zero_vault` `:91`, `seed_node` `:104`, `seed_edge` `:119`, and `graph_live_projection.rs` `pin_decision` `:22` (session `aaaaaaaa-…`, content `DECISION:`).

**Write new:** `seed_memory_projection` in the same hermetic test file as AC3 (prefer `graph_human_cli.rs`). T278 DROP COLUMN (`graph_live_projection.rs:187`) is fail-open AC5 — **not** a content seeder (OpenCode m1).

Dump session UUID **`00000000-0000-4000-8000-000000000001`** so F9 incoming UUID-sort puts it first in JSON. Dump memory UUID `…0002` + content `## Objective\n…` + `seed_edge(dump-session, RECALLS, dump-memory)` so `get_session_memories` + `ids.sort()` + `pick_first_nonempty` yields Objective PREVIEW. Also `seed_edge(dump-session, RECALLS, pin-id)` so the dump session is a 1-hop of the pin.

### 5.7 `sort_by_key` (Agy m2)

[`Vec::sort_by_key`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort_by_key) is **stable**. `sort_unstable_by_key` is not. Key is `(neighbor_authority_rank(row), original_index)` so within-tier order cannot regress even if someone later switches to unstable.

### 5.5 Why import classifier, not GLOB

1-hop set is tiny (≤200 pretty). Previews are already loaded. `classify_pin_kind` is the T274/T285/T287 retain rule (leading-line after envelope; buried `decision:` is Other). No store GLOB. No `query_store` growth.

### 5.6 `graph.rs` size

**1073** lines with a large `#[cfg(test)]` tail. Production helper is ~40 lines. Prefer same file (T278). Split only if production net ≥80.

---

## 6. Non-goals

- 2-hop pretty rows / mermaid / hierarchy captions (T246 F17 / T278 F18/F19)
- Live `graph rebuild` / floor retune / Cargo default-on (T300 / T213 / T200)
- Projector more edges / `SessionSummaryCreated` edges
- JSON `kind` / `preview` keys / array reorder
- `--authority` flag / `--sort`
- Recall `--graph-boost` chrome-seed (T285)
- `graph session` / `hierarchy` / `update` restyle
- clap 5 / rusqlite 0.40 / lock bumps / new crates
- `cargo install` / live `.env` / extra live `policy bootstrap`
- Pin→Approved (T263 H2)

---

## 7. Verification plan (TDD)

**Red first (required):**

1. `prefer_authority_neighbor_rows__dump_then_decision_memory__memory_first` (AC1)
2. `session_caption_body__memories_dot_decision__strips_prefix` (AC13)
3. Hermetic `graph_neighbors__pretty__authority_before_dump_session` (AC3) — fail while pretty still F9 incoming-first
4. Same fixture JSON `graph_neighbors__json__dump_session_still_first` (AC4)

Then rstest AC2. Pretty `--limit 1` AC14.

**Green:** call prefer on pretty path only in `neighbors`.

**Stay-green:** AC5–AC9 / AC8.

**Docs:** AC10/AC11 after behavior is green.

**Manual:** AC12 `cargo run --features graph` only.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| JSON array-order break | F2 / AC4 / AC6 / AC9; pretty-only permute |
| Accidental 2-hop | F3; no `get_session_memories` extra rows |
| T278 PREVIEW restyle | F6; AC5 stay-green |
| `classify_pin_kind` on `{n} memories · …` without strip | AC13 unit |
| Hotspot `project.rs` / `ranking.rs` | F17 import-only |
| Chrome-only live vault | F25 / AC12 hermetic SoT |
| PATH-behind | F15 `cargo run --features graph` / hermetic |
| last-PR leftover missed | #208 empty (verified). Dependabot not tracks. |
| `graph.rs` bloat | F18 same-file cap |
| AC3 INSERT missing columns | F31 9-col CLI test shape; required 0006 NOT NULL |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit `graph neighbors` dump sessions U=7 | **Absorb** F1–F6 / AC1–AC4 / AC12 |
| Placeholder Manual `--format human --limit 8` | **Absorb** AC12 |
| Placeholder JSON freeze vs human-only | **Absorb** F2 / AC4 (T283/T287 analog) |
| Placeholder PREVIEW still `{n} memories · first line` | **Absorb** F6 / AC5 |
| T278 F18 2-hop pretty rows | **Affirm decline** F3 |
| T246 F5 keys / F9 JSON sort | **Affirm freeze** F2 |
| T262 pin = node / no prefix match | **Affirm** F10 / AC7 |
| T287 human prefer-fill | **Reuse pattern** F1; **do not** reuse `prefer_fill_authority` F30 |
| T285 chrome-seed skip (recall graph-boost) | **Decline** F20 — not CLI neighbors order |
| T292 policy-check human | **Decline** — Completed `#208` |
| T294 leftover dest-missing | **Decline → T294** |
| T295–T299 peers | **Decline →** those placeholders |
| T300 live rebuild / floors | **Decline → T300** F7 |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F21 |
| last-PR Cursor **#208** | **N/A empty** — **no T301** F19 |
| Identity leftover `7d97a456` | **Not this track** — T258 / T294 |
| Open T294–T300 | **Not related** except named declines |
| Closed T274–T292 | **Stay closed** |
| Dependabot `#58–#72` | **Not this track** |
| T142 archive / connector cursor / … | **Not related** (no neighbors-order overlap) |

---

## 10. Implement order (on go)

1. Phase 0 re-verify HEAD / deferred / #208 still empty / live pretty still dump-first / JSON still F9
2. FEATURE TX
3. Red AC1 helper + AC13 strip + AC3 hermetic pretty
4. Green: pretty path calls prefer; JSON untouched
5. Red/green AC4 JSON freeze; AC2 rstest (incl. four-tier case 6); AC14 limit; F31 INSERT helper
6. Stay-green AC5–AC9
7. Docs AC10/AC11
8. Clippy + nextest (`--features graph` where required) + deny/audit
9. Manual AC12
10. Phase-1 review → codex-review
11. Publish: push `track/T293-*` → PR → watch GHA `CI` green → squash-merge → prune

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| PATH until `cargo install --features graph` | F15 |
| Live chrome-only 1-hop still dump-first | F25 — honest; `graph session` is the remediator |
| Session PREVIEW first-line is still chrome when the pin is buried later in the session | F6 — do not rewrite caption |
| N+1 `get_session_memories` on huge sessions | T278 residual; not this track |
| Hierarchy pretty still id-only | F11 |
| Sparse E/N ~0.12 | T300 |
| T294–T299 | Not stolen |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/graph.rs` | `prefer_authority_neighbor_rows` + rank/strip helpers; pretty path only; units AC1/AC2/AC13 |
| `crates/ai-brains-cli/src/main.rs` | GraphCommands `after_help` dual-truth sentence only (no flag changes) |
| `crates/ai-brains-cli/tests/graph_human_cli.rs` (prefer) or `graph_live_projection.rs` | AC3/AC4/AC14 hermetics (`test(graph)`); **new** `seed_memory_projection` (F31) |
| `Docs/CAPABILITIES.md` | graph neighbors prefer-fill / JSON freeze |
| `Docs/PROTOCOL-COMPAT.md` | §5 array-order pretty human-only note |
| `Docs/OPERATIONS.md` | extend existing neighbors paragraph `:948` (T246/T262/T278) |
| `CHANGELOG.md` | on implement |
| `conductor/conductor.md` | Planned now; Completed on implement only |
| `conductor/deferred.md` | this planning table (now); closeout later |
| `conductor/tracks/README-T285-T300-CLI-QUALITY.md` | T293 Planned |

**Do not touch:** `projector.rs`; `queries.rs` `get_neighbors`; `ranking.rs` body; `session_chrome.rs`; `memory.rs` `prefer_fill_authority`; `graph_density.rs` floors; `doctor.rs`; `recall.rs`; `pin.rs` write; CLI `preflight.rs`; `project.rs`; `forget.rs` production; `.github/workflows/ci.yml`.

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` (HEAD `fe9fb89`) + `opencode-review.md` (HEAD `fe9fb89`). Fold-in HEAD `fe9fb89` on `main`. Live verify: `neighbors` `:361` pretty `:388` JSON `:401`; `sort_neighbor_hits` `:140`; `format_session_neighbor_preview` `" · "` `:256`; `pretty_neighbor_rows` `:308`; `seed_node` `:104` / `seed_edge` `:119` / **no** projection insert helper; T278 DROP COLUMN `:187`; MemoryPinned INSERT columns `memory.rs:22`; clap Neighbors `:2846` dispatch `:5121`; PROTOCOL-COMPAT graph `:94` array-order `:95` (`:103` is `project scan-roots`, not neighbors); OPERATIONS neighbors `:948`; `classify_pin_kind` `:122`. Hotspot `project.rs` **#1** (3.932). Pins **snapshot — re-verify at execute** (clap lock 4.6.1 / crates.io 4.6.6; rusqlite 0.39.0; no clap 5). Last merged PR still **#208**. **No T301.** Fold-in preflight: Pinned **4048** / in-context **1/2/1** / word **1416** (volatile). Doctor **4** warn; :8081/:8083 ok at fold-in (OpenCode: unreachable — volatile).

### Pins locked by fold-in

1. **F4 / AC13 (Agy m1 / OpenCode m2):** `split_once(" · ")` exact; dots in first-line stay in remainder.
2. **F1 (Agy m2):** `sort_by_key` on `(rank, original_index)`; no `sort_unstable_by_key`.
3. **F31 / §5.4 (OpenCode m1):** new `seed_memory_projection`; do **not** reuse T278 DROP COLUMN as a seeder.
4. **AC2 case 6 (Agy O2 / OpenCode O2):** four-tier mixed exact order.
5. **AC3 (OpenCode O3):** first pretty data row is authority (DECISION/needle) and **not** dump UUID `00000000-…0001`.
6. **F14 / AC11 (Agy O1 / OpenCode O1):** PROTOCOL-COMPAT **`:95`** pretty note; OPERATIONS **extend `:948`**.
7. **F5 F-id slip:** chrome-only is **F25**, not F32.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** `" · "` split | **Already** F4/AC13; **tightened** `split_once` + AC13 dots case |
| Agy | **m2** stable `sort_by_key` | **Already** F1 original index; **folded** `sort_by_key` / no unstable |
| Agy | **O1** PROTOCOL-COMPAT array-order | **Already** F14/AC11; **tightened** `:95` |
| Agy | **O2** four-tier rstest | **Folded** AC2 case 6 |
| OpenCode | B / M | None filed |
| OpenCode | **m1** no projection insert helper | **Folded** F31 / §5.4 |
| OpenCode | **m2** `" · "` split | **Same as Agy m1** |
| OpenCode | **O1** PROTOCOL-COMPAT `:94–95` + “`:103` neighbors” | **Partial** — `:95` folded; **decline** `:103` (live `project scan-roots`) |
| OpenCode | **O2** four-tier rstest | **Same as Agy O2** |
| OpenCode | **O3** exact AC3/AC4 UUID | **Already** AC4 dump UUID; **folded** AC3 first-row not dump id |
| OpenCode | HEAD `80a10d9` vs `fe9fb89` / word 280→314 | **Snapshot only** — plan preflight refreshed; not DoD |
| both | last-PR #208 Cursor | **Affirm F19** — no T301 |
| both | deferred T294–T300 / 2-hop / H2 / clap 5 | **Affirm** |

No Blockers. No Majors. No new placeholder minted. Do **not** edit `*-review.md`.
