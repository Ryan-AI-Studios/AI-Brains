# T204 Review Log — CLI Help Information Architecture

## Scope
Presentation-only CLI help IA: root `after_long_help` groups, one-line `after_help` tip, F31 `display_order`, F33 `[dangerous]` markers (incl. `daemon update`), F9 progressive project-id after_help, CONTRIBUTING/CAPABILITIES/CHANGELOG docs.

## Reviewers / rounds
| Round | Source | Verdict |
|-------|--------|---------|
| Internal R1 | explore (spec + regression) | NEEDS_FIX mediums M1/M2 (test locks); product ACs Met |
| Internal R2 | explore (post-fix) | **CLEAN** |
| Codex R1 | gpt-5.6-luna high | **FAIL** — P1-002 daemon update unmarked; P2-001 `migrate --confirm` invalid shorthand; P1-001 process closeout |
| Fix pass | orchestrator | P1-002 + P2-001 fixed; F33 test + appendix + CAPABILITIES |
| Cross-model R2 | Claude Sonnet 5 high (Codex usage limit) | **PASS** — no open P0–P2; prior findings verified fixed |

## Findings disposition
| ID | Sev | Disposition |
|----|-----|-------------|
| R1/M1 AC7 weak vs ingest | medium | **verified_fixed** — Commands-section Daily before `ingest` |
| M2 F33 ≥1 only | medium | **verified_fixed** — full F33 table incl. daemon update |
| Codex P1-002 daemon update | high/P1 | **verified_fixed** — `[dangerous]` + appendix install\|uninstall\|update |
| Codex P2-001 migrate --confirm | medium/P2 | **verified_fixed** — `migrate governed --confirm` |
| Codex P1-001 process closeout | process | closed via PR/merge/chore |
| L1–L5 | low | **verified_fixed** |
| R4 §12.3 numeric examples | info | wontfix — bands + §5 order OK |

## Gate (local, pre-PR)
| Check | Result |
|-------|--------|
| `cargo fmt --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo nextest run --workspace` | **2020 passed**, 1 skipped |
| `cargo deny check` | ok |
| `cargo audit` | ok (allowed warnings only) |
| focused `cli_help_ia` + `exit_contract` | 18 passed |

## Residual after ship
- clap 5 multi-heading (#1553)
- man / help topics (F12)
- Full TTY human matrix soft remainder (F8)
- T203 HTTP list soft

## Completion decision
Engineering DoD met; internal CLEAN; Codex R1 FAIL → fix → Claude cross-model **PASS** as final clean gate (Codex R2 rate-limited). Ready for PR + CI + squash-merge + conductor closeout.


## Ship evidence
- PR #87 squash-merged `c3a7d66` (2026-08-04)
- CI: gate-windows / gate-linux / gate-macos all SUCCESS
- Local full gate: 2020 nextest pass; clippy/deny/audit clean
- Cross-model final: Claude **PASS** (Codex R2 rate-limited after R1 FAIL→fix cycle)
