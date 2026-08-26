# T304 Review Log — tower-http 0.7

**Track:** T304-TowerHttp07  
**Category:** CHORE / DEPS / HTTP  
**CHORE TX:** `f1edfb28-9f51-4910-be4c-14bef88fe09e`  
**Branch:** `track/T304-tower-http-0-7`  
**Date:** 2026-08-25

## Scope

Dependabot `#58` tower-http **0.6.11 → 0.7.0**. Workspace pin **`0.6.6` → `0.7`** (F1 caret-unblock; features `limit`/`cors`/`trace`). Keep T161 CORS deny (no `CorsLayer`). Do not merge Dependabot remote. No rusqlite / clap / thiserror / tokio bump (F4).

## Pin resolution (execute 2026-08-25)

| Pin | Before | After | crates.io | Notes |
|-----|--------|-------|-----------|-------|
| workspace tower-http | `0.6.6` limit/cors/trace | **`0.7`** same features | — | F1 required (`^0.6.6` cannot reach 0.7) |
| lock tower-http (api-server) | 0.6.11 only | **0.7.0** + **0.6.11** dual | **0.7.0** latest | api-server → 0.7.0; reqwest → 0.6.11 |
| tokio | 1.53.1 | **1.53.1** | — | F4 |
| rusqlite | 0.39.0 | **0.39.0** | — | F4 / decline T305 |
| clap | 4.6.1 | **4.6.1** | — | F4 |
| thiserror | 1.0.69 + 2.0.20 | same | — | F4 |
| F9 extras | windows-sys / socket2 / windows-core possible | **minimal** — dual package entry only | — | Live graph after T303; no windows-sys churn |

`cargo pkgid tower-http` ambiguous → `tower-http@0.7.0` + `tower-http@0.6.11`.  
`cargo update -p tower-http --precise 0.7.0` **fails** (reqwest `^0.6.8`). Resolution: workspace pin + resolver adds 0.7.0 alongside 0.6.11 (F8 hygiene intent met; unify impossible).  
`git diff -- crates/` empty (constructors unchanged on docs.rs 0.7.0).

## DoD / AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 workspace `0.7` + lock 0.7.x | **Met** | `Cargo.toml:83`; lock `tower-http@0.7.0` (api-server) |
| AC2 targeted clippy/nextest | **Met** | `cargo clippy -p ai-brains-api-server --all-targets -- -D warnings` exit 0; `cargo nextest run -p ai-brains-api-server` **39 passed** |
| AC3 CORS deny | **Met** | `http_cors__default__no_allow_origin_star` PASS |
| AC4 layers wired | **Met** | `routes.rs:66` `RequestBodyLimitLayer::new`; `:68` `TraceLayer::new_for_http` unchanged |
| AC5 full gate | **Met** | `.\scripts\dev-check.ps1` + `ledgerful verify --scope full` **exit 0** (3529 passed / 1 skipped) |
| AC6 CHANGELOG | **Met** | Unreleased Changed row for T304 |

Body-limit regression: `http_body__over_limit__413` PASS (same nextest run).

## Internal findings

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| R1 | low-info | Dependabot `#58` still OPEN — close as superseded after squash; do not delete remote | **deferred** — F6 / standing hygiene |
| R2 | low-info | Dual tower-http **0.7.0** (api-server) + **0.6.11** (reqwest `^0.6.8`) — bare `--precise 0.7.0` cannot unify | **deferred** — not easy; needs reqwest that accepts 0.7 |
| R3 | low-info | F9 `#58` windows-sys/socket2/windows-core extras did **not** appear on live HEAD after T303 — lock delta is dual package only | **deferred** — expected variance; do not hand-edit |
| R4 | low-info | Opt-in `csrf` feature exists in 0.7 — intentionally not enabled (F2 / F3) | **deferred** — product non-goal; monitor if future HTTP CSRF track |

No critical / high / medium. Easy lows closed by pin + targeted nextest (CORS + body limit).

## Cross-model

**Skipped** — track category is **CHORE / DEPS**, not FEATURE / SECURITY / ARCHITECTURE (implement-track Phase 3 gate). Internal DoD matrix covers all ACs. T161 CORS deny reaffirmed by existing security test.

## Gates

- Targeted AC2/AC3/AC4: **exit 0** (2026-08-25; 39 passed).
- Full `dev-check.ps1` + `ledgerful verify --scope full`: **exit 0** (2026-08-25; nextest 3529 pass / 1 skipped; deny + audit green with allowed warnings only).
- Publish GHA: Phase 6.
