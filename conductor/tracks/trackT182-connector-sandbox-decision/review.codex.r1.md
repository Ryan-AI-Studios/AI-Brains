# Track Completion Audit — T182
## Verdict: FAIL

## Scope Reviewed
Branch `track/T182-connector-sandbox-decision` vs `origin/main`, commits `55a85bb` and `4db30cf`.

Reviewed artifacts:
- [ADR-0019](/C:/dev/AI-Brains/Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md:1)
- [threat-model.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/threat-model.md:1)
- [spec.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/spec.md:1)
- [plan.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/plan.md:1)
- [review.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/review.md:1)
- [conductor.md](/C:/dev/AI-Brains/conductor/conductor.md:128)
- [deferred.md](/C:/dev/AI-Brains/conductor/deferred.md:643)
- [manifest.rs](/C:/dev/AI-Brains/crates/ai-brains-sources/src/manifest.rs:74)
- [registry.rs](/C:/dev/AI-Brains/crates/ai-brains-sources/src/registry.rs:68)
- [lib.rs](/C:/dev/AI-Brains/crates/ai-brains-sources/src/lib.rs:17)

## Requirement and DoD Matrix
**AC1-AC9**
| Item | Status | Notes |
|---|---|---|
| AC1 | Pass | ADR accepted as technical freeze. |
| AC2 | Pass | Threat model includes assets, actors, STRIDE, capability matrix, residuals, WASI risk classes. |
| AC3 | Pass | Built-in inventory covers `mock/obsidian/git/ledgerful/hermes/honcho`. |
| AC4 | Pass | Future subprocess-before-WASI order, OS primitives, two-crate Wasmtime pin, Extism lag, tokio tension documented. |
| AC5 | Pass | License locks and AGPL prohibition documented. |
| AC6 | Pass | Non-claims are explicit. |
| AC7 | Pass | No Cargo manifest/lockfile changes in diff; no `wasmtime`/`wasmtime-wasi`/`extism`/`cap-std` introduced. |
| AC8 | Fail | Still unchecked in [spec.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/spec.md:262). |
| AC9 | Pass | Both soft layers shipped. |

**L1-L10**
| Item | Status | Notes |
|---|---|---|
| L1 | Pass | Production registry still allows only `TrustedBuiltin`. |
| L2 | Pass | No production Wasmtime/Extism/cap-std/WASI host deps added. |
| L3 | Pass | ADR/threat model forbid DLL/native plugin loading and AGPL hosts. |
| L4 | Pass | Policy-not-bypass stance documented. |
| L5 | Pass | `propose_write` remains artifact-only claim. |
| L6 | Pass | `CloudOk` residual is documented honestly. |
| L7 | Pass | Future preference order is documented and justified. |
| L8 | Pass | Future gate conditions are specific and complete. |
| L9 | Pass | Serde fail-closed and registry-deny layers are both implemented. |
| L10 | Pass | Non-claims include TOCTOU residual and no “WASI makes it safe” claim. |

**DoD**
- Fail: the spec requires “full workspace gate green if any code” and “design review clean”; both remain open in the track artifacts ([spec.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/spec.md:280), [plan.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/plan.md:38), [plan.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/plan.md:59)).

## Findings
1. **P2 — Track is not completion-ready under its own AC8/closeout criteria.**  
   AC8 is still unchecked in [spec.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/spec.md:262). The plan still shows `Cross-model (SECURITY) review` pending and `Conductor row → Completed` pending in [plan.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/plan.md:38) and [plan.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/plan.md:57). The conductor entry still says `In Review` in [conductor.md](/C:/dev/AI-Brains/conductor/conductor.md:128). This blocks a completion PASS.

2. **P2 — Required final verification evidence is still open for a code-changing track.**  
   The plan leaves `Full gate only if code changed` unchecked in [plan.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/plan.md:59), and the review log still carries `R1-04` open: “Full workspace gate not re-evidenced” in [review.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/review.md:24). Because this branch changed Rust source files, the DoD line requiring a full workspace gate remains unmet.

## Completeness Sweep
No blocking content gaps beyond the open completion gates.

- ADR status provenance is honest: it says `Accepted` as a technical freeze and explicitly points review evidence to [review.md](/C:/dev/AI-Brains/Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md:5).
- `#12` residual is not falsely closed; ADR and threat model both preserve it as open ([ADR-0019](/C:/dev/AI-Brains/Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md:135), [threat-model.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/threat-model.md:173)).
- No problematic placeholders/stubs/fake production values found. The only placeholder is the explicit `#[cfg(test)] TestUntrustedPlaceholder`, which is correctly test-only ([manifest.rs](/C:/dev/AI-Brains/crates/ai-brains-sources/src/manifest.rs:90)).

## Wiring and Regression Review
The soft test split is correct.

- Layer 1 is serde/parse failure, not registry failure: [manifest.rs](/C:/dev/AI-Brains/crates/ai-brains-sources/src/manifest.rs:237) and tests at [manifest.rs](/C:/dev/AI-Brains/crates/ai-brains-sources/src/manifest.rs:263).
- Layer 2 is registry denial via a test-only construct: [registry.rs](/C:/dev/AI-Brains/crates/ai-brains-sources/src/registry.rs:77) and test at [registry.rs](/C:/dev/AI-Brains/crates/ai-brains-sources/src/registry.rs:210).
- Production `SandboxMode` remains effectively single-variant; the extra variant is gated by `#[cfg(test)]` only ([manifest.rs](/C:/dev/AI-Brains/crates/ai-brains-sources/src/manifest.rs:86)).
- No production subprocess/WASI host wiring was added. No `Cargo.toml` or `Cargo.lock` changes are present in the diff, and repo search found no new `wasmtime`, `wasmtime-wasi`, `extism`, or `cap-std` usage outside the design docs/specs.

## Verification Evidence
Read-only evidence gathered:

- `git diff --name-status origin/main...HEAD`
- `git log --oneline origin/main..HEAD`
- `ai-brains preflight --summary`
- `rg -n "wasmtime|wasmtime-wasi|extism|cap-std|Subprocess|Wasi|LoadLibrary|dlopen|CloudOk|TestUntrustedPlaceholder" ...`
- Line review of [manifest.rs](/C:/dev/AI-Brains/crates/ai-brains-sources/src/manifest.rs:1), [registry.rs](/C:/dev/AI-Brains/crates/ai-brains-sources/src/registry.rs:1), [ADR-0019](/C:/dev/AI-Brains/Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md:1), and [threat-model.md](/C:/dev/AI-Brains/conductor/tracks/trackT182-connector-sandbox-decision/threat-model.md:1).

I did not rerun the full Cargo workspace gate in this review. The repo already records that gate as pending, and this review was constrained to read-only audit work.

## Deferred Candidates
None. The remaining issues are P2 completion blockers, not deferrable P3s.

## Completion Decision
Substantive implementation is in place: ADR-0019, the companion threat model, the two-layer soft tests, the CAPABILITIES cite, and the honesty around residuals all look correct.

The track still fails completion as of August 1, 2026 because its own final gates are not closed: SECURITY cross-model review is still marked pending in the repo, the conductor row remains `In Review`, and the required full workspace gate for this code-changing branch is not re-evidenced. Once those P2 items are closed and recorded, I do not see additional fix-required gaps.