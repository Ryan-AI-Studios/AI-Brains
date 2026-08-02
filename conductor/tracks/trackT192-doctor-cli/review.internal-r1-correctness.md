# T192 Internal Correctness Review R1

**Reviewer role:** read-only second internal review (correctness + docs/claims honesty)  
**Date:** 2026-08-02  
**Scope:** `commands/doctor.rs`, contracts DTO, main early handler, F17b backup read path, claims script, Docs residual language, consumer impact of DTO break  
**Code modified by reviewer:** none (this file only)

## Verdict: FAIL

One **P1** docs/claims residual remains on an elevated surface (`Docs/INSTALL.md`: “Doctor still absent”). Core doctor check logic, exit mapping, open path, event_type SQL, F17b, and rule #54 removal look correct against live code and hermetic tests.

---

## Findings (P0–P3)

### P0 — none

No production logic defects found that would systematically mis-report hard checks, invent kit paths, migrate/create vaults, or steal T160 exit codes 3–7.

---

### P1 — Docs honesty residual: INSTALL still says “Doctor still absent”

| Field | Detail |
|-------|--------|
| **ID** | R1-C-P1-01 |
| **Severity** | P1 |
| **Area** | Docs / AC11 / F26 |
| **Evidence** | `Docs/INSTALL.md` §11 table row for `ai-brains recovery export` still ends with **“Doctor still absent.”** while the row above correctly marks `ai-brains doctor` as **Shipped (T192)**. |
| **Why it matters** | INSTALL is in the elevated release-claims scan set (`scripts/check-release-claims.ps1` `$elevated`). Contradictory “absent” language fails AC11 / F26 honesty flip and can mislead operators even after the R-DOC-CLI rewrite elsewhere. Rule #54 removal means this phrase is **not** auto-gated. |
| **Expected** | Strike “Doctor still absent.” (or rephrase residual solely as offline kit without `--kit-path`). |
| **Code impact** | None; docs-only. |

---

### P2 — Implementation-Plan drift banner still treats `doctor` as unshipped example

| Field | Detail |
|-------|--------|
| **ID** | R1-C-P2-01 |
| **Severity** | P2 |
| **Area** | Docs honesty |
| **Evidence** | `Docs/Implementation-Plan.md` §8 drift notice (≈L1268): lists `doctor` among commands that were “renamed, never built, or only exist as DTOs”. |
| **Why it matters** | Section has a drift banner pointing operators to live `--help`, so not elevated-claim P1; still actively false post-T192 and conflicts with CAPABILITIES / RELEASE-CLAIMS / SECURITY-LIMITS. |
| **Expected** | Drop `doctor` from the “never built / DTO-only” example list (or note “shipped T192”). |

---

### P2 — Kit parse-failure path may surface serde text with kit field content (F22 residual)

| Field | Detail |
|-------|--------|
| **ID** | R1-C-P2-02 |
| **Severity** | P2 |
| **Area** | Leakage / F22 |
| **Evidence** | `check_recovery_kit_file` → `RecoveryKit::from_json` → `DeserializationError(e.to_string())` → message `failed to parse RecoveryKit: {e}` (`doctor.rs` ≈L389–396; `recovery_kit.rs` `from_json`). Happy-path leakage test (`doctor__stdout__no_secrets`) covers successful unlock only — not malformed kit JSON. F22 forbids salt/nonce/wrap ciphertext in operator-facing doctor messages. |
| **Why it matters** | `serde_json` error strings sometimes include a token/snippet from input; a crafted/corrupt kit could place wrap material into the check `message` (stdout JSON/human). Not proven leaking on every parse fail, but unmitigated and untested. |
| **Expected** | Generic parse-fail message without raw `{e}`; or sanitize; hermetic test with distinctive ciphertext/salt bytes in invalid kit asserting absence on stdout/stderr. |

---

### P3 — `daemon_reachable` never emits probe-error `warn` (F6/F14 ideal)

| Field | Detail |
|-------|--------|
| **ID** | R1-C-P3-01 |
| **Severity** | P3 |
| **Area** | Check logic / F6 / F14 |
| **Evidence** | `probe_restore_daemon_busy` returns `bool` only (`backup.rs` ≈L351–365). Doctor maps `true`→`up` / `false`→`down` both as `CheckSeverity::Ok` (`doctor.rs` ≈L137–142). Spec F6/F14: probe **errors** → `warn`. |
| **Why it matters** | IPC failure and “daemon down” are collapsed; residual is partially covered by after_help (“Daemon probe = our IPC only”). Acceptable given shared probe API; not a fail roll-up bug. |

---

### P3 — `zero_key_escape` env detection broader than product allowlist

| Field | Detail |
|-------|--------|
| **ID** | R1-C-P3-02 |
| **Severity** | P3 |
| **Area** | zero_key honesty / F20 |
| **Evidence** | Doctor: `std::env::var_os(ALLOW_ZERO_KEY_ENV).is_some()` (`doctor.rs` ≈L481). Store: `zero_key_allowed()` only treats `1`/`true`/`yes` (`connection.rs` ≈L124–131). |
| **Why it matters** | `AI_BRAINS_ALLOW_ZERO_KEY=0` still **warns** doctor while open still **refuses** zero key. Soft honesty only; no false `ok` on hard vault open. |

---

### P3 — `vault_open` message when reparse/missing conflates causes

| Field | Detail |
|-------|--------|
| **ID** | R1-C-P3-03 |
| **Severity** | P3 |
| **Area** | Messaging |
| **Evidence** | If `vault_exists` not Ok, open path uses fail message “vault missing; cannot open” (`doctor.rs` ≈L80–84) even when `vault_exists` failed for **reparse** (≈L198–210). Overall status still `fail`. |

---

### P3 — Invalid `--format` silently treated as human

| Field | Detail |
|-------|--------|
| **ID** | R1-C-P3-04 |
| **Severity** | P3 |
| **Area** | CLI surface / F10 |
| **Evidence** | `emit_report`: JSON only if `force_json \|\| format.eq_ignore_ascii_case("json")`; else human (`doctor.rs` ≈L544–549). No clap `value_parser` for `human|json`. |
| **Why it matters** | Typos (`--format josn`) produce human without usage exit 2. Default human path is correct; not OutputFormat::parse Json default. |

---

### P3 — Pre-report fatal key resolve skips `DoctorReport` (F31 preference)

| Field | Detail |
|-------|--------|
| **ID** | R1-C-P3-05 |
| **Severity** | P3 |
| **Area** | F31 |
| **Evidence** | `resolve_sqlcipher_key` Err → `build_report` Err → `handle_cli_result` ApiResult exit 1, no report object (`doctor.rs` ≈L60, L517–526; `main.rs` handle_cli_result). |
| **Why it matters** | F31 prefers emitting a report when vault-path resolved; rare invalid-key-format path only. |

---

## Focus-area checklist (reviewer notes)

### 1. doctor.rs check logic

| Check | Assessment |
|-------|------------|
| **vault_exists** | Reparse → fail; missing → fail; not file → fail; else ok. Aligns F3/matrix (prefer fail on reparse). |
| **vault_open** | Only when exists ok; **`VaultConnection::open_read_intent` only** — no `AppContext::from_cli` / migrate. |
| **schema_readable** | `has_core_tables` (events + memory_projection); skip if open failed. |
| **cipher_page** | Non-empty `cipher_version`; fail empty/err; skip if open failed. |
| **backup_recent** | `parse_duration` Nd/Nh/Nw; `list_backups(true)`; newest via sorted `Option` timestamps + `and_utc()`; soft warn only. |
| **recovery_kit_event** | See §4 — **correct vs live storage**. |
| **recovery_kit_file** | Skip if no path; reparse refuse; unlock passphrase-file/TTY/DPAPI; DataKey compare via `data_key_from_sqlcipher` + `expose_secret` equality. No invented path. |
| **zero_key_escape** | Soft warn on zero key or env present. |
| **integrity** | Skip unless `--full`; hard fail on non-ok. |
| **daemon_reachable** | Info ok up/down; never fail alone. |
| **roll-up** | Contracts `DoctorReport::roll_up`: fail ≻ warn/degraded ≻ ok; skip ignored. Fixed check order matches §6. |

### 2. Exit code mapping + `--fail-on-degraded`

```text
ok → 0
degraded → 0 (default) | 1 if --fail-on-degraded
fail → 1
```

Implemented in `exit_code_for`; unit-tested; AC10 hermetic. Fail path uses `process::exit(code)` **after** emit (report still printed). Does not use exit 3–7.

### 3. Human vs JSON default (not OutputFormat::parse Json)

- Clap: `#[arg(long, default_value = "human")]` on Doctor.format (`main.rs`).
- Emit: `--json` forces JSON; else format `"json"`; else human.
- Doctor does **not** call `OutputFormat::parse` (governed missing→Json). AC9/AC human default tests present.

### 4. recovery_kit_event SQL vs event_store (trim_matches)

**Live storage** (`event_store.rs` ≈L121–124):

```rust
let event_type_str = serde_json::to_string(&envelope.event_type)?
    .trim_matches('"')
    .to_string();
```

Column value is `RecoveryKitCreated` **without** surrounding JSON quotes. Spec F16 draft (`'"RecoveryKitCreated"'`) is **wrong against live code**. Implementer correctly queries:

```sql
SELECT COUNT(*) FROM events WHERE event_type = ?1
-- bind "RecoveryKitCreated"
```

Documented in-module; proven by `doctor__recovery_kit_created_event__ok_not_false_warn` (AC16). **Correctness: PASS** (spec draft residual only).

### 5. Claims rule #54 + R-DOC-CLI rewrite

- `check-release-claims.ps1`: comment documents removal of invented-doctor rule #54; no affirmative doctor forbid remains.
- `Docs/RELEASE-CLAIMS.md`: R-DOC-CLI = doctor **shipped**; residual = offline kit without `--kit-path` — honest.
- Residual hole: INSTALL line (P1 above).

### 6. Docs still saying “doctor absent”

| Location | Hit |
|----------|-----|
| `Docs/INSTALL.md:175` | **“Doctor still absent.”** — P1 |
| `Docs/Implementation-Plan.md` §8 drift | `doctor` as never-built example — P2 |
| CAPABILITIES / OPERATIONS / SECURITY-LIMITS / RECOVERY-DRILLS / RELEASE-CLAIMS / Docs/README / CHANGELOG / PROTOCOL-COMPAT | Shipped language present; residual = kit path only |

Broader repo grep for “doctor absent / not shipped” (Docs + scripts): only INSTALL + historical plan wording of concern.

### 7. Leakage risks

- Happy-path hermetic asserts no DataKey bytes, passphrase, full kit JSON dump, `"ciphertext"` field dump.
- Human/JSON report fields do not print key/passphrase/kit JSON.
- Residual: malformed-kit parse errors (P2); open-error Display of StoreError does not echo key material by construction.

### 8. Capture independence

- Doctor runtime path: store open_read_intent, brain BackupService/list helpers, crypto RecoveryKit, daemon probe — **no** model load, Ollama, embeddings, or graph feature.
- CLI still depends on `ai-brains-models` / optional graph for other commands; doctor path does not invoke them. F24 intent satisfied for doctor execution.

### 9. Backup F17b completeness

| Item | Status |
|------|--------|
| `backup_dir_read_only()` | Returns `Ok(None)` when absent; no `create_dir_all` |
| `list_backups` | Uses read-only helper |
| `find_backup_files` | Uses read-only helper |
| `preview_backup_path` | Computes path without create |
| Write paths | Still use creating `backup_dir()` |
| Tests | `list_backups__missing_dir__no_create`, `preview_backup_path__missing_dir__no_create`, doctor AC15 |

### 10. main.rs early handler + process::exit vs clap exit 2

- Doctor handled **before** `AppContext::from_cli` (with Recovery / rotate-datakey).
- Not `is_vault_path_free` (requires vault path).
- Clap usage errors from `Cli::parse()` exit **2** before doctor runs — no collision with doctor `process::exit(1)`.
- Doctor overall fail/degraded+flag: `process::exit(1)` after report emit; success returns `Ok(())` → process exit 0 via normal main.
- Windows large stack worker for `Commands` size (Doctor landing) is defensive, not a correctness defect.

### 11. Breaking DTO: old `DoctorReport { status: String }`

| Consumer | Impact |
|----------|--------|
| `ai-brains-cli` doctor + `tests/doctor_cli.rs` | Updated to `DoctorStatus` enum |
| `ai-brains-contracts` unit tests | Updated |
| Other crates / TS / frontend | **No in-repo consumers** of old free-form `status: String` doctor DTO |
| Wire shape | `status` now enum snake_case (`ok`/`degraded`/`fail`); `schema_version: 1`; `severity` + `ok` flag; optional message/remediation skipped when None |

**Consumer impact:** workspace-internal only; no broken out-of-tree consumers found in repo. F11 “breaking-compatible within workspace” holds.

---

## Docs/claims residual grep hits

| Path | Note |
|------|------|
| `Docs/INSTALL.md:175` | **P1** “Doctor still absent.” |
| `Docs/Implementation-Plan.md` §8 | **P2** doctor listed as never-built/DTO example |
| `scripts/check-release-claims.ps1:54` | Comment only — rule #54 **removed** (expected) |
| Elevated shipped language | CAPABILITIES, OPERATIONS, SECURITY-LIMITS, RECOVERY-DRILLS, RELEASE-CLAIMS, Docs/README, CHANGELOG — doctor shipped + kit residual |

---

## Consumer impact of DTO change

- **In-repo:** only T192 doctor CLI + contracts tests.
- **Wire:** JSON automation must deserialize `status` as enum values `ok|degraded|fail` (not free-form strings) and accept `schema_version`, `severity`, `generated_at`, `vault_path`.
- **No** other workspace crate imported the old stub shape for runtime behavior.
- Safe to ship from monorepo consumer perspective after INSTALL residual fix.

---

## What looks solid (no finding)

- Read-only open path (F2/F3/F5): early handler, `open_read_intent` only.
- Exit policy F9 + `--fail-on-degraded` AC10.
- Human default F10 (not governed Json default).
- Event type query matches **trim_matches** storage (AC16); intentional divergence from incorrect F16 draft text.
- Kit reparse refuse F15b; no default kit path F15.
- F17b non-creating list path + AC15.
- Claims rule #54 removed; R-DOC-CLI residual correctly narrowed to offline kit without `--kit-path` in primary release docs.
- No `unwrap`/`expect` in production `doctor.rs`.
- Hermetic coverage maps well to AC1–AC16 subset.

---

## Closure criteria for this review

1. Fix **R1-C-P1-01** (INSTALL “Doctor still absent”) — required for PASS.  
2. Strongly preferred: **R1-C-P2-01** (Implementation-Plan drift example) + **R1-C-P2-02** (parse-fail leakage hygiene/test).  
3. P3 items deferrable with ISSUES tracking if desired.

**Re-review trigger:** docs P1 fix (and any P2s closed or explicitly deferred).
