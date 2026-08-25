# T303 — tokio 1.52.3 → 1.53.1

- **Track ID:** T303-Tokio153
- **Status:** ✅ **Completed** (2026-08-25)
- **Category:** CHORE / DEPS
- **Owner:** Grok
- **Source:** Dependabot `#59` tokio 1.52.3→**1.53.1**. Owner requested 2026-08-25.
- **Depends on:** workspace `tokio = { version = "1.52", features = ["full"] }` (`Cargo.toml:44`). `"1.52"` is Cargo default/caret **`>=1.52.0, <2.0.0`** — **already permits 1.53.1**. Dependabot `#59` is **lockfile-only**. Raise workspace floor to **`1.53`** so 1.52.x cannot resolve after this track (1.52.4 exists and does **not** include `#8300`). Daemon (`ai-brainsd`) + CLI async + api-server.
- **F0:** Plan-only until go. Do **not** merge `dependabot/cargo/tokio-1.53.1`.
- **Ledger:** series DOCS TX `30b7ca9d-4932-4f00-97b8-82d5d25e633b`. Fold-in DOCS TX `6014c95c-0514-4b07-bf26-d4ad8dddc137`. Implement starts **CHORE** TX on go.
- **AI fold-in:** 2026-08-25 `agy-review.md` + `opencode-review.md` (HEAD `33cf7ea`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** OpenCode m1 last-PR `#219`; OpenCode m2 caret already allows 1.53.1 (spec “must widen to resolve” was wrong); Agy m3 `--precise 1.53.1`; OpenCode O-1 `#59` windows-sys extras. **Already:** Agy m1 = F1/AC1 as **floor**; Agy m2 = F1 (1.53.1 not 1.53.0); Agy O1 = AC3. **Partial:** Agy 1.53.1 `#8252` is **unstable** alt-timer, not a stable product AC. Disposition **§13**.

## 1. Objective

Move the lock to **tokio 1.53.1** (Windows signal MSRV fix: remove `OnceLock::wait` from the Windows handler — `#8300`). Keep `features = ["full"]`. Raise the workspace **minimum** to `1.53`. Prove daemon status + CLI async still compile and nextest green. No runtime redesign.

## 2. Live baseline (2026-08-25 fold-in)

| Pin | Workspace | Lock | crates.io (fold-in) | Action |
|-----|-----------|------|---------------------|--------|
| tokio | **1.52** full (`Cargo.toml:44`) | **1.52.3** (unique pkgid) | **1.53.1** (`#59`, 2026-07-20) | Floor `1.53` + `cargo update -p tokio --precise 1.53.1` |
| clap / rusqlite / tower-http / thiserror | 4.5 / 0.39.0 / 0.6 / 2.0 | 4.6.1 / 0.39.0 / 0.6.11 / 2.0.20 | — | **Do not bump** |

**`cargo pkgid tokio`** → `tokio@1.52.3` only (not ambiguous). `--precise` is hygiene (T302 F8), not disambiguation.

**Research (verified fold-in; re-read [tokio CHANGELOG](https://github.com/tokio-rs/tokio/blob/master/tokio/CHANGELOG.md) 1.52.3…1.53.1 at execute):**

- **1.53.0** (2026-07-17): additive `fs` `From<OwnedHandle>` / `OwnedFd` (`#8266`), task schedule-latency metrics (`#7986`), Unix `SocketAddr` (`#8144`). Changed: mpsc receivers drop waker even if senders remain (`#8095`); Windows signal globals refactor (`#8231`).
- **1.53.1** (2026-07-20): **stable** [signal: restore MSRV, remove `OnceLock::wait` from Windows handler `#8300`](https://github.com/tokio-rs/tokio/blob/master/tokio/CHANGELOG.md). `#8252` timer cancellation race is **Fixed (unstable)** alt-timer — out of F2 (no io-uring / unstable).
- LTS remains 1.47 / 1.51 — we are on **current minor**, not LTS. MSRV **1.71**; repo rustc **1.95.0**. Latest 1.52 patch is **1.52.4** (not our target; no `#8300`).
- `#59` lock diff is **Cargo.lock only** (17/17): tokio version/checksum **plus** many `windows-sys` edge flips (`0.60.2→0.61.2`, `0.52.0→0.59.0`, one `0.60.2→0.59.0`). Lock already has windows-sys **0.45 / 0.52 / 0.59 / 0.60.2 / 0.61.2**. Expected extras — do not revert. tokio 1.52.3 already depends on `windows-sys 0.61.2`; `tokio-macros 2.7.0` unchanged.

**Live `#8095` / `#8300` blast radius:**

- tokio mpsc: only `crates/ai-brainsd/src/lib.rs:98` `mpsc::channel(64)`, single consumer `recv().await` loop (`:119`), daemon-lifetime sender; send `.map_err("daemon queue closed")` (`:178,:201,:241`). **No** drop-receiver-while-senders-live product path. Safe for `#8095`.
- `windows_service.rs` and `ai-brains-sources/src/hermes.rs` use **`std::sync::mpsc`** — out of `#8095`.
- Windows signal: `shutdown_signal.rs:41` `tokio::signal::ctrl_c()` (`cfg(not(unix))`); CLI `main.rs:3793` same. `#8300` is directly relevant. SCM path does **not** use this helper.

last-PR Cursor: **`#219`** (T302, HEAD `33cf7ea`) — comments/reviews **empty**. `#218` / `#217` also empty. **No T306.**

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Lock **1.53.1** (not 1.53.0). Workspace floor **`1.53`** + `features = ["full"]`. Toml bump is a **minimum-version floor**, not a caret-unblock (`^1.52` already allows 1.53.1). |
| **F2** | Keep `features = ["full"]`. No io-uring unstable. No product `LocalRuntime`. |
| **F3** | No tower-http / rusqlite / clap / thiserror / GHA this track. |
| **F4** | Review `#8095` vs daemon pipe client. If a test hangs, fix **our** wait, do not pin 1.52. Fold-in: pattern is daemon-lifetime single consumer — no src change expected. |
| **F5** | `cargo deny` + `audit` green. |
| **F6** | Do not merge Dependabot remote. Never `git push origin main`. |
| **F7** | CHANGELOG Unreleased. Manual: `ai-brains daemon status` Running/Stopped contract (T199/T297) via **hermetic tests** (`cargo run` allowed). **Do not** stop the live daemon as DoD. |
| **F8** | `cargo update -p tokio --precise 1.53.1` ([cargo-update `--precise`](https://doc.rust-lang.org/cargo/commands/cargo-update.html)). Re-check latest 1.53.x patch at execute. |
| **F9** | Expected lock extras from `#59`: `windows-sys` edge re-resolutions listed in §2. Do not revert. Abort only if rusqlite / tower-http / clap / thiserror versions move. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | workspace tokio **1.53**; lock **1.53.1**. |
| **AC2** | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo nextest run --workspace` + `cargo deny check` + `cargo audit`. |
| **AC3** | Existing daemon status / pipe tests stay green (T199 vault-independence, T297 contrast). Include `-p ai-brainsd` (shutdown_signal abort test) and `-p ai-brains-cli` `daemon_status` filters. |
| **AC4** | No rusqlite / tower-http / clap / thiserror lock bump. F9 windows-sys edges **allowed**. |
| **AC5** | CHANGELOG Unreleased. |

## 5–12

**Non-goals:** LocalRuntime product use; tower-http 0.7 (T304); rusqlite (T305); service install; live `daemon stop`.

**Risk:** mpsc `#8095` behavioral change. Mitigation: targeted daemon/CLI nextest (AC3). Fold-in src review: single-consumer daemon-lifetime — low.

**§9:** Absorb `#59`. Decline T304/T305 steal. last-PR `#219` N/A empty — **no T306**.

**Touch:** `Cargo.toml` workspace tokio floor; `Cargo.lock`; CHANGELOG; conductor.

**Isolation:** No live `daemon stop` as DoD. No `cargo install` unless owner asks.

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` + `opencode-review.md` (HEAD `33cf7ea`). Fold-in verify: [Cargo default req](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) `"1.52"` := `>=1.52.0, <2.0.0`; `cargo pkgid tokio` unique; `#59` files = `Cargo.lock` only; tokio CHANGELOG 1.53.1 `#8300` stable / `#8252` unstable; lib.rs mpsc `:98/:119/:178`; `shutdown_signal.rs:41`; last-PR `#219` comments/reviews empty.

### Pins locked by fold-in

1. **F1 (OpenCode m2 + Agy m1 restated):** caret already allows 1.53.1; workspace `1.53` is a **floor** so 1.52.x (incl. 1.52.4) cannot resolve.
2. **F8 (Agy m3):** `--precise 1.53.1`.
3. **F9 / AC4 (OpenCode O-1):** `#59` windows-sys edge flips are expected extras.
4. **§2 / §9 (OpenCode m1):** last-PR Cursor is `#219`; empty; no T306.
5. **F4:** `#8095` blast radius verified — no product src change expected.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** must edit `Cargo.toml` because caret blocks 1.53 | **Partial** — toml **floor** kept (F1/AC1); **decline** “caret prevents 1.53.x”. Re-trigger: Cargo changes `"1.52"` to not include 1.53 |
| Agy | **m2** target 1.53.1 not 1.53.0 | **Already** F1 |
| Agy | **m3** `--precise 1.53.1` | **Folded** F8 |
| Agy | **O1** targeted daemon/CLI nextest | **Already** AC3; plan names `-p ai-brainsd` + `daemon_status` |
| Agy | 1.53.1 `#8252` timer race | **Partial** — changelog **Fixed (unstable)**; not an AC (F2) |
| OpenCode | B / M | None filed |
| OpenCode | **m1** last-PR `#216` → `#219` | **Folded** §2 / §9 |
| OpenCode | **m2** caret already allows 1.53.1; `#59` lock-only | **Folded** §2 / F1 restated |
| OpenCode | **O-1** windows-sys extras | **Folded** F9 / AC4 |
| both | last-PR Cursor empty | **Affirm** — `#219` N/A; **no T306** |

No Blockers/Majors to decline. No new placeholder. Do **not** edit `*-review.md`. Do **not** execute until go.
