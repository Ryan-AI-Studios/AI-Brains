# T274 Plan stub — Pins vs harness ingest

**Status:** **Pending** (Placeholder). Full F-list on `/plan-track 274`.
**Spec:** [spec.md](./spec.md)
**Category:** FEATURE / UX
**Ledger:** series DOCS TX `89a8a2b9-d69d-471f-857b-b9e634138499`

- [ ] `/plan-track 274` before implement (F0)
- [ ] Re-dogfood `recall` / `preflight --pretty` / `memory list --limit 5` on live vault
- [ ] Hermetic pin-vs-ingest ranking test (spec Manual DoD)
- [ ] Do not steal T275 grants / T276 leftover / T279 Safety header / T284 retention Work

## DoD (checkable after full plan + go)

- [ ] Unique pin needle is recall hit #1
- [ ] Preflight Index or summary shows a decision/pin, not only session chrome
- [ ] `sync query` vault half does not regress T271 ledger pane
