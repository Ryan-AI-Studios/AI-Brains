# T307 — Unify tower-http (drop reqwest 0.6.11 dual)

- **Track ID:** T307-ReqwestTowerHttpUnify
- **Status:** **Planned** (Pending until **go**; **upstream-blocked** at this plan)
- **Category:** CHORE / DEPS
- **Owner:** Grok
- **Source:** T304 R2 — api-server `tower-http` **0.7.0**; reqwest keeps **0.6.11**. Owner leftover placeholders 2026-08-26; this pass upgrades the stub to a full plan.
- **Depends on:** T304 `#221` workspace `tower-http = { version = "0.7", features = ["limit", "cors", "trace"] }`. Workspace `reqwest = { version = "0.13", features = ["json"] }` (`Cargo.toml:50`).
- **Blocks / feeds:** One lock line of `tower-http` (smaller graph, quieter `deny.toml` `multiple-versions = "warn"`). Does **not** unblock T308/T309/T310. Capture path does not use `tower-http`.
- **Absorbs:** T304 R2 (dual required after `#221`); leftover README T307 problem text.
- **Not absorbed (DoD):** T304 R4 csrf; T308 sparse remediator; T309 `table_exists`; T310 `run_update` + PATH daemon; clap 5; floor retune; `[patch.crates-io]`; git-dep reqwest / tower-http; new crates (`tower-reqwest`).
- **Research date:** 2026-08-26 (HEAD `34379bf` T306 `#223`).
- **Ledger:** planning DOCS TX `6e17c94a-a250-4f24-b579-3b4a66970aa6`. Series mint DOCS `c62396f6-4532-4335-b10b-f31b3fa02ec2`. Implement starts a **CHORE** TX on **go** only if Phase 0 F3 does **not** halt.
- **Isolation:** Do **not** `[patch.crates-io]`. Do **not** git-dep [reqwest#3062](https://github.com/seanmonstar/reqwest/pull/3062) or tower-http `main`. Do **not** add CorsLayer / CsrfLayer. Do **not** merge Dependabot remotes. Never `git push origin main`. Do **not** `cargo install` / live HTTP bind / `daemon stop` as planning.

---

## 1. Objective

1. **One `tower-http` in the lock.** After go (if unblocked), `Cargo.lock` has **0.7.x only**. The leftover **0.6.11** copy pulled by `reqwest 0.13.4` is gone. Workspace api-server stays on 0.7 (do **not** unify by reverting T304).
2. **Keep T161 CORS deny and T304 layers.** `RequestBodyLimitLayer` + `TraceLayer` stay wired. No `CorsLayer`. Same reqwest features (`json` only).
3. **Honest halt if upstream is still closed.** crates.io `reqwest` **0.13.4** still pins `tower-http = "0.6.8"` (`>=0.6.8, <0.7.0`). If that is still true on go, **Stop-Before (F3)** — dual remains; track becomes **Blocked**; do **not** Complete a no-op.
4. **Capture independence.** Deps-only. No new events, no contracts DTO, no models API change expected. Capture does not depend on `ai-brains-models` / `reqwest`.

This unblocks lock hygiene the T304 bump could not: two semver-incompatible `tower-http` copies compile twice and split CVE/audit surface. It does **not** change the append-only event log.

---

## 2. Live baseline (re-scan 2026-08-26)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `34379bf` — T306 `#223` Completed. Tree **CLEAN**. `origin/main...HEAD` **0/0**. Branch `main`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` LastWriteTime **2026-08-26 6:54:32 AM**; `ai-brains 0.1.3`. |
| PATH `doctor --json` | `cipher_page` **`cipher_version=4.14.0 community`** (T306 done). `graph_feature=available`. `vault_open` opened read-only. **Not this hole.** |
| rustc / cargo | **1.95.0** / **1.95.0**. |
| Last GitHub PR | [#223](https://github.com/Ryan-AI-Studios/AI-Brains/pull/223) T306 (`mergedAt` **2026-08-26T12:34:00Z**). `pulls/223/comments`, `/reviews`, `issues/223/comments` all **`[]`**. Body has a Bugbot **CURSOR_SUMMARY** (low-risk overview, no defect). **last-PR Cursor: N/A.** Open PRs: **none**. **No leftover from Cursor. No T311.** |
| Ledger | **0 pending / 0 drift** at scan (before this DOCS TX). |
| `ISSUES.md` | **Does not exist.** |
| Planning bump | **Not run.** |

### 2.2 Why this residual still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Dual `tower-http` **0.7.0** + **0.6.11** | T304 could not unify: reqwest `^0.6.8` := `>=0.6.8, <0.7.0`. `cargo tree -i tower-http` is ambiguous. `deny.toml` `multiple-versions = "warn"` (not deny — dual is **not** a CI fail today). **DoD when unblocked.** |
| Bare `cargo update -p tower-http --precise 0.7.0` | **Fails** (T304 R2). Wrong tool. Unify by bumping **reqwest**, not by forcing the 0.6 pkgid. |
| Revert api-server to 0.6 | Undoes T304. **Decline.** Unify **up**. |
| `[patch.crates-io]` / fork reqwest | [Cargo patch](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html) replaces **source**, cannot **weaken** a `^0.6.8` requirement onto 0.7. Forking reqwest is out of scope. **Decline.** |
| git-dep [reqwest#3062](https://github.com/seanmonstar/reqwest/pull/3062) | **Open** since 2026-06-29 (`merged: false`, last update 2026-07-13). Owner: brotli tests hanging; filed [tower-http#712](https://github.com/tower-rs/tower-http/pull/712) (merged Jul 13) + follow-up [#722](https://github.com/tower-rs/tower-http/pull/722) (merged Aug 15). crates.io tower-http still **0.7.0** (those fixes unpublished). `deny.toml` `unknown-git = "deny"`. **Decline.** |
| `tower-reqwest` adapter | crates.io **0.6.1** MIT OR Apache-2.0 (not AGPL). Adds a crate; does **not** drop 0.6.11. **Decline** (F4). |
| csrf feature | T304 R4. **Decline.** |
| Dual `http` 0.2.12 + 1.4.1 / `hyper` 0.14.32 + 1.9.0 | Ecosystem (legacy hyper vs hyper 1). Not reqwest’s tower-http pin. **Not this track.** |
| Desktop crate version **0.1.2** vs workspace **0.1.3** | Tauri app version, not the dual. **Not this track.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Workspace reqwest | `Cargo.toml:50` | `{ version = "0.13", features = ["json"] }`. Caret := `>=0.13.0, <0.14.0`. |
| Workspace tower-http | `Cargo.toml:83` | `{ version = "0.7", features = ["limit", "cors", "trace"] }`. **Keep.** |
| Workspace axum | `Cargo.toml:81` | **0.8.9**. Compatible with tower-http 0.7 (`http ^1` / `tower ^0.5`). |
| Lock reqwest | `Cargo.lock` | **0.13.4** checksum `219c5811…`. Depends on `tower-http 0.6.11`. |
| Lock tower-http 0.6.11 | unique pkgid via reqwest | `follow-redirect` only (reqwest default-features false). |
| Lock tower-http 0.7.0 | unique pkgid via api-server | limit / cors / trace graph. |
| Invert tree 0.6.11 | `cargo tree -i tower-http@0.6.11 --locked` | `reqwest` → `ai-brains-models` → cli/brain/retrieval/daemon; also `ai-brains-desktop`. |
| Invert tree 0.7.0 | `cargo tree -i tower-http@0.7.0 --locked` | `ai-brains-api-server` → `ai-brainsd`. |
| Direct reqwest crates | `crates/ai-brains-models/Cargo.toml:15`; `apps/desktop/src-tauri/Cargo.toml:26` | `workspace = true`. Models tests: `wiremock`. Desktop tests: `httpmock`. |
| Models callers | `llama_cpp.rs:65/:114/:150`; `ollama.rs:45/:95` | `reqwest::Client` / `Client::builder()`. JSON + HTTP. **No gzip/brotli feature.** |
| Production layers | `crates/ai-brains-api-server/src/routes.rs` | Import `:11–12`; `BODY_LIMIT_BYTES = 1 MiB` `:29`; `RequestBodyLimitLayer::new` **`:66`**; `TraceLayer::new_for_http()` **`:68`**. Comment `:46` CORS deny. |
| CORS test | `tests/security.rs` `http_cors__default__no_allow_origin_star` **`:154`** | ACAO **absent** (also not `*`). Body: `http_body__over_limit__413` **`:184`**. |
| CorsLayer / CsrfLayer | **none** in `*.rs` | `ledgerful search CorsLayer` empty. |
| `deny.toml` | `:21–23` | `multiple-versions = "warn"`; `unknown-git = "deny"`. Dual is warn. Git reqwest would **fail deny**. |
| Hotspots | `project.rs` #1 | **Do not touch.** Expected product diff is lock (+ maybe `Cargo.toml:50` + CHANGELOG). |

### 2.4 Dependency / standards research (2026-08-26) — snapshot, re-verify at execute

| Pin | Workspace / lock | crates.io / docs | Action |
|-----|------------------|------------------|--------|
| reqwest | **0.13** json / **0.13.4** | **0.13.4** latest (`cargo info` + `cargo search` 2026-08-26). Published **2026-05-25**. | **Bump only if** a newer crates.io line allows tower-http **0.7**. |
| reqwest → tower-http | lock **0.6.11** | crates.io **and** [master `Cargo.toml`](https://github.com/seanmonstar/reqwest/blob/master/Cargo.toml) still `tower-http = { version = "0.6.8", default-features = false, features = ["follow-redirect"] }`. Master package version still **0.13.4**. | **F3** if unchanged. |
| tower-http | **0.7** / **0.7.0 and 0.6.11** | latest **0.7.0** (2026-06-15). git `main` has #712/#722 **unpublished**. | Keep 0.7.x for api-server. Prefer **0.7.1+** if published when reqwest allows 0.7; **accept 0.7.0** if that is what reqwest pins (our `json` path does not enable decompression). |
| axum | **0.8.9** | — | **No bump.** |
| tokio | **1.53** / **1.53.1** | — | **No bump / no revert.** |
| rusqlite | exact **0.40.2** | — | **No bump.** |
| clap | **4.5** / lock **4.6.1** | clap **5 declined** | **No bump.** |
| thiserror | **2.0** / **2.0.20** + **1.0.69** | — | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged. |
| New crates | — | `tower-reqwest` 0.6.1 MIT/Apache | **Zero.** |

**reqwest upstream (verified 2026-08-26):**

- [crates.io reqwest 0.13.4 `Cargo.toml`](https://docs.rs/crate/reqwest/0.13.4/source/Cargo.toml): `tower-http` version **`0.6.8`**, features `follow-redirect`, default-features false. gzip/brotli/zstd/deflate are **optional** and map to tower-http decompression — workspace does **not** enable them.
- [reqwest#3062](https://github.com/seanmonstar/reqwest/pull/3062) **open**, not merged. CI hang on brotli; seanmonstar linked tower-http decompression regressions (#712 merged; #722 merged). **No crates.io reqwest > 0.13.4.** GitHub issue search `tower-http 0.7` in reqwest: **0 issues**.
- [A More Modular reqwest](https://seanmonstar.com/blog/modular-reqwest/) (2025-03-04): reqwest **intentionally** uses tower-http for redirects + decompression. Dual copies are the ecosystem cost until reqwest’s pin moves.

**Cargo (verified [Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) + [Overriding](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html) 2026-08-26):**

- `"0.13"` := `>=0.13.0, <0.14.0`. A **0.13.5** that allows tower-http 0.7 needs **lock-only** `--precise` (caret already open). A **0.14.x** needs workspace **`0.14`** (T304-class caret-unblock).
- `"0.6.8"` on reqwest’s tower-http := `>=0.6.8, <0.7.0`. **Cannot** resolve 0.7.0.
- `[patch.crates-io]` cannot widen a transitive requirement. `unknown-git = "deny"`.

**tower-http 0.7.0 ([release](https://github.com/tower-rs/tower-http/releases/tag/tower-http-0.7.0)):** `follow-redirect` still a feature. Breaking changes are compression / ServeDir / csrf **new** — unused by us today. api-server constructors unchanged (T304). When reqwest moves, re-read docs.rs for the **reqwest-declared** 0.7.x at execute.

**N/A (implementation pattern):** No new API. Pattern is T302/T303/T304 CHORE: workspace floor if needed + `cargo update -p <crate> --precise` + existing tests. Reference: T304 itself.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts CHORE **only if** Phase 0 F3 does not halt. Do **not** bump as planning. |
| **F1 — Unify up** | Target a **single** lock `tower-http` **0.7.x**. Do **not** revert api-server to 0.6. Workspace tower-http stays `{ version = "0.7", features = ["limit", "cors", "trace"] }`. |
| **F2 — reqwest features** | Keep `json` only. Do **not** add gzip/brotli/zstd/deflate/native-tls. |
| **F3 — Stop-Before (primary today)** | If latest **crates.io** reqwest still requires tower-http **0.6.x** (or master still `0.6.8` with no newer published crate): **halt**. Dual stays. Conductor → **Blocked** (not Completed). Write crates.io + `#3062` evidence in `deferred.md`. **No** product commit. **No** patch. **No** git dep. |
| **F4 — No new crates** | No `tower-reqwest`, no fork, no `[patch.crates-io]`. |
| **F5 — T161 CORS deny** | No `CorsLayer` / `CsrfLayer` / `ServeDir`. Test AC still ACAO **absent**. Layers stay `:66` / `:68`. |
| **F6 — Peer pins** | No rusqlite / clap / thiserror / tokio steal or revert. axum stays 0.8.9 unless a reqwest bump **forces** a documented follow (then **Stop-Before** — not silent). |
| **F7 — CHANGELOG** | Unreleased Changed row **only if** a bump ships. F3 halt: no CHANGELOG. |
| **F8 — Precise pkgid** | `cargo update -p reqwest --precise <ver>` ([cargo-update `--precise`](https://doc.rust-lang.org/cargo/commands/cargo-update.html)). Today `cargo pkgid reqwest` is unique (`0.13.4`). If 0.14: workspace `"0.14"` first. **Do not** `cargo update -p tower-http --precise 0.7.0` as the unify method (T304: fails). |
| **F9 — Lock extras** | Accept resolver extras (windows-sys / socket2 / hyper). Do **not** hand-edit lock. Abort if rusqlite / clap / thiserror / tokio leave their pins. |
| **F10 — Git** | Never `git push origin main`. Do not merge Dependabot remotes. |
| **F11 — Git deps** | Do **not** point reqwest at `#3062` / `refs/pull/3062/head`. `unknown-git = "deny"`. |
| **F12 — tower-http 0.7.1** | Prefer published **0.7.1+** (includes #712/#722) **if** crates.io has it when reqwest allows 0.7. If reqwest pins **0.7.0** and our features stay `json` (no decompression), **accept 0.7.0**. Do not wait on unpublished git tower-http. |
| **F13 — F3 is not Complete** | Documented halt is **Blocked**, not a fake Completed DoD. Owner may leave it parked until a crates.io reqwest ships. |
| **F14 — Isolation** | No live loopback bind as DoD (`127.0.0.1:0` tests). No `daemon stop`. No `cargo install`. |
| **F15 — TDD** | No new tests expected (lock/deps). Proof = lock tree + existing `http_cors__default__no_allow_origin_star` + `http_body__over_limit__413`. F3 halt: no red tests. |
| **F16 — Cross-model** | CHORE deps. `codex-review` if a bump ships (SECURITY-adjacent HTTP stack). F3 halt: skip. |
| **F17 — clap 5** | **Still declined.** |
| **F18 — Density / table_exists / update** | **T308 / T309 / T310.** Do not steal. |
| **F19 — Capture independence** | No events. models HTTP client stays the same crate surface (`Client` / `json`). |
| **F20 — Debt file** | `conductor/ISSUES.md` does **not** exist. Residuals → `deferred.md`. |
| **F21 — deny multiple-versions** | Do **not** flip `warn` → `deny` this track (other duals: thiserror 1+2, http, hyper). Quieting tower-http dual is the win; policy stays warn. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | **Either** `Cargo.lock` has **one** `name = "tower-http"` at **0.7.x**, **or** F3 halt with dated crates.io evidence (`cargo info reqwest` still 0.13.4 / tower-http 0.6.8) and conductor **Blocked**. |
| **AC2** | If bump: `cargo tree -i tower-http@0.6.11 --locked` **fails** (no such pkgid). If F3: command still lists reqwest → 0.6.11 (expected). |
| **AC3** | If bump: `http_cors__default__no_allow_origin_star` green; ACAO absent. If F3: not run as a product change. |
| **AC4** | If bump: `http_body__over_limit__413` green; `routes.rs:66/:68` still the two layers. |
| **AC5** | If bump: `cargo clippy -p ai-brains-api-server --all-targets -- -D warnings` + `cargo nextest run -p ai-brains-api-server` + models nextest (`-p ai-brains-models`) green. |
| **AC6** | If bump: workspace clippy / nextest / deny / audit green. rusqlite **0.40.2**, clap **4.x**, tokio **1.53.1**, thiserror **2.0.20** still present. |
| **AC7** | If bump: CHANGELOG Unreleased Changed. If F3: no CHANGELOG. |
| **AC8** | `git diff -- crates/` empty **unless** a reqwest API break forces a documented models/desktop compile fix (then list files; still no CorsLayer). Expected default: toml + lock + CHANGELOG + conductor only. |
| **AC9** | Phase 0: cwd `C:\dev\AI-Brains`; `cargo info reqwest`; read reqwest’s declared `tower-http` version from crates.io / docs.rs source. If still 0.6.x → **F3 halt** (do not start CHORE product TX). |

No live HTTP server as DoD.

---

## 5. Design notes

### 5.1 Why F3 halt is the expected go outcome today

crates.io latest reqwest **is** 0.13.4; its Cargo.toml **and** git master still declare `tower-http 0.6.8`. The only in-flight bump is **unmerged** `#3062`, blocked on decompression hangs that needed tower-http `#712`/`#722` — and those fixes are **not** on crates.io 0.7.0. Taking git HEAD would fail `cargo deny` (`unknown-git`) and still not be a published contract.

### 5.2 Caret: 0.13.5 vs 0.14

| Published reqwest | Workspace edit | Lock command |
|-------------------|----------------|--------------|
| 0.13.x allowing tower-http 0.7 | **None** (`"0.13"` already open) | `cargo update -p reqwest --precise 0.13.x` |
| 0.14.x | `reqwest = { version = "0.14", features = ["json"] }` | `cargo update -p reqwest --precise 0.14.x` |

Re-read reqwest changelog for 0.14 breaking `Client` / rustls defaults before compiling models.

### 5.3 Why not unify down

T304 moved the **server** stack to 0.7 for csrf-era maintenance and constructor-stable limit/trace. Putting api-server back on 0.6 reopens that Dependabot hole. Unify by waiting for the **client** pin.

### 5.4 Capture independence

`ai-brains-capture` has no `reqwest` / `tower-http`. models HTTP is the LLM probe path (feature-gated from capture). A lock unify does not add events or CoT.

---

## 6. Non-goals

- `[patch.crates-io]` / forking reqwest / git `#3062`
- `tower-reqwest` or any new crate
- Enabling csrf / fs / compression / CorsLayer
- Reverting T304
- Flipping `deny.toml` `multiple-versions` to deny
- clap 5; rusqlite; tokio; axum 0.9
- T308 remediator; T309 `table_exists`; T310 update/daemon
- Dual thiserror 1.x; dual hyper 0.14; desktop `0.1.2` version string
- Live daemon HTTP; `cargo install`; live rebuild/encrypt
- Dependabot remote merge / close hygiene
- Publishing a 0.7.1 of tower-http ourselves

---

## 7. Verification plan

TDD: **no new named tests** (F15). On go:

```powershell
# Phase 0 — re-verify (do not bump yet)
cargo info reqwest --color never
cargo info tower-http --color never
# Confirm reqwest's declared tower-http (docs.rs source or crates.io download)
# If still 0.6.x → F3 halt. Do not start CHORE product TX.

# Only if F3 does not halt:
cargo update -p reqwest --precise <ver>
# if 0.14: edit Cargo.toml:50 first
Select-String -Path Cargo.lock -Pattern 'name = "tower-http"' -Context 0,1
cargo tree -i tower-http --locked --color never
# expect a single 0.7.x (or two 0.7.x patches — never 0.6)

cargo clippy -p ai-brains-api-server --all-targets -- -D warnings
cargo nextest run -p ai-brains-api-server
cargo nextest run -p ai-brains-models
```

Named stay-green tests (bump path only):

- `http_cors__default__no_allow_origin_star`
- `http_body__over_limit__413`

Full workspace gate only if a bump ships. F3 halt: `ledgerful verify --scope fast` on conductor closeout (Blocked row + deferred evidence).

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Fake Complete while dual remains | **F13 / AC1** Blocked ≠ Completed |
| `cargo update -p tower-http --precise 0.7.0` “to unify” | **F8** — that command **fails**; bump reqwest |
| git `#3062` to skip wait | **F11** deny unknown-git; decompression hang |
| `[patch]` fork | **F4** |
| reqwest 0.14 Client / TLS break | Phase 0 changelog; models compile; Stop-Before if axum/tokio forced |
| Accidental CorsLayer while “fixing compile” | **F5 / AC3** |
| clap 5 / rusqlite drift in lock extras | **F6 / F9 / AC6** |
| Enabling gzip to “use 0.7 decompression” | **F2** — hang class; not our feature |
| Unpublished tower-http 0.7.1 | **F12** accept 0.7.0 on json-only |
| Live HTTP / daemon stop “to prove CORS” | **F14** hermetic oneshot |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-26.

| Item | Disposition |
|------|-------------|
| T304 R2 dual tower-http 0.6.11 via reqwest 0.13.4 | **Absorb** F1–F3 / AC1–AC2 |
| T304 R1 Dependabot `#58` close hygiene | **Decline** — standing |
| T304 R3 lock extra variance | **Decline** — F9; do not hand-edit |
| T304 R4 csrf | **Decline** F5 |
| T306 R2 PATH `ai-brainsd` 4.10 | **Decline steal → T310** |
| T306 R3 T84 `run_update` graph-off | **Decline steal → T310** |
| T306 R4 sparse remediator | **Decline steal → T308** |
| T306 R5 `recovery_kit_event` | **Not this track** |
| T306 R6 INSTALL.md 0.1.2 header | **Decline** — docs drift |
| T213 L4 / T305 R2 `table_exists` | **Decline steal → T309** |
| T302 R2 dual thiserror 1.x | **Decline** — Tauri/json-patch |
| Dual hyper 0.14 / http 0.2 | **Decline** — not tower-http |
| Desktop version 0.1.2 vs workspace 0.1.3 | **Decline** — not the dual |
| clap 5 | **Decline** F17 |
| T278 floor retune | **Decline** F18 |
| last-PR Cursor `#223` | **N/A empty** — comments/reviews/issue comments `[]`. Bugbot CURSOR_SUMMARY is upsell/overview, not a defect. **No T311.** |
| T240 F2 / leftover `--write` / T263 H2 | **Decline** — standing |

---

## 10. Implement order (on go)

1. Phase 0: AC9 `cargo info reqwest` + declared tower-http range. If 0.6.x → **F3 halt** (Blocked + deferred evidence). **Stop.**
2. If unblocked: CHORE TX. Workspace caret only if 0.14. `cargo update -p reqwest --precise <ver>` (F8).
3. Confirm single 0.7.x (AC1/AC2). F9 extras; abort on peer-pin drift.
4. AC3–AC6 tests + gate. AC7 CHANGELOG. AC8 crate diff empty (or listed compile fixes).
5. Conductor **Completed** only if dual gone. Phase 6: `track/T307-*` → PR → watch `CI` → squash-merge. Never `git push origin main`.

---

## 11. Soft residuals (post-close)

| Residual | Note |
|----------|------|
| Dual remains until crates.io reqwest | **Expected** until F3 lifts; Blocked |
| reqwest#3062 still open | Watch only; do not merge into this repo |
| tower-http 0.7.1 unpublished | F12 |
| `deny.toml` multiple-versions still warn (thiserror/http/hyper) | F21 |
| T308 / T309 / T310 | Not stolen |
| PATH CLI already 4.14 | T306; not this hole |

---

## 12. Touch map

| Path | Role |
|------|------|
| `Cargo.toml:50` | reqwest caret **only if** 0.14+ |
| `Cargo.lock` | reqwest + drop tower-http 0.6.11 |
| `CHANGELOG.md` | Unreleased if bump |
| `conductor/conductor.md` | Pending → Completed **or** Blocked |
| `conductor/deferred.md` | F3 evidence or R2 done |
| `crates/ai-brains-api-server/src/routes.rs` | **No edit** unless 0.7.x constructor break (unexpected) |
| `crates/ai-brains-models/src/*.rs` | **No edit** unless reqwest 0.14 Client break |
| `apps/desktop/src-tauri/**` | **No edit** unless compile-forced |

Do **not** touch `project.rs` (hotspot #1).
