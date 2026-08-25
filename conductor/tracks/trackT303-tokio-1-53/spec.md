# T303 — tokio 1.52.3 → 1.53.1

- **Track ID:** T303-Tokio153
- **Status:** **Planned** (Pending until **go**)
- **Category:** CHORE / DEPS
- **Owner:** Grok
- **Source:** Dependabot `#59` tokio 1.52.3→**1.53.1**. Owner requested 2026-08-25.
- **Depends on:** workspace `tokio = { version = "1.52", features = ["full"] }` — **must widen caret to 1.53** (or `1.53`) to take 1.53.1. Daemon (`ai-brainsd`) + CLI async + api-server.
- **F0:** Plan-only until go. Do **not** merge `dependabot/cargo/tokio-1.53.1`.
- **Ledger:** series DOCS TX `30b7ca9d-4932-4f00-97b8-82d5d25e633b`.

## 1. Objective

Move the workspace tokio pin to **1.53.1** (Windows signal MSRV fix in 1.53.1: remove `OnceLock::wait` from the Windows handler — `#8300`). Keep `features = ["full"]`. Prove daemon status + CLI async still compile and nextest green. No runtime redesign.

## 2. Live baseline (2026-08-25)

| Pin | Workspace | Lock | crates.io | Action |
|-----|-----------|------|-----------|--------|
| tokio | **1.52** full | **1.52.3** | **1.53.1** (2026-07-20) | Widen workspace + lock |

**Research (snapshot):** 1.53.0 (2026-07-17) additive (fs From\<OwnedHandle\>, metrics, Unix SocketAddr). Changed: mpsc receivers drop waker even if senders remain (`#8095`) — **review daemon/CLI mpsc**. 1.53.1 restores Windows MSRV. LTS remains 1.47 / 1.51 — we are on current minor, not LTS. MSRV 1.71; repo rustc **1.95.0**. **Re-read tokio/CHANGELOG.md 1.52.3…1.53.1 at execute.**

last-PR `#216` empty.

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Target **1.53.1** (not 1.53.0). Workspace `1.53` or `1.53.1`. |
| **F2** | Keep `features = ["full"]`. No io-uring unstable. |
| **F3** | No tower-http / rusqlite / clap / GHA this track. |
| **F4** | Review `#8095` mpsc waker-on-drop vs daemon pipe client. If a test hangs, fix **our** wait, do not pin 1.52. |
| **F5** | `cargo deny` + `audit` green. |
| **F6** | Do not merge Dependabot remote. Never `git push origin main`. |
| **F7** | CHANGELOG Unreleased. Manual: `ai-brains daemon status` still Running/Stopped contract (T199/T297) after a local `cargo run` — **do not** stop the live daemon as DoD unless a test requires it (prefer hermetic). |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | workspace tokio **1.53**; lock **1.53.1**. |
| **AC2** | clippy `-D warnings` workspace; nextest workspace; deny; audit. |
| **AC3** | Existing daemon status / pipe tests stay green (T199 vault-independence, T297 contrast units). |
| **AC4** | No rusqlite/tower-http/clap lock bump in this diff. |
| **AC5** | CHANGELOG. |

## 5–12

**Non-goals:** LocalRuntime product use; tower-http 0.7 (T304); service install.

**Risk:** mpsc `#8095` behavioral change. Mitigation: targeted daemon/CLI nextest.

**§9:** Absorb `#59`. Decline T304/T305 steal. last-PR `#216` N/A.

**Touch:** `Cargo.toml` workspace tokio; `Cargo.lock`; CHANGELOG; conductor.

**Isolation:** No live `daemon stop` as DoD. No `cargo install` unless owner asks.
