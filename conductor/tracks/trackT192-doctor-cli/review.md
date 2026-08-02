# T192 Review Log — Doctor CLI

## Scope
Ship read-only `ai-brains doctor` (F1–F34, AC1–AC16): contracts v1, F17b non-mutating backup list, early handler + `open_read_intent`, kit/event recoverability, docs/claims flip, remove invented-doctor claims rule #54.

## Reviewers / rounds

| Round | Reviewer | Verdict |
|-------|----------|---------|
| Internal R1 completeness | subagent | PASS (P2 AC8 test open) |
| Internal R1 correctness | subagent | FAIL (P1 INSTALL residual + P2s) |
| Fixes | orchestrator | P1/P2 + easy P3s |
| Internal R2 | subagent | **PASS WITH DEFERRED P3** |
| Codex R1 | gpt-5.6-luna high | FAIL (P1 overflow, claims, P2 reparse/docs) |
| Fixes | orchestrator | checked_mul, claims export rule, reparse unit inject, INSTALL |
| Codex R2 (final gate) | gpt-5.6-luna high | **PASS WITH DEFERRED P3** |

## DoD matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | `doctor` in help; `doctor_cli` hermetic suite |
| AC2 | Met | happy path exit 0 ok\|degraded |
| AC3 | Met | wrong key exit 1 |
| AC4 | Met | missing vault fail no create |
| AC5 | Met | daemon down ok; unit inject daemon-up |
| AC6 | Met | kit good/bad + reparse unit inject |
| AC7 | Met | no-secrets process test |
| AC8 | Met | build_report daemon-up + process read-only language |
| AC9 | Met | schema_version=1 JSON + human default |
| AC10 | Met | `--fail-on-degraded` exit 1 |
| AC11 | Met | docs flipped; rule #54 removed; claims OK |
| AC12 | Met | zero new prod deps; deny/audit green |
| AC13 | Met | full gate + reviews; deferred #2 struck at closeout |
| AC14 | Met | no models/graph on doctor path |
| AC15 | Met | no backups/ create |
| AC16 | Met | RecoveryKitCreated event ok (live unquoted storage) |

## Findings dispositions

| ID | Sev | Disposition |
|----|-----|-------------|
| INSTALL “Doctor still absent” | P1 | **verified_fixed** |
| Impl-Plan §8 doctor never-built | P2 | **verified_fixed** |
| Kit parse serde echo | P2 | **verified_fixed** |
| AC8 no-migrate test | P2 | **verified_fixed** |
| parse_duration overflow panic | P1 | **verified_fixed** (checked_mul) |
| Claims invented recovery export | P1 | **verified_fixed** (rule removed; T188 product) |
| Reparse skip-pass only | P2 | **verified_fixed** (unit inject) |
| INSTALL recovery-export missing row | P2 | **verified_fixed** |
| process::exit in run_with_daemon_state | P3 | **verified_fixed** (returns i32) |
| zero_key env mere presence | P3 | **verified_fixed** (truthy) |
| Invalid --format silent human | P3 | **verified_fixed** (clap value_parser) |
| Daemon probe bool only | P3 | **deferred** (shared probe API) |
| Spec F16 JSON-quote draft vs live trim | P3 | **deferred** (erratum; code correct) |

## Gates (local)

```
cargo fmt --check                         OK
cargo clippy --workspace --all-targets -D warnings  OK
cargo nextest run --workspace             1821 passed (1 skipped)
cargo deny check                          OK
cargo audit                               OK (19 allowed pre-existing)
scripts/check-release-claims.ps1          OK
doctor_cli + doctor unit                  20 passed
```

## Deferred after ship

1. Daemon probe error vs down distinction (probe API returns bool).
2. Spec F16 erratum: live `event_type` is unquoted after store `trim_matches('"')`.

## Completion decision

Engineering DoD met. Codex R2 final gate: **PASS WITH DEFERRED P3**. Proceed PR → CI → squash-merge → strike deferred #2 / R-DOC-CLI doctor residual.
