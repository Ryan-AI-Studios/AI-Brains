# T308 Plan — Sparse remediator honesty (no rebuild loop)

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Category:** FEATURE / CLI
**Ledger (planning):** `96f0ce16-3a64-43cc-92ac-b9a4d89c46ae` (DOCS)
**Ledger (implement):** FEATURE TX on **go**.

---

## Preflight (plan time — 2026-08-26)

| Check | Result |
|-------|--------|
| HEAD / tree | `037262e` CLEAN; `origin/main...HEAD` **0/0**. Branch `main`. T307 `#224` on origin/main. |
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
| Ledger | 0 pending / 0 drift at scan; plan TX `96f0ce16` |
| `ISSUES.md` | **Does not exist** |
| Planning rebuild / install | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| T306 R4 sparse remediator rebuild | **DoD** F2 / AC1 / AC4 / AC9 |
| T300 still sparse after live rebuild | **Absorb** |
| Floor retune 0.50 | **Declined** F1 |
| T309 `table_exists` | **Not stolen** |
| T310 `run_update` / daemon | **Not stolen** |
| T307 Blocked dual | **Not stolen** |
| last-PR `#224` Cursor | **N/A empty** |
| clap 5 | **Declined** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains` (not Helping Hands)
- [ ] Re-read `graph_density.rs` Sparse arm `:214–226` vs `density_remediation` `:143`
- [ ] Confirm `MIN_EDGE_NODE_RATIO` still **0.50**
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] `ledgerful ledger start T308-graph-sparse-remediator --category FEATURE`
- [ ] **Do not** live `graph rebuild` / `daemon stop` / `cargo install` in Phase 0

## Phase 1 — Red (AC1 / AC4)

- [ ] Flip `assess_graph_density_with__sparse_1304_95_graph_on__rebuild` → `…__no_rebuild_remediator` (`remediation.is_none()`, note still lag nuance)
- [ ] Flip `assess_graph_density_with__ratio_0_4__warn_sparse` remediator **None**
- [ ] Flip `graph_health_output__sparse_fixture__status_sparse_with_remediation` → `…__omits_remediation` (JSON key absent)
- [ ] Confirm those tests **fail** on current Sparse arm

## Phase 2 — Green

- [ ] Sparse arm graph-on: `remediation: None`; graph-off: still `GRAPH_REINSTALL_SOOT`
- [ ] Keep `density_warn_note(..., sparse_nuance=true)`
- [ ] Other warn arms still `density_remediation(...)` (AC2)
- [ ] Do **not** edit `doctor.rs` unless compile-forced (unexpected)
- [ ] Do **not** edit `has_graph_tables` (AC10 / T309)
- [ ] Do **not** change floors

## Phase 3 — Stay-green + docs

- [ ] AC2 empty_lag / orphan / projection_lag rebuild units
- [ ] AC3 graph-off Sparse reinstall
- [ ] AC5 floors + smoke F17
- [ ] AC6 matrix 15
- [ ] AC7 clippy + targeted nextest
- [ ] AC8 OPERATIONS / CAPABILITIES / CHANGELOG (Sparse omits remediator; lag arms still rebuild)
- [ ] Optional AC9 live `doctor --format json` — Sparse has no remediator key. **No rebuild.**

## Phase 4 — Closeout

- [ ] Conductor T308 **Completed**; deferred T306 R4 **Done**
- [ ] `ledgerful verify --scope fast` then full as implement-track requires
- [ ] Phase 6: `track/T308-*` → PR → watch `CI` green → squash-merge. Never `git push origin main`

## DoD

- [ ] AC1–AC8 (AC9 optional live); AC10 T309 not stolen
- [ ] Floors unchanged; no projector rewrite; no live rebuild as required DoD
- [ ] T307 / T309 / T310 not stolen
- [ ] Never `git push origin main`

## Evidence commands

```
cargo nextest run -p ai-brains-cli --lib assess_graph_density_with__sparse
cargo nextest run -p ai-brains-cli --lib graph_health_output__sparse_fixture
cargo clippy -p ai-brains-cli --all-targets -- -D warnings
cargo nextest run -p ai-brains-cli graph_stub__reinstall_hint__matches_install_soot
Select-String -Path crates/ai-brains-cli/src/graph_density.rs -Pattern 'MIN_EDGE_NODE_RATIO'
ai-brains doctor --format json
```
