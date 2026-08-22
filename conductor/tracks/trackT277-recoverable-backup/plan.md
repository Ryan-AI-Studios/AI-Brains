# T277 Plan — Recoverable encrypted backup under the current key

**Status:** **Completed** 2026-08-22 (hermetic F2; live create skipped)
**Spec:** [spec.md](./spec.md) F0–F44 / AC1–AC15 + §13 AI fold-in
**Category:** OPS / FEATURE / UX
**Ledger TX (planning):** `58645bcf-c537-4907-807e-87d63e028fea` (DOCS)
**Ledger TX (fold-in):** `29398ea9-d46c-4f6b-b3bf-37bc6d7c69b7` (DOCS)
**Ledger TX (implement):** start **FEATURE** on **go**

---

## AI fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F42:** `drop(dst)` after meta, then classify, then `remove_file` (Windows sharing).
2. **F43 / AC1:** `is_err()` + `!exists()` + `Incomplete` + `core tables`.
3. **F2 rustdoc:** post-create path is `is_usable_class` (Agy O2).
4. **F44:** other-key helper is file-local (Agy O1 partial).
5. **F36:** vault size volatile; Phase 0 re-dogfood.
6. **Decline:** shared crate test_support; `let _ = remove_file` after F42.

---

## Preflight (plan time — 2026-08-22)

| Check | Result |
|-------|--------|
| HEAD / tree | **Plan dogfood:** `5ece8d5` T276 `#191`. **This fold-in:** `c51d673` (plan docs; product crates identical). CLEAN |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-21 05:55. **T270** on PATH. Backup classify is T244-era. **Do not `cargo install`.** |
| `preflight --summary` | Pinned **3428** this fold-in (plan 3376; volatile). In-context 0/0/0; grants **0 of 3**; Scope `3581317d` |
| Fleet | **22** `backups/vault-*.db.bak`. T244 `vault-2026-08-12T15-50-06.db.bak` **78 MB** exists |
| `backup list --quiet` | T244 file **`(unreadable key)`**; rest legacy plain / `(no core tables)`; **0** Readable |
| `doctor` `backup_recent` | warn `no usable encrypted backup under current key` rem=`ai-brains backup create` |
| `backup create --dry-run --no-prune` | Size **volatile** (plan 122433536 → OpenCode 122560512 → fold-in **122953728**). Re-dogfood Phase 0 |
| Default dry-run keep 10 | Would prune **12**; printed remaining **22** (`exists()` — F20) |
| Daemon | **Stopped** |
| Last PR comments | #191 T276 — **empty** (N/A). #188 Mediums stay **T284**. No T285 |
| Open PR on HEAD | none (Dependabot remotes only) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); rusqlite **0.39.0** (0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** (3.971) — do not grow. `preflight.rs` #7 — do not grow. `doctor.rs` **1738** — do not grow. Brain `backup.rs` ~1169; CLI `backup.rs` ~847; `main.rs` **4691** |
| Ledger | 0 pending / 0 drift at scan; planning TX `58645bcf` |
| `ISSUES.md` | **Does not exist** (F31) |
| ledgerful search | `run_backup_from_conn` brain `:146` / CLI `:98`; `classify_backup_read` `:440`; Readable unit `:1090` |
| Online | Zetetic Backup API encrypted-to-encrypted same key; rusqlite `Backup::run_to_completion`; NIST SP 1339 integrity testing; NIST 800-57 re-backup after rotation; CISA 3-2-1-1-0 “zero verified errors” → ≥1 OK **not** offsite; clap 4.6.6; rusqlite 0.40.2 **not** bumped |

---

## Phase 0 — on go (re-verify)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [x] Re-read `run_backup_from_conn` (`backup.rs` ~`:146–218`), `classify_backup_read` ~`:440` (own conn `:457`), `is_usable_class` ~`:33`. F42: `dst` still live at `:218` today — `drop(dst)` is the green insert
- [x] Re-read CLI `run_create` ~`:55`, clap `Create` ~`:2804`, `effective_keep` ~`:4443`
- [x] Re-read `check_backup_recent` ~`:330` (do **not** edit `doctor.rs`)
- [x] Confirm T244 file still `(unreadable key)`; fleet still 22; doctor still `no usable…`
- [x] Rescan `conductor/deferred.md` — T277 rows already absorbed; no new overlapping open rows
- [x] Confirm #191 comments/reviews still empty (N/A); #188 Mediums stay T284; no mint
- [x] Re-dogfood `--dry-run --no-prune` only. **Did not** live create unless owner confirms
- [x] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [x] FEATURE TX on go (`e877fd0d-7573-49c1-b76e-758b99116e41`)
- [x] Did **not** `cargo install`; did **not** grow `doctor.rs` / `project.rs`; did **not** restore; did **not** prune live residuals

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit 22/22 FAIL; doctor no usable | **DoD** F1–F7 / AC1–AC7 |
| T225 operator still runs live create | **DoD** F4 / AC7 (owner confirm) |
| T244 2026-08-12 Readable now KeyMismatch | **DoD** F5/F6 + new snapshot |
| T209 L3 other-key fixture | **Partial** F33 — mixed fleet hard |

---

## Declined (written)

| Item | Why |
|------|-----|
| Rekey / transcode T244 `.bak` | F5 |
| Live default `--keep 10` | F4/F19 — would prune 12 |
| Nightly auto-create / offsite 3-2-1-1-0 | F14 |
| Restore / create daemon probe | F9/F35 — T188 restore-only |
| T244 F17/F18 verify quiet / archive | F14 |
| `cipher_integrity_check` | F14 |
| last-PR #191 Cursor | N/A empty |
| #188 Work/samples | T284 |
| leftover 11 roots | T276 |
| clap 5 / rusqlite 0.40 / DTO / `cargo install` | F10 / F16 |

---

## Phase 1 — Red (TDD)

- [x] Brain unit `run_backup_from_conn__missing_cores__fails_and_deletes` (AC1/F43) — **required red** (today create succeeds as Incomplete). Asserts `is_err` + `!exists` + `Incomplete` + `core tables`
- [x] Scaffold mixed-fleet CLI tests (AC2–AC4/AC13) — may lock-pass on create-already-works; AC1 is the fail-first

---

## Phase 2 — Green (F2 + mixed lock)

- [x] `run_backup_from_conn`: after meta, **`drop(dst)`** (F42), `classify_backup_read`, `!is_usable_class` → `remove_file` + Err with class (F2/F34/F43). Rustdoc usable invariant
- [x] File-local `write_other_key_bak` in CLI tests (F44) — not a new crate module
- [x] AC5 Readable unit stays green
- [x] Hermetic other-key `.bak` + `backup create --no-prune` (F28)
- [x] AC2 list Readable first + residual `(unreadable key)`
- [x] AC3 doctor `backup_recent` ok
- [x] AC4 verify `1 OK` `1 FAIL` exit 1, **no** nudge (F41)
- [x] AC13 residual summary `not recoverable under current key`
- [x] AC14 smoke substring `Backup created and verified:`
- [x] No clap flags; no `doctor.rs` growth; no `project.rs`

---

## Phase 3 — Docs

- [x] CAPABILITIES §11: current-key create after KeyMismatch / KEY change (F6/F24)
- [x] OPERATIONS Backup green path + T244 filename as exhibit (no key)
- [x] CHANGELOG T277
- [x] Soft: one RECOVERY-DRILLS line if cheap

---

## Phase 4 — Live (owner confirm only)

- [ ] Owner says yes to mutating `backup create --no-prune`
- [ ] Run create; list; verify; doctor json `backup_recent`
- [ ] Paste exact outputs into Manual evidence below
- [x] If owner declines: hermetic AC1–AC6/AC13 is still DoD; AC7 recorded as skipped with reason (`/implement-track 277` did not confirm live create)

---

## Phase 5 — Review + gate + publish

- [x] Internal review vs spec; F2 data-safety
- [x] Cross-model **hard** on F2 (F25) — Codex CX1 **PASS** (no P0–P3)
- [x] Targeted nextest: `-p ai-brains-brain backup` (28); `-p ai-brains-cli --test backup_recoverable --test backup_list_honesty` (16); `--test doctor_cli --test smoke --test recovery_drills` (118)
- [x] `cargo clippy -p ai-brains-brain -p ai-brains-cli --all-targets -- -D warnings`
- [x] Full gate at closeout: `dev-check.ps1` SUCCESS nextest **3270** passed / 1 skipped; `ledgerful verify --scope full` exit 0
- [ ] implement-track Phase 6: push `track/T277-*` → PR → watch GHA `CI` → squash-merge → prune. Never `git push origin main`

---

## Manual evidence (fill on go)

```text
Binary: PATH ai-brains 0.1.1 (T270-era; F16 no cargo install). Hermetic ACs use cargo test bin.
PRE list: T244 vault-2026-08-12T15-50-06.db.bak (unreadable key); 22 residual
PRE doctor backup_recent: warn, no usable encrypted backup under current key, rem=ai-brains backup create
PRE dry-run --no-prune: C:\dev\ai-brains\backups\vault-2026-08-22T02-30-51.db.bak, size 123043840 (F36)
Daemon: Stopped
CREATE: skipped — owner did not confirm live backup create --no-prune
POST list / verify / doctor: N/A (AC7 skip; hermetic AC1–AC6/AC13 is DoD)
```

---

## DoD

- [x] AC1 red-then-green (fail-closed Incomplete create; F42 drop + F43 three asserts)
- [x] AC2–AC4/AC13 mixed KeyMismatch + create hermetic
- [x] AC5/AC6/AC10/AC14 stay green
- [x] AC7 live create **or** explicit owner skip in this file (skipped — `/implement-track 277` did not confirm)
- [x] AC8 docs; AC9 no pin/DTO bumps
- [ ] Conductor **Completed** only after merge + hygiene
- [x] No live prune of 21 residuals; no restore; no `cargo install`
