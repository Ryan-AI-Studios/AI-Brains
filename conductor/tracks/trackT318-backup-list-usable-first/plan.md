# T318 Plan — backup list usable-first (collapse residual fleet)

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** UX / OPS
**Ledger (planning):** DOCS `156b2a03-b5aa-4905-b840-d14fb182aa90`
**Ledger (fold-in):** DOCS `5f4aace2-b78d-4757-961f-12bc2366f5b3`

---

## Preflight (plan time — 2026-08-29)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `93a788a` plan commit CLEAN; `origin/main` = `ed2f5f8` (ahead **1**). Plan-write was `ed2f5f8` / ahead **0** (Agy m1). Branch `track/T318-backup-list-usable-first`. Product `src/` = T316 `#240`. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T244/T225/T295 **on PATH**. T316 **not**. Hole **is** (full residual table + F6 stderr + verify first-5 FAIL on mixed). |
| `preflight --summary` (PATH) | Pinned **4601**; in-context **0/0/0**; `Total Word Count: 802` (PATH-behind T315) |
| PATH `backup list` | Header + **23** files; first row T295 Readable `vault-2026-08-24T10-01-54.db.bak`; **22** residual rows; stderr F6 `22 backup(s) not recoverable…`. Exit 0 |
| PATH `backup verify` | `Verified 23 backups: 1 OK, 22 FAIL.` + 5 `FAIL —` + `… and 17 more FAIL`. Exit **1**. No create nudge |
| `run_list` | `backup.rs:163–229` prints all rows then F6 stderr |
| `run_verify` default | `:377–405` always `format_fail_preview` |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; serde_json **1.0.150**; uuid ws `"1.13"` / lock **1.23.1**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#240` empty (overview). `#239` empty. `#237` → **T326**. `#230` → **T325**. |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan |
| Hotspots | `project.rs` #1 **3.665** — do not touch. `forget.rs` #5 — do not grow production. `doctor.rs` **1738** nonblank — do not grow. CLI `backup.rs` not in top 10. |
| Line counts | CLI `backup.rs` **847** nonblank / **936** physical; `verify_report.rs` **139**; brain `backup.rs` **1254**. F22 = go-HEAD diff. |
| `ISSUES.md` | **Does not exist** |
| Planning install / live create | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit backup list 6/6 residual noise | **DoD** F1–F5 / AC1–AC5 / AC17 |
| T244 F7 sort already usable-first | **Freeze** F6; emit collapse is the hole |
| T244 F6 stderr | **Move to stdout** F2 |
| T225 first-5 FAIL | **Partial** F5 — keep zero-OK; drop on mixed |
| T225/T244 F17 verify `--quiet` / JSON summary | **Decline** F13 |
| T244 F18 prune/archive | **Decline** F13 |
| T277 engine / doctor remediator / keep-10 | **Freeze** F10 |
| T295 ≥1 usable + mixed exit 1 | **Affirm** |
| T316 stderr analog | **F2/F30** move F6 |
| OpenCode m1 Default-mode flip census | **F31** / Phase 0 names `:82/:164/:336/:394/:430` |
| OpenCode m2 AC5 all-plain quiet | **AC5** mixed-quiet new fixture; **AC20** all-residual quiet + dual-flag |
| OpenCode O1 mixed trailer helper | **F9** `format_mixed_fail_trailer` + unit |
| OpenCode O2 list empty untested | **AC6** named hermetic |
| Agy m1 HEAD | **§2.1** `93a788a` / ahead **1** |
| Agy m2 empty vs residuals-only | **Already** F4 / AC3 / AC6 |
| last-PR `#240`/`#239` | **N/A empty** |
| last-PR `#237` / `#230` | **T326** / **T325** — not stolen |
| T321–T324 / clap 5 | **Not stolen** / **Decline** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains`
- [ ] Re-read `run_list` `:163–229` + F6 `:222–228` + `list_sort_key` `:156`
- [ ] Re-read `run_verify` JSON/verbose/default arms `:349–405`
- [ ] Re-read `verify_report.rs` cap/counts/preview/nudge
- [ ] Re-read `is_usable_class` / `residual_for_summary` / `ListMode::from_flags` (import only; do **not** edit brain production)
- [ ] Re-read hermetics `backup_list_honesty.rs` mixed/AC1–AC5/AC20 + F31 census (`:82/:164/:336/:394/:430`) + `backup_recoverable.rs` list + `smoke.rs` mixed verify
- [ ] Re-dogfood `backup list` (stdout vs stderr) + `backup verify` default (source)
- [ ] Record live N = count of `vault-*.db.bak`
- [ ] Confirm clap lock still **4.6.1**; doctor remediator still `ai-brains backup create`
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] Confirm T325 / T326 / T321 still Pending (do not steal)
- [ ] `ledgerful ledger start T318-backup-list-usable-first --category FEATURE`
- [ ] **Do not** `cargo install` / live `backup create` / prune / restore / `.env` rewrite / clap 5 / grow `doctor.rs`

## Phase 1 — Red

- [ ] `backup_list_honesty__mixed_usable_and_residual__usable_first` asserts Default omits residual tokens (AC1) — must **fail** today
- [ ] Footer on stdout / absent stderr (AC2) — must **fail** today
- [ ] `backup_list__all_residual__no_usable_and_footer` (AC3)
- [ ] F31 census: flip Default-mode token/stderr asserts in `__plain_unset_rust_log…` / `__two_plain…` / `__large_key_mismatch…` / `__incomplete…` / `__incomplete_default_rust_log_warn…` (same commit as AC1)
- [ ] `backup_list_honesty__quiet_mixed__usable_row_no_footer` (AC5) — mixed fixture + `--quiet`
- [ ] `backup_list__empty__no_backups_found_exit_0` (AC6)
- [ ] Update `__quiet__no_summary` + `__quiet_and_verbose__quiet_wins` for all-residual quiet (AC20)
- [ ] `backup_list_help__after_help__names_usable_only_and_verbose` (AC14)
- [ ] `backup_verify_all__mixed__reports_per_file` asserts **no** `FAIL —` (AC8) — must **fail** today
- [ ] `format_mixed_trailer__contains_verbose_and_count` (AC8 / F9)

## Phase 2 — Green

- [ ] Default/Quiet: print usable rows only; residuals-only → `No usable backups.`
- [ ] Default footer `println!` of current F6 sentence; delete `eprintln!`
- [ ] Verbose: all rows, no footer (T209 WARNs stay)
- [ ] `format_mixed_fail_trailer` in `verify_report.rs`; `run_verify` human default: `format_fail_preview` only when `ok == 0`; mixed → counts + helper
- [ ] List `after_help` one additive sentence
- [ ] F31 same-commit flips; keep doctor mixed Ok

## Phase 3 — Stay-green + docs

- [ ] `list_sort_tests`; T198 empty **verify**; AC6 list empty; T225 zero-OK 5-FAIL; verbose mixed stream; JSON verify; T295 create help
- [ ] CAPABILITIES §11; OPERATIONS list/verify; CHANGELOG
- [ ] AC16 empty diff `doctor.rs` / brain `backup.rs` / `project.rs` / `forget.rs` production

## Phase 4 — Manual + gate + publish

- [ ] AC17 `cargo run -p ai-brains-cli -- backup list` + `backup verify` (record N / stdout / exit)
- [ ] Targeted `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` ; nextest backup hermetics + smoke verify
- [ ] FEATURE cross-model (`codex-review`)
- [ ] Full gate; conductor Completed; deferred residuals; implement-track Phase 6 (push `track/T318-*`, PR, watch CI, squash-merge). **Never** `git push origin main`

---

## DoD (checkable)

- [ ] Default `backup list` table = usable rows only
- [ ] Residual summary on **stdout** with `not recoverable under current key`; stderr omits it
- [ ] `--verbose` lists every class; `--quiet` usable-only without footer
- [ ] Empty dir still `No backups found.`; residuals-only `No usable backups.`
- [ ] Mixed `backup verify` default: counts + `--verbose` trailer; **no** first-5 `FAIL —`; exit 1; no nudge
- [ ] Zero-OK verify still 5 `FAIL —` + nudge
- [ ] JSON verify `results[]` unchanged
- [ ] Brain `list_backups` / doctor remediator / classify unchanged
- [ ] after_help names the dual-truth
- [ ] Status stays Pending until go; Completed only after merge hygiene

## Isolation

No live prune/create/restore as this track. No `cargo install`. Never `git push origin main`. No product commits as planning.
