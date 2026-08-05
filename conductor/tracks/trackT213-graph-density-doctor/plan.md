# T213 Plan — Graph density doctor

Status: **In progress / implementing** (user **go**). Spec: [spec.md](./spec.md).

## Goal

1. Stop hard-coded `graph update` **`status: "live"`** when the relational graph is empty or implausibly under-linked vs vault scale.  
2. Add doctor soft check **`graph_density`** (warn → degraded; never hard-fail alone).  
3. Single pure assessor (F3/F32) shared by both surfaces; SQL gather only; capture-independent.  
4. Docs + hermetic/unit proof; zero new crates; no auto-rebuild.

## Absorbed deferred / audit / AI fold-in

| Source | Item | Handling |
|--------|------|----------|
| deferred.md T213 | graph update effect 6; 1223/24 vs 15k memories | Hard DoD F7–F12 |
| residual series order | T213 after T212/T215 | This track |
| audit2 “status live without validation” | density half | F12; freshness → soft F31 |
| OPERATIONS under-specified rebuild rule | live+sparse slips | F27 docs |
| T74 smoke | ≥1 live only | F21 keep live + **keep `note`** (M3) |
| T198/T200 | feature-off exit 2 | F14/F26/L3 — do not regress |
| T192 doctor model | soft warn / skip / exit | F15/F16; **11th check** (M5) |
| AI1 **M1** | 0.15 bug-calibrated | **F9 → 0.50** typed-lineage floor; **reject 1.0** (tree E/N&lt;1) |
| AI1 **M2** | AC6 retune | AC6 ratio 0.8→Ok; **AC6b** 0.4→warn |
| AI1 **M3** | `note` silent | **Keep note** F13 |
| AI1 **M4** | governed kinds | F10/F20 document kind=`memory` only |
| AI1 **M5** | doctor order test | F16 + Phase 4 explicit |
| AI1 L1/L2/L3/L5/L7 | conn reuse / probe / stubs / deferred.md / density vocab | F19, implement notes, F14, Phase 5, F13 |
| AI2 | pure module + expand JSON + doctor SQL | Affirms |

**Not absorbed:** auto rebuild; graph default-on; projector rewrite; WCC/GDS; T214; T216; MIN=1.0 (rejected); two-tier coverage v1 (L6 soft residual); rusqlite 0.40 bump (L4).

## Live dogfood freeze (2026-08-05)

| Metric | Value |
|--------|-------|
| nodes / edges | **1304 / 95** (E/N ≈ **0.073**) |
| `status` today | always **`live`** |
| sample project memories | **8398** pinned (one project) |
| historical status.md | 861n / 72e ≈ **0.084** (also under 0.50) |
| PATH install graph | FEATURE_UNAVAILABLE (graph-off) |
| doctor checks | 10 — **no** density |

Defaults: `MIN_PINNED=100`, `MIN_NODES=50`, **`MIN_EDGE_NODE_RATIO=0.50`**, `MIN_MEMORY_COVERAGE=0.10` (severe floor).

## Phases

### Phase 0 — Plan freeze

- [x] Live dogfood `graph update` + doctor JSON
- [x] Online KG structure-health research (Adaptive GraphRAG / NetworkX density ≠ E/N)
- [x] Spec F1–F32 + AC1–AC17 + AC6b
- [x] AI fold-in M1–M5 / L1–L8 disposition **§14**
- [x] deferred.md + conductor registry note
- [x] User **go** before code / ledger TX

### Phase 1 — Ledger + red pure tests

- [x] `ledgerful doctor` / `ledgerful ledger status --compact` (ledger TX pre-started by operator)
- [x] `ledgerful ledger start T213-graph-density-doctor --category FEATURE --message "graph density honesty: assessor + doctor check + graph update status"` (TX `13a57574-…` — do not re-start)
- [x] Red pure units: AC1 empty_lag; AC2 small skip; AC3 orphan; AC4 live-sparse (0.073); AC5 small ok; **AC6 healthy 0.8**; **AC6b 0.4→warn**; AC7 projection_lag; priority F11
- [x] Red helpers: pinned-fail skip (AC12); tables-missing skip path (AC11 shape)

### Phase 2 — Green pure + gather SQL

- [x] Add `graph_density.rs` (always-compiled): snapshot + `Assessment` + consts + optional env parse
- [x] SQL gather on **held connection** only (L1 — **not** `try_count_pinned_optional`)
- [x] Prefer `has_graph_tables` sqlite_master pattern like `has_core_tables` (L2)
- [x] Green pure AC1–AC7, AC6b, AC12

### Phase 3 — graph update

- [x] Expand `GraphHealthOutput` (F13): **keep `note`**; add pinned/memory_nodes/ratio/density/remediation
- [x] Map verdict → `status` live|sparse|empty; `density` ok|warn|skip; remediation
- [x] Green AC8–AC9; keep T74 AC14 (`status=live` on healthy hermetic; note still present) — `test_graph_health_smoke` green
- [x] **Do not** touch feature-off graph stubs (L3)

### Phase 4 — doctor

- [x] Insert `graph_density` as check **#10** before `integrity` (F16)
- [x] **Update** `health_check_order_names__fixed_matrix` → len **11** + name at position 10 (M5)
- [x] Bump `Vec::with_capacity(11)` in `build_report` (M5)
- [x] Open-failed / tables-missing / pinned-count-fail → skip
- [x] Feature-off compile path still includes check (AC13)
- [x] Green AC10–AC11, AC17

### Phase 5 — Docs + ship

- [x] CAPABILITIES §9 graph update fields + doctor matrix row (11 checks)
- [x] OPERATIONS graph health (sparse/empty + rebuild + graph-off lag; shape includes note + new fields)
- [x] PROTOCOL-COMPAT if graph update shape listed
- [x] CHANGELOG minor
- [ ] Soft: skill one-liner if free (residual)
- [x] **Append soft residuals to `conductor/deferred.md`** (L5 — not ISSUES.md): F17/F24/F31 + L4/L6 if still open
- [x] Internal review → cross-model (Claude final **PASS**; Codex rate-limited)
- [x] Full CI gate + manual dogfood §9 of spec (2177 nextest; live sparse dogfood)
- [x] `ledgerful ledger commit` / PR #97 squash-merged `355bf09`; mark conductor **Completed**; deferred T213 residuals recorded

## Touch map (implement)

| File | Change |
|------|--------|
| `crates/ai-brains-cli/src/graph_density.rs` | **New** pure assessor + types + optional `has_graph_tables` |
| `crates/ai-brains-cli/src/commands/graph.rs` | Expanded health report (**keep note**) |
| `crates/ai-brains-cli/src/commands/doctor.rs` | `graph_density` check #10; capacity 11; **fixed_matrix** update |
| `crates/ai-brains-cli/src/lib.rs` or `main`/mod | `mod graph_density` |
| `crates/ai-brains-cli/tests/*` | hermetic density / doctor check if needed |
| `crates/ai-brains-cli/tests/smoke.rs` | T74 assert still live (no note drop) |
| `Docs/CAPABILITIES.md` | §9 + doctor |
| `Docs/OPERATIONS.md` | graph health shape |
| `CHANGELOG.md` | minor |
| `conductor/deferred.md` | ship residuals |

## Threshold quick reference

| Condition | Verdict | graph `status` | doctor severity |
|-----------|---------|----------------|-----------------|
| tables missing / pinned count fail / open fail | Skip | n/a (update errors on missing tables) | skip |
| pinned &lt; 100 ∧ nodes=0 ∧ edges=0 | Skip | n/a | skip |
| pinned ≥ 100 ∧ nodes=0 ∧ edges=0 | empty_lag | **empty** | warn |
| nodes ≥ 50 ∧ edges=0 | orphan_nodes | **sparse** | warn |
| nodes ≥ 50 ∧ E/N &lt; **0.50** | sparse | **sparse** | warn |
| coverage lag (kind=`memory` only) | projection_lag | **sparse** | warn |
| else | Ok | **live** | ok |

## Stop-before

- User has not said **go** → no production code, no ledger TX.  
- Do not flip release.yml / Cargo default graph-on.  
- Do not call rebuild from doctor/update.  
- Do not add GDS/petgraph deps.  
- Do not set MIN_EDGE_NODE_RATIO = 1.0.  
- Do not density-enable feature-off `graph update` stubs.

## Success

Audit effect-6 closed: operators see **sparse/empty** + rebuild guidance; doctor one-command surfaces density (11-check matrix); pure tests lock **0.50** lineage floor + AC6/AC6b; T74 still green with `note`; gate green.
