# T232 — Graph density remediation path

- **Track ID:** T232-GraphDensityRemediation
- **Phase:** T217–T232 post-audit CLI quality (P2 honesty)
- **Status:** 📋 **Planning** (plan-only until **go**)
- **Depends on:** **T213** density assessor + doctor `graph_density`; **T222** doctor `graph_feature` + `GRAPH_REINSTALL_SOOT` (PR #122 `c1ac594`)
- **Blocks / feeds:** None hard; improves agent/operator next-action honesty after T222 install path
- **Category:** FEATURE / UX (doctor + assessor text only)
- **Source:** CLI audit 2026-08-05 — doctor says `graph rebuild` on graph-off binary → dead-end; T222 F11 handoff; deferred.md “Doctor graph rebuild vs graph-off”
- **Deferred absorbed:** deferred.md T232 placeholder; T222 soft residual “density remediation branching”; T213 soft “skill one-liner for density / rebuild” (**soft** docs/skill only)
- **Not absorbed:** Auto `graph rebuild`; density threshold retune (T213 floors stay); Cargo default / release graph-on; rusqlite 0.40 / clap bumps; event↔graph freshness (T213 F31 soft); projector edge rewrite; MSI/binstall
- **Research date:** 2026-08-10 (live dogfood post-T222 + T213/T222 re-read + Cargo features docs + crates.io pins)
- **AI fold-in:** 2026-08-10 — AI1 **H1 hard** + **M1–M5 hard**; **L1–L4 elevated** (notes/prefs); **L5–L6 affirm**; **O1/O4 hard**; **O2 soft**; **O3 preferred**. AI2 affirms F1–F28 design + remediation matrix (reject AI2 Skip `status=empty` cell — T213 Skip stays `status=live`). Disposition **§11**.
- **Ledger on go:** `ledgerful ledger start T232-graph-density-remediation --category FEATURE --message "Capability-aware graph_density remediations: rebuild vs GRAPH_REINSTALL_SOOT"`

## 1. Objective

1. **Remediation matches binary capability.** When `graph_density` (or `graph update`) warns, the primary remediation must be an **executable next step on this binary**:
   - **Graph-on:** `ai-brains graph rebuild`
   - **Graph-off:** `GRAPH_REINSTALL_SOOT` only (not rebuild → `FEATURE_UNAVAILABLE` dead-end)
2. **Preserve T213 honesty.** Thresholds, verdict priority, SQL-only gather, soft check severity, capture independence, and no auto-rebuild stay frozen.
3. **Consume T222 signal.** Use the same compile-time truth as `graph_feature` (`cfg!(feature = "graph")`) via a pure capability parameter for testability.
4. **Single SOOT for reinstall.** Never invent a fourth reinstall string — only `governed_common::GRAPH_REINSTALL_SOOT`.
5. **Zero new crates; no dep bumps.**

## 2. Live baseline (re-scan 2026-08-10, post-T222)

### 2.1 Dogfood (this machine)

| Probe | Result |
|-------|--------|
| `ai-brains doctor` `graph_feature` | **ok** / `available` (T222 PATH scripts worked) |
| `ai-brains doctor` `graph_density` | **warn** sparse `nodes=1334 edges=115 E/N=0.086 pinned=16795 memory_nodes=862` |
| Density remediation (live) | **`ai-brains graph rebuild`** (correct **on this** graph-on binary) |
| `ai-brains graph update` | JSON sparse + same rebuild remediation (works — feature present) |
| Feature-off still exists | Cargo `default = []`; GitHub Release graph-off; bare `cargo build -p ai-brains-cli` graph-off |
| Doctor matrix | **13** checks; order … `graph_feature`, `graph_density`, `harness_wiring`, `integrity` |

### 2.2 Root cause (honesty)

| Layer | Reality | Gap |
|-------|---------|-----|
| Assessor remediations | Orphan / sparse / projection_lag → always `REMEDIATION_REBUILD` | Graph-off doctor still prints rebuild (dead-end) |
| Empty-lag remediation | Hybrid: rebuild + parenthetical SOOT | Graph-off still leads with rebuild; graph-on still carries “if unavailable” noise |
| Doctor gather-error path | Hardcoded `"ai-brains graph rebuild"` | Same dead-end on graph-off |
| Notes | “run graph rebuild” | Capability-blind (secondary; pin for warn paths) |
| `graph_feature` (T222) | Correct available\|unavailable + SOOT when off | Not yet consumed by density remediations |
| Thresholds / verdicts | T213 pure assessor correct | **Do not** change floors or priority |

### 2.3 Code / file touch map

| Path | Role in T232 |
|------|----------------|
| `crates/ai-brains-cli/src/graph_density.rs` | **Hard:** capability-aware remediation (+ notes on warn paths); pure API for tests |
| `crates/ai-brains-cli/src/commands/doctor.rs` | **Hard:** gather-error remediation uses same SOOT helper; no hardcoded rebuild |
| `crates/ai-brains-cli/src/commands/graph.rs` | Soft/pass-through — already uses `assess_graph_density`; graph-on only → rebuild |
| `crates/ai-brains-cli/src/commands/governed_common.rs` | **No new constant** unless extracting rebuild SOOT for symmetry (optional `GRAPH_REBUILD_SOOT`) |
| Units in `graph_density.rs` | Dual capability matrix for empty/orphan/sparse/projection_lag |
| `Docs/OPERATIONS.md`, `Docs/CAPABILITIES.md` | Capability-aware remediation sentence |
| `CHANGELOG.md` | T232 entry |
| Soft: skill one-liner | `.agents/skills/ai-brains/SKILL.md` (+ onboarding graph section if present) |
| `conductor/*` | Status / deferred strike |
| Thresholds / gather SQL / matrix order | **No** |

### 2.4 Dep pins (research 2026-08-10)

| Item | Pin / note |
|------|------------|
| Rust | Workspace **1.95.0** (`rust-toolchain.toml`) |
| clap | Workspace **4.5** (crates.io max **4.6.6**) — **no bump** |
| rusqlite | Workspace **0.39.0** SQLCipher (crates.io max **0.40.2**) — **deferred** (T222 M6; no `table_exists` adoption this track) |
| thiserror | crates.io **2.0.20** — not touched |
| Cargo features | [Cargo Book](https://doc.rust-lang.org/cargo/reference/features.html): `cfg!(feature = "…")` is the compile-time capability probe; features stay additive |
| Zero new crates | Required |

## 3. Research summary

| Finding | T232 application |
|---------|------------------|
| T222 shipped `graph_feature` + `GRAPH_REINSTALL_SOOT` | Density remediations **consume** both; no parallel strings |
| Live PATH is graph-on after T222 | Graph-on sparse → rebuild is still correct; track proves **graph-off** path via units + optional feature-off hermetic |
| T213 pure assessor + single SOOT for thresholds | Keep threshold purity; branch **only** remediation/note text on capability |
| Hybrid empty_lag was a half-measure | Replace with **capability-primary** remediations (no dual primary on either path) |
| Cargo features additive / cfg | `cfg!(feature = "graph")` is authoritative for this binary; pass as `bool` for pure tests |
| Doctor must never auto-remediate | Text only — RELEASE-CHECKLIST / SECURITY-LIMITS forbid auto doctor fixes |

## 4. Frozen decisions (F1–F28)

| ID | Decision |
|----|----------|
| **F1 — Problem pin** | Graph-off `graph_density` warn must not remediate with `ai-brains graph rebuild` as the primary (dead-end → exit 2 + FEATURE_UNAVAILABLE). |
| **F2 — Capability source** | Capability = compile-time graph CLI presence, same truth as `graph_feature`: `cfg!(feature = "graph")`. **Not** runtime probe of PATH, not doctor check re-query, not vault state. |
| **F3 — Pure API (testability) + name** | `assess_graph_density_with(snap, graph_cli_available: bool) -> Assessment` is the pure SOOT for thresholds **and** remediation/note text. Public convenience `assess_graph_density(snap)` calls `_with(…, cfg!(feature = "graph"))` — **production callers only**. **Name `_with` is deliberate** (L1: “assess with this capability”); not `_for`/`_using`. Units that assert remediation/note **must** call `_with` with an explicit bool — never rely on the wrapper in graph-off CI (H1). |
| **F4 — Graph-on remediation SOOT** | For warn verdicts (EmptyLag, OrphanNodes, Sparse, ProjectionLag): remediation **exact** `ai-brains graph rebuild`. **`pub(crate) const REMEDIATION_REBUILD: &str = "ai-brains graph rebuild"`** (M2 hard — not optional private). Tests use `assert_eq!(a.remediation.as_deref(), Some(REMEDIATION_REBUILD))` (O1). **No** reinstall parenthetical on graph-on. |
| **F5 — Graph-off remediation SOOT** | For same warn verdicts: remediation **exact** `GRAPH_REINSTALL_SOOT` (T222 F27). Tests: `assert_eq!(…, Some(GRAPH_REINSTALL_SOOT))`. **No** leading rebuild in `remediation`. Secondary guidance lives in **note** only (F7). |
| **F6 — Empty-lag special case retired** | Drop hybrid `"rebuild (if graph CLI unavailable: SOOT)"` / `remediation_empty_lag()`. Empty-lag uses F4/F5 like other warn arms (capability-primary). |
| **F7 — Notes capability-aware (pinned wording)** | **Warn paths (hard):** branch note suffix on capability. Prefer single helper `density_warn_note(message, graph_cli_available) -> String` (O3 preferred). **Pinned templates:** Graph-on EmptyLag/Orphan/ProjectionLag: `"{message}; run graph rebuild"`. Graph-on Sparse: `"{message}; rebuild if projection lag suspected"` (keep T213 sparse nuance). **All graph-off warn verdicts (uniform, M5):** `"{message}; see remediation to install a graph-capable binary"`. **Ok-path note (M1 hard pin):** remains capability-blind informational — `"… use 'graph rebuild' for full resync."` **Rationale:** Ok severity has **no remediation** and no required operator action; graph-off Ok is rare (small vault or healthy density on slim install). Do **not** put rebuild in Ok `remediation`. Skip notes unchanged. |
| **F8 — Doctor gather-error path (M3 hard)** | `check_graph_density` COUNT failure **must** call **`pub(crate) fn density_remediation(graph_cli_available: bool) -> &'static str`** from `graph_density` (alias/name of `remediation_for` — prefer return **`&'static str`**, L2). Not optional; not inline `if cfg!` duplicated in doctor. Pass `cfg!(feature = "graph")`. Doctor module is always-compiled — cfg resolves at compile time per feature build (correct). |
| **F9 — Thresholds / priority frozen (T213)** | `MIN_PINNED=100`, `MIN_NODES=50`, `MIN_EDGE_NODE_RATIO=0.50`, `MIN_MEMORY_COVERAGE=0.10`; priority empty_lag → orphan → sparse → projection_lag → skip → ok. Env overrides unchanged. |
| **F10 — Soft check policy frozen** | `graph_density` never alone forces `fail`; warn → overall degraded; skip when open failed / tables missing / pinned count failed / small empty. Severity policy unchanged. |
| **F11 — Capture independence** | No new dep on `ai-brains-graph` for density; gather stays rusqlite COUNT; feature-off doctor still emits `graph_density`. |
| **F12 — No auto rebuild** | Text remediation only. Never call `GraphRebuilder` from doctor or assessor. |
| **F13 — graph update surface** | Graph-on only (`#[cfg(feature = "graph")]`). Remains rebuild remediation via F3 default. PROTOCOL-COMPAT field set unchanged (still optional `remediation` string). |
| **F14 — Contracts** | No new DTO / schema_version change. `HealthCheck.remediation` string content only. |
| **F15 — Doctor matrix** | Still **13** checks; order unchanged (T222 F10). No new check name. |
| **F16 — Zero new crates / no dep bumps** | clap 4.6.6 / rusqlite 0.40.2 available — **defer**. |
| **F17 — SOOT discipline + smoke guard (M4 hard)** | Reinstall string only via `GRAPH_REINSTALL_SOOT`. Rebuild string only via `REMEDIATION_REBUILD`. High finding if fourth divergent reinstall wording or reintroduced hard-coded rebuild in doctor. **Extend smoke** (or sibling guard): (1) `graph_density.rs` references `GRAPH_REINSTALL_SOOT` by name (no reinstall literal); (2) `doctor.rs` has **no** hardcoded `"ai-brains graph rebuild"` literal; (3) `REMEDIATION_REBUILD` value equals exact rebuild SOOT. Existing stub guard (2 feature-off Graph stubs) stays. |
| **F18 — Claims honesty** | Do not claim density warn “always fixable by rebuild.” Document capability-aware next action. Do not claim auto-remediation. |
| **F19 — Soft residuals** | Skill one-liner (absorb soft); event freshness F31; threshold CLI flags; rusqlite `table_exists`; release/Cargo default; auto rebuild; two-tier coverage. |
| **F20 — Series order** | After T222 close. Peer polish T223/T225+ may run in parallel (low file conflict if they avoid `graph_density.rs`). |
| **F21 — Plan-only** | No production code until user **go**. |
| **F22 — Ledger** | On go: `ledgerful ledger start T232-graph-density-remediation --category FEATURE`. |
| **F23 — Review** | Primary FEATURE/UX. Cross-model **optional** (text-only, no Cargo default/security surface); run if remediation SOOT regresses feature-off stubs. |
| **F24 — Implement order (H1 hard in Phase 1)** | (0) Preflight + ledger. (1) **Red + migrate:** convert **all** existing `assess_graph_density(&snap)` tests that assert remediation/note to `assess_graph_density_with(&snap, true\|false)`; split empty_lag / orphan / sparse (and projection_lag if asserting rem) into dual capability cases; use `assert_eq!` vs consts (O1); migrate `graph.rs` `graph_health_output__sparse_fixture` to `_with(…, true)` (L3). (2) Green: `_with` + `density_remediation` + notes F7 + empty-lag F6. (3) Doctor gather-error → helper (F8). (4) Docs AC10/O4 + soft skill. (5) Smoke F17 extension. (6) Manual dual build + full gate. |
| **F25 — Determinism** | Capability is an explicit parameter in tests; production wrapper uses compile-time cfg; no clocks in remediation strings. |
| **F26 — Feature-off proof** | Units with `graph_cli_available=false` assert `assert_eq!(remediation, Some(GRAPH_REINSTALL_SOOT))` and note suffix F7. Soft: optional `#[cfg(not(feature = "graph"))]` doctor gather-error integration (O2 — not hard DoD if pure helper unit covers AC7). |
| **F27 — Feature-on proof** | Units with `true` assert `assert_eq!(…, Some(REMEDIATION_REBUILD))`; empty-lag must **not** contain SOOT. Live dogfood: sparse → rebuild (this machine graph-on). |
| **F28 — High findings if…** | Graph-off warn still remediates rebuild-only; graph-on warn remediates reinstall-only; **existing wrapper tests left asserting rebuild in graph-off CI (H1)**; thresholds changed; auto rebuild; matrix order broken; fourth SOOT string / doctor rebuild literal reintroduced; capture→graph edge; doctor hard-fail alone from density. |
| **F29 — AI1 H1 test migration** | CI default workspace nextest is **graph-off** (`ci.yml` no `--features graph`); graph-on job filters `test(graph)` only. Any test calling the convenience wrapper and asserting rebuild **fails** graph-off CI after F3. Migrate per F24 Phase 1 **before** green merges. |
| **F30 — AI1 M1–M5 / O1 / O4** | Folded hard into F3–F8, F17, F24, F7 note templates, AC10/AC17–AC19. |
| **F31 — AI2 affirm + reject** | Affirm remediation matrix EmptyLag/Orphan/Sparse/ProjectionLag/GatherError × on/off; Ok/Skip → None. **Reject** AI2 table cell Skip `status=empty` — T213 Skip remains `status=live` with `density=skip` (F9 frozen). |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| Doctor graph rebuild vs graph-off | **Absorb** (hard DoD) |
| T222 density remediation handoff | **Absorb** |
| T213 skill one-liner density/rebuild | **Soft absorb** (skill + OPERATIONS sentence) |
| Hybrid empty-lag half-measure | **Absorb** (replace with F4/F5) |
| Density floors / env thresholds | **Keep** (T213) |
| Auto rebuild / Cargo default / release graph-on | **Out** |
| rusqlite 0.40 / clap bump | **Out** (defer) |
| Event freshness F31 | Soft residual (unchanged) |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Graph-on sparse/orphan/projection_lag: remediation exact rebuild SOOT | Unit `_with(…, true)` |
| **AC2** | Graph-off sparse/orphan/projection_lag: remediation exact `GRAPH_REINSTALL_SOOT`; does **not** equal rebuild-only | Unit `_with(…, false)` |
| **AC3** | Graph-on empty_lag: remediation = rebuild only (no SOOT parenthetical required) | Unit |
| **AC4** | Graph-off empty_lag: remediation = `GRAPH_REINSTALL_SOOT` only | Unit |
| **AC5** | Skip/Ok: remediation `None` (both capability sides) | Unit |
| **AC6** | Verdicts, density, status, thresholds, priority unchanged vs T213 fixtures (Skip remains `status=live`) | Existing units pass after remediation/note-only edits |
| **AC7** | Doctor gather-error remediation uses `density_remediation(cfg!(…))` (same SOOT helper) | Diff + unit preferred; soft O2 cfg-not-graph integration |
| **AC8** | Feature-off binary: doctor still emits `graph_density` (skip or real); no graph crate link | Existing regression |
| **AC9** | Feature-off `graph *` still exit 2 + FEATURE_UNAVAILABLE | Existing smoke |
| **AC10** | **Docs (O4/L4 hard):** `Docs/OPERATIONS.md` density section **~717–736** — “When to rebuild” capability-aware (if `graph_feature=unavailable`, reinstall SOOT first); retire line **736** hybrid “Empty-lag remediation may mention a graph-on reinstall”; CAPABILITIES capability-aware `graph_density` sentence; CHANGELOG T232 | Diff |
| **AC11** | Soft: skill one-liner — rebuild only on graph-capable binary; else reinstall SOOT / doctor `graph_feature` | Diff (non-blocking if deferred with note) |
| **AC12** | Full gate green (**includes graph-off workspace nextest** — H1) | Process |
| **AC13** | Manual: graph-on vault sparse → remediation rebuild; optional feature-off build doctor sparse/empty → reinstall SOOT | Manual evidence |
| **AC14** | No clap/rusqlite lockfile bump; zero new crates | Diff + lock |
| **AC15** | Claims: no auto-remediation; no “always rebuild” | Review |
| **AC16** | `GRAPH_REINSTALL_SOOT` remains single reinstall SOOT (stubs + graph_feature + density graph-off) | Smoke / grep |
| **AC17** | Existing remediation-asserting units call `_with(…, true\|false)` — no wrapper tests left that assume rebuild under graph-off CI (H1) | Diff + graph-off nextest |
| **AC18** | Graph-off warn notes match F7 uniform suffix; graph-on notes match F7 templates (M5) | Unit |
| **AC19** | Smoke guard covers F17 (graph_density SOOT by name; doctor no rebuild literal; `REMEDIATION_REBUILD` value) (M4) | Smoke |

## 7. Non-goals

- Auto `graph rebuild` from doctor or update  
- Changing density floors, env keys, or verdict priority  
- Flipping Cargo `default` or `release.yml`  
- New doctor check or matrix reorder  
- Promoting `GraphHealthOutput` to contracts  
- rusqlite 0.40 `table_exists`  
- Cozo / projector rewrites  
- MSI / binstall / dual artifact  

## 8. Handoffs

| To | What |
|----|------|
| deferred.md | Strike “Doctor graph rebuild vs graph-off” on ship; leave threshold/auto-rebuild residuals |
| T222 | Consumes `graph_feature` + `GRAPH_REINSTALL_SOOT`; no further install-script work |
| T213 | Threshold SOOT stays; remediation text evolves under capability |
| Packaging residual | Release graph-on still out |

## 9. Implementation sketch

### 9.1 Pure assessor API (M2/M3/L2 hard)

```rust
pub(crate) const REMEDIATION_REBUILD: &str = "ai-brains graph rebuild";

/// Capability-aware primary remediation (warn paths + doctor gather-error).
pub(crate) fn density_remediation(graph_cli_available: bool) -> &'static str {
    if graph_cli_available {
        REMEDIATION_REBUILD
    } else {
        crate::commands::governed_common::GRAPH_REINSTALL_SOOT
    }
}

fn density_warn_note(message: &str, graph_cli_available: bool, sparse_nuance: bool) -> String {
    if graph_cli_available {
        if sparse_nuance {
            format!("{message}; rebuild if projection lag suspected")
        } else {
            format!("{message}; run graph rebuild")
        }
    } else {
        format!("{message}; see remediation to install a graph-capable binary")
    }
}

pub fn assess_graph_density(snap: &GraphDensitySnapshot) -> Assessment {
    assess_graph_density_with(snap, cfg!(feature = "graph"))
}

pub fn assess_graph_density_with(
    snap: &GraphDensitySnapshot,
    graph_cli_available: bool,
) -> Assessment {
    // … same threshold arms …
    // remediation: Some(density_remediation(graph_cli_available).into())
    // note: density_warn_note(&message, graph_cli_available, sparse_arm)
}
```

### 9.2 Doctor gather error (M3 hard)

```rust
// was: Some("ai-brains graph rebuild".into())
Some(crate::graph_density::density_remediation(cfg!(feature = "graph")).into())
```

### 9.3 Test matrix (minimal; H1 + O1)

| Case | available | Expected remediation (`assert_eq!`) |
|------|-----------|-------------------------------------|
| sparse 1304/95 | true | `REMEDIATION_REBUILD` |
| sparse 1304/95 | false | `GRAPH_REINSTALL_SOOT` |
| empty_lag pinned500 | true | `REMEDIATION_REBUILD` (no SOOT) |
| empty_lag pinned500 | false | `GRAPH_REINSTALL_SOOT` (no rebuild) |
| orphan 200/0 | true / false | rebuild / SOOT |
| small skip | either | `None` |
| graph.rs sparse fixture | `_with(…, true)` | rebuild (L3) |

**Migrate (H1):** do **not** leave `assess_graph_density(&snap)` + `contains("rebuild")` in always-compiled unit modules.

## 10. Verification plan

```powershell
# Units (default + graph-on if needed for smoke)
cargo nextest run -p ai-brains-cli graph_density
cargo nextest run -p ai-brains-cli doctor
cargo clippy -p ai-brains-cli --all-targets -- -D warnings

# Manual graph-on (PATH)
ai-brains doctor --json   # graph_density remediation = rebuild when sparse
ai-brains graph update

# Optional feature-off dogfood
cargo build -p ai-brains-cli   # no --features graph
# run target\debug\ai-brains.exe doctor --json against vault copy / env
# expect graph_feature unavailable + density remediation = GRAPH_REINSTALL_SOOT when warn

# Full gate
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

## 11. AI fold-in disposition (2026-08-10)

### AI1

| ID | Severity | Disposition | Spec site |
|----|----------|-------------|-----------|
| **H1** | High | **Accept hard** — migrate all remediation-asserting tests to `_with(…, bool)`; graph-off CI will fail otherwise | F3, F24, F29, AC17 |
| **M1** | Medium | **Accept hard** — Ok note informational-only rationale pinned; no required Ok note branch | F7 |
| **M2** | Medium | **Accept hard** — `pub(crate) const REMEDIATION_REBUILD` + exact `assert_eq!` | F4, O1, §9.1 |
| **M3** | Medium | **Accept hard** — `pub(crate) fn density_remediation(bool) -> &'static str` required for doctor | F8, §9.2 |
| **M4** | Medium | **Accept hard** — extend smoke guard for density + doctor SOOT discipline | F17, AC19 |
| **M5** | Medium | **Accept hard** — uniform graph-off warn note suffix pinned | F7, AC18 |
| **L1** | Low | **Accept note** — `_with` name deliberate | F3 |
| **L2** | Low | **Accept preferred** — helper returns `&'static str` | F8, §9.1 |
| **L3** | Low | **Accept elevated** — migrate `graph_health_output__sparse_fixture` to `_with(…, true)` | F24 Phase 1 |
| **L4** | Low | **Accept elevated** — OPERATIONS ~736 hybrid stale | AC10 |
| **L5** | Low | **Affirm** — cross-model optional; smoke better regression guard | F23 |
| **L6** | Low | **Affirm** — deferred strike at ship | §8 |
| **O1** | Opp | **Accept hard** — `assert_eq!` vs consts, not weak `contains` | F4/F5, AC1–4 |
| **O2** | Opp | **Soft** — cfg-not-graph doctor gather integration optional | F26, AC7 |
| **O3** | Opp | **Preferred** — `density_warn_note` helper | F7, §9.1 |
| **O4** | Opp | **Accept hard** — OPERATIONS “When to rebuild” ~717–721 capability-aware | AC10 |

### AI2

| Item | Disposition |
|------|-------------|
| Design affirm (capability matrix, hybrid retire, F9–F12 freeze, capture independence) | **Accept** — matches F1–F28 |
| GatherError capability row | **Accept** — F8 |
| graph update isolation (`#[cfg(feature = "graph")]` → rebuild) | **Accept** — F13 |
| Note + remediation both branch on warn | **Accept** — F7 |
| Skip status = empty | **Reject** — T213 Skip is `status=live` + `density=skip` (F9/F31) |
| Action table items 1–6 | **Accept** as implement map (already in plan phases) |

### Research verification notes (AI1)

AI1 code/CI claims re-verified against plan baseline: `GRAPH_REINSTALL_SOOT`, always-compiled `graph_density`, cfg-gated `graph`, doctor:704 rebuild literal, empty_lag hybrid, graph-off workspace nextest, existing rebuild asserts — **agree**. No change to dep defer pins.
)
