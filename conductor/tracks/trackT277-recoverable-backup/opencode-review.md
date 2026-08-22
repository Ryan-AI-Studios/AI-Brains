# Track review: T277-RecoverableBackup

**Harness:** OpenCode (`opencode`)
**Track:** `conductor/tracks/trackT277-recoverable-backup`
**Date:** 2026-08-21
**HEAD:** `c51d673` (docs(conductor): plan T277 current-key recoverable backup; tree CLEAN)

## Summary

Plan audit of T277 (OPS/FEATURE/UX, owner Grok, status Pending/Planned, F0 = plan-only until **go**). The plan is well-scoped, honest about the live hole (22/22 residual fleet, T244 2026-08-12 file regressed to KeyMismatch), and correctly keeps the fix minimal: post-create `classify_backup_read` fail-closed in `run_backup_from_conn` (F2) plus a hermetic mixed-fleet lock (F3/F28) and docs. Every code anchor, pin, test name, and deferred/last-PR disposition in the plan was verified against live `src/`, `Cargo.lock`, and GitHub. One `m` finding (Windows file-lock ordering in F2) and one low-info drift note. Verdict: **Planned**.

## Findings (B/M/m/O)

### m — F2 must `drop(dst)` before `fs::remove_file` on Windows

- **Where:** `crates/ai-brains-brain/src/backup.rs` `run_backup_from_conn` `:146–218`.
- **Detail:** The plan's F2 insertion point is after the meta insert (ends ~`:216`) and before `Ok(backup_path)` (`:218`). At that point `dst` (`rusqlite::Connection`, opened `:165`) is still alive. On Windows, `fs::remove_file` on a file with an open SQLite handle fails with a sharing violation (SQLite opens files with `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE` only when `SQLITE_OPEN_DELETEONCLOSE`/`SQLITE_OPEN_URI` semantics apply; a plain `Connection::open` does not grant delete sharing). The plan says "best-effort" remove; AC1 asserts "dest path **absent** (deleted)". The implementer must `drop(dst)` (and the `Backup` handle is already out of scope at `:174`) before calling `remove_file`, otherwise the delete arm silently no-ops on Windows and AC1's "absent" assertion fails.
- **Severity rationale:** `m`, not `M` — the primary fail-closed behavior is the `Err` return (never print "verified" for a non-usable file); the delete is secondary. But AC1's red/green assertion depends on it, so it must fold.
- **Suggested fold:** in `run_backup_from_conn`, after meta insert: `drop(dst);` then `classify_backup_read(&backup_path, &self.key)`; if `!is_usable_class(...)` → `let _ = fs::remove_file(&backup_path);` → `Err(...)`. Note `classify_backup_read` opens its own connection (`:457`), so it is safe to call after `drop(dst)`.

### O — AC1 test should assert the delete arm explicitly

- **Where:** planned `run_backup_from_conn__missing_cores__fails_and_deletes`.
- **Detail:** The plan names the test but not its assertions. Given the `m` above, the test should assert both `Err` **and** `!dest_path.exists()` (not just `is_err`), per AGENTS.md "assert specific values". Cheap and on-scope.

### Low-info drift (not a finding)

- Live `backup create --dry-run --no-prune` on this machine now estimates **122,560,512** bytes vs the plan's **122,433,536** (vault grew ~127 KB since the plan's 2026-08-22 scan). Relative ranking and all conclusions unchanged; re-dogfood at Phase 0.
- `ledgerful doctor` optional warnings (timings-0 25,419 rows >10k; completion model unreachable) are accelerators only; 0 pending / 0 drift confirmed.

## What looks solid

- **F0 go-gate is real:** plan.md Phase 0 is re-verify-only; no production code, no live create/prune/restore, no `cargo install` until **go**. Phase 4 live create is owner-confirm only, with an explicit skip path recorded.
- **F2 gate site is correct:** `run_backup_from_conn` is the single choke point for all create callers (CLI `run_create` `:98`, `run_backup` `:232`, brain tests `:799/:837`). Verified `classify_backup_read` `:440` and `is_usable_class` `:33` (`Readable | PreT109` only). AC1's red claim is valid — today `run_backup_from_conn` does `integrity_check` + meta only, so a junk-only vault create succeeds and classifies Incomplete.
- **AC1 is a genuine fail-first:** the named TDD test does not exist yet; the existing `classify_backup_read__real_backup__readable_with_meta` (`:1090`) is the green arm F2 does not cover.
- **Mixed-fleet lock is hermetic and honest:** other-key SQLCipher fixture (≥512 bytes, not plain) matches production KeyMismatch; `--no-prune` (F28) preserves the residual; AC2/AC3/AC4/AC13 assertions map to real code paths (`list_sort_key` usable-first `:158`, doctor `is_usable_class` filter `:367` + exact message `:373`, verify nudge `should_emit_create_nudge` `ok==0 && total>=1` `:48-50` + exit 1 `:407-409`, residual summary `:224-228`).
- **No scope creep:** no new clap flags (Create `:2804-2820` unchanged), no `doctor.rs` growth (F8), no daemon probe on create (F35 — `probe_restore_daemon_busy` is restore-only `:471`), no rekey/transcode, no prune of live residuals, no DTO keys, no pin bumps.
- **Pins verified against Cargo.lock today:** clap **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44**, serde_json **1.0.150** — all match the plan; no bumps. rusqlite 0.39.0 `Backup::new`/`run_to_completion` API confirmed on docs.rs (feature `backup`).
- **Deferred §9 + last-PR Cursor audit complete and current:** #191 (T276) pull/issue/review comments all **0** (N/A, matches plan); #188 = 2 Cursor Bugbot Mediums (Work table hides dispose rows; Apply audit samples prefer inventory) → T284, no T285 mint needed. `ISSUES.md` does not exist (F31). deferred.md rows match the plan's absorb/decline table.
- **Contracts:** no public DTO surface changes (T244 F19 freeze respected); no PROTOCOL-COMPAT impact.

## Deferred fold-in table

| deferred.md row / leftover | Plan disposition | Review |
|----------------------------|-------------------|--------|
| 22/22 backup FAIL; no usable encrypted file | Absorb F1–F7 / AC1–AC7 | Agree — this is the DoD |
| T225 operator still runs live create | Absorb F4 / AC7 | Agree (owner-confirm) |
| T244 live Readable 2026-08-12 regressed | Absorb regression F5/F6 | Agree — new snapshot, no transcode |
| T244 F17/F18 verify quiet/JSON/archive | Decline F14 | Agree (soft) |
| T209 L3 wrong-key fixture | Partial F33 | Agree — mixed other-key is the hard fixture |
| T187 `cipher_integrity_check` | Decline F14 | Agree |
| T181 restore drills / offsite | Decline | Agree |
| Prune dry-run `remaining_count` | Decline F20 | Agree (soft; verified `:333` `exists()` count) |
| last-PR Cursor #191 | N/A (empty) | Verified 0/0/0 |
| last-PR #188 Work/apply samples | Affirm T284 | Verified 2 inline comments still present |

## Last-PR Cursor comments

- **#191 (T276, last merged):** `gh api pulls/191/comments`, `issues/191/comments`, `pulls/191/reviews` all return **0**. N/A — matches plan.
- **#188 (prior):** 2 Cursor Bugbot Mediums confirmed live ("Work table hides dispose rows", "Apply audit samples prefer inventory"). Plan correctly keeps them at **T284**; no new placeholder needed.
- **Open PRs on HEAD:** Dependabot remotes only (rusqlite 0.40.2, chrono 0.4.45, actions). No leftover to mint.

## Research / tools notes

- **Pins (verified vs Cargo.lock + crates.io/docs.rs):** clap 4.6.1 (no clap 5), rusqlite 0.39.0 (0.40.2 exists, not bumped — SQLCipher build risk), chrono 0.4.44, serde_json 1.0.150. All match plan §2.4.
- **rusqlite Backup API (docs.rs 0.39.0):** `Backup::new(from: &Connection, to: &mut Connection)` + `run_to_completion(pages_per_step, pause, progress)` confirmed; `to` is `&mut` and SQLite forbids other API calls on the destination during backup — consistent with the existing `:171-174` usage and F1's "engine unchanged".
- **Zetetic / SQLCipher:** encrypted→encrypted Online Backup API (same key src/dest) is the correct daily-create primitive; `sqlcipher_export` is a migrate path, correctly declined (F5/F29).
- **NIST/CISA:** SP 1339 backup integrity testing + CISA 3-2-1-1-0 "zero verified errors" map to ≥1 verify OK + doctor usable — offsite/immutable correctly not DoD.
- **ai-brains / ledgerful used:** `preflight --summary` (Pinned 3376, in-context 0/0/0, grants 0 of 3, Scope `3581317d`); `ledgerful ledger status --compact` (0 pending / 0 drift); live `backup list --quiet` (22 files, T244 file `(unreadable key)`, 0 Readable); `doctor --format json` (`backup_recent` warn, `ok:false`, exact message + remediator confirmed).
- **Live src opened:** brain `backup.rs` (`:1-40`, `:100-129`, `:146-233`, `:320-349`, `:440-494`, `:593-609`, `:1090`); CLI `backup.rs` (`:36-45`, `:55-104`, `:158-232`, `:269-411`, `:428-487`); CLI `doctor.rs` (`:42`, `:196`, `:330-389`); CLI `main.rs` (`:1174-1201`, `:2804-2843`, `:4440-4459`); `verify_report.rs` (`:40-57`); tests `smoke.rs:1200`, `doctor_cli.rs:837/:1058`. No invented paths or stale flags found.
- **Skipped:** `ledgerful scan --impact` (plan-time impact scan is a Phase 0 go-gate item; this is a read-only plan audit and the touch map is small and fully enumerated in spec §12). `cargo clippy` not run (plan gate, not plan-review gate).

## Verdict: Planned

Plan is accurate against live code, pins, and GitHub state. One `m` fold (F2 `drop(dst)` before `remove_file` on Windows) and one `O` (AC1 test asserts `!exists()` explicitly). No blockers, no DoD holes, no closed-decline reopen, no contract impact.
