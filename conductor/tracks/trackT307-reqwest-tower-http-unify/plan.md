# T307 Plan — reqwest / tower-http unify

**Status:** **Pending**. Spec [spec.md](./spec.md). **Upstream-blocked** until reqwest allows tower-http 0.7.
**Ledger:** series DOCS `c62396f6`. Implement **CHORE** on go.

## Phase 0

- [ ] `cargo info reqwest` — latest still 0.13.4? tower-http dep range
- [ ] If still `tower-http 0.6.x` only → **Stop-Before F3** (document; do not patch)
- [ ] CHORE TX; do not merge Dependabot remotes

## Tasks (only if F3 does not halt)

- [ ] Bump workspace reqwest to the line that allows tower-http 0.7
- [ ] Confirm lock has tower-http 0.7.x **only**
- [ ] clippy + nextest `-p ai-brains-api-server` CORS + workspace gate
- [ ] CHANGELOG
- [ ] PR → CI → squash (never `git push origin main`)

## DoD

- [ ] Dual gone **or** F3 halt with crates.io evidence
- [ ] CORS deny intact if a bump ships
