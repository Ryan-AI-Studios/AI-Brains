# T246 — Graph human CLI presentation

- **Track ID:** T246-GraphHumanCli
- **Status:** ✅ **Completed** 2026-08-13 PR #159 squash `06cdcde`
- **Category:** UX / FEATURE
- **Owner:** Grok
- **Source:** CLI audit 2026-08-11 P2 — `graph neighbors` **E7/Q6** raw JSON; hierarchy/session same class. README improvement: `graph update` pretty → T246.
- **Depends on:** T198 feature-off exit 2; T213 density + `graph update` JSON; T222 graph-on install; T232 capability remediations; T216 `preview_line` / `clamp_list_limit`; T224 `strip_role_prefix`; T243 `text`≡pretty honesty
- **Blocks / feeds:** Operators can scan 1-hop / hierarchy / session on a TTY. T248/T249 stay separate presentation tracks. Density/rebuild honesty stays T213/T232.
- **Absorbs:** deferred.md “Graph neighbors JSON-only”; placeholder F1–F4; README `graph update (pretty → T246)` as **opt-in** `--format human` (default JSON stays); T213 skill one-liner leftover (docs/skill only)
- **Not absorbed (DoD):** Auto `graph rebuild`; projector edge rewrite / LiveGraphHook pin-edge completeness; T213 F31 event↔graph freshness; density threshold flags / `GraphHealthOutput` contracts promote; rusqlite 0.40 `table_exists`; is-terminal → `std::io::IsTerminal` (T214 F24); clap 5; comfy-table/tabled; mermaid/ASCII-box tree; T248 retention human; T249 doctor `--summary`; Cargo default graph-on
- **Research date:** 2026-08-13 (live dogfood + PROTOCOL-COMPAT + CLIG + crate pins + T213/T232 residuals)
- **AI fold-in:** 2026-08-13 `C:\dev\AI-review.md` **T246** AI1 + AI2. No Highs. **Agree hard:** AI2 M1 `get_synthesized_hierarchy_with_depth` (reject CLI-local CTE); AI2 M2 F9 sort is a PROTOCOL-COMPAT §5 behavioral note. **Decline:** AI1 M3 `Some(other) => other` passthrough; AI1 single-line comma `graph update --format human`; AI1 remapped AC numbers. Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start T246-graph-human-cli --category FEATURE`)
- **Isolation:** Do **not** rewrite T213 assessor or T232 remediations. Do **not** run live `graph rebuild`. Do **not** change recall `--graph-boost` scoring. Do **not** add fields to JSON neighbor/hierarchy/session objects.

---

## 1. Objective

1. **Make graph reads human on a TTY.** `graph neighbors|hierarchy|session` print a scannable table/list when stdout is a TTY (or `--format pretty`/`human`/`text`). Compact JSON when piped or `--format json`.
2. **Tell the truth about empty.** Distinguish **no graph node** from **node exists but no edges/children**. Print a copy-paste next step. Missing UUID must not look like a successful empty object with no prose.
3. **Keep machine JSON stable.** Existing compact object keys stay. Sort collections for determinism. Pretty-only enrichment (kind, preview, depth indent, pretty `--limit`) must not appear as required JSON keys.
4. **Do not break `graph update` scripts.** Default remains T213 **pretty JSON**. Add opt-in `--format human` labeled lines. T74 smoke (piped JSON parse) stays green.
5. **Keep feature-off and capture independence.** Graph-off still exit **2** + `FEATURE_UNAVAILABLE`. No models, no new events, no contracts DTO, no new crates, no pin bumps.

---

## 2. Live baseline (re-scan 2026-08-13)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| `graph update` | Pretty JSON; `nodes=1314` `edges=104` `pinned_memories=15721` `memory_nodes=863` `E/N=0.079` `density=warn` `status=sparse` `remediation=ai-brains graph rebuild` |
| Doctor `graph_feature` | **ok** / `available` |
| Doctor `graph_density` | **warn** sparse + rebuild |
| Recent pin `5df056c0-…` (T247 complete) | `{"neighbors":[]}` — **no** prose, **no** next-step |
| Same pin hierarchy | `{"synthesized_from":[]}` — leaf / no synthesis, silent |
| Current session `26aab1e0-…` | `{"memories":[]}` |
| Older session `3b4e95b8-…` | **5** memory ids (usable pretty fixture) |
| Neighbors of `5a0e0a71-…` | One hit: `incoming` / `RECALLS` / session id `3b4e95b8-…` — UUID-only; **kind not shown** |
| Nil UUID `00000000-…` | **Identical** empty neighbors JSON as a real leaf pin |
| `graph *` flags | **No** `--format` / `--limit` |
| Feature-off | Still exit **2** + `FEATURE_UNAVAILABLE` (T198) — do not touch |

### 2.2 Why the audit scored neighbors 7/6

| Layer | Truth |
|-------|--------|
| Wire | Always `serde_json::to_string` compact. PROTOCOL-COMPAT pins **compact**. |
| Payload | `NeighborHit { external_id, label, direction }`. Hierarchy/session are string arrays. No kind, depth, preview. |
| Empty | Success + `[]` for miss, leaf, and sparse-projection alike. |
| Live graph | Sparse vs vault scale (T213). Recent pins often have **zero** 1-hop edges until rebuild / live hook. Human JSON `[]` looks like “this memory is isolated” rather than “projection lag / missing node.” |
| `graph update` | Already pretty-printed JSON and scored ≥8. Changing TTY default to prose would break TTY JSON consumers + T74 parse. |

T213/T232 closed **density honesty**. T246 closes **read-surface presentation**. It does **not** make the projection denser.

### 2.3 Code truth

| Site | Role |
|------|------|
| `commands/graph.rs` `neighbors`/`hierarchy`/`session` | Query + `println!(to_string)` only |
| `commands/graph.rs` `update` | `to_string_pretty` T213 `GraphHealthOutput` |
| `GraphCommands` in `main.rs` | Rebuild / Neighbors / Hierarchy / Session / Update — positional ids only |
| `GraphSearch::get_neighbors` | UNION ALL in+out; **no** `ORDER BY`; **no** `kind` |
| `get_synthesized_hierarchy` | Recursive `SYNTHESIZED_FROM` depth&lt;10; returns **ids only** (SQL has `depth`, discarded) |
| `get_session_memories` | Turn walk ∪ session `RECALLS`; ids only |
| `ai-brains-retrieval` recall | `get_neighbors` for `--graph-boost` — uses `external_id` only. **Freeze this API.** |
| `display_text::strip_role_prefix` / `memory::preview_line` | Existing pretty preview SOOT (80 chars) |
| `clamp_list_limit` | Default **50**, max **200** |
| `recall::resolve_format` | TTY pretty / pipe json; `text`≡pretty; unknown pass-through |
| Feature-off stubs | `main.rs` vault-free + dispatch — exit 2 + `GRAPH_REINSTALL_SOOT` |
| T74 `smoke.rs` | Piped `graph update` **must** parse as JSON |

### 2.4 Event / projection honesty (do not “fix” here)

`MemoryPinned` creates a memory node; a `RECALLS` session edge needs a session node + projector path. Graph-off daily binaries never run `LiveGraphHook`. Live vault is **sparse**. Empty neighbors on a fresh pin is often **true of the projection**, not a formatter bug. Pretty must say so and point at `graph update` (which already names rebuild when sparse).

---

## 3. Research (2026-08-13)

| Topic | Finding | Use in T246 |
|-------|---------|-------------|
| **[CLIG — Output](https://clig.dev/)** | Humans first; TTY heuristic; `--json` for structure; changing human output is usually OK; scripts should opt into `--json`/`--plain` | TTY pretty for the three reads; compact JSON piped / `--format json` |
| **PROTOCOL-COMPAT §5** | neighbors/hierarchy/session **compact**; `graph update` **pretty** JSON. Compact↔pretty without a flag is **breaking** | Add `--format`. JSON **keys** frozen. Update the inventory row to document TTY/pipe split |
| **T74 smoke** | Subprocess stdout is **not** a TTY → auto still JSON | Default `auto` is safe for smoke |
| **clap** | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** (fetched 2026-08-13). clap **5 not released** | `value_parser` on `--format`; **no bump**; clap 5 future-guard |
| **serde_json** | lock **1.0.150** / crates.io **1.0.151** | **No bump**; `to_string` compact; update stays `to_string_pretty` |
| **is-terminal** | lock **0.4.17**. Crate docs: prefer `std::io::IsTerminal` since Rust 1.70 | Keep crate (recall SOOT). Migration remains **T214 F24** / T249-class — not this track |
| **rusqlite** | workspace **0.39.0** (crates.io 0.40.2) | **No bump**; no `table_exists` |
| **comfy-table / tabled** | Would be new crates | **Forbidden**. Hand-roll columns like `project list` / `memory list` |
| **Recall format** | `text`≡pretty (T243). Unknown formats pass through | Graph: `text`/`human`≡pretty; **unknown → clap exit 2** (inventory, not search) |
| **whoami** | `auto` \| `human` \| `json` | Graph reads default **`auto`** |
| **memory list** | `clamp_list_limit` 50/200; preview 80 + role strip | Pretty `--limit` + preview |
| **Neighbor live shape** | Almost always `incoming RECALLS` → **session** id | Pretty **must** show `kind` or humans misread a session as a memory |
| **0022 CHECK kinds/labels** | Longest kind `source_version` (**14**); longest label `SYNTHESIZED_FROM` (**15**) | F2 column widths |
| **`project.rs` TTY** | Uses `std::io::IsTerminal`; six other files use the crate | F1 keeps crate (T214 F24 leftover) |
| **`get_session_memories` callers** | CLI + `live_graph.rs` + projector test | Do not change signature; CLI-local sort |

---

## 4. Findings (DoD)

| ID | Severity | Requirement |
|----|----------|-------------|
| **F1** | Hard | `neighbors` / `hierarchy` / `session` gain `--format: String` (not a clap enum — match recall/memory) default **`auto`**, `value_parser = ["auto","pretty","human","text","json"]` on **each variant** (not the parent `Graph`). Resolve: `pretty`\|`human`\|`text` → pretty; `json` → compact JSON; `auto` → TTY pretty else compact JSON. Probe `std::io::stdout().is_terminal()` via **`is_terminal::IsTerminal`** (majority SOOT; `project.rs` already uses `std::io::IsTerminal` — pre-existing, not this track). Invalid token → clap usage **exit 2** at parse time. **`resolve_graph_format` has no `other` passthrough** (AI1 M3 declined — recall passthrough exists for ndjson; graph has none). |
| **F2** | Hard | Pretty neighbors: title `Neighbors of <id> (n)` then header `DIR LABEL ID KIND PREVIEW` then rows. Column widths: **DIR 3**, **LABEL 16** (`SYNTHESIZED_FROM`=15), **ID 36**, **KIND 14** (`source_version`=14), **PREVIEW rest**. Format `{:<3} {:<16} {:<36} {:<14} {}`. **DIR** maps `incoming`→`in`, `outgoing`→`out` for display only. JSON keeps `incoming`/`outgoing` + `external_id`. |
| **F3** | Hard | Empty pretty **must** distinguish via **`node_kind` first** (AI2 L7): `None` → `No graph node for <id>.`; `Some(kind)` wrong kind → hierarchy `No memory node for <id>.` / session `No session node for <id>.`; expected kind + empty edges → neighbors `No neighbors for <id>.` / hierarchy `No SYNTHESIZED_FROM children (leaf).` / session `No memories in this session via graph edges.`. Every pretty diagnostic/empty line is followed by `next: ai-brains graph update`. Exit **0**. JSON empty stays `{ memory_id, neighbors: [] }` (no new required keys). Two queries when the node exists; one when missing. |
| **F4** | Hard | Feature-off `graph *` (including new flags) still **exit 2** + `FEATURE_UNAVAILABLE` + `GRAPH_REINSTALL_SOOT`. `graph --help` exit **0**. Do not add density logic to stubs. |
| **F5** | Hard | JSON **keys frozen**: neighbors `{ memory_id, neighbors: [{ external_id, label, direction }] }`; hierarchy `{ root, synthesized_from: [id…] }`; session `{ session_id, memories: [id…] }`. Compact `to_string`. **No** `kind` / `preview` / `depth` / `truncated` required keys. Additive optional keys only if `skip_serializing_if` and tests prove N−1 shape when unset — **prefer zero additive keys**. |
| **F6** | Hard | `graph update` `--format` default **`json`** (not `auto`). Omitted / `json` / `auto` / pipe = existing `to_string_pretty`. **`auto` does not TTY-switch** (T74 + scripts). `--format human` = **one labeled line per field** (`status:`, `density:`, `nodes:`, `edges:`, `pinned_memories:`, `memory_nodes:`, `edge_node_ratio:`, `note:`, `remediation:` only when `Some`) via `println!` — **not** a comma-joined single line (AI1 M4 declined). Still runs `gather_density_snapshot` + `assess_graph_density`. `GraphHealthOutput` stays serde-only for the JSON path. No `--limit` on Update. |
| **F7** | Hard | Pretty hierarchy shows **depth** (2-space indent × depth, depth≥1). **Pin (AI2 M1):** new `GraphSearch::get_synthesized_hierarchy_with_depth(&self, root: &str) -> Result<Vec<(String, i64)>>` in `queries.rs`. SQL keeps the existing CTE; `SELECT n.external_id, MIN(r.depth) AS depth … WHERE r.depth > 0 GROUP BY n.external_id ORDER BY MIN(r.depth), n.external_id`. `MIN(depth)` = shortest path on diamonds. **Do not** duplicate the CTE in the CLI. **Do not** modify `get_synthesized_hierarchy` (JSON + DISTINCT stay). Pretty calls the new method; JSON calls the old method then F9-sorts ids. No box-drawing (`└─`) in DoD. |
| **F8** | Hard | `--limit` on the three reads: `#[arg(short = 'l', long)]` (memory-list SOOT). Pretty: **always** `clamp_list_limit(opts.limit)` (None/0 → **50**, max **200**); print `… and N more` when truncated. JSON: **`if opts.limit.is_some() { clamp_list_limit(opts.limit) } else { usize::MAX }`** — do **not** call `clamp_list_limit(None)` on the JSON path (that would silently cap at 50). `rebuild` / `update` have no `--limit`. |
| **F9** | Hard | Sort **before emit** (pretty **and** JSON) using **raw** fields: neighbors by `NeighborHit.direction` (`incoming` &lt; `outgoing` lexicographically), then `label`, then `external_id`. Pretty DIR `in`/`out` is display-only and is **not** the sort key. Hierarchy **pretty** rows: depth then id (from the new method). JSON `synthesized_from` and `memories`: **lexicographic**. This **changes array order** vs pre-T246 SQL encounter order. **Not** a key-level break (F5). **Is** a PROTOCOL-COMPAT §5 behavioral note (AI2 M2 → F14). Do **not** add `ORDER BY` to `get_neighbors` / `get_session_memories` SQL (`get_session_memories` has live_graph + projector test callers — sort is CLI-local). |
| **F10** | Hard | **Do not** change `NeighborHit` serde field set. **Do not** change `get_neighbors` signature. Recall `--graph-boost` (`recall.rs` uses `external_id` only) stays source-compatible. Pretty KIND = `node_kind` per row. **PREVIEW only when `kind == "memory"`** via `preview_line` 80 + role strip; session/turn/other → empty preview cell. Missing projection → empty cell, do not fail. Zero additive JSON keys (AI2 O11). |
| **F11** | Hard | `GraphSearch::node_kind(external_id) -> Result<Option<String>>` — `SELECT kind FROM graph_node WHERE external_id = ?1 LIMIT 1` (prefer `query_row` + `optional()`). None = missing. Kind is one of migration **0022** CHECK values; do not invent. Hierarchy root must be `memory`; session root `session`. N+1 `node_kind` calls for ≤50 pretty rows is acceptable (~5ms). **No** `node_kinds` batch this track (soft F17). |
| **F12** | Hard | Zero new crates; **no version pin bumps**; no CLI reqwest; no contracts DTO; capture-independent (no events, no models). clap 5 is **not released** (max 4.6.6). |
| **F13** | Hard stop-before | Do **not** run live `ai-brains graph rebuild` from this track. Do not retune T213 floors. Do not rewrite the projector. |
| **F14** | Hard docs | CAPABILITIES graph table: TTY pretty / pipe JSON + `--format` / `--limit`. PROTOCOL-COMPAT §5: neighbors/hierarchy/session “TTY pretty human; **compact** JSON when piped or `--format json`; **keys unchanged**.” **Add array-order row (AI2 M2):** “Array order: sorted for determinism (neighbors: direction→label→id; hierarchy/session: lexicographic). Pre-T246: SQL encounter order.” OPERATIONS one paragraph. Skill one-liner. Repo-root `CHANGELOG.md` T246 row only. |
| **F15** | Hard clap | `#[command(after_help = "…")]` on the **`GraphCommands` enum** (not the parent `Graph`). One example each of neighbors pretty + `--format json`. Do not restack T204 groups. |
| **F16** | Hard verify | Existing `graph_health_output__*` units stay green. T74 `graph update` JSON parse stays green. Feature-off smoke exit 2 stays green. |
| **F17** | Soft residual | ASCII/`└─` tree; mermaid export; color; `--plain`; promote `GraphHealthOutput` to contracts; TTY-auto for `graph update`; is-terminal → std; pager for huge dumps; batch `node_kinds` |
| **F18** | Soft residual | Projector / LiveGraphHook completeness so fresh pins have `RECALLS` edges (T213 cause class — not presentation) |
| **F19** | Soft residual | Event↔graph freshness (T213 F31) |
| **F20** | Soft | help_ia GRAPH already listed; no new group |

---

## 5. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `resolve_graph_format` — `auto`+TTY → pretty; `auto`+pipe → json; `pretty`/`human`/`text` → pretty (TTY or not); `json` → json |
| **AC2** | Unit: `format_neighbors_pretty` fixture one incoming `RECALLS` session + one outgoing — header, `in`/`out`, labels, ids, kinds; JSON helper still emits `incoming`/`outgoing`/`external_id` |
| **AC3** | Unit: empty pretty — missing node vs present-empty strings pinned in F3; both include `graph update` |
| **AC4** | Unit: `format_hierarchy_pretty` depths 1 and 2 indent 2 and 4 spaces; no `└` |
| **AC5** | Unit: pretty truncate — 51 rows + `--limit` default → 50 lines + `… and 1 more` |
| **AC6** | Unit: sort — incoming before outgoing; labels A then B; ids lexicographic |
| **AC7** | Hermetic graph-on: pin + session edge (or seeded SQL) → `graph neighbors --format json` keys **only** `memory_id`/`neighbors`/`external_id`/`label`/`direction`; `graph neighbors --format pretty` contains `DIR` or `in`/`RECALLS` |
| **AC8** | Hermetic: unknown id `--format pretty` mentions `No graph node`; `--format json` still `{ neighbors: [] }` exit 0 |
| **AC9** | Hermetic: `graph session --format pretty` on a session with ≥1 memory prints id + preview; `--format json` is compact array of ids |
| **AC10** | Hermetic/unit: `graph update` (no flags) still pretty-JSON parseable with T213 keys; `--format human` contains `status:` and `density:` |
| **AC11** | Feature-off: `graph neighbors x --format pretty` exit **2** + `FEATURE_UNAVAILABLE` (extend T198 smoke if needed) |
| **AC12** | clap: `graph neighbors --format xml id` exit **2** (usage) |
| **AC13** | Live (on go, graph-on PATH): any real pin that has neighbors is TTY-scannable (`in`/`RECALLS` + kind). Plan-time ids `5a0e0a71-…` / session `3b4e95b8-…` are **baseline-specific** (2026-08-13) — re-discover via `memory list` if the vault moved. Piped neighbors is compact JSON. |
| **AC14** | Docs: CAPABILITIES + PROTOCOL-COMPAT §5 (TTY/pipe **and** array-order row) + CHANGELOG T246 |
| **AC15** | Recall `--graph-boost` existing unit/smoke still pass (no `NeighborHit` serde/API break) |
| **AC16** | Unit on `ai-brains-graph`: `get_synthesized_hierarchy_with_depth` fixture with depth-1 and depth-2 + diamond (same child two parents) → `MIN(depth)` once; existing `get_synthesized_hierarchy` id set unchanged |

---

## 6. Non-goals

- Making the live graph dense (rebuild, projector, default-on feature)
- Desktop / xyflow / mermaid
- `graph rebuild --format`
- Changing T213 status vocabulary or floors
- Governed policy on graph reads
- clap 5 / rusqlite 0.40 / new table crates
- T248 / T249 / T250 presentation

---

## 7. Capture independence / contracts / exits

| Topic | Rule |
|-------|------|
| Capture | Presentation + read-only SQL. No `MemoryPinned` / no nightly / no models |
| Contracts | No `ai-brains-contracts` change. CLI-local formatters only |
| Exits | Empty success **0**; feature-off / bad `--format` **2**; vault errors unchanged |
| Privacy | Preview inherits vault content already shown by `memory list` / `recall`. Do not print keys |

---

## 8. File touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/main.rs` | `--format` / `--limit` on GraphCommands (not Rebuild) |
| `crates/ai-brains-cli/src/commands/graph.rs` | resolve + pretty formatters + JSON path; update human |
| `crates/ai-brains-graph/src/queries.rs` | **Required:** `node_kind`; `get_synthesized_hierarchy_with_depth` (existing id-only method stays) |
| `crates/ai-brains-cli/tests/` | hermetic pretty/json + feature-off format flag |
| `Docs/CAPABILITIES.md` | graph table |
| `Docs/PROTOCOL-COMPAT.md` | §5 inventory |
| `Docs/OPERATIONS.md` | short graph-read paragraph |
| `.agents/skills/ai-brains/SKILL.md` | pretty default one-liner |
| `CHANGELOG.md` | T246 row |
| `conductor/*` | status / deferred |

**Do not touch:** `graph_density.rs` assessor, `live_graph.rs` projector, retrieval fuse, T247 nightly.

---

## 9. Verification plan

```powershell
# Units
cargo nextest run -p ai-brains-cli graph
cargo nextest run -p ai-brains-graph
cargo clippy -p ai-brains-cli -p ai-brains-graph --all-targets -- -D warnings

# Hermetic (graph-on)
cargo nextest run -p ai-brains-cli --features graph -- graph

# Feature-off regression
cargo nextest run -p ai-brains-cli --test smoke graph__default_build

# Live on go (PATH graph-on; do not rebuild)
ai-brains graph neighbors 5a0e0a71-1ee7-445b-84a9-aa06fe499c2e
ai-brains graph neighbors 5a0e0a71-1ee7-445b-84a9-aa06fe499c2e --format json
ai-brains graph session 3b4e95b8-a011-48a8-b5ea-72e36c6a2458
ai-brains graph update
ai-brains graph update --format human

# Full gate
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

---

## 10. Risk / review

- **Category:** FEATURE / UX (not SECURITY). Cross-model still useful: JSON key freeze + feature-off + recall API.
- **Highest regression:** silent JSON key add; TTY-switching `graph update`; breaking `get_neighbors`.
- **Cap deferred mediums:** ≤3; presentation softs go to F17–F19 / ISSUES-via-deferred.md.

---

## 11. Suggested implement order (locked)

1. Pure formatters + resolve + empty strings (Red → Green units, no vault)
2. `node_kind` + pretty/json wiring for neighbors
3. Hierarchy depth pretty + session pretty + `--limit`
4. `graph update --format human`
5. Hermetic + feature-off + docs

---

## 12. Placeholder disposition

| Draft | Disposition |
|-------|-------------|
| F1 Pretty default TTY / JSON non-TTY | **Absorbed** F1 (`auto`) |
| F2 Columns direction, label, id, optional preview | **Absorbed** F2 + kind (live session-id trap) |
| F3 Empty neighbors honest next-step | **Absorbed** F3 (node-miss vs empty) |
| F4 Feature-off exit 2 | **Absorbed** F4 |

---

## 13. Deferred fold-in

| Item | Source | Disposition |
|------|--------|-------------|
| Graph neighbors JSON-only | deferred.md T246 | **DoD** F1–F5 / AC1–AC9 |
| hierarchy/session human missing | README / placeholder | **DoD** F1/F3/F7/F9 |
| `graph update` pretty → T246 | README scored ≥8 | **F6 opt-in human**; default JSON stays |
| T213 skill one-liner | T213/T232 leftover | **F14** skill + CAPABILITIES |
| T213 F31 freshness | T213 residual | **F19 soft** |
| Auto rebuild / projector / Cargo default | T213/T232 | **Not absorbed** F13/F18 |
| rusqlite 0.40 / clap 5 / is-terminal std | T213 L4 / T214 F24 | **F12 / F17** |
| T248 / T249 | peer placeholders | **Not absorbed** |

---

## 14. AI fold-in disposition (2026-08-13) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 restates the plan (freeze `NeighborHit`, `node_kind`, `--format`/`--limit`, update human opt-in). AI2 two mediums are **must-pin** before go (new hierarchy-depth method; PROTOCOL-COMPAT array-order note).

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1** | AI1 | **Agree** (already F10) | Freeze `NeighborHit` + `get_neighbors` |
| **AI1 M2** | AI1 | **Agree** (already F11) | `node_kind` SQL; prefer `query_row`+`optional()` over raw `rows.next()` |
| **AI1 M3** | AI1 | **Agree dispatch / decline passthrough** | `resolve_graph_format` exists; **no** `Some(other) => other` — clap `value_parser` exit 2 |
| **AI1 M4** | AI1 | **Agree default JSON / decline one-liner** | F6 labeled lines, not `status: live, density: ok, …` |
| **AI1 L1 / L2 / O1** | AI1 | **Agree** | Already F4 / F14 / Phase 1 ACs |
| **AI1 AC remumber** | AI1 | **Decline** | Keep spec AC1–AC16 |
| **AI2 M1** | AI2 | **Agree hard** | F7 `get_synthesized_hierarchy_with_depth` + `MIN(depth)` `GROUP BY`; no CLI CTE |
| **AI2 M2** | AI2 | **Agree hard** | F9 sort is behavioral; F14 PROTOCOL-COMPAT array-order row |
| **AI2 L1** | AI2 | **Agree** | Use `is_terminal` crate; `project.rs` std inconsistency stays T214 F24 |
| **AI2 L2** | AI2 | **Agree** | Deliberate vs recall passthrough |
| **AI2 L3** | AI2 | **Agree hard** | JSON `limit.is_some()` gate |
| **AI2 L4** | AI2 | **Agree, corrected** | KIND width **14** (`source_version`), not 13 |
| **AI2 L5–L7 / L9–L13 / L15** | AI2 | **Agree** | Pinned into F2/F3/F6/F8/F9/F10/F15 |
| **AI2 L8** | AI2 | **Agree as soft** | N+1 ok; batch → F17 |
| **AI2 L14** | AI2 | **Agree** | AC13 baseline-specific ids |
| **AI2 L16 / O11 / O12** | AI2 | **Affirm** | Scope / zero additive keys / `format: String` |
| **AI2 O8 batch DoD** | AI2 | **Decline as DoD** | F17 only |

### Pins locked by fold-in

1. **F7:** `get_synthesized_hierarchy_with_depth` in the graph crate; existing DISTINCT method stays for JSON.
2. **F9/F14:** JSON arrays sorted; PROTOCOL-COMPAT documents pre-T246 encounter order vs new order.
3. **F1:** clap reject; no resolve passthrough; `format: String`.
4. **F8:** JSON unlimited unless `--limit` is `Some`.
5. **F2:** DIR/LABEL/ID/KIND widths 3/16/36/14; preview only for `kind=memory`.
6. **F6:** update `--format` default `json`; `auto` still JSON; human = labeled `println!`.
