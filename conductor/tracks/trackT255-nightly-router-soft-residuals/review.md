# T255 Review Log — Nightly / router soft residuals

**Track:** T255-NightlyRouterSoftResiduals
**Status:** ✅ **Completed** — product DoD met; CX1 product PASS; CX2 is the fresh final gate
**Ledger TX:** `646bd578-95ab-4220-9c05-306996ae6930` (FEATURE)

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| Internal completeness R1 | explore | FAIL (easy P3) | Product F/AC wired. P3 OPERATIONS cited T255/F16 instead of F14. |
| Internal correctness R1 | explore | PASS WITH DEFERRED P3 | No P0/P1/P2. Three easy coverage P3s. |
| Fix | orchestrator | — | Running fixture; multi-import unreadable/ok units; F20 `{}`; OPERATIONS F14; Pretty clap unit. |
| Internal R2 | explore | **PASS** | Prior P3s verified_fixed. No new P0–P2. |
| Codex CX1 | gpt-5.6-luna high | **PASS** | 0 product P0–P3. Process closeout left to orchestrator. |
| Codex CX2 | pending | — | Fresh final gate after closeout. |

## Findings

| ID | Sev | Status | Description |
|----|-----|--------|-------------|
| CR1-P3a | low | verified_fixed | AC5 did not assert `Status: Running` |
| CR1-P3b | low | verified_fixed | Multi-import JSON unreadable/ok untested |
| CR1-P3c | low | verified_fixed | F20 non-array `{}` untested |
| IR1-P3a | low | verified_fixed | OPERATIONS cited T255/F16 for `.cmd` (is F14) |
| IR2-P3a | low | verified_fixed | No clap unit for `--format Pretty` |

## Manual AC11 (source bin, 2026-08-16)

| Command | Result |
|---------|--------|
| `cargo run -q -p ai-brains-cli -- nightly --status --quick` | Human header; Last Result **1** + hint; missing `.cmd` + dry-run next; **Router: Running  last result: 267009** + SCHED_S hint; `probe=skipped`; Multi-import never; exit **0** |
| `… nightly --status --format json` | JSON object; `schema_version` 1; `action_target_missing: true`; `router.last_result: "267009"` / `Running`; embedding host `127.0.0.1:8083`; exit **0**. Probes `timeout` this session (env; F18 unchanged). |
| `… nightly --status --quick --format json` | `completion.probe` / `embedding.probe` == `"skipped"` |
| `… nightly --help` | after_help: default human; pipes stay human; `--format json` example |
| `… nightly --format json` | clap requires `--status`; exit **2** |

Did **not** `schtasks /change`, write `nightly-run.cmd`, or mutate Router.

## Gate evidence

| Check | Result |
|-------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo nextest run --workspace` | **2962 passed**, 1 skipped |
| Targeted hermetics | AC9/AC10/AC14 in `tests/nightly_status.rs` |
| `cargo deny check` / `cargo audit` | not on PATH (same T251/T252 residual) |
| Codex CX1 | **PASS** (0 findings) |

## Completion decision

Engineering DoD met. Internal reviews clean. Findings greater than low resolved. CX1 product **PASS**. Soft residuals F12 recorded in `conductor/deferred.md`. Fresh CX2 after closeout is the final gate.
