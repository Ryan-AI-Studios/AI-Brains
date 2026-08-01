# T182 Review Log — Connector Sandbox Decision (P12.4)

- **Track:** T182-ConnectorSandboxDecision
- **Category:** SECURITY / ARCHITECTURE
- **Status:** implementation complete; internal review pending
- **Date opened:** 2026-08-01
- **Owner (implement):** Grok

## Scope reviewed

- ADR-0019 Accepted: `Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md`
- Companion threat model: `threat-model.md`
- Soft two-layer tests in `ai-brains-sources` (`manifest.rs` unit tests + `registry.rs` unit tests)
- CAPABILITIES one-line connector-trust cite
- Conductor/deferred closeout

## Findings

| ID | Severity | Source | Summary | Status | Notes |
|----|----------|--------|---------|--------|-------|
| *(none yet)* | | | | | |

## Review rounds

| Round | Reviewer | Outcome | Date |
|-------|----------|---------|------|
| Internal R1 | — | pending | |
| Cross-model (SECURITY) | — | pending | |

## Closure criteria

- Critical/High → `verified_fixed` before clearance
- Medium → fix by default; ≤3 deferred with ISSUES/deferred note
- Low-info → defer freely with ISSUES append

## Implementation notes (for reviewers)

- Production `SandboxMode` remains single-variant (`TrustedBuiltin`); `TestUntrustedPlaceholder` is `#[cfg(test)]` only
- Layer 1: unknown JSON sandbox → `ManifestError::Json` (not `SandboxNotAllowed`)
- Layer 2: registry `register` with test-only mode → `RegistryError::SandboxNotAllowed`
- No wasmtime/extism/cap-std deps added
