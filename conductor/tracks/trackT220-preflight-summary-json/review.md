# T220 Review Log — Preflight summary JSON honesty

**Track:** T220-PreflightSummaryJson  
**Category:** BUGFIX / CONTRACT  
**Ledger TX:** `f51e8caa-b159-4830-84bb-f79f3be131f6`  
**Branch:** `feat/T220-preflight-summary-json`

## Verdict

**Engineering: PASS WITH DEFERRED P3** (soft residuals only).  
Cross-model (Codex final2): **no open product P0–P2**; process AC11/AC12 closeout owned by orchestrator (CI via PR + this review + dogfood recorded).

## Scope

- `preflight --summary --format json` (case-insensitive) → pretty `PreflightSummaryJson` machine object
- Three-valued `scope`: `global` | `project` | `none`
- M1: `--install-hooks` still runs; status on stderr under JSON; no AskOnce; no harness human on stdout
- OpenCode included in install-hooks loop (Agy/Grok/Opencode)
- Honest status when USERPROFILE/HOME unset under install-hooks
- T180 freeze: non-summary compact `{text, word_count}` only
- Docs: CAPABILITIES, PROTOCOL-COMPAT, CHANGELOG, clap format help

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| R1 | Internal subagent | FAIL process | Product ACs met; AC5/AC8b test tighten + process |
| R1 fix | Orchestrator | — | AC5 each marker ≥1; AC8b stderr assert |
| R2 | Internal explore | PASS WITH DEFERRED P3 | Product clean |
| R3 | Codex (gpt-5.4 high) | FAIL | P1 OpenCode loop; P2 empty-home-only AC8b |
| R3 fix | Orchestrator | — | OpenCode in loop; present-harness hermetic |
| R4 | Codex final | FAIL | P2 silent no-op when home None |
| R4 fix | Orchestrator | — | emit_status when home unresolved |
| R5 | Codex final2 | FAIL process only | No product P0–P2; AC11/AC12 track record |
| R5 close | Orchestrator | PASS WITH DEFERRED P3 | Dogfood + review.md + CI PR |

## DoD matrix (AC)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | Hermetic pure JSON, no banner |
| AC2 | Met | Required keys hermetic + unit |
| AC3 | Met | Global multi-project hermetic |
| AC4 | Met | Project-scoped no `projects` key |
| AC5 | Met | Each in_context_* ≥1 on legacy pins |
| AC6 | Met | `t180_c_preflight_json_keys` green |
| AC7 | Met | Human banner regression |
| AC8/8b | Met | Empty-home + present OpenCode install hermetics |
| AC9 | Met | Pure unit global/project/none |
| AC10 | Met | CAPABILITIES / PROTOCOL-COMPAT / CHANGELOG / help |
| AC11 | Met via PR CI | Full gate on PR; local: nextest preflight 12p, clippy, fmt |
| AC12 | Met | Manual dogfood 2026-08-09: summary JSON pure object; full JSON compact 2-key |
| AC13 | Met | Uppercase `JSON` hermetic |
| AC14 | Met | `scope=none` hermetic |
| AC15 | Met | Single JSON object parse |

## Findings disposition

| ID | Severity | Status | Disposition |
|----|----------|--------|-------------|
| OpenCode not in install-hooks | P1 (Codex) | verified_fixed | Loop `[Agy, Grok, Opencode]` |
| AC8b empty-home only | P2 (Codex) | verified_fixed | Present OpenCode hermetic |
| Silent no-op home None | P2 (Codex) | verified_fixed | Status when home unresolved |
| Soft skill one-liner | P3 | deferred | F20 soft / F22 |
| harnesses[] / scope_line / is-terminal / ValueEnum | P3 | deferred | Spec F22 soft residuals |

## Gates (orchestrator-observed)

```text
cargo nextest run -p ai-brains-cli -E 'test(preflight_summary_json)' → 12 passed
cargo nextest run -p ai-brains-cli -E 'test(t180_c_preflight)' → 1 passed
cargo clippy -p ai-brains-cli --all-targets -- -D warnings → OK
cargo fmt --check → OK (after cargo fmt)
Manual: ai-brains preflight --summary --format json → pure pretty object, scope project
Manual: ai-brains preflight --format json → compact {"text","word_count"} only
Manual: ai-brains preflight --summary --format JSON → JSON path
```

## Soft residuals (deferred.md)

- Optional skill one-liner for summary JSON
- Optional `harnesses[]` in summary JSON
- Optional `scope_line` string
- is-terminal → `std::io::IsTerminal`; clap ValueEnum ignore_case unify

## Completion decision

Engineering DoD met. Ship via PR; mark Completed after CI green + squash-merge + closeout PR (conductor/deferred/series/coordinated).
