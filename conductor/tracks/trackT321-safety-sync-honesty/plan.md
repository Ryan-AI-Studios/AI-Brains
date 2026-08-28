# T321 Plan — safety sync honesty (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full plan on `/plan-track T321`.
**Ledger (planning):** series DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`

## Phase 0 (on `/plan-track T321`)

- [ ] Re-read `safety.rs` pin path + preflight remediator string
- [ ] Decide: dry-run-by-default (breaking) vs banner-only (safer)

## DoD (after full plan + go)

Default invocation cannot be mistaken for a read. `--dry-run` still previews.

## Isolation

No live hotspot pins as planning. No `cargo install`. Never `git push origin main`.
