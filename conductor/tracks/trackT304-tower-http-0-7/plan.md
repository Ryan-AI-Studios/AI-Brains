# T304 Plan — tower-http 0.7

**Status:** **Completed**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Fold-in DOCS `b24d43a4`. Implement **CHORE** `f1edfb28-9f51-4910-be4c-14bef88fe09e`.

## Phase 0

- [x] Branch `track/T304-tower-http-0-7`
- [x] Re-read https://github.com/tower-rs/tower-http/releases/tag/tower-http-0.7.0 + docs.rs `RequestBodyLimitLayer::new` / `TraceLayer::new_for_http`
- [x] Confirm axum 0.8 still compatible; `cargo info tower-http` latest 0.7.x → **0.7.0**
- [x] Confirm `cargo pkgid` — after bump: dual `0.7.0` + `0.6.11` (reqwest)
- [x] Note: `#58` lock extras (windows-sys / socket2 / windows-core) may **not** match live HEAD after T303 — accept resolver; do not hand-edit (F9). Live: dual package only, no windows-sys churn.
- [x] CHORE TX; do **not** merge Dependabot remote `#58`

## Tasks

- [x] Workspace `tower-http = { version = "0.7", features = ["limit", "cors", "trace"] }` (F1 — **required**; `^0.6.6` cannot reach 0.7)
- [x] F8 intent: `--precise 0.7.0` **fails** (reqwest `^0.6.8`); resolver adds **0.7.0** beside **0.6.11** — correct dual outcome
- [x] Confirm lock **0.7.x** for api-server; F9 extras minimal; abort if rusqlite / clap / thiserror move or tokio leaves 1.53.1 (F4) — peers unchanged
- [x] Fix compile in api-server if needed (limit/trace only). Do **not** add CorsLayer / CsrfLayer / `fs` — **no src change**
- [x] `cargo clippy -p ai-brains-api-server --all-targets -- -D warnings` ; `cargo nextest run -p ai-brains-api-server` (CORS `:154` + body-limit `:184`) — **39 passed**
- [x] Full workspace clippy/nextest/deny/audit
- [x] CHANGELOG Unreleased
- [x] PR body: 0.7 constructors unchanged; T161 CORS deny; dual 0.6.11 via reqwest; F9 extras expected vs `#58`
- [ ] PR → CI watch → squash (never `git push origin main`)

## DoD

- [x] workspace `0.7`; lock **0.7.x** for api-server (AC1); dual 0.6.11 via reqwest documented
- [x] T161 CORS deny intact (AC3); layers still at `routes.rs:66/:68` (AC4)
- [x] F9 extras only; tokio stays 1.53.1 (F4)
- [x] Full gate green (AC5); CHANGELOG (AC6)

## Evidence (targeted)

```
cargo clippy -p ai-brains-api-server --all-targets -- -D warnings  # exit 0
cargo nextest run -p ai-brains-api-server  # 39 passed incl. http_cors__default__no_allow_origin_star + http_body__over_limit__413
```
