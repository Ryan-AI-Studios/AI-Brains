# T199 Review Log — Daemon Status Vault Independence

## Scope

Make `ai-brains daemon status` answer IPC/process liveness without vault open or key; share Status/Safety probe SOOT; soft tasklist; optional vault section with swallow-only memories.

## Reviewers / rounds

| Round | Reviewer | Verdict |
|-------|----------|---------|
| Internal R1 | explore subagent | PASS WITH DEFERRED P3 (P3-1 hermetic soft AC7; P3-2 unreachable) |
| Fix | orchestrator | P3-1 fixed (exact skip assert); P3-2 deferred (matches doctor pattern) |
| Codex R1 | gpt-5.6-luna high | FAIL engineering: P1 dotenv hermetic false-positive; P2 fake 0 B size; P1 process AC10 |
| Fix | orchestrator | `hermetic_bin_no_key` adds `--no-project-context`; size → `unavailable` + unit |
| Codex R2 | gpt-5.6-luna high | **PASS WITH DEFERRED P3** (unreachable only); R1 P1/P2 verified fixed |
| Final Codex | gpt-5.6-luna high (post-merge) | **PASS WITH DEFERRED P3** (same P3 unreachable only; no new >low) |

## Findings disposition

| ID | Sev | Status | Notes |
|----|-----|--------|-------|
| Internal P3-1 | P3 | verified_fixed | exact Memories skip assert |
| Internal P3-2 / Codex P3 | P3 | deferred | dead-arm `unreachable!` matches doctor; non-blocking |
| Codex R1 P1 dotenv | P1 | verified_fixed | `--no-project-context` in helper |
| Codex R1 P2 size 0 B | P2 | verified_fixed | `Vault size: unavailable` |
| Codex R1 P1 process | P1 process | process | AC10 full gate + PR + deferred strike |

## Gates (local)

| Gate | Result |
|------|--------|
| `cargo fmt --check` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| targeted nextest (daemon_status/probe/format) | 15 passed |
| `cargo nextest run --workspace` | **1918 passed**, 1 skipped |
| `cargo deny check` | (recorded at ship) |
| `cargo audit` | (recorded at ship) |
| Manual no-key `daemon status` | `Status: Stopped` exit 0 |

## DoD matrix

| AC | Status |
|----|--------|
| AC1–AC9, AC11–AC13 | Met |
| AC10 | Local nextest+clippy+fmt green; CI PR gate remaining |

## Deferred

- Dead `Status` match arm `unreachable!` (P3) — pattern parity with doctor early-route.

## Ship

- PR #82 squash-merged `721d41f` (2026-08-03)
- CI: gate-windows / gate-linux / gate-macos SUCCESS
- Codex final: **PASS WITH DEFERRED P3**
