# T308 Plan — sparse remediator honesty

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `c62396f6`. Implement **FEATURE** on go.

## Phase 0

- [ ] Re-read `graph_density.rs` Sparse vs `density_remediation`
- [ ] Confirm floors still 0.50
- [ ] FEATURE TX; no live rebuild in Phase 0

## Tasks

- [ ] Red: Sparse assessment `remediation` is not `ai-brains graph rebuild`
- [ ] Green: Sparse remediator copy (honest sparse / floors frozen); other warn arms unchanged
- [ ] Keep AC2 orphan/empty_lag/projection_lag rebuild
- [ ] CHANGELOG
- [ ] Optional live `doctor --summary` on current vault (no rebuild)
- [ ] PR → CI → squash (never `git push origin main`)

## DoD

- [ ] AC1–AC5; floors unchanged
- [ ] No projector rewrite; no live rebuild as required DoD
