# T310 — `ai-brains update` graph-on + PATH daemon SQLCipher 4.14

- **Track ID:** T310-UpdateGraphOnDaemon
- **Status:** **Planned** (Pending until **go**)
- **Category:** CHORE / FEATURE (light)
- **Owner:** Grok
- **Source:** T306 mint (F8/F9): PATH `ai-brainsd` still 4.10-era; T84 `run_update` omits `--features graph`. Not last-PR Cursor.
- **Depends on:** T306 PATH CLI 4.14 (`#223`); T222 `GRAPH_REINSTALL_SOOT`; T84 `run_update`; T305 rusqlite **0.40.2** / SQLCipher **4.14.0 community**.
- **Blocks / feeds:** `ai-brains update` must not undo T222. PATH daemon (WAL writer + SCM ImagePath) gets the 4.14 WAL-reset fix. Does **not** unblock T307.
- **Absorbs:** T306 F8/F9 / soft residuals “PATH `ai-brainsd` 4.10” + “`run_update` omits `--features graph`”; T309 R3 T310 placeholder.
- **Not absorbed (DoD):** T307 dual tower-http; T308 floors / remediator (PATH-behind may clear as install side-effect); T309 `table_exists` (done); clap 5; Cargo `default = []` flip; doctor 16th; daemon `cipher_page`; SCM `sc start`; T197 `AI_BRAINS_VAULT_KEY` silent zero; live `vault encrypt` / `graph rebuild`.
- **Research date:** 2026-08-26 (plan-write HEAD `e577c8c`; fold-in HEAD `87919dd` ahead **1**). Snapshot — re-verify pins at execute.
- **AI fold-in:** 2026-08-26 `agy-review.md` + `opencode-review.md` (HEAD `87919dd`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** Disposition **§13**. Fold-in DOCS TX `20060ded-80be-4a78-b10b-a7dd69e4f817`.
- **Ledger:** planning DOCS TX `4e15b2eb-cc78-40e0-aaf2-0dd362814c7e`. Fold-in DOCS TX `20060ded-80be-4a78-b10b-a7dd69e4f817`. Series mint DOCS `c62396f6`. Implement starts a **FEATURE** TX on **go** (src change in `run_update`).
- **Isolation:** Do **not** `daemon stop` / `cargo install` / `sc start` / edit `run_update` as planning. Do **not** print or commit `AI_BRAINS_KEY`. Never `git push origin main`.

---

## 1. Objective

1. **`ai-brains update` must not undo T222.** Live `run_update` installs the CLI with `cargo install --path crates/ai-brains-cli --locked` **without** `--features graph`. After T306, PATH CLI is graph-on. Calling today’s PATH `ai-brains update` would reinstall **graph-off**.
2. **PATH `ai-brainsd` must be the T305/T306 SQLCipher 4.14 writer.** Live binary is still **2026-08-22 14:48:10** (21,045,248 B) while CLI is T306 **2026-08-26 6:54:32**. Daemon is the WAL writer; SCM `AI-Brains-Daemon` ImagePath is the **same** PATH exe (`--service`). Replacing it on Windows requires stopping the **Running** interactive process ([cargo#3486](https://github.com/rust-lang/cargo/issues/3486)).
3. **Capture independence.** Install argv + operator PATH replace. No new events, no contracts DTO, no pin bumps, no models on the install path.

This unblocks daily ops honesty: T306 put 4.14 on the **CLI**; T222 put graph-on on local install scripts; T84 `update` still teaches the operator a command that undoes graph-on and leaves the **writer** on 4.10.

---

## 2. Live baseline (re-scan 2026-08-26)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Fold-in `87919dd` (T310 full-plan commit). Tree **CLEAN**. `origin/main...HEAD` **ahead 1**. Plan-write snapshot was `e577c8c` / **0/0** (Agy m1). |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` — **26,501,632** B; LastWriteTime **2026-08-26 6:54:32 AM**; `ai-brains 0.1.3`; doctor `graph_feature=available`; `cipher_page` **`cipher_version=4.14.0 community`**. |
| PATH `ai-brainsd.exe` | **21,045,248** B; LastWriteTime **2026-08-22 2:48:10 PM**. Older than CLI. `ai-brainsd --version` → `Error: Missing` (not a version flag — **do not add**). |
| `daemon status` | **Running** PID **48960**; vault `C:\dev\ai-brains\vault.db`; LLM/Embedding TCP Open. |
| SCM `AI-Brains-Daemon` | **Stopped** / Automatic. `BINARY_PATH_NAME` = `C:\Users\RyanB\.cargo\bin\ai-brainsd.exe --service`. Same file as PATH. |
| PATH `doctor --summary` | `degraded`. `graph_density` sparse E/N **0.411** remediator still `ai-brains graph rebuild` — **PATH-behind T308** (CLI mtime is T306, before T308 merge). `recovery_kit_event` warn. **Not this DoD.** |
| `run_update` | `daemon.rs:1034–1100`. CLI install `:1070–1071` **omits** `--features graph`. Daemon install `:1083–1084` `--path crates/ai-brainsd --locked` (no graph feature on that crate — **correct**). Then `run_start` (interactive spawn, **not** `sc start`). |
| `GRAPH_REINSTALL_SOOT` | `governed_common.rs:45–46` exact `cargo install --path crates/ai-brains-cli --locked --features graph`. **Do not edit the string.** |
| `Build-AIBrains.ps1` | Already CLI `--features graph` + copies `ai-brainsd.exe`. Alternative PATH path (T306 F16). **Not** the `update` hole. |
| T84 hermetic | `smoke.rs` `test_daemon_update_command_exists` — help surface only; **no** argv assert. |
| Last GitHub PR | [#227](https://github.com/Ryan-AI-Studios/AI-Brains/pull/227) T309. `gh pr view --json mergedAt` **2026-08-26T22:22:01Z** (`gh pr list` showed **22:02:39Z** — do **not** treat list time as merge; T306 pin). `pulls/227/comments`, `/reviews`, `issues/227/comments` all **`[]`**. **last-PR Cursor: N/A. No T311.** Open PRs: **none**. |
| rustc / Perl | **1.95.0** / Perl **v5.42.2** (openssl-src Configure). |
| Ledger | **0 pending / 0 drift** at scan (before this DOCS TX). |
| `ISSUES.md` | **Does not exist.** |
| Planning install / daemon stop | **Not run.** |

### 2.2 Why these residuals still matter

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| `run_update` graph-off CLI | PATH `ai-brains update` is the documented self-heal (T84). After T306 it is a **T222 High** if invoked. **DoD.** |
| PATH `ai-brainsd` 2026-08-22 | Single-writer + SCM ImagePath. T305 WAL-reset is not on that binary. Mixed CLI 4.14 / daemon 4.10 was **accepted T306 F8** until this track. **DoD.** |
| Chicken-egg | Today’s PATH `update` would graph-off the CLI **while** installing a 4.14 daemon. Must install **new** CLI first (SOOT), **then** `update` (or owner-confirm stop + `cargo install --path crates/ai-brainsd --locked`). **F10.** |
| Daemon has no `cipher_page` | Doctor probe is CLI-only. Do **not** add a 16th check or daemon pragma status. Proof is locked install from rusqlite **0.40.2** HEAD + PATH mtime. **F11.** |
| SCM Stopped | Interactive PID 48960 is the live writer. `run_start` already spawns detached interactive — **do not** `sc start`. **F12.** |
| PATH-behind T308 remediator | Side-effect of CLI reinstall, not a floor/remediator steal. **Decline as DoD.** |
| `ai-brainsd --version` Missing | Clap/service parse, not a version string. **Do not add `--version`.** |
| Binary `find` for `4.14.0 community` | Planning probe: **not** in either PATH exe as UTF-8. Unreliable. **Not a probe.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| `run_update` | `daemon.rs:1034–1100` | Stop (probe 300 ms → shutdown → taskkill/pkill) → CLI cargo → daemon cargo → `run_start`. |
| CLI argv | `:1070–1071` | `["install", "--path", "crates/ai-brains-cli", "--locked"]` — **missing** `--features graph`. |
| Daemon argv | `:1083–1084` | `["install", "--path", "crates/ai-brainsd", "--locked"]`. `ai-brainsd` has **no** `graph` feature (`crates/ai-brainsd/Cargo.toml`). Keep. |
| `run_start` | `daemon.rs:5–20` | Sibling `ai-brainsd.exe` or PATH; `spawn_daemon` detached. **Not** SCM. |
| SOOT | `governed_common.rs:45–46` | Human string. Smoke `tests/smoke.rs` ~3259 pins it to INSTALL. |
| CLI features | `ai-brains-cli/Cargo.toml` `default = []`; `graph = [...]` | Unchanged (T200 A2=no). |
| Help | `main.rs` `DaemonCommands::Update`; smoke T84 | Stay-green. |
| Hotspots | Plan-time `ledgerful hotspots`: `project.rs` #1; `governed_common.rs` **#3**. OpenCode audit `scan --impact` at `87919dd` reported `"hotspots": []`. | **F1 stands either way.** Keep argv slices in `daemon.rs`. Phase 0 re-run `ledgerful hotspots`. |

### 2.4 Dependency / standards research (2026-08-26) — snapshot, re-verify at execute

| Pin | Workspace / lock | crates.io / docs | Action |
|-----|------------------|------------------|--------|
| rusqlite | exact **0.40.2** / lock **0.40.2** (`23f2a97d…`) | **0.40.2** (`cargo search` 2026-08-26) | **No bump.** `--locked`. |
| libsqlite3-sys | lock **0.38.2** | via rusqlite | Unchanged. |
| clap | workspace **4.5** / lock **4.6.1** | clap **5 declined** | **No bump.** |
| tokio | **1.53** / **1.53.1** | — | **No bump.** |
| reqwest / tower-http | **0.13.4** / dual **0.7.0**+**0.6.11** | T307 Blocked | **No steal.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged. |
| workspace version | **0.1.3** | — | **No bump** (`--version` is not 4.14 proof). |
| New crates | — | — | **Zero.** |

**Cargo install** ([Cargo Book](https://doc.rust-lang.org/cargo/commands/cargo-install.html) 2026-08-26):

- `--path` **always** builds and installs (same `0.1.3` does not skip).
- `--locked` required so rusqlite stays **0.40.2**.
- `--features graph` is **not** implied by `default = []`. Two argv tokens `--features` + `graph` reconstruct `GRAPH_REINSTALL_SOOT`. Do **not** use `-F`.
- Windows: replacing a **running** target exe fails ([cargo#3486](https://github.com/rust-lang/cargo/issues/3486) os error 5). CLI install while **daemon** runs is OK (different file) — T306 F13. Daemon install needs the Running `ai-brainsd.exe` stopped — T84 `run_update` already stops it.

**SQLCipher:** T305/T306 / [Zetetic 4.14.0](https://www.zetetic.net/blog/2026/03/17/sqlcipher-4.14.0-release/) WAL-reset; `cipher_compatibility = 4`. Mixed 4.10 writer + 4.14 CLI was accepted until this track. No live encrypt.

**N/A (new API):** This is argv + operator install, same class as T84/T222/T306. Pattern: reconstruct SOOT in a unit; do not parse the human string in production.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until **go**. No `cargo install`, `daemon stop`/`start`, `sc start`/`stop`, `taskkill` as planning. |
| **F1** | `run_update` CLI cargo argv **reconstructs** `GRAPH_REINSTALL_SOOT` exactly: `install --path crates/ai-brains-cli --locked --features graph`. Do **not** edit the SOOT string. `pub(crate)` slices **`UPDATE_CLI_CARGO_ARGS`** and **`UPDATE_DAEMON_CARGO_ARGS`** live in `daemon.rs` (Agy O1). Do **not** grow `governed_common.rs` for this (F1 stands even if Phase 0 `hotspots` rank differs). |
| **F2** | Daemon cargo argv stays `install --path crates/ai-brainsd --locked`. **No** `--features graph` on `ai-brainsd`. |
| **F3** | Stop/restart policy stays T84 (`run_update` already stops then `run_start`). Do not add SCM start. |
| **F4** | Owner-confirm live PATH daemon replace on go (interactive daemon is Running). Planning does not stop it. |
| **F5** | Never `git push origin main`. No Dependabot remote merge. clap 5 declined. |
| **F6** | No `unwrap`/`expect`/`panic` in production. No new crates. No pin bumps. |
| **F7** | Capture independence: no events, no contracts DTO, doctor matrix stays **15**. |
| **F8** | Do **not** steal T307 / T308 floors / T309 (done). PATH-behind T308 remediator is **not** an AC. |
| **F9** | Do **not** call **today’s** PATH `ai-brains update` (graph-off CLI). |
| **F10 — Live sequence** | On go, after src is on HEAD: (1) `GRAPH_REINSTALL_SOOT` CLI install **first** (daemon may stay Running); (2) then PATH `ai-brains update` **or** owner-confirm `daemon stop` + `cargo install --path crates/ai-brainsd --locked` + `daemon start`. |
| **F11 — Daemon 4.14 proof** | No daemon `cipher_page`. AC2 = PATH `ai-brainsd.exe` LastWriteTime **after 2026-08-22 14:48:10** after F10 step 2 from this workspace `--locked`. Supporting: lock still rusqlite **0.40.2**. CLI `doctor --json` `cipher_page` still contains `4.14` and `graph_feature=available`. |
| **F12 — SCM** | Do **not** `sc start AI-Brains-Daemon`. Service stays Stopped unless the owner separately asks. ImagePath already PATH exe — replacing the file updates the service binary for a future start. |
| **F13 — Perl** | Phase 0 `perl -v`. Stop-Before if openssl-src Configure fails. |
| **F14 — File lock** | If CLI install fails because `ai-brains.exe` is locked, halt. Do **not** kill random `ai-brains` editor processes without asking. |
| **F15 — `ai-brainsd --version`** | Out of scope. |
| **F16 — Scripts** | `Build-AIBrains.ps1` already graph-on + daemon copy. **Not** required to change. Primary hole is `run_update`. |
| **F17 — Workspace version** | Stay **0.1.3**. |
| **F18 — Key leak** | Captured logs must not contain `AI_BRAINS_KEY` / `x'<64 hex>'`. |
| **F19 — Git cwd** | Install from `C:\dev\AI-Brains` workspace root. PowerShell `;` not `&&`. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit `run_update_cli_args__reconstruct_graph_reinstall_soot`: `format!("cargo {}", args.join(" ")) == GRAPH_REINSTALL_SOOT`. `run_update` uses those args (not a parallel literal). |
| **AC2** | After owner-confirm F10 step 2: PATH `ai-brainsd.exe` LastWriteTime **newer than 2026-08-22 14:48:10**. Lock rusqlite still **0.40.2**. |
| **AC3** | PATH `ai-brains doctor --json`: `graph_feature` message **`available`**; `cipher_page` message contains **`4.14`**. Filter those checks only (F18). |
| **AC4** | Unit `run_update_daemon_args__no_graph_feature`: `UPDATE_DAEMON_CARGO_ARGS` is exactly `["install", "--path", "crates/ai-brainsd", "--locked"]` (no `--features`, no `graph`). `run_update` uses that slice. |
| **AC5** | T84 smoke `test_daemon_update_command_exists` stay-green. Existing `run_start` / schedule units stay-green. |
| **AC6** | clippy `-D warnings` on `ai-brains-cli`; nextest that package (plus any new unit). |
| **AC7** | CHANGELOG Unreleased notes the graph-on `update` + daemon PATH 4.14 ops. |
| **AC8** | `conductor.md` T310 **Completed** on go with PATH mtime evidence; deferred T306 F8/F9 struck as done. |
| **AC9** | If owner does **not** confirm live daemon stop/install, AC2 is a **soft residual** (PATH-behind) and AC1/AC4–AC7 still close the src hole. Record that honestly. |

---

## 5. Design notes

### 5.1 Reconstruct, don’t parse

`GRAPH_REINSTALL_SOOT` is operator copy-paste. Production `Command::args` should be a `&[&str]` slice that a unit proves equals `"cargo " + join`. Parsing the human string in `run_update` is unnecessary.

### 5.2 Chicken-egg

PATH CLI still contains **old** `run_update`. F10 step 1 puts new `run_update` on PATH without touching `ai-brainsd.exe`. Step 2 uses the **new** CLI to stop the daemon and install both.

### 5.3 Why mtime is SoT for the daemon

T306 F25: CLI mtime is supporting; `cipher_page` is the 4.14 distinguisher. Daemon has no equivalent probe. T305 already proved this lock builds `4.14.0 community`. This track’s daemon proof is **locked rebuild of that lock** + newer PATH mtime.

### 5.4 Capture independence

No vault writes except T84’s existing restart (daemon process). No events. No models.

---

## 6. Non-goals

T307; clap 5; Cargo default graph-on; doctor 16th / daemon `cipher_page`; `ai-brainsd --version`; SCM start; T197 silent zero; T308 floor retune; T309 helpers; `Build-AIBrains.ps1` rewrite; live encrypt/rebuild; workspace 0.1.4.

---

## 7. Verification plan

TDD **red first** (unlike T309 — this **is** a distinguishable argv change):

1. **Red:** add `run_update_cli_args__reconstruct_graph_reinstall_soot` against today’s argv (fails: no `--features graph`).
2. **Green:** `UPDATE_CLI_CARGO_ARGS` includes `--features graph`; `run_update` uses it.
3. **AC4 lock:** `run_update_daemon_args__no_graph_feature` is stay-green on today’s daemon argv (OpenCode O2) — not a behavioral red.
4. Stay-green AC5/AC6.
5. Owner-confirm F10 live (AC2/AC3) or record AC9.

Do **not** require full workspace nextest to finish the **plan**. On go: targeted nextest + clippy; implement-track full gate before publish.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| PATH `update` before new CLI | **F9 / F10** |
| Graph-off CLI | **F1 / AC1 / AC3** |
| Windows file lock on `ai-brainsd.exe` | T84 stop already in `run_update`; **F4** |
| SCM accidentally started | **F12** |
| Fake 4.14 via mtime-only of a non-locked build | F10 requires `--locked` from this repo; Phase 0 re-check rusqlite **0.40.2** |
| openssl-src / Perl | **F13** |
| Key leak in doctor JSON dump | **F18 / AC3** filter |
| Owner refuses live stop | **AC9** src still ships |
| Growing `governed_common.rs` | **F1** keep argv in `daemon.rs` |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-26 (header through T142 residuals). Overlapping **open** rows:

| Item | Disposition |
|------|-------------|
| T306 F8 PATH `ai-brainsd` 4.10 | **Absorb** F4 / F10 / F11 / AC2 |
| T306 F9 `run_update` graph-off | **Absorb** F1 / F9 / AC1 |
| T309 R3 T310 placeholder | **Absorb** — this plan |
| T308 PATH remediator still rebuild | **Decline as DoD** — PATH-behind T308; may clear as F10 step 1 side-effect |
| T307 dual tower-http Blocked | **Not stolen** |
| T305 R2 `table_exists` | **Done T309** — not reopen |
| Floor retune 0.50 / projector | **Decline** |
| `recovery_kit_event` | **Not this track** |
| T197 daemon `AI_BRAINS_VAULT_KEY` silent zero | **Decline** — honesty residual, not install argv |
| clap 5 | **Decline** F5 |
| last-PR Cursor `#227` | **N/A empty** — `mergedAt` **22:22:01Z**; comments `[]`; **no T311** |
| T240 F2 / leftover `--write` / T263 H2 | **Decline** — standing |
| Cargo `default = []` / A2 | **Decline** — T200 |

---

## 10. Implement order (on go)

1. Phase 0: re-read `run_update` `:1070–1084` vs SOOT; lock rusqlite **0.40.2**; `perl -v`; daemon still Running; SCM still Stopped; `ledgerful hotspots`; FEATURE TX. **Do not install yet.**
2. Red AC1 test.
3. Green `UPDATE_CLI_CARGO_ARGS` + CHANGELOG. AC4 lock (`UPDATE_DAEMON_CARGO_ARGS`) may land green.
4. Targeted clippy/nextest AC5/AC6.
5. Owner-confirm F10 live → AC2/AC3, else AC9.
6. Conductor Completed + deferred. Phase 6: `track/T310-*` → PR → watch `CI` → squash-merge. Never `git push origin main`.

---

## 11. Soft residuals (post-close)

| Residual | Note |
|----------|------|
| AC9 PATH daemon still 2026-08-22 if no owner confirm | Honest PATH-behind |
| PATH-behind T308 remediator until CLI reinstall | Side-effect of F10.1 |
| SCM service remains Stopped | **F12** |
| `ai-brainsd --version` Missing | **F15** |
| T307 Blocked | Not stolen |
| `recovery_kit_event` | Not this track |
| T197 silent zero | Not this track |

---

## 12. Touch map

| Path | Role |
|------|------|
| `crates/ai-brains-cli/src/commands/daemon.rs` | `UPDATE_CLI_CARGO_ARGS` / `UPDATE_DAEMON_CARGO_ARGS`; AC1 + AC4 units |
| `crates/ai-brains-cli/src/commands/governed_common.rs` | **Read-only** SOOT |
| `CHANGELOG.md` | Unreleased |
| `C:\Users\RyanB\.cargo\bin\ai-brains.exe` | F10.1 ops (not in git) |
| `C:\Users\RyanB\.cargo\bin\ai-brainsd.exe` | F10.2 ops (not in git) |
| `conductor/conductor.md` | Pending → Completed **on go** |
| `conductor/deferred.md` | T306 F8/F9 done **on go** |
| `Cargo.toml` / `Cargo.lock` | **No bump** |
| `graph_density.rs` / doctor matrix | **No** |

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` + `opencode-review.md` (HEAD `87919dd`). Fold-in verify: `run_update` `daemon.rs:1034–1100` (`Ok(())` `:1099`); CLI argv `:1070–1071` omit `--features graph`; daemon argv `:1083–1084`; `run_start` `:5–20`; `GRAPH_REINSTALL_SOOT` `:45–46`; `#227` `mergedAt` **22:22:01Z**; comments `[]`; tree CLEAN ahead **1**.

### Pins locked by fold-in

1. **HEAD (Agy m1):** fold-in HEAD is `87919dd` / ahead **1**. Plan-write was `e577c8c` / 0/0.
2. **`#227` mergedAt (OpenCode m1):** `gh pr view` **2026-08-26T22:22:01Z**. List **22:02:39Z** is not merge.
3. **Line ranges (OpenCode m2):** `run_update` **`:1034–1100`**; `run_start` **`:5–20`**. Argv lines `:1070–1071` / `:1083–1084` unchanged.
4. **Argv names (Agy O1):** `UPDATE_CLI_CARGO_ARGS` / `UPDATE_DAEMON_CARGO_ARGS` in `daemon.rs`. F1 / no `governed_common` growth.
5. **AC4 unit (OpenCode O2):** `run_update_daemon_args__no_graph_feature` is a **lock**, not a red.
6. **Hotspots (OpenCode O1):** F1 stands if `scan --impact` reports no hotspots. Phase 0 re-runs `ledgerful hotspots`.
7. **last-PR Cursor `#227`:** N/A empty; **no T311.**

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** HEAD `e577c8c` / 0/0 | **Folded** §2.1 + plan preflight → `87919dd` / ahead **1** |
| Agy | **m2** F10 chicken-egg | **Already** F9 / F10 / plan Phase 0 |
| Agy | **O1** argv slices in `daemon.rs` | **Folded** F1 names `UPDATE_CLI_CARGO_ARGS` / `UPDATE_DAEMON_CARGO_ARGS` |
| Agy | **O2** reconstruct unit | **Already** AC1 |
| OpenCode | B / M | None filed |
| OpenCode | **m1** `#227` `mergedAt` 22:02:39Z → 22:22:01Z | **Folded** §2.1 / §9 |
| OpenCode | **m2** line drift `:1099`/`:19` → `:1100`/`:20` | **Folded** §2.1 / §2.3 |
| OpenCode | **O1** `scan --impact` hotspots empty | **Folded** §2.3 + Phase 0 re-check; F1 unchanged |
| OpenCode | **O2** AC4 dedicated unit | **Folded** AC4 lock `run_update_daemon_args__no_graph_feature` |
| both | last-PR Cursor empty; deferred map; no T311 | **Affirm** |

No Blockers/Majors to decline. No new placeholder. Do **not** edit `*-review.md`. Do **not** execute until go.

