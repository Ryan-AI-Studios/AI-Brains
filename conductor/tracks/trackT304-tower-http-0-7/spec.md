# T304 — tower-http 0.6 → 0.7

- **Track ID:** T304-TowerHttp07
- **Status:** **Planned** (Pending until **go**)
- **Category:** CHORE / DEPS / HTTP
- **Owner:** Grok
- **Source:** Dependabot `#58` tower-http 0.6.11→**0.7.0**. Owner requested 2026-08-25.
- **Depends on:** workspace `tower-http = { version = "0.6.6", features = ["limit", "cors", "trace"] }` (`Cargo.toml:83`). `"0.6.6"` is Cargo default **`>=0.6.6, <0.7.0`** — **cannot** resolve 0.7.0 without a manifest edit. Lock **0.6.11**. Live callers: `ai-brains-api-server` `RequestBodyLimitLayer` + `TraceLayer` (`routes.rs:66/:68`). CORS is a **feature** + `http_cors__default__no_allow_origin_star` — **no** `CorsLayer` in production (T161 CORS deny).
- **F0:** Plan-only until go. Do **not** merge `dependabot/cargo/tower-http-0.7.0`.
- **Ledger:** series DOCS TX `30b7ca9d-4932-4f00-97b8-82d5d25e633b`. Fold-in DOCS TX `b24d43a4-7258-4f28-a1a6-63883884b64e`. Implement starts **CHORE** TX on go.
- **AI fold-in:** 2026-08-25 `agy-review.md` + `opencode-review.md` (HEAD `bdd34f9`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy m1 caret **must** widen (unlike T303); Agy m3 `--precise 0.7.0`; OpenCode m1 last-PR `#220`; OpenCode O1 `#58` lock extras may differ after T303. **Already:** Agy m2 = F2; Agy O1 = AC2/AC3. **Partial:** OpenCode “limit/cors/trace have no breaking API” — constructors unchanged; gRPC `classify` `#[non_exhaustive]` unused. Disposition **§13**.

## 1. Objective

Upgrade tower-http to **0.7.x** with the **same feature set** (`limit`, `cors`, `trace`) unless 0.7 renames a feature. Keep T161 loopback + CORS deny. Fix compile breaks in `routes.rs` / tests only.

## 2. Live baseline (2026-08-25 fold-in)

| Item | Location |
|------|----------|
| Dep | `Cargo.toml:83` workspace **0.6.6** features limit/cors/trace; lock **0.6.11** (unique pkgid) |
| Production | `crates/ai-brains-api-server/src/routes.rs` imports `:11–12`; **wiring** `RequestBodyLimitLayer::new(BODY_LIMIT_BYTES)` **`:66`** (`BODY_LIMIT_BYTES = 1 MiB` `:29`) + `TraceLayer::new_for_http()` **`:68`**. No `CorsLayer` / `ServeDir` / `CsrfLayer` in tree. |
| CORS test | `tests/security.rs` `http_cors__default__no_allow_origin_star` `:154` — ACAO **absent** (also asserts not `*`). Body limit: `http_body__over_limit__413` `:184`. |
| axum | workspace **0.8.9** — tower-http 0.7.0 deps `http ^1` / `http-body ^1` / `tower ^0.5` — **compatible**. Re-confirm at execute. |
| tokio | lock **1.53.1** after T303 `#220`. Do **not** revert (F4). |

**`cargo pkgid tower-http`** → `tower-http@0.6.11` only. `--precise` is hygiene (T302/T303 F8).

**Research (verified fold-in; re-read [tower-http 0.7.0](https://github.com/tower-rs/tower-http/releases/tag/tower-http-0.7.0) at execute):**

- crates.io latest **0.7.0** (2026-06-15). MSRV **1.65**; repo rustc **1.95.0**. MIT.
- [docs.rs 0.7.0](https://docs.rs/tower-http/0.7.0/tower_http/limit/struct.RequestBodyLimitLayer.html) `RequestBodyLimitLayer::new(usize)` unchanged. [TraceLayer::new_for_http()](https://docs.rs/tower-http/0.7.0/tower_http/trace/struct.TraceLayer.html#method.new_for_http) unchanged.
- Breaking in 0.7.0 **outside our features/usage:** ServeDir `Backend`; compression `SizeAbove` u16→u64 + Accept-Encoding 406; follow_redirect extensions; trailing-slash 404 for files; remove no-op `tokio`/`async-compression` features. **New opt-in** CSRF (`#699`) — **do not enable**. `limit` only additive `Default` for `ResponseBody`. `cors` relaxes Vary defaults — unused without `CorsLayer`. `trace`/`classify` gRPC types `#[non_exhaustive]` — we use HTTP classifier only.
- `#58` is `Cargo.toml` 1/1 + `Cargo.lock` 39/23. Lock extras on that branch: `windows-sys` edges toward 0.61.2, `socket2 0.5.10→0.6.3`, `windows-core 0.61.2→0.62.2`. **HEAD after T303 already has** socket2 **0.5.10 and 0.6.3**, tokio **1.53.1**. Live `cargo update` **will not match `#58` byte-for-byte**. Accept resolver output; do not hand-revert. Abort only if rusqlite / clap / thiserror move or tokio leaves 1.53.1.

last-PR Cursor: **`#220`** (T303, HEAD `bdd34f9`) — comments/reviews **empty**. **No T306.**

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Target **0.7.0** (or current 0.7.x patch at execute). Workspace pin **`0.7`** — **required** (`^0.6.6` cannot reach 0.7). |
| **F2** | Keep features `limit`, `cors`, `trace`. Do **not** enable `fs` / `csrf` / compression. |
| **F3** | Do not add CorsLayer (or CsrfLayer) to production. Test AC still no `Access-Control-Allow-Origin: *` and header **absent**. |
| **F4** | No rusqlite / clap / GHA this track. Tokio is **1.53.1** — do not revert. |
| **F5** | Loopback bind + SDDL / bearer tests stay green. |
| **F6** | Do not merge Dependabot remote. Never `git push origin main`. |
| **F7** | CHANGELOG Unreleased. |
| **F8** | `cargo update -p tower-http --precise 0.7.0` ([cargo-update `--precise`](https://doc.rust-lang.org/cargo/commands/cargo-update.html)). Re-check latest 0.7.x at execute. |
| **F9** | Expected lock extras from graph re-resolution (windows-sys / socket2 / windows-core). Live diff may differ from `#58` after T303. Do not hand-edit. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | lock tower-http **0.7.x**; workspace `0.7`. |
| **AC2** | `cargo clippy -p ai-brains-api-server --all-targets -- -D warnings` + `cargo nextest run -p ai-brains-api-server`. |
| **AC3** | `http_cors__default__no_allow_origin_star` still asserts ACAO absent / not `*`. |
| **AC4** | `RequestBodyLimitLayer` + `TraceLayer` still wired in `routes.rs` (`:66` / `:68`). |
| **AC5** | Workspace clippy/nextest/deny/audit green. |
| **AC6** | CHANGELOG. |

## 5–12

**Non-goals:** CSRF middleware product; ServeDir; axum 0.9; rusqlite (T305); tokio steal/revert.

**Risk:** feature rename / Limit layer type change. Mitigation: compile + existing HTTP tests (`http_body__over_limit__413`). Fold-in: constructors unchanged on docs.rs 0.7.0 — no src change **expected**.

**§9:** Absorb `#58`. Decline T305 rusqlite. last-PR `#220` N/A empty — **no T306**.

**Touch:** `Cargo.toml` / `Cargo.lock`; maybe `routes.rs` / tests if signatures moved; CHANGELOG; conductor.

**Isolation:** No live daemon HTTP bind as DoD (hermetic `127.0.0.1:0`).

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` + `opencode-review.md` (HEAD `bdd34f9`). Fold-in verify: [Cargo `"0.6.6"` := `>=0.6.6, <0.7.0`](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html); `cargo pkgid tower-http` unique; docs.rs 0.7.0 constructors; `routes.rs:66/:68`; `security.rs:154`; `#58` files toml+lock; last-PR `#220` empty; tokio lock 1.53.1; no ServeDir/CorsLayer/CsrfLayer in src.

### Pins locked by fold-in

1. **F1 (Agy m1 + OpenCode summary):** manifest **must** go to `0.7` — 0.x caret does **not** include the next minor.
2. **F8 (Agy m3):** `--precise 0.7.0` (or execute-current 0.7.x).
3. **F9 (OpenCode O1):** `#58` extras expected; live lock after T303 may differ; do not hand-revert.
4. **§2 / §9 (OpenCode m1):** last-PR Cursor is `#220`; empty; no T306.
5. **AC4 lines:** wiring is `:66` / `:68`, not the import lines.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** workspace `0.7` required | **Folded** F1 (already) — **affirmed** as caret-unblock, unlike T303 |
| Agy | **m2** keep limit/cors/trace; no csrf/fs | **Already** F2 |
| Agy | **m3** `--precise 0.7.0` | **Folded** F8 |
| Agy | **O1** targeted `-p ai-brains-api-server` | **Already** AC2; plan names CORS + `http_body__over_limit__413` |
| OpenCode | B / M | None filed |
| OpenCode | **m1** last-PR `#216` → `#220` | **Folded** §2 / §9 |
| OpenCode | **O1** `#58` windows-sys/socket2/windows-core extras | **Folded** F9 |
| OpenCode | “limit/cors/trace no breaking API” | **Partial** — our constructors unchanged; gRPC classify `#[non_exhaustive]` unused |
| both | last-PR Cursor empty | **Affirm** — `#220` N/A; **no T306** |

No Blockers/Majors to decline. No new placeholder. Do **not** edit `*-review.md`. Do **not** execute until go.
