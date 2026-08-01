**Findings**

1. P3: The L11 human sign-off is still written as if it has already happened, but the current repo state is still on `track/T185-claims-sbom-release-gate`, so that acceptance event has not occurred yet. [Docs/RELEASE-CHECKLIST.md](C:/dev/AI-Brains/Docs/RELEASE-CHECKLIST.md:221) keeps the correct future condition, but [Docs/RELEASE-CHECKLIST.md](C:/dev/AI-Brains/Docs/RELEASE-CHECKLIST.md:226) flips to past-tense acceptance. That should stay conditional until the merge actually happens.

2. P3: The generated NOTICE output still treats first-party PolyForm crates as third-party NOTICE entries. [about.toml](C:/dev/AI-Brains/about.toml:34) says private workspace members are ignored, but [THIRD-PARTY.md](C:/dev/AI-Brains/conductor/tracks/trackT185-claims-sbom-release-gate/evidence/dry-run-2026-08-01/THIRD-PARTY.md:8277) still lists 22 `ai-brains*` product crates under `PolyForm-Noncommercial-1.0.0`. This is still only noise/presentation, not a release-gate blocker.

**Assumptions**

- `ledgerful doctor` and `ledgerful ledger status --compact` were unavailable in this read-only pass with `unable to open database file`, so this review was done from the tracked files and repo state.
- I did not rerun the full gate in this pass.

**Verdict**

The three claimed R2 fixes are verified in [spec.md](C:/dev/AI-Brains/conductor/tracks/trackT185-claims-sbom-release-gate/spec.md:5), [plan.md](C:/dev/AI-Brains/conductor/tracks/trackT185-claims-sbom-release-gate/plan.md:3), [generate-sbom.ps1](C:/dev/AI-Brains/scripts/generate-sbom.ps1:118), and [CLAIMS-REGREP.md](C:/dev/AI-Brains/conductor/tracks/trackT185-claims-sbom-release-gate/evidence/CLAIMS-REGREP.md:49). Fresh sweep found no P0-P2. Final disposition: **PASS WITH DEFERRED P3**.