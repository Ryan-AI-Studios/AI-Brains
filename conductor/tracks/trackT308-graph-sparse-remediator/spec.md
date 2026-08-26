# T308 — Sparse graph remediator honesty (no rebuild loop)

- **Track ID:** T308-GraphSparseRemediator
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE / CLI
- **Owner:** Grok
- **Source:** T300 live rebuild still `sparse`; T306 R4; leftover README P1. PATH doctor **2026-08-26**: nodes=**63040** edges=**25844** **E/N=0.410** pinned=49521 memory_nodes=39355; remediator still **`ai-brains graph rebuild`**. `graph update --format human` note already says `rebuild if projection lag suspected` **and** still prints the rebuild remediator.
- **Depends on:** T213 floors; T232 remediator exact `ai-brains graph rebuild` for **empty_lag / orphan / projection_lag**; T300 rebuild UX (owner-confirm live mutate, still sparse honest). **Floors frozen.**
- **Blocks / feeds:** Operators who already rebuilt stop copy-pasting DELETE+replay. Does **not** unblock T307 (Blocked), T309 `table_exists`, T310 `run_update` / PATH daemon. Capture path does not use density remediator copy.
- **Absorbs:** T306 R4 / leftover “T300 still sparse; doctor SOOT rebuild”; T300 closeout “still sparse honest.”
- **Not absorbed (DoD):** T278 / T300 floor retune (`MIN_EDGE_NODE_RATIO = 0.50`); projector more-edges / WCC; `GraphRebuilder` rewrite; auto-rebuild; T213 event↔graph freshness; T309 `has_graph_tables`; T310; clap 5; live rebuild as DoD.
- **Research date:** 2026-08-26 (plan wrote at `037262e` T307 `#224`; fold-in against `0d0fdab`, ahead **1** of `origin/main`).
- **AI fold-in:** 2026-08-26 `agy-review.md` + `opencode-review.md` (HEAD `0d0fdab`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** both m1 HEAD snapshot; OpenCode O1 PROTOCOL-COMPAT `:96` already optional (drop from stale-doc row). **Already:** Agy m2 AC8 docs; Agy O1 doctor.rs forward; Agy O2 smoke F17; Agy O3 / OpenCode O2 loop-stop. Disposition **§13**.
- **Ledger:** planning DOCS TX `96f0ce16-3a64-43cc-92ac-b9a4d89c46ae`. Fold-in DOCS TX `91f8fbcd-655e-4fbd-bd64-635e9fa271bf`. Series mint DOCS `c62396f6-4532-4335-b10b-f31b3fa02ec2`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** retune floors. Do **not** live `graph rebuild` / `daemon stop` / `cargo install` as planning. Do **not** grow `doctor.rs` (T300 freeze — it already forwards `assessment.remediation`). Do **not** rewrite `GraphRebuilder` / projector. Do **not** steal `has_graph_tables` (T309). Never `git push origin main`.

---

## 1. Objective

1. **Stop the rebuild loop.** After T300, a successful projection can still be **Sparse** (typed-lineage E/N below **0.50**). Doctor `graph_density` and `graph update` must **not** SOOT `ai-brains graph rebuild` for that verdict. Copy-pasting rebuild again DELETEs+replays the same projector and stays sparse (T300: E/N **0.149 → 0.407**, still warn).
2. **Keep rebuild for real lag.** Graph-on **empty_lag / orphan_nodes / projection_lag** still remediate exact `ai-brains graph rebuild` (T232 F4 remainder; no `--confirm`). Graph-off warn still remediates `GRAPH_REINSTALL_SOOT` only.
3. **Floors stay honest.** Do **not** retune `MIN_EDGE_NODE_RATIO`. Do **not** force `status: live`. Sparse stays `density: warn`.
4. **Capture independence.** Pure assessor text. No new events, no contracts DTO, no models, no graph crate rewrite.

This unblocks daily ops honesty: T213/T232 made density **honest**; T300 made rebuild **usable**. The remaining hole is the **Sparse remediator** still pointing at the command that already ran.

---

## 2. Live baseline (re-scan 2026-08-26)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Fold-in `0d0fdab` (T308 full-plan commit). Tree **CLEAN**. `origin/main...HEAD` **ahead 1**. Branch `main`. Plan-write snapshot was `037262e` / **0/0** (both-reviewer m1). |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` LastWriteTime **2026-08-26 6:54:32 AM**; `ai-brains 0.1.3`; T306 graph-on SQLCipher **4.14**. |
| PATH `doctor --format json` | `status=degraded`. `graph_density` **warn** `sparse: … floor 0.5 (nodes=63040 edges=25844 E/N=0.410 pinned=49521 memory_nodes=39355)` **`remediation: ai-brains graph rebuild`**. Other warn: `recovery_kit_event` (**not this track**). Matrix **15**. `cipher_page` `4.14.0 community`. `graph_feature=available`. `vault_open` opened read-only. |
| PATH `graph update --format human` | `status: sparse` `density: warn` nodes **63044** edges **25847** pinned **49522** memory_nodes **39357** `edge_node_ratio: 0.40998…` note **includes** `rebuild if projection lag suspected` **and** `remediation: ai-brains graph rebuild`. |
| Coverage | 39355/49521 ≈ **0.795** ≫ `MIN_MEMORY_COVERAGE` 0.10. Warn is **E/N**, not projection_lag. |
| rustc / cargo | **1.95.0** / workspace **0.1.3**. clap **4.5**; rusqlite exact **0.40.2** — **no bump**. |
| Last GitHub PR | [#224](https://github.com/Ryan-AI-Studios/AI-Brains/pull/224) T307 F3 halt (`mergedAt` **2026-08-26T14:16:50Z**; `createdAt` **13:59:03Z** — do **not** treat list time as merge). `pulls/224/comments`, `/reviews`, `issues/224/comments` all **`[]`**. Body has Bugbot **CURSOR_SUMMARY** (low-risk docs overview, no defect). **last-PR Cursor: N/A.** Open PRs: **none**. **No leftover from Cursor. No T311.** |
| Ledger | **0 pending / 0 drift** at scan (before this DOCS TX). |
| `ISSUES.md` | **Does not exist.** |
| Planning rebuild | **Not run.** Health-only dogfood. |

### 2.2 Why this residual still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Sparse + rebuild remediator | T300 proved typed-lineage sparse **survives** full replay. Doctor still copy-pastes DELETE+replay. Note already has lag nuance; **primary remediator** is the loop. **DoD.** |
| Floor retune 0.50 → live ~0.41 | T213 Adaptive GraphRAG: typed sparse can be healthy. T278 / T300 freeze. Coverage already **0.80**. Raising E/N needs projector more-edges. **Decline.** |
| Detect “already rebuilt” via freshness | T213 F31 event↔graph freshness. Would need event COUNT vs graph — different track. T308 is remediator copy, not a freshness arm. **Decline.** |
| Honesty-string remediator (not a command) | CLIG: next-action must be copy-pasteable **or omitted**. T267 `next_action: "none"` analog: omit when no command helps. **F2 = `None`.** |
| `graph rebuild --dry-run` as remediator | Inspect-only; `graph update` already is the check. Not a fix. **Decline.** |
| Auto-rebuild / nightly rebuild | Stop-Before. **Decline.** |
| Grow `doctor.rs` | T300 freeze; check already forwards `assessment.remediation`. **Decline.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Floors | `crates/ai-brains-cli/src/graph_density.rs` **`:10–16`** | `MIN_PINNED=100`; `MIN_NODES=50`; `MIN_EDGE_NODE_RATIO=0.50`; `MIN_MEMORY_COVERAGE=0.10`. Env `:18–21`. **Do not change.** |
| Rebuild SOOT | `REMEDIATION_REBUILD` **`:140`** | Exact `ai-brains graph rebuild`. T232 F4. **Keep const.** Sparse graph-on **must not use it.** |
| Capability helper | `density_remediation` **`:143–149`** | Graph-on → rebuild; graph-off → `GRAPH_REINSTALL_SOOT`. Still used by empty_lag / orphan / projection_lag **and** doctor gather-error. |
| Sparse arm | `assess_graph_density_with` **`:214–226`** | `sparse_nuance=true` note; **`remediation: Some(remediation.into())`** — **this is the hole.** Graph-on → `None`; graph-off → reinstall SOOT. |
| Priority | comment **`:165`** | empty_lag → orphan → sparse → projection_lag → skip → Ok. **Do not reorder.** |
| `has_graph_tables` | **`:281`** | sqlite_master. **T309 — do not steal.** |
| Doctor check | `doctor.rs` `check_graph_density` **`:868–918`** | Warn arms including Sparse pass `assessment.remediation`. Gather-error **`:892`** uses `density_remediation`. **Do not grow.** Matrix **15** (`:1066`). |
| Graph JSON | `graph.rs` `GraphHealthOutput` **`:35–48`** | `#[serde(skip_serializing_if = "Option::is_none")]` on `remediation`. Sparse omit is already valid shape. Human emitter **`:381–383`** already `if let Some` — **no production `graph.rs` edit** (test only). |
| Sparse JSON test | `graph.rs` **`:794–828`** | `graph_health_output__sparse_fixture__status_sparse_with_remediation` asserts rebuild. **Flip** to omit key. |
| Assessor units | `graph_density.rs` **`:458–528`** | `…sparse_1304_95_graph_on__rebuild` and `…ratio_0_4__warn_sparse` assert rebuild. **Flip.** Graph-off Sparse **`:475–490`** reinstall — **keep.** |
| Smoke F17 | `tests/smoke.rs` **`:3265–3340`** | Const equals exact rebuild; doctor.rs has **no** rebuild literal. **Stay-green.** |
| Contracts | `ai-brains-contracts/src/doctor.rs` `HealthCheck` **`:67–70`** | Optional remediator; `skip_serializing_if`. Schema v1. **No DTO change.** |
| Docs (stale) | `Docs/OPERATIONS.md` **`:918–923`**, **`:949–950`**, **`:1043`**; `Docs/CAPABILITIES.md` **`:557`** | “graph-on → rebuild if sparse/empty” / “rebuild when graph-on” is **stale** for Sparse. AC8. |
| Docs (already) | `Docs/PROTOCOL-COMPAT.md` **`:96`** | `remediation` already **optional**. Omit-on-None is in contract — **do not list as stale** (OpenCode O1 / Agy O3). |
| Hotspots | `project.rs` #1 | **Do not touch.** Expected product: `graph_density.rs` + `graph.rs` tests + docs + CHANGELOG. |

### 2.4 Dependency / standards research (2026-08-26) — snapshot, re-verify at execute

| Claim | Evidence |
|-------|----------|
| Crate pins | Workspace clap **`"4.5"`**; rusqlite exact **`0.40.2`**; tokio **`"1.53"`**. **No bump this track.** |
| clap 5 | Standing decline. Not this hole. |
| CLIG ([clig.dev](https://clig.dev/) § Suggest commands / saying just enough) | Suggest the **next** command when one exists. Do not invent a fake command. Omit when the recommended action already ran and cannot change the verdict. |
| Loop-stop | Addy Osmani *Practical Loop Engineering* (2026-08-14): same command with no change from the previous run → **stop**. Closed-loop SRE (zop.dev 2026): **no double-rollback** — if the action already applied and the metric is still off-target, looping the same action is the failure mode. |
| T213 KG health | Adaptive GraphRAG / Yu 2026: typed sparse ≠ unhealthy. Floor is product-local, not Erdős–Rényi. **Affirm freeze.** |
| T300 rebuild | Idempotent DELETE+replay of the **same** projector. Second rebuild cannot raise typed E/N to 0.50. |
| T267 analog | `harness status` wiring=ok → JSON `next_action: "none"` (omit command). Sparse graph-on uses the same pattern: **no remediator key.** |
| N/A | No new crate API. rusqlite `table_exists` is **T309**. |

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. No product commits as planning. |
| **F1** | Default floors **frozen**: `MIN_EDGE_NODE_RATIO = 0.50`, `MIN_PINNED = 100`, `MIN_NODES = 50`, `MIN_MEMORY_COVERAGE = 0.10`. Env overrides stay. Do **not** force `live`. |
| **F2** | Graph-on **Sparse**: `remediation = None` (JSON omits key; pretty omits the remediator line). Note **keeps** `rebuild if projection lag suspected`. Do **not** invent a non-command honesty string. Do **not** SOOT `graph update` or `graph rebuild --dry-run`. Pretty omit is **already** `emit_graph_health_human` `if let Some` (`graph.rs:381–383`) — no emitter rewrite. |
| **F3** | Graph-on **empty_lag / orphan_nodes / projection_lag**: still exact `REMEDIATION_REBUILD` (`ai-brains graph rebuild`, no `--confirm`). T232 F4 remainder. |
| **F4** | Graph-off **any** density warn (including Sparse): still `GRAPH_REINSTALL_SOOT` only. Install is a real next step; rebuild is a dead-end on graph-off. |
| **F5** | `REMEDIATION_REBUILD` const **stays**. Sparse graph-on simply does not use it. Smoke F17 stay-green. |
| **F6** | JSON **keys** frozen (T213 / PROTOCOL-COMPAT `:96`). Sparse graph-on changes remediator **presence** (already optional). Doctor `HealthCheck` schema_version **1**. |
| **F7** | Do **not** grow `doctor.rs`. Check already forwards `assessment.remediation`. Gather-error still `density_remediation(cfg!(feature = "graph"))`. |
| **F8** | Do **not** rewrite `GraphRebuilder` / `GraphProjector` / LiveGraphHook. No auto-rebuild. No nightly rebuild. |
| **F9** | Do **not** steal `has_graph_tables` / `has_core_tables` (**T309**). |
| **F10** | Do **not** steal T310 (`run_update` / PATH daemon). T307 stays **Blocked**. |
| **F11** | No clap 5; no rusqlite/tokio/reqwest bump; no new crates. |
| **F12** | No live `graph rebuild` / `daemon stop` / `cargo install` as DoD. Optional live `doctor --format json` **read-only** after go (hermetic units are SoT if owner skips live). |
| **F13** | Doctor matrix **15** frozen. Do not add a 16th check. |
| **F14** | T232 F4 is **carved** for Sparse only — not reopened for other warn arms. |
| **F15** | Missing-node pretty `next: ai-brains graph rebuild` (T246 / T262) is **not** this hole. |
| **F16** | Never `git push origin main`. No Dependabot remote merge. |
| **F17** | Capture-independent: rusqlite COUNT + assessor text only. |
| **F18** | last-PR Cursor `#224` **N/A empty** — no T311. |
| **F19** | PowerShell `;` not `&&`. No `unwrap`/`expect`/`panic` in production. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Graph-on Sparse fixture (existing `snap(1304, 95, …)` and/or `snap(100, 40, …)`): `verdict == Sparse`, `density == "warn"`, `status == "sparse"`, **`remediation.is_none()`**, note contains `rebuild if projection lag suspected`, remediator ≠ `ai-brains graph rebuild`. |
| **AC2** | Graph-on empty_lag / orphan / projection_lag still `Some(REMEDIATION_REBUILD)`. Existing unit names stay green (`…empty_lag_graph_on__rebuild_only`, `…orphan_graph_on__rebuild`, `…projection_lag_graph_on__rebuild`). |
| **AC3** | Graph-off Sparse still `Some(GRAPH_REINSTALL_SOOT)`; note does **not** use the lag-nuance suffix. |
| **AC4** | `graph_health_output__sparse_fixture__status_sparse_*`: JSON `status=sparse` `density=warn`; **`remediation` key absent**. Required keys (`nodes`…`note`) still present. |
| **AC5** | `MIN_EDGE_NODE_RATIO` still **0.50** in src (`:14`). Smoke F17 still holds (`REMEDIATION_REBUILD` const + doctor.rs has no rebuild literal). |
| **AC6** | Doctor matrix still **15**; `check_graph_density` still forwards assessor remediator (no new doctor.rs logic unless a compile force — unexpected). |
| **AC7** | clippy `-p ai-brains-cli --all-targets -- -D warnings` + nextest units covering `graph_density` / `graph_health_output` / smoke SOOT. |
| **AC8** | CHANGELOG Unreleased Changed. OPERATIONS / CAPABILITIES T232 “rebuild if sparse” sentences updated: Sparse graph-on **omits** remediator; empty/orphan/projection_lag still rebuild. **Not** PROTOCOL-COMPAT `:96` (already optional). |
| **AC9** | Optional live: PATH or `cargo run --features graph -- doctor --format json` on this vault — Sparse check has **no** `remediation` key (or value ≠ rebuild). Skip with written reason if owner declines live. **No rebuild.** |
| **AC10** | `has_graph_tables` sqlite_master **unchanged**. T309 not stolen. |

TDD: **red** AC1/AC4 test flips first, then green Sparse arm.

---

## 5. Design notes

### 5.1 Why `None` not a new SOOT

A remediator that is not a copy-pasteable command trains agents to run garbage. A remediator that is `graph rebuild` after T300 is a **mutate loop**. The lag hint already lives in **note**. Omit the command.

### 5.2 Sparse vs never-rebuilt

Priority already routes true empty graphs to **empty_lag** and zero-edge graphs to **orphan** (both keep rebuild). Sparse means **some** edges exist and E/N is still below 0.50 — the T300 class (typed-lineage under-link). Operators who suspect lag still have the note. Detecting “never rebuilt” needs freshness (declined).

### 5.3 Assessor change (sketch)

In the Sparse arm only (`:214–226`):

- Graph-on: `remediation: None` (do **not** call `density_remediation(true)`).
- Graph-off: `remediation: Some(GRAPH_REINSTALL_SOOT.into())` (same as today via helper).
- `density_warn_note(..., sparse_nuance=true)` **unchanged**.

Other arms keep `Some(density_remediation(graph_cli_available).into())`.

Doctor `check_graph_density` (`:914`) and `GraphHealthOutput` serde already forward/omit `None`. Production touch is the Sparse arm + unit/JSON test flips.

### 5.4 Capture independence

No `ai-brains-graph` in the doctor path. No new events. Rebuild engine untouched.

---

## 6. Non-goals

- Floor retune / forcing `live` / WCC / fake edges
- Projector rewrite / streaming `read_all_events`
- Auto-rebuild / `--confirm` on rebuild
- Event↔graph freshness arm
- Growing `doctor.rs` / 16th check
- T309 `table_exists`; T310 `run_update` / daemon 4.14; T307 dual tower-http
- clap 5; csrf; `[patch.crates-io]`
- Live rebuild / daemon stop / cargo install as DoD
- Changing missing-node pretty remediator
- `recovery_kit_event` doctor warn

---

## 7. Verification plan

TDD — failing tests first:

```powershell
# Red: flip Sparse remediator asserts
cargo nextest run -p ai-brains-cli --lib assess_graph_density_with__sparse
cargo nextest run -p ai-brains-cli --lib assess_graph_density_with__ratio_0_4
cargo nextest run -p ai-brains-cli --lib graph_health_output__sparse_fixture
# Green after Sparse arm None
cargo clippy -p ai-brains-cli --all-targets -- -D warnings
cargo nextest run -p ai-brains-cli --lib graph_density
cargo nextest run -p ai-brains-cli --lib graph_health_output
cargo nextest run -p ai-brains-cli graph_stub__reinstall_hint__matches_install_soot
# Stay-green lag arms
cargo nextest run -p ai-brains-cli --lib assess_graph_density_with__empty_lag
cargo nextest run -p ai-brains-cli --lib assess_graph_density_with__orphan
cargo nextest run -p ai-brains-cli --lib assess_graph_density_with__projection_lag
```

Named tests to **change**:

- `assess_graph_density_with__sparse_1304_95_graph_on__rebuild` → `…__no_rebuild_remediator`
- `assess_graph_density_with__ratio_0_4__warn_sparse` (remediator None)
- `graph_health_output__sparse_fixture__status_sparse_with_remediation` → `…__omits_remediation`

Named tests to **keep**:

- `assess_graph_density_with__empty_lag_graph_on__rebuild_only`
- `assess_graph_density_with__orphan_graph_on__rebuild`
- `assess_graph_density_with__projection_lag_graph_on__rebuild`
- `assess_graph_density_with__sparse_1304_95_graph_off__reinstall_soot`
- `graph_stub__reinstall_hint__matches_install_soot` (F17)

Optional live (AC9): `ai-brains doctor --format json` — Sparse object has no `remediation`. **No rebuild.**

Full workspace gate on implement closeout (implement-track), not as a plan gate.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Never-rebuilt Sparse loses copy-paste rebuild | **F2/§5.2** — empty_lag/orphan still rebuild; note keeps lag hint |
| Accidental floor retune | **F1 / AC5** assert `0.50` |
| Doctor.rs rewrite / 16th check | **F7 / F13 / AC6** |
| Graph-off Sparse loses reinstall | **F4 / AC3** |
| T232 SOOT const deleted | **F5 / AC5** smoke F17 |
| Stealing T309 `table_exists` | **F9 / AC10** |
| Live rebuild as “proof” | **F12** health-only |
| Fake Complete while remediator still rebuild | **AC1/AC4/AC9** |
| JSON key-set break | **F6** omit only optional remediator |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-26.

| Item | Disposition |
|------|-------------|
| T306 R4 `graph_density` sparse E/N≈0.409; remediator rebuild | **Absorb** F2 / AC1 / AC4 / AC9 |
| T300 closeout still sparse after live rebuild | **Absorb** — this hole |
| T300 leftover “still sparse after rebuild” (mint row) | **Absorb** |
| T278 / T300 floor retune `MIN_EDGE_NODE_RATIO=0.50` | **Decline** F1 — remediator copy, not 0.50 |
| T213 L4 / T305 R2 `table_exists` | **Decline steal → T309** |
| T306 R2 PATH `ai-brainsd` 4.10; T306 R3 T84 `run_update` | **Decline steal → T310** |
| T307 dual tower-http / F3 Blocked | **Not stolen** — Blocked |
| T306 R5 `recovery_kit_event` | **Not this track** |
| T306 R6 INSTALL.md 0.1.2 header | **Decline** — docs drift |
| T213 F31 event↔graph freshness | **Decline** — different arm |
| clap 5 | **Decline** F11 |
| T304 csrf / `[patch]` / git reqwest | **Decline** — T307 |
| last-PR Cursor `#224` | **N/A empty** — comments/reviews/issue comments `[]`. Bugbot CURSOR_SUMMARY is upsell/overview, not a defect. **No T311.** |
| T240 F2 / leftover `--write` / T263 H2 | **Decline** — standing |

---

## 10. Implement order (on go)

1. Phase 0: re-read Sparse arm `:214–226`; confirm floors still 0.50; rescan deferred; FEATURE TX. No live rebuild.
2. Red: flip AC1/AC4 tests (Sparse remediator None; JSON omits key).
3. Green: Sparse arm graph-on `None`; graph-off still reinstall; other warn arms unchanged.
4. Stay-green AC2/AC3/AC5/AC6/AC7/AC10.
5. Docs AC8 (OPERATIONS/CAPABILITIES/CHANGELOG). Optional AC9 live doctor JSON.
6. Conductor **Completed**. Phase 6: `track/T308-*` → PR → watch `CI` → squash-merge. Never `git push origin main`.

---

## 11. Soft residuals (post-close)

| Residual | Note |
|----------|------|
| Live E/N still ~0.41 after T300 | **Expected** honest sparse; floors frozen |
| Never-rebuilt Sparse has no rebuild remediator | **By design** F2 / note; empty_lag/orphan still rebuild |
| `recovery_kit_event` doctor warn | Not this track |
| PATH until `cargo install` after merge | Soft — source/hermetic SoT; T306 already installed 0.1.3 |
| T309 / T310 | Not stolen |
| T307 Blocked | Not stolen |

---

## 12. Touch map

| Path | Role |
|------|------|
| `crates/ai-brains-cli/src/graph_density.rs` | Sparse arm remediator; unit flips |
| `crates/ai-brains-cli/src/commands/graph.rs` | Sparse JSON test omit remediator (**`:794–828`**). Production emitter **`:381–383`** **no edit**. |
| `crates/ai-brains-cli/src/commands/doctor.rs` | **No edit** (forward already) |
| `crates/ai-brains-cli/tests/smoke.rs` | Stay-green F17 (no edit expected) |
| `Docs/OPERATIONS.md` | When-to-rebuild table; doctor comment; Graph Health row |
| `Docs/CAPABILITIES.md` | `graph_density` remediator sentence |
| `CHANGELOG.md` | Unreleased Changed |
| `conductor/conductor.md` | Pending → Completed on go |
| `conductor/deferred.md` | T306 R4 done on go |
| `has_graph_tables` / `GraphRebuilder` / floors | **No** |

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` + `opencode-review.md` (HEAD `0d0fdab`). Fold-in verify: Sparse arm `graph_density.rs:223` still `Some(remediation.into())`; floors `:14` **0.50**; `GraphHealthOutput` skip_serializing_if `:46–47`; human emitter `:381–383` `if let Some`; doctor `:914` forwards remediator; PROTOCOL-COMPAT `:96` already “optional `remediation`”; OPERATIONS `:918–923` / `:949–950` / `:1043` and CAPABILITIES `:557` still “rebuild if sparse”; smoke F17 `:3265–3340`; `#224` comments/reviews/issues `[]`; HEAD `0d0fdab` ahead **1**.

### Pins locked by fold-in

1. **§2.1 HEAD (both m1):** review-time HEAD is `0d0fdab` / ahead **1**. Plan-write was `037262e` / 0/0. Phase 0 re-verifies the working tree. Fold-in commit follows this snapshot.
2. **PROTOCOL-COMPAT `:96` (OpenCode O1 / Agy O3):** already optional remediator. **Not** a stale AC8 target. Omit-on-None is in contract.
3. **Emitter (OpenCode solid #3):** `graph.rs:381–383` already omits the remediator line when `None`. F2 pretty-omit needs **no** production `graph.rs` edit (test flip only).
4. **last-PR Cursor `#224`:** N/A empty; **no T311.**

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** stale HEAD `037262e` / 0/0 | **Folded** §2.1 + plan preflight → `0d0fdab` / ahead **1** |
| Agy | **m2** OPERATIONS + CAPABILITIES + CHANGELOG sync | **Already** AC8 / §12 |
| Agy | **O1** doctor.rs forwards `None` / skip_serializing_if | **Already** F7 / F6 |
| Agy | **O2** smoke F17 stay-green | **Already** F5 / AC5 |
| Agy | **O3** PROTOCOL-COMPAT optional | **Folded** with OpenCode O1 — drop from stale-doc row |
| OpenCode | B / M | None filed |
| OpenCode | **m1** stale HEAD | **Folded** (same as Agy m1) |
| OpenCode | **O1** PROTOCOL-COMPAT `:96` not stale | **Folded** §2.3 docs-already row / pin 2 |
| OpenCode | **O2** loop-stop Osmani corroboration | **Already** §2.4 |
| both | last-PR Cursor empty; deferred map; no T311 | **Affirm** |
| both | Sparse hole is `:223`; floors frozen; no doctor.rs growth | **Already** F1 / F2 / F7 |

No Blockers/Majors to decline. No new placeholder. Do **not** edit `*-review.md`. Do **not** execute until go.

**Planning + fold-in 2026-08-26.** Still **plan-only until go**.
