# T297 — `daemon status` must contrast Stopped vs backend TCP Open

- **Track ID:** T297-DaemonVsLlm
- **Status:** **Planned** (Pending until **go**; not Placeholder)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `daemon status` **9/9** labeled Stopped + Open; still **friction** two truths. Placeholder minted with T285–T300 (`76c4db9`). T296 OpenCode m3 / F11 pointed here.
- **Depends on:** T199 ✅ keyless status; T249 ✅ Stopped `next:`; T85/T94 ✅ config TCP + retry; T281 ✅ nightly HTTP ≠ daemon TCP (do **not** unify probes)
- **Blocks / feeds:** Operators can tell `Status: Stopped` from `LLM backend … Open`. Device/replicate empty **T298**. Forget-list **T299**. Graph sparse **T300**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “daemon Stopped vs llama Open”; T281 closeout F27 “Daemon Stopped + port Open”; T296 F11 steal; T249 F4 last-line `next:` **as a freeze this track must keep** (contrast is **not** last)
- **Not absorbed (DoD):** Start/stop/install the live daemon; raise 750 ms; unify daemon TCP with HTTP `/health`; `--format` / JSON status; `sc query` / uptime; doctor 16th; T298–T300; T240 F2; clap 5 / rusqlite 0.40; DTO keys
- **Research date:** 2026-08-24 (plan dogfood HEAD `0132707` T296 `#212`. Product `src/` = T296. PATH **0.1.2** 2026-08-22 19:41 **has T249 next: + T85 TCP**, not this contrast. Live hole is **human Stopped vs Open**; **this session daemon is Running** — do not stop it.)
- **Ledger:** planning DOCS TX `3f147d91-b4f9-42b2-a8c3-8ea01dd1292d`. Implement starts a **BUGFIX** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** rewrite `.env` (T240 F2). Do **not** `daemon start` / `stop` / `install` / `uninstall`. Do **not** raise `NIGHTLY_STATUS_PROBE_TIMEOUT`. Do **not** grow hotspot `project.rs` / `sync.rs` / `governed_common.rs` / `forget.rs` / `doctor.rs` / `nightly.rs`. Do **not** import `probe_health` into `daemon.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** live `retention apply --confirm`, `graph rebuild`, leftover `rebind-path --write --yes`, or `safety sync` without `--dry-run`.

---

## 1. Objective

1. **Stopped + backend Open is one-line obvious on `daemon status`.** Live report prints `Status: Stopped` (or today `Running`) and `LLM backend … Open` on the same block. Operators think the AI-Brains daemon is serving the model. T281 labeled nightly HTTP vs daemon TCP. **This command** still has no contrast when IPC is down and a model port is listening.
2. **Keep T199/T249/T85/T94/T281 contracts.** Status exit **0**. Keyless liveness. `next: ai-brains daemon start` remains the **last** line when Stopped. Backend lines stay `{name} {addr} [{desc}]: {Open|Closed}`. TCP 5×100 ms + jitter frozen. No `--format`. No JSON. 750 ms not raised. Nightly contrast untouched.
3. **North star.** Capture independence: status/docs/parse only. No events. No models crate. No contracts DTO. No new crates. No pin bumps. Do **not** start the daemon to prove Stopped.

This unblocks daily ops honesty for the Windows-first vault: llama.cpp / Ollama listening on `:8081`/`:11434` is **not** `ai-brainsd`. T249 told operators how to start the daemon. T297 tells them Open is a **different process**.

---

## 2. Live baseline (re-scan 2026-08-24)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Plan dogfood `0132707` T296 squash `#212`. Tree **CLEAN**. `origin/main` = HEAD (`left-right` `0 0`). Branch `main`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. **Has T249 `next:` + T85 TCP Open/Closed.** **Does not have T297 contrast.** **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4123** (volatile). In-context **0/0/0**. Word **393**. Capture independence holds. |
| PATH `daemon status` (project dotenv) | **`Status: Running`**. Vault `C:\dev\ai-brains\vault.db` 145.5 MB. Memories **48639**. **`LLM backend 127.0.0.1:8081 [http://127.0.0.1:8081]: Open`**. **`Embedding backend 127.0.0.1:8083 [http://127.0.0.1:8083]: Open`**. `PID: 4536`. **No `next:`.** **No contrast.** Exit **0**. |
| PATH `daemon status --no-project-context` | **Running**. **`LLM backend 127.0.0.1:11434 [Ollama default :11434 (AI_BRAINS_MODEL_URL=unset)]: Open`**. Embedding default `:8080` **Closed**. Same PID. **Placeholder “llama.cpp HTTP” is machine-specific** — env-unset is Ollama **11434**. Frozen const must say **backend**, not llama.cpp. |
| Audit hole vs this session | 2026-08-22 audit: **Stopped** + `:8081` Open. **Today: Running + Open.** Contrast correctly **absent** while Running. Do **not** `daemon stop` to reproduce Stopped. Hermetic **units** lock the Stopped+Open arm. Manual is pass-with-observed-data. |
| `DaemonCommands::Status` | Unit variant. **No** `--format`. **No** `after_help`. Early-route `main.rs` **`:4133–4142`** before AppContext (T199). |
| Last GitHub PR | [#212](https://github.com/Ryan-AI-Studios/AI-Brains/pull/212) T296 (merged 2026-08-24T12:12:25Z). `gh pr view --comments`, `/reviews`, `/comments`, `issues/212/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, `#58` tower-http, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / hotspots | Hotspot **#1** `project.rs` (**3.897**) — **do not touch.** `sync.rs` #2. `governed_common.rs` #3. `context.rs` #4. `forget.rs` #5. `daemon.rs` **1188** — not top-10. `doctor.rs` not top-10. |
| Ledger | **0 pending / 0 drift** at scan (before this DOCS TX). |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Status Stopped + LLM Open, no sentence | T249 added `next: start`. Operators still read Open as “the daemon is up.” clig.dev: saying (just) enough + suggest next + ease of discovery — contrast belongs on the report, gated on the dual-truth, not always. **DoD.** |
| Placeholder `llama.cpp HTTP Open ≠ daemon` | **HTTP is a lie here** — `run_status` is `TcpStream::connect_timeout` (T85/T199). Nightly is HTTP `/health` (T281). **llama.cpp is a lie on `--no-project-context`** (Ollama `:11434`). Freeze **`backend TCP Open ≠ daemon`**. |
| Always print the contrast | clig just-enough. Noise when Running (expected Open) or Stopped+both Closed (no dual-truth). **Decline.** Gate on `!running && (llm_open \|\| embed_open)`. |
| Unify daemon to HTTP `/health` | T199/T281: TCP liveness ≠ HTTP ready. Kubernetes `tcpSocket` vs `httpGet`. llama.cpp `#20684` / closed PR `#20799`: `/health` can queue. HTTP would lie Closed during generation. **Decline.** |
| Raise 750 ms | T255 F18 / T281 F2. Nightly budget. **Decline.** Not this command. |
| JSON / `--format` | T249 F4 / T199 F12 / T266 Family B. **Decline.** Human additive only. Placeholder “JSON if any: additive” → there is **no** JSON surface. |
| Force live Stopped | Isolation + T249 F11 / F5. Units lock Stopped+Open. **Decline** live `daemon stop`. |
| Doctor `daemon_reachable` ok vs status Stopped | T295 mis-mapped this here. Doctor uses Safety 3×1000 ms; status uses Status 1×300 ms (T199 F5). **Not** LLM-Open friction. **Decline** F27. |
| `sc query` / uptime / PID invent | T199 F18 / T249 F5/F12. **Decline.** |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|--------|
| Early-route | `main.rs` **`:4133–4142`** | `DaemonCommands::Status` → `run_status` before AppContext. **Untouched.** |
| Clap | `DaemonCommands::Status` **`:3109`** | No flags. **Additive** `after_help` only (F20). |
| `run_status` | `daemon.rs` **`:706–805`** | IPC probe → Status line → vault section → **TCP loop** → PID → `status_next_line`. **Insert tail after PID, before/via composed helper.** |
| Next-step | `status_next_line` **`:696–703`** | `Some("next: ai-brains daemon start")` iff `!is_running`. **Keep exact string.** T249 AC7 units **`:818–828`**. |
| TCP probe | `daemon.rs` **`:739–778`** | 5 attempts × 100 ms + jitter. Prints `LLM backend` / `Embedding backend`. Capture `state == "Open"` bools. **Do not retune delays.** **Do not extract** the connect loop (T199 F19 accepted debt). |
| `resolve_backend` | `daemon.rs` **`:601–`** | `AI_BRAINS_MODEL_URL` default host `127.0.0.1` **port 11434** Ollama; embedding default **8080** llama.cpp. Live dotenv **8081/8083**. |
| T281 contrast (peer) | `nightly_status.rs` **`:25–34`** | `HTTP_VS_TCP_CONTRAST` / `completion_timeout_contrast_line`. **Do not import.** Copy the **pattern** (const + `Option<&'static str>` + U+2260). |
| Hermetic | `tests/daemon_status_vault_independence.rs` | Exit 0 no-key; Stopped omits vault. **If live Running, skip Stopped asserts.** Mirror for contrast (F28). |
| Smoke T85/T94/T128 | `tests/smoke.rs` **`:2901–3146` / `:3373`** | URL/port; Closed on dead port; delayed listener **Open**. T94 **`:3142–3145`** `contains("Open")` only — extra contrast line **allowed**. |
| CAPABILITIES | **`:110`** | Family B. “Stopped last line: `next: …`”. **Keep last-line.** Additive contrast **before** `next:`. |
| OPERATIONS | **`:552–558`** | Keyless + next-step. Additive backend-Open sentence. |
| PROTOCOL-COMPAT | no `daemon status` keys | **Untouched.** |
| `ISSUES.md` | — | Does not exist. |

### 2.4 Dependency / standards research (2026-08-24) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (docs.rs `Command::after_help` current). **No clap 5.** | **No bump.** Additive Status `after_help`. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** No JSON. |
| `rusqlite` | lock **0.39.0** + sqlcipher | crates.io **0.40.2** (Dependabot `#61`) | **No bump.** |
| `tokio` | workspace **1.52** / lock **1.52.3** | crates.io **1.53.1** (`#59`) | **No bump.** Status already async for IPC only. |
| `thiserror` | lock **2.0.18** | crates.io **2.0.20** (`#60`) | **No bump.** |
| `reqwest` | workspace **0.13** / lock **0.13.4** | models crate | **No CLI dep.** |
| rustc / edition | **1.95.0** / **2024** | toolchain | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.2** | — | **No bump.** |
| New crates | — | — | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Human-first; saying (just) enough; suggest next; `git status` shows state + hint | [clig.dev](https://clig.dev/) (fetched 2026-08-24) | Contrast on the dual-truth only. `next:` stays. Do not always-print. Human output may evolve; no JSON wire. |
| TCP open ≠ HTTP ready | [Kubernetes probes](https://kubernetes.io/docs/concepts/configuration/liveness-readiness-startup-probes) (current): `tcpSocket` succeeds if the port is open; `httpGet` needs HTTP status | Affirm T281 split. T297 contrasts **IPC vs TCP**, not HTTP vs TCP. |
| `/health` can miss a budget under load | [llama.cpp #20684](https://github.com/ggml-org/llama.cpp/issues/20684); PR [#20799](https://github.com/ggml-org/llama.cpp/pull/20799) **closed** (no special `/health` fast-path) | Do **not** switch daemon status to HTTP. Open TCP is the model process. |
| clap `after_help` | [docs.rs/clap/4.6.6 `Command::after_help`](https://docs.rs/clap/latest/clap/struct.Command.html) | Derive string on `Status`. No new flags. |
| T249 last line | CAPABILITIES `:110` + OPERATIONS `:558` | Contrast **must not** replace `next:` as last line. |

**N/A:** SQLCipher page encrypt, schtasks mutate, T180 DTO new keys, Windows service install, Safety GLOB, policy HINT, nightly Router HRESULT (T296 Completed).

**Could not verify:** live Stopped+Open on this session (daemon **Running** PID 4536). DoD does not depend on stopping it. `ledgerful ask` local model returned no snippets (HNSW dim 384 vs 768 warn) — `ledgerful search` **did** hit `status_next_line` `daemon.rs:697` / `run_status` `:706`.

**ledgerful / ai-brains:** `preflight --summary` pinned **4123** / 0/0/0; `recall` “daemon status Stopped…” returned T281 review-track dumps + onboard note T297 still Placeholder; `sync query` ledger hits T115/T144/T249 not this hole; `ledgerful ledger status --compact` 0 pending / 0 drift; `scan --impact` CLEAN at `0132707`; `hotspots` `project.rs` #1 — do not grow.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `3f147d91`. Implement starts a **BUGFIX** TX. |
| **F1 — Contrast const** | `pub(crate) const BACKEND_OPEN_NE_DAEMON: &str = "backend TCP Open ≠ daemon"` in `daemon.rs` (U+2260 `≠`, **not** ASCII `!=`). **Not** placeholder `llama.cpp HTTP Open ≠ daemon`. **Not** T281 `HTTP /health 750ms ≠ daemon TCP`. |
| **F2 — Gate** | Pure helper `status_backend_contrast_line(is_running: bool, llm_open: bool, embed_open: bool) -> Option<&'static str>` returns `Some(BACKEND_OPEN_NE_DAEMON)` iff `!is_running && (llm_open \|\| embed_open)`. One line even if **both** backends Open (F30). |
| **F3 — Running** | `is_running == true` → `None` even if both Open. Today’s live PATH report is this arm. |
| **F4 — Stopped + both Closed** | `None`. No dual-truth. |
| **F5 — Tail composition** | `status_report_tail(is_running, llm_open, embed_open) -> Vec<&'static str>`: contrast (if any) **then** `status_next_line`. When Stopped, **last** element is always `"next: ai-brains daemon start"` (T249 F4 / CAPABILITIES last-line). Running → empty vec. `run_status` prints the vec after the PID block. |
| **F6 — Existing chrome freeze** | Keep exact: `Status: Running` / `Status: Stopped`; backend `{name} {addr} [{desc}]: {state}`; vault T128/T199 rules; `PID:` only when tasklist yields one; `next:` exact string. Do **not** restyle Open/Closed. |
| **F7 — TCP probe freeze** | 5×100 ms + jitter **untouched**. Capture `llm_open` / `embed_open` from the existing `state` local. Do **not** extract `TcpStream::connect_timeout`. Do **not** add HTTP. |
| **F8 — No JSON / no `--format`** | Affirm T249 F4 / T199 F12 / T266 Family B. Placeholder “JSON if any: additive” → **N/A** (no surface). |
| **F9 — Do not unify HTTP** | Do **not** import `probe_health` / `LlamaCppProvider`. Do **not** print T281 contrast from daemon status. |
| **F10 — Do not raise 750** | Affirm T255 F18 / T281 F2. Nightly only. |
| **F11 — Do not start/stop/install** | Even after go: no live `daemon start`/`stop`/`install` unless the owner explicitly confirms that remediating action. Units + pass-with-observed-data are DoD. |
| **F12 — Pins / crates** | No clap 5, no rusqlite 0.40, no lock bumps, no new crates, workspace **0.1.2**. No CLI `reqwest`. |
| **F13 — PATH-behind** | Live `ai-brains` is **0.1.2** 2026-08-22. Do **not** `cargo install` unless the user asks. Tests/manual use `cargo run` / hermetic. |
| **F14 — last-PR Cursor** | `#212` empty. **No T301.** Dependabot `#61` rusqlite **not** this track. |
| **F15 — Module** | Const + helpers + units live in `daemon.rs` next to `status_next_line` (existing `status_vault_tests`). **Do not** new file. **Do not** grow `project.rs`. `run_status` dispatch only (~bool capture + print tail). |
| **F16 — Capture independence** | Status/docs only. No events. No models on this path beyond existing TCP. |
| **F17 — Decline peers** | T298–T300; leftover `--write`; T240 F2; T263 H2; T255 750 raise; T296 Router (Completed). |
| **F18 — U+2260** | AC must `contains('\u{2260}')` and `assert_ne!(BACKEND_OPEN_NE_DAEMON, "backend TCP Open != daemon")` (T281 AC1 analog). |
| **F19 — Docs** | CAPABILITIES `:110` additive (contrast **before** last-line `next:`). OPERATIONS `:558` additive. Root CHANGELOG T297 Unreleased. PROTOCOL-COMPAT **untouched**. CLI-EXIT-CODES status footnote already exit 0 both states — **no change required** unless a sentence is missing “Open is not Running”. |
| **F20 — after_help** | `DaemonCommands::Status` `after_help` one sentence: LLM/Embedding **Open** is TCP connect to the **model process**, not the AI-Brains daemon. No new flags. |
| **F21 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. No for-loop `#[test]`. `rstest` if ≥2 cases share one helper. |
| **F22 — Cross-model** | BUGFIX (human honesty). After Phase-1 review clean, run read-only `codex-review`. |
| **F23 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F24 — T249 last-line** | `next:` remains last when Stopped. Contrast is the line **immediately above** it when F2 fires. |
| **F25 — Stay-green** | T199 no-key hermetic; T85 URL/port; T94 delayed Open `contains("Open")`; T128 vault-when-running; `status_next_line` units. Extra contrast line on T94 Stopped+Open is **allowed**. |
| **F26 — Hotspots** | Do not touch `project.rs` / `sync.rs` / `forget.rs` / `doctor.rs` / `nightly.rs` / `nightly_status.rs`. |
| **F27 — Doctor IPC** | Do **not** reconcile `daemon_reachable` Safety vs status Status policy. Not this hole. |
| **F28 — No forced-Stopped hermetic** | Affirm T249 AC7: do **not** require a hermetic that asserts `Status: Stopped` against a live Running daemon. Optional hermetic: **if** stdout has `Status: Stopped` **and** `Open`, **then** it contains the const; **if** `Status: Running`, **then** it does **not**. |
| **F29 — Placement** | After backend prints + PID block; print `status_report_tail`. Do **not** insert between LLM and Embedding lines. |
| **F30 — Single line** | Both backends Open → still **one** contrast line. |
| **F31 — Wording** | Do **not** say HTTP. Do **not** say llama.cpp in the frozen const (Ollama default 11434). |
| **F32 — `status_next_line` freeze** | Signature and strings unchanged. Tail helper **calls** it. |
| **F33 — rustc / edition** | 1.95.0 / 2024. Unchanged. |
| **F34 — No live mutate** | No schtasks, no service, no `.env` write, no vault pin, no nightly run. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `status_backend_contrast_line(false, true, false) == Some(BACKEND_OPEN_NE_DAEMON)` |
| **AC2** | Unit: `status_backend_contrast_line(false, false, true) == Some(BACKEND_OPEN_NE_DAEMON)` |
| **AC3** | Unit: `status_backend_contrast_line(false, false, false).is_none()` |
| **AC4** | Unit: `status_backend_contrast_line(true, true, true).is_none()` |
| **AC5** | Unit: const contains U+2260; `assert_ne!` vs ASCII `!=` string; length/content freeze `backend TCP Open ≠ daemon` |
| **AC6** | Unit: `status_report_tail(false, true, false) == [BACKEND_OPEN_NE_DAEMON, "next: ai-brains daemon start"]` (last is `next:`). `status_report_tail(false, false, false) == ["next: ai-brains daemon start"]`. `status_report_tail(true, true, true).is_empty()` |
| **AC7** | Unit/hermetic: `daemon status --help` (or parent `daemon --help` if clap bubbles) stdout contains `TCP` and `daemon` from F20 after_help. Unknown extra flags still clap **2**. |
| **AC8** | Hermetic `daemon status` (no-key OK): exit **0**; **if** `Status: Stopped` and `Open` → stdout contains `BACKEND_OPEN_NE_DAEMON`; **if** `Status: Running` → stdout does **not** contain that const. Never require Stopped. |
| **AC9** | Existing T199 / T85 / T94 / T128 / `status_next_line__*` stay green |
| **AC10** | Manual (source bin, **no** `daemon stop`): `ai-brains daemon status` — record Status + LLM/Embedding states + presence/absence of const + `next:` last-when-Stopped. This plan-time: **Running** + 8081/8083 Open → const **absent**, no `next:`. Exit **0**. |
| **AC11** | Docs: CAPABILITIES `:110` still says last line `next:`; additive contrast-when-Stopped-and-Open. OPERATIONS additive. CHANGELOG T297. PROTOCOL-COMPAT untouched |
| **AC12** | No contracts DTO; no pin bumps; no new CLI dep; `llama_cpp.rs` / `nightly_status.rs` untouched; `scripts/dev-check.ps1` on go |
| **AC13** | `status_next_line(false) == Some("next: ai-brains daemon start")` still (T249 AC7) |
| **AC14** | T94 delayed-Open smoke still `contains("Open")`; must **not** `!contains` the new const (would flake if hermetic daemon is Stopped) |

---

## 5. Design notes

### 5.1 Human shape (Stopped + Open)

```
Status: Stopped
LLM backend 127.0.0.1:8081 [http://127.0.0.1:8081]: Open
Embedding backend 127.0.0.1:8083 [http://127.0.0.1:8083]: Open
backend TCP Open ≠ daemon
next: ai-brains daemon start
```

PID omitted when tasklist finds no `ai-brainsd.exe`. When PID is present it stays **above** the tail (existing order).

### 5.2 Helper sketch (in `daemon.rs`)

```rust
pub(crate) const BACKEND_OPEN_NE_DAEMON: &str = "backend TCP Open ≠ daemon";

pub(crate) fn status_backend_contrast_line(
    is_running: bool,
    llm_open: bool,
    embed_open: bool,
) -> Option<&'static str> { /* F2 */ }

pub(crate) fn status_report_tail(
    is_running: bool,
    llm_open: bool,
    embed_open: bool,
) -> Vec<&'static str> { /* F5 */ }
```

`run_status`: `let mut llm_open = false; let mut embed_open = false;` inside the existing for-loop when `state = "Open"`. After PID: `for line in status_report_tail(...) { println!("{line}"); }` — replace the current `if let Some(line) = status_next_line`.

### 5.3 Why not “llama.cpp HTTP”

Placeholder allowed that string. Live `--no-project-context` is **Ollama :11434**. Status probe is **TCP**, not HTTP. T281 already owns HTTP vs TCP on **nightly**. A second “HTTP Open” line on daemon status would **undo** T281.

### 5.4 Why not always print

Running + Open is the healthy machine (this session). Always-on contrast would train operators to ignore it. Gate on the dual-truth only.

---

## 6. Non-goals

- `daemon start` / `stop` / `install` / service detect / `sc query`
- Raise 750 ms / persist probe / doctor 16th / product `.cmd`
- Unify daemon TCP with HTTP `/health` (T281 freeze)
- `--format json` / daemon status DTO / TTY switch
- Edit `nightly_status.rs` / T281 const
- Retune TCP 5×100 ms
- clap 5 / rusqlite 0.40 / workspace 0.1.3
- T298–T300 / leftover `--write` / T240 F2 / T263 H2
- `cargo install`

---

## 7. Verification plan (TDD)

**Red first (must fail on current tree):**

1. `status_backend_contrast_line__stopped_llm_open__frozen_const`
2. `status_backend_contrast_line__stopped_embed_open__frozen_const`
3. `status_backend_contrast_line__stopped_both_closed__none`
4. `status_backend_contrast_line__running_open__none`
5. `backend_open_ne_daemon__uses_u2260_not_ascii`
6. `status_report_tail__stopped_open__contrast_then_next`
7. `status_report_tail__stopped_closed__next_only`
8. `status_report_tail__running__empty`
9. `daemon__help__status_names_backend_tcp` (after_help)

**Then green:** const + helpers + `run_status` bool capture + tail print + after_help + docs.

**Stay-green:** AC9 / AC13 / AC14.

**Manual:** AC10 classify-only. Pass-with-observed-data if Status is Running (const absent is **correct**).

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Live daemon Running hides Stopped+Open | F11/F28/AC6 units are SoT. Manual records Running. |
| T94 Stopped+Open grows a new line | AC14 forbids `!contains(const)`. T94 only requires `Open`. |
| CAPABILITIES “last line next:” broken | F5/F24/AC6 last element. |
| Saying HTTP / llama.cpp | F31 + live 11434 evidence. |
| Growing `daemon.rs` (1188) | F15 helpers in same file; no new module; do not peel TCP loop. |
| Hotspot `project.rs` | Do not touch. |
| Operators equate T281 “daemon TCP” with this “backend TCP” | Docs: T281 = nightly HTTP vs this command’s Open line; T297 = Open line vs IPC Status. |

---

## 9. Deferred absorb / decline

**Entire `conductor/deferred.md` scanned** (T142 archive through T296 closeout + T285–T300 mint). Overlapping open rows:

| Item | Disposition |
|------|-------------|
| Audit / mint “daemon Stopped vs llama.cpp `:8081` Open” | **Absorb** F1–F6 / AC1–AC6 / AC10 |
| Placeholder Manual `daemon status` — do not start daemon | **Absorb** AC10 / F11 |
| T281 closeout “Daemon Stopped + port Open” F27 | **Absorb** (this track) |
| T296 F11 / OpenCode m3 “daemon Stopped is T297” | **Absorb** (this track) |
| T249 F4 last-line `next:` / no JSON / no `--format` | **Affirm** F5 / F8 / F24 |
| T249 F5 / F11 no live start-stop / no sc query | **Affirm** F11 |
| T249 F12 daemon json / uptime / sc query | **Decline** F8 / F17 — still soft there |
| T199 F1/F2 keyless / F8 exit 0 / F19 TCP debt | **Affirm** F6 / F7 |
| T199 F12 JSON leftover | **Affirm decline** F8 |
| T255 F18 / T281 F2 raise 750 | **Decline** F10 |
| T281 F10 unify HTTP | **Decline** F9 |
| T255 doctor 16th / persist / `.cmd` | **Decline** F17 |
| PATH until `cargo install` (T285–T296) | **Residual** F13 |
| T298–T300 | **Decline** F17 |
| T294 leftover `--write` | **Decline** — Completed `#210`; live 5 roots still F11 there |
| T295 doctor remediator / T277 engine | **Decline** — Completed |
| T296 Router HRESULT | **Decline** — Completed `#212` |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F12 / F17 |
| last-PR Cursor **#212** | **N/A empty** — **no T301** F14 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Not this track** — T258 / leftover data |
| Doctor `daemon_reachable` vs status Stopped (T295 note) | **Decline** F27 — probe-policy, not LLM Open |
| Closed T85/T94/T128/T199/T249/T281 DoDs | **Stay closed** |

---

## 10. Implement order (on go)

1. Phase 0 re-verify (plan.md) + BUGFIX TX.
2. Red AC1–AC7 units.
3. Green const + helpers + `run_status` tail + after_help.
4. Docs CAPABILITIES / OPERATIONS / CHANGELOG.
5. Hermetic AC8 + stay-green AC9/AC13/AC14.
6. Manual AC10 (read-only; **no** daemon stop).
7. `scripts/dev-check.ps1`; Phase-1 review; `codex-review`.
8. conductor Completed + deferred closeout + pin.
9. Phase 6 publish (`track/T297-*` → PR → watch GHA `CI` green → squash-merge). Never `git push origin main`.

---

## 11. Soft residuals

| Residual | Notes |
|----------|--------|
| PATH until `cargo install` | F13 — source/hermetic SoT |
| Live daemon Running (PID 4536) | Honest; Stopped+Open is unit-locked |
| Doctor Safety vs Status 1×300 ms | F27 |
| T249 F12 daemon `--format json` / uptime / `sc query` | Still declined |
| `--no-project-context` Ollama `:11434` Open | Expected default; const says backend |
| T298–T300 | Next placeholders |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/daemon.rs` | F1–F5 helpers + units; `run_status` bools + tail print |
| `crates/ai-brains-cli/src/main.rs` | `DaemonCommands::Status` `after_help` + AC7 help unit (or sibling test module) |
| `crates/ai-brains-cli/tests/daemon_status_vault_independence.rs` | AC8 optional if/Running skip |
| `Docs/CAPABILITIES.md` | `:110` additive |
| `Docs/OPERATIONS.md` | `:558` additive |
| `CHANGELOG.md` | T297 Unreleased |
| `conductor/conductor.md` / `deferred.md` / this spec+plan / README-T285-T300 | Planning now; Completed on go |

**Do not touch:** `nightly.rs` / `nightly_status.rs`; `doctor.rs`; `project.rs`; `llama_cpp.rs`; `embeddings.rs`; `ai-brains-contracts`; `Cargo.lock`; PROTOCOL-COMPAT; TCP attempt/timeout literals.

---

## 13. AI fold-in

Empty until `/review-track T297` + `/fold-in T297`. Do **not** edit `*-review.md` in this pass.
