# T182 Review Log — Connector Sandbox Decision (P12.4)

- **Track:** T182-ConnectorSandboxDecision
- **Category:** SECURITY / ARCHITECTURE
- **Status:** design clean; full gate green; final Codex R2 pending
- **Date opened:** 2026-08-01
- **Owner (implement):** Grok

## Scope reviewed

- ADR-0019 Accepted (technical freeze): `Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md`
- Companion threat model: `threat-model.md`
- Soft two-layer tests in `ai-brains-sources` (`manifest.rs` unit tests + `registry.rs` unit tests)
- CAPABILITIES one-line connector-trust cite
- Conductor/deferred closeout

## Findings

| ID | Severity | Source | Summary | Status | Notes |
|----|----------|--------|---------|--------|-------|
| R1-01 | medium | Internal R1 | ADR Status falsely claimed internal+SECURITY cross-model already complete | `verified_fixed` | R2: Status = technical freeze only; design-review provenance in review.md |
| R1-02 | low | Internal R1 | Conductor/plan Completed ahead of design-review DoD | `verified_fixed` | R2: In Review then Completed after gates |
| R1-03 | info | Internal R1 | ADR lacked explicit L1–L10 labels | `verified_fixed` | R2: L1–L10 table in ADR Decision |
| R1-04 | info | Internal R1 | Full workspace gate not re-evidenced in R1 | `verified_fixed` | Full gate 2026-08-01: fmt/clippy green; nextest 1708 passed 1 skipped; deny ok; audit warnings only (pre-existing) |
| CX1 | P2 | Codex R1 | AC8/conductor still open at review time | `verified_fixed` | Closeout after Codex R1 design PASS + full gate; no design defect |
| CX2 | P2 | Codex R1 | Full workspace gate not re-evidenced | `verified_fixed` | Same as R1-04 evidence |

## Review rounds

| Round | Reviewer | Outcome | Date |
|-------|----------|---------|------|
| Internal R1 | explore subagent | **FAIL** (R1-01 medium) | 2026-08-01 |
| Internal R2 | explore subagent | **PASS** (R1-01/02/03 verified_fixed) | 2026-08-01 |
| Cross-model R1 | Codex gpt-5.4 high | **FAIL** (CX1/CX2 closeout gates only; **zero design P0–P2**) | 2026-08-01 |
| Cross-model R2 | Codex | pending final fresh gate after closeout | |

## Codex R1 design summary (no design fixes required)

- AC1–AC7, AC9, L1–L10: **Pass**
- Soft-test BS1 split correct; no wasmtime/extism/cap-std; non-claims/#12 honest
- Only blockers were process closeout (AC8 + full gate evidence)

## Full gate evidence (2026-08-01)

```text
cargo fmt --check                          OK
cargo clippy --workspace --all-targets -D warnings  OK
cargo nextest run --workspace --no-fail-fast
  Summary: 1708 tests run: 1708 passed, 1 skipped
  (earlier fail-fast hit flaky ai-brains-brain::nightly_consecutive_errors
   QueryReturnedNoRows — re-ran alone PASS; unrelated to T182)
cargo deny check                           advisories/bans/licenses/sources ok
cargo audit                                exit 0; allowed warnings only (pre-existing)
```

Manual: production `SandboxMode` single-variant; `TestUntrustedPlaceholder` is `#[cfg(test)]` only; `Cargo.lock` has no wasmtime/extism/cap-std.

## Closure criteria

- Critical/High → `verified_fixed` before clearance
- Medium → fix by default; ≤3 deferred with ISSUES/deferred note
- Low-info → defer freely with ISSUES append

## Implementation notes (for reviewers)

- Production `SandboxMode` remains single-variant (`TrustedBuiltin`); `TestUntrustedPlaceholder` is `#[cfg(test)]` only
- Layer 1: unknown JSON sandbox → `ManifestError::Json` (not `SandboxNotAllowed`)
- Layer 2: registry `register` with test-only mode → `RegistryError::SandboxNotAllowed`
- No wasmtime/extism/cap-std deps added
