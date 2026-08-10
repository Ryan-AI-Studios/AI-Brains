# T232 — Graph density remediation path — Plan

**Status:** 🚧 **In review** (PR #124; engineering complete)  
**Category:** FEATURE  
**Depends:** T213 density assessor; T222 `graph_feature` + `GRAPH_REINSTALL_SOOT`  
**Feeds:** Operator/agent honesty for sparse/empty graph next actions  
**Ledger:** ed9d7672-2b41-4324-ad98-1d0267c4358e (started; not committed)

## Goal

Doctor (and shared assessor) remediations for density **warn** paths must match **this binary’s** graph capability:

| Capability | Primary remediation |
|------------|---------------------|
| Graph-on (`cfg!(feature = "graph")`) | `REMEDIATION_REBUILD` (`ai-brains graph rebuild`) |
| Graph-off | `GRAPH_REINSTALL_SOOT` only |

Thresholds, matrix order, soft severity, capture independence, and no auto-rebuild stay frozen (T213/T222).

## Absorbed deferred

- deferred.md **Doctor graph rebuild vs graph-off** (T232 placeholder)  
- T222 soft residual **density remediation branching**  
- T213 soft **skill one-liner for density / rebuild** (soft DoD)

**Not absorbed as DoD:** auto rebuild; threshold retune; Cargo default / release graph-on; rusqlite 0.40 / clap; event freshness F31; projector rewrites.

## Decision pins (hard)

| ID | Pin |
|----|-----|
| **F2/F3** | Capability via pure `assess_graph_density_with(snap, bool)` + wrapper `cfg!`; name `_with` deliberate |
| **F4** | Graph-on warn → exact `pub(crate) REMEDIATION_REBUILD` |
| **F5** | Graph-off warn → exact `GRAPH_REINSTALL_SOOT` |
| **F6** | Empty-lag hybrid retired |
| **F7** | Warn notes branched; graph-off uniform suffix; Ok note informational (M1) |
| **F8** | `pub(crate) density_remediation(bool) -> &'static str` for doctor gather-error (M3) |
| **F9–F12** | Thresholds / soft policy / capture / no auto rebuild frozen |
| **F15** | Matrix still **13** |
| **F17** | Smoke guard extended (M4) |
| **H1/F29** | Migrate all remediation-asserting tests off convenience wrapper (graph-off CI) |

## AI fold-in pins (hard)

| ID | Pin |
|----|-----|
| **H1** | Phase 1 migrates empty_lag / orphan / sparse (etc.) to `_with(true\|false)` **before** green relies on wrapper |
| **M1** | Ok note stays capability-blind; rationale: no remediation / no required action |
| **M2** | `REMEDIATION_REBUILD` is `pub(crate)` for exact equality |
| **M3** | Doctor **must** call `density_remediation` — not optional |
| **M4** | Smoke: density SOOT by name; doctor no rebuild literal; rebuild const value |
| **M5** | Graph-off warn note: `"{message}; see remediation to install a graph-capable binary"` |
| **O1** | `assert_eq!` vs consts — not weak `contains("rebuild")` |
| **O4** | OPERATIONS ~717–736 capability-aware; retire hybrid line 736 |
| **L3** | `graph.rs` sparse fixture → `_with(…, true)` |
| **O3** | Prefer `density_warn_note` helper |
| **O2** | Soft optional cfg-not-graph doctor gather test |
| **AI2** | Affirm matrix; **reject** Skip `status=empty` |

## Research pins (2026-08-10)

| Fact | Pin |
|------|-----|
| Live PATH | graph-on; sparse warn; remediation rebuild (correct here) |
| Gap | Feature-off / Release still graph-off; orphan/sparse/projection always rebuild text |
| empty_lag | Hybrid half-measure → replace with F4/F5 |
| CI | Workspace nextest **graph-off**; graph-on job filters `test(graph)` only |
| clap / rusqlite | 4.6.6 / 0.40.2 available — **defer** |
| Rust | 1.95.0 |
| Cargo features | cfg compile-time; additive features |

## Phased checklist

### Phase 0 — Preflight (on go)

- [x] Ledger already started: ed9d7672-2b41-4324-ad98-1d0267c4358e (do not start another)
- [x] Branch: `feat/T232-graph-density-remediation`
- [x] `ledgerful doctor` / `ledgerful ledger status --compact` (orchestrator)
- [x] `ledgerful scan --impact` (orchestrator)
- [x] Manual dogfood baseline (orchestrator)

### Phase 1 — Red + migrate (AC1–6, AC17–18, H1, O1, L3)

- [x] **Migrate** existing remediation-asserting tests to `_with(…, true|false)`:
  - empty_lag dual capability
  - orphan dual capability
  - sparse (1304/95) dual capability
  - projection_lag dual capability
- [x] Use `assert_eq!(a.remediation.as_deref(), Some(REMEDIATION_REBUILD|GRAPH_REINSTALL_SOOT))`
- [x] Assert F7 note suffixes (graph-off uniform; graph-on templates)
- [x] Migrate `graph.rs` `graph_health_output__sparse_fixture` to `_with(&snap, true)` + exact `REMEDIATION_REBUILD`
- [x] Secret-deny / threshold-only tests use `_with(&s, true)`
- [x] Skip still `status=live` (F31)

### Phase 2 — Green: assessor (AC1–6, F3–F7)

- [x] `pub(crate) const REMEDIATION_REBUILD`
- [x] `pub(crate) fn density_remediation(bool) -> &'static str`
- [x] `density_warn_note(…)` for F7
- [x] `assess_graph_density_with` + convenience wrapper
- [x] Wire all warn arms; drop `remediation_empty_lag` hybrid
- [x] Ok note unchanged (M1)

### Phase 3 — Doctor gather-error (AC7, M3)

- [x] Replace hardcoded rebuild with `density_remediation(cfg!(feature = "graph"))`
- [x] Helper unit covers density_remediation true/false (O2 soft integration skipped)

### Phase 4 — Docs + soft skill (AC10, AC11, O4, L4)

- [x] `Docs/OPERATIONS.md` ~717–736: “When to rebuild” capability-aware; hybrid empty-lag line retired
- [x] `Docs/CAPABILITIES.md` — capability-aware density remediation sentence
- [x] `CHANGELOG.md` — T232
- [x] Soft: skill one-liner (rebuild only when graph-capable)

### Phase 5 — Smoke + manual + gate (AC8–9, AC12–16, AC19)

- [x] Extend smoke SOOT guard (F17/M4)
- [x] Manual graph-on: sparse → rebuild (orchestrator dogfood)
- [x] Units prove feature-off density warn → GRAPH_REINSTALL_SOOT
- [x] Regression: feature-off graph * exit 2 covered by existing suite
- [x] No clap/rusqlite lockfile bump
- [x] `cargo nextest run -p ai-brains-cli` **without** `--features graph` (H1 proof — 771 passed)
- [x] `cargo nextest run -p ai-brains-cli --features graph -E 'test(graph)'` (38 passed, incl. sparse fixture)
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [x] Full workspace gate 2497 + `ledgerful verify --scope full` + commit PR #124
- [ ] Conductor **Completed** / deferred strike / pin (post-merge closeout)
- [x] `review.md`; Codex R1 FAIL→fix; R2 pending

## File touch map

| File | Change |
|------|--------|
| `crates/ai-brains-cli/src/graph_density.rs` | `_with` API; `density_remediation`; notes; unit migrate/split |
| `crates/ai-brains-cli/src/commands/doctor.rs` | gather-error → helper; no rebuild literal |
| `crates/ai-brains-cli/src/commands/graph.rs` | sparse fixture → `_with(…, true)` |
| `crates/ai-brains-cli/tests/smoke.rs` | F17 SOOT guard extension |
| `Docs/OPERATIONS.md` (~717–736), `CAPABILITIES.md`, `CHANGELOG.md` | Honesty |
| Soft skill paths | One-liner |
| `conductor/*` | Status / strike |
| `Cargo.toml` / lock / release.yml | **No** |

## Non-goals (reminder)

Auto rebuild · threshold retune · matrix reorder · Cargo/release flip · dep bumps · contracts DTO · capture coupling

## Verification matrix

| AC | Proof |
|----|-------|
| AC1–6, AC17–18 | pure units dual capability + migration |
| AC7 | doctor path + helper |
| AC8–9 | existing feature-off / smoke |
| AC10–11 | docs + skill (O4) |
| AC12–14 | gate (graph-off nextest) + lock + manual |
| AC15–16, AC19 | review + smoke F17 |

## Out of scope checklist

- [ ] Auto rebuild  
- [ ] MIN_* retune  
- [ ] release.yml / Cargo default  
- [ ] rusqlite 0.40 / clap 4.6  
- [ ] event freshness F31  
- [ ] new doctor check  
- [ ] Branch Ok-path note (explicitly declined — M1)  

## Implement notes

1. **H1 first:** migrate tests in the same change set as the wrapper uses `cfg!`, or red tests use `_with` before green lands.  
2. **High findings:** graph-off rebuild-only; wrapper tests left assuming rebuild; hybrid empty_lag left; doctor rebuild literal; thresholds touched.  
3. **Stop-before:** Cargo default flip; auto rebuild; dep bumps.  
4. **After ship:** series density honesty closed; polish T223/T225–T231.  
5. **Category:** `FEATURE`.
)
