# T325 Plan — F8 OR-fill PreferRecency (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full plan on `/plan-track T325`.
**Ledger (planning):** minted with T315 DOCS `ca5b1614-6849-416d-ad27-1d44a23198d7`

## Phase 0 (on `/plan-track T325`)

- [ ] Re-read `match_query` AND recency `:197–213` vs F8 `:220–250`
- [ ] Re-read `AuthorityFilter::PreferRecency` SQL `:390–392`
- [ ] Confirm Cursor `#230` still true on that day’s HEAD
- [ ] Hermetic: TAGS flood Prefer LIMIT vs older OR-only pin

## DoD (after full plan + go)

F8 empty Prefer-OR retain recency-retries like AND. T312 F8/F42 stay green. T315 not stolen.

## Isolation

Do not implement from T315. No `cargo install`. Never `git push origin main`.
