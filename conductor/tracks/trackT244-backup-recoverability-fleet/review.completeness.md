# T244 Completeness Check

**Date:** 2026-08-12  
**Branch:** `feature/T244-backup-recoverability-fleet`  
**Scope:** Spec AC1–AC17 + plan phases 1–4; static code/docs/test review (gate not re-run)  
**Reviewer:** Grok Build (read-only)

## Verdict: **GAPS**

Phases **1–4 product implementation is complete** (classify/list/doctor/verify/docs/hermetics). Full track DoD still has open closeout items below.

---

## Checklist sweeps (all pass)

| # | Check | Result |
|---|--------|--------|
| 1 | No residual SOOT `not fully readable` | **PASS** — repo-wide grep empty |
| 2 | No `empty_meta_token` | **PASS** — renamed to `backup_class_token`; grep empty |
| 3 | Brain `list_backups` `Reverse(timestamp)` only | **PASS** — `crates/ai-brains-brain/src/backup.rs` ~410; usable-first only in CLI `run_list` |
| 4 | All `BackupReadClass` match arms cover `Incomplete` | **PASS** — `emit_list_noise`, `backup_class_token` exhaustive; usable via `is_usable_class` |
| 5 | No production `unwrap`/`expect` on touched paths | **PASS** — only under `#[cfg(test)]` in backup/doctor |
| 6 | CAPABILITIES decision table | **PASS** — `Docs/CAPABILITIES.md` §11 “Usable / class decision table (T244)” |
| 7 | CHANGELOG T244 entry | **PASS** — `CHANGELOG.md` Unreleased |
| 8 | TODO/FIXME/stub for T244 incomplete work | **PASS** — none on backup/doctor surfaces |

---

## AC matrix (phases 1–4 focus)

| AC | Status | Evidence |
|----|--------|----------|
| **AC1** Incomplete + `(no core tables)` | **Met** | Unit `classify_backup_read__openable_without_core_tables__incomplete`; honesty token/residual |
| **AC2** PreT109 = cores + no meta | **Met** | Unit `…__openable_with_core_tables_no_meta__pre_t109`; doctor PreT109 ok |
| **AC3** Incomplete-only → no usable + create | **Met** | `doctor__backup_recent__all_incomplete__warn_no_usable` |
| **AC4** Readable/PreT109 in-age → ok | **Met** | `doctor__backup_recent__readable_within_age__ok`, `…__pret109_within_age__ok` |
| **AC5** Stale usable + fresher Incomplete | **Met** | `doctor__backup_recent__stale_usable_plus_fresher_incomplete__warns` |
| **AC6** Residual all non-usable; SOOT | **Met** | `residual_for_summary`; 8+ honesty sites on `not recoverable under current key` |
| **AC7** CLI usable-first; brain ts-desc | **Met** | Honesty mixed order; brain sort unchanged |
| **AC8** Verify both cores; JSON `tables` | **Met** | `tables_out.len() < 2`; `backup_verify__incomplete_and_single_core__missing_core_tables` |
| **AC9** T225 quiet verify preserved | **Met** | Smoke multi-fail / nudge / verbose retained |
| **AC10** Hermetic create → ≥1 OK | **Met** | `backup_verify__valid_backup__reports_ok` |
| **AC11** Docs CAPABILITIES + OPERATIONS + CHANGELOG | **Met** | Decision table + green path + Unreleased entry |
| **AC12** Live dogfood create→verify≥1 OK→doctor | **Open** | Plan Phase 5 unchecked; Manual evidence still `(pending go)` |
| **AC13** Full CI gate | **Open** | Plan Phase 6 unchecked; not re-executed this pass |
| **AC14** Exhaustive match; no prod unwrap | **Met** | Match arms + test-only unwraps |
| **AC15** Capture independence | **Met** | Backup path: crypto/store/rusqlite only; no models/graph |
| **AC16** No `not fully readable` asserts | **Met** | Grep clean |
| **AC17** Incomplete noise debug/warn | **Code met / test thin** | `emit_list_noise` F27; no Incomplete RUST_LOG hermetic twin of PreT109 |

### Phases 1–4 plan boxes

All marked complete in `plan.md` and backed by code+tests above. Phase 5 (live) and Phase 6 (gate/closeout) remain open by design.

---

## GAPS list

1. **AC12 — Live dogfood not recorded**  
   No operator evidence of `ai-brains backup create --no-prune` → verify `1 OK, 21 FAIL` → doctor `backup_recent` ok. Plan Manual evidence still pending. **Phase 5 (mutating, go only).**

2. **AC13 — Full workspace gate not verified here**  
   `fmt` / `clippy -D warnings` / `nextest` / `deny` / `audit` not re-run in this completeness pass. **Phase 6.**

3. **AC17 — Incomplete noise hermetic thin (P3)**  
   F27 implemented (Default/Quiet `debug!`, Verbose `warn!`); no dedicated Incomplete RUST_LOG hermetic. Non-blocking; optional follow-up.

---

## Product pins (confirmed present)

- F1: `has_core_tables` before meta → `Incomplete` (`backup.rs` classify)
- F4: doctor usable = `is_usable_class` only; create-only remediation
- F5: verify IN query + `len() < 2`; fail `missing core tables`
- F6: residual SOOT `not recoverable under current key`
- F7: CLI `list_sort_key` only; brain stays timestamp-desc
- F8: `backup_class_token` / `(no core tables)`
- F27: Incomplete noise pattern

Cross-ref: `review.internal-r1.md` (PASS WITH DEFERRED P3) aligns.

---

## Orchestrator next

1. Phase 5 live dogfood + paste evidence into `plan.md`  
2. Phase 6 full gate + ledger/F25 hard cross-model + conductor closeout  
3. Optional: Incomplete noise hermetic for AC17 belt-and-suspenders  
