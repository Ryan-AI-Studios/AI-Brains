# T201 Review — CLI Error Envelope + Exit Code Contract

- **Track:** T201-CliErrorExitContract  
- **Branch:** `agent/T201-cli-error-exit-contract`  
- **Reviewer:** internal read-only (cross-model / AC13)  
- **Date:** 2026-08-03  
- **Scope:** Uncommitted implementation on branch vs track spec/plan (code + docs + hermetic suite)  
- **Verdict:** **CLEAN** (R1 lows L1–L3 fixed; Codex R1 product PASS; process P1 resolved with full-gate evidence)

---

## Summary

T201 lands the preferred F4 path (clap-required `--scope: String` on `policy show`, `review list`, `erasure request`), structured `POLICY_DENIED` `details.hint`, normative `Docs/CLI-EXIT-CODES.md`, dual-envelope honesty in CAPABILITIES/OPERATIONS, CONTRIBUTING link, CHANGELOG **BREAKING**, soft `INVALID_TRANSITION`→1, and hermetic `tests/exit_contract.rs` locks. Daemon defensive None arms are retained. No high/critical product defects found against AC1–AC12 behavioral requirements.

Process residual: plan Phase D full gate (D1) not marked complete at review time — not a code finding.

---

## DoD / AC matrix

| AC | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| **AC1** | CLI-EXIT-CODES.md complete (0–7, FEATURE_UNAVAILABLE→2, doctor `--fail-on-degraded`, status 0, 130 footnote, vault exception, daemon IPC note, dual envelopes) | **PASS** | `Docs/CLI-EXIT-CODES.md` full table + footnotes §§ FEATURE_UNAVAILABLE, Doctor, Daemon status, Exit 130, Vault exception, Dual envelopes, F35 daemon IPC |
| **AC2** | `policy show` missing `--scope` → exit **2** | **PASS** | clap `scope: String` in `main.rs` PolicyCommands::Show; CLI None branch removed; lock `policy_show__missing_scope__exit_2` |
| **AC3** | `review list` missing `--scope` → exit **2** | **PASS** | clap `scope: String` ReviewCommands::List; lock `review_list__missing_scope__exit_2` |
| **AC3b** | `erasure request` missing `--scope` → exit **2** | **PASS** | clap `scope: String` ErasureCommands::Request; wire still `Some(scope)`; lock `erasure_request__missing_scope__exit_2` |
| **AC4** | POLICY_DENIED Json: `details.hint` non-empty; exit 3 | **PASS** | `policy_cmd::run_check` `.with_details(policy_denied_hint_details())`; locks in `exit_contract.rs` + `governed_surface.rs` |
| **AC5** | F18 suite ≥6 locks incl graph 2 | **PASS** | `tests/exit_contract.rs`: (1) show miss 2, (2) list miss 2, (3) deny 3+hint, (4a) graph 2 `#[cfg(not(feature="graph"))]`, (4b) vault 1, (5) success 0, (6) INVALID_PAYLOAD 6 — ≥6 |
| **AC6a** | CAPABILITIES + OPERATIONS stream honesty | **PASS** | CAPABILITIES dual-envelope table (Json stdout / Human stderr); OPERATIONS lines 134–136 link CLI-EXIT-CODES + format-dependent streams; no false “always stderr” for governed Json |
| **AC6b** | CONTRIBUTING links CLI-EXIT-CODES | **PASS** | `CONTRIBUTING.md` § CLI exit codes → `Docs/CLI-EXIT-CODES.md` |
| **AC7** | CHANGELOG **BREAKING** missing-scope 6→2 | **PASS** | `CHANGELOG.md` Unreleased ### Changed: **T201 BREAKING — missing required `--scope`** for show/list/erasure request; envelope flip documented (not silent) |
| **AC8** | Graph exit 2 in suite; no exit 8+ | **PASS** | Graph lock in suite; product codes remain 0–7 in `governed_common.rs` + docs; no 8+ introduced |
| **AC9** | Full gate | **PROCESS PENDING** | Plan D1 unchecked at review; not a product defect |
| **AC10** | Dual envelope + format-dependent streams documented | **PASS** | CLI-EXIT-CODES dual table + CAPABILITIES/OPERATIONS honesty |
| **AC11** | Clap help shows scope required on flipped commands | **PASS (weak assert)** | `--help` tests exist for three flipped commands; see L1 (assertion not bracket-strict) |
| **AC12** | Daemon None arm retained; F36 inventory recorded | **PASS** | `services.rs` `list_review_items` / `process_request_erasure` None → INVALID_PAYLOAD retained; plan F36 inventory table complete |
| **AC13** | Cross-model / review after F4 | **PASS (this review)** | This log |

### Extra checklist

| Check | Status | Notes |
|-------|--------|-------|
| F4: CLI None branches removed only on three commands | **PASS** | evidence/source/retention/migrate still Option (F36 leave) |
| Daemon services.rs defensive None arms | **PASS** | Untouched |
| No `unwrap`/`expect` in T201 production paths | **PASS** | `policy_denied_hint_details` avoids `json!`; production modules clean |
| Tests hermetic + naming | **PASS** | `hermetic_bin`, `tempdir`, `feature__condition__expected` |
| evidence/source Option leave | **OK** | Documented F36 leave; residual class not F4 |
| Silent envelope flip without CHANGELOG | **PASS** | BREAKING present → not high |
| F9 dual envelope not forced single | **PASS** | Docs affirm dual |
| Soft F30 INVALID_TRANSITION | **PASS** | Map arm → EXIT_INTERNAL=1; unit test; CLI-EXIT-CODES lists it under exit 1 |
| Completeness TODO/FIXME T201 | **PASS** | No incomplete T201 TODOs found |

---

## Findings

### Open

None above low severity.

### Deferred lows

None remaining after post-R1 fixes.

### Fixed (R1 lows)

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| **T201-R1-L1** | low | Weak AC11 help asserts | **verified_fixed** | `assert_help_scope_required` checks Usage has `--scope` and not `[--scope` |
| **T201-R1-L2** | low | Vault 4b should use `hermetic_bin_no_key` | **verified_fixed** | `recall__missing_key__exit_1_vault_key_family` uses helper |
| **T201-R1-L3** | low | Broad hint doc vs implemented sites | **verified_fixed** | CLI-EXIT-CODES narrows to policy check + review list local |

### Fixed / verified (none prior — first review)

N/A — no prior `review.md` findings.

### Out of scope (intentional)

| id | note |
|----|------|
| evidence/source `scope: Option` still exit 6 on missing | F36 **leave**; not F4 flip targets |
| Force single error envelope | F9 / non-goal |
| Remove daemon None arms | Stop-before; AC12 requires retain |
| T203 soft-default | F27 boundary |
| `--exit-codes` flag | F33 deferred |
| Human mode surface of `details.hint` | Soft F6 only; Json is AC |
| Full CI gate run | AC9 process (plan D1) |

---

## F36 inventory cross-check

| Site | Disposition | Verified |
|------|-------------|----------|
| policy show | clap `String` | `main.rs` ~945; `ShowOptions.scope: String` |
| review list | clap `String` | `main.rs` ~889; `ListOptions.scope: String` |
| erasure request | clap `String` + wire `Some(...)` | `main.rs` ~987; `RequestOptions.scope: String` |
| evidence show | leave Option + runtime INVALID_PAYLOAD | still present |
| source show | leave Option + runtime INVALID_PAYLOAD | still present |
| retention apply | leave optional | still Option |
| migrate default_scope | leave optional | still Option |
| Daemon list_review / request_erasure None | retain INVALID_PAYLOAD | `services.rs` ~426–430, ~702–707 |

---

## Regression / safety notes

- **Envelope flip** is documented in CHANGELOG BREAKING → not silent (F19 high class avoided).  
- **Graph FEATURE_UNAVAILABLE→2** locked under default no-graph build (`cfg(not(feature = "graph"))`).  
- **No product exit 8+.**  
- **Capture independence / CQRS / event log** untouched.  
- **Rust safety:** no new production `unwrap`/`expect` in T201 production edit surface.  
- Unit: `exit_code_for_api_error__invalid_transition__1` pins soft F30.

---

## Verdict rationale

All product/docs/hermetic AC (AC1–AC8, AC10–AC12) pass. AC11 help asserts hardened (Usage requires `--scope`, not `[--scope`). AC9 full gate observed locally (1931 nextest). R1 lows L1–L3 fixed. No open critical/high/medium product findings.

**Verdict: CLEAN** (lows fixed; full gate observed)

---

## Codex R1 (cross-model, 2026-08-04)

Raw: [review.codex.md](./review.codex.md)

| Finding | Severity | Disposition | Notes |
|---------|----------|-------------|-------|
| **T201-P1-001** Completion gate not verifiable in Codex sandbox | P1 process | **validated → verified_fixed (orchestrator gates)** | Product AC verified clean by Codex. Orchestrator: fmt OK; clippy OK; nextest **1931** pass; deny OK; audit exit 0; `ledgerful verify --scope fast` passed. Final Codex after CI green + closeout. |

Product findings P0–P3 from Codex R1: **none**.

---

## Gate evidence (orchestrator)

```
cargo fmt --check                              → 0
cargo clippy --workspace --all-targets -D warnings → 0
cargo nextest run --workspace                  → 1931 passed, 1 skipped
cargo deny check                               → 0
cargo audit                                    → 0 (allowed warnings only)
ledgerful verify --scope fast                  → Verification passed
cargo nextest run -p ai-brains-cli --test exit_contract → 11/11
```

## Closeout (2026-08-03 / 2026-08-04)

| Item | Result |
|------|--------|
| PR #84 | Squash-merged `a9e3b85` |
| CI | gate-windows / gate-linux / gate-macos all SUCCESS |
| Final Codex product | **PASS** (process P1 closeout addressed this commit) |
| conductor / deferred | Completed + struck |
| coordinated | T201 Completed note + deferred strike |

## Residual (non-blocking)

- evidence/source `scope: Option` still exit-6 missing-scope class (F36 leave)
- non-universal POLICY_DENIED `details.hint` (policy check + review list local only)
- dual envelopes retained (F9)
