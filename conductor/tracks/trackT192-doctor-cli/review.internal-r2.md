# T192 Internal Review R2 (post–R1 fixes)

**Track:** T192 Doctor CLI  
**Reviewer:** Grok Build (read-only re-review)  
**Date:** 2026-08-02  
**Repo:** `C:\dev\AI-Brains`  
**Scope:** Verify R1 P0–P2 dispositions with file evidence; fresh sweep for new P0–P2 regressions.  
**Code modified by reviewer:** none (this file only)

## Verdict: **PASS WITH DEFERRED P3**

All prior **P0–P2** findings from R1 / R1-correctness are **verified fixed**. No new **P0–P2** product defects found. Remaining items are **P3** polish / closeout residuals (may defer with ISSUES notes) plus expected track closeout (deferred #2 strike, full gate, ledger).

---

## Prior finding disposition matrix

| ID / source | Severity | Claim | Status | Evidence |
|-------------|----------|-------|--------|----------|
| **R1-C-P1-01** INSTALL “Doctor still absent” | P1 | FIXED | **Verified fixed** | `Docs/INSTALL.md:174–175` — doctor row **Shipped (T192)**; recovery-export row ends “Doctor is separate (`ai-brains doctor`, T192).” Grep: no `Doctor still absent` / `doctor not shipped` under `Docs/`. |
| **R1-C-P2-01** Implementation-Plan §8 drift | P2 | FIXED | **Verified fixed** | `Docs/Implementation-Plan.md:1268` — drift notice: “Some names were **renamed, never built, or only exist as DTOs** (for example `unlock`/`lock`, `install-hooks`…).” **Shipped since:** `recovery export` (T188), **`doctor` (T192 read-only health).** Doctor no longer in the never-built example list. |
| **R1-C-P2-02** kit parse echo of serde/kit fields | P2 | FIXED | **Verified fixed** | `commands/doctor.rs:396–405` — `Err(_)` → generic `"failed to parse RecoveryKit JSON (invalid schema or corrupted file)"` (no `{e}`). Unit `doctor__kit_parse_fail__message_has_no_secretish_payload` (`doctor.rs:717–739`) writes toxic `ciphertext`/`DEADBEEFSECRET` JSON and asserts absence. |
| **R1 P2** AC8 no-migrate test missing | P2 | FIXED | **Verified fixed** | Unit `doctor__no_migrate_while_daemon_up__build_report_ok` (`doctor.rs:663–715`): `build_report(..., daemon_up: true)`, status ok\|degraded, `daemon_reachable` message `"up"`, `vault_open` Ok, re-open via `open_read_intent`. Process `doctor__no_migrate_while_daemon_up__process_read_only` (`doctor_cli.rs:64–114`): exit 0, vault_open message contains `"read-only"`, no `backups/`, no `vault.db.bak`. Structural: early handler `main.rs:1942–1968` before `AppContext::from_cli`; match arm `unreachable!("doctor handled before AppContext")` (`main.rs:2022`); only open path `open_read_intent` (`doctor.rs:75`). |
| **R1 P3** `process::exit` in `run_with_daemon_state` | P3 | FIXED | **Verified fixed** | `run_with_daemon_state` returns `Result<i32, _>` (`doctor.rs:45–57`); `process::exit` only in production `run` when `code != 0` (`doctor.rs:35–42`). |
| **R1-C-P3-02** zero_key env mere presence | P3 | FIXED | **Verified fixed** | `check_zero_key_escape` (`doctor.rs:488–496`) matches store truthy `1`/`true`/`yes` (see `connection.rs:124–131`), not `var_os(...).is_some()`. |
| **R1-C-P3-03** vault_open “vault missing” on reparse | P3 | FIXED | **Verified fixed** | When `!vault_exists_ok`, message is generic: `"vault not openable (missing, not a regular file, or reparse refused)"` (`doctor.rs:84–90`). |

---

## Fresh sweep (new P0–P2?)

| Area | Result |
|------|--------|
| Hard-check logic (exists / open / schema / cipher / kit file) | **OK** — fail roll-up correct; open only `open_read_intent`; kit reparse refuse before read |
| Exit codes F9 + `--fail-on-degraded` | **OK** — `exit_code_for`; hermetic AC10; no exit 3–7 |
| Human default / not OutputFormat::parse Json | **OK** — clap `default_value = "human"`; emit path |
| F16 event_type vs live storage | **OK** — bare `RecoveryKitCreated`; AC16 hermetic; in-module note |
| F17b / AC15 | **OK** — `backup_dir_read_only`; process asserts no `backups/` |
| F22 leakage | **OK** — happy path + parse-fail unit; kit unlock errors use crypto Display (no ciphertext in AEAD fail path) |
| F5 / AppContext | **OK** — early async handler only |
| F23 humantime | **OK** — absent from workspace `Cargo.toml` |
| F24 capture independence | **OK** — doctor module: store / brain backup / crypto kit / daemon probe only |
| Production `unwrap`/`expect` in `doctor.rs` | **OK** — only under `#[cfg(test)]` |
| Docs residual “doctor absent” | **OK** — CAPABILITIES / RELEASE-CLAIMS / OPERATIONS / SECURITY-LIMITS / RECOVERY-DRILLS / INSTALL / README / CHANGELOG shipped language; residual = offline kit without `--kit-path` |
| Claims rule #54 | **OK** — removed; comment only (`scripts/check-release-claims.ps1:54`) |
| DTO consumers | **OK** — workspace CLI + contracts only |
| New regressions from R1 fixes | **None found** |

**No new P0, P1, or P2 findings.**

---

## Remaining P3 / process residuals (deferrable)

These are **not** blockers for this re-review verdict. Prefer ISSUES notes if not fixed before closeout.

### [P3] Invalid `--format` silently treated as human
**Location:** `doctor.rs:554–560`; clap `format: String` without `value_parser`.  
**Note:** Typos (`josn`) → human, not clap exit 2. Default human path remains correct.

### [P3] Spec F16 draft still claims JSON-quoted `event_type`
**Location:** `spec.md` F16 / live baseline vs implementer note in `doctor.rs:309–313` + `event_store` `trim_matches('"')`.  
**Note:** Code + AC16 correct; freeze text stale — amend on closeout errata to avoid future “fix” to wrong SQL.

### [P3] `daemon_reachable` never emits probe-error `warn` (F6/F14 ideal)
**Location:** `probe_restore_daemon_busy` → `bool` only; doctor maps true→`up` / false→`down` both Ok.  
**Note:** Documented residual (after_help / CAPABILITIES); shared T188 probe surface.

### [P3] Pre-report fatal key resolve skips `DoctorReport` (F31 preference)
**Location:** invalid key format → `build_report` Err → no report object. Rare path.

### [P3] INSTALL §11 section title vs shipped rows
**Location:** `Docs/INSTALL.md` heading “What is **not** shipped as CLI” still hosts **Shipped** doctor/export rows.  
**Note:** Content honest; heading slightly misleading — cosmetic.

### [P3 / process] AC13 / F27 deferred #2 not struck
**Location:** `conductor/deferred.md:16` still open T192; plan/status still mid-track language.  
**Note:** Expected until E5 closeout (full gate + ledger + strike). Not a product defect.

### Residual honesty (documented, not findings)
- Offline kit without `--kit-path` (R-DOC-CLI residual half).  
- Daemon probe = our IPC only.  
- Windows reparse kit test soft-skips without `mklink`.  
- `list_backups` metadata may open RW when backups exist (F17b scoped to no dir create).

---

## AC / freeze spot-check (post-fix)

| AC | Status |
|----|--------|
| AC1–AC4, AC6–AC7, AC9–AC12, AC14–AC16 | Met (prior R1 + unchanged hermetics) |
| AC5 | **Met** (was Partial) — unit injects `daemon_up=true` → message `"up"`, non-fail; process path covers ambient down |
| AC8 | **Met** (was Partial) — unit + process no-migrate / read-only guards |
| AC11 | **Met** — INSTALL P1 residual gone; elevated shipped language consistent |
| AC13 | **Pending closeout** only (gate / deferred strike / ledger) — process |

F highlights F1–F34: production freezes satisfied; F27 closeout strike and F16 *spec text* errata remain process/doc residuals.

---

## Test inventory (doctor-named)

| Suite | Count / names |
|-------|----------------|
| Integration `doctor_cli.rs` | **16** tests: no_migrate process, happy, wrong_key, missing_vault, daemon, kit good/bad/reparse/skip, json schema, human default, fail_on_degraded, no_secrets, no_backups_dir, recovery_kit_event, help |
| Unit `commands/doctor.rs` | **5** tests: roll_up, exit_code_for, health_check_order, **no_migrate build_report**, **kit_parse_fail** |
| Contracts `doctor.rs` | serde roundtrip, roll_up variants, ok_flag, omit optionals |
| Brain F17b | `list_backups__missing_dir__no_create`, `preview_backup_path__missing_dir__no_create` (etc.) |

R1 claimed “16 integration + unit (18 doctor-named)” — count is consistent with 16 process + multi unit (doctor-named unit subset ≥2 new; total doctor-named across crates ≥18 including contracts helpers).

---

## Summary judgment

R1 correctness **FAIL** (INSTALL P1) and R1 general **P2** (AC8 test, kit parse, Implementation-Plan drift) are **closed with code/docs evidence**. Claimed P3 polish on exit API, zero-key env, and vault_open messaging is also landed. Fresh sweep found **no new P0–P2**. Ship-quality operator surface holds for freezes; remaining work is closeout process + optional P3 deferrals.

**Verdict: PASS WITH DEFERRED P3.**
)
