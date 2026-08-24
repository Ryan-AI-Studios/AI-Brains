# T295 — Operator vault must have ≥1 usable encrypted backup

- **Track ID:** T295-UsableBackup
- **Status:** **Completed** (2026-08-24; F2a live file + after_help)
- **Category:** OPS / RECOVERY / UX
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `backup list`/`verify` honest **8/8**; **not working:** 0 usable; doctor `backup_recent` warn. Placeholder minted with T285–T300 (`76c4db9`). T277 ✅ fail-closed create (hermetic); live `--no-prune` **skipped**.
- **Depends on:** T277 ✅ `drop(dst)` + classify fail-closed create + mixed hermetic; T209 ✅ list honesty; T225 ✅ verify quiet + create nudge; T244 ✅ usable class; T187 ✅ SQLCipher; T126 ✅ default `--keep 10`; T188 restore daemon hard-fail (do **not** steal onto create); T192 doctor `backup_recent`
- **Blocks / feeds:** Doctor `backup_recent` can be **ok** under the **current** key on this machine. T181 drills stay credible. DataKey rotation (T189) still expects a verified recent backup. Nightly Router **T296**. Graph sparse **T300**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “0 usable encrypted backup”; T277 closeout “live 22 residual until owner `backup create --no-prune`”; T225 residual “operator still runs live `backup create`” (T277 skipped); CAPABILITIES §11 green path omitting `--no-prune` / custom `--output-dir` vs doctor sibling dir
- **Not absorbed (DoD):** T277 F2 engine rewrite; rekey/transcode T244 `.bak`; default keep-10 change; doctor remediator string (T277 F8); growing `doctor.rs`; prune `remaining_count` dry-run honesty (T277 F20); class-aware prune / archive (T244 F18); verify `--quiet` / JSON summary / `VerifyError` (T244 F17); `cipher_integrity_check` (T187); restore / create daemon probe (T188); clap 5 / rusqlite 0.40; T296–T300; T294 leftover `--write`
- **Research date:** 2026-08-24 (plan dogfood HEAD `56d905a` T294 `#210`. Fold-in HEAD `cd9701a`. Product `src/` = T277 F2 already in `run_backup_from_conn`. PATH **0.1.2** 2026-08-22 19:41 **has T277**. Live hole is **operator file**, not missing engine.)
- **AI fold-in:** 2026-08-24 `agy-review.md` (`cd9701a`) + `opencode-review.md` (`cd9701a`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy m1 AC5 distinct substrings; Agy O1 OPERATIONS list+doctor vs `--output-dir`; Agy O2 / OpenCode O1 AC5 `--dry-run --no-prune` example; OpenCode m2 F12 `help_ia.rs` + `tests/cli_help_ia.rs`; OpenCode O2 F35/F37 no-vault combined streams; OpenCode O3 F38 live N+1 count. **Already:** Agy m2 F14/AC3; OpenCode m3 F19. **Snapshot:** OpenCode m1 HEAD `56d905a`→`cd9701a`; word/pin/hotspot/doctor volatile. **Decline:** none of B/M (none filed). Disposition **§13**.
- **Ledger:** planning DOCS TX `37c18651-f942-4732-afca-31b5e6269134`. Fold-in DOCS TX `f02074c2-b30c-40f2-9ac4-5c784f960844`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** rewrite `.env` (T240 F2). Do **not** live `backup create` / prune / restore until owner confirms **at go**. Do **not** `retention apply --confirm`. Do **not** mutate schtasks. Do **not** `graph rebuild`. Do **not** leftover `rebind-path --write --yes`. Do **not** grow hotspot `project.rs` (**#1** **3.906** fold-in) / `sync.rs` / `governed_common.rs` / `forget.rs` / CLI `preflight.rs` / **`doctor.rs`** / `src/help_ia.rs` / `tests/cli_help_ia.rs`. Touch clap `main.rs` Create `after_help` + docs + `backup_recoverable.rs`. Reuse T277 engine. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **This operator vault has ≥1 doctor-usable encrypted backup under the current key.** Honesty is done (T209/T225/T244). Recoverability engine is done (T277 F2). The live file is not: `backup verify` is **0 OK / 22 FAIL**, doctor `backup_recent` is `no usable encrypted backup under current key` + remediator `ai-brains backup create`.
2. **Live create uses `--no-prune` and the default sibling `backups/` directory.** Default keep-10 **would prune 12** residual files (dry-run this pass). Custom `--output-dir` would **not** satisfy doctor (list is vault-parent `backups/` only). T277 Completed hermetic **with live skip** — this track does **not** Complete on hermetic alone.
3. **Help and docs match the live remediator.** Create currently has **no** `after_help`. CAPABILITIES green path omits `--no-prune`. Doctor remediator stays T277 F8 (`ai-brains backup create` only — **do not grow `doctor.rs`**). Clap + OPERATIONS/CAPABILITIES tell the operator `--no-prune` + default dir.
4. **North star.** Capture independence: backup path stays SQLCipher Online Backup API + event-log copy. No models/graph required. No hidden CoT. A FAIL-heavy residual fleet with **≥1 OK** is success (verify still exit **1**). Old ciphertext stays (no transcode).

This unblocks daily recoverability on the machine the audit scored. T277 locked “create means usable.” T295 **runs it** on the live vault (owner-confirm) and locks the operator runbook so the next KEY change cannot leave doctor warn with only a hermetic proof.

---

## 2. Live baseline (re-scan 2026-08-24)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `56d905a` T294 squash `#210`. Tree **CLEAN**. `origin/main` = HEAD. Branch `main`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. **Has T277** fail-closed create. **Does not have T285–T294.** Backup hole is **live file**, not PATH-behind engine. **Do not `cargo install`.** Live Manual may use PATH (operator binary) **or** `cargo run` (source also has T277). |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4059** (volatile). In-context **0/0/0**. Word **305**. Capture independence holds. |
| Fleet | **22** `vault-*.db.bak` under `C:\dev\ai-brains\backups\`. Newest: `vault-2026-08-12T15-50-06.db.bak` **78 200 832** bytes (T244 live create; now KeyMismatch). Vault **150 437 888** bytes (~143.5 MiB; T277 plan ~123 MB — grew). |
| `backup list --quiet` | Row 1 = T244 file **`(unreadable key)`**. Mix of `(legacy plain)` + `(no core tables)`. **0** Readable / PreT109. |
| `backup verify` | **`Verified 22 backups: 0 OK, 22 FAIL.`** First FAIL: T244 file `Key verification failed: file is not a database`. Nudge: `No usable encrypted backup under current key. Run: ai-brains backup create`. Exit **1**. |
| `doctor --format json` `backup_recent` | **warn**, `ok: false`, message `no usable encrypted backup under current key`, remediation **`ai-brains backup create`**. Other doctor warns this pass (not this track): `recovery_kit_event`, `graph_density` (T300), `project_identity` (T258), `policy_grants` (T275 residual). Matrix still **15** checks. |
| `backup create --dry-run --no-prune` | Would write `C:\dev\ai-brains\backups\vault-<now>.db.bak`, source `C:\dev\ai-brains\vault.db`, estimated size **150437888**. **Did not create.** |
| `backup create --dry-run` (default keep 10) | Same preview + **would prune 12** residuals + `[dry-run] Would prune 12 backup(s), 22 remaining. Would free 22.07 MB.` — T277 F20 remaining = pre-delete. Live go **`--no-prune`**. |
| `backup create --help` | Options `--output-dir` / `--keep` / `--no-prune` / `--dry-run`. **No after_help.** No residual-fleet / default-dir / doctor-sibling warning. |
| Daemon | `daemon status` **Stopped**. LLM :8081 **Open**; Embedding :8083 **Open**. Doctor `daemon_reachable` **ok** this pass (probe ≠ status — **T297**, not this track). T188 restore probe is N/A for create. |
| Disk | `C:` free **~142.5 GB** — 150 MB snapshot is not a space block. |
| T277 hermetic | `crates/ai-brains-cli/tests/backup_recoverable.rs` + brain `run_backup_from_conn__missing_cores__fails_and_deletes` exist. **Stay-green.** |
| Last GitHub PR | [#210](https://github.com/Ryan-AI-Studios/AI-Brains/pull/210) T294 (merged 2026-08-24). `gh pr view --json comments,reviews` **empty**; `/reviews` `[]`; `/comments` `[]`; `issues/210/comments` `[]`. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, `#58` tower-http, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / doctor (ledgerful) | ledgerful doctor **4** warn (legacy `.changeguard` / sig-pin / sig-version / timings). Optional :8081 unreachable at doctor; :8083 **ok**. **0 pending / 0 drift.** Hotspot **#1** `project.rs` (**3.915**) — **do not touch.** `sync.rs` **#2**. `governed_common.rs` **#3**. `commands/context.rs` **#4** (T294 grew it). `forget.rs` **#5**. CLI `preflight.rs` **#8**. Brain/CLI `backup.rs` **not** top-10. |
| `ISSUES.md` | **Does not exist.** |
| Planning live create | **Not run.** Dry-run only. |

### 2.2 Why the live hole remains after T277

| Layer | Truth |
|-------|--------|
| T277 F2 | Engine fail-closes Incomplete after `drop(dst)` + classify. Hermetic mixed: new Readable + residual KeyMismatch + doctor ok + verify 1 OK/1 FAIL exit 1 no nudge. **Shipped.** |
| T277 AC7 / F4 | Live `--no-prune` **owner-confirm**. Owner did **not** confirm. T277 **Completed hermetic**. Residual: live 22/22 FAIL. |
| T244 file | `vault-2026-08-12T15-50-06.db.bak` was Readable under the then-current key. Now KeyMismatch. Compensating **new** snapshot — not a converter. |
| Doctor remediator | Exact `ai-brains backup create` (**no** `--no-prune`). Following it on this fleet **would prune 12** residuals (T244 F16 / T126). T277 F8: **do not change** that string / grow `doctor.rs`. |
| CAPABILITIES §11 | Green path `ai-brains backup create` then verify. T277 paragraph mentions `--no-prune` **if residuals must be kept**. Example block still demos `--output-dir D:\backups --dry-run` — that path is **invisible** to doctor. |
| OPERATIONS Backup | Already shows `--no-prune` in the green-path snippet. Live this vault **must** use it + default dir. Tighten, do not add a second Backup heading. |
| Default keep-10 | Product default **unchanged** (T126 / T277 F19). Live go **`--no-prune`**. |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|--------|
| Create CLI | `crates/ai-brains-cli/src/commands/backup.rs` `run_create` **`:55–115`** | `run_backup_from_conn`; prints `Backup created and verified:`; prune if `keep` Some. Dry-run `preview_backup_path` + optional `prune_backups(..., true)`. |
| Engine | brain `backup.rs` `run_backup_from_conn` **`:152–241`** | `Backup::new` + `run_to_completion(100000, ZERO)`; dest `apply_key_pragmas`; `integrity_check`; T109 meta; **`drop(dst)` `:227`**; `classify_backup_read`; `!is_usable_class` → `remove_file` + `Err` naming class. **Do not rewrite.** |
| Classify / usable | `:462` / `:33` | Readable \| PreT109 only. |
| List sort | CLI `run_list` **`:163`** | T244 F7 usable-first **CLI only**. Brain `list_backups` timestamp-desc (doctor). |
| Verify | CLI `run_verify` + `should_emit_create_nudge` (`ok==0 && total>=1`) | Any FAIL → exit **1**. Nudge present today (`ok==0`). After live create: ≥1 OK, nudge **gone**, exit still **1**. |
| Doctor | `doctor.rs` `check_backup_recent` **`:330–376`** | Zero usable → exact message + remediator `ai-brains backup create`. **Do not grow `doctor.rs`.** |
| Restore daemon | `probe_restore_daemon_busy` | Restore only. Create does **not** call it. |
| Prune remaining | `prune_backups` **`:355`** | `remaining_count = candidates.filter(exists)` — dry-run remaining is **pre-delete** (T277 F20). |
| clap Create | `main.rs` `BackupCommands::Create` **`:3148–3164`** | `--output-dir` / `--keep` / `--no-prune` / `--dry-run`. **No after_help.** Dispatch **`:4784–4798`**: `effective_keep = if no_prune { None } else { keep.or(Some(10)) }` **`:4790`**. Bare `backup` **`:4815`** still `Some(10)`. |
| T277 tests | `tests/backup_recoverable.rs` | Mixed `--no-prune` AC2–AC4/AC13. **Stay-green.** |
| Brain AC1 | `run_backup_from_conn__missing_cores__fails_and_deletes` **`:1153`** | **Stay-green.** |
| CAPABILITIES | §11 `:498–540` | Green path `:536` omits `--no-prune`. T277 KEY-change `:538`. |
| OPERATIONS | Backup `:749–775` | Green path already `--no-prune`. Live runbook needs **default sibling dir** + **this vault must**. |
| PROTOCOL-COMPAT | `:97` `backup create` JSON **compact** | Create has **no** `--format`. **No new JSON row / keys.** Human after_help is **not** a wire contract. |
| CLI-EXIT-CODES | create success **0**; verify any FAIL **1**; doctor 15-check | Unchanged. |
| Contracts | no public `BackupReadClass` DTO | T244 F19 freeze. |
| Dir | `backup_dir` = vault parent `backups/` | T192 F17b list/preview do not `create_dir_all`. Doctor only sees this dir. |
| Hotspot | `project.rs` **#1 3.915** | **Do not touch.** |

### 2.4 Dependency / standards research (2026-08-24) — snapshot; re-verify at execute

| Pin / source | Workspace / live | Action |
|--------------|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** / GitHub **v4.6.6** (2026-08-06) — **no clap 5** (latest release is 4.6.6) | **No bump.** `after_help` only. Snapshot — re-verify at execute. |
| `rusqlite` | lock **0.39.0** + `bundled-sqlcipher-vendored-openssl` + `backup` / crates.io **0.40.2** (`#61`; 0.40.x bundles SQLite 3.53.2) | **No bump** (SQLCipher build risk). [docs.rs 0.39.0 `Backup::new` + `run_to_completion`](https://docs.rs/rusqlite/0.39.0/rusqlite/backup/struct.Backup.html) still the create engine. |
| `chrono` | lock **0.4.44** / crates.io **0.4.45** (`#62`) | **No bump.** |
| `serde_json` | lock **1.0.150** | **No bump.** |
| `uuid` | lock **1.23.1** | **No bump.** |
| `thiserror` | lock **2.0.18** / crates.io **2.0.20** (`#60`) | **No bump.** |
| `tokio` | lock **1.52.3** (`#59`) | **No bump.** |
| rustc / edition / nextest | **1.95.0** / **2024** / **0.9.140** | Unchanged. |
| workspace version | **0.1.2** | **No bump.** |
| New crates | — | **Zero.** |

**Online (primary sources, this pass):**

| Claim | Source | Fit |
|-------|--------|-----|
| Encrypted→encrypted Online Backup API (same key); `sqlcipher_export` is **migrate** | [Zetetic 4.3.0 release](https://www.zetetic.net/blog/2019/12/20/sqlcipher-430-release/) + [discuss #2631](https://discuss.zetetic.net/t/using-the-sqlite-online-backup-api/2631): 4.3.0+ permits encrypted-to-encrypted / plaintext-to-plaintext, **not mixed**. [sqlcipher-api](https://www.zetetic.net/sqlcipher/sqlcipher-api/) lists `sqlcipher_export` under Migration. 2019 “backup API disabled” threads are **pre-4.3**. | Keep `rusqlite::backup::Backup`. Do **not** switch create to `sqlcipher_export` (T187 vault encrypt). |
| `Backup::new` + `run_to_completion` | [docs.rs rusqlite 0.39.0](https://docs.rs/rusqlite/0.39.0/rusqlite/backup/struct.Backup.html); [sqlite.org/backup.html](https://www.sqlite.org/backup.html) (updated 2025-11-13): snapshot of source as copying commenced; `SQLITE_BUSY` on dest lock — existing busy_timeout (T96). `VACUUM INTO` / `sqlite3_rsync` are **other** techniques — not this track. | Already used brain `:178–179`. |
| Recovery **proof** > file exists | T181 RECOVERY-DRILLS; CISA 3-2-1-1-0 “zero verified errors” maps to **≥1 verify OK + doctor usable**, not a second disk. | Live create + verify ≥1 OK + doctor not zero-usable. Offsite/immutable **not** DoD. |
| After key rotation, re-backup | NIST SP 800-57pt1r6; T277 F6; exhibit T244 filename | Docs already say so. Live file is the missing proof. |
| [clig.dev Output](https://clig.dev/#output) (fetched 2026-08-24) | Humans first; if you change state, tell the user; make the default the right thing; actions crossing the program boundary should be explicit; **changing human output is usually OK**; `-n`/`--dry-run` is the standard preview | Default keep-10 stays (T126 — most users without a 22-file residual fleet). Residual-fleet operators need **discoverable** `--no-prune` (`after_help` + docs). Live mutate is owner-confirm + dry-run already exists. Create already prints `Backup created and verified:` (clig “if you change state, tell the user”). |
| T180 P-CLI | Human output may evolve; JSON is the wire | Create has no `--format`. PROTOCOL-COMPAT row stays empty compact. **No new JSON.** |

Training data is not a pin. Re-verify clap / rusqlite backup API at execute.

**Could not verify:** live `backup create --no-prune` (Stop-Before until go+owner). Hermetic T277 is the engine proof. Doctor JSON `daemon_reachable: ok` vs `daemon status` Stopped — T297.

**ledgerful / ai-brains:** `preflight --summary` 4059 pins @ `3581317d`; `recall` lexical/semantic thin (T277 review-track dumps / unrelated T256–T260). `ledgerful search "run_backup_from_conn"` hit brain `:152` / CLI `:98` / AC1 `:1153`. Index `--incremental` OK (973 docs). `scan --impact` CLEAN.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. Planning **must not** live-create, prune, restore, or `cargo install`. Dry-run research OK. |
| **F1 — Engine freeze** | T277 F1/F2/F42/F43 stand. SQLCipher Online Backup API (`Backup::new` + `run_to_completion`) + dest key + `integrity_check` + T109 meta + `drop(dst)` + `classify_backup_read` + delete-if-not-usable. **Do not** rewrite `run_backup_from_conn`. **Do not** switch to `sqlcipher_export`. |
| **F2 — Live file is the DoD** | Unlike T277 (Completed hermetic + skip), T295 **does not Complete** on hermetic stay-green alone. Completion requires **either** (a) live `--no-prune` create + list Readable on top + verify ≥1 OK + doctor `backup_recent` not `no usable encrypted backup under current key`, **or** (b) owner **explicit skip** in the go prompt, recorded in `review.md` + deferred closeout, status stays **Pending** (skip ≠ Complete). Hermetic is necessary, not sufficient. |
| **F3 — Live flags** | Mutating only with owner confirm at go. Command: `ai-brains --no-project-context backup create --no-prune`. **No** `--keep`. **No** `--output-dir` (doctor only lists vault-parent `backups/`). **No** restore. **No** `backup prune`. Daemon stays Stopped; do not start it for this track. |
| **F4 — Keep-10 product default** | T126 / T277 F19. `keep.or(Some(10))` **`:4790`** and bare `backup` **`:4815`** unchanged. Live go `--no-prune` so 12 residuals survive. |
| **F5 — Old ciphertext stays** | Do **not** rekey, transcode, or `sqlcipher_export` the T244 file or other residuals. KeyMismatch / LegacyPlain / Incomplete stay honest. |
| **F6 — Create `after_help`** | Additive clap `after_help` on `BackupCommands::Create` (`main.rs` **`:3148`**). Frozen themes (exact wording in **AC5 / §5.1**): residual fleet (unreadable-key / legacy-plain / no-core-tables) is kept only with `--no-prune`; default `--keep 10` deletes older files **by timestamp, not class**; doctor `backup_recent` only sees the vault sibling `backups/` (do not use `--output-dir` if the goal is doctor-ok); after KEY change create a **new** snapshot. Examples include `backup create --dry-run --no-prune` and `backup create --no-prune`. Do **not** restyle List/Verify/Restore/Prune help. |
| **F7 — Doctor remediator freeze** | Stay `ai-brains backup create` only (T277 F8 / T244 F4). **Do not grow `doctor.rs`.** Docs + after_help carry `--no-prune`. |
| **F8 — Docs** | CAPABILITIES §11 green path (`:536`): live remediator includes `--no-prune` when residuals must be kept; doctor-usable means **default sibling `backups/`**, not `--output-dir D:\backups`. OPERATIONS Backup (`:749`) extend (do not add a second Backup heading): this-vault runbook with `--no-project-context`, `--no-prune`, no `--output-dir`, verify ≥1 OK / exit 1 OK, doctor not zero-usable. **Explicit:** `--output-dir` is a manual export; **`backup list` (default dir) and doctor `backup_recent` only scan the vault sibling `backups/`** (Agy O1). CHANGELOG T295. Soft RECOVERY-DRILLS one-liner. PROTOCOL-COMPAT **no new JSON row** (F20). |
| **F9 — No restore / no daemon mutate / no mass-delete** | T188 restore stays. Create does **not** gain a daemon probe (T277 F35). No live `backup prune`. No nightly scheduled create. |
| **F10 — Pins** | No rusqlite 0.40, clap 5, chrono/serde_json/thiserror/tokio/uuid bumps, new crates, DTO keys. Workspace **0.1.2**. |
| **F11 — Capture independence** | Backup path stays store/crypto/brain/cli. No models/graph **required**. Feature-off still creates via SQLCipher. |
| **F12 — Hotspots** | **Do not grow** `project.rs` / CLI `preflight.rs` / `sync.rs` / `governed_common.rs` / `forget.rs` / `ranking.rs` / **`doctor.rs`**. Do **not** edit T277 production engine unless a compile forces it (then stop). Clap Create `after_help` in `main.rs`. Tests in `backup_recoverable.rs` (help AC) — do **not** mint a new integration binary unless help test cannot live there. Do **not** grow `crates/ai-brains-cli/src/help_ia.rs` (root `ROOT_AFTER_HELP_TIP` / `ROOT_AFTER_LONG_HELP` at `main.rs:11` / `:1279–1280`). Do **not** grow `crates/ai-brains-cli/tests/cli_help_ia.rs` (T204/T291 extra lock analog — OpenCode m2). |
| **F13 — Success substring** | Keep `Backup created and verified:` (T277 F22 / smoke). Do not require class token in that line. |
| **F14 — Verify exit frozen** | Any FAIL → exit **1**. Recoverability success = `ok >= 1`, not exit 0 on a residual fleet. Nudge only when `ok==0 && total>=1` (T225 F9). After live create: nudge **absent**. |
| **F15 — List sort** | Usable-first stays CLI `run_list` only (T244 F7). After F3 the new Readable is top row. |
| **F16 — PATH** | Soft. Source/hermetic SoT for after_help AC. Live Manual may use PATH 0.1.2 (has T277) **or** `cargo run`. Do not `cargo install` as implement. PATH-behind T285–T294 is **irrelevant** to backup classify. |
| **F17 — Secrets** | Never print `AI_BRAINS_KEY` / `--help` env defaults that contain it. |
| **F18 — T240 F2** | No `.env` rewrite. No live leftover `--write --yes`. No live `retention apply --confirm`. |
| **F19 — Soft declines** | T277 F20 remaining_count dry-run lie; T244 F17/F18; T209 L3 remainder; `cipher_integrity_check`; 3-2-1-1-0 offsite; class-aware prune; doctor remediator `--no-prune` suffix. |
| **F20 — PROTOCOL-COMPAT** | `backup create` JSON row stays empty compact. Create has no `--format`. Human after_help is **not** a wire contract. **No** new required keys anywhere. |
| **F21 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in **production**. Hermetic `tempfile` + `hermetic_bin`. New help AC in `backup_recoverable.rs`. T277 mixed tests **stay-green**. |
| **F22 — ISSUES.md** | Does not exist. Debt is `deferred.md`. |
| **F23 — Decline peers** | T294 leftover dest / `--write` — Completed `#210`, live 5 roots still F11 there. T296 nightly Router; T297 daemon vs LLM; T298 device; T299 forget-list; T300 graph sparse. T277 Completed — do not reopen F2. |
| **F24 — Standing declines** | T240 F2 reopen; T263 H2; 750 ms; clap 5; rusqlite 0.40; DTO new required keys; dest mint in rebind; silent `.env`; schtasks mutate. |
| **F25 — No T301** | #210 Cursor **N/A empty**. Dependabot remotes are not tracks. |
| **F26 — Identity leftover `7d97` vs `fcb8a40f`** | **Not this track.** T258 / leftover data T276. |
| **F27 — Cross-model** | FEATURE / OPS / live vault snapshot. After Phase-1 review clean, run read-only `codex-review`. |
| **F28 — Size volatile** | Source vault **150437888** this pass (T277 ~123 MB). Re-dogfood dry-run at Phase 0. Hermetic stays tiny. |
| **F29 — `--no-project-context`** | Live Manual uses it so cwd `.env` does not switch Scope (T240 F2 analog). Hermetic tests already do. |
| **F30 — Not vault encrypt** | `vault encrypt` / `sqlcipher_export` stay T187. |
| **F31 — Stop-before** | Even after go: no live create without owner confirm; no prune of residuals; no restore; no extra live `policy bootstrap`; no `retention apply --confirm`; no `graph rebuild`; no schtasks mutate; no `cargo install`; no `.env` rewrite. |
| **F32 — PowerShell** | `;` not `&&`. |
| **F33 — Doctor matrix** | Frozen **15** checks. `backup_recent` stays soft. Do not add check 16. |
| **F34 — No compensating rewrite of `.bak`** | Analog of no `MemoryMoved`. New file only. |
| **F35 — Help AC isolation** | Help test runs `backup create --help` only (no vault write). Do not require a live vault. Pattern: `tests/cli_help_ia.rs` `help_stdout` — `hermetic_bin()` + args, **no** `--vault-path`, assert exit **0**. Combined stdout+stderr (OpenCode O2). If clap help does **not** short-circuit without a vault, fallback hermetic init + record in `review.md`; prefer no vault. |
| **F36 — Bare `ai-brains backup`** | Still default-create keep-10 (`:4815`). after_help lives on `Create` subcommand; bare `backup --help` need not duplicate (optional same after_help only if clap shares it — do **not** invent a new subcommand). |
| **F37 — Help AC distinct locks (Agy m1 / OpenCode O1)** | AC5 uses **separate** `contains` asserts (not one concatenated haystack that could pass on a single mashed line): (1) `--no-prune`; (2) timestamp-not-class / residual-fleet wording; (3) `backups/` **or** `backup_recent`; (4) the example substring `backup create --dry-run --no-prune`. Combined stdout+stderr. |
| **F38 — Live residual count (OpenCode O3)** | Phase 0 records **N** = count of `vault-*.db.bak` (plan-time **22**). After live `--no-prune` create: list has **N+1** files; new Readable first. Paste the exact `backup list --quiet` transcript into `review.md`. Do **not** freeze 22 if Phase 0 N drifted. |

---

## 4. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | T277 `run_backup_from_conn__missing_cores__fails_and_deletes` still Err + dest **absent** + `Incomplete` + `core tables` | Brain unit stay-green |
| **AC2** | T277 mixed: `backup_create__key_mismatch_residual__new_readable_and_doctor_ok` still first-row Readable + residual `(unreadable key)` + doctor `backup_recent` Ok | CLI hermetic stay-green |
| **AC3** | T277 `backup_verify__mixed_ok_and_key_mismatch__one_ok_exit_1_no_nudge` still 1 OK / 1 FAIL / exit 1 / no create nudge | CLI hermetic stay-green |
| **AC4** | T277 `backup_list__mixed_after_create__residual_summary_not_recoverable` still `not recoverable under current key` | CLI hermetic stay-green |
| **AC5** | `backup create --help` (hermetic bin, **no** `--vault-path`, F35) combined stdout+stderr: **separate** asserts (F37) for `--no-prune`, timestamp-not-class / residual-fleet, `backups/` or `backup_recent`, and example `backup create --dry-run --no-prune`. Does **not** require `--output-dir` for doctor-ok. Exit **0**. | New CLI help test |
| **AC6** | clap Create still `--output-dir` / `--keep` / `--no-prune` / `--dry-run`; dispatch `keep.or(Some(10))` unless `--no-prune`; no new flags | Review + help |
| **AC7** | CAPABILITIES §11 green path mentions `--no-prune` when residuals must be kept **and** doctor-usable = default sibling `backups/` (not custom `--output-dir`). OPERATIONS Backup this-vault runbook **plus** one sentence that `--output-dir` is a manual export and **`backup list` + doctor only scan sibling `backups/`** (Agy O1 / F8). CHANGELOG T295. | Grep |
| **AC8** | Live on go (owner confirm): `backup create --no-prune` (no `--output-dir`) → list first row **not** residual token; verify ≥1 OK (exit 1 OK; **nudge absent** — F14 / Agy m2); doctor `backup_recent` not `no usable encrypted backup under current key`; file count **N+1** where N is Phase 0 `vault-*.db.bak` count (plan-time 22; F38). Paste list transcript in `review.md`. **Planning pass must not create.** If owner **explicitly skips**: status stays Pending (F2b). | Manual |
| **AC9** | No production `unwrap`/`expect`/`panic` on touched paths; lockfile clap/rusqlite/chrono/serde_json unchanged; no DTO keys; `doctor.rs` production **untouched** | Review / diff |
| **AC10** | T188 restore-daemon units stay green; create path still has **no** `probe_restore_daemon_busy` | Grep + nextest |
| **AC11** | Capture independence: no `ai-brains-models` / graph **required** on backup create | Grep |
| **AC12** | Smoke `backup_verify__valid_backup__reports_ok` still finds `Backup created and verified:` | Stay-green |
| **AC13** | T244 `backup_list_honesty` + `doctor__backup_recent__all_incomplete__warn_no_usable` stay green | Regression |
| **AC14** | No leftover UUID / live backup filename hardcoded in product `--help` (T244 exhibit filename OK in **docs** only, as T277 F6) | Review |
| **AC15** | Full gate at implement closeout only (fmt, clippy -D, nextest workspace, deny, audit) | CI |

Test names (TDD). **Must fail red before F6 after_help exists:**

- `backup_create_help__after_help__mentions_no_prune_default_dir`

**Stay-green (do not “fix” unless this track broke them):** AC1–AC4, AC12, AC13, T188 restore daemon.

---

## 5. Design notes

### 5.1 Create `after_help` (F6)

Suggested frozen text (implement may wrap for clap width; AC5 / F37 lock **separate** substrings `--no-prune`, residual/timestamp, `backups/` or `backup_recent`, and `backup create --dry-run --no-prune`):

```text
Default --keep 10 prunes older vault-*.db.bak by timestamp, not class.
A residual fleet ((unreadable key) / (legacy plain) / (no core tables)) is kept only with --no-prune.
Doctor backup_recent only lists the vault sibling backups/ directory — omit --output-dir when the goal is doctor-ok.
After AI_BRAINS_KEY change, old .bak stay KeyMismatch; create a new snapshot.
Examples:
  ai-brains --no-project-context backup create --dry-run --no-prune
  ai-brains --no-project-context backup create --no-prune
```

Do not print KEY. Do not hardcode `vault-2026-08-12T15-50-06.db.bak` in clap (docs only).

### 5.2 Live go runbook (not executed in planning)

```powershell
ai-brains --no-project-context backup create --dry-run --no-prune
# owner confirms — ~150 MB next to vault; would prune 12 without --no-prune
ai-brains --no-project-context backup create --no-prune
ai-brains --no-project-context backup list --quiet   # new Readable first
ai-brains --no-project-context backup verify         # ≥1 OK; exit 1 OK; no create nudge
ai-brains --no-project-context doctor --format json  # backup_recent not zero-usable
```

Do **not** restore. Do **not** `--keep 10`. Do **not** `--output-dir`. Daemon is Stopped; do not start it for this track.

### 5.3 Why this is not another T277

T277 made create **honest** (fail-closed) and **hermetically** recoverable. The operator vault still has **zero** files the current key can open. Completing T277 with a skip left the audit hole. T295 is the operator file + discoverable `--no-prune` so the doctor remediator is not a footgun.

### 5.4 Why not change doctor remediator

T277 F8 / T244 AI2 L5 freeze the string `ai-brains backup create`. Growing `doctor.rs` (1738+ lines, 15-check matrix) for a `--no-prune` suffix is out of isolation. after_help + OPERATIONS is the discoverability layer. Operators who follow doctor literally on a **clean** vault still get the T126 default (correct). Operators with residuals read `--help` / OPERATIONS.

### 5.5 Help test

`backup_create_help__after_help__mentions_no_prune_default_dir` in `backup_recoverable.rs`, same shape as `tests/cli_help_ia.rs` `help_stdout`: `hermetic_bin().arg("backup").arg("create").arg("--help")` — **no** `--vault-path` (F35). Combined stdout+stderr. Separate `contains` asserts (F37). Exit **0**. If help fails without a vault, fallback hermetic init + record; prefer no vault. Do **not** add this lock to `tests/cli_help_ia.rs` (F12).

---

## 6. Non-goals

- Rekey / transcode / `sqlcipher_export` of residual `.bak`.
- Auto-delete, `backups/legacy/` move, clap archive subcommand.
- Nightly scheduled create; offsite / immutable / 3-2-1-1-0 media.
- Restore; recovery export; DataKey rotation.
- Daemon probe on create; doctor 16th check; growing `doctor.rs`; doctor remediator `--no-prune`.
- verify `--quiet` flag; JSON `summary`; structured `VerifyError`.
- `cipher_integrity_check`; rusqlite 0.40 `table_exists`.
- clap 5; new DTO keys; new crates; `cargo install`.
- T240 F2 `.env`; T275 live bootstrap; T276/T294 leftover `--write --yes`.
- Prune `remaining_count` dry-run honesty (T277 F20); class-aware prune.
- Changing default `--keep 10`.
- Completing the track on hermetic stay-green without live file **or** explicit owner skip-stay-Pending.

---

## 7. Verification plan (TDD)

**Phase 1 red (required before green):** AC5 help test (today `--help` has flags but **no** residual-fleet / default-dir after_help).

Then green: F6 `after_help` on Create → AC6 flags unchanged → docs F8/AC7.

**Stay-green:** AC1–AC4, AC10, AC12, AC13, T188 restore daemon.

**Manual:** AC8 live create **only if owner confirmed at go**. Planning: dry-run only.

Targeted: `cargo nextest run -p ai-brains-cli --test backup_recoverable` ; `-p ai-brains-brain backup` ; `-p ai-brains-cli --test backup_list_honesty --test doctor_cli --test smoke --test recovery_drills` ; `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`.

Full workspace gate only at implement closeout — **not** a plan gate.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Live create ~150 MB next to vault | F3 `--no-prune` + owner confirm; dry-run shows path/size; C: free ~142 GB this pass |
| Default keep-10 deletes 12 residuals | F3/F4 live `--no-prune`; T126 default stays; after_help F6 |
| `--output-dir` hide from doctor | F3/F8 forbid on live go; after_help + CAPABILITIES |
| Completing like T277 without live file | F2: skip → stay Pending |
| F2 engine regression | AC1–AC4 stay-green; do not edit engine |
| Restore while testing | F9; T188 if someone tries anyway |
| `doctor.rs` / `project.rs` growth | F7/F12 |
| PATH-behind T285–T294 | F16; backup hole is live file; PATH has T277 |
| Dry-run remaining 22 vs 10 | T277 F20 documented; not DoD |
| KEY printed in help | F17; no KEY in after_help |
| WAL writer during ~150 MB copy | Create borrows `ctx.conn`; daemon Stopped; existing busy_timeout |
| Doctor other warns look like failure | AC8 asserts **backup_recent** only; graph_density / policy_grants / identity stay |

---

## 9. Deferred absorb / decline

### 9.1 Open overlapping rows (entire `deferred.md` scan)

| Item | Disposition |
|------|-------------|
| 0 usable encrypted backup (T285–T300 mint table) | **Absorb** F2–F8 / AC5–AC8 |
| Placeholder Manual `backup create --no-prune` + list + verify + doctor | **Absorb** AC8 |
| T277 closeout live 22 residual until owner create | **Absorb** F2 / F3 / AC8 |
| T225 “Operator still runs live `backup create`” | **Absorb** F2 / AC8 (T277 skipped) |
| T244 live Readable 2026-08-12 now KeyMismatch | **Absorb exhibit** F5; new snapshot is DoD |
| CAPABILITIES green path omits `--no-prune`; example `--output-dir` | **Absorb** F6 / F8 / AC5 / AC7 |
| T277 F2 engine / mixed hermetic | **Affirm freeze** F1 / AC1–AC4 — do not reopen |
| T277 F8 doctor remediator | **Affirm** F7 — do not grow `doctor.rs` |
| T277 F20 prune dry-run `remaining_count` | **Decline** F19 |
| T244 F17 / T225 F17 verify quiet/JSON/VerifyError | **Decline** F19 |
| T244 F18 archive | **Decline** F19 |
| T209 L3 wrong-key fixture remainder | **Decline** — T277 F33 mixed other-key already hard |
| T187 `cipher_integrity_check` | **Decline** F19 |
| T181 restore drills / offsite | **Decline** — T181 closed |
| T294 leftover dest-missing / 5 roots | **Decline** — Completed `#210`; live `--write` still T294 F11 |
| T296 nightly Router / T297 daemon vs LLM / T298–T300 | **Decline →** those placeholders |
| T240 F2 / T263 H2 / 750 ms / clap 5 / rusqlite 0.40 | **Decline** F24 |
| last-PR Cursor **#210** | **N/A empty** — **no T301** F25 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Decline** F26 — T258 / leftover data |
| Closed T277/T209/T244/T188 DoDs | **Stay closed** |

### 9.2 Last-PR Cursor (#210 T294)

`gh pr view 210 --json comments,reviews`, `pulls/210/reviews`, `pulls/210/comments`, `issues/210/comments`: **all empty**. Open PR on HEAD: **none** (Dependabot remotes only). **No leftover to mint. No T301.**

### 9.3 Closed rows

T274–T294 Completed rows stay closed. Do not reopen T277 F2, T240 F2, T255 750 ms, T263 H2, T188 restore-on-create, T126 keep-10 default.

---

## 10. Implement order (on go)

1. Phase 0 re-verify `run_backup_from_conn` `:227` `drop(dst)`, clap Create `:3148` still no after_help, dispatch `:4790`, doctor `:370` zero-usable, **N** `vault-*.db.bak` (F38), T244 still KeyMismatch, dry-run size, #210 empty, pins, hotspots, `src/help_ia.rs` + `tests/cli_help_ia.rs` still not grown.
2. Red: AC5 help test (F37 distinct substrings + `--dry-run --no-prune` + no vault).
3. Green: F6 after_help; docs F8/AC7 (Agy O1 list+doctor vs `--output-dir`). **No** engine edit. **No** `help_ia.rs` / `cli_help_ia.rs` edit.
4. Stay-green AC1–AC4 / AC12 / AC13.
5. Live AC8 **only if owner confirmed** (N+1 + list transcript F38); else F2b stay Pending.
6. Phase-1 review → `review.md`. Cross-model FEATURE (`codex-review`).
7. Full gate. Publish implement-track Phase 6 **only if F2a landed** (live file). Never `git push origin main`.

---

## 11. Soft residuals

| Residual | Disposition |
|----------|-------------|
| PATH until `cargo install` (T285–T294 not on PATH) | F16 — backup engine is on PATH |
| Live 22 residual `.bak` still KeyMismatch / plain / Incomplete after one OK | F5 — expected; verify exit 1 |
| Doctor remediator still omits `--no-prune` | F7 freeze |
| Prune dry-run `remaining_count` lie | T277 F20 |
| Class-aware prune / archive | F19 / T244 F18 |
| Offsite / immutable copy | local-first |
| Doctor other warns (`graph_density`, `policy_grants`, `project_identity`, `recovery_kit_event`) | T300 / T275 / T258 / T192 — not this track |
| T296 Router Last Result dual-truth | **T296** |
| Daemon Stopped vs LLM Open | **T297** |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/main.rs` | Create `after_help` only (`:3148`) |
| `crates/ai-brains-cli/tests/backup_recoverable.rs` | AC5 help test (F37) |
| `crates/ai-brains-cli/src/help_ia.rs` | **Do not edit** (F12) |
| `crates/ai-brains-cli/tests/cli_help_ia.rs` | **Do not edit** (F12) |
| `Docs/CAPABILITIES.md` | §11 green path `--no-prune` + default dir |
| `Docs/OPERATIONS.md` | Backup this-vault runbook (extend `:749`) |
| `CHANGELOG.md` | T295 |
| `conductor/conductor.md` | T295 Planned → Completed on go+F2a |
| `conductor/deferred.md` | Planning absorption (this pass); closeout on implement |
| `conductor/tracks/README-T285-T300-CLI-QUALITY.md` | T295 Planned |
| `Docs/RECOVERY-DRILLS.md` | Optional one-liner |
| `crates/ai-brains-brain/src/backup.rs` | **Do not edit** (F1) |
| `crates/ai-brains-cli/src/commands/backup.rs` | **Do not edit** unless after_help cannot live in clap (prefer clap) |
| `crates/ai-brains-cli/src/commands/doctor.rs` | **Do not edit** (F7) |
| `crates/ai-brains-cli/src/commands/project.rs` | **Do not edit** |

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` (HEAD `cd9701a`) + `opencode-review.md` (HEAD `cd9701a`). Fold-in HEAD `cd9701a` on `main` (ahead of `origin/main` `56d905a` T294 `#210`). Live verify: clap Create **`:3148–3164`** no after_help; dispatch `keep.or(Some(10))` **`:4790`**; `drop(dst)` brain `:227`; `should_emit_create_nudge` `verify_report.rs:48` (`ok==0 && total>=1`); `format_create_nudge` `:55`; `run_verify` imports those at CLI `backup.rs:276–279`; `mod help_ia;` `main.rs:11`; `ROOT_AFTER_HELP_TIP` `:1280`; file **`src/help_ia.rs` exists**; **`src/cli_help_ia.rs` does not**; help lock analog is **`tests/cli_help_ia.rs`** `help_stdout` (no `--vault-path`). OPERATIONS `:749` already shows `--no-prune` and `--output-dir` without saying doctor/list ignore custom dir. Hotspot `project.rs` **#1** (**3.906** fold-in; plan 3.915). Pins **snapshot — re-verify at execute** (clap lock 4.6.1 / crates.io 4.6.6; rusqlite 0.39.0; **no clap 5**). Last merged PR still **#210** (comments/reviews **empty**). **No T301.** Fold-in preflight: Pinned **4101** / in-context **0/0/0** / word **333** (plan 4059/305; OpenCode 4090/947 — volatile). Doctor **4** warn; :8083 **ok**; :8081 **ok** this pass (plan: unreachable at doctor — volatile).

### Pins locked by fold-in

1. **F37 / AC5 (Agy m1 + OpenCode O1):** separate `contains` asserts for `--no-prune`, timestamp-not-class, `backups/` or `backup_recent`, and example `backup create --dry-run --no-prune`.
2. **F12 (OpenCode m2):** do not grow `src/help_ia.rs` **or** `tests/cli_help_ia.rs`. AC5 lives in `backup_recoverable.rs`.
3. **F35 / F37 (OpenCode O2):** no `--vault-path`; combined stdout+stderr; `cli_help_ia.rs` `help_stdout` pattern.
4. **F38 / AC8 (OpenCode O3):** Phase 0 **N**; after create **N+1**; list transcript in `review.md`.
5. **F8 / AC7 (Agy O1):** OPERATIONS sentence: `--output-dir` is export; list + doctor scan sibling `backups/` only.
6. **F6 typo:** after_help wording pointer is **AC5 / §5.1**, not AC8.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** AC5 distinct substrings | **Folded** F37 / AC5 |
| Agy | **m2** verify exit 1 mixed + no nudge when `ok>=1` | **Already** F14 / AC3 (hermetic) / AC8 (live) |
| Agy | **O1** OPERATIONS `--output-dir` vs doctor/list | **Already** F8; **tightened** F8 / AC7 |
| Agy | **O2** `--dry-run --no-prune` in after_help examples | **Already** §5.1; **folded** AC5 example lock (F37) |
| OpenCode | B / M | None filed |
| OpenCode | **m1** plan HEAD `56d905a` vs `cd9701a` | **Snapshot** — preflight refreshed; not DoD |
| OpenCode | **m2** F12 `cli_help_ia.rs` vs `help_ia.rs` | **Folded** F12 — both `src/help_ia.rs` and `tests/cli_help_ia.rs` |
| OpenCode | **m3** T277 F20 remaining_count excluded consistently | **Already** F19 / plan Declined |
| OpenCode | **O1** AC5 assert `--dry-run --no-prune` example | **Folded** F37 / AC5 (same as Agy O2) |
| OpenCode | **O2** combined streams + clap help no-vault | **Already** §5.5; **tightened** F35 / F37 |
| OpenCode | **O3** AC8 exact 22+1 | **Folded** F38 — N from Phase 0, not a frozen 22 |
| OpenCode | word 305→947 / pin 4059→4090 | **Snapshot only** — fold-in 333 / 4101; not DoD |
| both | last-PR #210 Cursor | **Affirm F25** — no T301 |
| both | deferred T296–T300 / T277 F2 / T277 F8 / T240 F2 | **Affirm** |

No Blockers. No Majors. No new placeholder minted. Do **not** edit `*-review.md`.
