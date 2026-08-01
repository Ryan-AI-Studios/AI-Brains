**Findings**

- `P2` [Docs/RELEASE-CHECKLIST.md](/C:/dev/AI-Brains/Docs/RELEASE-CHECKLIST.md:215), [spec.md](/C:/dev/AI-Brains/conductor/tracks/trackT185-claims-sbom-release-gate/spec.md:285): AC11/L11 is not actually satisfied. The checklist still records an agent-prepared operator, marks `Claims accepted (L1) = Yes`, leaves `Binary ship?` unresolved, and leaves the human signature line blank. The spec requires the human sign-off field to be filled on the dry-run checklist.

- `P2` [spec.md](/C:/dev/AI-Brains/conductor/tracks/trackT185-claims-sbom-release-gate/spec.md:332), [DRY-RUN.md](/C:/dev/AI-Brains/conductor/tracks/trackT185-claims-sbom-release-gate/evidence/dry-run-2026-08-01/DRY-RUN.md:33), [plan.md](/C:/dev/AI-Brains/conductor/tracks/trackT185-claims-sbom-release-gate/plan.md:112), [conductor.md](/C:/dev/AI-Brains/conductor/conductor.md:131): the track is marked `Completed` even though the Definition of Done requires the full local gate on the dry-run commit. The recorded evidence only proves `deny`/`audit` plus the release scripts; it explicitly defers `fmt`/`clippy`/`nextest` to later PR CI. If PR CI substitution was intended, the spec/DoD needed to say that before closeout.

- `P3` [release.yml](/C:/dev/AI-Brains/.github/workflows/release.yml:54), [spec.md](/C:/dev/AI-Brains/conductor/tracks/trackT185-claims-sbom-release-gate/spec.md:168): the release workflow is SHA-pinned, but it does not meet the stated least-write-scope rule. `contents: write`, `id-token: write`, and `attestations: write` are granted for the whole job even when `skip_attest=true` or no GitHub Release is created.

- `P3` [Docs/RELEASE-CHECKLIST.md](/C:/dev/AI-Brains/Docs/RELEASE-CHECKLIST.md:150), [conductor.md](/C:/dev/AI-Brains/conductor/conductor.md:125): the executed checklist leaves the Windows/Linux smoke evidence cells blank. The upstream T179 run exists, but the dry-run checklist does not carry the concrete run ID or path it claims to rely on.

- `P3` [SHA256SUMS](/C:/dev/AI-Brains/conductor/tracks/trackT185-claims-sbom-release-gate/evidence/dry-run-2026-08-01/SHA256SUMS:1), [DRY-RUN.md](/C:/dev/AI-Brains/conductor/tracks/trackT185-claims-sbom-release-gate/evidence/dry-run-2026-08-01/DRY-RUN.md:24): the archived checksum file still points at `sbom/...` paths, but the archived dry-run directory is flat. The evidence pack therefore cannot be re-verified as archived.

- `P3` [check-release-claims.ps1](/C:/dev/AI-Brains/scripts/check-release-claims.ps1:28): the L13 scanner still has a false-negative gap for the forbidden invented `recovery export` CLI claim, and its elevated file list is Windows-path-specific. Current elevated docs are clean, but the automation is not fully covering the stated non-claim set.

**Verdict**

`FAIL`

The core T185 artifacts are mostly there: `RELEASE-CLAIMS` covers all T184 residual IDs, the dry-run SBOMs are per-binary CycloneDX 1.5, NOTICE generation works, no AGPL/GPL release-gate tooling was introduced, and `release.yml` is SHA-pinned. But AC11 and the stated DoD are not met, so the track should not be cleared yet.

No P3 above is difficult enough to justify `deferred.md`; they should be fixed inline with the P2 items. `ledgerful doctor` and `ledgerful ledger status --compact` were not reviewable here because both failed with `unable to open database file`.