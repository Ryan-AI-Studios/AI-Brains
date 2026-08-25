# T304 — tower-http 0.6 → 0.7

- **Track ID:** T304-TowerHttp07
- **Status:** **Planned** (Pending until **go**)
- **Category:** CHORE / DEPS / HTTP
- **Owner:** Grok
- **Source:** Dependabot `#58` tower-http 0.6.11→**0.7.0**. Owner requested 2026-08-25.
- **Depends on:** workspace `tower-http = { version = "0.6.6", features = ["limit", "cors", "trace"] }`; lock **0.6.11**. Live callers: `ai-brains-api-server` `RequestBodyLimitLayer` + `TraceLayer` (`routes.rs`). CORS is a **feature** + `http_cors__default__no_allow_origin_star` test — **no** `CorsLayer` in production routes today (T161 CORS deny).
- **F0:** Plan-only until go. Do **not** merge `dependabot/cargo/tower-http-0.7.0`.
- **Ledger:** series DOCS TX `30b7ca9d-4932-4f00-97b8-82d5d25e633b`.

## 1. Objective

Upgrade tower-http to **0.7.x** with the **same feature set** (`limit`, `cors`, `trace`) unless 0.7 renames a feature. Keep T161 loopback + CORS deny. Fix compile breaks in `routes.rs` / tests only.

## 2. Live baseline (2026-08-25)

| Item | Location |
|------|----------|
| Dep | `Cargo.toml` workspace **0.6.6** features limit/cors/trace; lock **0.6.11** |
| Production | `crates/ai-brains-api-server/src/routes.rs` `:11–12` `RequestBodyLimitLayer` + `TraceLayer` |
| CORS test | `tests/security.rs` `http_cors__default__no_allow_origin_star` `:154` |
| axum | workspace **0.8.9** — confirm 0.7 tower-http still supports axum 0.8 at execute |

**Research (snapshot):** 0.7.0 breaking: ServeDir `Backend` trait; compression `SizeAbove` u16→u64; follow_redirect preserves extensions; **new** CSRF layer (opt-in — **do not add** unless a test requires). We do **not** use ServeDir. Limit/trace likely source-compatible — **prove by compile**. **Re-read tower-http 0.7.0 notes at execute.**

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Target **0.7.0** (or current 0.7.x patch at execute). Workspace pin `0.7`. |
| **F2** | Keep features `limit`, `cors`, `trace`. Do **not** enable `fs` / CSRF unless already used. |
| **F3** | Do not add CorsLayer to production (T161 deny stays). Test AC still no `Access-Control-Allow-Origin: *`. |
| **F4** | No rusqlite / tokio / clap / GHA this track (tokio may already be 1.53 if T303 landed first — do not revert). |
| **F5** | Loopback bind + SDDL / bearer tests stay green. |
| **F6** | Do not merge Dependabot remote. Never `git push origin main`. |
| **F7** | CHANGELOG Unreleased. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | lock tower-http **0.7.x**; workspace `0.7`. |
| **AC2** | `cargo clippy -p ai-brains-api-server --all-targets -- -D warnings` + nextest `-p ai-brains-api-server`. |
| **AC3** | `http_cors__default__no_allow_origin_star` still asserts no `*` origin. |
| **AC4** | `RequestBodyLimitLayer` + `TraceLayer` still wired in `routes.rs`. |
| **AC5** | Workspace clippy/nextest/deny/audit green. |
| **AC6** | CHANGELOG. |

## 5–12

**Non-goals:** CSRF middleware product; ServeDir; axum 0.9; tokio steal (T303).

**Risk:** feature rename / Limit layer type change. Mitigation: compile + existing HTTP tests.

**§9:** Absorb `#58`. Decline T305 rusqlite. last-PR `#216` N/A.

**Touch:** `Cargo.toml` / `Cargo.lock`; maybe `routes.rs` / tests if signatures moved; CHANGELOG; conductor.

**Isolation:** No live daemon HTTP bind as DoD (hermetic `127.0.0.1:0`).
