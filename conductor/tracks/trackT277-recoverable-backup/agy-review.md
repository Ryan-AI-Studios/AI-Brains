# Track review: T277-RecoverableBackup

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT277-recoverable-backup`  
**Date:** 2026-08-21  
**HEAD:** `c51d673`  

---

## Summary

Track T277 addresses a critical operational safety and recoverability gap discovered during the 2026-08-21 CLI audit:
The vault's `backups/` directory currently contains 22 `.db.bak` files, but every single file fails verification under the active `AI_BRAINS_KEY` (including the 2026-08-12 T244 backup, which regressed to `(unreadable key)` after key rotation). Consequently, `ai-brains doctor` warns with `no usable encrypted backup under current key` and remediator `ai-brains backup create`.

T277 resolves this through fail-closed verification and mixed-fleet validation:
1. **Fail-Closed Post-Creation Gate:** In `crates/ai-brains-brain/src/backup.rs` (`run_backup_from_conn`), after creating a backup and writing metadata, the engine immediately calls `classify_backup_read(&backup_path, &self.key)`. If the resulting file does not satisfy `is_usable_class` (e.g. `Incomplete`, `KeyMismatch`, `Corrupt`), the file is deleted and an explicit error is returned. This guarantees that `ai-brains` never prints `Backup created and verified:` for an unrecoverable snapshot.
2. **Mixed-Fleet Recoverability Lock:** Adds hermetic tests verifying that running `backup create --no-prune` in a fleet containing other-key residual backups produces a `Readable` snapshot at the top of `backup list`, turns `doctor backup_recent` to `Ok`, and causes `backup verify` to report `1 OK` (and suppress the create nudge).
3. **Conservative Operator Path:** Preserves old ciphertext backups as immutable historical records (no automated rekeying or mass-deletion); the live creation runbook uses `--no-prune` and requires explicit owner confirmation at execute time.

The plan is well-bounded, adheres to NIST SP 800-57 / CISA recovery guidance, and maintains capture independence.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Explicit destination connection drop before classification (F2 / AC1):** In `run_backup_from_conn`, ensure the destination `rusqlite::Connection` is dropped before calling `classify_backup_read` to prevent SQLite file-locking conflicts on Windows.
- **m2: Informative error diagnostic on classification failure (F2):** Ensure the returned error string from `run_backup_from_conn` explicitly specifies the failed classification type (e.g. `Incomplete: missing core tables`).

### Opportunities (O)
- **O1: Shared helper for other-key test fixtures (F33 / AC2):** Create a clean test utility for generating valid keyed SQLite backups with distinct keys and schemas to share across hermetic CLI and brain test suites.
- **O2: Documentation on `is_usable_class` post-creation invariant:** Add doc comments on `run_backup_from_conn` noting that newly created backups are guaranteed to satisfy `doctor` usable requirements.

---

## What Looks Solid

1. **Fail-Closed Invariant:** Enforcing `classify_backup_read` immediately upon backup creation ensures defective or core-table-missing snapshots are never registered as successful.
2. **Immutable Historical Ciphertext:** Treating older backups as immutable records (rather than attempting in-place re-encryption or auto-pruning) strictly adheres to cryptographic best practices and event sourcing principles.
3. **Mixed-Fleet Realism:** Testing with genuine other-key encrypted backups alongside valid current-key snapshots accurately mirrors real-world key rotation scenarios.
4. **Hotspot Restraint:** Zero changes to `project.rs`, CLI `preflight.rs`, `sync.rs`, or `doctor.rs` logic. Changes are isolated to `crates/ai-brains-brain/src/backup.rs` and CLI test suites.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| 22/22 backup FAIL; no usable encrypted file | Absorbed into DoD (F1–F7 / AC1–AC7) | Solved via fail-closed post-create gate + mixed-fleet lock |
| T225 operator still runs live `backup create` | Absorbed (F4 / AC7) | Addressed via live `--no-prune` runbook on go |
| T244 2026-08-12 Readable now KeyMismatch | Absorbed regression (F5/F6) | Re-snapshot under current key is standard NIST recovery |
| Auto-delete / quarantine of 21 residuals | Declined (F5 / F14) | Preserves historical snapshots |
| Rekey / transcode old `.bak` | Declined (F5) | Preserves immutable ciphertext |
| Restore daemon gate on create | Declined (F9 / F35) | Create is copy-out; probe remains restore-only |
| PR #188 Bugbot Mediums | Declined (F284) | Properly tracked in Track T284 |
| Last-PR Cursor #191 | N/A (empty) | Scanned with 0 findings |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#191](https://github.com/Ryan-AI-Studios/AI-Brains/pull/191) (merged 2026-08-22, T276 `Leftover --global prefer-fill and pretty tags`).
- **Cursor Comments:** 0 comments (`[]` on PR #191).
- **Disposition:** N/A (no pending findings).

---

## Research / Tools Notes

- **Backup Integrity Standards:** NIST SP 800-57 Part 1 Rev 5 and CISA zero-error verified recovery emphasize periodic active-key verification and re-snapshotting following key rotation.
- **Dependencies:** `clap` (4.6.1), `serde_json` (1.0.150), `rusqlite` (0.39.0), `chrono` (0.4.44), `uuid` (1.23.1).
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,376 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search run_backup_from_conn`: Located at `crates/ai-brains-brain/src/backup.rs:146`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
