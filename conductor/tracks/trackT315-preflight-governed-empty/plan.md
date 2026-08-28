# T315 Plan — preflight governed-empty (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full plan on `/plan-track T315`.
**Ledger (planning):** series DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`

## Phase 0 (on `/plan-track T315`)

- [ ] Re-read summary renderer + T220/T265 JSON envelopes
- [ ] Confirm in-context decision_count source (vault GLOB vs governed projection)
- [ ] Pick exact next-step string (must not imply H2)

## DoD (after full plan + go)

Summary with 0 decisions is not a dead 0/0/0. Word-count label is unambiguous. Pins remain un-promoted.

## Isolation

Do not grow `preflight.rs` beyond summary. No H2. No `cargo install`. Never `git push origin main`.
