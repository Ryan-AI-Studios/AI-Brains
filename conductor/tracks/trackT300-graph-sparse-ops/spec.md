# T300 — Live graph sparse: owner-confirm rebuild, density stays honest

- **Track ID:** T300-GraphSparseOps
- **Status:** **Planned** (Pending until **go**)
- **Category:** OPS / GRAPH / UX
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `graph update` **8/8** honest sparse E/N ~0.14; **not working / opp:** useful graph vs 40k pins; doctor `graph_density` warn. Placeholder minted with T285–T300 (`76c4db9`). T213 ✅ floors; T232 ✅ remediator; T262 ✅ pin=node; T278 ✅ PREVIEW + **do not retune floors**; T293 ✅ neighbors ranking. This track is the **operator rebuild** (T295 analog).
- **Depends on:** T213 ✅ density doctor; T232 ✅ capability remediator `ai-brains graph rebuild`; T262 ✅ pin = memory node without rebuild; T278 ✅ floors frozen; T188 ✅ daemon Safety probe for mutate; T246 ✅ `graph update` JSON keys
- **Blocks / feeds:** Operators who follow doctor/`graph update` remediator get a **useful** rebuild (daemon-safe, density printed) instead of a silent DELETE+replay that races the live writer. Series closer for **T285–T300**. No T301 from last-PR Cursor.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “graph sparse live rebuild” (every T278/T293/T295–T299 decline pointer); T278 F8 Stop-Before **lifted to owner-confirm** (same class as T295 live `--no-prune`); T232 remediator string freeze
- **Not absorbed (DoD):** T213 floor retune (`MIN_EDGE_NODE_RATIO=0.50`); Cargo `default` graph-on (T200); projector rewrite / fake edges / WCC; `GraphRebuilder` `read_all_events` RAM rewrite; `--confirm` on rebuild (would dead-end T232 remediator); doctor 16th check / grow `doctor.rs`; nightly auto-rebuild; T240 F2; leftover `--write`; clap 5 / rusqlite 0.40; daemon start/stop as silent DoD
- **Research date:** 2026-08-25 (plan dogfood HEAD `d953a20` T299 `#215`. Product `src/` = T299. PATH **0.1.2** 2026-08-22 19:41 **graph-on**, has T213/T232 silent rebuild, **not** this remediator UX.)
- **Ledger:** planning DOCS TX `d7d6f57c-4f12-4cc4-8425-395aa678f6c8`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** rewrite `.env` (T240 F2). Do **not** `graph rebuild` the live vault as planning. Do **not** `daemon stop` / `daemon start` as planning. Do **not** retune T213 floors. Do **not** rewrite `GraphRebuilder` / `GraphProjector`. Do **not** grow hotspot `project.rs` / `sync.rs` / `governed_common.rs` / `forget.rs` / `doctor.rs` / `graph_density.rs` (floors). Helpers in `graph.rs` (**1130** lines — not top-10). Do **not** print or commit `AI_BRAINS_KEY`. Do **not** live `retention apply --confirm`, leftover `rebind-path --write --yes`, or `safety sync` without `--dry-run`.

---

## 1. Objective

1. **The remediator is usable.** Today `graph update --format human` and doctor `graph_density` honestly say `sparse` (live E/N **0.149** vs floor **0.50**) and point at `ai-brains graph rebuild`. Running that command prints **nothing** on stdout (only `tracing::info`), does **not** probe the daemon, and DELETEs `graph_node`/`graph_edge` while this machine’s daemon is **Running**. After go: mutating rebuild **fail-closes** when the daemon is up; `--dry-run` previews current density + event COUNT without DELETE; a successful rebuild **emits the same density report as `graph update`** (human default).
2. **Density stays honest, not inflated.** Do **not** retune floors, invent edges, or claim live E/N ≥ 0.50. After owner rebuild, `graph update` + doctor `graph_density` **agree** (`ok` **or** still `sparse` — **pass-with-observed-data**, never force `live`). Typed provenance graphs can stay under 0.50 after a full replay (T213 / T278).
3. **T262 stays.** A just-pinned memory is still a graph node with `RECALLS` **without** rebuild. Hermetic AC6/AC7 stay green. Rebuild is recovery, not the daily pin path.
4. **North star.** Capture independence: rebuild still replays the append-only event log through the existing projector. No models. No new events. No contracts DTO. No Cargo default-on. The Windows-first vault’s doctor remediator must not race the live writer or look like a no-op.

This unblocks daily ops honesty: T213/T232 made density **honest**; T262/T278/T293 made neighbors useful. The remaining hole is the **operator remediator** — same class as T295 live `--no-prune` create.

---

## 2. Live baseline (re-scan 2026-08-25)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `d953a20` T299 squash `#215`. Tree **CLEAN**. `origin/main` = HEAD (`left-right` `0 0`). Branch `main`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. **Graph-on** (`graph update --format human` works). Has T213/T232 silent rebuild. **Does not have T285–T299.** **Do not `cargo install`.** Tests/manual AC use `cargo run --features graph` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4161** (volatile). In-context **0/0/0**. Word **493**. Capture independence holds. |
| PATH `graph update --format human` | `status: sparse` `density: warn` `nodes: 31201` `edges: 4635` `pinned_memories: 48787` `memory_nodes: 28640` `edge_node_ratio: 0.14855…` note sparse below floor **0.5**; `remediation: ai-brains graph rebuild`. Exit **0**. Coverage 28640/48787 ≈ **0.587** (above `MIN_MEMORY_COVERAGE` 0.10). Warn is **E/N**, not coverage. |
| `doctor --summary` | `status=degraded` `ok=11 warn=2 fail=0 skip=2`. **`graph_density` warn** — same sparse sentence + remediator `ai-brains graph rebuild`. Other warn: `recovery_kit_event` (**not this track**). Matrix still **15**. |
| `graph rebuild --help` | `Rebuild graph from all events`. **No flags.** **No after_help.** No daemon / density / dry-run honesty. |
| `graph update --help` | `--format` default **json**; tokens `json\|auto\|human`. **Unchanged this track.** |
| `daemon status` | **Running** PID **4536**. Vault `C:\dev\ai-brains\vault.db` **147.0 MB**. Memories **48787**. LLM `:8081` Open; Embedding `:8083` Open. **Do not stop as planning.** Live mutate rebuild **would race** LiveGraphHook. |
| `graph.rs` `rebuild` | `:386–397` — `tracing::info` start/complete; `GraphRebuilder::rebuild()?;` **no stdout**; **no daemon probe**. |
| `GraphRebuilder` | `rebuild.rs:20–59` — `DELETE FROM graph_edge` then `graph_node`; `read_all_events()` into a `Vec`; `projector.apply` each; `flush`. **Engine freeze.** |
| Last GitHub PR | [#215](https://github.com/Ryan-AI-Studios/AI-Brains/pull/215) T299 (merged 2026-08-25T13:50:23Z). `gh pr view --comments`, `/reviews`, `/comments`, `issues/215/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, `#58` tower-http, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / hotspots | Hotspot **#1** `project.rs` (**3.871**) — **do not touch.** `sync.rs` #2. `governed_common.rs` #3. `context.rs` #4. `forget.rs` #5. `graph.rs` **1130** / `graph_density.rs` / `rebuild.rs` **not** top-10 — **extend `graph.rs`.** `doctor.rs` **1738** — **do not grow.** |
| Ledger | **0 pending / 0 drift** at scan (before this DOCS TX). |
| `ISSUES.md` | **Does not exist.** |
| Planning live rebuild | **Not run.** `--help` + health only. |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Honest sparse + unused remediator | T213/T232 shipped honesty. Live E/N **0.149** vs floor **0.50**. Doctor still warns. Usefulness of density is **running** rebuild (T295 class), not lowering the floor. **DoD = remediator UX + owner-confirm live.** |
| Silent rebuild stdout | CLIG: too little = operator wonders if it worked. Backup create prints `Backup created and verified:`. Rebuild prints nothing. **DoD: emit density.** |
| Daemon Running | Event-sourcing rebuilds need exclusive access (Architecture Weekly 2026 advisory-lock rebuilds; event-driven.io truncate-and-replay **if you can afford downtime**). LiveGraphHook on the daemon will project into tables this CLI just DELETEd. T188 restore already fail-closes. **DoD: Safety probe.** |
| Floor retune 0.50 → live E/N | Typed provenance (T213 Adaptive GraphRAG / TRACE-KG). Coverage already **0.59**. Raising E/N requires projector more-edges. T278 F7 freeze. **Decline.** |
| `--confirm` on rebuild | T232 remediator is exact `ai-brains graph rebuild`. `--confirm` would make doctor copy-paste a dead-end (T232 graph-off analog). Daemon fail-closed is the safety gate. **Decline.** |
| `GraphRebuilder` streaming | `read_all_events` loads all envelopes. 147 MB vault — RAM risk. Engine rewrite is a different track. **Decline as DoD; residual.** |
| Cargo default-on | T200 / T262 F16. Graph-off rebuild stays exit **2**. **Decline.** |
| Nightly auto-rebuild | Stop-Before. **Decline.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| CLI rebuild | `graph.rs` `rebuild` **`:386–397`** | Silent. **Wrap:** daemon probe + dry-run + `GraphRebuilder` + emit health. |
| CLI update | `update` **`:605–663`** | Gather + assess + `GraphHealthOutput`. **Reuse** for post-rebuild / dry-run current density. Default `--format json` **frozen** (T246 F6). |
| Human emit | `emit_graph_health_human` **`:372–384`** | Labeled lines. Reuse. |
| JSON DTO | `GraphHealthOutput` **`:35–48`** | `nodes`, `edges`, `pinned_memories`, `memory_nodes`, `edge_node_ratio`, `density`, `status`, `note`, optional `remediation`. **Freeze keys** (T213 / PROTOCOL-COMPAT `:96`). |
| Floors | `graph_density.rs` **`:10–16`** | `MIN_PINNED=100`; `MIN_NODES=50`; `MIN_EDGE_NODE_RATIO=0.50`; `MIN_MEMORY_COVERAGE=0.10`. Env `:18–21`. Assessor **`:167`**. **Do not change.** |
| Remediator SOOT | `REMEDIATION_REBUILD` **`:140`** | Exact `ai-brains graph rebuild`. T232 F4. **Do not change.** |
| Doctor check | `doctor.rs` `check_graph_density` **`:868`** | Soft warn. Matrix **15**. **Do not grow `doctor.rs`.** |
| Engine | `ai-brains-graph/src/rebuild.rs` **`:20–59`** | DELETE + `read_all_events` + apply. Idempotent test `rebuild_is_idempotent.rs`. **Do not rewrite.** |
| Projector pin | `projector.rs` `MemoryPinned` **`:63–84`** | T262. **Do not rewrite.** |
| Daemon Safety | `backup.rs` `probe_restore_daemon_busy` **`:471`** | 3×≥1000 ms. Restore message **`:483–488`** substring classes `daemon is running` / `ai-brains daemon stop` / `sc stop AI-Brains-Daemon`. **Reuse probe + same substring classes.** |
| clap Rebuild | `main.rs` `GraphCommands::Rebuild` **`:2941`** | Unit variant. Dispatch **`:5222`** sync. Parent `async fn run` **`:4117`** — rebuild **may `.await`**. Feature-off stub **`:5240–5250`** exit **2**. |
| Enum after_help | `main.rs` **`:2937`** | Neighbors PREVIEW / T293 prefer. **Do not steal.** Rebuild gets **variant** `after_help`. |
| T262 hermetic | `tests/graph_live_projection.rs` AC6 **`:44`** / AC7 **`:79`** | Pin → neighbors JSON `RECALLS` **without** rebuild. **Stay-green.** |
| Serde keys unit | `graph.rs` `graph_health_output__serde_keys__include_density_fields` **`:675`** | Stay-green. |
| Feature-off | `exit_contract.rs` `graph_update__feature_off` **`:236`** | Exit **2**. Add rebuild sibling (same stub). |
| CAPABILITIES | **`:460–461`** | Update JSON keys; rebuild “Full resync (recovery).” **Extend rebuild row.** Doctor matrix **`:557`** — do not add a 16th. |
| OPERATIONS | Graph health **`:893–949`** | Rebuild snippet **`:927`**. **Extend:** daemon stop first; `--dry-run`; stdout density; may stay sparse. |
| WORKFLOWS | Backup-before-rebuild **`:215–216`** | Additive daemon-stop sentence. |
| PROTOCOL-COMPAT | **`:96`** | `graph update` keys. Rebuild `--format json` **same keys** (document “rebuild JSON = update keys”; do not add a new required DTO). |
| CHANGELOG | Root `CHANGELOG.md` Unreleased | T300 row. |
| CLI-EXIT-CODES | **`:13`** usage 2; store fail 1 | **Add:** mutating rebuild daemon-up → exit **1**; success (including still-sparse) → **0**. |

### 2.4 Dependency / standards research (2026-08-25) — snapshot, re-verify at execute

| Pin | Workspace / lock | crates.io / docs (today) | Action |
|-----|------------------|--------------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | **4.6.6** (docs.rs 2026-08-11). GitHub latest tag **v4.6.6**. **clap 5 not released.** `Command::after_help` current. | **No bump.** Additive Rebuild `--dry-run` + `--format` `human\|json`. |
| `rusqlite` | workspace **0.39.0** | **0.40.2** (Dependabot `#61` open) | **No bump.** Event COUNT is `SELECT COUNT(*) FROM events` on held conn. |
| `serde_json` | lock **1.0.150** | **1.0.151** | **No bump.** `GraphHealthOutput` keys frozen. |
| `tokio` | workspace **1.52** / lock **1.52.3** | crates.io **1.53.1** (`#59` open) | **No bump.** Rebuild becomes `async` to reuse Safety probe. |
| `rstest` | cli dev-dep **0.25** | already in crate | Reuse for daemon/dry-run cases. |
| rustc / edition | **1.95.0** / **2024** | Unchanged | Unchanged. |
| workspace version | **0.1.2** | — | **No bump.** |
| New crates | — | comfy-table / indicatif / petgraph | **Zero.** No spinner crate. |

### 2.5 Online best-practice / implementation research

| Topic | Finding | Use in T300 |
|-------|---------|-------------|
| **[CLIG — Saying (just) enough](https://clig.dev/)** | Too little = user wonders what is going on | Rebuild stdout = density report (same labeled lines as `graph update --format human`). |
| **[CLIG — stdout vs stderr](https://clig.dev/)** | Data on stdout; progress/logs on stderr | Keep `tracing::info` start/complete on stderr. Density is the **result** → stdout. No spinner crate. |
| **[CLIG — Ease of discovery](https://clig.dev/)** | Suggest what to run next | Daemon-up error names `ai-brains daemon stop`. Dry-run notice names the same. Success may still print T232 remediator when sparse (honest). |
| **[CLIG — Human-first](https://clig.dev/)** | Human output may evolve; scripts pin JSON | Rebuild default **human**. `--format json` same keys as update. Do **not** TTY-switch (`auto` not a rebuild token). |
| **Event-driven.io — projection rebuild** (current) | Simplest rebuild = truncate read model then reapply all events **if you can afford downtime** | Existing DELETE+replay is that pattern. Downtime = **daemon stop**. Not blue-green (out of scope). |
| **Architecture Weekly 2026 — exclusive rebuild locks** | Rebuild takes exclusive lock; inline projections skip | Product analog: fail-closed while daemon (inline LiveGraphHook) holds the vault. SQLite busy_timeout is **not** exclusive rebuild. |
| **T213 Adaptive GraphRAG / TRACE-KG** | Typed sparse ≠ unhealthy | Floors frozen. Pass-with-observed-data. |
| **T188 / T189** | Mutating restore/rotate Safety-probe + dry-run notice | Copy probe + substring classes. Do not invent a third policy. |
| **T232 F4** | Graph-on remediator exact `ai-brains graph rebuild` | **No `--confirm`.** |
| **T295 analog** | Live file + after_help; engine frozen | Live rebuild owner-confirm; `GraphRebuilder` frozen. |
| N/A | SQLCipher page crypto, schtasks, llama.cpp `/health`, clap 5, T180 DTO | This track does not touch them. |

**Could not verify:** live E/N **after** a full operator-vault rebuild (Stop-Before this planning pass; daemon Running). Hermetic 2-node vault **skips** sparse arms (`MIN_NODES=50`) — AC is “prints density + T262 still holds,” not “must print `live` on the operator vault.” Exact `events` COUNT without vault SQL (do not print `AI_BRAINS_KEY`). Dry-run COUNT is hermetic SoT.

**ledgerful / ai-brains:** `preflight --summary` pinned **4161**; `graph update --format human` sparse **0.149**; doctor `graph_density` same remediator; `daemon status` **Running**; `ledgerful doctor` 5 warn (legacy `.changeguard` / sig-pin / sig-version / timings / graph-content-stale) — not this hole; ledger 0 pending / 0 drift at scan; `scan --impact` CLEAN at `d953a20`; `hotspots` `project.rs` #1 — do not grow; `search GraphRebuilder` → `rebuild.rs:10` + `graph.rs:391`. Lexical/semantic recall returned T293 plan-audit chrome (PATH-behind ranking) — **not** a contradicting pin; live src `rebuild` `:386–397` is SoT.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `d7d6f57c`. Implement starts a FEATURE TX. |
| **F1 — Live rebuild owner-confirm (hard)** | Mutating `graph rebuild` on the operator vault runs **only** if the owner confirms **at go** (and daemon is Stopped, or owner confirms `daemon stop` first). If owner skips: hermetic ACs + written skip — **not** a floor lie. Track may Complete hermetic like T277; live file remains residual until confirm (T295 later absorbed that residual). |
| **F2 — Floors frozen (hard)** | `MIN_EDGE_NODE_RATIO=0.50`, `MIN_MEMORY_COVERAGE=0.10`, `MIN_PINNED=100`, `MIN_NODES=50`, env names, verdict priority, SQL gather — **untouched**. Doctor check count stays **15**. Do **not** edit `graph_density.rs` except if a compile break (expect **zero** edits). |
| **F3 — Pass-with-observed-data (hard)** | After rebuild, `status` may be `live` **or** `sparse` **or** `empty`. Never coerce `live`. Manual pass = `graph update --format human` and doctor `graph_density` **agree** (same `status` / same remediator-or-omit). |
| **F4 — Post-rebuild stdout (hard)** | Successful mutating rebuild emits `GraphHealthOutput` via existing `emit_graph_health_human` (default) or pretty JSON (`--format json`). Same keys as `graph update`. Exit **0** even when still `sparse`. |
| **F5 — Rebuild `--format` (hard)** | Tokens **`human\|json` only**. Default **`human`**. Unknown / `JSON` / `Pretty` / `auto` → clap `InvalidValue` exit **2**. Do **not** add `auto` (T246 update `auto` stays JSON — do not copy that trap onto rebuild). `graph update --format` **unchanged**. |
| **F6 — `--dry-run` (hard)** | Read-only. Prints current density (same human/json as F4/F5) **then** a `[dry-run]` block: `would DELETE graph_node/graph_edge then replay N events; no mutation.` `N` = `SELECT COUNT(*) FROM events` on the held conn; COUNT `Err` → omit `N` (fail-open: still print the would-DELETE sentence without a number). **Does not** call `read_all_events` / `GraphRebuilder`. Allowed while daemon is up. |
| **F7 — Daemon fail-closed (hard)** | Mutating rebuild (`!dry_run`) uses `probe_restore_daemon_busy` (Safety 3×≥1000 ms). `daemon_up` → `Err` exit **1**. Message substring classes (T188): `daemon is running`, `ai-brains daemon stop`, `sc stop AI-Brains-Daemon`. **`--force` does not exist** and must not be added. Dry-run + daemon_up: stdout NOTICE (restore analog) + continue. Injectable `rebuild_with_daemon_state` for units (no live IPC). |
| **F8 — No `--confirm` (hard)** | T232 remediator stays exact `ai-brains graph rebuild`. Adding `--confirm` would dead-end doctor copy-paste. |
| **F9 — Engine freeze (hard)** | Do **not** edit `rebuild.rs` / `projector.rs` / `queries.rs` `get_neighbors`. CLI wraps `GraphRebuilder::rebuild()`. Idempotent crate test stays green. |
| **F10 — JSON keys freeze (hard)** | Rebuild `--format json` object = T213 `GraphHealthOutput` keys. **No** `next_step` / `events_replayed` / `dry_run` JSON fields on the health object. Dry-run `[dry-run]` lines are **human extra after** the health block; JSON dry-run prints the health object then a **second** stdout line or object? **Pin:** JSON dry-run prints the health object **only** (scripts already have counts); the `[dry-run]` sentence is **human-only** (stderr `tracing::info` + human stdout extra lines). JSON dry-run still must **not** mutate. |
| **F11 — T262 freeze** | Pin printed id is a memory node + `RECALLS` without rebuild. AC6/AC7 stay green. Rebuild must not become required for a new pin. |
| **F12 — Cargo default-off** | `default = []` stays. Feature-off `graph rebuild` (any flags) exit **2** + `FEATURE_UNAVAILABLE` + `GRAPH_REINSTALL_SOOT`. |
| **F13 — No live rebuild as planning** | Planning did not run it. Go: Stop-Before unless owner confirms. |
| **F14 — Pins / crates** | No workspace/lock bumps. Zero new crates. No `indicatif`. |
| **F15 — Capture independence** | Rebuild still event-log replay. No new events. No models. No contracts crate. No new `EventStore` method (COUNT is CLI SQL). |
| **F16 — Isolation** | No T240 F2. No leftover `--write`. No doctor 16th. No floor CLI flags. No `project.rs` / `sync.rs` / `forget.rs` / `doctor.rs` / `graph_density.rs`. No daemon start/stop as DoD. |
| **F17 — PATH** | Do not `cargo install`. Source/hermetic SoT. PATH 0.1.2 until owner asks. Manual AC14 via **`cargo run -p ai-brains-cli --features graph`**. |
| **F18 — last-PR Cursor** | **#215** comments/reviews/issue **empty**. **No T301.** Dependabot `#61` rusqlite / `#58–#62` / `#68–#72` **not stolen**. |
| **F19 — Docs** | CAPABILITIES rebuild row **`:461`** additive (daemon stop; `--dry-run`; stdout density; may stay sparse). OPERATIONS Graph health **`:927–949`** extend — do not add a second Graph heading. WORKFLOWS **`:215`** additive daemon-stop. Root CHANGELOG T300 Unreleased. Rebuild `after_help` on the **variant**. CLI-EXIT-CODES: **add** daemon-up exit **1**; still-sparse success exit **0**. PROTOCOL-COMPAT `:96` additive: rebuild `--format json` uses the same keys. Phase 0 re-locates anchors. |
| **F20 — `graph update` freeze** | Default JSON, `auto` = JSON, human opt-in. Do not TTY-switch. Do not print density on `update` twice. |
| **F21 — High findings** | Live rebuild while daemon Running; retuning 0.50; `--confirm` dead-end remediator; rewriting projector to “pass” E/N; growing `doctor.rs`; `cargo install`; silent stdout left in place; clap 5. |
| **F22 — Help** | `GraphCommands::Rebuild` variant `after_help`: daemon must be Stopped; `--dry-run` first; floors 0.50 may still warn after replay; stdout is density; T232 remediator unchanged; examples `graph rebuild --dry-run` and `graph rebuild`. Enum-level neighbors after_help **unchanged**. |
| **F23 — Exit** | Success (sparse or live) → **0**. Daemon-up mutate → **1**. Feature-off / clap unknown format → **2**. Store/rebuild `Err` keeps today’s fail path (exit 1 class). |
| **F24 — Decline peers** | Floor retune; Cargo default-on; projector more-edges; nightly auto-rebuild; leftover `--write`; T240 F2; T263 H2; T255 750 raise; clap 5 / rusqlite 0.40; `--confirm`; spinner; streaming `read_all_events`. |
| **F25 — Soft residuals** | PATH until install; live skip if owner refuses daemon stop; `read_all_events` RAM; `SessionSummaryCreated` nodes without edges (T278 F11); coverage vs E/N dual (warn is E/N). |
| **F26 — Helper (hard)** | Required `pub(crate) fn rebuild_daemon_busy_message() -> String` in `graph.rs` (or file-local const) with T188 substring classes. Required `rebuild_with_daemon_state(ctx, dry_run, format, daemon_up)` sync core. Production `async fn rebuild` probes then calls it. Units rstest daemon_up × dry_run (AC10). |
| **F27 — Shared health builder** | Extract `fn graph_health_report(ctx) -> Result<GraphHealthOutput>` from `update` so rebuild/dry-run/update **cannot** drift keys. `update` stays the JSON-default CLI. |
| **F28 — Existing tests stay green** | T213 assessor units; T232 capability remediations; T246 JSON neighbors keys; T262 AC6/AC7; T278 PREVIEW; T293 prefer-fill; `graph_health_output__serde_keys`; feature-off update; doctor 15-check matrix. |
| **F29 — Small-vault density** | Hermetic 2-node vault may **skip** sparse arms (`MIN_NODES=50`). AC: rebuild stdout still has `status:` + `nodes:` + `edges:`; never a false `empty` when nodes>0. Do **not** change `MIN_NODES`. |
| **F30 — No progress bar** | `tracing::info` start/complete stays. Do not add `indicatif` / tick lines on stdout (would break JSON). Long-run honesty lives in `after_help` (“minutes on large vaults”). |
| **F31 — Do not COUNT on mutate success JSON** | Extra SQL COUNT is **dry-run only**. Post-rebuild JSON is health-only. |
| **F32 — Dispatch async** | `GraphCommands::Rebuild { dry_run, format } => commands::graph::rebuild(&ctx, *dry_run, format).await`. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Hermetic graph-on vault, ≥1 pin (T262 fixture): `graph rebuild --dry-run` exit **0**. Stdout human contains `status:` and `nodes:`. Contains `[dry-run]` and `no mutation`. Does **not** drop `graph_node` count (neighbors JSON still has `RECALLS` after dry-run — T262 still holds). |
| **AC2** | Same vault: `graph rebuild` (mutate) exit **0**. Stdout human contains `status:` + `nodes:` + `edges:` + `edge_node_ratio:`. After: `graph neighbors <pin-id> --format json` still has incoming `RECALLS` (T262). `graph update --format human` `status:` **equals** rebuild stdout `status:` (`assert_eq!` parsed). |
| **AC3** | Unit/inject: `rebuild_with_daemon_state(..., dry_run=false, daemon_up=true)` is `Err`; message contains `daemon is running` **and** `ai-brains daemon stop` **and** `sc stop`. Does **not** call rebuild engine (hermetic: node COUNT unchanged — or unit does not need a vault if the error returns before `GraphRebuilder`). |
| **AC4** | Inject `daemon_up=true`, `dry_run=true`: `Ok`; human stdout contains NOTICE (`daemon` + `stop`) **and** `[dry-run]`; **no** DELETE. |
| **AC5** | Hermetic mutate `--format json`: parse object; required keys `nodes`, `edges`, `pinned_memories`, `memory_nodes`, `edge_node_ratio`, `density`, `status`, `note` present; `status` ∈ {`live`,`sparse`,`empty`}; **no** `next_step` / `events_replayed`. `density` ∈ {`ok`,`warn`,`skip`}. |
| **AC6** | Stay-green T262 AC6/AC7 (pin without rebuild). **Do not** rewrite those tests except if a fixture helper is reused. |
| **AC7** | Feature-off `graph rebuild` and `graph rebuild --dry-run` exit **2** + `FEATURE_UNAVAILABLE` + reinstall SOOT. |
| **AC8** | Floors unchanged: unit still asserts `MIN_EDGE_NODE_RATIO == 0.50` (existing T213 unit **or** one-liner stay-green in `graph_density.rs` tests — do not retune). Doctor `health_check_order_names__fixed_matrix` still **15**. |
| **AC9** | `graph rebuild --format auto` / `JSON` / `Pretty` → clap exit **2**. `--format json` accepted. `graph update --help` still default json. |
| **AC10** | rstest `rebuild_daemon_busy_message` contains the three T188 substrings. Each of AC3/AC4 covered by `#[case]` or explicit tests. |
| **AC11** | `graph rebuild --help` after_help contains `daemon` / `--dry-run` / `0.50` (or `sparse`) / `graph update`. Enum neighbors after_help still names PREVIEW / prefer-fills. |
| **AC12** | Docs: CAPABILITIES `:461` additive; OPERATIONS Graph health extend; WORKFLOWS `:215` daemon-stop; CHANGELOG T300 Unreleased; CLI-EXIT-CODES daemon-up **1** + still-sparse **0**; PROTOCOL-COMPAT `:96` rebuild JSON = update keys. Phase 0 re-locates. |
| **AC13** | No `ai-brains-contracts` type. No pin bumps. No new crate. `rebuild.rs` production **unchanged** (grep: T300 consts not referenced from `rebuild.rs`). `graph_density.rs` production **unchanged**. `doctor.rs` production **unchanged**. |
| **AC14** | Manual on **live** vault **via `cargo run -p ai-brains-cli --features graph`**. **Before:** `graph update --format human` still `sparse` with E/N printed (not a false `live`) + doctor `graph_density` warn. **`--dry-run`:** current density + `[dry-run]`; daemon Running → NOTICE. **Mutate:** **only if owner confirmed daemon stop.** After: `graph update --format human` and `doctor --summary` `graph_density` **agree**. Pass-with-observed-data (`sparse` or `live`). If owner skips mutate: record skip; AC1–AC13 still required. **Do not** start/stop daemon without owner. |
| **AC15** | `graph update --format human` stay-green labeled lines (no extra rebuild sentence on update). |
| **AC16** | Stay-green: T213 assessor sparse 1304/95; T232 graph-on remediator exact `ai-brains graph rebuild`; T246/T278/T293 neighbors; serde keys unit. |

---

## 5. Design notes

### 5.1 Human mutate success (may still be sparse)

```
status: sparse
density: warn
nodes: …
edges: …
pinned_memories: …
memory_nodes: …
edge_node_ratio: …
note: sparse: edge/node ratio below typed-lineage floor 0.5 (…); rebuild if projection lag suspected
remediation: ai-brains graph rebuild
```

Same shape as `graph update --format human`. Numbers are live-volatile.

### 5.2 Human dry-run (daemon up)

Health block (current snapshot) then:

```
NOTICE: live rebuild will fail while the daemon is running. Stop with `ai-brains daemon stop` or `sc stop AI-Brains-Daemon` before a real rebuild.
[dry-run] would DELETE graph_node/graph_edge then replay N events; no mutation.
```

Pin NOTICE to contain T188 substrings. Exact NOTICE text is implementer-owned as long as AC4 substrings hold.

### 5.3 Helper sketch (`graph.rs`)

```rust
pub(crate) fn rebuild_daemon_busy_message() -> String {
    "Cannot rebuild graph: daemon is running and holds the vault open. \
     Stop it first with `ai-brains daemon stop`, or if installed as a Windows \
     service: `sc stop AI-Brains-Daemon` (service hosts `ai-brainsd`)."
        .to_string()
}

pub async fn rebuild(
    ctx: &AppContext,
    dry_run: bool,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = crate::daemon_client::DaemonClient::new();
    let daemon_up = crate::commands::backup::probe_restore_daemon_busy(&client).await;
    rebuild_with_daemon_state(ctx, dry_run, format, daemon_up)
}
```

`rebuild_with_daemon_state`: if `!dry_run && daemon_up` → `Err(rebuild_daemon_busy_message())`. If dry-run: `graph_health_report` + emit + human extra lines. If mutate: `GraphRebuilder::rebuild()` then `graph_health_report` + emit.

### 5.4 Why not `--confirm`

Doctor and `graph update` print `ai-brains graph rebuild` (T232 F4). T232’s whole point was that remediator must be executable on this binary. `--confirm` would repeat the graph-off dead-end class.

### 5.5 Why not retune 0.50

Live coverage **0.59** already clears `MIN_MEMORY_COVERAGE`. E/N **0.15** is typed-lineage (IN_SESSION / RECALLS / SYNTHESIZED_FROM only). Rebuild replays the **same** projector — T262 already warned historical capture without `turn_id` still will not mint those memory nodes. Lowering the floor would make a still-sparse graph report `live` (T213 regression).

### 5.6 Why daemon Safety, not Status 1×300 ms

DELETE of graph tables is T188-class destructive. Weakening to Status would miss a slow Ping. Reuse `probe_restore_daemon_busy` as-is.

### 5.7 Why JSON dry-run omits `[dry-run]` on stdout

`--format json` must stay a single parseable object (T213 scripts). Human extra lines would break `ConvertFrom-Json`. Scripts that need “was this dry-run?” already passed `--dry-run`.

---

## 6. Non-goals

- Floor retune / env-default change / doctor 16th
- Cargo `default = ["graph"]` / GitHub Release graph-on
- Projector more-edges / fake `SYNTHESIZED_FROM` / WCC / 2-hop
- Streaming `read_all_events` / progress bar crate
- `--confirm` / `--force` override daemon gate
- Nightly auto-rebuild / schtasks mutate
- Growing `doctor.rs` / `graph_density.rs` / `rebuild.rs` / hotspots
- clap 5 / rusqlite 0.40 / workspace 0.1.3
- leftover `--write` / T240 F2 / T263 H2
- `cargo install` / silent `daemon stop`
- Changing `graph update` default JSON

---

## 7. Verification plan (TDD)

**Red first (must fail on current tree):**

1. `graph_rebuild__dry_run__prints_density_no_mutation` (AC1)
2. `graph_rebuild__mutate__prints_density_and_keeps_pin_node` (AC2)
3. `rebuild_with_daemon_state__daemon_up_mutate__err` (AC3)
4. `graph_rebuild__format_json__health_keys` (AC5) — helper missing / silent stdout fails parse

**Then green:** async wrap + `rebuild_with_daemon_state` + shared `graph_health_report` + clap flags + docs.

**Stay-green:** AC6 T262 / AC8 floors+15 / AC16 T213/T232/T246/T278/T293 / AC7 feature-off / AC9 clap / AC15 update human.

**Manual:** AC14 via **`cargo run -p ai-brains-cli --features graph`**. `--dry-run` always. Mutate **only** with owner confirm + daemon Stopped. Pass-with-observed-data. **Do not** live rebuild as a planning leftover.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Rebuild while daemon Running corrupts graph | F7 Safety fail-closed; AC3; Manual AC14 stop-before. |
| Floor lie (`live` while E/N 0.15) | F2 / F3; AC8; pass-with-observed-data. |
| `--confirm` dead-ends doctor | F8. |
| Dual-truth update vs rebuild JSON | F27 shared builder; AC2 status equality; AC5 keys. |
| T262 pin path broken by rebuild tests | AC6 stay-green; hermetic tempfile only. |
| `read_all_events` RAM on 147 MB vault | F9 freeze; after_help “minutes”; residual F25. |
| PATH 0.1.2 hides T300 UX | F17; hermetic + `cargo run --features graph` SoT. |
| Hotspot `project.rs` | Do not touch. |
| JSON dry-run extra lines | F10 human-only extras. |
| Owner refuses daemon stop | F1 skip residual; hermetic still DoD. |

---

## 9. Deferred absorb / decline

**Entire `conductor/deferred.md` scanned** (T142 archive through T299 closeout + T285–T300 mint). Overlapping open rows:

| Item | Disposition |
|------|-------------|
| Audit / mint “graph sparse E/N ~0.14 live rebuild” | **Absorb** F1–F8 / AC1–AC5 / AC14 |
| Placeholder Manual `graph update` + owner-confirm `graph rebuild` + doctor | **Absorb** AC14 / F1 / F3 |
| Placeholder floors frozen; never force `live` | **Absorb** F2 / F3 |
| Placeholder skip = hermetic T262 + written skip | **Absorb** F1 / F11 / AC6 |
| T278 F8 no live rebuild as DoD | **Lift to owner-confirm** F1 / F13 (T295 class) |
| T278 F7 / T213 floors | **Affirm freeze** F2 / AC8 |
| T232 remediator `ai-brains graph rebuild` | **Affirm** F8 / AC16 |
| T262 pin = node without rebuild | **Affirm** F11 / AC6 |
| T293 neighbors ranking | **Decline steal** — Completed `#209` |
| T188 daemon Safety for mutate | **Absorb pattern** F7 / AC3 |
| T295 live `--no-prune` analog | **Absorb class** F1 |
| T299 closeout “T300 graph sparse not stolen” | **Absorb** (this track) |
| T293/T278 closeout “Sparse E/N / live rebuild → T300” | **Absorb** |
| leftover `--write` / T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F14 / F24 |
| last-PR Cursor **#215** | **N/A empty** — **no T301** F18 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Not this track** — T258 / leftover data |
| Closed T213/T232/T262/T278/T293/T299 DoDs | **Stay closed** |
| recovery_kit_event doctor warn | **Not this track** |
| `read_all_events` RAM | **Decline as DoD** F9 / F25 residual |

---

## 10. Implement order (on go)

1. Phase 0 re-verify (plan.md) + FEATURE TX.
2. Red AC1 / AC2 / AC3 / AC5.
3. Green `rebuild_with_daemon_state` + shared health + clap + `.await`.
4. Stay-green AC6–AC9 / AC15 / AC16.
5. Docs AC12 / after_help AC11.
6. Manual AC14 `--dry-run`; mutate **only** with owner confirm.
7. `scripts/dev-check.ps1`; Phase-1 review; `codex-review`.
8. conductor Completed + deferred closeout + pin.
9. Phase 6 publish (`track/T300-*` → PR → watch GHA `CI` green → squash-merge). Never `git push origin main`.

---

## 11. Soft residuals

| Residual | Notes |
|----------|--------|
| PATH until `cargo install --features graph` | F17 — source/hermetic SoT |
| Live skip if owner refuses daemon stop | F1 — not a floor lie |
| `read_all_events` full Vec | F9 / F25 |
| `SessionSummaryCreated` nodes without edges | T278 F11 |
| Still-sparse after rebuild | F3 honest |
| Doctor `recovery_kit_event` warn | Unrelated |
| JSON dry-run has no `dry_run` key | F10 by design |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/graph.rs` | F26/F27 helpers; async `rebuild`; dry-run COUNT; emit health; units AC10 |
| `crates/ai-brains-cli/src/main.rs` | `GraphCommands::Rebuild { dry_run, format }` + variant `after_help`; dispatch `.await` |
| `crates/ai-brains-cli/tests/graph_live_projection.rs` **or** new `tests/graph_rebuild_ops.rs` | AC1–AC5 / AC7 / AC9 hermetics |
| `crates/ai-brains-cli/tests/exit_contract.rs` | AC7 feature-off rebuild |
| `Docs/CAPABILITIES.md` | Rebuild row **`:461`** additive |
| `Docs/OPERATIONS.md` | Graph health **`:927–949`** extend |
| `Docs/WORKFLOWS.md` | **`:215`** daemon-stop |
| `CHANGELOG.md` | T300 Unreleased |
| `Docs/CLI-EXIT-CODES.md` | Daemon-up **1**; still-sparse **0** |
| `Docs/PROTOCOL-COMPAT.md` | **`:96`** rebuild JSON = update keys |
| `conductor/conductor.md` / `deferred.md` / this spec+plan / README-T285-T300 | Planning now; Completed on go |

**Do not touch:** `rebuild.rs`; `graph_density.rs`; `doctor.rs`; `projector.rs`; `project.rs`; `ai-brains-contracts`; `Cargo.lock`; live vault rebuild/daemon as planning.

---

## 13. AI fold-in

Reserved for `/fold-in 300`. Do **not** edit `*-review.md`.
