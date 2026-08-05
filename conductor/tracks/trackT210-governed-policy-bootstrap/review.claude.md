I now have all the information needed for a complete review. Here is the full independent completion audit.

---

# T210 Independent Completion Review

**Reviewer:** Claude Sonnet 4.6 (cross-model, read-only)  
**Date:** 2026-08-04  
**Branch:** `track/T210-governed-policy-bootstrap`  
**Scope:** Working tree vs `main`, uncommitted + untracked files

---

## Verdict: PASS

All F-decisions and AC1ΓÇôAC11 acceptance criteria are implemented, tested, and wired into production paths. No critical, high, or medium findings remain. One non-blocking P3 observation is noted below.

---

## Requirement Matrix

| ID | Requirement | Status | Evidence |
|----|-------------|--------|----------|
| F1 | `policy bootstrap` subcommand | Γ£à | `main.rs:1121ΓÇô1134`, `PolicyCommands::Bootstrap`, wired at `main.rs:2681` |
| F2 | Exactly ReadEvidence + ReadConclusions + ReadDecisions | Γ£à | `DISCOVERY_CAPS` const, `policy_cmd.rs:43ΓÇô47`; T3 dangerous-caps hermetic |
| F3 | No dangerous caps (Erase, Approve*, Export) | Γ£à | Loop iterates only `DISCOVERY_CAPS`; `policy_bootstrap__after__dangerous_caps_still_denied` test |
| F4 | `resolve_principal(options.principal_id.as_deref())` | Γ£à | `policy_cmd.rs:207` |
| F5/F39 | Soft-resolve scope via `resolve_scope_key_for_cli`; `fail_usage` exit 2 | Γ£à | `policy_cmd.rs:214ΓÇô225`; AC8 hermetic |
| F6 | `Privacy::LocalOnly` always | Γ£à | `policy_cmd.rs:285`; T3 hermetic checks privacy on show |
| F7/M1 | `active_grants` (typed), not `list_applied_grants` | Γ£à | `policy_cmd.rs:255`; trait import `GrantPrincipalStore` |
| F8 | Local CP only (`StorePorts::from_store`) | Γ£à | `policy_cmd.rs:209ΓÇô211` |
| F9 | `--dry-run` zero event appends | Γ£à | `policy_cmd.rs:239ΓÇô251`, `271ΓÇô278`; AC6 locks second dry-run still `would_register` and first real bootstrap after still `registered` |
| F10 | Output shape: `api_version:"1"`, `principal_id`, `scope`, `registered`, `grants[]`, `dry_run`; human next-cmd hint | Γ£à | `PolicyBootstrapResponse` `policy_cmd.rs:51ΓÇô68`; human output `policy_cmd.rs:311ΓÇô331` |
| F11 | Exit codes: 0=success, 2=soft-resolve, 1=internal, 6=scope parse | Γ£à | `fail_usage` ΓåÆ 2; `fail_cp` ΓåÆ ApiError mapping; AC8 hermetic |
| F12 | Dual-site `POLICY_DENIED_HINT` mentions bootstrap first | Γ£à | CLI `governed_common.rs:46`; daemon `services.rs:989`; identical wording; sync comment |
| F13 | No auto-init grant | Γ£à | `run_init` calls no grant CP functions; INSTALL docs explicit |
| F14 | No DefaultPolicyEvaluator change | Γ£à | No evaluator file in diff |
| F15 | Domain logic forbidden in CLI | Γ£à | Only CP calls: `register_principal`, `issue_grant`, `active_grants`, `get_principal` |
| F16 | No new crates, no clap pin bump | Γ£à | No Cargo.toml changes |
| F17 | Capture independence (no models/graph) | Γ£à | Grant-only event writes |
| F18 | Event sourcing (append only) | Γ£à | No raw projection SQL; no revoke in path |
| F19 | CLI-local DTO; no contracts lift | Γ£à | `PolicyBootstrapResponse` defined locally in `policy_cmd.rs`; zero new imports from `ai-brains-contracts` |
| F21 | ΓëÑ7 hermetic tests | Γ£à | 9 tests covering AC1ΓÇôAC8 + F2/F3 |
| F23/F28 | CAPABILITIES, OPERATIONS, INSTALL, CLI-EXIT-CODES, CHANGELOG; empty vs deny honesty | Γ£à | All five docs updated; OPERATIONS "Empty vs deny" paragraph |
| F29 | `after_help` examples on bootstrap | Γ£à | `main.rs:1119ΓÇô1120` |
| F30 | Deterministic sort by capability name | Γ£à | `policy_cmd.rs:298`; AC2 asserts `[ReadConclusions, ReadDecisions, ReadEvidence]` |
| F33/M2 | `get_principal` probe before `register_principal` (DoD) | Γ£à | `policy_cmd.rs:235ΓÇô252` |
| F34 | Non-interactive | Γ£à | No prompts; flags/env only |
| F37/M5 | `#[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]` | Γ£à | `main.rs:1129` |
| F38/L2 | `format` default `"json"` | Γ£à | `main.rs:1131`, `#[arg(long, default_value = "json")]` |
| AC1 | Before bootstrap ΓåÆ exit 3 POLICY_DENIED | Γ£à | `policy_bootstrap__before__policy_check_read_evidence_exit_3` |
| AC2 | First bootstrap ΓåÆ exit 0; `registered:"registered"`; 3 caps issued; sorted; `grant_id` present | Γ£à | `policy_bootstrap__first_run__registers_and_issues_three` |
| AC3 | After bootstrap, 3 checks exit 0 allowed | Γ£à | `policy_bootstrap__after__three_checks_allowed` |
| AC4 | After bootstrap, `source list` + `review list` exit 0 | Γ£à | `policy_bootstrap__after__source_and_review_list_exit_0` |
| AC5 | Second bootstrap ΓåÆ exit 0, all `already_present`, `registered:"already"`, no crash | Γ£à | `policy_bootstrap__second_run__already_present` |
| AC6 | `--dry-run` ΓåÆ exit 0, zero appends; second dry-run still `would_register`; real bootstrap after still `registered` | Γ£à | `policy_bootstrap__dry_run__no_grants` (R1 medium fixed) |
| AC7 | CLI `POLICY_DENIED_HINT` and deny JSON contain `"bootstrap"` | Γ£à | `governed_common.rs:46`; `policy_bootstrap__deny_hint__contains_bootstrap`; `exit_contract.rs:164ΓÇô167` |
| AC8 | Omit `--scope` + no context ΓåÆ exit 2 | Γ£à | `policy_bootstrap__no_scope_no_context__exit_2` |
| AC9 | Docs: CAPABILITIES, OPERATIONS/INSTALL, CHANGELOG | Γ£à | All four verified |
| AC10 | No production unwrap | Γ£à | New code uses `match`/`?`/`unwrap_or_else`; no `.unwrap()`/`.expect()` in production paths |
| AC11 | Daemon `POLICY_DENIED_HINT` unit asserts `"bootstrap"` | Γ£à | `services.rs:1226ΓÇô1249` `policy_denied_with_hint__includes_details_hint` |

---

## Findings

### No P0, P1, or P2 findings.

All critical-path findings from Round 1 (R1ΓÇôR5) were addressed before this cross-model review:

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| T210-R1 | medium | AC6 dry-run did not lock zero-append for `register_principal` | `verified_fixed` ΓÇö second dry-run + first real bootstrap assertions added |
| T210-R2 | low_info | `exit_contract` hint assert soft-OR | `verified_fixed` ΓÇö hard `contains("bootstrap")` |
| T210-R3 | low_info | No negative hermetic for dangerous caps | `verified_fixed` ΓÇö `policy_bootstrap__after__dangerous_caps_still_denied` test |
| T210-R4 | low_info | Soft-resolve success path not hermetically locked | `deferred` ΓÇö AC8 fail path is DoD per spec; success path is soft |
| T210-R5 | low_info | Issued grant privacy not asserted | `verified_fixed` ΓÇö T3 test checks `privacy` field via `policy show` |

---

## Completeness

### No stubs or placeholders found.

- **`run_bootstrap`** is fully implemented end-to-end with no TODO, panic, or silent fallback paths.
- **`POLICY_DENIED_HINT`** dual-site strings are identical at both sites with a sync comment linking them.
- **All 9 hermetic tests** exercise concrete behavior with specific value assertions (not just `is_ok`/`is_err`).
- **Documentation** (5 files) contains actionable operator instructions, not placeholder text.
- **`PolicyBootstrapResponse`** is a complete CLI-local serde struct with stable JSON; no contracts DTO leak.

---

## Wiring

The command is registered end-to-end:

1. `main.rs:1121ΓÇô1134`: Clap `PolicyCommands::Bootstrap` variant with `scope: Option<String>`, `dry_run: bool`, `principal_id` (env `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID`), `format` (default `"json"`).
2. `main.rs:2681ΓÇô2694`: Matched and dispatched to `commands::policy_cmd::run_bootstrap`.
3. `policy_cmd.rs:202ΓÇô333`: Full implementation: `resolve_principal` ΓåÆ `StorePorts::from_store` ΓåÆ scope resolve ΓåÆ `parse_scope_key` ΓåÆ `get_principal` ΓåÆ conditional `register_principal` ΓåÆ `active_grants` ΓåÆ per-cap issue/skip ΓåÆ sort ΓåÆ emit.
4. `common/mod.rs`: `hermetic_bin()` correctly strips `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` from the ambient denylist, confirming test isolation is correct.
5. `main.rs:1913ΓÇô1934`: `is_vault_path_free` does not include `Policy { Bootstrap }` ΓÇö bootstrap correctly opens the vault (requires `AppContext`).

---

## Verification

### Test correctness audit

| Test | What it proves | Regression-catching? |
|------|---------------|---------------------|
| `AC1 before` | Deny-by-default is intact on fresh vault | Yes ΓÇö would catch silent allow without grant |
| `AC2 first_run` | Exact three caps issued; sorted; `grant_id` present; `registered:"registered"`; `api_version:"1"` | Yes ΓÇö would catch wrong cap set, missing sort, missing registration |
| `AC3 three_checks_allowed` | Policy evaluator sees grants after bootstrap | Yes ΓÇö would catch broken projection or wrong capability mapping |
| `AC4 source_and_review_list` | Full stack from grant ΓåÆ list command | Yes ΓÇö would catch broken principal resolution in list commands |
| `AC5 second_run` | Idempotency ΓÇö unique index not crashed; no double-register | Yes ΓÇö would catch missing `active_grants` probe or `get_principal` probe |
| `AC6 dry_run` | Three-phase lock: dry-run no-op, second dry-run `would_register`, real bootstrap `registered` | Yes ΓÇö most rigorous gate; would catch any dry-run path that mutates |
| `AC7 deny_hint` | Hint contains `"bootstrap"` on live deny | Yes ΓÇö would catch hint regression |
| `AC8 no_scope` | exit 2 on missing scope with no context | Yes ΓÇö would catch wrong exit code or missing soft-resolve |
| `F2/F3 dangerous_caps` | Erase/Approve*/Export still exit 3 after bootstrap | Yes ΓÇö would catch scope creep into dangerous cap set; also checks `privacy` field |
| `exit_contract.rs:policy_check__deny` | exit_contract suite locks bootstrap in hint | Yes ΓÇö regression across suites |
| `services.rs:policy_denied_with_hint` | Daemon AC11 dual-site unit | Yes ΓÇö would catch daemon hint drift |

---

## Deferred Candidates

### P3 ΓÇö Skill one-liner (soft-optional per spec, not DoD)

**F23** says "optional skill one-liner if agent-facing." Plan.md Phase C has this unchecked. This is explicitly "soft" in both spec and plan. No operator workflow depends on it. Not a regression from prior tracks.

**Proposed deferral:** Absorb into T211+ documentation pass if agent-facing bootstrap guidance is desired.

---

## Completion Decision

| DoD Item | Status |
|----------|--------|
| F-decisions + AC1ΓÇôAC11 met | Γ£à Complete |
| `policy bootstrap` shipped + hermetic suite (ΓëÑ7) | Γ£à 9 tests |
| Hint strings updated CLI + daemon + tests | Γ£à Both sites; both test suites |
| Docs/skill honesty (5 files) | Γ£à Complete; skill one-liner deferred (P3, soft) |
| Review clean for critical/high; Γëñ3 deferred mediums | Γ£à 0 critical/high; 0 mediums; 1 low_info deferred |
| Full gate green (policy_bootstrap 9/9; exit_contract+policy_bootstrap 20/20; daemon AC11 2/2) | Γ£à Per known gates |
| No production `unwrap`/`expect` | Γ£à Confirmed |
| No contracts DTO lift | Γ£à CLI-local only |
| No auto-init grant | Γ£à `run_init` unchanged |
| No dangerous cap bootstrap | Γ£à Hard-enforced by `DISCOVERY_CAPS` const + F2/F3 test |
| Dual-site hint parity (identical wording, sync comment) | Γ£à Confirmed character-for-character |

**PASS ΓÇö T210 is complete and clear for merge.**
