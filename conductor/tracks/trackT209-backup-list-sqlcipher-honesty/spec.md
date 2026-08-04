# T209 — Backup list SQLCipher honesty

- **Track ID:** T209-BackupListSqlCipherHonesty
- **Phase:** Post-T208 skill·CLI audit follow-ups (P2)
- **Status:** 📋 **Proposed / Expanded + AI fold-in** (plan-only until go)
- **Depends on:** T109 meta table; T120 content-based pre-T109 demote; T134 `--quiet`; **T187** page encryption + `is_plain_sqlite_header`; **T197** `sqlcipher_log_policy` (native hmac spam already filtered); **T208** closed (`RUST_LOG` denylist)
- **Blocks / feeds:** Clean operator `backup list` after encrypt/rotate; doctor `backup_recent` already uses quiet list
- **Category:** FEATURE / DOCS (light)
- **Source:** Non-destructive skill/CLI audit 2026-08-04 — **backup list 4/3** WARN flood post-vault encrypt
- **Deferred absorbed:** Audit backup list 4/3; deferred.md T209; T208 residual “T209 backup WARN flood”; T120/T134 incomplete for post-T187 plain/wrong-key classes
- **Not absorbed:** Restore/verify redesign; prune; MSI; clap 5; rusqlite 0.40; demote all `ai_brains*=info`; T210 POLICY_DENIED; T211 ranking; header.rs redundant magic check (L4)
- **Research date:** 2026-08-04
- **AI fold-in:** 2026-08-04 — AI1 affirms F2–F9/flags/table. AI2 **M1–M5** accepted; **L1–L3** elevated/soft; **L4–L6** notes. Disposition **§14**.
- **Ledger:** plan-only until go

## 1. Objective

1. **Classify** each `vault-*.db.bak` as openable-with-current-key vs **legacy plain** vs **unreadable** (wrong key / corrupt), using the same plain-header sniff SOOT as vault open / `backup verify` (T187).  
2. **Default quiet honesty:** no per-file `WARN` flood for expected post-encrypt residuals (pre-T187 plain backups; wrong-key after rotate). One **summary** line when any expected skips exist.  
3. **Table honesty:** class tokens instead of always `(no metadata)`.  
4. **`--verbose`:** per-file detail. **`--quiet`:** stronger silence (no summary).  
5. Do **not** change restore/encrypt semantics, exit codes, or capture independence.

## 2. Live baseline (code re-scan + AI2 verify 2026-08-04)

### 2.1 Audit signal — confirmed live (AI2)

| Signal | Detail |
|--------|--------|
| Audit score | **backup list 4/3** |
| Live default | Plain-header `vault-*.db.bak` → `WARN … Backup key verification or open failed … error=Key verification failed: file is not a database`; table `(no metadata)` ×3 |
| Live `--quiet` | WARN silenced; table still `(no metadata)` — **F9 gap** |
| Encrypted salt | Real SQLCipher backup first 16 bytes random → `is_plain_sqlite_header` **false** (no false-positive plain) |
| Prior demotes | **T120** openable pre-T109 → `debug!`; **T134** `--quiet` opt-in |
| Native spam | **T197** filters CORE hmac lines — **not** our `tracing::warn!` |

### 2.2 Code map — confirmed (AI2 nuance)

| Site | Role / gap |
|------|------------|
| `list_backups(quiet)` ~310–408 | No plain-header branch. Plain **invalid** short files: open may succeed, key pragmas set OK, schema read fails → **Err(key_err)** → `warn!` unless quiet. Valid plain SQLite with schema could hit other arms — F3 short-circuits all. |
| `read_backup_metadata` | Always open + key + meta SELECT |
| `header.rs` `is_plain_sqlite_header` | Missing/empty/unreadable → **false** (M1); re-exported from store |
| `run_list` | `(no metadata)` only — F9 |
| `verify_single_backup` | Already refuses plain — F13 regression only |
| `BackupCommands::List { quiet }` | No `--verbose` — F7 |
| `doctor` `list_backups(true)` | Must keep working under **ListMode::Quiet** (M3) |
| Unit `list_backups__{quiet,not_quiet}__*` | Length only — F22 must add class/noise asserts |
| Smoke pre-T109 / corrupt | T120 + AC2 anchors |

### 2.3 Failure classes after T187 (normative)

| Class | Detection (header-first + F31) | Default noise | Table token (when meta empty) |
|-------|--------------------------------|---------------|-------------------------------|
| **Readable** | Meta SELECT succeeds under current key | silent | Real values |
| **PreT109** | Key opens; core tables; no `_aibrains_backup_meta` | `debug!` only | `(no metadata)` |
| **LegacyPlain** | `is_plain_sqlite_header(path)` **before** key probe | `debug!` + summary count | **`(legacy plain)`** |
| **KeyMismatch** | Not plain; file size **≥ 512**; key/schema fail after open path | `debug!` + summary count | **`(unreadable key)`** |
| **Corrupt** | Not plain **and** (I/O open failure **or** file size **&lt; 512** **or** missing/empty — M1) | **`tracing::warn!` per-file** | **`(corrupt)`** |

**Footnote (M1):** `is_plain_sqlite_header` returns `false` for missing, empty, or unreadable files. Those fall through classify: open fails or size &lt; 512 → **Corrupt** → per-file WARN (acceptable for TOCTOU/permission races).

**Zetetic note:** Encrypted salt in first 16 bytes (random). Plain magic is the pre-key SOOT. `PRAGMA key` does not encrypt existing plain DBs. Key proof = post-key schema read. **Error strings alone cannot separate garbage from wrong-key** (both often `file is not a database` after lazy open) — **F31 size heuristic is required**.

### 2.4 Touch map

| File | Role |
|------|------|
| `crates/ai-brains-brain/src/backup.rs` | `BackupReadClass`, `ListMode`, classify, `list_backups(mode)`, `BackupInfo.class` |
| `crates/ai-brains-cli/src/commands/backup.rs` | `run_list` tokens + **eprintln!** summary (L2); mode from flags |
| `crates/ai-brains-cli/src/commands/doctor.rs` | `list_backups(ListMode::Quiet)` (M3) |
| `crates/ai-brains-cli/src/main.rs` | `List { quiet, verbose }` — **no** `conflicts_with` (M4) |
| `crates/ai-brains-cli/tests/backup_list_honesty.rs` (preferred) | Hermetic AC1–AC7 |
| Brain unit tests | Pure classify + update existing list call sites |
| `Docs/CAPABILITIES.md` §11 | + CHANGELOG; soft OPERATIONS |

### 2.5 Deps

| Crate | Pin / note |
|-------|------------|
| `rusqlite` | **0.39.0** + sqlcipher features — keep (0.40.1 out of scope) |
| SQLCipher | Community **4.10.0** |
| `tracing` / subscriber | **0.1.44** / **0.3.23** latest line — keep |
| **Zero new crates** | F15; reuse `is_plain_sqlite_header` |

## 3. Research summary

| Finding | Application |
|---------|-------------|
| Live plain → WARN + `(no metadata)` (AI2) | F2/F3 + F9 |
| Error string same for garbage vs wrong-key (M2) | **F31 size ≥512** discriminator |
| Missing/empty → plain false (M1) | Corrupt footnote |
| Doctor `list_backups(true)` (M3) | **ListMode** enum + Quiet |
| clap both flags (M4) | Runtime quiet wins — not conflicts_with |
| T208 RUST_LOG denylist (M5) | AC1 `env_remove`; AC2 `.env(warn)` |
| T206 mismatch style (L2) | Summary **eprintln!** |
| clig.dev | Creator diagnostics → debug for expected classes |
| Zetetic | Header-first + post-key schema read |

## 4. Frozen decisions (F1–F35)

| ID | Decision |
|----|----------|
| **F1 — Scope** | List classify + default noise + table tokens + `--verbose`. Not restore/verify redesign, prune, packaging. |
| **F2 — SOOT classify (required)** | Header-first then key probe. **Only** `ai_brains_store::is_plain_sqlite_header` — no duplicate magic. |
| **F3 — Header before key** | Plain header → **LegacyPlain** immediately; **no** key_probe. |
| **F4 — PreT109 preserved** | T120 path stays `debug!`. Smoke `backup_list__pre_t109_backup__no_warn_on_stderr` green. |
| **F5 — Default noise** | Per-file `tracing::warn!` **only** for **Corrupt**. LegacyPlain + KeyMismatch + PreT109 → `debug!`. Summary when legacy_plain + key_mismatch ≥ 1 (F6). |
| **F6 — Summary channel (L2 elevated)** | **Required:** one **`eprintln!`** line (always visible; not EnvFilter-dependent), T206 style. **Not** `tracing::warn!` for the summary (avoids double-counting with log-format). Template substring classes: `not fully readable` (or `legacy plain`), `--verbose`, `verify`. Zero summary when only Readable/PreT109. Deterministic counts. **Omit summary under `--verbose`** when per-file detail is emitted (F7). |
| **F7 — `--verbose`** | Per-file detail (tracing `warn!` or human lines) for LegacyPlain, KeyMismatch, Corrupt; PreT109 stays debug. Prefer **omit** summary when verbose. |
| **F8 — `--quiet` (M4)** | No per-file metadata WARNs **and** no summary. Table labels still apply (honesty). **Runtime priority:** if both flags, **quiet wins**. **Do not** use clap `conflicts_with` (hostile error on dual flags). |
| **F9 — Table honesty** | Empty meta → `(legacy plain)` / `(unreadable key)` / `(corrupt)` / PreT109 `(no metadata)`. Filename/timestamp unchanged. |
| **F10 — Exit codes** | List exit **0** on successful scan with any mix of classes. Dir I/O still errors. |
| **F11 — Capture independence** | Unchanged. |
| **F12 — Doctor / prune (M3)** | Doctor: `list_backups(ListMode::Quiet)`. Prune untouched. Soft class in doctor not DoD. |
| **F13 — Verify / restore** | Regression only: verify still refuses plain. |
| **F14 — API shape (M3 + L1 required)** | **`ListMode` enum:** `Default` \| `Quiet` \| `Verbose` with `ListMode::from_flags(quiet: bool, verbose: bool)` (quiet wins). `list_backups(mode: ListMode)`. **`BackupReadClass`** on **`BackupInfo.class`** (CLI must not re-sniff). Soft: `#[derive(Serialize)]` on class for F24 (L3). Update all call sites (CLI, doctor, brain tests) in same change. |
| **F15 — Zero new crates** | — |
| **F16 — Hermetic env (M5)** | **AC1:** `env_remove("RUST_LOG")` (product default filter) — prove no per-file WARN. **Never** prove AC1 with only `RUST_LOG=warn` as the sole “quiet default” story. **AC2:** `.env("RUST_LOG", "warn")` after denylist strip so Corrupt `tracing::warn!` is visible. Tempdir vaults; denylist already has `RUST_LOG` (T208 F29). |
| **F17 — High findings** | Default per-file WARN for LegacyPlain/KeyMismatch; auto-delete plain; flip verify/restore; require `--quiet` for clean list; rusqlite bump; silent skip without table tokens; clap conflicts_with on quiet+verbose; AC1 via empty `RUST_LOG=""` false patterns where default matters. |
| **F18 — Key material** | Never log key bytes. |
| **F19 — Review** | FEATURE; primary required. Cross-model soft. |
| **F20 — Series** | After T208; before T210. |
| **F21 — Determinism** | Stable summary template; path sort already newest-first. |
| **F22 — Tests** | Unit classify: plain / garbage&lt;512 / large key-fail KeyMismatch / readable. Hermetic AC1–AC7. Existing length-only units updated to new API + class where useful. |
| **F23 — Docs** | CAPABILITIES §11 + CHANGELOG. Soft OPERATIONS. |
| **F24 — Soft residual** | Soft `--json` + class; soft doctor class; **not** “finer split” — F31 is DoD discriminator. |
| **F25 — T120/T134** | Extends; does not reopen timestamp work. |
| **F26 — Soft decline** | rusqlite 0.40 · auto-migrate plain · MSI · clap 5 · ranking · POLICY_DENIED · demote all info · header.rs L4 cleanup |
| **F27 — Ledger** | On go: `ledgerful ledger start T209-backup-list-sqlcipher-honesty --category FEATURE`. |
| **F28 — Implement order** | C1 enum+class → C2 classify header-first+F31 → C3 list_backups(mode) noise → C4 clap from_flags → C5 tokens+eprintln summary → hermetic → docs → gate. |
| **F29 — sqlcipher_log_policy** | Keep install; no string changes for DoD. |
| **F30 — Hint language** | Summary may cite verify / re-backup after encrypt; list does **not** auto-upgrade plain. |
| **F31 — KeyMismatch vs Corrupt (M2 resolved)** | Discriminator (**required**, not soft): after not-plain: **(1)** I/O failure opening file or `metadata` → **Corrupt**; **(2)** `len < 512` → **Corrupt** (covers smoke garbage `not a valid sqlite database` and empty/truncated junk); **(3)** `len ≥ 512` and key/schema verification fails → **KeyMismatch** (summary, not per-file WARN). Rationale: min SQLite page is 512; real vault backups are multi-page; error strings alone cannot split garbage vs wrong-key (AI2). Constant name e.g. `MIN_PLAUSIBLE_BACKUP_BYTES: u64 = 512`. |
| **F32 — AI fold-in** | §14 applied 2026-08-04. |
| **F33 — Plain fixture honesty (AC1)** | AC1 plain fixture should use **valid plain SQLite magic** (and preferably a minimal valid DB or magic+padding that still sniffs plain). Classification is header-based — do not rely on key_probe for AC1. |
| **F34 — AC2/AC9 non-conflict** | AC2 garbage short file → Corrupt → per-file `tracing::warn!`. AC9 large wrong-key backup → KeyMismatch → summary only (no per-file WARN). Both required for M2 proof. |
| **F35 — ISSUES.md (L6)** | If any medium deferred at closeout, create/append `conductor/ISSUES.md` (file may be missing repo-wide). Not a DoD otherwise. |

## 5. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Plain-header bak under default list: **no** per-file key/metadata `WARN`; table `(legacy plain)` | Hermetic + **`env_remove("RUST_LOG")`** (F16) |
| **AC2** | Short garbage bak: **per-file** Corrupt `WARN` (`RUST_LOG=warn`) | Hermetic / adapt smoke |
| **AC3** | ≥2 LegacyPlain: **≤1** `eprintln` summary; no N key WARNs | Hermetic |
| **AC4** | `--verbose`: per-file detail for LegacyPlain and/or Corrupt | Hermetic |
| **AC5** | `--quiet`: no summary, no metadata WARN; dual `--quiet --verbose` → quiet wins | Hermetic |
| **AC6** | Pre-T109 keyed meta-stripped: no WARN | Existing smoke |
| **AC7** | Tokens + T121 path-end for readable | Hermetic stdout |
| **AC8** | CAPABILITIES §11 + CHANGELOG | Doc |
| **AC9** | Large wrong-key encrypted bak → summary / KeyMismatch token; **no** per-file WARN flood | Hermetic (**required** for F31 — elevated from soft) |
| **AC10** | Soft: doctor `backup_recent` healthy with plain residual | Soft/manual |

## 6. Non-goals

- Auto-delete / auto-encrypt plain backups  
- Restore/verify redesign  
- rusqlite 0.40  
- JSON list as DoD  
- MSI / clap 5 / T210+  
- Cleaning `header.rs` starts_with redundancy (L4)  

## 7. Risk & verification

| Risk | Mitigation |
|------|------------|
| AC2 vs AC9 conflict | **F31** size split + **F34** / AC9 required |
| Over-quiet wrong-key | Summary + table `(unreadable key)` still visible |
| Misclassify encrypted as plain | Magic only; AI2 salt check |
| Doctor break | **ListMode::Quiet** (M3) |
| clap dual-flag error | **F8** runtime priority only |
| AC1 false filter story | **F16** env_remove for AC1 |
| Summary lost under log-format off | **F6** eprintln! |

**Implement order:** F14 enums → F2/F3/F31 classify → F5–F8 noise → F9 table + F6 summary → hermetic F16 → docs → gate.

## 8. Residual after ship

- Soft F24 `--json` + class  
- Soft doctor class display  
- Soft F35 ISSUES only if deferrals  
- Operator: plain files remain until re-backup after encrypt  

## 9. Series

… → T208 closed → **T209** → T210 → …

## 10. Normative behavior

### 10.1 Default (modern + legacy plain)

```text
Filename … Timestamp … Source Vault … Version … Size
vault-2026-08-04T12-00-00.db.bak  …  …\vault.db  …  …
vault-2026-05-01T10-00-00.db.bak  …  (legacy plain)  (legacy plain)  (legacy plain)
N backup(s) not fully readable (legacy plain or current key): use --verbose or ai-brains backup verify
```

(Summary via **eprintln!**; pin substrings in tests.)

### 10.2 `--quiet`

Table with class tokens; **no** summary; no metadata WARNs.

### 10.3 `--verbose`

Per-file detail for non-readable classes; **no** summary line preferred.

### 10.4 Dual flags

`backup list --quiet --verbose` → **Quiet** behavior.

## 11. Manual verification (on go)

1. Post-T187-only backups → clean, no summary.  
2. Plain-header bak → tokens + one summary.  
3. Short garbage → per-file WARN.  
4. Quiet / verbose / both matrix.  
5. Verify plain still fails closed.  
6. Soft: doctor with plain residual.

## 12. Stop-before

- Auto-delete plain backups  
- rusqlite major bump  
- T160 exit changes for list  
- T210 scope  

## 13. Done when

AC1–AC9 green (AC10 soft); review clear for >low; full gate; conductor Completed; deferred T209 struck; audit 4/3 residual closed.

## 14. AI fold-in disposition (2026-08-04)

| ID | Source | Action |
|----|--------|--------|
| **AI1 #1** | Header-first before key | **Affirm** → F2/F3 |
| **AI1 #2** | Five-class model + default noise | **Affirm** → F5 + §2.3 |
| **AI1 #3** | Table tokens | **Affirm** → F9 |
| **AI1 #4** | `--verbose` / `--quiet` quiet wins | **Affirm** → F7/F8 |
| **AI1 summary table** | Touch map | **Affirm** — already plan C* |
| **M1** | Missing/empty plain=false → open-fails | **Accept** → §2.3 footnote + Corrupt |
| **M2** | Error string cannot split KeyMismatch/Corrupt | **Accept** → **F31 size≥512** + **F34** + AC9 **required** |
| **M3** | list_backups API / doctor | **Accept** → **F14 ListMode** + F12 Quiet |
| **M4** | No clap conflicts_with | **Accept** → F8 runtime priority |
| **M5** | AC1 env_remove vs AC2 RUST_LOG=warn | **Accept** → **F16** |
| **L1** | BackupInfo.class | **Elevate required** → F14 |
| **L2** | eprintln! summary | **Elevate required** → F6 |
| **L3** | Serialize class for later JSON | **Soft accept** → F24 |
| **L4** | header starts_with redundancy | **Decline** — out of scope |
| **L5** | verify plain refuse | **Affirm** → F13 |
| **L6** | ISSUES.md missing | **Soft** → F35 |
| **AI2 live repro** | WARN text + salt check | **Affirm** → §2.1 |
| **AI2 clig/Zetetic** | Align | **Affirm** → §3 |
