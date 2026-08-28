# T324 Plan — PowerShell empty TERM (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full plan on `/plan-track T324`.
**Ledger (planning):** series DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`

## Phase 0 (on `/plan-track T324`)

- [ ] Reproduce `ai-brains decision in-force ""` vs `decision in-force --term ""` on this PowerShell
- [ ] Re-read T311 fail_usage tests
- [ ] Pick `--term` vs `--%` docs-only (prefer a flag if docs-only fails IA)

## DoD (after full plan + go)

A documented PowerShell invocation reaches empty-term usage (exit 2), not “missing argument.”

## Isolation

No clap 5. No `cargo install`. Never `git push origin main`.
