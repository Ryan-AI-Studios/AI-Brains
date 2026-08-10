# T222 Review Log — Graph-on install path

## Scope

- **Track:** T222-GraphOnInstallPath
- **Category:** INFRA
- **A2:** no (Cargo `default = []` unchanged; release.yml not flipped)
- **Ledger TX:** `c0316e6e-7115-4ef9-8507-0d614edeb6bf`

## Implemented

1. `GRAPH_REINSTALL_SOOT` in `governed_common.rs` — single SOOT for stubs, doctor remediation, empty-lag install substring
2. Doctor soft check `graph_feature` (`available`|`unavailable`, Ok severity); matrix **12→13** order F10
3. Scripts `Build-AIBrains.ps1` / `build.ps1`: `--features` graph + F7 fail-closed probe (known-missing vault; doctor JSON primary; stub secondary)
4. Docs: CAPABILITIES **11→13** (+ harness_wiring + graph_feature); INSTALL/OPERATIONS/CONTRIBUTING/CHANGELOG honesty
5. Density remediation **branching** left for T232

## Internal review (subagent)

**Verdict: CLEAN** (no critical/high/medium AC failures)

### Easy P3s addressed before Codex

| ID | Issue | Disposition |
|----|-------|-------------|
| P3-1 | Probe `2>&1` can break doctor JSON parse | **Fixed** — stdout only + `--log-format off` |
| P3-2 | Secondary `graph update` may create TEMP vault | **Fixed** — cleanup probe path + wal/shm in `finally` |
| P3-3 | Multi-package `--features graph` clarity | **Fixed** — `ai-brains-cli/graph` on Build-AIBrains |
| P3-4 | Hardcoded cargo bin paths | **Deferred** — pre-existing; not introduced as T222 logic |
| P3-5 | Matrix name unit docs-only | **Deferred soft** — live order covered in `doctor__graph_density_present__open_failed_is_skip` |

### Residual soft (not blocking)

- build.ps1 vs Build-AIBrains deprecation fold (F18 / L1)
- Human checks=N unit assert (L3)
- MSI / dual artifact / release graph-on / Cargo default flip
- T232 density remediation rebuild vs reinstall

## Codex cross-model

| Round | Verdict | Notes |
|-------|---------|-------|
| R1 | FAIL / NOT CLEAR | P1 unique probe path; P1 process AC13/AC14 |
| R2 | PASS WITH DEFERRED P3 | P1-1 verified fixed; AC13/AC14 process-only |
| Final | **PASS** | Fresh product-only gate; no blocking findings |

### R1 dispositions

| Finding | Disposition |
|---------|-------------|
| P1 unique fixed TEMP path | **verified_fixed** — GUID-owned probe dir + assert absent + owned cleanup |
| P1 AC13/AC14 process | **verified_fixed** after full gate + AC14 manual |

## Gates (AC13)

| Check | Result |
|-------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo nextest run --workspace` | **2491 passed**, 1 skipped |
| `cargo deny check` | PASS |
| `cargo audit` | PASS (19 allowed warnings) |
| `ledgerful verify --scope fast` | PASS |

## Manual AC14

```text
scripts/build.ps1 → exit 0
  "Graph feature probe: available"
PATH doctor --json → graph_feature message=available severity=ok
PATH graph update → density JSON (not FEATURE_UNAVAILABLE / not exit 2)
```

## Status

- Internal: CLEAN
- Cross-model final: **PASS**
- Track: ready for PR / ship
