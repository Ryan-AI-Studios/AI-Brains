# T304 Plan — tower-http 0.7

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Implement **CHORE** on go.

## Phase 0

- [ ] Branch `track/T304-tower-http-0-7`
- [ ] Re-read tower-http 0.7.0 release notes + docs.rs `RequestBodyLimitLayer` / `TraceLayer`
- [ ] Confirm axum 0.8 still compatible
- [ ] CHORE TX; do not merge Dependabot remote

## Tasks

- [ ] Workspace `0.7` + `cargo update -p tower-http`
- [ ] Fix compile in api-server if needed (limit/trace only)
- [ ] Stay-green CORS test + clippy/nextest/deny/audit
- [ ] CHANGELOG
- [ ] PR → CI watch → squash (never `git push origin main`)

## DoD

- [ ] tower-http 0.7.x; T161 CORS deny intact
- [ ] Full gate green
