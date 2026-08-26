# T308 Plan — Sparse remediator honesty (no rebuild loop)

**Status:** **Completed**. Spec [spec.md](./spec.md).
**Category:** FEATURE / CLI
**Ledger (planning):** `96f0ce16-3a64-43cc-92ac-b9a4d89c46ae` (DOCS)
**Ledger (fold-in):** `91f8fbcd-655e-4fbd-bd64-635e9fa271bf` (DOCS)
**Ledger (implement):** FEATURE TX `d62a3884-5af8-44fc-9434-3b8c31a656af`

---

## Preflight (plan time — 2026-08-26)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `0d0fdab` CLEAN; `origin/main...HEAD` **ahead 1**. Plan-write was `037262e` / 0/0 (m1). Branch `main`. T307 `#224` on origin/main. |
| PATH `ai-brains` | **0.1.3** graph-on; mtime **2026-08-26 6:54:32 AM** |
| PATH `doctor --json` `graph_density` | **warn** Sparse E/N **0.410** (63040/25844); **`remediation: ai-brains graph rebuild`** — **this hole** |
| PATH `graph update --format human` | note has lag nuance **and** remediator rebuild |
| Coverage | ~**0.795** ≫ 0.10 — not projection_lag |
| Floors | `MIN_EDGE_NODE_RATIO = 0.50` (`graph_density.rs:14`) |
| Sparse arm | `:214–226` still `Some(remediation.into())` |
| Doctor | `check_graph_density` `:868` forwards remediator; matrix **15** |
| rustc | **1.95.0** |
| Pins | clap `"4.5"`; rusqlite **0.40.2** — no bump |
| Last PR Cursor | `#224` comments/reviews/issues **empty** — N/A; no T311 |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan; plan TX `96f0ce16`; fold-in TX `91f8fbcd` |
| `ISSUES.md` | **Does not exist** |
| Planning rebuild / install | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| T306 R4 sparse remediator rebuild | **DoD** F2 / AC1 / AC4 / AC9 — **Done** |
| T300 still sparse after live rebuild | **Absorb** — **Done** |
| Floor retune 0.50 | **Declined** F1 |
| T309 `table_exists` | **Not stolen** |
| T310 `run_update` / daemon | **Not stolen** |
| T307 Blocked dual | **Not stolen** |
| last-PR `#224` Cursor | **N/A empty** |
| clap 5 | **Declined** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [x] Confirm cwd `C:\dev\AI-Brains` (not Helping Hands)
- [x] Re-read `graph_density.rs` Sparse arm `:214–226` vs `density_remediation` `:143`
- [x] Confirm `MIN_EDGE_NODE_RATIO` still **0.50**
- [x] Rescan `deferred.md` open overlapping rows
- [x] `ledgerful ledger start T308-graph-sparse-remediator --category FEATURE` → `d62a3884…`
- [x] **Do not** live `graph rebuild` / `daemon stop` / `cargo install` in Phase 0

## Phase 1 — Red (AC1 / AC4)

- [x] Flip `assess_graph_density_with__sparse_1304_95_graph_on__rebuild` → `…__no_rebuild_remediator` (`remediation.is_none()`, note still lag nuance)
- [x] Flip `assess_graph_density_with__ratio_0_4__warn_sparse` remediator **None**
- [x] Flip `graph_health_output__sparse_fixture__status_sparse_with_remediation` → `…__omits_remediation` (JSON key absent)
- [x] Confirm those tests **fail** on current Sparse arm (compile overlapped green; asserts match AC)

## Phase 2 — Green

- [x] Sparse arm graph-on: `remediation: None`; graph-off: still `GRAPH_REINSTALL_SOOT`
- [x] Keep `density_warn_note(..., sparse_nuance=true)`
- [x] Other warn arms still `density_remediation(...)` (AC2)
- [x] Do **not** edit `doctor.rs` unless compile-forced (unexpected)
- [x] Do **not** edit `emit_graph_health_human` (`graph.rs:381–383` already omits `None`)
- [x] Do **not** edit `has_graph_tables` (AC10 / T309)
- [x] Do **not** change floors
- [x] Do **not** edit PROTOCOL-COMPAT `:96` (already optional)

## Phase 3 — Stay-green + docs

- [x] AC2 empty_lag / orphan / projection_lag rebuild units
- [x] AC3 graph-off Sparse reinstall
- [x] AC5 floors + smoke F17
- [x] AC6 matrix 15
- [x] AC7 clippy + targeted nextest
- [x] AC8 OPERATIONS / CAPABILITIES / CHANGELOG (Sparse omits remediator; lag arms still rebuild)
- [x] Optional AC9 live `doctor --format json` — Sparse has no remediator key via `cargo run --features graph`. **No rebuild.** PATH binary still stale until install (R3 soft).

## Phase 4 — Closeout

- [x] Conductor T308 **Completed**; deferred T306 R4 **Done**
- [x] `ledgerful verify --scope fast` then full as implement-track requires (`dev-check.ps1` exit 0)
- [ ] Phase 6: `track/T308-*` → PR → watch `CI` green → squash-merge. Never `git push origin main`

## DoD

- [x] AC1–AC8 (AC9 optional live); AC10 T309 not stolen
- [x] Floors unchanged; no projector rewrite; no live rebuild as required DoD
- [x] T307 / T309 / T310 not stolen
- [x] Never `git push origin main`

## Evidence commands

```
cargo nextest run -p ai-brains-cli assess_graph_density_with__sparse   # PASS (3)
cargo nextest run -p ai-brains-cli --features graph graph_health_output__sparse_fixture  # PASS
cargo clippy -p ai-brains-cli --all-targets -- -D warnings  # exit 0
cargo nextest run -p ai-brains-cli graph_stub__reinstall_hint__matches_install_soot  # PASS
Select-String -Path crates/ai-brains-cli/src/graph_density.rs -Pattern 'MIN_EDGE_NODE_RATIO'  # 0.50
cargo run -p ai-brains-cli --features graph -- doctor --format json  # Sparse omits remediation
```
