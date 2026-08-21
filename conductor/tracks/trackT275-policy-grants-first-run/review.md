# T275 review log — Policy grants first-run (grant-wall + CLI bootstrap lock)

**Track:** T275-PolicyGrantsFirstRun  
**Status:** Completed  
**FEATURE TX:** `1f2c1ddb-5657-4af9-9a30-8285efca8895`  
**HEAD (implement):** `track/T275-policy-grants-first-run`

## Reviewers / rounds

| Round | Reviewer | Result |
|-------|----------|--------|
| R1 | Implementer (Grok) vs spec AC1–AC16 / DoD | **PASS** — AC1/AC2 red then green; AC3 JSON lock added; AC4/AC5 hermetic green; F35/F36/F37 held |
| R1b | Internal explore subagent vs spec | **PASS** — no P0–P3 product findings |
| CX1 | Codex (FEATURE) `review.codex.md` | Product AC1–AC16 **PASS**. P1-1 full gate / P2-1 governance were process (mid-implement). Both `verified_fixed` at closeout. |

## Finding fields

id, severity, description, source, files, required_fix, status, evidence.

## Findings

### CX1-P1-1 — Mandatory full verification not evidenced at Codex snapshot

- **severity:** high (process)
- **source:** Codex CX1
- **status:** `verified_fixed`
- **evidence:** `.\scripts\dev-check.ps1` **SUCCESS** (fmt + workspace clippy `-D warnings` + nextest **3253 passed / 1 skipped** + deny + audit 19 allowed). `ledgerful verify --scope full` exit 0 (fmt/clippy/nextest/deny/audit all ok).

### CX1-P2-1 — Track governance not finalized at Codex snapshot

- **severity:** medium (process)
- **source:** Codex CX1
- **status:** `verified_fixed`
- **evidence:** `review.md` + `review.codex.md` force-added; conductor/spec/plan/README/deferred closed to Completed; FEATURE TX commit + publish Phase 6.

## DoD matrix (AC1–AC16)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | `render_project_markdown__denied__no_none_placeholder` — red on `_None_`, green after grant-wall + hidden |
| AC2 | Met | GRANT_WALL **88** chars, no `\n`, ≤140; renderer order next → wall → `## Decisions` |
| AC3 | Met | `briefing_project__no_grants__json_denied_empty_arrays` — `denied: true`, `[]` not null, `denial_hint` bootstrap, exit 0 |
| AC4 | Met | `policy_bootstrap__after_system__briefing_project_denied_false` omit `--principal-id` (F36) |
| AC5 | Met | `policy_bootstrap__after_system__evidence_list_exit_0` `--scope` + `--local`, omit `--principal-id` |
| AC6 | Met | allowed-empty still `_None_` + empty_authority; grant-wall denied-only |
| AC7 | Met | T221 `progressive__no_grants` exit 3 + `progressive__after_system_bootstrap` `denied: false` (33-test targeted run) |
| AC8 | Met | T263 list deny exit 3 (same targeted run) |
| AC9 | Met | T210 `policy_bootstrap__after__dangerous_caps_still_denied` |
| AC10 | Met | grant-wall names `recall`; no new grant gate on capture/recall |
| AC11 | Met | CAPABILITIES denied packets, OPERATIONS grant-wall sentence, CHANGELOG, skill one-liner |
| AC12 | Met | no production `unwrap`/`expect`/`panic`; clap lock 4.6.1; rusqlite 0.39.0; no DTO keys |
| AC13 | Met | `POLICY_DENIED_HINT` + twins untouched |
| AC14 | Met | `doctor.rs` not edited; 15-check matrix stands |
| AC15 | Met | live `policy bootstrap --dry-run` only; owner did not confirm apply |
| AC16 | Met | personal deny extended T263 unit — no GRANT_WALL / HIDDEN / NEXT_STEP / `policy bootstrap` |

## Targeted gates (R1)

```text
cargo nextest run -p ai-brains-control-plane --lib <renderer T275 filters>
  8 passed (2026-08-21)

cargo nextest run -p ai-brains-cli --test policy_bootstrap --test governed_first_run_deny_exit --test governed_vault_pin_honesty
  33 passed (2026-08-21); later AC3 lock +1 passed

cargo clippy -p ai-brains-control-plane -p ai-brains-cli --all-targets -- -D warnings
  exit 0
```

## Manual (AC15)

```text
ai-brains policy bootstrap --dry-run
  registered: already; would_issue ×3 ReadConclusions/ReadDecisions/ReadEvidence
  dry_run: true

ai-brains briefing project --format human
  (PATH binary, pre-T275) still `_None_` until cargo install — expected F18
  Hermetic proof is AC4/AC5 via cargo run test bin
```

Live operator vault **not** bootstrapped.

## Residuals

- PATH `ai-brains` until user asks `cargo install` / `Build-AIBrains.ps1` (F18)
- Live grants 0 of 3 until owner confirms bootstrap (F10)
- T280 omit-`--scope` on deny hint (peer)
- Personal denied `_None_` left (F32)
