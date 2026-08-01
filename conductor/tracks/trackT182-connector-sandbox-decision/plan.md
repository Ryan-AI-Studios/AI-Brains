# T182 Plan — Connector Sandbox Decision + Threat Model (P12.4)

Status: **In Review** (2026-08-01) — ADR-0019 Accepted technical freeze + soft two-layer tests shipped. Design-review rounds open.

## Track shape

| Phase | Work | Code? |
|-------|------|-------|
| **A** | Inventory + research freeze + AI fold-in | No |
| **B** | Threat model finalize + ADR-0019 draft → Accept | Docs |
| **C** | Soft: two-layer sandbox tests (optional) | Tiny test-only |
| **D** | Conductor/deferred/closeout; cite from T183–T185 | Docs |

## Ledgerful preflight (expansion day)

- [x] `ledgerful doctor` — ready (3 warnings: legacy `.changeguard`, sig-pin, sig-version)
- [x] `ledgerful ledger status --compact` — 0 pending, 0 unaudited drift
- [x] `ledgerful search "SandboxMode"` — registry + manifest + all built-ins
- [x] `ledgerful hotspots` — sources not in top-10; CLI sync/context dominate
- [x] `ledgerful scan --impact` — clean tree refresh OK (note: branch may differ at implement)
- [ ] At implement start: `ledgerful ledger start T182-connector-sandbox --category SECURITY --message "ADR-0019 connector sandbox decision"`
- [ ] At close: `ledgerful ledger commit` + pin decision

## Phase A — Expansion (complete)

- [x] Inventory P6 connectors (`builtin.mock|obsidian|git|ledgerful|hermes|honcho`)
- [x] Live `SandboxMode` / registry refuse baseline
- [x] Online research: Wasmtime (Apr 2026 batch + May/Jun WASI-host advisories), Extism pin lag, cap-std triple license, Pulley, tokio/sync, deny.toml
- [x] Roll deferred: R1-06, #12 residual honesty, T154 cap-std adjacent, vision §7.2
- [x] Write expanded `spec.md`, `plan.md`, `threat-model.md`, `adr-0019-draft.md`
- [x] Update `conductor.md` + `deferred.md` §60
- [x] **AI1/AI2 fold-in** → spec §15 disposition (A1–A13); amend threat-model + ADR

## Phase B — ADR accept (implement on go-ahead)

- [x] Human / orchestrator review of threat model + ADR draft (post-fold-in) — implement phase
- [x] Internal review log → resolve findings — R1 FAIL (R1-01 provenance) → fixed; R2 pending re-verify
- [ ] Cross-model (SECURITY) review until clean or justified deferrals
- [x] Promote `adr-0019-draft.md` → `Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md` with **Accepted** (technical freeze; review provenance in `review.md`)
- [x] Cross-link from CAPABILITIES one-line “connector trust” pointer (T183 owns full pack)
- [ ] Optional pin: `ai-brains pin "DECISION: ADR-0019 v1 TrustedBuiltin only; third-party subprocess then WASI; no DLL load; no AGPL host; two-crate wasmtime pin if ever"`

## Phase C — Soft code (optional)

Two-layer defense (L9) — **do not** assert `SandboxNotAllowed` on unknown-string JSON (that fails at serde):

- [x] **Layer 1 (cheap):** unit test — deserialize manifest with `"sandbox":"Subprocess"` (or `UntrustedExternal`) → serde/manifest error
- [x] **Layer 2 (R1-06):** decide ship `#[cfg(test)]` constructible non-`TrustedBuiltin` → `register` returns `SandboxNotAllowed`, **or** re-defer as info with L9 citation
- [x] If ship: no production/serde variants that imply a host exists
- [x] **Decline DoD:** Cargo.lock walk forbidding wasmtime/extism/cap-std (deny/audit suffice)
- [x] `cargo nextest run -p ai-brains-sources` + clippy package
- [x] Confirm **zero** new prod deps; deny/audit green

## Phase D — Closeout

- [x] AC1–AC7 + AC9 checked in `spec.md`; AC8 pending design-review clean
- [ ] Conductor row → Completed after design review clean; owner filled (currently In Review)
- [x] deferred.md §60 struck/promoted notes
- [ ] Full gate only if code changed:  
  `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit ; ledgerful verify --scope full`
- [ ] Design-only closeout: document review evidence; no binary change claim  
  *(Package-level fmt/clippy/nextest green for sources; full workspace gate + reviews = orchestrator)*

## License gate (always)

- [x] Wasmtime / wasmtime-wasi: Apache-2.0 WITH LLVM-exception + two-crate patch discipline
- [x] Extism: BSD-3 future-only + pin-lag honesty
- [x] cap-std: triple-license precision
- [x] No AGPL host
- [x] No unknown-git runtimes
- [x] deny advisories: unsound + unmaintained workspace documented

## Stop-before checklist

- [ ] No Wasmtime/Extism/cap-std Cargo.toml add without new track
- [ ] No production `SandboxMode` variants that imply a host exists
- [ ] No “sandboxed plugins available” / “WASI isolation” marketing language
- [ ] No claim that #12 TOCTOU is closed
- [ ] No claim that WASI FilePerms always hold
