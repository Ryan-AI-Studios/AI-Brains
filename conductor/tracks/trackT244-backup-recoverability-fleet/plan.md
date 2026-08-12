# T244 Plan — Backup recoverability fleet

**Status:** ✅ **Completed** (2026-08-12 PR #149 `948d2ae`)  
**Spec:** [spec.md](./spec.md) F0–F29 / AC1–AC17 + §12 AI fold-in  
**Category:** OPS / FEATURE / UX  
**Ledger:** `526e64a0-39ee-474b-b373-820f8a846948`

---

## AI fold-in (2026-08-12) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. Spec design affirmed (AI1). Three AI2 mediums are **must-fold** before go; AI1 mediums restate already-planned work with concrete API shape.

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1–M4** | AI1 | **Agree** | Restate F1/F5/F6/F7 — no design change |
| **AI1 L1** | AI1 | **Agree** | Phase 5: `--no-prune` dogfood |
| **AI1 L2 / O1** | AI1 | **Agree** | Docs + unit names Phase 1 |
| **AI2 M1** SOOT migration 8 sites | AI2 | **Agree hard** | Phase 1: migrate `not fully readable` → `not recoverable under current key` |
| **AI2 M2** verify JSON `tables` | AI2 | **Agree hard** | Phase 2: `tables_out.len() < 2` (not has_core_tables swap) |
| **AI2 M3** sort brain vs CLI | AI2 | **Agree hard** | Phase 3: F7 in `run_list` only |
| **AI2 L1–L2** perf | AI2 | **Agree soft** | F28 notes only |
| **AI2 L3** residual = !usable | AI2 | **Agree hard** | Phase 2 pure helper |
| **AI2 L4** rename token fn | AI2 | **Agree hard** | Phase 2 `backup_class_token` |
| **AI2 L5** create-only rem | AI2 | **Agree** | Keep doctor remediation SOOT |
| **AI2 L6** F11 expected outs | AI2 | **Agree** | Phase 5 pins |
| **AI2 L7** u64::MAX max-age | AI2 | **Agree** | Phase 1/4 hermetic |
| **AI2 L8** verify --quiet soft | AI2 | **Agree** | F17 unchanged |
| **AI2 L9** Incomplete fixture | AI2 | **Agree hard** | Phase 1 junk-table recipe |
| **AI2 L10** serde Incomplete | AI2 | **Agree** | F19 |
| **AI2 L11** Incomplete noise | AI2 | **Agree hard** | Phase 2 F27 |
| **AI2 O12** decision table docs | AI2 | **Agree** | Phase 3 CAPABILITIES |
| **AI2 O13** hard cross-model | AI2 | **Agree hard** | F25 upgrade |

### Pins locked by fold-in

1. **F5 (AI2 M2):** keep IN query for `tables_out`; gate `len() < 2`; fail `missing core tables`.  
2. **F6 (AI2 M1):** residual SOOT `not recoverable under current key`; migrate **8** honesty asserts (`backup_list_honesty.rs` ~119,177,190,191,227,257,280,353).  
3. **F7 (AI2 M3):** CLI `run_list` usable-first only; brain `list_backups` stays `Reverse(timestamp)`.  
4. **F8 (AI2 L4):** rename `empty_meta_token` → `backup_class_token`.  
5. **F13 (AI2 L9):** SQLCipher + `CREATE TABLE junk(x)` fixture.  
6. **F25 (AI2 O13):** hard cross-model on classify + doctor usable.  
7. **F27 (AI2 L11):** Incomplete noise = debug Default/Quiet, warn Verbose.

---

## Preflight (plan time — 2026-08-12)

| Check | Result |
|-------|--------|
| Live `backup list` | 21 files; residual summary **14** (undercount); SOOT `not fully readable` |
| Live `backup verify` | `0 OK, 21 FAIL` + T225 quiet + create nudge; exit 1 |
| Live `doctor` `backup_recent` | false-usable PreT109 (verify FAIL missing cores) |
| Live `backup create --dry-run` | ~78 MB path; would prune 11 under default keep |
| `backup_list_honesty` SOOT sites | **8** × `not fully readable` (AI2 M1) |
| Verify JSON `tables` | Filled from IN query (AI2 M2 — must preserve) |
| Brain list sort | `Reverse(timestamp)` — doctor depends (AI2 M3) |
| `--no-prune` | Confirmed clap flag |
| `rusqlite` | **0.39.0** — no bump |
| Root cause | Classify lacks core-table gate; doctor trusts PreT109 shells |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Backup fleet 0 usable / legacy plain | deferred.md | **DoD** honesty + green path |
| Doctor false-usable PreT109 without cores | T225 F9 gap + live | **DoD** F1–F4 |
| List residual undercount | Live residual_count | **DoD** F6 |
| List bury usable | Placeholder F2 | **DoD** F7 CLI sort |
| Verify either-table vs both | Code drift | **DoD** F5 `len() < 2` |
| Honesty SOOT migration | AI2 M1 | **DoD** F6 + AC16 |
| T225 F17 full structured error / quiet / JSON summary | T225 soft | **Soft F17** |
| Archive/quarantine legacy | Placeholder F4 | **Soft F18** |
| Auto-delete / nightly create / rusqlite 0.40 | series non-goals | **Not absorbed** |

---

## Phases

### Phase 0 — Plan freeze

- [x] Full spec + plan
- [x] Live dogfood + dep research (2026-08-12)
- [x] Roll T225 usable honesty gap + deferred fleet residual
- [x] AI fold-in AI1+AI2 → F5–F8, F11–F13, F22, F25, F27–F29, AC16–17, §12
- [x] User **go** before production code **or** live create

### Phase 1 — Red (TDD)

- [x] Unit: `is_usable_class` — Readable/PreT109 true; Incomplete/LegacyPlain/KeyMismatch/Corrupt false
- [x] Unit: `residual_for_summary` = `!is_usable_class`
- [x] Unit: `list_sort_key` — usable before residual; `Reverse(None)` last within band; path tiebreak
- [x] Unit: `classify_backup_read__openable_without_core_tables__incomplete` (F13 junk table)
- [x] Unit: `classify_backup_read__openable_with_core_tables_no_meta__pre_t109`
- [x] Unit: `list_sort_order__usable_ranked_before_residuals`
- [x] Migrate **8** `backup_list_honesty` asserts: `not fully readable` → **`not recoverable under current key`** (AI2 M1)
- [x] Hermetic residual-count includes Incomplete fixture
- [x] Hermetic doctor: all Incomplete → **no usable** + create (not stale-on-Incomplete)
- [x] Hermetic doctor: Readable in-age → ok (`max-age` = `18446744073709551615d`)
- [x] Hermetic doctor: stale usable + fresher Incomplete/plain → stale usable warn
- [x] Hermetic verify: zero-core + single-core shells → FAIL `missing core tables`; JSON `tables` still array
- [x] Keep T225 verify smokes green

### Phase 2 — Green (classify + verify SOOT)

- [x] Add `BackupReadClass::Incomplete` (`Default` remains Readable)
- [x] `classify_backup_read`: after `key_ok` → `if !has_core_tables(&conn) { Incomplete }` → else meta branch
- [x] `emit_list_noise` Incomplete: **debug** Default/Quiet, **warn** Verbose; message mentions missing core tables (F27)
- [x] Export `is_usable_class` / `residual_for_summary` from brain (or CLI pure shared with units)
- [x] `verify_single_backup`: keep IN query → `tables_out`; **gate `tables_out.len() < 2`** (F5 / AI2 M2)
- [x] Rename `empty_meta_token` → **`backup_class_token`**; Incomplete → `(no core tables)`
- [x] Exhaustive match compile fixes across workspace

### Phase 3 — Green (list + doctor + docs)

- [x] `run_list`: residual via `residual_for_summary`; F6 SOOT string pinned
- [x] `run_list`: sort with `list_sort_key` (**CLI only** — do not change brain `list_backups` sort)
- [x] Doctor `check_backup_recent`: usable via `is_usable_class`; keep create-only remediation
- [x] CAPABILITIES §11: Incomplete + usable SOOT + **§7 decision table** + residual wording + sort note
- [x] OPERATIONS Backup: create→verify→doctor green path; CHANGELOG T244

### Phase 4 — Hermetic green path

- [x] Temp vault create → verify ≥1 OK → doctor ok (u64::MAX max-age)
- [x] AC1–AC11, AC16–AC17 locked

### Phase 5 — Live dogfood (mutating — go only)

- [x] Prefer: `ai-brains backup create --no-prune` (target/debug binary 2026-08-12)
- [x] Expect: **22** files; verify **`1 OK, 21 FAIL`**; list usable-first; doctor `backup_recent` ok
- [x] Used `--no-prune` (no default keep-10 prune)
- [x] Record exact commands + outputs below

### Phase 6 — Gate + close

- [x] Full CI gate (fmt, clippy -D warnings, nextest workspace 2681, deny, audit) + CI Win/Linux/macOS PR #149
- [x] `ledgerful verify --scope fast` (pre-push) + local full gate commands
- [x] Primary review + **hard cross-model** on F1/F4 classify + doctor usable (F25) — Codex CX2 PASS WITH DEFERRED P3
- [x] Update `conductor.md` T244 → Completed; deferred.md close fleet row; series README (closeout PR)
- [x] `ledgerful ledger commit` (`526e64a0-39ee-474b-b373-820f8a846948`)
- [x] Pin: `DECISION: T244 Incomplete + core-table usable SOOT; F5 len<2; F7 CLI-only sort; residual not recoverable` (`3956b1ac-…`)

---

## Implementation notes

### Class priority for sort (F7 — CLI only)

| Priority | Classes |
|----------|---------|
| 0 (first) | Readable, PreT109 |
| 1 (after) | Incomplete, LegacyPlain, KeyMismatch, Corrupt |

Within priority: `Reverse(Option<NaiveDateTime>)` then `PathBuf`. **None timestamps last within band.**

### Verify F5 (do not break JSON)

```text
// KEEP
SELECT name FROM sqlite_master WHERE type='table' AND name IN ('events', 'memory_projection')
// COLLECT → tables_out (JSON display)

// GATE (was is_empty)
if tables_out.len() < 2 {
    return Err("backup is missing core tables".into());
}
```

### List residual SOOT (F6)

```text
{n} backup(s) not recoverable under current key (legacy plain / incomplete / key / corrupt): use --verbose or ai-brains backup verify
```

Stable assert substring: **`not recoverable under current key`**.

### Doctor message SOOT (preserve T225 family)

| Condition | Message class |
|-----------|---------------|
| empty list | no backups found … create |
| zero usable | **no usable encrypted backup under current key** … create only |
| usable unparseable ts | usable … unparseable … create |
| usable age ≤ max | ok: newest usable within … |
| usable age > max | warn: newest usable older than … create |

Do **not** emit “newest usable older than” when only Incomplete/plain are recent.

### Create flags (confirmed)

```powershell
ai-brains backup create --no-prune
```

Default keep-10 still valid; document prune count if used.

### Files expected

| Area | Paths |
|------|--------|
| Classify / class / usable helpers | `crates/ai-brains-brain/src/backup.rs` |
| List / verify | `crates/ai-brains-cli/src/commands/backup.rs` |
| Doctor | `crates/ai-brains-cli/src/commands/doctor.rs` |
| Tests | `backup_list_honesty.rs` (SOOT×8), `doctor_cli.rs`, smoke verify, brain units |
| Docs | `Docs/CAPABILITIES.md` §11, `Docs/OPERATIONS.md` Backup, `CHANGELOG.md` |

### Out of scope reminders

- No mass delete of live 21 without operator prune
- No rusqlite 0.40
- No restore / recovery export redesign
- No brain `list_backups` sort change
- No emptying JSON `tables` field

---

## Manual evidence (fill on go)

```text
Binary: target/debug/ai-brains.exe (T244 build) — 2026-08-12

PRE list residual: "21 backup(s) not recoverable under current key …"
PRE verify: Verified 21 backups: 0 OK, 21 FAIL. (exit 1) + create nudge
PRE doctor backup_recent: warn "no usable encrypted backup under current key" rem=create

CREATE: ai-brains backup create --no-prune
  → Backup created and verified: C:\dev\ai-brains\backups\vault-2026-08-12T15-50-06.db.bak
  exit 0

POST verify: Verified 22 backups: 1 OK, 21 FAIL. (exit 1 expected for residual FAIL)
POST list: vault-2026-08-12T15-50-06 first (Readable, 78192640 bytes); residuals after
POST doctor backup_recent: ok "newest usable backup within 7d (timestamp 2026-08-12T15:50:06)"
BACKUP_COUNT=22
```

---

## AI fold-in

**Complete** 2026-08-12 — see disposition table above and spec §12.
