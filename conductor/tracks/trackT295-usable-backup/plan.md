# T295 Plan — live usable encrypted backup (current key)

**Status:** **Pending** (Planned; F0 until **go**). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F36 / AC1–AC15
**Category:** OPS / RECOVERY / UX
**Ledger TX (planning):** `37c18651-f942-4732-afca-31b5e6269134` (DOCS)
**Ledger TX (implement):** FEATURE TX on **go**

---

## Preflight (plan time — 2026-08-24)

| Check | Result |
|-------|--------|
| HEAD / tree | `56d905a` T294 squash `#210`. `main` = `origin/main`. CLEAN. |
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
| `preflight --summary` | Pinned **4059**. In-context **0/0/0**. Word **305** |
| Last PR comments | #210 T294 — Cursor/Bugbot/reviews/issue **empty**. **N/A. No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; GitHub **v4.6.6**; **no clap 5**); rusqlite **0.39.0** (0.40.2); chrono **0.4.44**; serde_json **1.0.150**; thiserror **2.0.18**; tokio **1.52.3** — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** (**3.915**) — do not touch. `doctor.rs` — do not touch. `backup.rs` not top-10. |
| Ledger | 0 pending / 0 drift at scan; planning TX `37c18651` |
| `ISSUES.md` | **Does not exist** (F22) |
| Online | SQLCipher 4.3.0+ encrypted-to-encrypted Online Backup; rusqlite 0.39.0 `Backup::new` + `run_to_completion`; sqlite.org/backup.html 2025-11-13; clig.dev default-right + tell-user + `--dry-run`; `sqlcipher_export` is migrate (T187) |
| Skill | CAPABILITIES §11 + OPERATIONS `:749` + Create after_help |
| doctor (ledgerful) | **4** warn. :8083 **ok**; :8081 unreachable at doctor (volatile vs daemon status Open) |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [ ] Re-read `run_backup_from_conn` — still `drop(dst)` **`:227`** then classify; **do not edit** unless compile forces (then stop)
- [ ] Confirm clap Create still `:3148–3164` **no** after_help; dispatch `:4790` `keep.or(Some(10))` unless `--no-prune`
- [ ] Confirm doctor `check_backup_recent` still zero-usable remediator `ai-brains backup create` (`:370–375`) — **do not grow `doctor.rs`**
- [ ] Confirm T277 tests still in `backup_recoverable.rs` — stay-green AC1–AC4
- [ ] Re-dogfood: 22 files? T244 still `(unreadable key)`? dry-run `--no-prune` size (F28 volatile); default keep still would prune N
- [ ] Confirm `#210` still empty Cursor; no mint; Dependabot `#61` still not this track
- [ ] Rescan `conductor/deferred.md` — T295 absorbed; T296–T300 / T277 F2 / T240 F2 / T277 F8 not stolen
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0** — **no bump**
- [ ] FEATURE TX (new)
- [ ] Did **not** `cargo install`; did **not** live create until owner confirm; did **not** grow `project.rs` / `doctor.rs`

---

## Absorbed deferred

- [ ] 0 usable encrypted backup / 22/22 FAIL / doctor warn → F2–F8 / AC5–AC8
- [ ] Placeholder Manual `--no-prune` + list + verify + doctor → AC8
- [ ] T277 closeout live skip / T225 operator still runs create → F2 / F3
- [ ] CAPABILITIES green path omit `--no-prune` / `--output-dir` vs doctor dir → F6 / F8
- [ ] last-PR #210 Cursor N/A → F25 no T301

## Declined (written)

- [ ] T277 F2 engine reopen / mixed hermetic steal
- [ ] Doctor remediator `--no-prune` / grow `doctor.rs` (F7)
- [ ] Default keep-10 change (F4)
- [ ] Rekey / transcode T244 `.bak` (F5)
- [ ] T277 F20 remaining_count / T244 F17/F18 / T187 cipher_integrity_check
- [ ] Restore / create daemon probe / nightly auto-create / offsite
- [ ] T294 leftover `--write` / T296–T300 / T240 F2 / clap 5 / rusqlite 0.40 / H2 / 750 ms
- [ ] Completing on hermetic-only without live file (F2)

---

## Phase 1 — Red (TDD)

- [ ] `backup_create_help__after_help__mentions_no_prune_default_dir` (AC5) — **must fail** before after_help exists
- [ ] Commit red allowed

---

## Phase 2 — Green

- [ ] clap Create `after_help` (F6 / §5.1) — no new flags (AC6)
- [ ] CAPABILITIES §11 + OPERATIONS Backup extend + CHANGELOG T295 (AC7)
- [ ] Optional RECOVERY-DRILLS one-liner
- [ ] Commit green allowed
- [ ] **Do not** edit brain `backup.rs` / CLI `backup.rs` production / `doctor.rs`

---

## Phase 3 — Stay-green + docs

- [ ] AC1 brain missing-cores
- [ ] AC2–AC4 T277 mixed fleet
- [ ] AC12 smoke `Backup created and verified:`
- [ ] AC13 T244 list honesty + doctor all-incomplete
- [ ] AC10 T188 restore daemon; grep create has no `probe_restore_daemon_busy`
- [ ] AC9 no unwrap production; lockfile pins; `doctor.rs` diff empty
- [ ] AC11 capture independence
- [ ] AC14 no leftover UUID in `--help`

---

## Phase 4 — Live Manual (owner-confirm only)

- [ ] Owner confirmed live `backup create --no-prune` in the go prompt
- [ ] Dry-run once more; record size/path
- [ ] `ai-brains --no-project-context backup create --no-prune` (no `--output-dir`, no `--keep`)
- [ ] `backup list --quiet` — new Readable first
- [ ] `backup verify` — ≥1 OK; exit 1 OK; **no** create nudge
- [ ] `doctor --format json` — `backup_recent` not `no usable encrypted backup under current key`
- [ ] Residual files still present (22 + 1)
- [ ] If owner **did not** confirm: **do not** create; **do not** mark Completed (F2b); record skip in `review.md`

---

## Phase 5 — Review + gate + publish

- [ ] Phase-1 review → `conductor/tracks/trackT295-usable-backup/review.md`
- [ ] Cross-model FEATURE (`codex-review`) — F27
- [ ] `cargo fmt --check` ; clippy workspace `-D warnings` ; nextest workspace ; `cargo deny check` ; `cargo audit`
- [ ] `ledgerful verify --scope full`
- [ ] conductor **Completed** only if F2a (live file); deferred closeout table
- [ ] Pin `DECISION:` (live `--no-prune` + default sibling dir; T277 engine frozen; doctor remediator unchanged)
- [ ] implement-track Phase 6 **only if Completed**: push `track/T295-*`, PR, watch GHA `CI` green, squash-merge, prune. Never `git push origin main`. Never force-push.

---

## DoD (checkable)

- [ ] Hermetic T277 mixed still: new Readable + residual KeyMismatch + doctor ok + verify 1 OK/1 FAIL exit 1 no nudge
- [ ] `backup create --help` mentions `--no-prune` + residual timestamp-not-class + default `backups/` / doctor
- [ ] CAPABILITIES + OPERATIONS live remediator `--no-prune` + default dir (not `--output-dir`)
- [ ] Live (if owner confirmed): ≥1 usable backup under current key; doctor `backup_recent` not zero-usable warn; verify ≥1 OK
- [ ] No live create unless owner confirmed; no prune of residuals; no restore
- [ ] No `doctor.rs` / `project.rs` / engine production edit
- [ ] F0 was respected (no product commits as planning)

---

## Isolation recap

Do **not** `cargo install`. Do **not** live create until go+owner. Do **not** grow `doctor.rs` / `project.rs`. Do **not** reopen T277 F2. Do **not** change keep-10 default. Do **not** steal T296–T300 / T294 leftover `--write`.
