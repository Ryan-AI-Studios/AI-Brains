# T317 — Graph neighbors: RECALLS noise vs signal

- **Track ID:** T317-GraphNeighborsRecalls
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / GRAPH
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `graph neighbors` 6/**5**; 19 `RECALLS` edges on a memory; `hierarchy` empty `synthesized_from`. Series README `README-T312-T324-CLI-DOGFOOD.md`.
- **Depends on:** T293 ✅ human prefer-authority 1-hop; T246 ✅ pretty table + JSON freeze; T262 ✅ live projection + RECALLS survive rebuild; T278 ✅ session PREVIEW; T67 `RECALLS` edges (do **not** delete the event type); T68 `SYNTHESIZED_FROM` from nightly
- **Blocks / feeds:** Graph as a daily tool. Sparse floors remain **T308** (frozen). Hierarchy table restyle / 2-hop stay declined.
- **Absorbs:** Audit RECALLS spam + empty hierarchy honesty (named next-step)
- **Not absorbed (DoD):** T293 dump-session reorder (already shipped); live `graph rebuild`; floor retune; JSON `kind`/`preview` keys; 2-hop pretty; T312 recall graph-hop; `--label` / `--exclude`; deleting projector `RECALLS`
- **Research date:** 2026-08-28 (plan-write product HEAD `dae7df3` T313 `#233`). Snapshot — **re-verify at execute**.
- **Ledger:** planning DOCS TX `0db2a64d-6ae6-4c25-b2fc-3a6db62d0dfa`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install`. Do **not** live `graph rebuild`. Do **not** grow hotspot `project.rs` / `sync.rs`. Do **not** edit `projector.rs` / `queries.rs` `get_neighbors`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Neighbors first page is signal.** Human `graph neighbors` must not be an undifferentiated wall of `RECALLS` session rows. After T293 prefer-authority, cap pretty `RECALLS` at **3** and print `+N more RECALLS`. Keep every non-`RECALLS` 1-hop. Keep at least the capped RECALLS (T67 meaning stays visible).
2. **Hierarchy empty is named.** Pretty leaf keeps `No SYNTHESIZED_FROM children (leaf).` and adds `next: ai-brains nightly --status`. Do **not** point at `graph update` / `graph rebuild` (they cannot invent synth edges). Do **not** fake `SYNTHESIZED_FROM`.
3. **JSON 1-hop contract stays.** `--format json` / pipe `auto` keep T246 keys `{memory_id, neighbors:[{external_id,label,direction}]}` and T246 F9 array order **direction → label → id**. No pretty cap. No `kind` / `preview`. Hierarchy JSON `{root, synthesized_from}` empty array on leaf unchanged.
4. **North star.** Capture independence: pretty overlay on existing edges. No new events. No hidden CoT. No projector rewrite.

This unblocks daily CLI: operators who paste a pin id into `graph neighbors --format human` must not conclude the graph is only RECALLS spam, and operators who run `graph hierarchy` on a real memory must not treat an honest leaf as a broken command.

---

## 2. Live baseline (re-scan 2026-08-28)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `dae7df3` `feat(cli): T313 sync query rescued-heading provenance (#233)`. Tree **CLEAN**. Branch `track/T317-graph-neighbors-recalls` (plan). `origin/main` = `dae7df3` (ahead **0** at plan-write). |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. **T293 is on PATH** (prefer-authority). **T312 / T315 / T314 / T313 are not.** T317 RECALLS-cap hole **is** on PATH (same pretty table). **Do not `cargo install`.** Tests/manual AC use `cargo run --features graph` / hermetic / units. |
| `preflight --summary` (PATH) | Pinned **4549**. In-context **0/0/0**. `Total Word Count: 785` (PATH-behind T315 `Budget window words:`). **Not this DoD.** |
| `graph update --format human` | `status: sparse` `density: warn` `nodes: 64273` `edges: 26618` `pinned_memories: 51167` `memory_nodes: 40361` `edge_node_ratio: 0.4141396854044467`. Note: typed-lineage floor 0.50. Remediator omitted (T308). **Do not retune floors. Do not live rebuild.** |
| Audit dump `431f6505-50d7-5176-8cda-f8ba2534fe14` | Pretty **Neighbors of … (11)** — **11/11** `in RECALLS` / `KIND session`. PREVIEW filled (T278): `## Objective` × many, plus the audit dump itself. JSON `n=11` `RECALLS=11`. T293 prefer-authority is a no-op here (all rank-3 dump sessions). **This is the hole.** Audit 2026-08-27 said **19**; live now **11** on this id — cardinality moved; the product hole is “first page is RECALLS-only,” not the exact 19. |
| Hierarchy on the same id | Pretty: `No SYNTHESIZED_FROM children (leaf).` **No `next:`.** JSON `{"root":"431f6505-…","synthesized_from":[]}`. Honest leaf that still looks like a missing feature. |
| `graph neighbors --help` | `--format` default **`auto`**, tokens `auto\|pretty\|human\|text\|json`. `--limit`. Parent after_help: TTY table; JSON compact; T278 PREVIEW; T293 dual-truth. **No** RECALLS-cap sentence. **No** `--label`. |
| Last GitHub PR | [#233](https://github.com/Ryan-AI-Studios/AI-Brains/pull/233) T313. `mergedAt` **2026-08-28T12:28:19Z**. Issue comments **[]**. Review comments **[]**. Reviews **[]**. Commit comments **[]**. PR body Cursor Bugbot block is **overview / Low Risk** (no defect). **last-PR Cursor: N/A empty.** `#230` Bugbot already **T325**. Open PRs: **none**. **No T326.** |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX). Hotspot **#1** `project.rs` (3.724) — **do not touch.** `sync.rs` **#2** (3.528) — **do not touch.** `governed_common.rs` **#3**. CLI `preflight.rs` #7 — **do not touch.** `graph.rs` **not** top-10 (**1539** lines). |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why the first pretty page is still RECALLS spam

| Layer | Truth |
|-------|--------|
| 1-hop of a pin is sessions | `MemoryPinned` projector (`projector.rs:70–81`) adds session→memory `RECALLS`. Live 1-hop of the audit dump is **only** incoming `RECALLS` / `kind=session`. |
| T293 prefer-authority does not cap | `prefer_authority_neighbor_rows` (`graph.rs:364–370`) **same length; no drops**. When every 1-hop is a dump session, ranks are all **3** and F9 UUID order wins. Live 11 rows all stay on the page (`--limit` default 50). |
| T278 filled PREVIEW | Captions are scannable **and** they say `## Objective`. Captions are not the hole; **cardinality of RECALLS** is. |
| `--limit` is the wrong knob | Default 50 > 11. Lowering the global pretty default would hide non-RECALLS on mixed 1-hop. Cap the **label**, not the table. |
| JSON must list all | Scripts / `jq` count RECALLS. T246 F9 + T293 F2 freeze. Human cap is the dual-truth (clig.dev: changing human output is usually OK). |
| Hierarchy leaf is correct data | Most memories are not synthesis parents. T246/T262: leaf has **no** `graph update` / `graph rebuild` remediator (`pretty_hierarchy_leaf__no_graph_update_or_rebuild__ac9`). The hole is **unnamed**: operators cannot tell “leaf by design” from “command broken.” Named next-step is `nightly --status` (T68 is how synth edges appear), **not** rebuild. |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| CLI neighbors | `graph.rs` `neighbors` **`:524–571`** | `sort_neighbor_hits` then pretty vs JSON. Pretty: `pretty_neighbor_rows` → `prefer_authority_neighbor_rows` → `format_neighbors_pretty(..., clamp_list_limit(limit))`. **No RECALLS cap today.** |
| Pretty rows | `pretty_neighbor_rows` **`:309–333`** | kind + preview. Session T278 caption. |
| Pretty format | `format_neighbors_pretty` **`:158–184`** | Header `Neighbors of {id} ({rows.len()})`. `… and {n} more` from `--limit` only. **3-arg today.** |
| JSON emit | `format_neighbors_json` **`:186–194`** | `NeighborsOutput` `{memory_id, neighbors}`. Compact. |
| JSON sort | `sort_neighbor_hits` **`:141–148`** | direction→label→id. **Stay green.** Pretty must not change JSON. |
| T293 prefer | `prefer_authority_neighbor_rows` **`:364`** | Stable; no drops. **Stay green; run before cap.** |
| Hierarchy pretty | `hierarchy` **`:573–647`** | Empty pretty: `pretty_hierarchy_leaf()` `:125–127` exact `No SYNTHESIZED_FROM children (leaf).` **No `next:`.** |
| Hierarchy JSON | `HierarchyOutput` **`:22–25`** | `{root, synthesized_from}`. Leaf = `[]`. Unit `:1345–1356`. |
| T246 AC9 | `pretty_hierarchy_leaf__no_graph_update_or_rebuild__ac9` **`:1321–1328`** | Leaf / no-neighbors / empty-session must **not** contain `graph update` or `graph rebuild`. **Stay-green that forbid.** New next-step must be nightly. |
| Hermetic leaf | `tests/graph_human_cli.rs:532` | `contains("No SYNTHESIZED_FROM children (leaf).")` — **keep first line** so this substring stay-green. |
| T262 pretty RECALLS | `tests/graph_live_projection.rs:99` | `pretty_out.contains("in") && pretty_out.contains("RECALLS")` — 1 edge, cap 3, **stay-green**. |
| Rebuild RECALLS | `graph.rs:1017` / `:1065` | Pin RECALLS survive blocked rebuild. **Do not drop the edge type.** |
| `get_neighbors` | `queries.rs` **`:62–102`** | UNION ALL in+out; no ORDER BY; no kind. **Do not change signature or SQL.** |
| Projector pin | `projector.rs` `MemoryPinned` **`:70–81`** | `relation: "RECALLS"`. **Do not rewrite.** |
| clap | `main.rs` `GraphCommands::Neighbors` **`:3244–3251`**; enum after_help **`:3227–3229`** | default `auto`; five tokens; `--limit`. **No new flags.** |
| Limit | `clamp_list_limit` / `format_neighbors_pretty` `take(shown)` | Pretty default 50 max 200. JSON unlimited unless `--limit`. Cap **before** `--limit` take. |
| PROTOCOL-COMPAT | `:94–95` | Keys unchanged. Array order T293 pretty prefer / JSON F9. T262: “leaf / empty edges have no `next:`” — **pretty hierarchy leaf is the lift this track documents.** |
| CAPABILITIES | command **`:104`** / **`:464–465`** | T293 prefer-fill. Hierarchy: “honest leaf … **no remediator**.” **DoD docs.** |
| OPERATIONS | neighbors paragraph **`:960`** | “Honest leaf / no-neighbors / empty-session print no `next:`.” **Extend**; supersede **leaf only**. |
| Hotspots | `project.rs` #1 3.724 | **Do not touch.** Helpers in `graph.rs` (not top-10; **1539** lines). Split sibling only if production net ≥80. |

### 2.4 Dependency / standards research (2026-08-28) — snapshot; re-verify at execute

| Pin / source | Workspace / live | Action |
|--------------|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** checksum `1ddb117e…` / crates.io **4.6.6** (2026-08-06) / GitHub **v4.6.6** / **no clap 5** | **No bump.** No new flags. Additive after_help only. |
| `serde_json` | lock **1.0.150** | **No bump.** Compact JSON keys frozen. |
| `rusqlite` | workspace exact **0.40.2** | **No bump.** No extra SQL. |
| `uuid` | workspace **1.13** | **No bump.** |
| rustc / edition / nextest | **1.95.0** / **2024** / (workspace gate on go) | Unchanged. |
| workspace version | **0.1.3** | **No bump.** |
| New crates | — | **Zero.** No petgraph / regex / comfy-table. |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Human CLI may change; JSON stays stable | [clig.dev Output](https://clig.dev/#output) (fetched 2026-08-28): humans first; `--json` for structure; **changing human output is usually OK** | Pretty RECALLS cap is human-only. JSON F9 list stays complete. |
| Do not lie by omission | [clig.dev Saying (just) enough](https://clig.dev/#saying-just-enough) + Output | Hidden RECALLS **must** have a footer `+N more RECALLS`. Header `{full_hop_count}` must stay the real 1-hop cardinality. |
| Future-proof additive | [clig.dev Future-proofing](https://clig.dev/#future-proofing) | Do not add required JSON keys. Dual-truth in after_help (T293 pattern). |
| Metadata relations hide by default | [sift-kg visualization](https://www.mintlify.com/juanceresa/sift-kg/guides/visualization) (2026-03): `MENTIONED_IN` **hidden by default** (metadata clutter) | `RECALLS` is provenance metadata (T67 session-recalled-this). Cap display; do not delete the edge. |
| Typed graphs; nothing stripped from the store | T278/T293 cite knowgraph USER_GUIDE §10: evidence-backed ranking; **nothing is ever stripped** | Cap pretty; JSON keeps every 1-hop. |
| Neighbor lists need ranked captions | Neo4j Browser styling (T293 cite); Wikidata Help:Ranking: preferred first, deprecated stay listed | T293 already ranks; this track **caps** the long tail. |
| CLI neighbor APIs return full 1-hop JSON | [pm-graph neighbors](https://www.pkgstats.com/pkg:pm-graph) (2026-08): JSON lists every relationship; human views filter separately | JSON uncapped. No `--label` required this track. |
| T180 P-CLI | Additive extras OK; compact↔pretty without a flag is breaking | Pretty **row cap** is not a key change. JSON array **frozen**. Document the human-only lift on PROTOCOL-COMPAT array-order + next-action rows. |
| SQLite / SQLCipher / schtasks | N/A — no new SQL, no tasks | N/A (written). |

**Could not verify:** live 1-hop that mixes `RECALLS` with `SYNTHESIZED_FROM` / `IN_PROJECT` on PATH (audit dump is RECALLS-only). Hermetic mixed fixture is SoT. Live Manual on `431f6505-…` is pass-with-observed-data for the RECALLS-only shape (header still 11; 3 rows; `+8 more RECALLS`).

**ledgerful / ai-brains:** `preflight --summary` Pinned **4549** / 0/0/0 / word **785**. `graph update` sparse E/N **0.414**. `graph neighbors` 11× RECALLS on `431f6505`. `graph hierarchy` unnamed leaf. `ledgerful doctor` 4 warn; 0 pending / 0 drift; `index --incremental` 0 files; `search "prefer_authority_neighbor_rows"` → `graph.rs:364`; `search "pretty_hierarchy_leaf"` → `:125` / AC9 `:1321`; `scan --impact` CLEAN at `dae7df3`; hotspots `project.rs` #1. Semantic recall of “graph neighbors RECALLS” still returns review dumps (PATH T312 not installed) — not SoT. `ledgerful ledger search --json -- "graph neighbors RECALLS"` → `[]` (phrase miss; T313 heading is not this DoD).

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `0db2a64d`. Implement starts a **FEATURE** TX. |
| **F1 — Human-only RECALLS cap** | After T293 `prefer_authority_neighbor_rows`, pretty path calls `cap_recalls_pretty_rows`. Keep **all** non-`RECALLS` rows. Keep the first **`RECALLS_PRETTY_CAP = 3`** `RECALLS` rows in T293 order. Hidden count = remaining `RECALLS`. Label match is exact `"RECALLS"` (projector string). Applies to `pretty`/`human`/`text` and TTY `auto`. |
| **F2 — Hierarchy leaf next-step** | `pretty_hierarchy_leaf()` becomes two lines, **first line exact** `No SYNTHESIZED_FROM children (leaf).` Second line exact `next: ai-brains nightly --status`. Do **not** mention `graph update` or `graph rebuild` (AC9 stay-green). Do **not** suggest mutating `ai-brains nightly` without `--status`. Do **not** claim this memory *should* have synth children (most are correctly leaves). |
| **F3 — JSON freeze** | `sort_neighbor_hits` remains the JSON order (T246 F9). Keys T246. Compact. **No** pretty cap on JSON. Pipe `auto` is JSON. Hierarchy JSON empty array on leaf unchanged. **No** `next_step` / `recalls_hidden` JSON keys. |
| **F4 — 1-hop only** | Do **not** add 2-hop sibling memories (T278 F18 / T293 F3). Do **not** change `get_neighbors` SQL/signature. Do **not** walk `get_session_memories` to emit extra rows. |
| **F5 — T293 prefer stays first** | Cap **after** prefer-authority so an authority `RECALLS` session occupies a kept slot. Prefer still **no drops**. T293 F1/F5 “dumps stay” is **superseded for pretty RECALLS cardinality only**; dumps of other labels still stay; hidden RECALLS remain in JSON. |
| **F6 — T278 PREVIEW freeze** | Session cell remains `{n} memories · first line`. Cap does not rewrite captions. |
| **F7 — Projector / density / rebuild freeze** | No `projector.rs` rewrite. No T213 floor change (`MIN_EDGE_NODE_RATIO=0.50`). No live `graph rebuild`. `graph update` JSON/human unchanged. Do **not** delete `RECALLS` from the projector (T67 / rebuild tests). |
| **F8 — No new clap flag** | No `--label` / `--exclude` / `--recalls-cap` / `--pins-only`. Silent human cap (T293 F8 analog). Format tokens / default `auto` frozen. `--limit` still the row cap **after** the label cap. |
| **F9 — Header tells the truth** | Pretty header `Neighbors of {id} ({full_hop_count})` uses **pre-cap** 1-hop length (same cardinality JSON would list without `--limit`). Capped table may be shorter. Footer `+{n} more RECALLS` when hidden > 0. Existing `--limit` line `… and {n} more` still applies to the **kept** list. |
| **F10 — NeighborHit / recall freeze** | Do not add serde fields. Do not change `get_neighbors`. T285 `recall_rank_v2_graph.rs` stays green without edits. T262 AC6/AC7 stay-green. |
| **F11 — Session / update / rebuild freeze** | Do **not** cap those commands. Neighbors pretty + hierarchy **leaf copy** only. `pretty_no_neighbors` / `pretty_session_empty` stay without `next:` (not this hole). |
| **F12 — Feature-off freeze** | Exit **2** + `FEATURE_UNAVAILABLE` stays. |
| **F13 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. `tempfile::tempdir` hermetic. **AC2 required rstest `#[case]`** for the cap helper. |
| **F14 — Docs** | CAPABILITIES graph row + command rows: human caps RECALLS at 3 + footer; JSON lists all; hierarchy leaf `next: ai-brains nightly --status`. PROTOCOL-COMPAT §5 neighbors keys row **and** array-order row **and** T262 next-action clause: pretty hierarchy leaf may print nightly `--status`; JSON stays empty array; still no graph update/rebuild on leaf. GraphCommands `after_help` one dual-truth sentence. OPERATIONS **extend** `:960` (do **not** add a second graph block). CHANGELOG on implement. |
| **F15 — PATH** | Soft. Source/hermetic SoT. Do not `cargo install` as implement. T293 already on PATH — Manual AC uses `cargo run --features graph` so the cap is visible. |
| **F16 — Capture independence** | Pretty cap + leaf string. No models, embeddings, new events, ledgerful writes. |
| **F17 — Isolation hotspots** | Do not grow `project.rs` / `sync.rs` / `forget.rs` production / CLI `preflight.rs` / `personal.rs` / `briefing.rs` / `session_chrome.rs` / `ranking.rs` / `projector.rs` / `queries.rs` / `doctor.rs` / `.github/workflows/ci.yml`. |
| **F18 — File growth** | `RECALLS_PRETTY_CAP`, `cap_recalls_pretty_rows`, and `format_neighbors_pretty` arity live `pub(crate)` in `graph.rs`. **Not** `pub`. **Not** re-exported from `commands/mod.rs`. Split `graph_neighbors_pretty.rs` only if production net ≥80. Do not relocate T300 rebuild helpers. |
| **F19 — last-PR Cursor** | #233 empty → **N/A**. `#230` already **T325**. **No T326.** |
| **F20 — Decline peers** | T316 memory-list; T318 backup list; T319 handle UUID; T320 unified status; T321 safety sync; T322–T324 T311 residuals; T325 F8 recency; T313 Completed `#233`; T312/T314/T315 Completed. T307 Blocked. T308 floors. |
| **F21 — Standing declines** | T240 F2; T263 H2; clap 5; DTO new required keys; Cargo default-on graph; floor retune; csrf; FTS schema split. |
| **F22 — ISSUES.md** | Does not exist. Debt is `deferred.md`. |
| **F23 — Cross-model** | FEATURE (operator pretty contract). After Phase-1 review clean, run read-only `codex-review`. |
| **F24 — Stop-before** | Even after go: no live `graph rebuild`; no `.env` write; no extra `policy bootstrap`; no `retention apply --confirm`; no schtasks mutate; no `cargo install`; no mutating `nightly` without owner. |
| **F25 — RECALLS-only 1-hop** | 11 (or N) dump RECALLS → show 3 + `+{N-3} more RECALLS`. Header still `(N)`. 0 RECALLS → no footer (today’s table). 1..=3 RECALLS → no footer (T262 1-edge stay-green). |
| **F26 — Dual-truth after_help** | Human table caps RECALLS at 3 and prints `+N more RECALLS`; JSON lists all 1-hop (direction→label→id). Hierarchy leaf pretty may add `next: ai-brains nightly --status`. |
| **F27 — Stay-green** | T246 JSON keys/dir; T262 AC6 JSON incoming RECALLS + AC7 pretty `in`+`RECALLS`; T278 AC3 PREVIEW; T293 prefer rstest + dump-then-decision; `sort_neighbor_hits` unit; feature-off exit 2; `format_neighbors_json` key units; AC9 **no** `graph update`/`graph rebuild` in leaf / no-neighbors / empty-session; `graph_human_cli.rs` leaf `contains("No SYNTHESIZED_FROM children (leaf).")`; rebuild RECALLS survive. |
| **F28 — PowerShell** | `;` not `&&`. |
| **F29 — Identity stdout** | JSON still `note_machine_stdout` (T257). Pretty does not. |
| **F30 — Do not group into one row** | Do **not** collapse RECALLS into a single `RECALLS · 11 sessions` summary (loses T278 captions of the 3 kept). Cap + footer. |
| **F31 — `format_neighbors_pretty` arity** | Today 3 args `(id, rows, limit)` and header uses `rows.len()`. After this track: `(id, rows, limit, full_hop_count, recalls_hidden)`. Existing unit `format_neighbors_pretty__incoming_and_outgoing__header_in_out_kinds` passes `full_hop_count=2`, `recalls_hidden=0` (behavior stay-green). Hidden footer is a new assert (AC5). |
| **F32 — Cap walk is stable** | Walk T293 order once. Non-RECALLS always kept. RECALLS kept until cap then counted as hidden. Do **not** `sort_unstable`. Do **not** re-sort after cap. |

---

## 4. Acceptance criteria

| ID | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Cap helper: 11 `RECALLS` → 3 kept, hidden **8** | `cap_recalls_pretty_rows__eleven_recalls__keeps_three_hidden_eight` |
| **AC2** | rstest: 0 / 1 / 3 RECALLS → hidden **0**, same length; 4 RECALLS → kept 3 hidden 1 | `cap_recalls_pretty_rows__at_or_under_cap__unchanged` |
| **AC3** | Mixed: 2 `SYNTHESIZED_FROM` + 5 `RECALLS` → kept **5** (2+3), hidden **2**; all non-RECALLS present | `cap_recalls_pretty_rows__mixed_labels__keeps_all_non_recalls` |
| **AC4** | After T293, authority `RECALLS` session is among the 3 kept when dumps follow | `cap_recalls_pretty_rows__authority_recalls__kept_before_dumps` |
| **AC5** | Pretty format: header `(11)` + 3 data rows + `+8 more RECALLS` | `format_neighbors_pretty__recalls_hidden__header_total_and_footer` |
| **AC6** | `recalls_hidden=0` → no `+N more RECALLS` substring; header uses `full_hop_count` | `format_neighbors_pretty__no_hidden__no_recalls_footer` |
| **AC7** | Leaf string two lines; second is `next: ai-brains nightly --status`; first line still `No SYNTHESIZED_FROM children (leaf).` | `pretty_hierarchy_leaf__nightly_status_next` |
| **AC8** | AC9 stay-green: leaf / no-neighbors / empty-session still omit `graph update` and `graph rebuild` | existing `pretty_hierarchy_leaf__no_graph_update_or_rebuild__ac9` |
| **AC9** | JSON neighbors of a 4+ RECALLS fixture lists **all** RECALLS (no cap) | hermetic `graph_neighbors__json__no_recalls_cap` |
| **AC10** | Docs: CAPABILITIES `:104` + `:464–465`; PROTOCOL-COMPAT `:94–95`; OPERATIONS `:960`; GraphCommands `after_help`; CHANGELOG | review + `rg` |
| **AC11** | Manual `cargo run --features graph -- graph neighbors 431f6505-50d7-5176-8cda-f8ba2534fe14 --format human` (or current live N): header `(N)`, **3** RECALLS data rows, `+{N-3} more RECALLS`. PATH-behind not a fail. Pass-with-observed-data if N≤3 (record N). | plan evidence |
| **AC12** | Manual same id `graph hierarchy --format human`: first line leaf; `next: ai-brains nightly --status`. JSON still `synthesized_from: []`. | plan evidence |
| **AC13** | `git diff --stat` production names stay under `crates/ai-brains-cli/` (`graph.rs` and tests/docs). Do **not** pass `C:\dev\Ledgerful` to `git diff`. `projector.rs` / `queries.rs` **empty** in the product diff. | review |
| **AC14** | Hermetic pretty neighbors with 4+ seeded `RECALLS` shows footer and keeps a non-RECALLS row if seeded | `graph_neighbors__human__caps_recalls_with_footer` in `graph_human_cli.rs` (or graph.rs unit + one CLI if cheaper — CLI required if format path is only in `neighbors()`) |
| **AC15** | T262 AC7 pretty still contains `in` and `RECALLS` | stay-green |
| **AC16** | Feature-off `graph neighbors` still exit 2 | stay-green |
| **AC17** | `--limit 2` after cap: `… and {n} more` still works on the **kept** list; RECALLS footer still uses hidden-from-cap not limit remainder | unit `format_neighbors_pretty__limit_and_recalls_hidden__two_footers` |

---

## 5. Design notes

### 5.1 Why cap 3, not `--limit` and not `--label`

`--limit` default 50 already lets 11 RECALLS through. A new `--label` flag would be T293 F8 + discoverability tax. Silent cap matches T287/T293 silent prefer-fill. **3** matches T271 token cap; T262 1-edge fixtures stay under the cap.

### 5.2 SoT helpers

```rust
pub(crate) const RECALLS_PRETTY_CAP: usize = 3;

pub(crate) fn cap_recalls_pretty_rows(
    rows: &[PrettyNeighborRow],
) -> (Vec<PrettyNeighborRow>, usize) {
    let mut kept = Vec::with_capacity(rows.len());
    let mut recalls_kept = 0usize;
    let mut recalls_hidden = 0usize;
    for row in rows {
        if row.label == "RECALLS" {
            if recalls_kept < RECALLS_PRETTY_CAP {
                kept.push(row.clone());
                recalls_kept += 1;
            } else {
                recalls_hidden += 1;
            }
        } else {
            kept.push(row.clone());
        }
    }
    (kept, recalls_hidden)
}

pub(crate) fn pretty_hierarchy_leaf() -> String {
    "No SYNTHESIZED_FROM children (leaf).\nnext: ai-brains nightly --status".to_string()
}
```

`neighbors()` pretty arm (order frozen):

1. `pretty_neighbor_rows`
2. `prefer_authority_neighbor_rows` (T293)
3. `full_hop_count = rows.len()`
4. `(kept, recalls_hidden) = cap_recalls_pretty_rows(&rows)`
5. `format_neighbors_pretty(id, &kept, clamp_list_limit(limit), full_hop_count, recalls_hidden)`

Footer line exact: `+{recalls_hidden} more RECALLS` (no period). Place **after** the optional `… and {n} more` limit line.

### 5.3 Why nightly `--status`, not rebuild

T68: `MemorySynthesized` during nightly creates `SYNTHESIZED_FROM`. T262: rebuild cannot invent a `turn_id` that was never logged; it also cannot invent synth edges that were never appended. T246 AC9 already forbids `graph update` / `graph rebuild` on the leaf. `nightly --status` is read-only orientation (last run / errors). Do **not** auto-run nightly.

### 5.4 Why not collapse RECALLS

A single grouped row would hide T278 captions of the kept 3. Cap keeps three scannable sessions; footer names the rest.

---

## 6. Non-goals

- Deleting `RECALLS` from the projector or event type (T67).
- Floor retune / live `graph rebuild` / Cargo graph default-on.
- 2-hop pretty rows / `get_neighbors` SQL filter.
- `--label` / `--exclude` / `--recalls-cap` clap flags.
- JSON `kind` / `preview` / `recalls_hidden` / `next_step`.
- Prefer-fill on `graph session` / `graph update` / `graph rebuild`.
- `next:` on `pretty_no_neighbors` or empty session (not this hole).
- T312 recall rank / T325 PreferRecency / T316 memory-list / T318–T324.
- clap 5 / new crates / silent `.env` / `cargo install` / live pin.

---

## 7. Verification plan (TDD)

Red first (must fail while pretty prints all 11 RECALLS and leaf has no `next:`):

1. `cap_recalls_pretty_rows__eleven_recalls__keeps_three_hidden_eight` (AC1)
2. `cap_recalls_pretty_rows__at_or_under_cap__unchanged` (AC2 rstest)
3. `cap_recalls_pretty_rows__mixed_labels__keeps_all_non_recalls` (AC3)
4. `format_neighbors_pretty__recalls_hidden__header_total_and_footer` (AC5) — **fails** on 3-arg helper / header `rows.len()`
5. `pretty_hierarchy_leaf__nightly_status_next` (AC7) — **fails** on one-line leaf
6. Hermetic AC14 `graph_neighbors__human__caps_recalls_with_footer`
7. Hermetic AC9 `graph_neighbors__json__no_recalls_cap` is **green-on-arrival** (JSON already lists all) — **write it** even though it passes on HEAD (T313 F24 analog)

Green: F1 cap + F9 arity + F2 leaf + `neighbors()` wire.

Stay-green: AC8 / AC15 / AC16 / T293 rstest / T246 JSON keys / rebuild RECALLS.

Manual AC11–AC12 on go. Docs AC10. No full workspace nextest as a plan gate.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Header `(3)` lies about 1-hop size | F9 `full_hop_count` pre-cap |
| Hidden RECALLS with no footer | F9 / AC5 exact `+N more RECALLS` |
| T293 dumps-stay misread as “do not cap” | F5 written supersede; JSON still full |
| AC9 leaf remediator accidentally `graph update` | F2 nightly `--status` only; AC8 stay-green |
| PROTOCOL-COMPAT “leaf has no next” rot | F14 lift **hierarchy leaf pretty only** |
| `graph.rs` 1539 grows | F18 split if production net ≥80 |
| `queries.rs` / projector “quick filter” | F4 / F7 / AC13 empty diff |
| PATH-behind false AC fail | F15 / AC11 `cargo run --features graph` |
| `#233` leftover dropped | F19 N/A empty; T325 already minted |
| `--limit` remainder vs cap remainder | AC17 two distinct footers |
| `graph_human_cli` leaf `contains` breaks | F2 keep first line exact |
| Live N on `431f6505` moves from 11 | AC11 pass-with-observed-data; hermetic SoT is AC1/AC14 |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-28.

| Item | Disposition |
|------|-------------|
| Audit `graph neighbors` RECALLS spam (19 edges; live 11 on `431f6505`) | **Absorb** F1 / F9 / AC1 / AC5 / AC11 / AC14 |
| Audit hierarchy `synthesized_from` empty | **Absorb** F2 / AC7 / AC12 — named nightly `--status`, not rebuild |
| T293 dump-session reorder | **Affirm shipped** F5; do not re-implement prefer |
| T293 F5 dumps stay / F8 no flags / F11 hierarchy freeze | **Partial supersede** F5 pretty RECALLS cap; **affirm** F8 no flags; **supersede F11 for leaf copy only** |
| T246 JSON keys/order / T262 next-action leaf has no graph remediator | **Affirm** F3 / AC8 / AC9 |
| T308 R1 live E/N ~0.41 | **Decline** floor change F7 |
| T278 F18 2-hop pretty | **Decline** F4 |
| T67 RECALLS event type | **Affirm keep** F7 |
| T316 memory-list preview | **Not stolen** |
| T318 backup list usable-first | **Not stolen** |
| T319 handle vs memory UUID | **Not stolen** |
| T320 unified status | **Not stolen** |
| T321 `safety sync` write | **Not stolen** |
| T322–T324 T311 residuals | **Not stolen** |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T313 Completed `#233` / T312 / T314 / T315 | **Not stolen** |
| T307 Blocked / T308 floors | **Not stolen** / **Decline** |
| T263 H2 / T240 F2 / clap 5 | **Decline** F21 |
| T92 pull/push / T298 device | **Decline** |
| last-PR Cursor `#233` | **N/A empty** F19 — **no T326** |
| last-PR `#230` F8 recency | **T325** already Pending |
| conductor/archive / cargo-audit allowlist | **Not related** |
| PATH T315 `Total Word Count` / T312 dump-first | **Not this DoD** |

---

## 10. Implement order (on go)

1. Phase 0 re-read `neighbors()` `:524–571`, `format_neighbors_pretty` `:158–184`, `pretty_hierarchy_leaf` `:125–127`, T293 prefer `:364`, AC9 `:1321`, `graph_human_cli.rs:532`, `queries.rs:62`, `projector.rs:70–81`; rescan deferred; FEATURE TX.
2. Red AC1–AC5 / AC7 units (must fail on uncapped pretty / one-line leaf). **Write AC9 JSON hermetic** (green-on-arrival).
3. Green F1 cap + F9 arity + F2 leaf; wire `neighbors()` order in §5.2; do not grow `project.rs`.
4. Stay-green AC8 / AC15 / AC16 / T293 / T246 JSON / rebuild RECALLS.
5. Hermetic AC14 + AC17.
6. Docs F14 / AC10.
7. Manual AC11–AC12 → AC13 → review → full gate → Complete.

---

## 11. Soft residuals (expected)

| Residual | Note |
|----------|------|
| PATH until `cargo install` | F15 |
| Live N on `431f6505` may move | Hermetic SoT; Manual observed-data |
| Dump-session PREVIEW still `## Objective` on the 3 kept | T278/T293 honesty; cap is cardinality |
| Hierarchy of a real pin stays a leaf | By design; next-step is orientation |
| Sparse E/N ~0.41 | T308 floors frozen |
| JSON still lists 11 RECALLS | F3 by design |
| T312 PATH dump-first | Other track |
| T325 F8 PreferRecency | Placeholder |

---

## 12. Touch map (expected)

| Site | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/graph.rs` | F1 cap; F9 arity; F2 leaf; `neighbors()` wire; units AC1–AC8 / AC17 |
| `crates/ai-brains-cli/tests/graph_human_cli.rs` | AC14 pretty cap hermetic; leaf still `contains` first line |
| `crates/ai-brains-cli/src/main.rs` | GraphCommands `after_help` dual-truth sentence only |
| `Docs/CAPABILITIES.md` | `:104` / `:464–465` |
| `Docs/PROTOCOL-COMPAT.md` | `:94–95` pretty cap + leaf next lift |
| `Docs/OPERATIONS.md` | `:960` extend |
| `CHANGELOG.md` | T317 Unreleased |
| `conductor/conductor.md` | T317 Planned (status **Pending**) |
| `conductor/deferred.md` | This absorption table |
| `conductor/tracks/README-T312-T324-CLI-DOGFOOD.md` | T317 Planned |

**Do not touch:** `project.rs`, `sync.rs`, `queries.rs`, `projector.rs`, retrieval ranking / CLI `preflight.rs`, contracts, daemon, Ledgerful sources, `.env`, schtasks.

---

## 13. AI fold-in disposition

Reserved for `/fold-in 317`. Inputs not present at plan-write.
