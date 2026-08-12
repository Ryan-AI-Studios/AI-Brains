# Track Internal Review — T244 R1

**Track:** T244-BackupRecoverabilityFleet  
**Reviewer:** Grok Build (read-only internal, codex-review style)  
**Date:** 2026-08-12  
**Scope:** Product + hermetic tests + docs against hard pins F1–F8, F13–F16, F27, AC1–AC11, AC16–AC17  
**Method:** Static review of `spec.md`, `plan.md`, production/test/doc surfaces; grep sweeps for forbidden SOOT leftovers. Tests reported green by implementer (not re-run here).

## Verdict: PASS WITH DEFERRED P3

Product implementation matches the locked pins for classify Incomplete, doctor usable SOOT, verify `len() < 2` + JSON `tables`, residual wording, CLI-only usable-first sort, capture independence, and no auto mass-delete. Docs include the decision table. Residual gaps are closeout/ops and a narrow AC17 noise hermetic, not false-usable or SOOT regressions.

---

## Requirement and DoD Matrix

| ID | Requirement | Status | Evidence |
|----|-------------|--------|----------|
| **F1** | After key_ok, `has_core_tables` before meta; false → `Incomplete` | **Met** | `classify_backup_read` gate at `backup.rs` ~478–480; meta branch only after cores |
| **F2** | PreT109 = key + cores + meta unusable/absent | **Met** | Core gate first; unit `classify_backup_read__openable_with_core_tables_no_meta__pre_t109`; debug text “core tables present” now true |
| **F3** | Readable = key + cores + meta map | **Met** | Unchanged meta success path after F1 |
| **F4** | Doctor usable via `is_usable_class` only; create-only remediation | **Met** | `doctor.rs` `check_backup_recent` filter `is_usable_class`; rem `"ai-brains backup create"` only; AC3 hermetic asserts no `verify` in rem |
| **F5** | Keep IN query → `tables_out`; gate `len() < 2`; not `has_core_tables` swap | **Met** | `verify_single_backup` IN + collect + `len() < 2`; fail `"backup is missing core tables"`; JSON hermetic asserts `tables` array + single-core `["events"]` |
| **F6** | residual = `!is_usable`; SOOT `not recoverable under current key`; 8 honesty sites migrated | **Met** | `residual_for_summary`; CLI summary string; grep: **0** `not fully readable`; honesty file has **11** new SOOT sites (incl. T244 Incomplete extras) |
| **F7** | CLI `run_list` usable-first only; brain `list_backups` stays `Reverse(timestamp)` | **Met** | `list_sort_key` + `backups.sort_by_key` in CLI only; brain line 410 unchanged |
| **F8** | `backup_class_token`; Incomplete → `(no core tables)` | **Met** | Rename complete; grep: **0** `empty_meta_token` |
| **F9** | T225 quiet verify frozen | **Met** | `run_verify` quiet path + smokes retained |
| **F10** | No new create engine; docs green path | **Met** | Create path untouched; CAPABILITIES/OPERATIONS green path |
| **F12** | Hermetic create → list/verify/doctor | **Met** | smoke `backup_verify__valid_backup__reports_ok`; doctor readable-within-age; list mixed usable-first |
| **F13** | Junk fixture Incomplete | **Met** | Unit + honesty `write_incomplete_bak` + doctor Incomplete-only |
| **F14** | Capture independence | **Met** | Backup path: crypto/store/rusqlite only; no models/graph deps in classify/list/verify |
| **F15** | Zero new crates | **Met** | No rusqlite/chrono bump in scope |
| **F16** | No auto mass-delete | **Met** | No new delete/quarantine path; prune remains operator |
| **F19** | No shared DTO; Incomplete serializes | **Met** | Brain-local enum + `Serialize` |
| **F22** | CAPABILITIES §11 decision table + OPERATIONS + CHANGELOG | **Met** | Decision table present; OPERATIONS green path; CHANGELOG Unreleased T244 |
| **F23** | Determinism sort / None last | **Met** | Unit `list_sort_key__none_timestamp_last_within_band` + path tiebreak |
| **F24** | Exhaustive matches for Incomplete | **Met** | `emit_list_noise`, `backup_class_token`, usable helpers exhaustive |
| **F26** | Exit codes | **Met** | verify fail→1; list 0; doctor soft warn unchanged |
| **F27** | Incomplete noise debug Default/Quiet, warn Verbose | **Met (code)** | `emit_list_noise` Incomplete arm; message mentions events/memory_projection |
| **AC1** | Incomplete + token | **Met** | Unit + honesty token/residual |
| **AC2** | PreT109 with cores | **Met** | Unit + doctor PreT109 ok hermetic |
| **AC3** | Incomplete-only fleet → no usable | **Met** | `doctor__backup_recent__all_incomplete__warn_no_usable` |
| **AC4** | Readable/PreT109 in-age → ok | **Met** | readable_within_age + pret109_within_age |
| **AC5** | Stale usable + fresher Incomplete → age usable | **Met** | `doctor__backup_recent__stale_usable_plus_fresher_incomplete__warns` |
| **AC6** | Residual counts all non-usable; SOOT substring | **Met** | Honesty Incomplete residual count + SOOT migration |
| **AC7** | Usable-first list; brain order for doctor | **Met** | Mixed list hermetic + brain sort pin |
| **AC8** | Verify both cores; JSON tables populated | **Met** | `backup_verify__incomplete_and_single_core__missing_core_tables` |
| **AC9** | T225 quiet verify | **Met** | Existing smoke multi-fail / mixed / nudge |
| **AC10** | Hermetic create → ≥1 verify OK | **Met** | `backup_verify__valid_backup__reports_ok` |
| **AC11** | Docs | **Met** | CAPABILITIES §11, OPERATIONS Backup, CHANGELOG |
| **AC12** | Live dogfood create→verify≥1 OK→doctor | **Open** | plan Phase 5 pending manual evidence |
| **AC13** | Full CI gate | **Open** | plan Phase 6 unchecked (implementer claims tests green; gate not re-audited here) |
| **AC14** | Exhaustive matches; no unwrap/expect in prod paths touched | **Met** | Production classify/list/doctor/verify use Result/`unwrap_or`; unwrap/expect only in tests |
| **AC15** | Capture independence | **Met** | See F14 |
| **AC16** | No remaining `not fully readable` asserts | **Met** | Workspace grep clean |
| **AC17** | Incomplete Default quiet=debug; Verbose may warn | **Code Met / Test thin** | Implementation correct; no Incomplete-specific RUST_LOG hermetic (PreT109 pattern exists) |

---

## Findings

### P0 — None

### P1 — None

No false-usable path remains: Incomplete cannot pass `is_usable_class`; PreT109 requires cores; verify requires both tables without emptying JSON `tables`; brain list sort not broken for doctor.

### P2 — None

### P3 — Deferred (non-blocking)

#### P3-1 — AC17 Incomplete noise lacks dedicated hermetic
- **What:** F27/AC17 Incomplete debug/warn is implemented and mirrors LegacyPlain, but there is no Incomplete twin of `backup_list__pre_t109_backup__no_warn_on_stderr` (RUST_LOG=warn + junk Incomplete → no WARN; verbose → warn).
- **Evidence:** `emit_list_noise` Incomplete arms at `crates/ai-brains-brain/src/backup.rs` ~531–543; honesty covers token/residual only.
- **Risk:** Low — static match is exhaustive and pattern-identical to pinned LegacyPlain; regression would need intentional edit.
- **Disposition:** Defer; optional follow-up test or accept static review for AC17.

#### P3-2 — Live dogfood (AC12 / Phase 5) not recorded
- **What:** plan Manual evidence still `(pending go)`; live fleet still 21 residual files under repo `backups/`; no recorded `backup create --no-prune` → `1 OK, 21 FAIL`.
- **Risk:** Ops only — product honesty now correctly reports zero usable until create.
- **Disposition:** Orchestrator Phase 5; not a code defect.

#### P3-3 — Closeout bookkeeping stale
- **What:** `conductor/deferred.md` still lists T244 as **Planning** / plan-only; plan Phase 6 (full gate, ledger commit, conductor.md Completed) unchecked.
- **Disposition:** Closeout after AC12 + gate + F25 hard cross-model.

#### P3-4 — F12 u64::MAX max-age pattern not used in new doctor tests
- **What:** Spec/plan cite `18446744073709551615d`; new AC4 hermetics use `7d` with fresh create (equivalent for ok path).
- **Risk:** Nil for correctness.
- **Disposition:** Informational; no change required.

---

## Completeness Sweep

| Sweep target | Result |
|--------------|--------|
| `not fully readable` | **Absent** (repo-wide) |
| `empty_meta_token` | **Absent** |
| `tables_out.is_empty` as core gate | **Absent** (gate is `len() < 2`) |
| PreT109 without cores | **Impossible** post-F1 (unit: meta-without-cores → Incomplete) |
| Brain `list_backups` sort change | **None** — still `Reverse(timestamp)` |
| CLI-only sort | **Present** — `run_list` after `list_backups` |
| `is_usable_class` SOOT single | **Yes** — doctor + list sort + residual complement |
| Exhaustive `BackupReadClass` matches | **Complete** in `emit_list_noise`, `backup_class_token` |
| TODO/FIXME in T244 surfaces | **None** material |
| Production `unwrap`/`expect` in touched paths | **None** (tests only) |
| Auto mass-delete | **None** |
| Capture/graph/models on backup path | **None** |
| CAPABILITIES §7 decision table | **Present** under §11 |
| SOOT residual string exact class | **Pinned** with legacy/incomplete/key/corrupt wording |

---

## Wiring

| Surface | Wired |
|---------|-------|
| `BackupReadClass::Incomplete` | brain enum + Serialize + Default stays Readable |
| `is_usable_class` / `residual_for_summary` | brain + re-export `lib.rs` → CLI list + doctor |
| `classify_backup_read` F1 gate | before meta; Incomplete returns empty meta map |
| `emit_list_noise` Incomplete | F27 |
| `run_list` residual + sort + token | F6/F7/F8 |
| `verify_single_backup` | F5 IN + `len() < 2` |
| `check_backup_recent` | F4 `is_usable_class` |
| Hermetics | honesty Incomplete residual/order/verify; doctor AC3/AC5; units F1/F2/F13 |
| Docs | CAPABILITIES §11, OPERATIONS Backup, CHANGELOG T244 |

Doctor continues to call brain `list_backups` (timestamp-desc), then filters usable — F7 pin satisfied so `find_map` newest usable still works.

---

## Residual notes for orchestrator

1. **Ship code honesty now:** live dogfood will correctly show residual wall + zero usable until operator `ai-brains backup create --no-prune` (or default keep-10 with prune count recorded).
2. **Phase 5:** run and paste exact outputs into plan Manual evidence; expect **22** files / **1 OK, 21 FAIL** under `--no-prune`.
3. **Phase 6:** full CI gate + `ledgerful verify` + **F25 hard cross-model** on classify+doctor (data-safety) + deferred.md / conductor.md closeout + pin DECISION.
4. **Optional P3:** Incomplete RUST_LOG=warn noise hermetic for AC17 belt-and-suspenders.
5. **Do not** auto-delete the 21 legacy files as track DoD (F16).

---

## Pin audit detail (hard pins)

### F1 Incomplete after key_ok via has_core_tables
```478:480:crates/ai-brains-brain/src/backup.rs
    // T244 F1: key opens but missing product cores → Incomplete (never usable).
    if !has_core_tables(&conn) {
        return (BackupReadClass::Incomplete, HashMap::new());
```

### F5 verify gate (not has_core_tables swap)
```445:460:crates/ai-brains-cli/src/commands/backup.rs
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name IN ('events', 'memory_projection')",
    )?;
    // ... collect into tables_out ...
    // T244 F5: require both core tables; keep IN query so JSON `tables` stays populated.
    if tables_out.len() < 2 {
        return Err("backup is missing core tables".into());
```

### F4 / F6 SOOT helpers
```32:39:crates/ai-brains-brain/src/backup.rs
pub fn is_usable_class(class: BackupReadClass) -> bool {
    matches!(class, BackupReadClass::Readable | BackupReadClass::PreT109)
}
pub fn residual_for_summary(class: BackupReadClass) -> bool {
    !is_usable_class(class)
}
```

### F7 brain sort unchanged
```410:411:crates/ai-brains-brain/src/backup.rs
        infos.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        Ok(infos)
```

### F7 CLI sort
```171:172:crates/ai-brains-cli/src/commands/backup.rs
    // T244 F7: usable-first presentation only in CLI list (not brain list_backups).
    backups.sort_by_key(list_sort_key);
```

---

**End of internal R1 review.**
