# T225 — Backup verify quiet + encrypted backup nudge — Plan

**Status:** ✅ **Completed** 2026-08-11 — PR #128 squash `927b8db`; Codex R3 PASS WITH DEFERRED P3
**Category:** UX  
**Depends:** T131 / T138 / T187 / T198 / T209 / T192 doctor  
- [x] `ledgerful ledger start/commit T225-backup-verify-quiet-nudge` — TX `2b97a127-cdd6-4973-802a-b4218ac94479` **committed**

## Goal

Make `backup verify` operator-usable on large/legacy fleets: **quiet-by-default** counts + first **5** FAIL reasons, **`--verbose`** full stream only (no summary), demote progress INFO → debug, keep JSON full + exit 1 on any fail, **verify create-nudge when ok==0**, and **doctor class-aware usable** (zero usable or stale usable → `ai-brains backup create`).

## Absorbed deferred

| Item | Disposition |
|------|-------------|
| deferred.md “Backup verify noise + legacy fleet” | **DoD** |
| Series README T225 (quality 6) | **DoD** |
| T209 soft AC10 doctor honesty with plain residual | **Elevate** into F9/AC7 |
| Placeholder “quiet summary + create nudge” | **DoD** |

**Not absorbed as DoD:** auto-delete legacy; rusqlite 0.40; verify exit 0 on residual FAIL; JSON summary field; verify `--quiet`; structured VerifyError (O1); MSI; clap 5; T209 L3/L4 fixtures; nightly create (T229).

## Research pins (2026-08-11)

| Pin | Evidence |
|-----|----------|
| Live flood | Dogfood: 21 backups → ~43 INFO + 21 stdout FAIL; exit 1 |
| Fleet class | list quiet: only legacy / no metadata |
| Doctor gap | age-only `find_map(timestamp)`; no class filter |
| Doctor tests | **No** direct `backup_recent` assert (M3) |
| Smoke M1 | `backup_verify__valid_backup__reports_ok` `!contains("FAIL")` breaks on `0 FAIL` |
| Smoke M2 | mixed filter requires per-file OK under default — will break when OK lines omitted |
| Deps | rusqlite **0.39.0**; tracing **0.1.44**; subscriber **0.3.23** — no bump |

## AI fold-in pins (hard)

| ID | Pin |
|----|-----|
| **M1** | Migrate all-OK smoke: `!contains("FAIL —")` and/or summary counts — **not** bare `!contains("FAIL")` |
| **M2** | Mixed default: assert summary `1 OK, 1 FAIL` + one FAIL preview; **no** per-file OK; verbose twin both lines |
| **M3** | Net-new doctor hermetic matrix (no migrate); all-plain / Readable ok / stale / PreT109 usable |
| **M4** | Verify nudge = `ok==0 && total>=1` only; doctor owns stale usable |
| **M5** | Rollup ≤3 classes (plain / tables / other) or omit; no string-based key-vs-corrupt |
| **L1** | Verbose = full stream **only** (no summary/nudge) |
| **L3** | `--verbose --format json` → JSON; verbose ignored |
| **L4** | `const VERIFY_FAIL_PREVIEW_CAP: usize = 5` in `verify_report.rs` |
| **L5** | Doctor soft preserved (not new hard fail) |
| **L6** | Migrated verify smokes prefer `--no-project-context` |
| **O4** | Hermetic: `count("FAIL —") == min(fail, 5)` preferred |
| **O5** | CAPABILITIES: verify now quiet-by-default; list still cites verify |

**Rejected as DoD:** O1 structured class; O2 double-open; verify stale age.

See `spec.md` §10 full disposition.

## Frozen decision index

See `spec.md` §4 **F1–F25**. Hard summary:

1. Integrity + exit frozen (F1).  
2. Default human = counts + first 5 FAIL + trailer (F2/F3).  
3. Verbose = full stream only, no summary (F4/L1).  
4. JSON full; verbose ignored for JSON (F5/L3).  
5. Progress `debug!` (F6).  
6. Verify nudge only zero-usable (F8/M4).  
7. Doctor class-aware + stale (F9).  
8. Pure helpers unit-first (F12).  
9. Smoke M1/M2 + net-new doctor tests M3 (F13).  
10. Optional 3-class rollup only (F7/M5).

### Suggested SOOT (pin in units)

Default human:

```text
Verified {total} backup(s): {ok} OK, {fail} FAIL.
{up to 5 lines: name: FAIL — reason}
… and {more} more FAIL (use --verbose for full list).
No usable encrypted backup under current key. Run: ai-brains backup create
```

- Trailer only when `fail > 5`.  
- Nudge only when `ok == 0 && total >= 1` (verify).  
- All-OK still uses `0 FAIL` in counts (M1 smoke must not ban substring `FAIL`).  
- L2 optional: singular `backup` when `total == 1`.  
- Verbose: **no** summary / trailer / nudge — only `name: OK` / `name: FAIL — …` lines.

## Phased checklist

### Phase 0 — Preflight (on go)

- [x] `ai-brains preflight --summary`
- [x] `ledgerful doctor` / `ledgerful ledger status --compact`
- [x] `ledgerful scan --impact` (expect `backup.rs`, `doctor.rs`, `main.rs`, tests, CAPABILITIES)
- [x] `ledgerful ledger start/commit T225-backup-verify-quiet-nudge` — TX `2b97a127-cdd6-4973-802a-b4218ac94479` **committed**

### Phase 1 — Red: pure formatters (AC1, AC6, F3/F8)

- [x] Add `crates/ai-brains-cli/src/verify_report.rs` (or backup-adjacent):
  - `const VERIFY_FAIL_PREVIEW_CAP: usize = 5` (L4)
  - `format_verify_counts(total, ok, fail) -> String`
  - `format_fail_preview(fails, cap) -> (lines, trailer_opt)`
  - `should_emit_create_nudge(ok, total) -> bool` — **only** zero-usable (M4)
  - optional `rollup_fail_classes` — 3 buckets only (M5)
- [x] Units:
  - 0 fail → counts with `0 FAIL`; no trailer; nudge false
  - 3 fail → 3 detail lines; no trailer
  - 6 fail → 5 detail + trailer (`more`, `--verbose`)
  - ok=0,total>0 → nudge true; substring `ai-brains backup create`
  - cap == 5
  - optional: total==1 singular polish
- [ ] Red commit allowed

### Phase 2 — Green: wire `run_verify` (AC1–AC6, AC9)

- [x] Clap: `Verify { …, verbose: bool }`
- [x] Demote progress `info!` → `debug!` (F6)
- [x] Default human: summary + preview + optional nudge; **no** OK list
- [x] Verbose: **only** full per-file lines (L1)
- [x] JSON always full; ignore verbose for JSON (L3)
- [x] Keep `exit(1)` on any_failed

### Phase 3 — Doctor class-aware (AC7–AC8 / M3)

- [x] `check_backup_recent`: usable = `Readable | PreT109`
- [x] No usable → warn + create (even recent plain)
- [x] Age newest usable only (mixed fleet)
- [x] Soft check remains soft (L5)
- [x] **Net-new hermetic** (no migrate):
  - [x] all-LegacyPlain → warn + create
  - [x] Readable within age → ok
  - [x] Readable stale → warn + create
  - [x] PreT109 newest + older LegacyPlain → ok (implemented as Readable recent + older/fresher plain)

### Phase 4 — Tests (AC1–AC5, AC9, AC13 / M1–M2)

- [x] **M1:** migrate `backup_verify__valid_backup__reports_ok` → `!contains("FAIL —")` (+ optional `1 OK` summary)
- [x] **M2:** migrate mixed default → summary counts + one FAIL preview; **no** per-file OK assert
- [x] Verbose twin: both `vault-…: OK` and `vault-…: FAIL`
- [x] Hermetic multi-fail: `count("FAIL —") == min(fail, 5)` (O4); trailer when >5
- [x] Hermetic create nudge when all fail
- [x] AC2: env_remove RUST_LOG → no `Verifying ` INFO
- [x] Prefer `--no-project-context` on migrated smokes (L6)
- [x] Keep T138, JSON, empty_states green

### Phase 5 — Docs + registry (AC10 / O5)

- [x] CAPABILITIES §11: quiet default, verbose = full only, doctor usable, list→verify still valid + now quiet
- [x] CHANGELOG T225
- [x] Soft OPERATIONS / RECOVERY-DRILLS
- [x] conductor / deferred / series README on ship

### Phase 6 — Gate + manual (AC11–AC12)

- [x] Full CI gate + `ledgerful verify --scope full` (2524 pass; ledgerful full PASS)
- [x] Manual dogfood per spec §9; record below
- [x] `review.md` (Codex R3 PASS WITH DEFERRED P3); pin+ledger on ship

## Manual evidence (fill on go)

| Step | Command | Result |
|------|---------|--------|
| Default fleet | `target\debug\ai-brains.exe --no-project-context backup verify` | **21** backups → `Verified 21 backups: 0 OK, 21 FAIL.` + **5** `FAIL —` + trailer `… and 16 more` + create nudge; exit **1**; no `Verifying ` INFO |
| Verbose | `… backup verify --verbose` | Full per-file FAIL stream only (no summary/nudge); first lines are vault-*: FAIL — |
| JSON | `… backup verify --format json` | `results_count=21` `status=fail` |
| Doctor | `… doctor --json` → `backup_recent` | severity=warn; message ages usable (stale) + rem `ai-brains backup create` (fleet has PreT109/openable residual) |
| Fresh encrypted | tempdir zero-key+ALLOW_ZERO_KEY init+create+verify | `Verified 1 backup: 1 OK, 0 FAIL.` exit **0** |
| Empty | empty backups dir | `No backups to verify.` exit **0** |

## Stop-before

- Auto-delete of live `backups/*.db.bak`
- rusqlite major bump
- Exit 0 on residual FAIL
- Scope into restore / prune / MSI / structured VerifyError as DoD

## After ship

1. Strike deferred T225; series next → T226+.  
2. Soft residuals (F17/O1–O3/O6) → deferred.md.  
3. Pin: quiet verify + doctor usable nudge shipped.  
4. Operator still runs `ai-brains backup create` on live encrypted vaults.
