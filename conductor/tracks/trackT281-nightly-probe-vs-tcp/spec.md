# T281 — Nightly Completion timeout vs daemon Open must be one-line obvious

- **Track ID:** T281-NightlyProbeVsTcp
- **Status:** **Planned** (Pending until **go**; Placeholder upgraded 2026-08-22)
- **Category:** OPS / UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — friction: `nightly --status` Completion `probe=timeout (750ms)` while `daemon status` LLM **Open**. Nightly itself scored **8/8** after T269. Placeholder minted with T274–T284.
- **Depends on:** T269 ✅ heading + `timeout (750ms)` + after_help TCP/`/health`; T255 F18 freeze (do **not** raise 750 ms); T247 ✅ `--quick` + parallel 750 ms; T199 ✅ daemon TCP
- **Blocks / feeds:** Operators can tell HTTP `/health` budget-timeout from TCP liveness without reading `--help`. `context --show` leftover **T282**. `project list` cwd-first **T283**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “nightly Completion timeout vs daemon Open (750 ms not raised)”; T269 closeout “operator still sees two truths on the status block”
- **Not absorbed (DoD):** Raise 750 ms; unify daemon TCP with HTTP `/health`; JSON `probe_budget_ms` / contrast field; doctor 16th; persist probe; product `.cmd` / schedule-Router; `--quick --no-vault`; TCP-probe from nightly status; T282/T283; T240 F2; leftover rebind; clap 5; rusqlite 0.40; DTO keys; live schtasks mutate
- **Research date:** 2026-08-22 (plan dogfood HEAD `d89f5e6` T280 `#196`; product `src/` = T280; nightly last product commit T269 `9008074` `#186`)
- **AI fold-in:** none yet (plan pass). Disposition after `/fold-in`.
- **Ledger:** planning DOCS TX `b9b8c77d-3a92-476d-9887-1b7dfeed7fe2`. Implement starts a **BUGFIX** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** write live `.env` (T240 F2). Do **not** raise `NIGHTLY_STATUS_PROBE_TIMEOUT`. Do **not** mutate `AI-Brains-Nightly` or `AI-Brains-Router`. Do **not** import `probe_health` into `daemon.rs`. Do **not** grow hotspot `project.rs` / `sync.rs` / `forget.rs` / `daemon.rs` / `doctor.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** live `policy bootstrap`, `safety sync` without `--dry-run`, `retention apply --confirm`, or `graph rebuild`.

---

## 1. Objective

1. **Timeout vs Open is one-line obvious on the status block.** T269 labeled `probe=timeout (750ms)` and documented HTTP `/health` vs daemon TCP in `after_help` + OPERATIONS. Operators who run `--status` still see two truths with no contrast *on that output*. When Completion raw probe is `timeout`, print the next line `HTTP /health 750ms ≠ daemon TCP`.
2. **Keep T247/T255/T269 contracts.** Status exit **0**. JSON `schema_version` **1** and probe tokens `ok|down|timeout|error|skipped` frozen. Human timeout suffix stays `timeout (750ms)`. `--quick` stays `probe=skipped` (no contrast, no HTTP). 750 ms parallel probes frozen. Daemon status stays TCP. No CLI `reqwest`. No live task mutate.
3. **North star.** Capture independence: status/docs/parse only. No events. No models on `--quick`. No contracts DTO. No new crates. No pin bumps.

This unblocks daily ops honesty for the Windows-first vault: the nightly job can succeed while llama.cpp queues `GET /health` (busy slot) and `daemon status` still reports **Open** because the port is listening. Help text is not the remediator operators read when confused.

---

## 2. Live baseline (re-scan 2026-08-22)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | **Plan dogfood:** `d89f5e6` T280 squash `#196`. Tree **CLEAN**. `origin/main` = HEAD (`git rev-list --left-right --count origin/main...HEAD` = `0 0`). Nightly product last touched T269 `9008074` `#186`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-21 05:55**, 25 368 576 bytes, **0.1.1**. **T270** on PATH (after T269). T269 heading + `timeout (750ms)` **are** on PATH. Contrast line is **not**. **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **3581** (volatile). In-context **0/0/0**. Grants **0 of 3** (T275 hermetic; live not bootstrapped). Capture independence holds. |
| PATH `nightly --status` (full) | `Nightly: AI-Brains-Nightly`. Last Result **0**. Last scheduled **8/22/2026 3:00:01 AM**. Last nightly run **2026-08-22T07:07:45Z**. Sessions summarized last run **4**. Completion `127.0.0.1:8081  model=gemma-4-E4B-it-Q6_K.gguf  probe=ok`. Embedding `probe=ok`. Router **267009**. **This session did not reproduce timeout** — dual-truth is load-dependent. Contrast line **absent** (correct while `ok`; the hole is the timeout arm). |
| PATH `nightly --status --format json --quick` | `schema_version` 1. `completion.probe` / `embedding.probe` = `"skipped"`. Frozen keys present. `router.last_result` `"267009"`. |
| `ai-brains daemon status` (same moment) | **Stopped** + `next: ai-brains daemon start`. LLM `127.0.0.1:8081` **Open**. Embedding `127.0.0.1:8083` **Open**. TCP Open can coexist with nightly HTTP timeout (T269 live hole 2) and with daemon Stopped (process vs port). |
| ledgerful doctor `:8081` | Completion model **unreachable** (`/v1/chat/completions` network error) while nightly `/health` was **ok**. Third probe class (chat vs health vs TCP) — **not this DoD** (no doctor 16th). |
| `nightly --help` after_help | Already: `probe=timeout is HTTP /health within 750 ms. daemon status Open is TCP connect.` T269 AC6 **green**. Operators running `--status` do not see this sentence. **That is the remaining hole.** |
| Last GitHub PR | [#196](https://github.com/Ryan-AI-Studios/AI-Brains/pull/196) T280 (2026-08-22). `gh pr view --comments`, `/reviews`, `/comments`, `issues/196/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, actions). **No leftover to mint. No T285.** |
| Prior #188 Bugbot | **T284 Completed** `#193`. Not this track. |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081). **0 pending / 0 drift.** Hotspot **#1** `project.rs` (displayScore **3.926**, volatile). `sync.rs` #2. `forget.rs` #3. `governed_common.rs` #4. `context.rs` #5. `nightly.rs` **2133** / `nightly_status.rs` **638** — not top-5. `daemon.rs` **1188**. **Do not grow hotspots.** |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Status block has no HTTP vs TCP line | T269 put the contrast in `--help` + OPERATIONS. `--status` is the daily surface. clig.dev: human-first + saying (just) enough + ease of discovery — do not hide the remediator in help when two probes disagree. **DoD.** |
| Timeout vs Open this session | Completion was **ok**. Hole is intermittent (busy llama.cpp queues `/health`). DoD is the timeout **arm**, not forcing load. Manual AC is pass-with-observed-data. |
| Raise 750 ms | T255 F18 / T247 `<1.5s` status. llama.cpp #20684: `/health` queued like inference; raising 750→2000 still fails a 30 s generation. **Decline.** |
| Unify daemon to HTTP `/health` | T199 daemon is TCP liveness (port listening). HTTP would wait on a busy slot and make `daemon status` lie as “Closed” during generation. Kubernetes: `tcpSocket` ≠ `httpGet`. **Decline.** |
| TCP-probe from nightly to print “Open” | Extra I/O + latency on `--status`; couples two commands; would claim Open when we did not ask daemon. Contrast is **mechanism**, not a live TCP result. **Decline.** |
| Always print the contrast (even when `ok`) | clig “saying (just) enough”. Noise when both probes agree. **Decline.** Gate on raw `== "timeout"`. |
| JSON `probe: "timeout (750ms)"` or contrast key | T255 AC4 / F5 frozen tokens + `FROZEN_KEYS`. Scripts already have `"timeout"`. **Decline.** |
| Embedding-only timeout contrast | Audit hole is Completion vs LLM Open. T269 already suffixes embedding timeout `(750ms)`. Same helper can serve later. **Decline as DoD** (soft residual). |
| Doctor 16th model-port check | T255 F11. ledgerful `:8081` chat ≠ nightly `/health` is a third class. **Decline.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|--------|
| Status human print | `nightly.rs` **`:156–224`** | Banner, heading, schedule, counts, then Completion + Embedding `println!` via `format_endpoint_line`. **Insert contrast after Completion `println!` (`:195–203`).** |
| `--quick` skip | `nightly.rs` **`:52–66`** | Literal `"skipped"` — no `LlamaCppProvider`. Contrast helper sees `skipped` → `None`. |
| Probe timeout const | `nightly.rs` **`:13`** | `NIGHTLY_STATUS_PROBE_TIMEOUT` = **750 ms**. Run-path `:11` = **2s**. **Do not change either.** |
| Human suffix | `nightly_status.rs` **`:11–17`** | `format_probe_label_human` exact `== "timeout"` → `timeout ({ms}ms)`. **Do not restyle.** Units `:383–397`. |
| Heading const | `nightly_status.rs` **`:8`** | `NIGHTLY_TASK_HEADING`. Untouched. |
| JSON builder | `nightly_status.rs` `build_nightly_status_json` + `FROZEN_KEYS` **`:260–279`** | `completion.probe` is the raw label. Timeout fixture `:400–408` asserts `"timeout"` without ` (750ms)`. |
| `format_endpoint_line` | `nightly.rs` **`:809–817`** | 4-arg frozen (T269 F8). Example `Completion: 127.0.0.1:8081  model=…  probe=ok`. **Keep signature.** Contrast is a **separate** `println!`. |
| Probe implementation | `ai-brains-models/src/llama_cpp.rs` **`:127–140`** | `GET /health` then `/v1/models`. Map 200→Ok, connect fail→Down, timeout→Timeout. **Do not edit this crate.** |
| Daemon TCP | `daemon.rs` **`:749`** | `TcpStream::connect_timeout` 100 ms × 5 with backoff → **Open**. Default env fallback is Ollama `:11434`; live 8081 is dotenv. **Do not import `probe_health`.** |
| clap after_help | `main.rs` **`:1137`** | Already TCP + `/health` + 750. Unit `nightly__help__names_nightly_heading_and_probe_budget` **`:658`**. **Freeze.** |
| Hermetic `--quick` | `tests/nightly_status.rs` **`:77–112`** | Heading + `probe=skipped` + no `(750ms)`. **Extend:** must **not** contain F1 contrast. |
| JSON `--quick` | `tests/nightly_status.rs` **`:116–131`** | `probe == "skipped"`. Stay green. |
| Hotspots | `project.rs` **#1** (1472) | **Do not touch.** Helpers in `nightly_status.rs`. `nightly.rs` dispatch only (~3 lines). |
| Docs | `Docs/CAPABILITIES.md` **`:439`**; `Docs/OPERATIONS.md` **`:680/:689`** | Already explain HTTP vs TCP in prose. **Additive:** the contrast **line prints on timeout**. CHANGELOG T281 on go. |
| `ISSUES.md` | — | Does not exist. |

### 2.4 Dependency / standards research (2026-08-22) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). **No clap 5.** | **No bump.** after_help freeze. No new flags. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** JSON keys frozen. |
| `chrono` | lock **0.4.44** | crates.io **0.4.45** (Dependabot #62 open) | **No bump.** |
| `rusqlite` | lock **0.39.0** + sqlcipher + backup | crates.io **0.40.2** (Dependabot #61) | **No bump.** |
| `uuid` | lock **1.23.1** | crates.io **1.25.0** (2026-08-22) | **No bump.** |
| `tokio` | workspace **1.52** / lock **1.52.3** | crates.io **1.53.1** (Dependabot #59) | **No bump.** |
| `reqwest` | workspace **0.13** / lock **0.13.4** | models crate only | **No CLI dep.** |
| rustc / edition | **1.95.0** / **2024** | workspace toolchain | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | — | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| `/health` can miss a 750 ms budget under load | [llama.cpp #20684](https://github.com/ggml-org/llama.cpp/issues/20684) (opened 2026-03-17; still labeled bug as of 2026-08-22 crawl). `/health` has no special fast-path — queued behind inference. T269 cited PR #20817 as closer; **this pass could not re-fetch #20817** (GitHub page load failed). Raising 750 ms does not fix a busy slot. | Label the mechanism on the status block. Do **not** raise 750. |
| TCP open ≠ HTTP ready | [Kubernetes probes](https://kubernetes.io/docs/concepts/configuration/liveness-readiness-startup-probes) (current): `tcpSocket` succeeds if the port is open; `httpGet` requires an HTTP status. Same split as daemon TCP vs nightly `/health`. | Do not unify. Print the contrast. |
| Human-first; saying (just) enough; ease of discovery; `git status` shows state + hint | [clig.dev](https://clig.dev/) (current) | Contrast belongs on `--status` when timeout, not only in `--help`. Do not print when `ok`. |
| clap `after_help` | docs.rs/clap/4.6.6 `Command::after_help` | Keep derive string. T269 AC6 already locks needles. |
| 267009 is success | Microsoft Learn `SCHED_S_TASK_RUNNING` `0x00041301` | Router line frozen (T255/T269). Not this chrome. |

**N/A:** SQLCipher page encrypt, schtasks mutate, T180 DTO new keys, Windows service install, Safety GLOB (T279 Completed), policy HINT (T280 Completed).

**Could not verify:** whether `C:\llm\router.bat` llama.cpp includes a `/health` fast-path (operator binary). DoD does not depend on that upgrade. Live timeout vs Open **this session** (Completion was `ok`). GitHub HTML for #20684/#20817 bodies (search snippet + T269 citation stand).

**ledgerful / ai-brains:** `preflight --summary` 0 of 3 grants @ **3581** pins; PATH `nightly --status` Completion `ok` + daemon LLM Open (Stopped); `ledgerful ledger status --compact` 0 pending / 0 drift; `search "format_probe_label_human"` → `nightly_status.rs:11` / `nightly.rs:187`; `search "NIGHTLY_STATUS_PROBE_TIMEOUT"` → `nightly.rs:13`; `scan --impact` CLEAN at `d89f5e6`; `hotspots --json --limit 5` `project.rs` #1 — do not grow. Recall of “HTTP /health 750ms daemon TCP” returned T269 review-track dumps (PATH-behind T274).

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `b9b8c77d`. Implement starts a **BUGFIX** TX. |
| **F1 — Contrast line (human, timeout only)** | Const `pub(crate) HTTP_VS_TCP_CONTRAST: &str = "HTTP /health 750ms ≠ daemon TCP"` in `nightly_status.rs` (**31** chars; U+2260 `≠`). Pure helper `completion_timeout_contrast_line(raw_label: &str) -> Option<&'static str>` returns `Some(HTTP_VS_TCP_CONTRAST)` iff `raw_label == "timeout"` (exact token, **not** `contains`, **not** case-fold — T269 F27 analog). `"skipped"`, `"ok"`, `"down"`, `"error"`, `""`, `"TIMEOUT"`, `"timeout-ish"` → `None`. After the Completion `format_endpoint_line` `println!`, `if let Some(line) = completion_timeout_contrast_line(completion_label) { println!("{line}"); }`. Do **not** append to the `probe=` token. Do **not** print “Open”. Do **not** TCP-probe. |
| **F2 — Do not raise 750 ms** | Affirm T255 F18 / T269 F4. `NIGHTLY_STATUS_PROBE_TIMEOUT` stays 750 ms. Run-path stays 2s. |
| **F3 — JSON frozen** | `schema_version` **1**. `FROZEN_KEYS` unchanged. Probe tokens `ok\|down\|timeout\|error\|skipped`. No `probe_budget_ms`. No contrast field. Timeout still serializes `"timeout"` without ` (750ms)`. |
| **F4 — `--quick`** | Still `probe=skipped`. No HTTP. No `(750ms)`. No F1 line. |
| **F5 — T269 suffix freeze** | `format_probe_label_human` unchanged. Human Completion on timeout stays `probe=timeout (750ms)` **plus** the next-line F1. |
| **F6 — `format_endpoint_line` signature freeze** | Still 4 args (T269 F8). Contrast is a separate `println!`. |
| **F7 — after_help freeze** | T269 AC6 needles (`AI-Brains-Nightly`, `267009`/`SCHED_S_TASK_RUNNING`, `750`, `TCP`, `/health`) stay. **No rewrite** unless a typo is found at go (none today). |
| **F8 — Router / heading / schedule freeze** | `NIGHTLY_TASK_HEADING`, `format_router_status_lines`, `format_status_schedule_block` untouched. |
| **F9 — Module** | Const + helper + their units live in `nightly_status.rs`. `nightly.rs` prints ~3 lines. **Do not** move T229/T247 tests out of `nightly.rs`. **Do not** grow `project.rs` / `sync.rs` / `forget.rs` / `daemon.rs` / `doctor.rs`. |
| **F10 — Decline daemon HTTP** | `daemon status` stays T199 TCP. Do not import `probe_health`. |
| **F11 — Decline doctor 16th / persist probe / embed sleep / wrapper / `--no-vault`** | T255 F11–F15 stand. |
| **F12 — Pins / crates** | No clap 5, no rusqlite 0.40, no chrono 0.4.45, no uuid 1.25, no tokio 1.53, no new crates, workspace **0.1.1**. No CLI `reqwest`. |
| **F13 — PATH** | Do not `cargo install` unless the user asks. Tests/manual use `cargo run` / hermetic. |
| **F14 — Contracts** | No DTO. PROTOCOL-COMPAT untouched. E1 empty/timeout shapes frozen. |
| **F15 — Capture independence** | Status/docs only. No events. No `sync_state` probe write. |
| **F16 — Stop-before live mutate** | Even after go: do not unschedule/reschedule Nightly, do not write `.cmd`, do not touch Router registration, do not load llama.cpp to force timeout. |
| **F17 — Decline peers** | T282 `context --show`; T283 list cwd-first; leftover rebind; T240 F2; T255 750 ms raise; T263 H2; T266 JSON freeze for nightly pipes; T275 live bootstrap; T277 live `--no-prune`; T278 live rebuild; T279 live pin; T284 live apply. |
| **F18 — last-PR Cursor** | #196 empty → **N/A**. #188 closed by T284. Dependabot `#61` rusqlite **not** this track. **No T285.** |
| **F19 — Docs** | CAPABILITIES T269 bullet: additive “on human timeout, next line is `HTTP /health 750ms ≠ daemon TCP`”. OPERATIONS: same sentence under Completion/Embedding. Root CHANGELOG T281 row. CLI-EXIT-CODES unchanged (status still exit **0**). Skill one-liner if nightly `--status` section exists. |
| **F20 — Exit 0** | Unchanged. Timeout / 267009 / down / daemon Stopped are still success for `--status`. |
| **F21 — Tests** | Naming `function_or_feature__condition__expected_result`. Units for F1 const + helper rstest. Existing T247/T255/T269 units stay green. Hermetic `--quick` must not contain F1. No `unwrap`/`expect`/`panic` in production. |
| **F22 — Cross-model** | Honesty UX on the status path (easy T269 regression). After Phase-1 review clean, run read-only `codex-review`. |
| **F23 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F24 — PowerShell** | `;` not `&&`. |
| **F25 — Unicode** | Production contrast uses `≠` (U+2260), matching this spec. Tests must use that character. ASCII `!=` is not the freeze. Source files are UTF-8. |
| **F26 — Embedding-only timeout** | Not DoD. Helper is one-label so it can be reused later. Do **not** print a second identical line. |
| **F27 — Do not claim Open** | F1 never includes `Open`. Daemon may be Stopped with port Open (this session). Mechanism only. |
| **F28 — Classify-only live** | Manual AC uses `cargo run -p ai-brains-cli -- nightly --status` from this repo. Do **not** treat PATH T270 as proof of the new line. Do **not** mutate tasks. Do **not** generate llama.cpp load. |
| **F29 — Existing tests stay green** | T229 truncate; T247 `--quick` / LIST /V / schedule `lines[1]`; T255 JSON keys / Router 267009 / default human pipes; T269 heading / suffix / after_help AC6 / AC8 hermetic. |
| **F30 — models crate freeze** | `llama_cpp.rs` `probe_health` untouched. CLI presentation only. |
| **F31 — Identity leftover** | `7d97a456` vs `fcb8a40f` is T258/T276/T282. **No T285.** |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `HTTP_VS_TCP_CONTRAST` `assert_eq!` F1 literal (**31** chars); contains `/health` and `750ms` and `daemon TCP`; contains `≠` (U+2260). **Required red** (`http_vs_tcp_contrast__equals_frozen_line`). |
| **AC2** | Unit (rstest `#[case]`): `completion_timeout_contrast_line("timeout") == Some(HTTP_VS_TCP_CONTRAST)`. `None` for `"skipped"`, `"ok"`, `"down"`, `"error"`, `""`, `"TIMEOUT"`, `"timeout-ish"`. **Required red.** |
| **AC3** | Existing T269: `format_probe_label_human("timeout", 750) == "timeout (750ms)"`; pass-through cases unchanged. |
| **AC4** | Existing T255 JSON timeout fixture: `completion.probe == "timeout"` **without** ` (750ms)` and **without** F1 text in JSON. `FROZEN_KEYS` still present. |
| **AC5** | Existing T255: `format_router_status_lines(true, Some("Running"), Some("267009"))` first line still `Router: Running  last result: 267009`. |
| **AC6** | Existing T269 AC6: `nightly --help` still contains `AI-Brains-Nightly`, `267009` or `SCHED_S_TASK_RUNNING`, `750`, `TCP`, `/health`. |
| **AC7** | All-OS hermetic `tests/nightly_status.rs` `--status --quick`: heading + `probe=skipped`; does **not** contain `(750ms)`; does **not** contain `HTTP /health` and does **not** contain `daemon TCP`. Exit **0**. Extend T269 AC8. |
| **AC8** | Existing T247: `format_status_schedule_block` still has `Last task result: 101` as today (do not insert contrast into the vec). |
| **AC9** | Existing T255 JSON `--quick`: `completion.probe == "skipped"`; no human header. |
| **AC10** | Manual classify-only (`cargo run`, no schtasks mutate, no forced llama load): full `nightly --status` still heading + Router 267009 (volatile). **If** Completion is `probe=timeout (750ms)`, the **next line** is exactly F1. **If** Completion is `ok`/`down`/`error`, F1 is **absent**. `daemon status` may still say Open. Exit **0**. Pass-with-observed-data. Source/hermetic is DoD — **not PATH.** |
| **AC11** | Docs: CAPABILITIES + OPERATIONS name the timeout next-line; CHANGELOG T281. PROTOCOL-COMPAT no new required keys. CLI-EXIT-CODES status exit 0 unchanged. |
| **AC12** | No production `unwrap`/`expect`/`panic`; no clap/rusqlite bump; no DTO keys; `NIGHTLY_STATUS_PROBE_TIMEOUT` still 750 ms; `daemon.rs` / `llama_cpp.rs` / `project.rs` / `sync.rs` / `forget.rs` / `doctor.rs` absent from the product diff. |
| **AC13** | `format_endpoint_line` still 4 args; T247 `format_endpoint_line__quick__probe_skipped` stays green. |
| **AC14** | `--quick` still does not construct `LlamaCppProvider` (T247 F19). |

---

## 5. Design notes

### 5.1 Human layout (timeout arm)

```text
=== Nightly Status ===
Nightly: AI-Brains-Nightly
Scheduled: Yes (next run: …)
Last task result: 0
Last scheduled run: …
Last nightly run: …
Unsummarized sessions remaining: 0
Sessions summarized in last run: 4
Errors in last run: []
Completion: 127.0.0.1:8081  model=gemma-4-E4B-it-Q6_K.gguf  probe=timeout (750ms)
HTTP /health 750ms ≠ daemon TCP
Embedding: 127.0.0.1:8083  model=nomic-embed-text-v1.5  probe=ok
Router: Running  last result: 267009
task still running (SCHED_S_TASK_RUNNING)
Multi-import: …
======================
```

`--quick` / `ok` / `down` / `error`: no F1 line (T269 suffix only on exact `timeout`).

JSON: unchanged object; `completion.probe` remains `"timeout"` when that is the token.

### 5.2 Helper sketch (not implement)

```rust
pub(crate) const HTTP_VS_TCP_CONTRAST: &str = "HTTP /health 750ms ≠ daemon TCP";

pub(crate) fn completion_timeout_contrast_line(raw_label: &str) -> Option<&'static str> {
    if raw_label == "timeout" {
        Some(HTTP_VS_TCP_CONTRAST)
    } else {
        None
    }
}
```

Call site uses the **raw** `completion_label` (`as_label()` / `"skipped"`), not the human-wrapped string.

### 5.3 Why not fold HTTP into `probe=timeout (750ms HTTP /health)`?

T269 AC2 / operators already parse `timeout (750ms)`. Growing the token hides the TCP half and reopens suffix tests. A next line is scannable and matches the stub.

### 5.4 Why not live-detect daemon Open?

`--status` would grow a TCP loop (daemon.rs 100 ms × 5) or a second HTTP. The confusing case is **timeout + Open**, but printing “Open” without probing is a lie when the port is closed. Mechanism-only is honest for every timeout.

---

## 6. Non-goals

- Raise 750 ms / change run-path 2s
- Unify `daemon status` to HTTP `/health`
- JSON schema bump / `probe_budget_ms` / contrast key
- Doctor 16th model-port check
- Persist probe in `sync_state`
- Product `.cmd` / schedule-Router / live schtasks mutate
- `--quick --no-vault`
- TCP-probe from nightly
- Always-on contrast when `ok`
- Embedding-only timeout line (soft)
- clap 5 / rusqlite 0.40 / new DTO keys
- Live leftover rebind / `.env` rewrite / `cargo install`
- T282 / T283 peers
- Forcing llama.cpp load to reproduce timeout

---

## 7. Verification plan

1. **Red:** AC1–AC2 fail (const/helper missing); AC7 hermetic does not yet forbid F1 (add asserts — they pass today because F1 is absent; the **red** is AC1–AC2).
2. **Green:** const + helper + 3-line print in `nightly.rs`.
3. Targeted: `cargo nextest run -p ai-brains-cli nightly_status format_probe_label_human http_vs_tcp completion_timeout` + `--test nightly_status` + `nightly__help__names_nightly_heading_and_probe_budget`; clippy `-p ai-brains-cli --all-targets -- -D warnings`.
4. Manual classify-only AC10. **No** schtasks mutate. **No** forced generation.
5. Review log; BUGFIX cross-model (F22).
6. Full gate before finalize. implement-track Phase 6 publish.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| T269 suffix / heading / after_help regression | AC3 / AC6 / AC8 / F8 |
| JSON token pollution | AC4 / F3 |
| `--quick` grows HTTP or contrast | AC7 / AC14 / F4 |
| Contrast prints on `ok` | AC2 exact `== "timeout"`; AC10 observed-data |
| Unicode `≠` in Windows console | Repo already UTF-8; after_help is ASCII; F1 is one UTF-8 line. AC1 `assert_eq!` |
| PATH-behind until install | F13; hermetic/source DoD |
| Implementer mutates Nightly/Router or loads llama | F16 Stop-Before |
| File growth in `nightly.rs` (2133) | F9 — 3 lines + helper in `nightly_status.rs` |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit nightly Completion timeout vs daemon Open (750 ms not raised) | **Absorb** F1–F5 / AC1–AC2 / AC7 / AC10 |
| T269 closeout “two truths remain on the status block” | **Absorb** F1 — after_help already shipped; status line is DoD |
| T269 F21 JSON budget field | **Decline** F3 |
| T255 F18 raise 750 / F11 doctor 16th / F12 persist / F14 wrapper | **Decline** F2 / F11 |
| Unify daemon TCP with HTTP | **Decline** F10 |
| TCP-probe from nightly to print Open | **Decline** F1 / F27 |
| last-PR Cursor #196 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — **no T285** |
| T282 / T283 / leftover 11 roots | **Decline** peers |
| T240 F2 / clap 5 / DTO required keys | **Decline** F12/F17 |
| Identity mismatch quiet | **Not this track** — T258 adopt-path; leftover data T276; shell leftover T282 |
| Embedding-only timeout contrast | **Decline as DoD** F26 |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `changeguard` | **Decline** — not status chrome |

**Entire `deferred.md` scanned.** Closed/strikethrough rows stay closed. Open overlapping row is this placeholder (absorb). T282/T283 remain Pending placeholders.

---

## 10. Implement order (on go)

1. Phase 0 re-verify `nightly.rs` `:13/:52–66/:195–212`, `nightly_status.rs` `:8/:11/:260`, `daemon.rs` `:749`, `main.rs` `:1137/:658`, T269 units, deferred rescan, #196 still empty, pins.
2. Red AC1–AC2.
3. Const + helper in `nightly_status.rs` (F1); 3-line print in `nightly.rs`.
4. Extend hermetic AC7; AC3–AC6 / AC8–AC9 / AC13–AC14 stay green.
5. Docs F19; CHANGELOG.
6. Classify-only AC10. **No** schtasks. **No** forced llama load.
7. Review → `review.md`; BUGFIX TX; implement-track Phase 6 publish.

---

## 11. Soft residuals

| Residual | Disposition |
|----------|-------------|
| PATH until `cargo install` | F13 |
| Timeout vs Open not reproduced this session | Honest; gated on timeout arm; llama.cpp #20684 still the mechanism |
| JSON has no budget / contrast field | F3 / T269 F21 |
| after_help already explains; F1 is the status-block copy | Intentional dual (help + timeout line) |
| Embedding-only timeout has no F1 | F26 |
| ledgerful doctor `:8081` chat ≠ nightly `/health` | Not doctor 16th (F11) |
| Daemon Stopped + port Open | F27 — F1 does not say Open |
| Operator llama.cpp without `/health` fast-path | Not product DoD |
| T282 / T283 peers | F17 |
| Live leftover 11 roots | T276 F9 |
| Live 0 of 3 grants | T275 F10 |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/nightly_status.rs` | F1 const + helper + AC1/AC2 units |
| `crates/ai-brains-cli/src/commands/nightly.rs` | After Completion `println!`, optional F1 `println!` (~3 lines) |
| `crates/ai-brains-cli/tests/nightly_status.rs` | AC7 extend: no F1 on `--quick` |
| `Docs/CAPABILITIES.md` / `Docs/OPERATIONS.md` / `CHANGELOG.md` | F19 |
| `conductor/conductor.md` / `deferred.md` / README | Planned + absorb table |

**Do not touch:** `project.rs`, `sync.rs`, `forget.rs`, `daemon.rs`, `doctor.rs`, `llama_cpp.rs`, `preflight.rs`, `.env`, live vault, scheduled tasks.

---

## 13. AI fold-in

Empty at plan time. Fill on `/fold-in` from `agy-review.md` / `opencode-review.md`. Do **not** edit those review files here.
