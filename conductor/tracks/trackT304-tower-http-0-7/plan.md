# T304 Plan — tower-http 0.7

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Fold-in DOCS `b24d43a4`. Implement **CHORE** on go.

## Phase 0

- [ ] Branch `track/T304-tower-http-0-7`
- [ ] Re-read https://github.com/tower-rs/tower-http/releases/tag/tower-http-0.7.0 + docs.rs `RequestBodyLimitLayer::new` / `TraceLayer::new_for_http`
- [ ] Confirm axum 0.8 still compatible; `cargo info tower-http` latest 0.7.x
- [ ] Confirm `cargo pkgid tower-http` unique
- [ ] Note: `#58` lock extras (windows-sys / socket2 / windows-core) may **not** match live HEAD after T303 — accept resolver; do not hand-edit (F9)
- [ ] CHORE TX; do **not** merge Dependabot remote `#58`

## Tasks

- [ ] Workspace `tower-http = { version = "0.7", features = ["limit", "cors", "trace"] }` (F1 — **required**; `^0.6.6` cannot reach 0.7)
- [ ] `cargo update -p tower-http --precise 0.7.0` (F8; re-check 0.7.x at execute)
- [ ] Confirm lock **0.7.x**; allow F9 extras; abort if rusqlite / clap / thiserror move or tokio leaves 1.53.1 (F4)
- [ ] Fix compile in api-server if needed (limit/trace only). Do **not** add CorsLayer / CsrfLayer / `fs`
- [ ] `cargo clippy -p ai-brains-api-server --all-targets -- -D warnings` ; `cargo nextest run -p ai-brains-api-server` (CORS `:154` + body-limit `:184`)
- [ ] Full workspace clippy/nextest/deny/audit
- [ ] CHANGELOG Unreleased
- [ ] PR body: 0.7 constructors unchanged; T161 CORS deny; F9 extras expected vs `#58`
- [ ] PR → CI watch → squash (never `git push origin main`)

## DoD

- [ ] workspace `0.7`; lock **0.7.x** (AC1)
- [ ] T161 CORS deny intact (AC3); layers still at `routes.rs:66/:68` (AC4)
- [ ] F9 extras only; tokio stays 1.53.1 (F4)
- [ ] Full gate green (AC5); CHANGELOG (AC6)
