# T278 — Graph neighbor previews must be readable; density stays honest

- **Track ID:** T278-GraphDensityPreview
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE / UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `graph update` **7/7** sparse E/N ~0.11; `graph neighbors` **6/6** RECALLS with blank PREVIEW. Placeholder minted with T274–T284.
- **Depends on:** T213 ✅ density doctor; T232 ✅ remediator; T246 ✅ TTY pretty (`DIR LABEL ID KIND PREVIEW`); T262 ✅ live projection (`turn_id` = memory + `RECALLS`)
- **Blocks / feeds:** Operators can scan 1-hop neighbors as a daily tool (session row has text, not UUID-only). Density floors / remediator stay T213/T232. Safety vs hotspots stays **T279**. Policy `--scope` **T280**. Nightly dual-probe **T281**. `context --show` leftover **T282**. `project list` cwd-first **T283**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “graph sparse + neighbors blank preview”; T246 **F10 lift** (PREVIEW for `kind == "session"`, not only `"memory"`); T246 F18 remainder that is presentation (session caption), not projector
- **Not absorbed (DoD):** Live `graph rebuild` of the operator vault; T213 floor retune (`MIN_EDGE_NODE_RATIO=0.50`); Cargo `default` graph-on (T200); projector rewrite / fake edges / WCC; 2-hop sibling rows; hierarchy preview; mermaid/tree (T246 F17); `GraphHealthOutput` contracts promote; rusqlite **0.40+** `table_exists`; clap 5; DTO keys; T279–T283 peers; T240 F2; leftover `7d97a456` rebind
- **Research date:** 2026-08-22 (plan dogfood HEAD `400dd78` T284 `#193`; product `src/` = T284). Agy fold-in against `46fc872`. OpenCode fold-in against `5defcc5` (docs-only; crates identical to `400dd78`).
- **AI fold-in:** 2026-08-22 `agy-review.md` + `opencode-review.md`. **B 0 / M 0.** **Already (Agy):** m2 F3; O1 `truncate_preview_chars`; O2 F14/AC1. **Agree (Agy):** m1 F33/AC5; m2 AC14; O1 CJK AC1; O2 case list. **Agree (OpenCode):** m1 §2.3 crate-root `:10–16`; m2/O1 `pick_first_nonempty` F34 / AC14 required-pure; m3 HEAD/`pinned` snapshot. **Decline (OpenCode):** O2 empty-first hermetic AC3 (AC14 is DoD). **Affirm:** #193 N/A; no T285. Disposition **§13**.
- **Ledger:** planning DOCS TX `977c5e7e-1043-4d5d-ab52-7803cd231f6a`. Agy fold-in DOCS TX `384ed242-bb9d-4125-9079-3f40b8d5486a`. OpenCode fold-in DOCS TX `0765b916-9f38-417b-a698-b39b657078dd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** run live `graph rebuild`. Do **not** retune T213 floors. Do **not** rewrite `GraphProjector`. Do **not** `cargo install`, rewrite `.env` (T240 F2), pin-as-implement, or mutate schtasks. Do **not** grow hotspot `project.rs` / `preflight.rs` / `sync.rs` / `doctor.rs`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Neighbors PREVIEW is usable.** `graph neighbors <id> --format human` (and `pretty` / `text`) prints a **non-blank PREVIEW** for a session-kind neighbor. Today a just-pinned memory’s only 1-hop is `in RECALLS` → session UUID with an empty PREVIEW cell (T246 F10). `graph session` already shows memory text; neighbors must not be UUID-only.
2. **Density stays honest, not inflated.** Live `graph update --format human` / doctor `graph_density` already say `sparse` (E/N ~0.13 vs floor 0.50) with remediator `ai-brains graph rebuild` on this graph-on binary. Do **not** retune floors, invent edges, or claim live E/N ≥ 0.50. Status vocabulary stays `live` | `sparse` | `empty`.
3. **Hermetic is DoD; live rebuild is Stop-Before.** Pin → neighbors pretty has session preview **without** `graph rebuild`. Optional: hermetic vault after pin reports `status: live` (one session + one memory + one `RECALLS` is E/N = 0.50). The 39k-pin operator vault is **not** rebuilt as DoD.
4. **North star.** Capture independence: pretty SQL over existing `memory_projection` / graph edges. No models. No new events. JSON keys frozen (T246 F5 / T262 F13). Append-only log unchanged.

This unblocks the daily product: T213/T232 made density **honest**; T246 made the table; T262 made the pin a graph node. The remaining usefulness hole is **captions** — Neo4j Browser-class neighbor lists show properties, not raw ids.

---

## 2. Live baseline (re-scan 2026-08-22)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | **Plan dogfood:** `400dd78` T284 squash `#193`. **Agy fold-in:** `46fc872`. **This OpenCode fold-in:** `5defcc5` (docs-only). `git diff 400dd78 HEAD -- crates/` empty — product `src/` identical to T284. Tree **CLEAN** at fold-in start. `main` ahead of `origin/main` by plan+fold docs. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-21 05:55**, 25 368 576 bytes, **0.1.1**. **T270** on PATH (includes T246 pretty + T262 live projection; before T274–T284). Graph hole is T246-era — **PATH is valid**. **Do not `cargo install`.** Tests/manual AC use `cargo run --features graph` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **volatile** (plan 3476; OpenCode 3495; this fold-in **3515**). In-context **0/0/0**. Grants **0 of 3** (T275 hermetic; live not bootstrapped). Capture independence holds. |
| `project whoami` | `mismatch: false`. Effective/path/detect `3581317d`. Shell leftover `7d97a456` overridden by local `.env`. **Not this track** (T258 adopt-path; leftover volume T276; `--show` leftover **T282**). |
| `graph update --format human` | `status: sparse` `density: warn` `nodes: 23082` `edges: 3005` `pinned_memories: 39382` `memory_nodes: 20961` `edge_node_ratio: 0.130…` note sparse below floor **0.5**; `remediation: ai-brains graph rebuild`. Coverage 20961/39382 ≈ **0.532** (above `MIN_MEMORY_COVERAGE` 0.10). Warn is **E/N**, not coverage. |
| Doctor `graph_feature` | **ok** / `available` |
| Doctor `graph_density` | **warn** `ok: false` same sparse sentence + `ai-brains graph rebuild`. Matrix still **15** checks. `backup_recent` warn (T277 live create skipped). `policy_grants` 0 of 3 (T275). |
| Recent pin `b189ad20-ba63-4cfa-a282-333c49996103` (`memory list` 1m, preview `## Objective`) | `--format human` neighbors **(3)** all `in RECALLS` / `KIND session` / **PREVIEW blank**. JSON: three `{external_id,label,direction}` incoming `RECALLS` (no `kind`/`preview` keys). **T262 hook works. T246 F10 blanks the cell.** |
| `graph session 13d5625b-… --format human` | **(4)** memories **with** previews (`## Objective`, ````json`, T246 review, …). Same session is a blank-PREVIEW neighbor row. **The text exists; neighbors does not show it.** |
| Last GitHub PR | [#193](https://github.com/Ryan-AI-Studios/AI-Brains/pull/193) T284 (2026-08-22). `gh pr view --comments`, `/reviews`, `/comments`, `issues/193/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, actions). **No leftover to mint. No T285.** |
| Prior #188 Bugbot | **T284 Completed** `#193`. Not this track. |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081). **0 pending / 0 drift.** Hotspot **#1** `project.rs` (**3.953**). CLI `preflight.rs` **#7**. `graph.rs` **914** / `graph_density.rs` **725** / `projector.rs` **351** — **not** top-10. `doctor.rs` **1855** / `main.rs` **4835** — **do not grow** doctor. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Blank PREVIEW on session RECALLS | T246 F10: `pretty_neighbor_rows` (`graph.rs` **`:260–264`**) fills PREVIEW **only** when `kind == "memory"`. Live 1-hop of a pin is **session**. KIND is scannable; PREVIEW is empty. Audit 6/6. **DoD.** |
| Live E/N 0.13 vs floor 0.50 | Typed provenance graph (T213): edges only for typed events. Coverage already 0.53. Raising live E/N requires projector more-edges or a **mutating** rebuild of 23k nodes / 39k pins. T213/T262 declined retune + fake edges. Status is already **not** false `live`. **Honesty is done. Usefulness of density is optional hermetic, not live rebuild.** |
| `graph rebuild` as DoD | T262 F4/F23: replay cost + Stop-Before. Rebuild does **not** invent captions. **Decline live rebuild.** |
| 2-hop sibling memories as extra pretty rows | Would diverge pretty vs JSON 1-hop. Operator already has `graph session <id>`. Session PREVIEW + existing session command is the remediator. **Decline.** |
| Projector `SessionSummaryCreated` has no edges (`projector.rs` **`:86–93`**) | Adds memory nodes without `SYNTHESIZED_FROM`. Soft residual — not this presentation track. **Decline projector rewrite.** |
| Hierarchy pretty still id-only | Audit hole was **neighbors**. T246 F7 depth indent is enough. **Decline.** |
| T213 floor CLI flags / contracts promote | Soft T213 residuals. **Decline.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|--------|
| Session PREVIEW skip | CLI `graph.rs` `pretty_neighbor_rows` **`:252–274`** | `kind == "memory"` → `memory_preview`; else `String::new()`. **Lift session here.** |
| Memory preview SQL | `memory_preview` **`:234–250`** | `SELECT content FROM memory_projection … LIMIT 1` + `preview_line` 80. Miss → empty. Query err → `?` fails neighbors. Session helper must **fail-open** (warn + `0 memories`), not `?`. |
| Pretty table | `format_neighbors_pretty` **`:157–183`** | Header `DIR LABEL ID KIND PREVIEW`. Widths T246 F2. Fixture **`:686–693`** session `preview: String::new()` — update for AC. |
| Neighbors command | `neighbors` **`:303–349`** | Exact `node_kind`; sort; pretty rows; JSON `format_neighbors_json` frozen keys. |
| Session command (already has text) | `session` pretty **`:477–491`** | `get_session_memories` + `memory_preview` per id. **Reuse count + first id** for neighbor caption. Do **not** change this command’s output as DoD. |
| `get_session_memories` | `ai-brains-graph` `queries.rs` **`:107`** | Recursive `IN_SESSION` + `RECALLS`/`SOURCE_FOR` UNION session `RECALLS`. **Do not change signature** (CLI + `live_graph.rs` + projector tests). |
| clap | `main.rs` `GraphCommands::Neighbors` **`:2534–2541`** | `--format` `auto\|pretty\|human\|text\|json` default `auto`; `--limit`. `after_help` **`:2528`**. `Update` format default **`json`** (T246 F6). **No new flags.** |
| TTY probe | `graph.rs` **`:10`** `std::io::IsTerminal` | T214 F24 closed (crate removed). **Do not revert.** |
| Density floors | `crates/ai-brains-cli/src/graph_density.rs` (crate root, **not** `commands/`) **`:10–16`** | `MIN_PINNED=100` `:10`; `MIN_NODES=50` `:12`; `MIN_EDGE_NODE_RATIO=0.50` `:14`; `MIN_MEMORY_COVERAGE=0.10` `:16`. Env overrides `:18–21`. Assessor **`:167`**. **Do not change constants.** (OpenCode **m1**.) |
| Doctor check | `doctor.rs` `check_graph_density` **`:868`** (assess call `:906`) | Soft warn. Matrix **15**. **Do not grow `doctor.rs`.** |
| Update human | `emit_graph_health_human` **`:276–288`** | Labeled lines. Status from assessor. |
| Projector pin / capture | `projector.rs` `MemoryPinned` **`:63–84`**; `project_capture_turn` **`:309–348`** | T262: `Some(turn_id)` → memory + session + `RECALLS`. Legacy `None` → turn + `IN_SESSION`. **Do not rewrite.** |
| `SessionSummaryCreated` | `projector.rs` **`:86–93`** | Node, **no** edge. Soft residual. |
| Hermetic AC6/AC7 | `tests/graph_live_projection.rs` **`:44–106`** | JSON incoming RECALLS; pretty `in`+`RECALLS` + not `No graph node`. **Does not** assert PREVIEW. Lift AC7 / add AC. |
| JSON freeze lock | `tests/graph_human_cli.rs` **`:213–264`** | Keys `memory_id`/`neighbors`; hit keys `external_id`/`label`/`direction`. Pretty scannable. **Keep.** |
| Feature-off | `graph_neighbors__format_pretty__feature_off_exit_2` | Exit **2** + `FEATURE_UNAVAILABLE`. **Keep.** |
| PROTOCOL-COMPAT | `Docs/PROTOCOL-COMPAT.md` **`:94–96`** | Keys unchanged. Pretty next-action human-only. Array-order T246. |
| Wrong-kind next | `PRETTY_NEXT` **`:12`** still `graph update` for `pretty_no_memory_node` / `pretty_no_session_node` | T262 F1 superseded **missing-node**. Not this hole. **Do not restyle.** |
| Hotspots | `project.rs` #1 3.953; `preflight.rs` #7 | **Do not touch.** Helpers stay in `graph.rs`. |

### 2.4 Dependency / standards research (2026-08-22) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). GitHub latest tag **v4.6.6**. **No clap 5.** | **No bump.** No new flags. Additive `after_help` only. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** JSON keys frozen (no new serde fields on `NeighborHit`). |
| `chrono` | lock **0.4.44** | crates.io **0.4.45** (Dependabot #62 open) | **No bump.** |
| `rusqlite` | lock **0.39.0** + sqlcipher + backup | crates.io **0.40.2** (Dependabot #61; T213 L4 `table_exists`) | **No bump.** Preview SQL is `memory_projection` SELECT already used. |
| `uuid` | lock **1.23.1** | crates.io **1.25.0** | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | endoflife.date: 1.98 current (2026-08-20); pin is workspace toolchain | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | comfy-table / tabled / petgraph / Neo4j drivers | **Zero.** Hand-roll like T246. |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Human CLI tables may change; JSON stays stable | [CLIG — Output](https://clig.dev/#output): humans first; scripts opt into `--json`; changing human output is usually OK | Pretty PREVIEW is human-only. JSON keys frozen (T246 F5). `human` already ≡ pretty (`resolve_graph_format`). |
| Neighbor lists need captions, not raw ids | [Neo4j Browser](https://neo4j.com/developer/neo4j-browser/): nodes are assigned **captions** from properties (`name`/`title`), not internal ids. GraSS `caption: '{name}, born in {born}'` ([Browser styling](https://neo4j.com/docs/browser/operations/browser-styling/), crawled 2026-08-19). Support: missing caption → blank nodes | Session KIND + UUID is the blank-node case. Caption = count + first memory line (properties we already have). Do **not** use generic `graph_node.label` `"Session"`. |
| Typed/sparse provenance graphs are healthy | T213 Adaptive GraphRAG / Yu 2026; [TRACE-KG 2026](https://arxiv.org/html/2604.03496v1) compact typed graphs vs OpenIE dump; [RIDE 2026](https://www.sciencedirect.com/science/article/abs/pii/S0031320325005369) **intrinsic** density ≠ stuffing facts | Do **not** retune 0.50 or add untyped edges to “pass” live E/N. Honesty is the assessor; usefulness is captions. |
| clap `after_help` / `value_parser` | [docs.rs/clap/4.6.6 `Command::after_help`](https://docs.rs/clap/4.6.6/clap/struct.Command.html) | Keep derive parser set. No clap 5. |
| Event-sourced projections | Azure Event Sourcing (T262 cite): replay from events; do not invent | No projector rewrite; no historical `MemoryPinned` backfill (T262 F35). |

**N/A:** SQLCipher page crypto, schtasks, llama.cpp `/health`, T180 preflight DTO, ISO 27001 deletion logs (not retention).

**Could not verify:** live E/N after a full operator-vault rebuild (Stop-Before; not DoD). Hermetic E/N after one pin is the density regression (2 nodes + 1 edge = 0.50 → `live` if nodes ≥ `MIN_NODES` skip… **check:** `MIN_NODES=50` so small hermetic vault **skips** sparse/orphan arms → status `live`/`skip` per T213 small-vault. Re-read assessor at execute; AC is “not false `empty`” + remediator honest, not “must print `live` on a 2-node vault.”)

**ledgerful / ai-brains:** `preflight --summary`; `project whoami` mismatch false; `graph update --format human` sparse 0.130; `graph neighbors` 3 blank session rows; `graph session` 4 previews; `ledgerful doctor` (4 warn, work root this repo); ledger 0 pending / 0 drift; `index --incremental`; `search "pretty_neighbor_rows"` → `graph.rs:252`; `scan --impact` CLEAN at `400dd78`; `hotspots` project.rs #1 (do not grow); `recall` T246 review-track dumps (PATH ranking T274; not this hole). Semantic `ask` not required (search hit).

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `977c5e7e`. Agy fold-in `384ed242`. OpenCode fold-in `0765b916`. Implement starts a **FEATURE** TX. |
| **F1 — T246 F10 lift (session)** | Pretty PREVIEW is filled when `kind == "memory"` (**unchanged**) **or** `kind == "session"` (new). Other kinds (`turn`, `project`, `source`, …) stay empty this track. |
| **F2 — Session caption SOOT** | `format_session_neighbor_preview(n, first_preview) -> String`: always `"{n} memories"` (n may be 0). If `first_preview` is non-empty after trim, append `" · "` + that string. Cap the **whole** caption with `truncate_preview_chars(..., 80)` (T216/T250). Never the literal `"Session"` from `graph_node.label`. |
| **F3 — Count = `graph session`** | `n` = `get_session_memories(session_id)` length (same walk as `graph session`). First preview: sort ids lexicographic (same as session pretty), `memory_preview` the first id; if that preview is empty, try subsequent ids until one is non-empty or the list ends. Do **not** change `get_session_memories` signature. N+1 session walks for ≤50 pretty session-rows is acceptable (T246 F11 class). |
| **F4 — Fail-open** | `get_session_memories` / `memory_preview` / lock errors on the session arm → `tracing::warn` + caption `"0 memories"` (or whatever `n` succeeded). **Do not** `?` those errors out of `neighbors` (T262 F18 pattern). Memory-kind arm may keep today’s `?` on `memory_preview` (pre-existing; not this lift). **F33** makes this structural. |
| **F5 — JSON keys frozen** | Neighbors `{ memory_id, neighbors: [{ external_id, label, direction }] }` only. **No** `kind` / `preview` / `truncated` keys (T246 F5 / T262 F13). PROTOCOL-COMPAT array-order unchanged. |
| **F6 — `NeighborHit` / `get_neighbors` frozen** | Do not change serde or signature. Recall `--graph-boost` stays source-compatible (T246 F10 / T262 F33). |
| **F7 — T213 floors frozen** | `MIN_EDGE_NODE_RATIO=0.50`, `MIN_MEMORY_COVERAGE=0.10`, `MIN_PINNED=100`, `MIN_NODES=50`, env names, verdict priority, SQL gather — **untouched**. Doctor check count stays **15**. |
| **F8 — No live rebuild** | Do **not** run `ai-brains graph rebuild` on the operator vault as planning or as DoD. Even after go: Stop-Before unless the owner explicitly confirms that remediating action. Hermetic temp vaults only. |
| **F9 — Remediator frozen** | Graph-on sparse → `ai-brains graph rebuild` (T232). Graph-off → `GRAPH_REINSTALL_SOOT` only. Do not point neighbors PREVIEW at rebuild. |
| **F10 — Cargo default-off** | `ai-brains-cli` `default = []` stays (T200 / T262 F16). Feature-off `graph *` exit **2** + `FEATURE_UNAVAILABLE` + `GRAPH_REINSTALL_SOOT`. |
| **F11 — No projector rewrite** | Do not add edges to `SessionSummaryCreated`. Do not fake `SYNTHESIZED_FROM`. Do not remint historical ids. T262 capture/`turn_id` path stands. |
| **F12 — No pin bumps / new crates** | clap 5, rusqlite 0.40, uuid 1.25, chrono 0.4.45, serde_json 1.0.151, comfy-table — **forbidden**. Workspace **0.1.1**. |
| **F13 — Do not grow doctor / hotspots** | No `doctor.rs` / `project.rs` / `preflight.rs` / `sync.rs` / `ranking.rs` edits. Density doctor already honest. |
| **F14 — Same-file helper + units** | `format_session_neighbor_preview` and `pick_first_nonempty` are `pub(crate)` in `graph.rs` with same-file units (T284 F41 class). Do **not** extract `graph_preview.rs` this track (`graph.rs` is 914, not T255-scale). Do **not** make the helpers `pub` for `tests/`. |
| **F15 — PATH-behind** | Live PATH is T270. Do **not** `cargo install` unless the user asks. Tests/manual AC use `cargo run --features graph` / hermetic bin. |
| **F16 — Capture independence** | Pretty + read-only SQL. No new events, no models, no contracts DTO, no CLI reqwest. |
| **F17 — `std::io::IsTerminal` stays** | Do not re-add `is-terminal` crate. |
| **F18 — No 2-hop pretty rows** | Neighbors stay 1-hop. Session caption may mention `N memories`; operator runs `graph session` for the list. |
| **F19 — No hierarchy / mermaid** | T246 F7/F17 stand. Soft residual. |
| **F20 — Cross-model** | FEATURE (CLI contract / pretty). After Phase-1 review clean, run read-only `codex-review`. |
| **F21 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals go to `conductor/deferred.md`. |
| **F22 — Decline peers** | T279 Safety; T280 `--scope` hint; T281 nightly 750 ms; T282 `context --show`; T283 list cwd-first; leftover rebind; T240 F2; T255 750 ms; T263 H2; T266 JSON freeze for `graph update` default. |
| **F23 — last-PR Cursor** | #193 empty → **N/A**. #188 closed by T284. Dependabot `#61` rusqlite **not** absorbed. **No T285.** |
| **F24 — Docs** | CAPABILITIES graph table: pretty PREVIEW for session = `{n} memories · first line`. OPERATIONS one sentence (neighbors captions; update ≠ rebuild). PROTOCOL-COMPAT: keys unchanged; preview is **human-only**. Root CHANGELOG T278 row. Skill one-liner if the graph section exists. |
| **F25 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. Existing T213/T232/T246/T262 units stay green except the session-preview fixture that this track supersedes (update those asserts). |
| **F26 — Live classify-only** | On go, `cargo run --features graph -- graph neighbors <live-id> --format human` on a rediscovered pin (or `b189ad20-…` if still present). **Do not** pin to the live vault as DoD. **Do not** rebuild. |
| **F27 — Small-vault density** | Hermetic 2-node vault may **skip** sparse arms (`MIN_NODES=50`). AC: `graph update --format human` status is `live` **or** `empty` skip — **never** a false `sparse` on a dense-enough small graph, and never invent `live` when E/N is actually 0. Re-read `assess_graph_density` at Phase 0. Do **not** change `MIN_NODES`. |
| **F28 — `human` ≡ pretty** | `--format human` already resolves to pretty (`resolve_graph_format`). DoD commands may use `human` or `pretty`. No new token. |
| **F29 — Privacy** | Session caption uses `preview_line` (role strip + 80). Same content `graph session` / `memory list` already print. Do not print keys. |
| **F30 — clap `after_help`** | Additive sentence on `GraphCommands`: session PREVIEW is `{n} memories · first line`. Do not restack T204 groups. |
| **F31 — Stop-before extras** | No `.env` rewrite, no schtasks, no `retention apply --confirm`, no live `policy bootstrap`, no live leftover rebind. |
| **F32 — Inherit T262 next-action** | Missing-node pretty still rebuild iff vault has the id. Present-empty neighbors still **no** remediator. This track does not restyle `PRETTY_NEXT`. |
| **F33 — Session I/O helper returns `String`** | Agy **m1.** Private/`pub(crate)` `session_neighbor_caption(ctx, searcher, session_id) -> String` (name flexible) **never** returns `Result`. Internals use `match` / `if let` + `tracing::warn` on `get_session_memories`, session-arm `memory_preview`, and lock errors. `pretty_neighbor_rows` session arm does **not** use `?`. `node_kind()?` stays **before** the kind branch (pre-existing). Memory-kind arm unchanged (`memory_preview()?`). Do not fork a second `memory_preview`. |
| **F34 — Pure skip-loop selector** | OpenCode **m2 / O1.** `pick_first_nonempty(previews: &[String]) -> Option<String>` is `pub(crate)` in `graph.rs`: skip items whose `trim()` is empty; return the first remaining as-is (not re-trimmed into a new semantic). `None` if all blank. `session_neighbor_caption` fetches previews in lex id order (fail-open `""` per id), `n = ids.len()`, `first = pick_first_nonempty(&previews).unwrap_or("")`, then `format_session_neighbor_preview(n, first)`. **AC14 is this unit** — not an I/O stub. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit (same-file, Agy **O2** cases): `format_session_neighbor_preview(0, "") == "0 memories"`; `(1, "preview")` contains `1 memories` + `preview` + ` · `; `(3, "hello")` same pattern; whitespace-only first_preview → **no** ` · `; 200-char ASCII first_preview caption `.chars().count() <= 80` and ends with `…`; CJK-over-budget first_preview (Agy **O1**) `.chars().count() <= 80`, no panic, no mid-char slice. Cap via existing `truncate_preview_chars` — do **not** byte-slice. |
| **AC2** | Unit: `format_neighbors_pretty` fixture one incoming `RECALLS` **session** with preview `"2 memories · pin text"` — header + `in` + `RECALLS` + `session` + `2 memories`. JSON helper still emits only `incoming`/`outgoing`/`external_id`. |
| **AC3** | Hermetic graph-on: `pin` unique DECISION → `graph neighbors <id> --format pretty` contains `session` and `memories` and at least one non-whitespace PREVIEW character on that row. **No** `graph rebuild` in the test. Extends T262 AC7. |
| **AC4** | Same hermetic: `--format json` keys **exactly** `memory_id`, `neighbors`; each hit keys **exactly** `external_id`, `label`, `direction`. T246 AC7 / `graph_human_cli` lock stays green. |
| **AC5** | F33: session arm of `pretty_neighbor_rows` has **no** `?` on `get_session_memories` or `memory_preview`. Helper returns `String`. Err → `"0 memories"` (or `"{n} memories"` if count succeeded) + `tracing::warn`. Neighbors exit **0**. Memory-kind `?` may remain. |
| **AC6** | Feature-off: `graph neighbors <id> --format pretty` exit **2** + `FEATURE_UNAVAILABLE` (existing). |
| **AC7** | Clap: `graph neighbors --format xml <id>` exit **2** (existing). |
| **AC8** | `graph update --format human` on the **live** vault (classify-only): `status:` is `sparse` (or `live`/`empty` if the vault changed); **not** a unlabeled `"live"` JSON blob. `remediation:` still `ai-brains graph rebuild` while sparse + graph-on. Do **not** rebuild. |
| **AC9** | Existing T262 AC6 JSON incoming `RECALLS` stays green. Existing T213/T232 density units stay green. |
| **AC10** | On go, classify-only live: `cargo run --features graph -- graph neighbors <rediscovered-pin> --format human` shows `memories` in PREVIEW. PATH may stay blank until operator install (F15). |
| **AC11** | Docs: CAPABILITIES + OPERATIONS + PROTOCOL-COMPAT human-preview note + CHANGELOG T278. |
| **AC12** | `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` on touched files; no `unwrap`/`expect`/`panic` in production graph.rs path. |
| **AC13** | Diff does **not** include `project.rs`, `preflight.rs`, `doctor.rs`, `sync.rs`, `projector.rs`, `graph_density.rs` (floors). |
| **AC14** | Unit on `pick_first_nonempty` (OpenCode **m2/O1**; Agy **m2**): `&["", "  ", "hello"]` → `Some("hello")`; `&["pin"]` → `Some("pin")`; `&["", "   "]` → `None`; empty slice → `None`. `format_session_neighbor_preview(3, pick.unwrap_or(""))` with that `"hello"` still contains `3 memories` + `hello` + ` · ` (`n` is full list length, not index of first nonempty). No vault I/O in this unit. |

---

## 5. Design notes

### 5.1 Caption, not a second query language

`graph neighbors` stays 1-hop. The daily path is: pin → neighbors shows session **with** `N memories · first line` → optional `graph session <id>` for the list (already pretty). Filling PREVIEW is a T246 F10 lift, not a new subcommand.

### 5.2 Why `{n} memories` even when n=0

A session node with zero RECALLS/IN_SESSION walks is still a real neighbor. Blank PREVIEW was the audit hole; `0 memories` is honest and scannable. Fail-open uses the same token.

### 5.3 Density

Live E/N 0.13 with coverage 0.53 is a **typed** graph, not a broken assessor. T213 research + TRACE-KG/RIDE 2026: do not densify with untyped edges. This track’s density DoD is **honesty regression** (AC8/AC9/AC13), not a live rebuild.

### 5.4 Helper placement

`format_session_neighbor_preview` and `pick_first_nonempty` are pure (no vault). I/O lives in `session_neighbor_caption` → `String` (F33), which calls `pick_first_nonempty` then `format_session_neighbor_preview`. `pretty_neighbor_rows` session arm does not `?`.

---

## 6. Non-goals

- Live `graph rebuild` / nightly auto-rebuild
- T213 floor retune or CLI threshold flags
- Cargo default graph-on / Release graph-on
- Projector more-edges / WCC / Cozo multiplex
- 2-hop pretty, mermaid, ASCII tree, batch `node_kinds`
- `GraphHealthOutput` → `ai-brains-contracts`
- Hierarchy captions
- T279–T283, leftover rebind, T240 F2
- clap 5 / rusqlite 0.40 / DTO required keys
- `cargo install`

---

## 7. Verification plan (TDD — failing names first)

Red (must fail on today’s tree):

1. `format_session_neighbor_preview__zero_and_blank__zero_memories_no_dot` — AC1
2. `format_session_neighbor_preview__count_and_first__dot_and_cap_80` — AC1 (include `(1,"preview")` + CJK)
3. `format_neighbors_pretty__session_recalls__preview_shows_memories` — AC2 (today’s fixture has empty session preview)
4. `pin__graph_on__neighbors_pretty__session_preview_nonblank` — AC3 (T262 AC7 does not assert PREVIEW)
5. `pick_first_nonempty__blank_then_hello__some_hello` — AC14 (required pure helper; no I/O stub)

Green: F1–F4 in `pretty_neighbor_rows`; F14 units; AC4/AC6/AC7/AC9 stay.

Manual (on go): AC8 live `graph update --format human`; AC10 `cargo run --features graph` neighbors of a rediscovered pin; **no** live rebuild; **no** live pin unless owner asks.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| `get_session_memories` on a huge session × 50 neighbor rows | Pretty cap 50; same cost as running `graph session` 50 times. Soft later: COUNT+LIMIT 1 SQL. Not DoD. |
| Caption leaks more than `graph session` | Same `preview_line` SOOT (F29). |
| Small hermetic `graph update` status surprise (`MIN_NODES=50`) | F27 / Phase 0 re-read assessor. Do not retune. |
| PATH still blank after merge | F15 / AC10. Operator `cargo install --features graph`. |
| Live rebuild pressure | F8 Stop-Before. Honesty already prints the remediator. |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit graph sparse E/N ~0.11; neighbors PREVIEW blank | **Absorb** F1–F4 / AC1–AC3 / AC8 — captions DoD; density honesty regression |
| T246 F10 memory-only PREVIEW | **Lift** F1 — session added; memory unchanged |
| T246 F18 projector completeness | **Partial** — T262 closed pin `RECALLS`; session **caption** this track; projector edges **decline F11** |
| T213 false `live` / doctor `graph_density` | **Already T213/T232** — AC8/AC9 regression |
| T213 floor flags / contracts promote / rusqlite `table_exists` / two-tier coverage | **Decline** F7/F12 |
| Auto rebuild / projector more edges / default-on / WCC | **Decline** F8/F10/F11 |
| T246 F17 mermaid/tree / batch `node_kinds` | **Decline** F19 |
| T213 F31 event↔graph freshness | **Decline** — T246 F19 |
| T262 F15 density floors / F16 default-off / F23 live rebuild | **Affirm** F7/F8/F10 |
| last-PR Cursor #193 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` |
| Dependabot `#61` rusqlite 0.40.2 / `#62` chrono 0.4.45 | **Decline** F12 — standing freeze; **no T285** |
| leftover `7d97a456` / `context --show` / `project list` | **Decline → T276/T282/T283** |
| T279 Safety / T280 hint / T281 nightly | **Decline** peers |
| T240 F2 / T255 750 ms / T263 H2 / clap 5 / DTO required keys | **Decline** F12/F22/F31 |
| Identity mismatch `7d97a456` vs `fcb8a40f` | **Not this track** — leftover data T276; adopt-path T258; shell leftover T282 |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `changeguard` | **Decline** — not graph captions |
| T277 live `backup create --no-prune` | **Decline** — T277 Completed hermetic; live skip residual |

---

## 10. Implement order (on go)

1. Phase 0 re-verify `pretty_neighbor_rows` `:260`, `memory_preview` `:234`, `get_session_memories` `:107`, density floors `:10–16`, clap Neighbors `:2534`, #193 still empty, pins unchanged.
2. Red AC1–AC3 + AC14 (`pick_first_nonempty`).
3. Green F1–F4 + F14 + F33 + F34 + F30 `after_help`.
4. Docs F24.
5. Targeted nextest `graph_human_cli` + `graph_live_projection` + `graph.rs` units; clippy `-p ai-brains-cli --all-targets`.
6. Review → `review.md`; Codex (F20); full gate at closeout; implement-track Phase 6 publish.

---

## 11. Soft residuals

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` until `cargo install --features graph` | F15 |
| Live vault still sparse E/N ~0.13 | Honest; rebuild Stop-Before (F8) |
| `SessionSummaryCreated` nodes without edges | F11 / T213 projector class |
| N+1 `get_session_memories` on huge sessions | §5; COUNT+LIMIT 1 later |
| Other-kind captions (`decision` title, `project` name) | F1 v1 session+memory only |
| Hermetic AC3 empty-first memory | OpenCode O2 declined as DoD; AC14 covers skip-loop |
| Hierarchy pretty still id-only | F19 |
| `pretty_no_memory_node` still `graph update` | F32 / T262 leftover wrong-kind |
| T279–T283 peers | F22 |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/graph.rs` | F1–F4 `pretty_neighbor_rows`; F14 helper + units; fixture AC2 |
| `crates/ai-brains-cli/src/main.rs` | F30 additive `after_help` only |
| `crates/ai-brains-cli/tests/graph_live_projection.rs` | AC3 lift / new test |
| `crates/ai-brains-cli/tests/graph_human_cli.rs` | AC4 stays; pretty may assert `memories` |
| `Docs/CAPABILITIES.md` | graph table caption sentence |
| `Docs/OPERATIONS.md` | one paragraph |
| `Docs/PROTOCOL-COMPAT.md` | §5 human-preview note (keys unchanged) |
| `CHANGELOG.md` | T278 row |
| `.agents/skills/ai-brains/SKILL.md` | graph neighbors pretty one-liner if section exists |
| `conductor/*` | Planned → Completed only on implement closeout |

**Do not touch:** `projector.rs`, `graph_density.rs`, `doctor.rs`, `project.rs`, `preflight.rs`, `sync.rs`, `ranking.rs`, `ai-brains-contracts`.

---

## 13. AI fold-in

Inputs: `agy-review.md` (HEAD `46fc872`) + `opencode-review.md` (HEAD `5defcc5`). Product crates identical to `400dd78`. **B 0 / M 0** both harnesses. last-PR #193 still empty. No T285. Do **not** edit the review files.

### Per-AI

| Source | Item | Disposition |
|--------|------|-------------|
| Agy m1 | Fail-open isolation: no `?` on session-arm DB/lock | **Already** F4; **folded** F33 / AC5 (`session_neighbor_caption` → `String`) |
| Agy m2 | Skip empty first memory preview; try subsequent ids | **Already** F3; **folded test** AC14 (now F34 pure helper) |
| Agy O1 | UTF-8 safe 80-char cap (`truncate_preview_chars`) | **Already** F2 (`display_text.rs` CJK/emdash units); **folded** AC1 CJK caption case — do not byte-slice |
| Agy O2 | Pure units `(0,"")` / `(1,"preview")` / long / whitespace | **Already** F14 / AC1; **tightened** AC1 case list |
| OpenCode m1 | `graph_density.rs` crate-root `:10–16` not `:14–16` | **Folded** §2.3 path + line range; doctor `:868` / assess `:906` |
| OpenCode m2 / O1 | AC14 skip-loop must be a pure `pick_first_nonempty` | **Folded** F34 / AC14 required-pure (no I/O stub) |
| OpenCode m3 | HEAD `5defcc5` / pinned / E/N snapshot drift | **Folded** §2.1 volatile snapshot; Phase 0 re-dogfood stands |
| OpenCode O2 | Empty-first case in hermetic AC3 | **Decline as DoD** — pin DECISION has content; skip-loop DoD is AC14. Soft residual only |
| last-PR #193 Cursor | empty | **Affirm N/A** — no T285 |
| No B/M | — | Nothing to decline of B/M |

### Declined / not new design

| Item | Why |
|------|-----|
| Always `{n} memories · {first}` even when first is blank | F2 — append ` · ` only after trim-non-empty (Agy summary oversimplified; F2 stands) |
| JSON object is only three keys | F5 — object keys `memory_id` + `neighbors`; **hit** keys are the three (`external_id`, `label`, `direction`) |
| Byte-slice / new truncate helper | F2 — reuse `truncate_preview_chars` |
| `node_kind()?` fail-open | Pre-existing before the kind branch; F33 scopes session-arm I/O only |
| Hermetic AC3 empty-first memory | OpenCode O2 — AC14 is the skip-loop lock; AC3 stays pin → non-blank PREVIEW |
| clap 5 / rusqlite 0.40 / live rebuild / floor retune | Unchanged F8/F7/F12 |

### Pins locked by fold-in

1. **F33 / AC5:** session caption I/O helper returns `String`; no `?` in the session arm.
2. **F34 / AC14:** `pick_first_nonempty` same-file units; `n` is full list length; no I/O stub.
3. **AC1:** explicit `(0,"")`, `(1,"preview")`, whitespace no-dot, 80-cap + CJK via `truncate_preview_chars`.
4. **§2.3:** density constants `graph_density.rs` crate-root `:10–16`; doctor `check_graph_density` `:868`.
5. **F2 / F3 / F4 / F14:** already specified; Agy confirmed live `pretty_neighbor_rows` `:252`; OpenCode confirmed `NeighborHit` `queries.rs:7–11`.
6. **§2.1:** OpenCode fold-in HEAD `5defcc5`; product crates identical to `400dd78`; pinned count volatile.

---

## How to ≥8 (this plan)

`--format human` neighbors show `{n} memories · first line` for the session neighbor. Density remediator stays `graph rebuild` when sparse/graph-on. Hermetic pin → PREVIEW non-blank **without** live rebuild. Live 39k E/N remains honestly `sparse`.
