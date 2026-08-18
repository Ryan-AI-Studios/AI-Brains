# T262 — Graph live projection + neighbors / hierarchy

- **Track ID:** T262-GraphLiveProjection
- **Status:** **Planned** (plan-only until go; registry stays **Pending**)
- **Category:** FEATURE / BUGFIX
- **Owner:** —
- **Source:** Audit 2026-08-16 — graph unused; `graph neighbors` **4/5**; `graph hierarchy` **3/4**; opportunity “rebuild or fix live projection”
- **Depends on:** T69 live hook ✅; T88 pin prints turn_id; T147 #10 turn `MemoryId::new()` residual; T213 density doctor; T232 capability remediations; T246 human CLI
- **Blocks / feeds:** A just-pinned DECISION is queryable via `graph neighbors` / `graph hierarchy` without a manual rebuild. Density floors stay T213. Governed honesty stays **T263**. Next-action for harness/whoami/list stays **T267**.
- **Absorbs:** Audit T262 row (sparse E/N, 4h pin no node, neighbors 4/5, hierarchy 3/4); T213 “projector more edges / auto rebuild” product half (ID alignment only); T246 **F18** projector / LiveGraphHook completeness; T261 closeout “graph sparse / 4h pin”
- **Not absorbed:** Density threshold retune (T213 floors stay); Cargo default-on graph feature (T200); Cozo INFO (T208 closed); nightly auto-`graph rebuild`; neighbor UUID prefix match; T267 harness/whoami/list next; T263 governed authority
- **Research date:** 2026-08-17 (plan dogfood HEAD `da785c1` T261 `#176`)
- **AI fold-in:** none yet (reserved §13)
- **Ledger:** planning DOCS TX `41977238-d85e-4e8a-bc80-3baba4937c90`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** run live `graph rebuild`. Do **not** remint historical `memory_projection` IDs. Do **not** retune T213 floors. Do **not** reopen T240 F2 / T255 declines. Do **not** `cargo install`, write live `.env`, bump clap, or add crates.

---

## 1. Objective

A memory that exists in the vault — especially a just-pinned DECISION whose printed id is what the operator pastes into `graph neighbors` — must be a graph **memory** node with a session `RECALLS` edge **without** a manual `graph rebuild`.

If the id is missing from the graph, doctor/neighbors must say **why** (vault has it but never projected vs unknown id vs honest leaf), not only `next: ai-brains graph update`. `graph update` is a health check. It does not create nodes.

That advances the north star: capture stays independent of the graph crate; the append-only log remains the source of truth; the live hook already exists (T69). The hole is **identity**, not a dead hook.

No models. No new crates. No contracts DTO. No live rebuild as DoD.

---

## 2. Live baseline (2026-08-17)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `da785c1` T261 `#176`. `main == origin/main`. Tree **CLEAN** at plan start. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` (mtime **2026-08-17 18:20**, 24 848 896 bytes). **Pre-T260** (`--symbols` unknown). Graph-on (T246 pretty works). **Do not `cargo install`.** |
| Source debug | `target\debug\ai-brains.exe` (mtime **2026-08-17 21:32**, 40 987 136 bytes). Newer than PATH (T261). Graph projector / pin / hook **unchanged** since T246 for this hole. |
| `preflight --summary` | Scope path owner `3581317d` (`C:\dev\ai-brains`); **2902** pinned. Discovery grants empty (T263). |
| `graph update --format human` | `status=sparse` `density=warn` `nodes=21477` `edges=1363` `pinned_memories=36162` `memory_nodes=19890` `E/N=0.063` `remediation=ai-brains graph rebuild`. T232 already names **rebuild** here. |
| Just-recalled symbol `7c3634fe-…` (this session) | Pretty neighbors: **1** `in RECALLS` → session `2ca5382c-…`. **T69 hook works** for `MemoryPinned` from `recall`. |
| Same id hierarchy | `No SYNTHESIZED_FROM children (leaf).` + `next: ai-brains graph update`. T246 pretty exists; next-action is wrong for a true leaf. |
| `graph session 2ca5382c-…` | **8** memories (this planning recall). Session walk works. |
| Prefix `46d88c87` / `aa0a75da` | `No graph node` + `next: ai-brains graph update`. JSON `{neighbors:[]}` / `{synthesized_from:[]}`. Neighbors is **exact** `external_id` (F17: do not add prefix). Audit writeup used `46d88c87-…` as shorthand, not a product prefix contract. |
| Last GitHub PR | [#176](https://github.com/Ryan-AI-Studios/AI-Brains/pull/176) T261. `gh pr view --comments`, `/reviews`, `/comments` all **empty**. HEAD is `main` (no open product PR). **last-PR Cursor: N/A.** |
| Ledgerful | `doctor` ready (legacy `.changeguard` / sig-pin / timings / :8081 unreachable; :8083 ok). 0 pending 0 drift at plan start. Hotspot **#1** `project.rs` (3.893) — **do not touch.** `graph.rs` / `projector.rs` / `live_graph.rs` not in top-10. |
| ai-brains recall | PATH still ranks T70 stubs (pre-T260). No prior “pin turn_id = graph memory node” pin. |

### 2.2 Why this still matters

| Residual | Why it is a product hole / why decline |
|----------|----------------------------------------|
| Audit “T69 skipped the 4h pin” | **False as stated.** `LiveGraphHook` + `StoreSink` **do** run on `pin`. `GraphAwareEventStore` **does** project `MemoryPinned` from recall (live proof: `7c3634fe`). The 4h pin was `ai-brains pin` (TAGS), not a recall pin. |
| Three IDs for one pin | (1) CLI prints `IngestRequest.turn_id` (T88 comment). (2) `TurnProjection` inserts `MemoryId::new()` (`turn.rs:61`) — T147 #10. (3) `GraphProjector` hashes session+content+occurred_at with `DefaultHasher` into a **turn** node. Neighbors looks up the printed UUID → miss. |
| `graph rebuild` as remediator for that miss | Rebuild **replays the same projector**. Legacy capture events have **no** `turn_id` on the payload → still no memory node for the printed id. Blind `next: graph rebuild` lies for unknown / ingest-turn_id strings. |
| `next: graph update` (T246 F3) | Update is T213 assessor. It never writes `graph_node`. Live `graph update` already says rebuild when sparse. Neighbors/hierarchy still point at update. |
| Hierarchy 3/4 | T246 already shipped pretty leaf copy. JSON keys stay frozen empty array. Remaining hole: next-action + “no node” vs “honest leaf”. Do **not** invent `SYNTHESIZED_FROM` edges. |
| E/N 0.044 → 0.063 | Typed provenance graph (T213). Pins without synthesis/recall stay sparse. **Do not** retune 0.50 or fake edges to pass density. New aligned pins add 1 memory node + 1 `RECALLS`. Historical coverage does not jump without a payload field on old events. |
| Graph-off daily binary | Default Cargo feature is still `[]`. Graph-off never runs the hook. Feature-off `graph *` stays exit **2** + `FEATURE_UNAVAILABLE` (T198/T232). |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Live hook | `crates/ai-brains-cli/src/live_graph.rs` | `apply_and_flush` non-fatal. `GraphAwareEventStore` wraps append (recall / nightly / symbol_bridge). |
| Pin / ingest sink | `pin.rs:83–90` + `context.rs` `StoreSink::append` | Constructs `LiveGraphHook` and applies **after** successful store append. Hook is **on**. |
| What pin emits | `ai-brains-capture` `build_user_prompt` / `build_assistant_final` | `UserPromptRecorded` / `AssistantFinalRecorded` only. **No** `MemoryPinned`. **No** `turn_id` on payload today. |
| Printed id | `pin.rs:117–118` | Prints ingest `turn_id`. Comment claims projection stores it. **False** on today’s tree. |
| Turn → memory | `ai-brains-store/src/projections/turn.rs:61` | `MemoryId::new()` per apply. T147 #10 accepted residual. |
| Projector capture | `ai-brains-graph/src/projector.rs:57–97` | `DefaultHasher` → `category: "turn"`. Not the printed UUID. Not `kind=memory`. |
| Projector pin | `projector.rs:99–120` | `MemoryPinned` → memory node + optional session + `RECALLS`. This path is healthy (live + `live_graph.rs` unit). |
| Rebuild | `rebuild.rs` | `DELETE graph_edge/graph_node` then `apply` every event. Same projector. |
| Neighbors | `commands/graph.rs:265` | `node_kind` exact `external_id`. Missing → pretty `PRETTY_NEXT` = `next: ai-brains graph update` (`:12`). |
| Hierarchy leaf | `pretty_hierarchy_leaf` | T246 F3 string + same `PRETTY_NEXT`. |
| `memory_exists` | `QueryStore` `query_store.rs:709` | `COUNT(*) FROM memory_projection`. `forget.rs` already calls `ctx.conn.memory_exists`. Reuse. |
| Density | `graph_density.rs` | Floors `MIN_EDGE_NODE_RATIO=0.50`, `MIN_MEMORY_COVERAGE=0.10`. **Do not change.** |
| Feature | `ai-brains-cli` `default = []`, `graph` optional | Unchanged (T200). |
| JSON keys | T246 F5 / PROTOCOL-COMPAT §5 | Frozen. Compact piped. |
| Hotspot | `project.rs` #1 | **Do not touch.** |

### 2.4 Dependency / standards research (2026-08-17)

| Claim | Evidence | Use |
|-------|----------|-----|
| `DefaultHasher` is not a stored identity | [Rust `DefaultHasher`](https://doc.rust-lang.org/std/collections/hash_map/struct.DefaultHasher.html): “hashes should not be relied upon over releases.” `new()` is same across instances **in one std**, not a product SoT. | **F10:** legacy turn node id = `envelope.event_id`, never hasher. |
| Projections must be rebuildable from event data | [Azure Event Sourcing](https://learn.microsoft.com/en-us/azure/architecture/patterns/event-sourcing) (crawled 2026-08-17): materialized views regenerate by replaying events; annotate request-derived events with a stable identifier. Doomen: projections must not crash; identity comes from the event. | Additive `turn_id` on capture payloads so replay = live hook. |
| Incremental hook already exists | T69 AC1–AC4 + live `7c3634fe` `RECALLS`. Status.md: “append now applies GraphProjector incrementally.” | Do **not** rewrite `LiveGraphHook`. Fix apply inputs. |
| Typed sparse graphs can be healthy | T213 research (Adaptive GraphRAG / Yu 2026): E/N is not Erdős–Rényi. | Decline density retune (F15). |
| CLIG / next-action | [clig.dev](https://clig.dev/) humans first; T232 already capability-aware on `graph update`. | Missing-node next is rebuild **only** when the vault has that memory. |
| clap / pins | workspace clap **4.5** / lock **4.6.1** / crates.io **4.6.6**. clap **5 not released**. rusqlite lock **0.39.0** / crates.io **0.40.2**. serde_json lock **1.0.150** / crates.io **1.0.151**. rustc **1.95.0** / edition **2024**. uuid lock **1.23.1**. | **No bumps.** Snapshot — re-verify at execute. |
| Event payload goldens | No `UserPromptRecorded` JSON fixtures in-tree. `KnownPayload` uses the same structs. | Additive `#[serde(default)] Option<TurnId>` is compatible. |
| N/A | SQLCipher page crypto, schtasks, ANN, clap JSON CLI flags. | Not this track. |

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Next-action split** | T246 F3 successor. Pretty empty is **caseful**: (a) no graph node **and** `memory_exists` → `next: ai-brains graph rebuild`; (b) no graph node **and** not in `memory_projection` → **no** rebuild/update; explain not a vault memory id; (c) node exists, no neighbors / hierarchy leaf / session empty → **no** remediator (honest isolation / leaf). Never `graph update` as the remediator for a missing node. Graph-off never reaches these helpers (F14). |
| **F2 — Hierarchy JSON freeze** | Pretty leaf copy may drop `graph update` (F1c). JSON stays `{ root, synthesized_from: [] }` (T246 F5). Do **not** add `reason` / `next` keys. Do **not** invent `SYNTHESIZED_FROM` edges. |
| **F3 — Diagnosis lock** | T69 hook is **not** off on pin. Hole is ID trifurcation (printed turn_id ≠ `MemoryId::new()` ≠ hasher turn node). Plan must not prescribe “turn the hook on.” |
| **F4 — No live / nightly rebuild** | Do **not** run `graph rebuild` on the operator vault as planning or as DoD. Do **not** schedule rebuild from nightly. Cost note: live replay is `read_all_events` + DELETE `graph_*` on **21 477** nodes / **36 162** pins. Hermetic temp vaults only. |
| **F5 — Capture independence** | Graph apply stays non-fatal (`warn` + continue). `ai-brains-capture` does **not** depend on `ai-brains-graph`. Capture only carries `turn_id` on the event. Fail-open append stands. |
| **F6 — Additive `turn_id`** | `UserPromptRecordedPayload` and `AssistantFinalRecordedPayload` gain `#[serde(default, skip_serializing_if = "Option::is_none")] turn_id: Option<TurnId>`. Old events → `None`. No new event kind. No contracts DTO. |
| **F7 — Capture writers** | `build_user_prompt` / `build_assistant_final` set `turn_id: Some(request.turn_id)`. All hook/import/pin/ingest paths inherit. Tests that build payloads by hand set `None` unless they assert the new path. |
| **F8 — TurnProjection** | If `turn_id` is `Some`, `memory_id = MemoryId::from_uuid(turn_id.as_uuid())`. If `None`, keep `MemoryId::new()` (T147 #10; do **not** remint historical rows). Do **not** change `rebuild_projections` behavior for old events. |
| **F9 — Projector new path** | When `turn_id` is `Some`: emit `kind=memory` node id = that UUID, ensure session node, `RECALLS` session→memory. **Do not** also emit the hasher/event_id **turn** node for that event. |
| **F10 — Projector legacy path** | When `turn_id` is `None`: replace `DefaultHasher` with `envelope.event_id` as the **turn** node id (`kind=turn`) + `IN_SESSION`. Rebuild-stable. Still **not** queryable as the printed ingest turn_id (honesty F1b). |
| **F11 — No extra `MemoryPinned` from pin** | Pin does **not** append a second `MemoryPinned` event. Projector + live hook on the capture envelope is enough. Recall may still emit `MemoryPinned` (T67); `INSERT OR IGNORE` / ON CONFLICT stays idempotent when ids match. |
| **F12 — Pin print** | Keep printing ingest `turn_id`. After F7+F8 that UUID **is** `memory_projection.memory_id`. Update the T88 comment so it is true. Do not switch to `event_id`. |
| **F13 — JSON keys** | Neighbors/hierarchy/session keys **frozen** (T246 F5). PROTOCOL-COMPAT array-order note stays. No additive required keys. |
| **F14 — Feature-off** | `graph *` still exit **2** + `FEATURE_UNAVAILABLE` + `GRAPH_REINSTALL_SOOT`. Pin on graph-off still writes the event with `turn_id` (so a later graph-on **rebuild** can project it). |
| **F15 — Density floors** | Do not change T213 constants or env names. Do not claim live E/N ≥ 0.50 as DoD. Doctor check count stays (T255 declined 16th). |
| **F16 — Graph default-on** | Cargo `default = []` stays. T200 closed. |
| **F17 — No prefix match** | `graph neighbors` / `hierarchy` / `session` stay exact `external_id`. Prefix is T125-class and not this audit hole. |
| **F18 — `memory_exists`** | Missing-node pretty calls `ctx.conn.memory_exists(id)` (same as `forget.rs`). Query error → treat as unknown (F1b), `warn`, do not fail the read (exit 0). |
| **F19 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.1**. No CLI `reqwest`. rusqlite stays **0.39.0**. |
| **F20 — Cross-model** | FEATURE (event payload + projector + CLI contract). After Phase-1 review clean, run read-only `codex-review`. |
| **F21 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals go to `conductor/deferred.md`. |
| **F22 — PATH-behind** | Live PATH is pre-T260. Do **not** `cargo install` unless the user asks. Tests/manual AC use `cargo run --features graph` / hermetic bin. |
| **F23 — Stop-before live rebuild** | Even after go: do not rebuild the operator vault unless the user explicitly confirms that remediating action. |
| **F24 — Decline `DecisionRecorded` projector** | `DecisionRecorded` is still `_ => {}` in `projector.rs`. Pin is ingest, not MADR. Soft residual only. |
| **F25 — Docs** | CAPABILITIES graph table: pin id = graph memory id; missing-node next = rebuild **iff** vault has the id. OPERATIONS one paragraph (update ≠ rebuild). PROTOCOL-COMPAT: keys unchanged; pretty next-action is human-only. Root CHANGELOG T262 row. Skill one-liner if the graph section exists. |
| **F26 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. Existing T69/T74/T213/T232/T246 units stay green except T246 AC3 strings that this track supersedes (update those asserts). |
| **F27 — Historical memory IDs** | Do **not** change `TurnProjection` for `turn_id=None`. Do **not** run store projection rebuild as DoD. Embeddings / FTS stay on existing UUIDs. |
| **F28 — Inherit all ingest** | Do not special-case `pin.rs` beyond the print comment. `build_*` is the SoT so hooks/imports get the same node ids. |
| **F29 — Missing-node copy** | Vault-has: `No graph node for <id>.` + rebuild next. Unknown: `No graph node for <id> (not a vault memory id).` and **no** `next:` line. |
| **F30 — Decline T267** | Harness/whoami/list self-next stay T267. This track only owns graph read next-action. |
| **F31 — T246 test update** | `empty_pretty__missing_and_present__exact_f3_and_graph_update` is rewritten for F1/F29. Not a silent delete. |
| **F32 — Serde** | Missing `turn_id` key → `None`. Extra unknown keys on old readers stay ignored. `event_kind_from_payload` unchanged. |
| **F33 — Recall graph-boost** | Do **not** change `NeighborHit` or `get_neighbors` signature. T246 F10 stands. |
| **F34 — Hotspots** | Do not touch `project.rs`, `forget.rs` match path, `sync.rs` ledger pane (T271). |
| **F35 — Decline extras** | Neighbor prefix; mermaid/tree (T246 F17); `GraphHealthOutput` contracts promote; rusqlite `table_exists`; Cozo multiplex removal; backfill `MemoryPinned` for historical `memory_projection` rows (would invent events). |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit (events): JSON without `turn_id` deserializes to `UserPromptRecordedPayload.turn_id == None`. JSON with `turn_id` round-trips. Same for `AssistantFinalRecorded`. |
| **AC2** | Unit (graph): `UserPromptRecorded` + `Some(turn_id)` + session → `node_kind(turn_id) == Some("memory")` and one `RECALLS` session→memory. No `kind=turn` node for that event. |
| **AC3** | Unit (graph): same payload with `turn_id: None` → `node_kind(printed_random_uuid) == None`; turn node id equals `envelope.event_id` (not a hasher hex). |
| **AC4** | Unit/store: `TurnProjection` with `Some(turn_id)` inserts `memory_projection.memory_id == turn_id` string. |
| **AC5** | Unit/store: `turn_id: None` still inserts **a** memory row (legacy `MemoryId::new()`). Do not assert the UUID equals anything on the envelope. |
| **AC6** | Hermetic graph-on: `pin` a DECISION → stdout memory id → `graph neighbors <id> --format json` has `memory_id` + ≥1 neighbor `{direction: incoming, label: RECALLS}` pointing at the session. **No** `graph rebuild` in the test. |
| **AC7** | Hermetic graph-on: same pin → `--format pretty` contains `in` and `RECALLS` and does **not** contain `No graph node`. |
| **AC8** | Unit: `pretty_no_graph_node(id, true)` contains `graph rebuild` and does not contain `graph update`. `pretty_no_graph_node(id, false)` contains `not a vault memory id` and does not contain `rebuild` or `update`. |
| **AC9** | Unit: `pretty_hierarchy_leaf` / `pretty_no_neighbors` / `pretty_session_empty` do **not** contain `graph update` or `graph rebuild`. |
| **AC10** | Unit: JSON missing-node neighbors is still exactly keys `memory_id`, `neighbors` (empty array). Hierarchy missing-node still `{root, synthesized_from:[]}`. |
| **AC11** | Feature-off: `graph neighbors <id> --format pretty` exit **2** + `FEATURE_UNAVAILABLE`. |
| **AC12** | Clap: `graph neighbors --format xml <id>` exit **2** (existing). |
| **AC13** | `capture_does_not_require_graph` and other capture independence tests stay green. |
| **AC14** | Existing `test_projector_links_pinned_recall_memory_to_session` + `live_graph.rs` `graph_aware_store_makes_recall_edge_visible_on_append` stay green. |
| **AC15** | On go, classify-only live: `graph neighbors 7c3634fe-01aa-5511-a03b-b157e363b462` still shows `RECALLS` (hook regression). Do **not** rebuild the live vault. |
| **AC16** | Docs: CAPABILITIES + OPERATIONS + PROTOCOL-COMPAT human-next note + CHANGELOG T262. |
| **AC17** | T246 `empty_pretty__…graph_update` replaced by F1/F29 asserts (AC8/AC9). T74 piped `graph update` still parses as JSON. |
| **AC18** | Capture hand-built test payloads compile with explicit `turn_id: None` (or `Some` in AC4). No `unwrap`/`expect`/`panic` in production. |

---

## 5. Design notes

### 5.1 Identity SoT

```text
IngestRequest.turn_id
  → UserPromptRecorded/AssistantFinalRecorded.turn_id (Some)
  → memory_projection.memory_id
  → graph_node.external_id (kind=memory)
  → pin stdout
  → forget --memory-id
  → graph neighbors
```

One UUID. Rebuild of **new** events reproduces the same node. Rebuild of **old** events cannot invent a `turn_id` that was never logged (F1b + F27).

### 5.2 Why not emit `MemoryPinned` from `pin`

`MemoryPinned` is T67 “this recall hit is now in the graph.” Pin is capture. A second event would ON CONFLICT a second memory row if the projection id still differed, or duplicate the same id if F8 landed. Projector-on-capture is the smaller log and keeps capture graph-free.

### 5.3 Why not `MemoryId::from(event_id)` for all turns

Store projection rebuild would remint every historical turn memory, breaking embeddings, FTS, forget lists, and existing `MemoryPinned` references. T147 #10 stays for `turn_id=None`.

### 5.4 Why `envelope.event_id` on the legacy turn node

`DefaultHasher` is not a durable key (Rust std docs). Switching the **graph-only** turn id to `event_id` is rebuild-stable and does not touch `memory_projection`. Neighbors of a pin’s printed turn_id still miss — F1b is the honest copy.

### 5.5 Pretty helpers

Replace the single `PRETTY_NEXT` constant with case helpers. `neighbors` / `hierarchy` / `session` pass `memory_exists` only on the **missing-node** branch (one extra indexed COUNT). Leaf/empty-edge branches do not query and do not print `next:`.

### 5.6 Density after this track

New pins add coverage. Live E/N stays sparse until synthesis/recall create more typed edges. Doctor continues to warn. That is correct.

---

## 6. Non-goals

- Live or nightly `graph rebuild`
- Reminting historical `memory_projection` IDs / store projection rebuild
- Density floor retune / WCC / giant-component
- Cargo default-on graph feature
- Neighbor/hierarchy UUID prefix
- Inventing `SYNTHESIZED_FROM` / `IN_SESSION` edges that the event log does not support
- `DecisionRecorded` projector (F24)
- Historical backfill `MemoryPinned` for every existing pin
- T267 harness/whoami/list next-action
- T263 governed promotion
- T264 leftover `--global` / preflight blender
- clap 5 / new crates / pin bumps / `cargo install` / live `.env`
- Reopening T240 F2, T255 declines, T246 JSON keys, T213 assessor
- Changing `get_neighbors` / recall `--graph-boost`

---

## 7. Verification plan (TDD — red names first)

Events:

- `user_prompt_recorded_payload__missing_turn_id_key__deserializes_none__ac1`
- `user_prompt_recorded_payload__turn_id_present__round_trip__ac1`
- `assistant_final_recorded_payload__missing_turn_id_key__deserializes_none__ac1`

Store:

- `turn_projection__with_turn_id__memory_id_matches__ac4`
- `turn_projection__legacy_none__still_inserts_memory__ac5`

Graph:

- `projector__user_prompt_with_turn_id__memory_node_and_recalls__ac2`
- `projector__user_prompt_legacy_none__turn_node_is_event_id__ac3`
- keep `test_projector_links_pinned_recall_memory_to_session`

CLI units (`graph.rs`):

- `pretty_no_graph_node__vault_memory__next_rebuild__ac8`
- `pretty_no_graph_node__unknown_id__no_rebuild__ac8`
- `pretty_hierarchy_leaf__no_update_or_rebuild__ac9`
- `empty_pretty__json_keys_frozen__ac10` (or keep existing JSON unit)

Hermetic graph-on (`tempdir` + `--features graph`):

- `pin__graph_on__printed_id_neighbors_json_without_rebuild__ac6`
- `pin__graph_on__printed_id_neighbors_pretty__ac7`

Keep green: T74 `graph update` JSON parse; T198/T222 feature-off exit 2; T213/T232 density units; T246 format/sort/limit; `capture_does_not_require_graph`; `live_graph.rs` MemoryPinned unit.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Adding `turn_id` breaks every payload struct literal | AC18; grep list is ~15 sites (brain nightly tests + store + retrieval common + capture writers). |
| Historical pins still missing after go | Expected (F27). F1b copy. Do not promise `46d88c87` appears. |
| Operators run live rebuild hoping for E/N 0.50 | F4/F15/F23. CHANGELOG + OPERATIONS: rebuild does not invent `turn_id` on old events. |
| Double memory rows if someone also emits `MemoryPinned` with the printed id before F8 | F11 forbids extra emit; F8 lands in the same FEATURE TX as F7/F9. |
| `memory_exists` error fails neighbors | F18 warn + F1b. Exit 0. |
| Hasher change remints graph turn nodes on rebuild | Graph-only; no product lookup uses hasher hex. AC3. Do not live-rebuild. |
| Capture independence | F5; AC13. |
| Hotspot `project.rs` | F34. |
| PATH-behind dogfood | F22; hermetic + `cargo run --features graph`. |

---

## 9. Deferred absorb / decline

Full `conductor/deferred.md` scan 2026-08-17. `ISSUES.md` does **not** exist.

| Residual | Disposition |
|----------|-------------|
| Graph sparse; 4h pin no node; neighbors 4/5; hierarchy 3/4 (audit T262) | **Absorb** F1–F12 / AC1–AC18 |
| T213 auto rebuild / projector more edges / graph default-on / WCC | **Partial:** projector ID alignment **absorb** (F6–F10). Auto rebuild / default-on / WCC **decline** F4/F15/F16 |
| T213 F31 event↔graph freshness | **Decline** (soft). Missing-node + `memory_exists` is enough honesty. |
| T213 CLI density flags / `GraphHealthOutput` contracts / rusqlite `table_exists` / two-tier 0.50+0.10 | **Decline** (stay soft / already declined v1) |
| T246 F18 projector completeness | **Absorb** F6–F11 / AC2 / AC6–AC7 |
| T246 F19 freshness | **Decline** (same as T213 F31) |
| T246 F17 tree/mermaid/batch `node_kinds` | **Decline** F35 |
| T246 F3 all-empty → `graph update` | **Supersede** F1 / F31 / AC8–AC9 |
| T261 closeout “graph sparse / 4h pin” | **Absorb** (this track) |
| T232 capability remediations | **Partial:** keep rebuild vs `GRAPH_REINSTALL_SOOT`. Neighbors never run graph-off (F14). |
| T208 Cozo INFO / T200 default-on / T198 exit 2 | **Closed** — do not reopen. F14/F16. |
| T147 #10 turn `MemoryId::new()` | **Partial:** keep for `turn_id=None` (F8/F27). New events use payload `turn_id`. |
| T88 pin prints turn_id | **Absorb** F12 (make the comment true) |
| T267 harness/whoami/list self-next | **Decline → T267** F30 |
| T263 governed 0 authority | **Decline → T263** |
| T264 leftover `--global` / preflight blender | **Decline → T264** |
| T265–T271 remaining audit rows | **Decline** (not graph projection) |
| T240 F2 / T255 declined bag | **Stay closed** |
| T256–T261 PATH `cargo install` | **Decline** F22 |
| MSI / notarization / R-CI-BRANCH / anyhow allowlist / archive changeguard sweep | **Not related** (not graph identity) |
| Connector cursors, CE residuals, desktop, sync threat leftovers | **Not related** |
| last-PR Cursor (#176) | **N/A** — comments/reviews/inline empty; nothing to mint |

---

## 10. Implement order (on go)

1. Phase 0 re-verify projector / pin / turn projection / `PRETTY_NEXT` / deferred / last PR / pins. Confirm hook still applies on pin and `MemoryPinned` still works.
2. Red: events serde + store turn_id + projector AC2/AC3 + pretty F1 units + hermetic pin→neighbors.
3. Green: F6–F10 + F1/F29 helpers + pin comment.
4. Docs F25.
5. Targeted nextest/clippy on `ai-brains-events` + `ai-brains-capture` + `ai-brains-store` + `ai-brains-graph` + `ai-brains-cli` (`--features graph` where required); full gate on finalize.
6. FEATURE TX commit; review.md; `codex-review`; publish per implement-track (not this skill).

---

## 11. Soft residuals

| Residual | Why not DoD |
|----------|-------------|
| `DecisionRecorded` → graph node | F24; pin is ingest |
| T213 F31 last-event vs last-graph timestamp | Honesty is F1 + `memory_exists` |
| Historical backfill `MemoryPinned` for 36k pins | Invents events; F35 |
| Neighbor UUID prefix | F17 |
| Skip hasher turn nodes entirely on rebuild | Would drop IN_SESSION walk for pre-field turns; keep F10 |
| `GraphHealthOutput` contracts promote | T213 soft |
| Skill one-liner if already present from T246 | Docs only if missing |
| PATH `cargo install` | F22 operator |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-events/src/payload.rs` | Additive `turn_id` on two payloads |
| `crates/ai-brains-events/tests/*` | AC1 + any struct literals |
| `crates/ai-brains-capture/src/user_prompt.rs` | `Some(request.turn_id)` |
| `crates/ai-brains-capture/src/assistant_final.rs` | `Some(request.turn_id)` |
| `crates/ai-brains-store/src/projections/turn.rs` | F8 branch |
| `crates/ai-brains-store/tests/*` | AC4/AC5 + literals `turn_id: None` |
| `crates/ai-brains-graph/src/projector.rs` | F9 + F10 |
| `crates/ai-brains-graph/tests/*` | AC2/AC3 |
| `crates/ai-brains-cli/src/commands/graph.rs` | F1/F18/F29 helpers; `memory_exists` on miss |
| `crates/ai-brains-cli/src/commands/pin.rs` | Comment only (F12) |
| `crates/ai-brains-brain/tests/*` + `ai-brains-retrieval/tests/common.rs` | `turn_id: None` on literals |
| `Docs/CAPABILITIES.md` `Docs/OPERATIONS.md` `Docs/PROTOCOL-COMPAT.md` `CHANGELOG.md` | F25 |
| `conductor/*` | Status / deferred / this spec+plan |
| Soft: `.agents/skills/ai-brains/SKILL.md` | One graph line if missing |

**Do not touch:** `project.rs`, `graph_density.rs` thresholds, `live_graph.rs` policy (keep non-fatal), `rebuild.rs` algorithm, `queries.rs` `get_neighbors` signature, nightly schedule, `.env`.

---

## 13. AI fold-in

Reserved for `/fold-in` after `/review-track`.
