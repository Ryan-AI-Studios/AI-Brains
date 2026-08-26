# T310 — `ai-brains update` graph-on + PATH daemon SQLCipher 4.14

- **Track ID:** T310-UpdateGraphOnDaemon
- **Status:** **Planned** (Pending until **go**) — **placeholder**
- **Category:** CHORE / FEATURE (light)
- **Owner:** Grok
- **Source:** T306 full plan 2026-08-26 live baseline. Not last-PR Cursor (`#222` empty).
- **Depends on:** T306 PATH CLI 4.14 (suggested after T306); T222 `GRAPH_REINSTALL_SOOT`; T84 `run_update`.
- **F0:** Plan-only until go. Do **not** `daemon stop` / `cargo install` / edit `run_update` as planning.

## 1. Objective

1. **T84 `run_update` must not undo T222.** Live `daemon.rs:1069–1072` runs `cargo install --path crates/ai-brains-cli --locked` **without** `--features graph`. After T306, `ai-brains update` would reinstall a **graph-off** CLI.
2. **PATH `ai-brainsd` is still SQLCipher 4.10-era.** `C:\Users\RyanB\.cargo\bin\ai-brainsd.exe` mtime **2026-08-22 14:48:10** (older than T300 CLI). Daemon is the WAL **writer**; T305’s 4.14 WAL-reset fix is not on that binary. Replacing it on Windows may require an **owner-confirm daemon stop** (T188 / T300 class).

## 2. Live baseline (2026-08-26)

| Signal | Observation |
|--------|-------------|
| `run_update` | `daemon.rs:1030–1099` — stop daemon, install CLI **without** graph feature, install `ai-brainsd --locked`, restart. |
| PATH CLI (pre-T306) | graph-on; `cipher_version=4.10.0 community` (T306 DoD). |
| PATH daemon | **21,045,248** B; **2026-08-22 14:48:10**. |
| `GRAPH_REINSTALL_SOOT` | includes `--features graph`. `run_update` does **not** use it. |

**Research:** Cargo `--path` always rebuilds; Windows cannot replace a running `ai-brainsd.exe` (file lock). Snapshot — re-verify at `/plan-track 310`.

last-PR `#222` Cursor **empty**. This leftover is live src, not Cursor.

## 3. Frozen decisions (stub — expand on `/plan-track 310`)

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | CLI install in `run_update` must equal `GRAPH_REINSTALL_SOOT` (or call the constant). |
| **F2** | Daemon PATH 4.14 is in scope here, not T306 (T306 F4 no daemon stop). |
| **F3** | Owner-confirm `daemon stop` before replacing `ai-brainsd.exe` if Running. |
| **F4** | No clap 5; no floor retune; no T307/T308/T309 steal. |
| **F5** | Never `git push origin main`. |

## 4. Acceptance criteria (stub)

| AC | Proof |
|----|-------|
| **AC1** | `run_update` CLI cargo args include `--features graph` (unit or string assert vs `GRAPH_REINSTALL_SOOT`). |
| **AC2** | After owner-confirm install, PATH `ai-brainsd` is `--locked` from HEAD (no 4.10-only writer claim without a doctor-equivalent probe — daemon has no `cipher_page`; define at full plan). |
| **AC3** | Graph-on CLI not regressed (`graph_feature=available`). |

**Non-goals:** T306 CLI PATH 4.14; T307–T309; live rebuild/encrypt as planning.

**§9:** Minted from T306 §9. Decline T306 steal (F3/F4). last-PR `#222` N/A.

**Touch:** `crates/ai-brains-cli/src/commands/daemon.rs` `run_update`; PATH `ai-brainsd.exe`; tests around T84.

**Isolation:** No daemon stop as planning. No print key.
