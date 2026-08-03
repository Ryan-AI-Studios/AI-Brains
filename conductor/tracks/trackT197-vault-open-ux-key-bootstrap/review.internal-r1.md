# Track Completion Audit — T197

## Verdict: PASS WITH DEFERRED P3

Read-only audit of implementation against `spec.md` AC1–AC13, freezes F1–F32, and the audit checklist. No code or git state was modified. Evidence is from source inspection and the track plan’s reported gate results (not re-executed in this review session).

## Requirement and DoD Matrix

| AC / DoD | Status | Evidence |
|----------|--------|----------|
| **AC1** Wrong/missing key → &lt;20 stderr lines; zero `hmac check failed` | **Met** | Process tests `doctor__wrong_key__no_hmac_spam_stderr_bounded`, `doctor__missing_key__vault_open_skipped`, `recall__wrong_key__vault_locked_no_hmac_spam` in `crates/ai-brains-cli/tests/vault_key_bootstrap.rs` (asserts no hmac substring + `stderr_lines < 20`). Filter: `sqlcipher_log_policy::is_known_noise` + `install()`; CLI/daemon entry install before vault open. |
| **AC2** AppContext paths share F8 prefixes | **Met** | `AppContext::from_cli` → `resolve_operator_sqlcipher_key` (`context.rs:45`). F8 prefixes pinned in `KeyResolveError::Display` (`key_resolve.rs:20–36`). Process: `recall__missing_key__vault_key_missing_code`. preflight/project list share SOOT (no separate process tests — P3 residual). |
| **AC3** Invalid format fails at resolve before open | **Met** | `try_from_raw` in `resolve_operator_sqlcipher_key` (`key_resolve.rs:58–59`). Unit: `resolve_operator_sqlcipher_key__format__bare_hex`. Process: `recall__invalid_format__vault_key_format_code`. Doctor early: `doctor__invalid_format__early_error`. |
| **AC4** Missing → Missing / `VAULT_KEY_MISSING`, not silent zero | **Met** | `KeyResolveError::Missing` when both CLI and env empty after trim (`key_resolve.rs:53–56`). Units + process recall missing. Hermetic tests set explicit zero+ALLOW (`tests/common/mod.rs:69–77`). |
| **AC5** INSTALL + OPERATIONS bootstrap + doctor help | **Met** | `Docs/INSTALL.md` § Key bootstrap (T197); `Docs/OPERATIONS.md` key troubleshooting table + env expand (`AI_BRAINS_KEY`, `AI_BRAINS_ALLOW_ZERO_KEY`, `AI_BRAINS_VAULT_KEY` honesty). Doctor `after_help` links INSTALL + missing/wrong semantics (`main.rs` Doctor command ~203). |
| **AC6** Hermetic missing/format/wrong/zero/ALLOW | **Met** | `vault_key_bootstrap.rs` process suite + `key_resolve` unit matrix (incl. ALLOW zero). |
| **AC7** No secrets logged; no crypto redesign; gate | **Met (code)** | Format errors from `CryptoError::InvalidKeyFormat` never embed material; `vault_locked_message` does not include key; filter does not log secrets; HMAC not disabled. Gate: implementer/plan claim targeted store+cli+daemon 676 + clippy `-D warnings` (full workspace gate not re-run here — P3 process). |
| **AC8** Explicit zero refused; wrong key fail-closed | **Met** | Zero unit + process `recall__zero_key_without_allow__vault_key_zero`. Wrong key store test `install__wrong_key_open__fails_closed_vault_locked`; doctor/recall process paths. |
| **AC9** Full gate if code changed | **Partial process evidence** | Plan E1: targeted nextest (store+cli+daemon) + clippy. Not full Agents.md gate (`nextest --workspace`, `deny`, `audit`) evidenced in track artifacts. See P3. |
| **AC10** All 7 resolve sites use SOOT | **Met** | (1) `context.rs` AppContext; (2) `doctor.rs`; (3) `recovery.rs` `resolve_sqlcipher_key`; (4) `vault.rs` rotate `resolve_sqlcipher_key`; (5) `vault.rs` encrypt; (6) `migrate.rs` `resolve_sql_key` → SOOT; (7) `shadow.rs` `default_sql_key` → SOOT. Grep: no production silent-zero `unwrap_or` of all-zero key in CLI `src/`. |
| **AC11** Doctor missing → skipped; wrong → fail | **Met** | `build_report` F9 (`doctor.rs:72–127`, force Fail status `237–240`). Unit: `doctor__missing_key__vault_open_skipped_status_fail`, `doctor__wrong_key__vault_open_fail`. Process: vault_key_bootstrap doctor tests. Missing: `key_opt=None` → **never** calls `open_read_intent`. |
| **AC12** JSON codes mapped | **Met** | `key_resolve_json_code` + `VAULT_LOCKED_JSON_CODE` (`key_resolve.rs:68–78`). Wired in `handle_cli_result` (`main.rs:1757–1786`) with typed downcast + string-family fallback. Unit matrix `key_resolve_json_code__matrix`. |
| **AC13** install sites CLI + both daemon entries + tests | **Met** | CLI `main_inner` (`main.rs:1580`) before tokio; daemon `main` (`ai-brainsd/src/main.rs:29`) before runtime; `windows_service::run_daemon_startup` (`windows_service.rs:218`); doctor wrong-key unit + store module tests call `install()`. |
| **F2 no silent zero (CLI)** | **Met** | Shared resolver; hermetic fixtures explicit ALLOW. |
| **F8 message prefixes** | **Met** | Exact starts: `Vault key missing:`, `Vault key invalid format:`, `Vault key refused:`, `Vault locked:` (Display + constants). |
| **F19 init generate+print once** | **Met** | `run_init` Missing → generate non-zero (`main.rs:1958–1970`); print only when `generated_key` Some (`init.rs:43–49`). Tests: `init__missing_key__generates_and_prints_bootstrap`, `init__provided_key__no_generate_banner`, `init__generated_key__opens_doctor`. |
| **F32 high findings (CLI scope)** | **Cleared** | Silent zero removed on 7 sites; hmac spam filtered; secrets not logged; HMAC not disabled; migrate/shadow on SOOT; doctor missing ≠ wrong. |
| **F11 daemon log only** | **Met** | install() only; daemon key policy intentionally out of scope (still silent-zero `AI_BRAINS_VAULT_KEY` — residual, not T197 AC). |
| **No unwrap/expect in new prod** | **Met** | `key_resolve.rs` / `sqlcipher_log_policy.rs` production paths use Result/`if let Err`; expect only in `#[cfg(test)]`. |
| **rusqlite `trace`** | **Met** | Workspace `Cargo.toml` rusqlite features include `"trace"`. |

## Findings

### [P3] AC2 process coverage is recall-only
Confidence: Medium  
Requirement: AC2 — AppContext paths (recall / preflight / project list) share F8 prefixes  
Location: `crates/ai-brains-cli/tests/vault_key_bootstrap.rs` (recall only); `context.rs:37–54`  
Problem: Spec lists three example AppContext commands; only `recall` has a process missing-key assertion.  
Evidence: Grep of tests shows no preflight/project-list missing-key F8 process tests; all three go through `AppContext::from_cli` → same resolver.  
Correction: Optional process tests for preflight and `project list` with env_remove(`AI_BRAINS_KEY`) asserting F8/`VAULT_KEY_MISSING` (nice-to-have).  
Verification: nextest on new cases.  
Deferrable: Yes  

### [P3] Full workspace CI gate not evidenced for this track
Confidence: Medium  
Requirement: AC9; Agents.md pre-commit gate  
Location: plan.md E1; track artifacts  
Problem: Plan records targeted `store+cli+daemon` nextest (676) + clippy, not the full workspace gate (`fmt`, `clippy --workspace`, `nextest --workspace`, `deny`, `audit`).  
Evidence: plan.md lines 80–81; this review did not re-run gates.  
Correction: Orchestrator closeout should run full gate before ledger commit / push, or document intentional scope if only those crates changed and workspace already green.  
Verification: Agents.md CI command block.  
Deferrable: Yes (process residual; not a code AC gap if gate is run before publish)  

### Residual (not a T197 finding): daemon silent zero on `AI_BRAINS_VAULT_KEY`
Confidence: High  
Spec places daemon key UX under F11 (log silence only) and T199 for status-without-vault. Daemon still defaults missing `AI_BRAINS_VAULT_KEY` to all-zero (`ai-brainsd/src/main.rs:78–80`, `windows_service.rs:254–256`) and uses `from_raw` without CLI SOOT. OPERATIONS documents the CLI vs daemon env name split. Out of T197 DoD; do not block T197.

## Completeness Sweep

| Check | Result |
|-------|--------|
| TODO/FIXME/stub in `key_resolve.rs` / `sqlcipher_log_policy.rs` | None |
| Silent zero default in CLI production resolve paths | None found (7 sites SOOT) |
| F8 prefixes exact enough for tests | Yes (`starts_with` in units; process OR human/JSON) |
| Doctor missing never opens vault | Yes — `key_missing` → no `open_read_intent` |
| Doctor wrong key no spam | Process + install + filter |
| init F19 generate only when missing; stdout once; not silent zero | Yes |
| install() CLI main, daemon main, windows_service | Yes |
| unwrap/expect in new production | No |
| Secrets never logged | Yes (format msgs generic; vault_locked sanitized) |
| Tests AC1/AC4/AC11/AC12 | Yes (process + unit) |
| handle_cli_result JSON wiring | Yes |
| F32 high (CLI) still present | No |

## Wiring

1. **Resolver SOOT:** `ai_brains_cli::key_resolve::resolve_operator_sqlcipher_key` — CLI → env → Missing; `try_from_raw`; zero refuse unless ALLOW.  
2. **Sites:** context, doctor, recovery, vault encrypt, vault rotate, migrate (`resolve_sql_key` multi-arg pick then SOOT), shadow (`default_sql_key`).  
3. **Error edge:** `handle_cli_result` maps `KeyResolveError` → `VAULT_KEY_*`, `StoreError::VaultLocked` → `VAULT_LOCKED`, plus string prefix fallback for boxed mid-path errors. Exit 1.  
4. **Doctor path:** intentional special-case — Missing continues report; Format/Zero return Err to `handle_cli_result`; success path uses `process::exit(code)` for report status (not ApiError envelope).  
5. **Log policy:** `ai_brains_store::sqlcipher_log_policy::install` OnceLock + `config_log` filter + fire-and-forget `PRAGMA cipher_log_level=NONE` (secondary; community-supported per implementer comments). Idempotent double-install safe.  
6. **init:** Missing → generate → `AppContext::from_resolved_key` → `init::run(..., Some(material))` stdout banner.  

## Verification Evidence

| Source | Claim |
|--------|-------|
| plan.md E1 | store+cli+daemon nextest 676 pass; clippy `-D warnings` |
| Orchestrator summary | Targeted 676 tests pass store/cli/daemon |
| This review | Static AC matrix vs code; no live nextest re-run |
| CHANGELOG Unreleased | T197 entry present |

## Deferred Candidates

1. **[P3] AC2 process coverage** for preflight / project list — optional.  
2. **[P3] Full workspace gate** confirmation at closeout.  
3. **Out-of-track residual:** daemon `AI_BRAINS_VAULT_KEY` silent zero + `AI_BRAINS_KEY` spawn vs `AI_BRAINS_VAULT_KEY` read naming (pre-existing; T199/ops honesty).  

## Completion Decision

**PASS WITH DEFERRED P3** — All AC1–AC13 are met in code with hermetic proof for the security/UX core (no silent zero on 7 CLI sites, F8 family + JSON codes, doctor missing vs wrong, spam control install sites, init generate+print, docs). No P0–P2 gaps versus T197 spec freezes. Two deferrable P3 process/coverage items; F32 CLI high findings cleared. Ready for orchestrator closeout (full gate + ledger/pin per plan E4–E5).

### Audit checklist cross-check

1. AC1–AC13 vs code — completed in matrix.  
2. Completeness — no TODO/stub/silent zero in production resolve paths.  
3. Grep silent zero CLI prod — clean.  
4. F8 prefixes — exact starts match tests.  
5. Doctor missing never opens; wrong no spam — yes.  
6. init F19 — yes.  
7. install sites — yes.  
8. No unwrap/expect in new prod — yes.  
9. Secrets never logged — yes.  
10. Tests AC1/AC4/AC11/AC12 — yes.  
11. handle_cli_result wiring — yes.  
12. F32 high still present (CLI) — no.  
