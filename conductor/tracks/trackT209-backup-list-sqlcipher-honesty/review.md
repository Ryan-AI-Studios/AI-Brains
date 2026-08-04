# T209 Internal Review R1

**Reviewer:** internal (read-mostly)  
**Date:** 2026-08-04  
**Branch:** `track/T209-backup-list-sqlcipher-honesty`  
**Scope:** Spec F1–F35 + AC1–AC10 vs `backup.rs` / CLI list / doctor / hermetic + smoke / docs

## Verdict: CLEAN (internal R2)

Product classify + noise + table tokens + flags + docs match the frozen decisions. R1 medium **T209-R1-M1** fixed; L1 collapsed. Remaining L2–L4 deferred as low/low-info.

## Findings

### [T209-R1-M1] medium — AC4 verbose per-file detail assert is under-constrained

`backup_list_honesty__verbose_plain__per_file_detail` previously allowed the per-file detail arm to pass solely because the table always prints the filename on stdout.

**Files:** `crates/ai-brains-cli/tests/backup_list_honesty.rs`  
**Required fix:** Require `legacy plaintext` without filename-only fallback.  
**status:** verified_fixed (R2) — assert now requires `legacy plaintext` only; nextest green.

### [T209-R1-L1] low — classify PreT109 branches ignore `has_core_tables`

Identical PreT109 returns after key-ok meta failure.

**Files:** `crates/ai-brains-brain/src/backup.rs`  
**status:** verified_fixed (R2) — collapsed identical arms.

### [T209-R1-L2] low — pre-T109 smoke WARN assertion is OR-weak

`backup_list__pre_t109_backup__no_warn_on_stderr` uses:

`!stderr.contains("WARN") || !stderr.contains("backup metadata")`

This only fails when **both** substrings appear. A regression that reintroduces a different WARN wording (without “backup metadata”) would still pass. Hermetic suite does not cover PreT109; this smoke remains the AC6 anchor.

**Files:** `crates/ai-brains-cli/tests/smoke.rs`  
**Required fix (optional):** Assert no `WARN` at all under `RUST_LOG=warn` for a single pre-T109 residual, or pin the debug-only class via a unit test on `BackupReadClass::PreT109`.  
**status:** open

### [T209-R1-L3] low — AC9 fixture is large non-plain garbage, not a real wrong-key encrypted bak

Hermetic AC9 and unit B3 use `vec![0xAB; 600]`. F31 intentionally uses size after key-fail (error strings cannot split garbage vs wrong-key). Behavior matches F31/F34; residual risk is only documentation/operator expectation that AC9 proves a real rotate-wrong-key path end-to-end.

**Files:** `crates/ai-brains-cli/tests/backup_list_honesty.rs`, unit in `backup.rs`  
**Required fix (optional):** Soft: build a real SQLCipher bak then list with a different key. Not required by F31 discriminator.  
**status:** open

### [T209-R1-L4] low-info — no dedicated unit for PreT109 class

F22 units cover plain / &lt;512 / ≥512 key-fail / readable + `ListMode::from_flags`. PreT109 is smoke-only (AC6). Acceptable per F22 wording.

**status:** open

## Completeness matrix (F / AC)

| Item | Status |
|------|--------|
| F1 scope (list only) | OK — verify/restore untouched (plain refuse still in `verify_single_backup`) |
| F2–F3 header-first SOOT | OK — `is_plain_sqlite_header` before key probe; no duplicate magic |
| F4 PreT109 debug | OK — `emit_list_noise` PreT109 always debug; smoke AC6 still present |
| F5 default noise | OK — Corrupt `warn!`; LegacyPlain/KeyMismatch `debug!`; residual summary counts only plain+key |
| F6 eprintln summary | OK — Default only; template has `not fully readable`, `--verbose`, `verify` |
| F7 verbose | OK product — per-file `warn!` for LegacyPlain/KeyMismatch/Corrupt; summary omitted; **test gap M1** |
| F8 quiet wins, no conflicts_with | OK — `ListMode::from_flags`; clap List has no `conflicts_with`; dual hermetic test |
| F9 table tokens | OK — `(legacy plain)` / `(unreadable key)` / `(corrupt)` / `(no metadata)` |
| F10 exit 0 on class mix | OK — hermetic asserts exit 0 |
| F11 capture independence | OK — brain list path only |
| F12 doctor Quiet | OK — `list_backups(ListMode::Quiet)` |
| F13 verify plain refuse | OK — unchanged refuse path |
| F14 ListMode + BackupInfo.class | OK — exported; all call sites updated (no `list_backups(bool)`) |
| F15 zero new crates | OK |
| F16 hermetic RUST_LOG | OK — AC1 relies on hermetic denylist strip; AC2 re-sets `warn` |
| F17 high anti-patterns | OK — no auto-delete, no conflicts_with, no default plain WARN |
| F18 no key logging | OK — path only in tracing fields |
| F22/F31/F33/F34 tests | OK product + units; AC4 assert soft (M1) |
| F23/AC8 docs | OK — CAPABILITIES §11, CHANGELOG Unreleased, OPERATIONS one-liner |
| F24 Serialize soft | OK — `BackupReadClass: Serialize` |
| AC1–AC3, AC5, AC7, AC9 | OK hermetic locks |
| AC6 | OK smoke present (L2 assert quality) |
| AC10 soft doctor | Soft — code uses Quiet; no hermetic AC10 (allowed soft) |

## Correctness notes (classify)

1. **Header-first:** plain magic → `LegacyPlain` immediately; no key probe.  
2. **F31:** metadata fail → Corrupt; open fail → Corrupt; key/schema fail + `len < 512` → Corrupt; key/schema fail + `len ≥ 512` → KeyMismatch. Constant `MIN_PLAUSIBLE_BACKUP_BYTES = 512`.  
3. **Readable / PreT109:** meta SELECT success → Readable with map; missing meta table / core path → PreT109 (L1 nuance).  
4. **Noise:** Quiet demotes Corrupt to debug; Verbose promotes expected residuals to warn; summary only `ListMode::Default` + residual_count ≥ 1.  
5. **Production `unwrap`/`expect`:** none on list/classify path (test module only). `has_core_tables` uses `unwrap_or(false)` (allowed).

## Smoke meaningfulness

- `backup_list__pre_t109_backup__no_warn_on_stderr` still creates real keyed bak, strips meta, lists under `RUST_LOG=warn` — still the AC6 regression anchor (assert quality L2).  
- `backup_list__corrupted_new_backup__stays_warn` still writes short garbage and expects Corrupt WARN text; aligns with F5/AC2 (also hermetic AC2 with `(corrupt)` token).

## Placeholders / incomplete wiring

None found. Old `list_backups(bool)` call sites gone. Doctor, CLI dispatch, lib re-exports, docs present.

## Test results (R2 — orchestrator)

```powershell
cargo nextest run -p ai-brains-brain --lib
# → 31 passed
cargo nextest run -p ai-brains-cli -E 'test(backup_list)'
# → 14 passed (8 honesty + 6 smoke)
cargo clippy -p ai-brains-brain -p ai-brains-cli --all-targets -- -D warnings
# → clean
```

## R2 disposition

| ID | Disposition |
|----|-------------|
| T209-R1-M1 | verified_fixed |
| T209-R1-L1 | verified_fixed |
| T209-R1-L2 | deferred low (smoke OR-weak; AC6 still anchored) |
| T209-R1-L3 | deferred low (F31 garbage fixture intentional) |
| T209-R1-L4 | deferred low-info |

**Internal CLEAN** — ready for cross-model review.

---

# Cross-model review (Claude R1 — Codex rate-limited)

**Source:** `review.claude.md`  
**Verdict: PASS WITH DEFERRED P3**

No P0–P2. Deferred candidates (orchestrator):

| ID | Disposition |
|----|-------------|
| T209-R1-L2 | **fixed** in R3 — smoke AC6 now asserts no WARN at all under RUST_LOG=warn |
| T209-R1-L3 | deferred low — F31 garbage fixture intentional |
| T209-R1-L4 | deferred low-info — PreT109 unit soft |

Fresh clean cross-model gate satisfied for >low findings.
