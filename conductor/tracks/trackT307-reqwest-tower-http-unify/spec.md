# T307 — Unify tower-http (drop reqwest 0.6.11 dual)

- **Track ID:** T307-ReqwestTowerHttpUnify
- **Status:** **Planned** (Pending until **go**; **upstream-blocked** today)
- **Category:** CHORE / DEPS
- **Owner:** Grok
- **Source:** T304 R2 — api-server `tower-http` **0.7.0**; reqwest keeps **0.6.11**.
- **Depends on:** T304 `#221`. Workspace `reqwest = { version = "0.13", features = ["json"] }`.
- **F0:** Plan-only until go. Do **not** `[patch.crates-io]`. Do **not** add new crates.

## 1. Objective

Single `tower-http` line in `Cargo.lock` at **0.7.x** (api-server + reqwest). Keep T161 CORS deny. Same reqwest features (`json`).

## 2. Live baseline (2026-08-26)

| Pin | Workspace | Lock | crates.io (mint day) |
|-----|-----------|------|----------------------|
| reqwest | **0.13** json (`Cargo.toml:50`) | **0.13.4** | **0.13.4** latest |
| tower-http | **0.7** limit/cors/trace | **0.7.0** *and* **0.6.11** | 0.7.0 latest |

**reqwest 0.13.4** `Cargo.toml` pins `tower-http = { version = "0.6.8", … follow-redirect }` → lock **0.6.11**. `cargo update -p tower-http --precise 0.7.0` **cannot** unify (T304 residual: reqwest `^0.6.8`).

**Research (snapshot — re-verify at execute):** [reqwest master](https://github.com/seanmonstar/reqwest/blob/master/Cargo.toml) still `tower-http 0.6.8` at 0.13.4. No crates.io reqwest release accepts 0.7. **Stop-Before** if still true on go.

last-PR `#222` Cursor **empty**. **No T310.**

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | One tower-http **0.7.x** in lock. Dual 0.6.11 gone. |
| **F2** | Keep reqwest `json` only. No gzip/brotli steal unless already on. |
| **F3** | **Stop-Before** if latest reqwest still requires tower-http 0.6. Leave dual; do not patch. |
| **F4** | No new crates (`tower-reqwest` AGPL-risk / extra). |
| **F5** | T161 CORS deny; no CorsLayer. No rusqlite/clap/tokio steal. |
| **F6** | Do not merge Dependabot remotes. Never `git push origin main`. |
| **F7** | CHANGELOG Unreleased if a bump ships. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | `Cargo.lock` has tower-http **0.7.x only** (or track **halted** with F3 evidence: crates.io reqwest still 0.6). |
| **AC2** | `cargo tree -i tower-http@0.6.11` empty **or** F3 halt documented. |
| **AC3** | `http_cors__default__no_allow_origin_star` green if a bump ships. |
| **AC4** | clippy/nextest/deny/audit green if a bump ships. |

## 5–12

**Non-goals:** csrf; clap 5; forcing 0.7 via patch; T306 install.

**§9:** Absorb T304 R2. Decline T304 R4 csrf. Dual thiserror 1.x (T302 R2) **not** this track. last-PR `#222` N/A.

**Touch:** `Cargo.toml` reqwest pin **only if** a compatible reqwest exists; `Cargo.lock`; CHANGELOG.

**Isolation:** No live HTTP bind as DoD (`127.0.0.1:0` tests).
