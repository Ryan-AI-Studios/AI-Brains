# T213 Review Log — Graph density doctor

## Scope
Pure density assessor + `graph update` status vocabulary (`live`|`sparse`|`empty`) + doctor soft check `graph_density` (#10 of 11). Capture-independent SQL only. Typed-lineage floor **0.50**. Keep `note`. No auto-rebuild. Zero new crates.

## Reviewers / rounds

| Round | Source | Verdict | Notes |
|-------|--------|---------|-------|
| Internal R1 | explore subagent | **PASS** | 0 P0–P2; 3 optional P3 (AC8 JSON keys, sparse remediation assert, CAPABILITIES skip list) |
| Internal R1 fix | orchestrator | fixed | AC8 serde units + T74 field asserts; sparse/orphan remediation asserts; CAPABILITIES pinned-fail skip |
| Completeness R2 | explore subagent | **PASS** | All P3 landed; ready for cross-model |
| Cross-model R1 | Claude (Codex rate-limited) | **PASS** | 0 P0–P2; process P3 nextest-pending (resolved) |
| Final cross-model | Claude `review.claude.final.md` | **PASS** | Fresh clean final gate; 0 findings |

## Findings

### Internal R1

| ID | Sev | Status | Disposition |
|----|-----|--------|-------------|
| R1-P3-AC8 | P3 | **verified_fixed** | `graph_health_output__serde_keys__include_density_fields` + T74 expanded field asserts |
| R1-P3-AC9 | P3 | **verified_fixed** | sparse/orphan remediation contains `rebuild`; AC9 sparse JSON unit |
| R1-P3-docs | P3 | **verified_fixed** | CAPABILITIES skip list includes pinned count failed |

### Cross-model
None open. Soft residuals only (F31/F17/F24/L4/L6) already in `conductor/deferred.md`.

## Gate evidence

| Gate | Result |
|------|--------|
| Focused density/doctor units | **25 passed** |
| Graph-on serde + T74 smoke | **3 passed** |
| `cargo nextest run --workspace` | **2177 passed**, 1 skipped |
| `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` | green |
| `cargo deny check` | ok |
| `cargo audit` | exit 0 (allowed warnings only) |
| Manual dogfood (graph-on) | live vault **1305n/96e**, E/N≈0.074 → `status=sparse` `density=warn` remediation rebuild; exit 0 |
| Feature-off `graph update` | exit **2** `FEATURE_UNAVAILABLE` |
| Doctor JSON | `graph_density` present; overall degraded when warn |

## Soft residuals (deferred.md)

- F31 event↔graph freshness
- F17 CLI threshold flags
- F24 promote GraphHealthOutput to contracts
- L4 rusqlite 0.40 table_exists
- L6 two-tier memory coverage
- Skill one-liner

## Completion decision

Engineering DoD met. Final cross-model **PASS**. Ship via PR; mark conductor **Completed** after squash-merge + CI green.
