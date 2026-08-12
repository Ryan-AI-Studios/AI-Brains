# T244 — Backup recoverability fleet

- **Track ID:** T244-BackupRecoverabilityFleet
- **Status:** ✅ **Completed** (2026-08-12 PR #149 `948d2ae`)
- **Category:** OPS / FEATURE / UX
- **Owner:** Grok
- **Source:** CLI audit 2026-08-11 P1 — backup fleet **0 OK / 21 FAIL** legacy; list **Q7**; verify **E7**; doctor `backup_recent` warn; post-T225 residual honesty gap
- **Depends on:** T225 quiet verify + usable nudge (shipped PR #128 `927b8db`); T209 list honesty; T187 SQLCipher; T131/T138 verify; T192 doctor; T104/T126 retention; T181 recovery drills (docs)
- **Blocks / feeds:** Honest DR green path for operators; DataKey rotation gate (“verified recent backup”); RECOVERY-DRILLS credibility
- **Absorbs:** deferred.md “Backup fleet 0 usable / legacy plain” → this track; T225 soft F17 class honesty where it closes false-usable (core-table gate); list residual-count undercount of non-recoverable PreT109
- **Not absorbed (DoD):** Auto-delete legacy plain without explicit operator flag; nightly auto-create; MSI; clap 5; rusqlite **0.40+** bump; restore redesign; verify exit 0 on residual FAIL; full structured `VerifyError` 4-class JSON schema as hard DoD; T209 L3 real wrong-key fixture; verify `--quiet` flag; JSON verify `summary` field
- **Research date:** 2026-08-12 (live dogfood + code re-scan + dep pins + enterprise recovery-proof practices + T225/T209 residuals)
- **AI fold-in:** 2026-08-12 — `C:\dev\AI-review.md` AI1 + AI2. No Highs. **AI1** affirms F1–F7 / AC1–AC11 (M1–M4 restatement of design). **AI2 hard:** M1 SOOT migration list (8 `backup_list_honesty` sites); M2 F5 = `tables_out.len() < 2` (preserve JSON `tables`, no drop field); M3 F7 sort **CLI `run_list` only** (brain `list_backups` stays `Reverse(timestamp)` for doctor). **Hard lows:** L3 single `is_usable_class` SOOT; L4 rename `empty_meta_token` → `backup_class_token`; L9 Incomplete fixture recipe; L11 Incomplete noise pattern. **Agree soft:** L1/L2 has_core_tables perf; L5 keep create-only remediation; L6 F11 expected outputs; L7 u64::MAX max-age; L10 serde; L13 race out of scope. **Upgrade:** F25 cross-model **hard** on classify+doctor (data-safety / false recovery hope — AI2 O13). Disposition **§12**.
- **Ledger:** plan-only until go (`ledgerful ledger start T244-backup-recoverability-fleet --category FEATURE`)

---

## 1. Objective

1. **Recoverability green path:** after create (or existing good snapshot), fleet has **≥1 verify-OK** encrypted backup under the current key, and doctor `backup_recent` reflects that honestly.
2. **Stop false hope:** openable-but-not-restorable residuals (key opens, **missing product core tables**) must **not** count as doctor “usable.”
3. **List clarity:** label residual fleet (legacy plain / incomplete / key / corrupt); **do not bury** usable rows; residual summary counts **all non-usable**.
4. **Keep T225 quiet verify** (counts + first 5 FAIL + create nudge when ok==0).
5. **Capture independence; zero new crates; no auto mass-delete of history.**

---

## 2. Live baseline (re-scan 2026-08-12)

### 2.1 Dogfood (this workspace vault)

| Signal | Observation |
|--------|-------------|
| Fleet size | **21** `vault-*.db.bak` under repo `backups/` |
| `backup list` (default) | Mix of `(legacy plain)` + `(no metadata)` tokens; stderr residual summary **14** “not fully readable (legacy plain or current key)” — **undercounts** incomplete openable files |
| `backup verify` (default) | `Verified 21 backups: 0 OK, 21 FAIL.` + first 5 FAIL — + trailer + create nudge. Exit **1**. Quiet UX from T225 works. |
| FAIL reasons | Legacy plaintext header **or** `backup is missing core tables` (openable PreT109-class shells without `events`/`memory_projection`) |
| `doctor` `backup_recent` | **warn:** `newest usable backup older than 7d (timestamp 2026-06-22T21:55:16)` — treats a **PreT109 `(no metadata)`** row as usable even though **verify FAILs** it |
| `backup create --dry-run` | Would write `backups\vault-<now>.db.bak`; source vault ~**78 MB**; default prune would drop 11 old of 21+1 |
| Operator path | Create **exists** and works; fleet simply never recreated under live SQLCipher after encrypt era |

### 2.2 Why “0 usable” is both product + ops

| Layer | Truth |
|-------|--------|
| Integrity (verify) | Correct: **0** snapshots pass key + integrity + core-table checks. |
| Presentation (list) | Partial: tokens honest per T209, but residual **summary excludes** openable-no-core; chronological table buries any future Readable under a sea of residuals. |
| Health (doctor) | **False recoverability hope:** usable = `Readable \| PreT109` without requiring core tables. PreT109 debug text claims “core tables present” but `classify_backup_read` never calls `has_core_tables`. |
| Ops | No recent `backup create` after SQLCipher / schema maturity. |

T225 closed **noise**. T244 closes **fleet recoverability honesty + green path**.

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|--------|
| Classify | `ai-brains-brain` `backup.rs` `classify_backup_read` | Header → LegacyPlain; key fail → KeyMismatch/Corrupt; key ok → meta? Readable : PreT109. **No core-table gate.** |
| `has_core_tables` | same file | Requires **both** `events` and `memory_projection`. Used by doctor `schema_readable`, **not** by classify/list usable. |
| Verify core check | `cli` `backup.rs` `verify_single_backup` | `IN ('events','memory_projection')` fills `tables_out` for JSON; then **`is_empty()` → fail**. Passes if **either** table present — drift vs `has_core_tables` (both). **F5 pin:** keep IN collection; gate with `len() < 2` (AI2 M2). |
| List honesty SOOT | `backup_list_honesty.rs` | **8** assertions on substring `not fully readable` — **F6** migrates to `not recoverable` (AI2 M1). |
| Brain list sort | `list_backups` | `Reverse(timestamp)` — doctor `find_map` newest usable depends on it. **F7 sort must not change brain** (AI2 M3). |
| List residual count | `run_list` | Counts only `LegacyPlain \| KeyMismatch` for summary |
| Doctor usable | `doctor.rs` `check_backup_recent` | `Readable \| PreT109` only; age newest usable |
| Create | `run_create` → `run_backup_from_conn` | Online Backup API; prints `Backup created and verified:` |
| T225 report | `verify_report.rs` | Counts, preview cap 5, create nudge |
| Contracts | `ai-brains-contracts` backup/doctor | No public `BackupReadClass` DTO; class stays brain-local |
| Hermetic | `backup_list_honesty`, `doctor_cli`, smoke verify | Must migrate if class/usable SOOT changes |

### 2.4 Dependency / standards research (2026-08-12)

| Pin | Workspace / lock | Ecosystem | Action |
|-----|------------------|-----------|--------|
| `rusqlite` | **0.39.0** + `bundled-sqlcipher-vendored-openssl` | crates.io **0.40.2** | **No bump** (SQLCipher build risk; series non-goal) |
| `chrono` | **0.4.x** (lock **0.4.44**) | stable 0.4 | **No bump** |
| `tracing` | **0.1.x** | stable | **No bump** |
| SQLCipher | bundled via rusqlite features | 4.15.x ecosystem notes (export defensive mode) | No product change; create stays Online Backup API path |

**External recoverability practices (2026):** recovery **proof** > “file exists”; verify/restore drills; fleet health = recent **valid** backup, not newest timestamp among trash. Aligns with T181 RECOVERY-DRILLS (“Do not treat a backup file exists as recovery proof”) and this track’s usable SOOT.

### 2.5 Prior track residuals rolled in

| Source | Item | T244 disposition |
|--------|------|------------------|
| deferred.md audit | Backup fleet 0 OK / 21 FAIL legacy | **DoD** this track |
| T225 F9 | usable = Readable\|PreT109 | **Tighten** with core-table gate (F1–F3) |
| T225 F17 | structured VerifyError / 4-class / `--quiet` / JSON summary | Soft only; optional **3-class human rollup** if cheap (F17 soft) |
| T225 F7 | optional 3-bucket rollup | Soft-DoD optional |
| T209 residual L3/L4 | wrong-key fixture / PreT109 unit | Soft; PreT109+core unit becomes **hard** via F1 tests |
| Placeholder F4 | archive/quarantine legacy | Soft (F18); prune remains operator path |

---

## 3. Problem analysis

1. **Doctor “usable” ≠ verify OK** on live fleet → operator can believe a 7d-stale PreT109 is the recovery candidate when restore would fail core tables.
2. **PreT109 semantic drift:** comment/debug say “core tables present”; classify only checks meta absence.
3. **List residual summary** undercounts incomplete openables → audit Q7 “legacy wall” without fleet rollup.
4. **Green path is already coded** (`backup create`); missing is honesty + optional list ordering + docs + live dogfood create on go.
5. Track is **classification SOOT + presentation + docs + hermetic proof + live create on go** — not a new backup engine.

---

## 4. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Plan-only** | No production code, no live `backup create` (mutating), no prune of live fleet until **go**. Dry-run research OK. |
| **F1 — Core-table gate in classify (hard)** | After key opens successfully, call `has_core_tables(&conn)` **before** meta classification. If **false** → class **`Incomplete`** (new `BackupReadClass` variant) with table token **`(no core tables)`**. Never call this class “usable.” |
| **F2 — PreT109 definition (hard)** | `PreT109` = key opens **and** `has_core_tables` **and** meta table/rows unusable/absent. Debug message claim becomes true. |
| **F3 — Readable definition (hard)** | Unchanged intent: key opens + core tables + meta SELECT success with rows map. |
| **F4 — Doctor usable SOOT (hard)** | `usable` = `Readable \| PreT109` **only** via pure `is_usable_class` (both imply core tables post-F1). Zero usable → warn + remediation **`ai-brains backup create` only** (AI2 L5 — do **not** append verify to remediation string; verify is the operator next step). Age **newest usable** only. Prefer `list_backups` brain order unchanged (F7). |
| **F5 — Verify core SOOT (hard / AI2 M2)** | Keep the existing `IN ('events','memory_projection')` query that fills `tables_out` for JSON `VerifyResult.tables`. Change gate from `tables_out.is_empty()` to **`tables_out.len() < 2`** (both names required). Fail reason substring **`missing core tables`** frozen. **Do not** replace the IN query with `has_core_tables` alone (would empty JSON `tables`). Classify uses `has_core_tables` (bool); verify uses len gate — both require both tables. |
| **F6 — List residual summary (hard / AI2 M1)** | Default residual count = `residual_for_summary` = `!is_usable_class` (all non-usable). **Pinned SOOT:** `{n} backup(s) not recoverable under current key (legacy plain / incomplete / key / corrupt): use --verbose or ai-brains backup verify`. **Stable assert substring:** `not recoverable under current key` (migrate **8** `backup_list_honesty.rs` sites off `not fully readable`). Quiet/verbose rules unchanged. |
| **F7 — List sort CLI-only (hard / AI2 M3)** | **`run_list` only** re-sorts usable-first after `list_backups`. **Brain `list_backups` sort stays `Reverse(timestamp)`** — doctor `check_backup_recent` `find_map` newest usable depends on it. Sort key: `(priority: u8, Reverse(Option<NaiveDateTime>), PathBuf)` — priority 0 = Readable\|PreT109, 1 = residual. **`Reverse(None)` sorts last within band** (unparseable ts at bottom of band — desirable; pin F23). |
| **F8 — List tokens (hard / AI2 L4)** | Rename `empty_meta_token` → **`backup_class_token`** (Incomplete is not “empty meta”). Incomplete → `(no core tables)`; keep existing tokens for other classes. |
| **F9 — Verify presentation frozen (hard)** | Keep T225: quiet counts + first 5 FAIL — + trailer + create nudge when `ok==0 && total>=1`; `--verbose` stream only; JSON full; any FAIL → exit 1; empty → 0. |
| **F10 — Create path (hard)** | No new create engine. Docs + CAPABILITIES recoverability paragraph: after encrypt / zero-usable doctor warn → `ai-brains backup create` then `backup verify` expect ≥1 OK. Create success string may stay `Backup created and verified:`. |
| **F11 — Live dogfood (hard on go / AI2 L6)** | **Mutating**, only with go. Prefer: `ai-brains backup create --no-prune` → expect fleet **22** (1 new + 21 legacy). **Expected:** verify `1 OK, 21 FAIL`; list usable-first with new Readable on top; doctor `backup_recent` ok (ages the new OK). If operator uses default keep-10, record prune deleted count instead. Record exact commands/outputs in plan. |
| **F12 — Hermetic green path (hard / AI2 L7)** | Temp vault + key → create → list Readable → verify 1 OK → doctor ok with max-age **`18446744073709551615d`** (`u64::MAX` days — existing `doctor_cli` pattern). Separate Incomplete path for zero-usable. |
| **F13 — Incomplete fixture (hard / AI2 L9)** | Recipe: `Connection::open` + `apply_key_pragmas` + `CREATE TABLE junk(x)` — **no** `events`/`memory_projection`. Classify → Incomplete; list token `(no core tables)`; doctor not usable; verify FAIL `missing core tables`. Single-table shell (only `events` or only `memory_projection`) also FAIL verify `len() < 2`. |
| **F14 — Capture independence** | No models/graph deps on backup path. |
| **F15 — Zero new crates** | No rusqlite 0.40, no chrono bump required. |
| **F16 — No auto mass-delete (hard)** | Do **not** auto-delete legacy plain as DoD. Operator uses existing `backup prune` / manual move. Soft F18 only for optional archive helper. |
| **F17 — Soft residuals** | verify `--quiet`; JSON verify `summary`; structured `VerifyError` enum in JSON; optional human 3-class rollup on default verify; T209 L3 wrong-key real fixture; clap archive subcommand; nightly scheduled create. |
| **F18 — Soft archive (optional)** | If cheap: document `backups/legacy/` manual quarantine; **or** `backup prune --older-than` recipe. No silent move. |
| **F19 — Contracts (AI2 L10)** | No shared DTO change. `BackupReadClass` already `Serialize`; new variant serializes as `"Incomplete"`. No contract consumer today. |
| **F20 — High findings if…** | Doctor ok on Incomplete-as-usable; classify PreT109 without core tables; list residual undercount Incomplete; verify accepts only one core table; live go without ≥1 OK; auto-delete legacy; rusqlite bump; capture/graph dep; JSON `tables` emptied by F5 mistake; brain sort broken for doctor. |
| **F21 — Parallel-friendly** | Touches `ai-brains-brain/backup.rs`, CLI `backup.rs` list/verify, `doctor.rs` usable filter, list/doctor/smoke tests, CAPABILITIES §11 / OPERATIONS Backup. Low conflict with T241/T243/T245 if they avoid backup/doctor. Soft parallel with T249 doctor presentation. |
| **F22 — Docs (hard / AI2 O12)** | CAPABILITIES §11: Incomplete class + usable SOOT + list sort + residual wording + **§7 decision table** + green path create→verify→doctor. OPERATIONS Backup recipe. CHANGELOG T244. Soft RECOVERY-DRILLS one-liner. |
| **F23 — Determinism** | Fixed class priority; `Reverse(Option<ts>)` None last within band; path tiebreaker; stable SOOT strings. |
| **F24 — Serde / Default** | `BackupReadClass` gains `Incomplete`; `Default` stays `Readable`; exhaustive matches compile-fail until all sites updated (`backup_class_token`, `emit_list_noise`, residual, doctor filter, tests). |
| **F25 — Ledger / review (AI2 O13 hard)** | On go: ledger start FEATURE. Primary review OPS/UX. **Cross-model review HARD** on F1/F4 classify gate + doctor usable (data-safety: false usable = false recovery hope). |
| **F26 — Exit codes frozen** | verify any fail → 1; list always 0 after successful scan; doctor soft backup_recent never alone forces fail (unchanged matrix). |
| **F27 — Incomplete noise (hard / AI2 L11)** | `emit_list_noise`: Incomplete = **`debug!`** in Default/Quiet, **`warn!`** in Verbose (same pattern as LegacyPlain/KeyMismatch). Message class includes `missing core tables` / `events` / `memory_projection`. |
| **F28 — Perf notes (soft pin)** | Keep `has_core_tables` two `query_row`s as-is (AI2 L1). F1 adds ~2 queries per classified file (~42 for 21-file fleet) — acceptable; no batching (AI2 L2). Create-vs-list mid-write race is **existing** behavior, not T244 scope (AI2 L13). |
| **F29 — Plan-only / go** | No production code until **go**. |

---

## 5. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Key-open file **without** both core tables → `Incomplete` + token `(no core tables)` | Unit classify + list honesty hermetic |
| **AC2** | Key-open + both cores + no meta → `PreT109` (not Incomplete) | Hermetic / unit |
| **AC3** | Doctor: only Incomplete+plain fleet → warn **no usable** + create remediation (not “stale usable” on Incomplete timestamp) | `doctor_cli` hermetic |
| **AC4** | Doctor: Readable (or PreT109 with cores) in-age → `backup_recent` ok | Hermetic |
| **AC5** | Doctor: usable stale + fresher Incomplete/plain → ages usable only / still warn stale | Discriminating hermetic (T225 style) |
| **AC6** | List residual summary counts all non-usable; stderr contains `not recoverable under current key` | Hermetic + migrate `backup_list_honesty` (8 sites) |
| **AC7** | List order: usable before non-usable when mixed; brain `list_backups` order still timestamp-desc for doctor | Hermetic list order + doctor still finds newest usable |
| **AC8** | Verify requires both core tables (`len() < 2`); single-table shell FAIL `missing core tables`; JSON `tables` still populated from IN query | Hermetic verify + JSON guard |
| **AC9** | T225 quiet verify still: counts, ≤5 FAIL —, trailer, nudge when ok==0 | Existing smoke green + guards |
| **AC10** | Hermetic create → ≥1 verify OK under test key | Integration / brain backup test or CLI hermetic |
| **AC11** | Docs CAPABILITIES §11 (incl. decision table) + OPERATIONS Backup + CHANGELOG | Review |
| **AC12** | Live dogfood on go: create → verify ≥1 OK → doctor backup_recent ok (or age-ok) | Manual evidence in plan |
| **AC13** | Full gate: fmt, clippy -D warnings, nextest workspace, deny, audit | CI / local |
| **AC14** | Exhaustive match sites for new class compile clean; no `unwrap`/`expect` in production paths touched | clippy + review |
| **AC15** | Capture independence preserved on backup path | grep / architecture review |
| **AC16** | SOOT migration: no remaining test asserts `not fully readable` for list residual | grep + nextest |
| **AC17** | Incomplete noise: Default quiet residual uses debug path (no WARN flood); Verbose may warn | Hermetic / unit |

---

## 6. Pure helpers (preferred extraction)

Prefer pure functions unit-tested first (TDD Red):

```text
// brain (preferred) — names pinned
fn is_usable_class(class: BackupReadClass) -> bool
// Readable | PreT109

fn residual_for_summary(class: BackupReadClass) -> bool
// !is_usable_class(class)  — single SOOT (AI2 L3)

// CLI-local for run_list only (AI2 M3)
fn list_sort_key(info: &BackupInfo) -> (u8, Reverse<Option<NaiveDateTime>>, PathBuf)
// priority 0 usable, 1 residual; Reverse(ts) None-last; PathBuf tiebreak

// classify_backup_read: after key_ok
//   if !has_core_tables(&conn) -> Incomplete
//   else meta branch -> Readable | PreT109

// verify_single_backup: keep IN query → tables_out; gate tables_out.len() < 2
```

---

## 7. Decision table — usable / class

| Key opens | Core tables (both) | Meta OK | Class | Doctor usable | Verify (integrity ok) |
|-----------|--------------------|---------|-------|---------------|------------------------|
| n/a plain header | — | — | LegacyPlain | no | FAIL legacy plain |
| no | — | — | KeyMismatch / Corrupt | no | FAIL key |
| yes | no | — | **Incomplete** | **no** | FAIL missing core tables |
| yes | yes | no | PreT109 | **yes** | OK if integrity passes |
| yes | yes | yes | Readable | **yes** | OK if integrity passes |

---

## 8. Non-goals

- Deleting or encrypting-in-place the historical 21-file fleet as an automatic product action
- Changing restore daemon hard-fail (T188)
- Nightly auto-backup schedule
- rusqlite 0.40 migration
- Making verify exit 0 when residuals FAIL
- Claiming NIST Purge or “backup file exists = recovered”

---

## 9. Risk register

| Risk | Mitigation |
|------|------------|
| Live create ~78 MB + default prune deletes 11 old | On go: confirm with user; document prune count; `--no-prune` if available / `--keep` high |
| Doctor tests assume PreT109 without cores | Migrate fixtures to Incomplete vs PreT109+cores |
| List consumers depend on chronological-only order | Spec F7; CAPABILITIES note; hermetic order test |
| Incomplete breaks exhaustive matches | Compile-driven fix all match arms |
| Double-open cost list/doctor | Accept (already open per file for classify); no second open for usable if class carries SOOT |
| Verify either-table reintroduced | F5 `len() < 2` gate + AC8 |
| Brain sort broken for doctor | F7 CLI-only sort pin |
| JSON `tables` emptied | F5 keep IN collection |
| SOOT assert break | F6 migrate 8 honesty sites |

---

## 10. Implement order (on go)

1. Red: pure `is_usable_class` / `list_sort_key` / Incomplete fixture units + SOOT migration stubs  
2. Green: `Incomplete` + classify `has_core_tables` gate + verify `len() < 2`  
3. List residual + CLI usable-first sort + `backup_class_token` rename  
4. Doctor via `is_usable_class` (logic already right post-F1; tests/messages)  
5. Hermetic AC1–AC10, AC16–AC17  
6. Docs (decision table in CAPABILITIES)  
7. Live create dogfood (mutating)  
8. Full gate + **hard** cross-model on classify/doctor + ledger commit  

---

## 11. Manual test script (go)

```powershell
# Non-mutating first
ai-brains backup list
ai-brains backup verify
ai-brains doctor  # note backup_recent

# Mutating green path (operator vault — only on go)
ai-brains backup create --no-prune
ai-brains backup verify              # expect 1 OK, 21 FAIL (if wall retained)
ai-brains doctor                     # backup_recent ok within 7d
ai-brains backup list                # new Readable first; residual summary uses "not recoverable"
```

---

## 12. AI fold-in disposition (2026-08-12)

Source: `C:\dev\AI-review.md` — AI1 + AI2. **No Highs.**

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1–M4** | AI1 | **Agree** (design restatement) | Already F1/F5/F6/F7; no new delta beyond AI2 pins |
| **AI1 L1** | AI1 | **Agree** | F11 `--no-prune` default dogfood |
| **AI1 L2 / O1** | AI1 | **Agree** | Docs F22; unit names in Phase 1 |
| **AI2 M1** | AI2 | **Agree hard** | F6 SOOT migration: 8 `backup_list_honesty` sites → `not recoverable under current key` |
| **AI2 M2** | AI2 | **Agree hard** | F5: `tables_out.len() < 2` (not `has_core_tables` swap) |
| **AI2 M3** | AI2 | **Agree hard** | F7: CLI `run_list` only; brain sort unchanged |
| **AI2 L1–L2** | AI2 | **Agree soft** | F28 perf pins |
| **AI2 L3** | AI2 | **Agree hard** | `residual_for_summary = !is_usable_class` |
| **AI2 L4** | AI2 | **Agree hard** | F8 rename → `backup_class_token` |
| **AI2 L5 / O8** | AI2 | **Agree** | Keep create-only doctor remediation |
| **AI2 L6 / O11** | AI2 | **Agree** | F11 expected 1 OK + 21 FAIL under `--no-prune` |
| **AI2 L7 / O9** | AI2 | **Agree** | F12 u64::MAX max-age |
| **AI2 L8** | AI2 | **Agree** | F17 soft verify `--quiet` stays soft |
| **AI2 L9 / O6** | AI2 | **Agree hard** | F13 junk-table fixture recipe |
| **AI2 L10** | AI2 | **Agree** | F19 `"Incomplete"` serialize |
| **AI2 L11 / O7** | AI2 | **Agree hard** | F27 Incomplete noise pattern |
| **AI2 L12–L13** | AI2 | **Agree** | No gap / race out of scope |
| **AI2 O12** | AI2 | **Agree** | Decision table in CAPABILITIES §11 |
| **AI2 O13** | AI2 | **Agree hard** | F25 cross-model **hard** on classify+doctor |

### Pins locked by fold-in

1. **F5 gate:** `if tables_out.len() < 2 { Err("…missing core tables") }` — keep IN collection for JSON.  
2. **F6 SOOT:** `not recoverable under current key` — migrate honesty tests (lines ~119,177,190,191,227,257,280,353).  
3. **F7 sort:** CLI-only; brain `list_backups` = `Reverse(timestamp)`.  
4. **F8 rename:** `backup_class_token`.  
5. **F13 fixture:** SQLCipher + junk table, no cores.  
6. **F25:** hard cross-model on F1/F4.  
7. **F27:** Incomplete debug/warn noise like LegacyPlain.

---

**Plan-only until go. Live create mutates disk — never without explicit go.**
