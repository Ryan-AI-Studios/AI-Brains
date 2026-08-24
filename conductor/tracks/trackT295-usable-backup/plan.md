# T295 Plan — live usable encrypted backup (current key)

**Status:** **Completed** (F2a live file + after_help + docs; FEATURE `aa31087f`). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F38 / AC1–AC15 + §13 AI fold-in
**Category:** OPS / RECOVERY / UX
**Ledger TX (planning):** `37c18651-f942-4732-afca-31b5e6269134` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `f02074c2-b30c-40f2-9ac4-5c784f960844` (DOCS)
**Ledger TX (implement):** FEATURE TX on **go**

---

## AI fold-in (2026-08-24) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 0**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F37/AC5:** separate `contains` for `--no-prune`, timestamp-not-class, `backups/` or `backup_recent`, example `backup create --dry-run --no-prune`.
2. **F12:** do not grow `src/help_ia.rs` or `tests/cli_help_ia.rs`.
3. **F35:** no `--vault-path`; combined stdout+stderr.
4. **F38/AC8:** Phase 0 **N**; after create **N+1**; list transcript in `review.md`.
5. **F8/AC7:** OPERATIONS list+doctor vs `--output-dir`.
6. **F6:** after_help pointer is AC5 / §5.1 (not AC8).

---

## Preflight (plan time — 2026-08-24; fold-in refresh)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in on `cd9701a` (`main`, T295 plan). Parent `56d905a` T294 `#210`. CLEAN at fold-in start. `origin/main` = `56d905a` until this commit. |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T277.** No T285–T294. Hole is **live file**. **Do not `cargo install`.** |
| Fleet | **22** `vault-*.db.bak`; newest T244 `vault-2026-08-12T15-50-06.db.bak` **`(unreadable key)`**; vault **150437888** bytes |
| `backup verify` | **0 OK / 22 FAIL** + nudge `ai-brains backup create`. Exit **1**. |
| `doctor` `backup_recent` | warn / `ok: false` / `no usable encrypted backup under current key` / remediator `ai-brains backup create` |
| Dry-run `--no-prune` | would write sibling `backups\vault-<now>.db.bak` size **150437888**. **Did not create.** |
| Dry-run default keep-10 | **would prune 12**, remaining **22** (T277 F20 lie) |
| `backup create --help` | flags only; **no after_help** |
| Daemon | status **Stopped**; :8081 Open / :8083 Open; doctor `daemon_reachable` ok — **T297** |
| Disk | C: free **~142.5 GB** |
| Engine | `drop(dst)` brain `backup.rs:227`; clap Create `:3148`; dispatch keep `:4790` |
| `preflight --summary` | Pinned **4101** (volatile; plan 4059 / OpenCode 4090). In-context **0/0/0**. Word **333** (plan 305 / OpenCode 947) |
| Last PR comments | #210 T294 — Cursor/Bugbot/reviews/issue **empty**. **N/A. No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; GitHub **v4.6.6**; **no clap 5**); rusqlite **0.39.0** (0.40.2); chrono **0.4.44**; serde_json **1.0.150**; thiserror **2.0.18**; tokio **1.52.3** — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** (**3.906** fold-in; plan 3.915) — do not touch. `doctor.rs` — do not touch. `backup.rs` not top-10. `src/help_ia.rs` exists; `tests/cli_help_ia.rs` is the T204 lock (F12). |
| Ledger | 0 pending / 0 drift at scan; planning TX `37c18651`; fold-in TX `f02074c2` |
| `ISSUES.md` | **Does not exist** (F22) |
| Online | SQLCipher 4.3.0+ encrypted-to-encrypted Online Backup; rusqlite 0.39.0 `Backup::new` + `run_to_completion`; sqlite.org/backup.html 2025-11-13; clig.dev default-right + tell-user + `--dry-run`; `sqlcipher_export` is migrate (T187) |
| Skill | CAPABILITIES §11 + OPERATIONS `:749` + Create after_help |
| doctor (ledgerful) | **4** warn. :8083 **ok**; :8081 **ok** this pass (plan: unreachable at doctor — volatile) |

---

## Phase 0 — on go (re-verify)

- [x] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [x] Re-read `run_backup_from_conn` — still `drop(dst)` **`:227`** then classify; **do not edit** unless compile forces (then stop)
- [x] Confirm clap Create still `:3148–3164` **no** after_help; dispatch `:4790` `keep.or(Some(10))` unless `--no-prune`
- [x] Confirm doctor `check_backup_recent` still zero-usable remediator `ai-brains backup create` (`:370–375`) — **do not grow `doctor.rs`**
- [x] Confirm T277 tests still in `backup_recoverable.rs` — stay-green AC1–AC4
- [x] Re-dogfood: record **N** = `vault-*.db.bak` count (F38; plan-time 22); T244 still `(unreadable key)`? dry-run `--no-prune` size (F28 volatile); default keep still would prune M
- [x] Confirm `src/help_ia.rs` + `tests/cli_help_ia.rs` still the F12 do-not-grow pair; AC5 stays in `backup_recoverable.rs`
- [x] Confirm `#210` still empty Cursor; no mint; Dependabot `#61` still not this track
- [x] Rescan `conductor/deferred.md` — T295 absorbed; T296–T300 / T277 F2 / T240 F2 / T277 F8 not stolen
- [x] Re-check clap lock **4.6.1**, rusqlite **0.39.0** — **no bump**
- [x] FEATURE TX (new)
- [x] Did **not** `cargo install`; did **not** live create until owner confirm; did **not** grow `project.rs` / `doctor.rs`

---

## Absorbed deferred

- [x] 0 usable encrypted backup / 22/22 FAIL / doctor warn → F2–F8 / AC5–AC8
- [x] Placeholder Manual `--no-prune` + list + verify + doctor → AC8
- [x] T277 closeout live skip / T225 operator still runs create → F2 / F3
- [x] CAPABILITIES green path omit `--no-prune` / `--output-dir` vs doctor dir → F6 / F8
- [x] last-PR #210 Cursor N/A → F25 no T301

## Declined (written)

- [x] T277 F2 engine reopen / mixed hermetic steal
- [x] Doctor remediator `--no-prune` / grow `doctor.rs` (F7)
- [x] Default keep-10 change (F4)
- [x] Rekey / transcode T244 `.bak` (F5)
- [x] T277 F20 remaining_count / T244 F17/F18 / T187 cipher_integrity_check
- [x] Restore / create daemon probe / nightly auto-create / offsite
- [x] T294 leftover `--write` / T296–T300 / T240 F2 / clap 5 / rusqlite 0.40 / H2 / 750 ms
- [x] Completing on hermetic-only without live file (F2)

---

## Phase 1 — Red (TDD)

- [x] `backup_create_help__after_help__mentions_no_prune_default_dir` (AC5 / F37) — **must fail** before after_help exists: separate `--no-prune` / timestamp-not-class / `backups/` or `backup_recent` / `backup create --dry-run --no-prune`; no `--vault-path` (F35)
- [x] Commit red allowed

---

## Phase 2 — Green

- [x] clap Create `after_help` (F6 / §5.1) — no new flags (AC6); examples include `--dry-run --no-prune`
- [x] CAPABILITIES §11 + OPERATIONS Backup extend (Agy O1 list+doctor vs `--output-dir`) + CHANGELOG T295 (AC7)
- [x] Optional RECOVERY-DRILLS one-liner
- [x] Commit green allowed
- [x] **Do not** edit brain `backup.rs` / CLI `backup.rs` production / `doctor.rs` / `src/help_ia.rs` / `tests/cli_help_ia.rs`

---

## Phase 3 — Stay-green + docs

- [x] AC1 brain missing-cores
- [x] AC2–AC4 T277 mixed fleet
- [x] AC12 smoke `Backup created and verified:`
- [x] AC13 T244 list honesty + doctor all-incomplete
- [x] AC10 T188 restore daemon; grep create has no `probe_restore_daemon_busy`
- [x] AC9 no unwrap production; lockfile pins; `doctor.rs` diff empty
- [x] AC11 capture independence
- [x] AC14 no leftover UUID in `--help`

---

## Phase 4 — Live Manual (owner-confirm only)

- [x] Owner confirmed live `backup create --no-prune` in the go prompt
- [x] Dry-run once more; record size/path
- [x] `ai-brains --no-project-context backup create --no-prune` (no `--output-dir`, no `--keep`)
- [x] `backup list --quiet` — new Readable first; **N+1** files (F38); paste transcript in `review.md`
- [x] `backup verify` — ≥1 OK; exit 1 OK; **no** create nudge (Agy m2 / F14)
- [x] `doctor --format json` — `backup_recent` not `no usable encrypted backup under current key`
- [x] Residual files still present (**N** old + 1 new)
- [x] If owner **did not** confirm: **do not** create; **do not** mark Completed (F2b); record skip in `review.md`

---

## Phase 5 — Review + gate + publish

- [x] Phase-1 review → `conductor/tracks/trackT295-usable-backup/review.md`
- [x] Cross-model FEATURE (`codex-review`) — F27
- [x] `cargo fmt --check` ; clippy workspace `-D warnings` ; nextest workspace ; `cargo deny check` ; `cargo audit`
- [x] `ledgerful verify --scope full`
- [x] conductor **Completed** only if F2a (live file); deferred closeout table
- [x] Pin `DECISION:` (live `--no-prune` + default sibling dir; T277 engine frozen; doctor remediator unchanged)
- [x] implement-track Phase 6 **only if Completed**: push `track/T295-*`, PR, watch GHA `CI` green, squash-merge, prune. Never `git push origin main`. Never force-push.

---

## DoD (checkable)

- [x] Hermetic T277 mixed still: new Readable + residual KeyMismatch + doctor ok + verify 1 OK/1 FAIL exit 1 no nudge
- [x] `backup create --help` (no vault) separate locks: `--no-prune` + timestamp-not-class + `backups/` or `backup_recent` + `--dry-run --no-prune` example (F37)
- [x] CAPABILITIES + OPERATIONS live remediator `--no-prune` + default dir (not `--output-dir`)
- [x] Live (if owner confirmed): ≥1 usable backup under current key; doctor `backup_recent` not zero-usable warn; verify ≥1 OK
- [x] No live create unless owner confirmed; no prune of residuals; no restore
- [x] No `doctor.rs` / `project.rs` / engine / `src/help_ia.rs` / `tests/cli_help_ia.rs` production edit
- [x] F0 was respected (no product commits as planning)

---

## Isolation recap

Do **not** `cargo install`. Do **not** live create until go+owner. Do **not** grow `doctor.rs` / `project.rs` / `src/help_ia.rs` / `tests/cli_help_ia.rs`. Do **not** reopen T277 F2. Do **not** change keep-10 default. Do **not** steal T296–T300 / T294 leftover `--write`.
