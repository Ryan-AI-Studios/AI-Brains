# T277 — At least one usable encrypted backup under the current key

- **Track ID:** T277-RecoverableBackup
- **Status:** **Planned** (Pending until **go**. F0 = plan-only.)
- **Category:** OPS / FEATURE / UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `backup list` **8/8** honest FAIL fleet; `backup verify` **8/9** 0 OK / 22 FAIL; doctor `backup_recent` warn. Placeholder minted with T274–T284.
- **Depends on:** T209 ✅ list honesty; T225 ✅ verify quiet + create nudge; T244 ✅ usable class + live create 2026-08-12; T187 ✅ SQLCipher; T131/T138 verify; T192 doctor; T126 default `--keep 10`; T188 restore daemon hard-fail (do **not** steal onto create)
- **Blocks / feeds:** Doctor `backup_recent` can be ok under the **current** key. T181 drills stay credible. DataKey rotation still expects a verified recent backup.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “22/22 backup FAIL; no usable encrypted file”; T225 residual “Operator still runs `ai-brains backup create` on live encrypted vaults”; T244 AC12 live Readable that **regressed** to KeyMismatch after KEY change
- **Not absorbed (DoD):** Auto-delete / quarantine of 21 residuals; nightly auto-create; rekey/transcode old `.bak`; restore redesign; T188 daemon gate on **create**; `cipher_integrity_check`; verify `--quiet` / JSON `summary` / `VerifyError` (T244 F17); clap archive (T244 F18); rusqlite **0.40+**; clap 5; DTO keys; 3-2-1-1-0 offsite/immutable; `cargo install`
- **Research date:** 2026-08-22 (live dogfood HEAD `5ece8d5` T276 `#191`; product `src/` = T276)
- **AI fold-in:** none yet (plan-track). Disposition lands in **§13** after `/fold-in`.
- **Ledger:** planning DOCS TX `58645bcf-c537-4907-807e-87d63e028fea`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`, rewrite `.env` (T240 F2), live `retention apply --confirm`, mutate schtasks, restore while daemon could start, or prune the live 21 residuals unless the owner confirms. Do **not** grow hotspot `project.rs` / CLI `preflight.rs` / `sync.rs` / `doctor.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Live `backup create` only with owner confirm at **go**.

---

## 1. Objective

1. **The current key has ≥1 doctor-usable encrypted backup.** Honesty is done (T209/T225/T244). Recoverability is not: the 2026-08-12 T244 Readable file is now `(unreadable key)`, the fleet is **22/22 residual**, and doctor `backup_recent` is `no usable encrypted backup under current key` + remediator `ai-brains backup create`.
2. **Create means usable, not just `integrity_check`.** After write + meta, `classify_backup_read` must be `is_usable_class` (Readable with meta on the green path). If not, delete the file and fail — never print `Backup created and verified:` for Incomplete / KeyMismatch.
3. **Key change does not rewrite history.** Old `.bak` files stay KeyMismatch / LegacyPlain / Incomplete. Do **not** transcode. Operator creates a **new** snapshot under the current key (`--no-prune` on live go so T244 F16 stands).
4. **North star.** Capture independence: backup path stays SQLCipher Online Backup API + event-log copy. No models/graph. No hidden CoT. A FAIL-heavy residual fleet with **≥1 OK** is success (verify still exit **1**).

This unblocks the daily product: T244 made class honesty and a green create path; KEY rotation (or `.env` rewrite) silently retired that snapshot. The remediator is already printed. This track **runs it** (hermetic + owner-confirm live) and **locks** the mixed-fleet + fail-closed invariants so the next KEY change cannot claim “verified” without a usable file.

---

## 2. Live baseline (re-scan 2026-08-22)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `5ece8d5` T276 squash `#191`. `main` = `origin/main`. Tree **CLEAN**. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-21 05:55**, 25 368 576 bytes, **0.1.1**. **T270** on PATH (before T274–T276). Backup classify/create are T244-era — **PATH is valid for this hole.** **Do not `cargo install`.** |
| Source debug | Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **3376**. In-context **0/0/0**. Grants **0 of 3** (T275 hermetic; live not bootstrapped). Capture independence holds. |
| Fleet | **22** `vault-*.db.bak` under `C:\dev\ai-brains\backups\`. Newest: `vault-2026-08-12T15-50-06.db.bak` **78 200 832** bytes (T244 live create). Vault now **~122 MB**. |
| `backup list --quiet` | Row 1 = T244 file **`(unreadable key)`**. Mix of `(legacy plain)` + `(no core tables)`. **0** Readable / PreT109. |
| `doctor --format json` `backup_recent` | **warn**, `ok: false`, message `no usable encrypted backup under current key`, remediation `ai-brains backup create`. **Not** the stale-usable arm. |
| `backup create --dry-run --no-prune` | Would write `C:\dev\ai-brains\backups\vault-<now>.db.bak`, source `C:\dev\ai-brains\vault.db`, estimated **122433536** bytes. |
| `backup create --dry-run` (default keep 10) | Same preview + **would prune 12** residuals. Dry-run `remaining_count` prints **22** because `prune_backups` counts `path.exists()` (**F20** — files not deleted on dry-run). Live go uses **`--no-prune`**. |
| Daemon | `daemon status` **Stopped**. T188 restore probe is N/A for create. |
| T244 manual evidence | 2026-08-12 create `--no-prune` → verify `1 OK, 21 FAIL`; doctor ok on that timestamp. **Regressed:** same file is KeyMismatch under today’s key. |
| Last GitHub PR | [#191](https://github.com/Ryan-AI-Studios/AI-Brains/pull/191) T276 (2026-08-22). `gh pr view --comments`, `/reviews`, `/comments`, `issues/191/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, actions). **No leftover to mint.** Prior #188 Bugbot Mediums remain **T284** (2 inline comments, still true). |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081). **0 pending / 0 drift.** Hotspot **#1** `project.rs` (**3.971**). CLI `preflight.rs` **#7**. `backup.rs` (brain ~1169 lines / CLI ~847) **not** top-10. `doctor.rs` **1738** — **do not grow**. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| 22/22 residual / doctor no usable | T244 create **worked**. Current key cannot open that snapshot. Recoverability proof is “file opens with **this** key,” not “a file exists.” **DoD.** |
| Rekey / transcode the T244 `.bak` | Event-sourcing analog: compensating **new** snapshot. Old ciphertext stays. NIST SP 800-57: after key change, re-backup (do not expect old media to open). **Decline rekey.** |
| Default `--keep 10` on live create | Would delete **12** historical residuals (T244 F16 no auto mass-delete). Product default stays T126. Live go **`--no-prune`**. |
| Nightly auto-create / offsite 3-2-1-1-0 | T244 F17 / local-first. CISA 3-2-1-1-0 “zero verified recovery errors” maps to **≥1 verify OK + doctor usable**, not a second disk. **Decline offsite.** |
| Restore / daemon gate on create | T188 is **overwrite**. Create copies out. Live daemon Stopped. **Do not steal.** |
| Verify `--quiet` / JSON summary | T244 F17 / T225 F17 **soft**. Quiet default already exists. |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|--------|
| Create | CLI `backup.rs` `run_create` **`:55–115`** | `run_backup_from_conn`; prints `Backup created and verified:`; default keep via `main.rs` **`:4443`** `keep.or(Some(10))` unless `--no-prune`. |
| Engine | brain `backup.rs` `run_backup_from_conn` **`:146–218`** | `Backup::new` + `run_to_completion(100000, ZERO)`; dest `apply_key_pragmas`; `PRAGMA integrity_check`; T109 `_aibrains_backup_meta`. **No post-write `classify_backup_read`.** |
| Classify | `classify_backup_read` **`:440`** | Header → LegacyPlain; size+key fail → KeyMismatch (≥512) / Corrupt; cores missing → Incomplete; meta ok → Readable. |
| Usable | `is_usable_class` **`:33`** | `Readable \| PreT109` only. |
| List | CLI `run_list` **`:163`** | T244 F7 usable-first **CLI only**. Brain `list_backups` timestamp-desc (doctor). |
| Verify | `verify_single_backup` **`:413`** | Plain refuse; key; quick_check; both cores (`len() < 2`). T225 quiet + nudge when `ok==0 && total>=1`. |
| Doctor | `check_backup_recent` **`:330`** | Zero usable → exact message above + create-only remediator. **Do not grow `doctor.rs`.** |
| Restore daemon | `probe_restore_daemon_busy` **`:471`** | Restore only. Create does **not** call it. |
| Prune remaining | `prune_backups` **`:333`** | `remaining_count = candidates.filter(exists)` — dry-run remaining is **pre-delete** (F20). |
| Brain lock | `classify_backup_read__real_backup__readable_with_meta` **`:1090`** | Create then classify Readable — **unit**, not fail-closed production. |
| CLI lock | `smoke.rs` `backup_verify__valid_backup__reports_ok` **`:1200`**; `doctor_cli.rs` `doctor__backup_recent__readable_within_age__ok` **`:837`** | Same-key hermetic. **No** KeyMismatch residual + create mixed fleet. |
| Dir | `backup_dir` = vault parent `backups/` | T192 F17b list/preview do not `create_dir_all`. |
| Contracts | no public `BackupReadClass` DTO | T244 F19 freeze. |
| Clap | `BackupCommands::Create` **`:2804`** | `--output-dir` / `--keep` / `--no-prune` / `--dry-run`. **No new flags.** |

### 2.4 Dependency / standards research (2026-08-22) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem | Action |
|-----|------------------|-----------|--------|
| `clap` | workspace **4.5**, lock **4.6.1** | crates.io **4.6.6** (GitHub latest **v4.6.6** 2026-08-06). **No clap 5.** | **No bump** |
| `rusqlite` | **0.39.0** + `bundled-sqlcipher-vendored-openssl` + `backup` | crates.io **0.40.2** (0.40.x bundles SQLite 3.53.2) | **No bump** (SQLCipher build risk) |
| `chrono` | lock **0.4.44** | crates.io **0.4.45** | **No bump** |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump** |
| rustc / nextest / workspace | **1.95.0** / **0.9.140** / **0.1.1** | — | — |

**Online (primary sources):**

| Claim | Source | Fit |
|-------|--------|-----|
| Encrypted→encrypted Online Backup API (not mixed plain/cipher) | Zetetic SQLCipher 4.3.0+; `sqlite3_backup_init` both-keyed; [sqlcipher-api](https://www.zetetic.net/sqlcipher/sqlcipher-api/) `sqlcipher_export` is **migrate**, not daily create | Keep `rusqlite::backup::Backup`. Do **not** switch create to `sqlcipher_export`. |
| `Backup::new` + `run_to_completion` | [rusqlite backup.rs](https://github.com/rusqlite/rusqlite/blob/master/src/backup.rs); [sqlite.org/backup.html](https://www.sqlite.org/backup.html) | Already used `:172–173`. WAL writer can restart a step — existing busy_timeout (T96). |
| Recovery **proof** > file exists | NIST SP 1339 (2026-06) backup integrity testing; CISA 3-2-1-1-0 “zero verified errors”; T181 RECOVERY-DRILLS | Doctor usable + verify ≥1 OK. Offsite/immutable **not** DoD. |
| After key rotation, re-backup | NIST SP 800-57pt1r6 backup of keying material; CMMC MP.L2-3.8.9 rotation + re-encrypt | Docs F6. Do not keep old KEY next to `.bak` as a product feature. |
| clig.dev mutate explicit | clig.dev | Live create is `--no-prune` + owner confirm; dry-run already exists. |

### 2.5 Prior track residuals rolled in

| Source | Item | T277 disposition |
|--------|------|------------------|
| deferred.md T274–T284 | 22/22 FAIL; no usable encrypted file | **DoD** |
| T225 closeout | Operator still runs live `backup create` | **Absorb** — this track |
| T244 AC12 / F11 | Live Readable 2026-08-12 | **Absorb regression** — same file KeyMismatch now |
| T244 F17 / T225 F17 | verify `--quiet` / JSON summary / VerifyError | **Decline** (soft) |
| T244 F18 | archive helper | **Decline**; prune recipe stays operator |
| T209 L3 | real wrong-key SQLCipher fixture | **Partial** — mixed-fleet other-key `.bak` is **hard** here; dedicated verify AC9 fixture stays soft |
| T187 | `cipher_integrity_check` on verify | **Decline** |
| T181 | restore drills / offsite | **Decline** — T181 closed; no restore this track |
| last-PR Cursor #191 | empty | **N/A** |
| last-PR #188 | Work / apply samples | **Affirm T284** — no T285 |

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Plan-only** | No production code, no live `backup create` / prune / restore, no `cargo install` until **go**. Dry-run research OK. |
| **F1 — Create engine unchanged** | SQLCipher Online Backup API (`Backup::new` + `run_to_completion`) + dest key pragmas + `integrity_check` + T109 meta. Not `sqlcipher_export`. Same key on src and dest (Zetetic constraint). |
| **F2 — Post-create usable fail-closed (hard)** | After meta insert in `run_backup_from_conn`, `classify_backup_read(&path, &self.key)`. If `!is_usable_class`, `fs::remove_file` (best-effort) and return `Err` naming the class (Incomplete / KeyMismatch / …). Never leave a non-usable file that the CLI will call “verified.” Brain SOOT — CLI print stays `Backup created and verified:` (smoke substring). |
| **F3 — Mixed-fleet lock (hard)** | Hermetic: current-key vault + **other-key** encrypted `vault-*.db.bak` (≥512, not plain) in `backups/` → `backup create --no-prune` → new file **Readable**; residual stays `(unreadable key)`; doctor `backup_recent` **ok** (7d); verify **1 OK, 1 FAIL**, exit **1**, **no** create nudge (`ok >= 1`). |
| **F4 — Live create `--no-prune` (hard on go)** | Mutating only with owner confirm. Prefer `backup create --no-prune`. Record exact path / list first row / verify counts / doctor `backup_recent`. Default keep-10 would prune 12 residuals — **not** the live DoD. |
| **F5 — Old ciphertext stays** | Do **not** rekey, transcode, or `sqlcipher_export` the T244 file. KeyMismatch is honest. |
| **F6 — KEY-change docs** | OPERATIONS + CAPABILITIES: after `AI_BRAINS_KEY` change, old `.bak` are KeyMismatch; run `backup create` then `backup verify` expect ≥1 OK. Cite T244 2026-08-12 file as the exhibit (filename only, **no key**). |
| **F7 — Verify exit frozen** | Any FAIL → exit **1**. Recoverability success = `ok >= 1`, not exit 0 on a residual fleet. Nudge only when `ok==0 && total>=1` (T225 F9). |
| **F8 — Doctor remediator** | Stay `ai-brains backup create` only (T244 F4 / AI2 L5). **Do not grow `doctor.rs`.** Zero-usable message substring frozen. |
| **F9 — No restore / no daemon mutate / no mass-delete** | T188 restore stays. Create does **not** gain a daemon probe (F35). No `backup prune` of live residuals as DoD. No nightly schedule. |
| **F10 — Pins** | No rusqlite 0.40, clap 5, chrono/serde_json bumps, new crates, DTO keys. |
| **F11 — Capture independence** | Backup path stays store/crypto/brain/cli. No models/graph. |
| **F12 — Hotspots** | Do **not** grow `project.rs` / CLI `preflight.rs` / `sync.rs` / `governed_common.rs` / `ranking.rs` / `doctor.rs`. Touch brain `backup.rs` + CLI tests + docs. CLI `backup.rs` only if the print path must surface F2 errors (it already `?`s the engine). |
| **F13 — Daemon vs create** | Live daemon **Stopped**. Create remains allowed if daemon later starts (copy-out, not overwrite). T188 Safety probe stays restore-only. |
| **F14 — Soft declines** | T244 F17/F18; T209 L3 remainder; `cipher_integrity_check`; 3-2-1-1-0 offsite; prune class-aware keep-usable. |
| **F15 — Peers** | T278 graph; T279 Safety; T280 hint; T281 nightly; T282 context leftover; T283 list cwd-first; T284 Work/samples; T276 leftover 11 roots; T275 live bootstrap. |
| **F16 — PATH** | Do not `cargo install`. Hermetic/source bin for ACs. PATH-behind T274–T276 is **irrelevant** to backup classify. |
| **F17 — Secrets** | Never print `AI_BRAINS_KEY` / `--help` env defaults that contain it. |
| **F18 — T240 F2** | No `.env` rewrite. No live `retention apply --confirm`. |
| **F19 — T126 keep 10** | Product default unchanged. Live go `--no-prune`. Hermetic mixed uses `--no-prune` so the KeyMismatch residual **survives** (F28). |
| **F20 — Prune dry-run remaining** | `remaining_count` uses `exists()` → dry-run remaining = pre-delete. **Soft** — not recoverability DoD. Do not “fix” unless it blocks F4. |
| **F21 — No new clap flags** | `--no-prune` / `--dry-run` / `--keep` already exist. |
| **F22 — Success substring** | Keep `Backup created and verified:` (smoke `:1200`). Do not require class token in that line. |
| **F23 — List sort** | Usable-first stays CLI `run_list` only (T244 F7). After F4 the new Readable is top row. |
| **F24 — Docs** | CAPABILITIES §11 green path + KEY-change sentence; OPERATIONS Backup; CHANGELOG T277. Soft RECOVERY-DRILLS one-liner. |
| **F25 — Ledger / review** | On go: FEATURE TX. Primary OPS/FEATURE. Cross-model **hard** on F2 fail-closed (data-safety: false “verified”). |
| **F26 — Exit / doctor matrix** | List 0; verify any FAIL → 1; doctor 15-check; `backup_recent` soft. |
| **F27 — Existing Readable unit** | `classify_backup_read__real_backup__readable_with_meta` stays green. F2 is the **fail** arm that unit does not cover. |
| **F28 — Mixed `--no-prune`** | AC2/AC3/AC4 must not default-prune the other-key residual away. |
| **F29 — Not vault encrypt** | `vault encrypt` / `sqlcipher_export` stay T187. |
| **F30 — Workspace** | `0.1.1`. |
| **F31 — ISSUES.md** | Does not exist. Debt = `deferred.md`. |
| **F32 — Prune is recency** | `prune_backups` is timestamp/keep, not `is_usable_class`. Default keep-10 still keeps the **newest** file (the new OK). Still use `--no-prune` live (F4/F16). |
| **F33 — T209 L3 partial** | Mixed-fleet other-key file is the hard fixture. Do not expand into a full verify-error taxonomy. |
| **F34 — Gate site** | F2 lives in `run_backup_from_conn` (all create callers). |
| **F35 — No create daemon probe** | Do not copy `probe_restore_daemon_busy` onto create. |
| **F36 — Live size** | ~122 MB source. Hermetic stays tiny. |
| **F37 — Backup dir** | Sibling `backups/` of the vault. F17b stands. |
| **F38 — Zero new crates** | — |
| **F39 — T244 honesty frozen** | Incomplete / residual summary / both-cores verify / doctor usable SOOT — do not reopen. |
| **F40 — No compensating rewrite of `.bak`** | Analog of no `MemoryMoved`. New file only. |
| **F41 — AC4 nudge** | Mixed verify with `ok >= 1` must **not** print the create nudge. |

---

## 4. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Source vault with **no** `events`/`memory_projection` (junk table only) → `run_backup_from_conn` **Err**; dest path **absent** (deleted). Required **red** before F2 exists (today: create succeeds, classify Incomplete). | Brain unit |
| **AC2** | Hermetic mixed: other-key SQLCipher `vault-*.db.bak` + `backup create --no-prune` → `backup list` first filename is the **new** file; its row is **not** `(unreadable key)` / `(legacy plain)` / `(no core tables)` (Readable meta columns populated); residual still `(unreadable key)` | CLI hermetic |
| **AC3** | Same fixture: `doctor --json --backup-max-age 7d` → `backup_recent` severity **Ok** (not `no usable encrypted backup under current key`) | CLI hermetic |
| **AC4** | Same fixture: `backup verify` (no path) → stdout has `1 OK` and `1 FAIL`; exit **1**; **no** create-nudge substring; **no** `0 OK` | CLI hermetic |
| **AC5** | `classify_backup_read__real_backup__readable_with_meta` still Readable + meta keys | Brain unit (stay green) |
| **AC6** | T244 `backup_list_honesty` + `doctor__backup_recent__all_incomplete__warn_no_usable` stay green | Regression |
| **AC7** | Live on go (owner confirm): `backup create --no-prune` → list Readable on top; verify ≥1 OK; doctor not `no usable…`. Record commands/outputs. **Planning pass must not create.** | Manual |
| **AC8** | CAPABILITIES §11 + OPERATIONS Backup + CHANGELOG mention current-key create after KeyMismatch / KEY change | Grep |
| **AC9** | No production `unwrap`/`expect`/`panic` on touched paths; lockfile clap/rusqlite/chrono/serde_json unchanged; no DTO keys | Review / diff |
| **AC10** | T188 restore-daemon units stay green; create path still has **no** `probe_restore_daemon_busy` | Grep + nextest |
| **AC11** | Capture independence: no `ai-brains-models` / graph on backup create | Grep |
| **AC12** | Default `--keep 10` clap/wiring unchanged (`main.rs` `:4443`) | Review |
| **AC13** | After mixed create, list residual summary still contains `not recoverable under current key` (the KeyMismatch file) | Hermetic default list (not `--quiet`) |
| **AC14** | Smoke `backup_verify__valid_backup__reports_ok` still finds `Backup created and verified:` | Stay green |
| **AC15** | Full gate at implement closeout only (fmt, clippy -D, nextest workspace, deny, audit) | CI |

Test names (TDD). **Must fail red before F2 exists:** AC1.

- `run_backup_from_conn__missing_cores__fails_and_deletes`
- `backup_create__key_mismatch_residual__new_readable_and_doctor_ok`
- `backup_verify__mixed_ok_and_key_mismatch__one_ok_exit_1_no_nudge`
- `backup_list__mixed_after_create__residual_summary_not_recoverable`

---

## 5. Design notes

### 5.1 Why the T244 file failed

T244 wrote a keyed snapshot and classified it **Readable**. List today tokens it **`(unreadable key)`** (`BackupReadClass::KeyMismatch`). Doctor’s usable filter is empty → “no usable encrypted backup under current key.” That is KEY/cryptoperiod drift, not a classify bug. SQLCipher Online Backup copies pages under the **then-current** key; a later `AI_BRAINS_KEY` cannot open them (Zetetic: backup dest must match src encryption + key). Product fix is a **new** snapshot, not a converter.

### 5.2 Fail-closed vs live hole

F2 would **not** have prevented the 2026-08-12 file from becoming KeyMismatch later. It prevents a different lie: `integrity_check` ok + missing cores (Incomplete) still printing “verified.” AC1 is that arm. AC2–AC4 lock the **operator** path T244 already coded, with a residual that matches production.

Other-key fixture (not random ≥512 garbage): `Connection::open` + `apply_key_pragmas(other)` + `CREATE TABLE junk(x)` + enough rows that `metadata.len() >= 512`. Distinct from T209 `write_large_non_plain` (random bytes). F33.

### 5.3 Live go runbook (not executed in planning)

```powershell
ai-brains --no-project-context backup create --dry-run --no-prune
# owner confirms
ai-brains --no-project-context backup create --no-prune
ai-brains --no-project-context backup list --quiet   # new Readable first
ai-brains --no-project-context backup verify         # ≥1 OK; exit 1 OK
ai-brains --no-project-context doctor --format json  # backup_recent ok
```

Do **not** restore. Do **not** `--keep 10`. Daemon is Stopped; do not start it for this track.

### 5.4 Why not daemon-gate create

Restore overwrites the vault the daemon holds (T188). Create writes a **new** file via the CLI connection (`ctx.conn.lock()` + Backup API). Adding a Safety 3×1s probe would block the remediator whenever the daemon is up. F13/F35.

---

## 6. Non-goals

- Rekey / transcode / `sqlcipher_export` of residual `.bak`.
- Auto-delete, `backups/legacy/` move, clap archive subcommand.
- Nightly scheduled create; offsite / immutable / 3-2-1-1-0 media.
- Restore; recovery export; DataKey rotation.
- Daemon probe on create; doctor 16th check; growing `doctor.rs`.
- verify `--quiet` flag; JSON `summary`; structured `VerifyError`.
- `cipher_integrity_check`; rusqlite 0.40 `table_exists`.
- clap 5; new DTO keys; new crates; `cargo install`.
- T240 F2 `.env`; T275 live bootstrap; T276 leftover `--write --yes`.
- Prune `remaining_count` dry-run honesty (F20); class-aware prune.

---

## 7. Verification plan (TDD)

**Phase 1 red (required before green):** AC1 brain unit (today `run_backup` succeeds on junk-only vault).

Then green: F2 classify+delete in `run_backup_from_conn` → AC5 still green → CLI mixed hermetic AC2–AC4/AC13 (F28 `--no-prune`).

**Stay green:** AC6, AC10, AC14, T244 list honesty, T225 quiet verify, T188 restore daemon.

Targeted: `cargo nextest run -p ai-brains-brain backup` ; `-p ai-brains-cli --test backup_list_honesty --test doctor_cli --test smoke --test recovery_drills` ; `cargo clippy -p ai-brains-brain -p ai-brains-cli --all-targets -- -D warnings`.

Full workspace gate only at implement closeout — **not** a plan gate.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Live create ~122 MB next to vault | F4 `--no-prune` + owner confirm; dry-run already shows path/size |
| Default keep-10 deletes 12 residuals | F4/F19 live `--no-prune`; T126 default stays |
| F2 false-fail on healthy create | AC5 Readable unit; classify after meta; cores from live/init vault |
| Operators think T244 file should reopen | F5/F6 docs; list token already honest |
| Restore while testing | F9; T188 if someone tries anyway |
| `doctor.rs` / `project.rs` growth | F8/F12 |
| PATH-behind T276 | F16; backup hole is T244-era |
| Dry-run remaining 22 vs 10 | F20 documented; not DoD |
| KEY printed in tests | F17; reuse `ZERO_KEY` / `hermetic_with_key` |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-22 (post-P12 through T276 closeout).

| Row / leftover | Disposition |
|----------------|-------------|
| 22/22 backup FAIL; no usable encrypted file | **Absorb** F1–F7 / AC1–AC7 |
| T225 “Operator still runs live `backup create`” | **Absorb** F4 / AC7 |
| T244 live Readable 2026-08-12 | **Absorb regression** F5/F6; new snapshot is DoD |
| T244 F17 / T225 F17 verify quiet/JSON/VerifyError | **Decline** F14 |
| T244 F18 archive | **Decline** F14 |
| T209 L3 wrong-key fixture | **Partial F33** — mixed other-key **hard**; verify taxonomy soft |
| T187 `cipher_integrity_check` | **Decline** F14 |
| T181 restore drills / offsite | **Decline** — T181 closed |
| Prune dry-run `remaining_count` | **Decline F20** (soft) |
| last-PR Cursor #191 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Affirm T284** — 2 inline comments still; no T285 |
| leftover `7d97a456` 11 roots | **Decline → T276 Completed** (live rebind still F9 there) |
| T278 graph / T279 Safety / T280 hint / T281 nightly / T282 context / T283 list | **Decline** peers |
| T240 F2 / T255 750 ms / clap 5 / rusqlite 0.40 | **Decline** |
| Packaging / MSI / `.changeguard` | **Decline** |

---

## 10. Implement order (on go)

1. Phase 0 re-verify `run_backup_from_conn` `:146`, classify `:440`, `run_create` `:55`, clap Create `:2804` / keep `:4443`, doctor `:330`, T244 file still KeyMismatch, 22 files, #191 empty, #188 T284.
2. Red: AC1 missing-cores unit.
3. Green: F2 in `run_backup_from_conn`.
4. CLI mixed hermetic AC2–AC4/AC13 (`--no-prune`).
5. Docs F6/F24. No live create until owner confirms; then AC7.
6. Review loop + FEATURE `codex-review` + full gate. implement-track Phase 6.

---

## 11. Soft residuals

| Residual | Why not DoD |
|----------|-------------|
| Live 21 residual files remain after `--no-prune` | F5/F16; honesty already T244 |
| PATH `cargo install` | F16 |
| Prune dry-run remaining lie | F20 |
| Class-aware prune (keep last usable) | F32; new file is newest |
| verify `--quiet` / JSON summary / VerifyError | T244 F17 |
| Archive / `backups/legacy/` | T244 F18 |
| `cipher_integrity_check` | T187 |
| Offsite / immutable copy | local-first |
| T209 dedicated verify wrong-key AC9 | F33 remainder |
| Create daemon notice if daemon later Running | F13/F35 |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-brain/src/backup.rs` | F2 after meta in `run_backup_from_conn` + AC1 unit |
| `crates/ai-brains-cli/tests/backup_list_honesty.rs` **or** new `backup_recoverable.rs` | AC2/AC4/AC13 mixed fleet |
| `crates/ai-brains-cli/tests/doctor_cli.rs` | AC3 (or same new file if shared fixture) |
| `Docs/CAPABILITIES.md` / `Docs/OPERATIONS.md` / `CHANGELOG.md` | F6/F24 |
| `conductor/conductor.md` / `deferred.md` / this folder | Registry + absorb notes |

Do **not** touch: `project.rs`, CLI `preflight.rs`, `sync.rs`, `doctor.rs` (logic), `governed_common.rs`, `ranking.rs`, contracts, migrations, live `.env`, live `backups/*.bak` (except AC7 on go).

---

## 13. AI fold-in disposition

*Empty until `/fold-in` of `*-review.md`. Plan-track does not invent findings.*
