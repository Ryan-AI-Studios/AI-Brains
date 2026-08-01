# T182 Review Log — Connector Sandbox Decision (P12.4)

- **Track:** T182-ConnectorSandboxDecision
- **Category:** SECURITY / ARCHITECTURE
- **Status:** Internal R2 **PASS**; SECURITY cross-model pending
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
| R1-02 | low | Internal R1 | Conductor/plan Completed ahead of design-review DoD | `verified_fixed` | R2: In Review across conductor/spec/plan/deferred |
| R1-03 | info | Internal R1 | ADR lacked explicit L1–L10 labels | `verified_fixed` | R2: L1–L10 table in ADR Decision |
| R1-04 | info | Internal R1 | Full workspace gate not re-evidenced in R1 | open | Orchestrator runs full gate before merge |

## Review rounds

| Round | Reviewer | Outcome | Date |
|-------|----------|---------|------|
| Internal R1 | explore subagent | **FAIL** (R1-01 medium) | 2026-08-01 |
| Internal R2 | explore subagent | **PASS** (R1-01/02/03 verified_fixed) | 2026-08-01 |
| Cross-model (SECURITY) | Codex | pending | |

## Closure criteria

- Critical/High → `verified_fixed` before clearance
- Medium → fix by default; ≤3 deferred with ISSUES/deferred note
- Low-info → defer freely with ISSUES append

## Implementation notes (for reviewers)

- Production `SandboxMode` remains single-variant (`TrustedBuiltin`); `TestUntrustedPlaceholder` is `#[cfg(test)]` only
- Layer 1: unknown JSON sandbox → `ManifestError::Json` (not `SandboxNotAllowed`)
- Layer 2: registry `register` with test-only mode → `RegistryError::SandboxNotAllowed`
- No wasmtime/extism/cap-std deps added
