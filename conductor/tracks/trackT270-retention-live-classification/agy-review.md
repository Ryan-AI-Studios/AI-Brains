# Track review: T270-RetentionLiveClassification

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT270-retention-live-classification`  
**Date:** 2026-08-20  
**HEAD:** `70d61cd`  

---

## Summary

Track T270 resolves a long-standing reporting gap identified in the 2026-08-16 CLI audit where `ai-brains retention plan` reported 0 candidates on vaults containing tens of thousands of pinned memories (currently 38,208 pins on this system).

Under T166/T248, `collect_candidates` scanned ephemeral turns, query traces, and disposable decisions, but never inventoried `memory_projection` as a class. Consequently, `retention plan` displayed `Nothing to dispose.` alongside `memory_legacy none_auto skip 0`, leading operators to question whether the retention engine inspected the vault at all.

T270 resolves this by introducing a lightweight, read-only inventory overlay:
1. **Lightweight SQL Counting:** Adds `memory_legacy_inventory` in `ai-brains-store` executing `COUNT(*)` and `LIMIT 5` sample queries without materializing 38k `Candidate` structs.
2. **Deterministic Classification:** Pinned memories are classified as `held` (under R11 policy), while other statuses (`forgotten`, `active`) are classified as `skip` under the `none_auto` horizon.
3. **Display Honesty:** `Nothing to dispose.` is refactored to indicate the absence of active disposal work (`would_ce_wipe + would_projection_delete == 0`), while the class matrix accurately reflects held memory inventory.
4. **Strict Immutability:** `retention plan` remains 100% read-only and appends zero events. `retention apply` remains gated with `--confirm`.

The plan is thorough, architecturally sound, and respects all project invariants.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Deterministic sample IDs on non-pinned inventory (AC1):** In `memory_legacy_inventory`, ensure that if `pinned == 0` and `other > 0` (e.g. only forgotten or active memories exist), the sample query falls back to `status != 'pinned' ORDER BY memory_id ASC LIMIT 5` so `sample_ids` remains populated and deterministic.
- **m2: Class bucket ordering in reports (F6 / AC5):** When upserting `CLASS_MEMORY_LEGACY` into `report.classes`, ensure `classes` is sorted deterministically (by class name or canonical class order) to guarantee bit-for-bit identical JSON output across runs.

### Opportunities (O)
- **O1: Centralized note constant in control plane:** Define `pub(crate) const NOTE_MEMORY_LEGACY_INVENTORY: &str = "inventory overlay; none_auto; pinned held (R11); other skip";` in `class_based_retention.rs` to ensure unit test assertions and report generators share the single source of truth.
- **O2: SQL-level LIMIT 5:** Ensure `memory_legacy_inventory` performs `LIMIT 5` directly in SQL rather than slicing in Rust to minimize memory allocation.

---

## What Looks Solid

1. **Overlay vs Materialization:** Using a `COUNT` + `LIMIT 5` overlay rather than instantiating 38k in-memory `Candidate` objects prevents latency spikes and avoids unnecessary churn in the apply prepare pipeline.
2. **Accurate Semantics of "Nothing to dispose.":** Decoupling `Nothing to dispose.` from `candidates == 0` ensures operators understand that held inventory requires no disposal action without hiding the existence of their pins.
3. **No Schema / Contract Regressions:** Preserves `api_version: "1"`, leaves `RetentionPlanReport` keys frozen, and introduces only an additive honesty warning string and class bucket.
4. **Hotspot Isolation:** Confines changes strictly to `crates/ai-brains-store/src/projections/retention.rs`, `crates/ai-brains-control-plane/src/class_based_retention.rs`, and CLI retention formatting. Top hotspots (`project.rs`, `sync.rs`, `preflight.rs`) are untouched.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| Audit 6/5 zero candidates on ~35k memories | Absorbed into DoD (F1–F11 / AC1–AC13) | Resolved via `memory_legacy` inventory overlay |
| Placeholder F1 honesty sentence | Absorbed as overlay + warning (F1/F10) | Resolved with `held` count + honesty short |
| Placeholder F2 optional overlay | Absorbed (F1 / F5) | Selected COUNT + samples approach |
| Placeholder F3 `none_auto` stands | Affirmed (F3) | Memory legacy horizon remains `none_auto` |
| Placeholder F4 apply gated | Affirmed (F4) | Apply remains JSON default + `--confirm` |
| T166 §5.1.5 memory legacy scan | Absorbed as inventory overlay | Replaces missing scan without age-wipe mutations |
| `classify_legacy` / migrate as remediator | Declined (F1 / F18) | Correctly avoided mutative migration |
| T248 empty-check lift | Absorbed (F8) | Evaluates `ce_wipe + projection_delete == 0` |
| Peer tracks (T272, T273) | Declined (F21) | Kept strictly isolated |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#187](https://github.com/Ryan-AI-Studios/AI-Brains/pull/187) (merged 2026-08-20, T272 `Preflight --global Safety skip vs Index`).
- **Cursor Comments:** None (`[]` on PR #187).
- **Disposition:** N/A (no pending Bugbot findings).

---

## Research / Tools Notes

- **Retention Standards:** ISO 27001:2022 A.8.10 and GDPR Art. 5(1)(e) emphasize maintaining documented retention schedules and logging review results even when zero items are due for disposal.
- **Dependencies:** `clap` (4.6.1), `serde_json` (1.0.150), `rusqlite` (0.39.0), `chrono` (0.4.44).
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,255 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search collect_candidates`: Located at `crates/ai-brains-control-plane/src/class_based_retention.rs:234/:269`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
