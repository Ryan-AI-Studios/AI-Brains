# T316 Plan — memory list preview (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full plan on `/plan-track T316`.
**Ledger (planning):** series DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`

## Phase 0 (on `/plan-track T316`)

- [ ] Re-read `preview_line` / F36 / T287 inherit callers
- [ ] Dogfood `memory list --limit 5 --format human`

## DoD (after full plan + go)

Default human list does not look failed. Previews are not raw “Let me verify…” when a better line exists.

## Isolation

Do not reopen JSON ORDER. No `cargo install`. Never `git push origin main`.
