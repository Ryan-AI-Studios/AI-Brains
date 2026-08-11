# T226 — Review Log (internal read-only)

**Track:** T226-PolicySoftResolveScope  
**Branch:** `feat/T226-policy-soft-resolve-scope`  
**Reviewer:** Grok Build subagent (strict READ-ONLY)  
**Date:** 2026-08-11  
**Scope reviewed:** CLI show/check soft-resolve, F23 canonicalize, CP `parse_scope_key` case-insensitive kinds, hermetic tests, docs

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| R1 internal | Grok Build (read-only) | **CLEAN** | AC1–AC12 Met in code+tests; special concerns checked; no critical/high/medium product defects |
| R1 Codex | gpt-5.4 high | **FAIL** (process P2 + help-test P3) | P2 closeout incomplete (expected mid-track); P3 help Usage lock weak |
| R1 fix | orchestrator | verified_fixed | Strengthened help tests (`!Usage --scope` + soft-resolves phrase); AC7 INVALID_PAYLOAD assert |
| R2 Codex | gpt-5.4 high | **PASS WITH DEFERRED P3** | Product clean; process closeout residual only |
| Final Codex | gpt-5.4 high | **PASS WITH DEFERRED P3** | O1 soft residual only; process checklist reconciled |

## Scope audited (claimed vs tree)

| Claim | Evidence |
|-------|----------|
| CLI Show/Check `Option` scope + after_help both parent sites | `main.rs` ~641, ~1211 parent; Show/Check/Bootstrap after_help; `scope: Option<String>`; arg docs soft-resolve |
| `policy_cmd` soft-resolve + F23 canonicalize | `run_show` / `run_check`: `resolve_scope_key_for_cli` → `parse_scope_key` → `scope_identity_key` before list/check/messages |
| Bootstrap F21 single helper | `run_bootstrap` uses same helper path (not dual branch) |
| exit_contract AC1–3, AC7–8 | `exit_contract.rs` tests present with three stderr asserts (AC1/2), help optional, erasure helper retained, malformed 6, missing capability clap English |
| policy_soft_resolve AC4/5/12 | `policy_soft_resolve.rs` seeded grants + canonical scope + lowercase parity |
| CP case-insensitive kind prefixes | `sources.rs` `to_ascii_lowercase` + unit tests + rustdoc |
| Docs | CLI-EXIT-CODES, CAPABILITIES, OPERATIONS, CHANGELOG T226 |

## Special-concern checklist

| Concern | Result |
|---------|--------|
| Erasure still clap-required (F12/M3); `assert_help_scope_required` retained | **Pass** — `erasure request|wipe` `scope: String`; helper used by `erasure_request__help__scope_required` only (not show/check) |
| AC1/AC2 three stderr asserts | **Pass** — both tests assert (a) not clap required text (b) `--scope`/`scope resolve` (c) not filled silently / not authoritative |
| Canonical scope in CheckResult, deny messages, grant list query | **Pass** — `scope_key` after F23 used in `list_applied_grants`, deny message `on {scope_key}`, `CheckResult.scope`, human headers |
| No `#[arg(env)]` on `--scope` | **Pass** — Show/Check/Bootstrap scope are `#[arg(long)]` only; `principal_id` retains env (out of F2) |
| Both parent after_help (~641 and ~1211) | **Pass** — identical omit-when-authoritative examples at Commands::Policy and PolicyCommands |
| CP case-insensitive change correct? unit tests? docs honesty? | **Pass** — required for AC12: store keys are always `scope_identity_key` (`scope_grant_projection.scope_key = ?` exact); without kind widen + F23, `repository:` would fail parse or miss rows. Unit tests in `parse_scope_key_tests`. Rustdoc honest; operator docs describe canonicalize (not kind-case prose) |
| `list_applied_grants` uses canonical key | **Pass** — queries with F23 `scope_key`; projection writes via store `scope_identity_key` |
| Production `unwrap`/`expect` in track paths | **Pass** — none in `policy_cmd.rs` production; `sources.rs` `expect` only under `#[cfg(test)]` |

## Findings

### T226-R1-L1

- **id:** T226-R1-L1
- **severity:** low_info
- **description:** AC7 hermetic (`policy_show__malformed_explicit_scope__exit_6_class`) locks exit **6** only; does not assert `INVALID_PAYLOAD` envelope body the way `policy_check__unknown_capability__exit_6_invalid_payload` does. Product path is correct (`fail_cp` after `parse_scope_key` → InvalidPayload → 6); test discrimination is slightly thinner than peers.
- **files:** `crates/ai-brains-cli/tests/exit_contract.rs`
- **required_fix:** Optional — add stdout/stderr `INVALID_PAYLOAD` or message contains assert if tightening red locks.
- **status:** verified_fixed — AC7 asserts stdout `INVALID_PAYLOAD` or `unparseable`

### T226-R1-L2

- **id:** T226-R1-L2
- **severity:** low_info
- **description:** AC3 help tests assert Usage `[OPTIONS]` and that `--scope` is documented, but do not lock the clap help phrase “soft-resolves when authoritative”. Arg doc comments in `main.rs` already supply that text to clap.
- **files:** `crates/ai-brains-cli/tests/exit_contract.rs`, `crates/ai-brains-cli/src/main.rs`
- **required_fix:** Optional — `stdout.contains("soft-resolves")` (or equivalent) on show/check help.
- **status:** verified_fixed — help tests assert soft-resolves phrase + Usage must not contain `--scope` (Codex R1 P3)

### T226-Codex-R1-P3 (help Usage regression lock)

- **id:** T226-Codex-R1-P3
- **severity:** medium → fixed
- **description:** Help tests only checked `[OPTIONS]` which clap emits even when `--scope` is required on Usage.
- **status:** verified_fixed — assert `!usage_line.contains("--scope")`

### T226-R1-L3

- **id:** T226-R1-L3
- **severity:** low_info
- **description:** Operator docs (CLI-EXIT-CODES / CAPABILITIES / CHANGELOG) describe soft-resolve + canonicalize but do not explicitly state that kind prefixes are case-insensitive (`repository:` ≡ `Repository:`). Control-plane rustdoc on `parse_scope_key` is honest; AC12 hermetic proves CLI parity. Docs honesty residual only.
- **files:** `Docs/CLI-EXIT-CODES.md`, `Docs/CAPABILITIES.md`, `CHANGELOG.md`, `crates/ai-brains-control-plane/src/sources.rs`
- **required_fix:** Optional one-liner under soft-resolve / T226 docs if product wants operator-visible case-insensitivity SOOT outside rustdoc.
- **status:** open

### T226-R1-L4

- **id:** T226-R1-L4
- **severity:** low_info
- **description:** Process DoD incomplete at review time (expected mid-track): `conductor.md` T226 still **Planning**; `deferred.md` “policy show/check required scope” not struck; plan Manual evidence still `pending`; this reviewer session did not execute `cargo nextest` (no shell in read-only subagent). Product AC tests are present and coherent; gate/manual evidence remains implementer/orchestrator duty for AC11/DoD.
- **files:** `conductor/conductor.md`, `conductor/deferred.md`, `conductor/tracks/trackT226-policy-soft-resolve-scope/plan.md`
- **required_fix:** On finalize — run full CI gate; record manual dogfood §11; strike deferred; mark conductor Completed; ledger commit.
- **status:** verified_fixed — conductor Completed, deferred struck, plan checklist [x], gate/manual recorded

## Non-findings (explicit)

- Soft-resolve **only** show/check; bootstrap already soft; erasure / review resolve / propose mutations stay clap-required `String`.
- Capability on `policy check` stays required (AC8 clap English opposite of AC1/AC2).
- No DTO / contracts / daemon IssueGrant / grant matrix / clap pin bump.
- F23 wire matches plan sketch; soft-fill already returns canonical from helper, but explicit path is re-canonicalized (required for AC12).
- F21 bootstrap unify done (soft residual completed, not required DoD).
- `list_applied_grants` exact match would fail on raw lowercase store keys; production writers always emit canonical keys — case-insensitive **parse** + F23 is the correct fix, not SQL `lower(scope_key)`.

## Verdict: CLEAN

No open critical / high / medium product findings. Low_info items are optional test/docs/process polish; none block ship of the implementation under review policy (mediums fixed-or-deferred; critical/high must be verified_fixed).

## AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| **AC1** | **Met** | `policy_show__missing_scope_no_context__exit_2_fail_usage`: exit 2 + three stderr asserts (not clap required; `--scope`/`scope resolve`; not filled silently / not authoritative). Uses `--no-project-context` + hermetic ambient strip. |
| **AC2** | **Met** | `policy_check__missing_scope_no_context__exit_2_fail_usage`: same three asserts with `--capability ReadEvidence`. |
| **AC3** | **Met** | `policy_show__help__scope_optional_soft_default` + `policy_check__help__scope_optional_soft_default`: Usage `[OPTIONS]`; `--scope` documented; arg docs soft-resolve. `assert_help_scope_required` retained for erasure (M3). |
| **AC4** | **Met** | `policy_show__authoritative_project_id__soft_resolve_seeded_exit_0`: F16 `AI_BRAINS_PROJECT_ID` + seed via `open_seeded_ports`/`issue_grant`; exit 0 non-empty grants with ReadEvidence on canonical scope. |
| **AC5** | **Met** | `policy_check__authoritative_project_id__soft_resolve_seeded_allow`: exit 0, `allowed: true`, `scope == Repository:{PROJECT}` (F23/M4). |
| **AC6** | **Met** | `policy_show__with_scope_empty_vault__exit_0` + deny/check explicit-scope paths (`policy_check__deny__exit_3_details_hint`, bootstrap suite). |
| **AC7** | **Met** | exit 6 + stdout `INVALID_PAYLOAD`/`unparseable` assert |
| **AC8** | **Met** | missing capability → clap English |
| **AC9** | **Met** | bootstrap + discovery + erasure help green in full nextest |
| **AC10** | **Met** | docs honesty |
| **AC11** | **Met** | full gate 2534 + CI Win/Linux/macOS + manual debug-bin dogfood |
| **AC12** | **Met** | lowercase explicit + CP unit |

## Definition of Done (final)

| DoD item | Status |
|----------|--------|
| AC1–AC12 | **Met** |
| No open critical/high/medium | **Clean** (O1 soft residual only) |
| Full CI gate green | **2534** local; CI green PR #130 |
| conductor Completed; deferred struck; series README | **Done** closeout |
| Ledger commit + pin | closeout branch |

## Gate evidence (orchestrator)

| Check | Result |
|-------|--------|
| `cargo fmt --check` | OK |
| `cargo clippy --workspace --all-targets -D warnings` | OK |
| `cargo nextest run --workspace` | **2534 passed**, 1 skipped |
| `cargo deny check` / `cargo audit` | OK |
| `ledgerful verify --scope full` | OK |
| CI PR #130 | Win/Linux/macOS **pass** |
| Manual | `target\debug\ai-brains.exe policy show --help` → `[OPTIONS]` + soft-resolves |

## Completion decision

**Engineering DoD met. Track T226 Completed** after PR #130 squash `5919f26` + governance closeout. Soft residual: O1 shared wrapper; T210 bootstrap success soft hermetic.
