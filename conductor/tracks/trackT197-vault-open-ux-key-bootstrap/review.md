# T197 Review Log — Vault Open UX + Key Bootstrap

## Scope

Stop SQLCipher wrong-key stderr floods; eliminate silent all-zero default on CLI key resolve; shared `resolve_operator_sqlcipher_key` + F8 message family / JSON codes; doctor missing vs wrong; init generate+print; docs bootstrap; log policy install CLI + daemon.

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| Internal R1 | subagent | PASS WITH DEFERRED P3 | AC2 process coverage optional; full gate pending |
| Orchestrator | — | Fixed AC2 preflight/project process tests | 12/12 vault_key_bootstrap |
| Codex R1 | gpt-5.6-luna | **FAIL** | P2 stale migrate zero-key docs; P1 full gate process |
| Orchestrator | — | Fixed docs/help; full gate 1898 pass; brain TempEnv | deny/audit/verify ok |
| Codex R2 | gpt-5.6-luna | **FAIL** | P2 init generated key `String` not zeroized |
| Orchestrator | — | `Zeroizing<String>` for bootstrap material | retest 12/12 |
| Codex final | gpt-5.6-luna | **PASS WITH DEFERRED P3** | No P0–P2; closeout process residual |

## Findings dispositions

| ID | Severity | Status | Disposition |
|----|----------|--------|-------------|
| Internal AC2 process | P3 | **verified_fixed** | preflight + project list missing-key tests |
| Codex R1 P2-001 migrate docs | P2 | **verified_fixed** | clap help + OPERATIONS Keys row |
| Codex R1 P1-001 full gate | P1 process | **verified_fixed** | workspace gate evidenced before PR |
| Codex R2 P2-001 Zeroizing | P2 | **verified_fixed** | `Zeroizing<String>` in `run_init` |
| Daemon AI_BRAINS_VAULT_KEY silent zero | residual | **out_of_scope** | F11 log silence only; T199/ops |
| Orchestrator closeout | P3 | process | PR + conductor + ledger |

## DoD matrix (final)

| AC | Status |
|----|--------|
| AC1 no hmac spam | Met |
| AC2 F8 family AppContext | Met (+ process tests) |
| AC3 format pre-open | Met |
| AC4 missing not zero | Met |
| AC5 docs | Met |
| AC6 hermetic | Met |
| AC7 secrets / no crypto redesign | Met |
| AC8 zero/wrong fail-closed | Met |
| AC9 full gate | Met (local) + CI |
| AC10 7 sites | Met |
| AC11 doctor skip vs fail | Met |
| AC12 JSON codes | Met |
| AC13 install sites | Met |

## Gate evidence (local)

```
cargo fmt --check                          OK
cargo clippy --workspace --all-targets -- -D warnings  OK
cargo nextest run --workspace              1898 passed, 1 skipped
cargo deny check                           OK
cargo audit                                warnings only (allowed)
ledgerful verify --scope fast              Verification passed
```

## Completion decision

Engineering DoD met; Codex final **PASS WITH DEFERRED P3**. **Shipped:** PR #80 squash-merged `72dfa62` (2026-08-03). CI gate-windows/linux/macos SUCCESS. Conductor Completed; deferred residual struck.
