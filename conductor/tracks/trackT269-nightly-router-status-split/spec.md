# T269 — Nightly vs Router status split + probe honesty

- **Track ID:** T269-NightlyRouterStatusSplit
- **Status:** **Planned** (Pending in `conductor.md`; plan-only until go)
- **Category:** OPS / UX / BUGFIX
- **Owner:** Grok
- **Source:** Audit 2026-08-16 — friction: human mixes Nightly Last Result **0** with Router **267009**; full `--status` Completion `probe=timeout` while `daemon status` says LLM backend **Open**
- **Depends on:** T229/T247/T255 ✅ JSON + read-only Router line (PR #140 / #157 / T255 squash)
- **Blocks / feeds:** Operators can tell **two tasks** apart and can tell HTTP `/health` budget-timeout from “backend down.” Does **not** unblock T270 retention classify or T272 Safety skip.
- **Absorbs:** deferred.md “Nightly human mixes Router 267009; completion probe timeout”; placeholder F1–F5
- **Not absorbed (DoD):** Raise 750 ms (T255 F18 / llama.cpp `/health` queue); unify daemon TCP with HTTP; JSON `probe_budget_ms` / schema bump; doctor 16th; persist probe; product `.cmd` / schedule-Router; `--quick --no-vault`; T270 / T272; T273 F7 `bridge_search_args`; clap 5 / pin bumps; contracts DTO
- **Research date:** 2026-08-20 (source HEAD `6825343`)
- **AI fold-in:** none yet (planning pass). Disposition lands in **§13** after review-track.
- **Ledger:** planning DOCS TX `7f7f7fd2-5ce1-4892-94d0-451699366dd0`. Implement starts a **BUGFIX** TX on **go**.
- **Isolation:** Do **not** reopen T247 `--quick` / 750 ms `join!` / LIST /V / missing-action, T255 JSON keys / Router line format / `found` vs `next_run`, T229 F5 truncate, T239 multi-import. Do **not** mutate `AI-Brains-Nightly` or `AI-Brains-Router`. Do **not** `cargo install`, pin to the live vault as implement, rewrite `.env`, or print `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Two tasks are unmistakable.** Human `nightly --status` prints a `Nightly: AI-Brains-Nightly` heading so `Last task result: 0` cannot be read as the same object as `Router: … 267009`.
2. **Timeout is a budget, not “backend down.”** Human `probe=timeout` becomes `probe=timeout (750ms)`. `--quick` stays `probe=skipped`. JSON probe tokens stay `ok|down|timeout|error|skipped`.
3. **Keep T247/T255 contracts.** Status exit **0** on down / timeout / missing action / nonzero Last Result / Router 267009. JSON `schema_version` **1** frozen. Router line format frozen. 750 ms parallel probes frozen. No CLI `reqwest`. No live task mutate.
4. **Capture independence.** Status/docs/parse only. No events, no models on the `--quick` path beyond existing skip, no contracts DTO, no new crates, no pin bumps.

This unblocks daily ops honesty for the Windows-first vault: the nightly job succeeded (Last Result **0**, 8 sessions) while the Router keep-alive is *supposed* to show **267009**. Mixing those numbers is a product lie.

---

## 2. Live baseline (re-scan 2026-08-20)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `6825343` — T273 squash-merged (#185). Tree **CLEAN**. `main` == `origin/main`. |
| PATH `ai-brains` | **0.1.1**. `nightly --status --quick` already has T255 Router line. **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic / PATH (PATH is current enough for this chrome). |
| PATH `--status --quick` | `Last task result: 0` (no `Nightly:` heading). Then `Router: Running  last result: 267009` + `task still running (SCHED_S_TASK_RUNNING)`. `probe=skipped`. Multi-import 2026-08-20 ok. **Live hole 1 confirmed.** |
| PATH `--status --format json` (full, not `--quick`) | `last_task_result: "0"` vs `router.last_result: "267009"` / `task_to_run: C:\llm\router.bat`. `completion.probe: "timeout"`. `embedding.probe: "ok"`. Exit **0**. JSON already split. |
| `ai-brains daemon status` (same moment) | Running. LLM `127.0.0.1:8081` **Open**. Embedding `127.0.0.1:8083` **Open**. **Live hole 2 confirmed** — Open ≠ HTTP `/health` in 750 ms. |
| `AI-Brains-Nightly` LIST /V | Ready; Last Result **0**; Task To Run `"C:\Users\RyanB\.cargo\bin\ai-brains.exe" nightly`; next **8/21/2026 3:00:00 AM**; Last Run **8/20/2026 3:00:02 AM**. |
| `AI-Brains-Router` LIST /V | **Running**; ONLOGON (`Next Run Time: N/A`); Last Result **267009**; Task To Run **unquoted** `C:\llm\router.bat`. |
| Nightly `--format` / `--quick` | Exist (T255/T247). Default human; pipes stay human. `after_help` exists (T255 F37) — format examples only; **no** 267009 / 750 ms honesty. |
| Doctor | Frozen **15** checks. Do not add model ports. |
| Last GitHub PR | [#185](https://github.com/Ryan-AI-Studios/AI-Brains/pull/185) T273 (2026-08-20). Issue comments **0**, review comments **0**, reviews **[]**. **last-PR Cursor: N/A (empty).** Open PRs: Dependabot remotes only (not this HEAD). #184 Linux Path units already `#[cfg(windows)]` (T273 F8) — **no T274**. |
| Identity / doctor ambient | Scope `3581317d`; discovery grants 0 of 3; ledgerful doctor leftover `.changeguard` / sig-pin / timings / command_timings. 0 pending / 0 drift at plan scan. Do not “fix” here. |

### 2.2 Why these residuals still matter

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Unlabeled `Last task result: 0` next to `Router: … 267009` | JSON is already split. Human is not. Operators (and the 2026-08-16 audit) misread 267009 as a failed nightly. Microsoft Learn: `SCHED_S_TASK_RUNNING` = `0x00041301` = **267009** is a **success** constant (“The task is currently running”), not an error. T247 already decodes the hint. **DoD is a Nightly heading.** |
| `probe=timeout` vs daemon **Open** | Different probes. Daemon (`daemon.rs` ~749): `TcpStream::connect_timeout` 100 ms × 5 with backoff → **Open** = port listening. Nightly: `LlamaCppProvider::probe_health(750ms)` = HTTP `GET /health` then `/v1/models`. Busy llama.cpp can queue `/health` (ggml-org/llama.cpp#20684, closed by #20817). Raising 750→2000 ms still fails during a 30 s generation and reopens T247 `&lt;1.5s` status. **DoD is label the budget on human timeout only.** |
| Raise 750 ms | T255 F18 freeze + T247 latency. llama.cpp evidence: `/health` was queued like other requests under load. **Decline.** |
| JSON `probe: "timeout (750ms)"` or `probe_budget_ms` | T255 AC4 / F5 frozen tokens + keys. Scripts already have `timeout`. **Decline as DoD** (soft residual). |
| Unify daemon to HTTP `/health` | T199 daemon status is TCP liveness; would make `daemon status` wait on busy llama. **Decline.** |
| Mutate Router / Nightly / write `.cmd` | T255 F14 / F30. **Decline.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Status branch | `nightly.rs` `run` `if status` ~40–217 | Human `println!` after JSON early-return. No `Nightly:` heading. |
| Banner | `println!("=== Nightly Status ===")` ~156 | Keep. Heading goes **after** this, **before** `format_status_schedule_block`. |
| Schedule block | `format_status_schedule_block` ~890 | `Scheduled:` → `Last task result:` → hint? → `Last scheduled run` → action missing. **Do not change internals** (T247 units lock `lines[1] == "Last task result: 101"`). |
| Endpoint line | `format_endpoint_line` ~800 | `{kind}: {host_port}  model={model}  probe={label}`. **Keep 4-arg signature.** Wrap the label on the **human** call site only. |
| Probe timeout | `NIGHTLY_STATUS_PROBE_TIMEOUT` ~13 = **750 ms**; run-path `NIGHTLY_PROBE_TIMEOUT` = **2s** | Do not change either. Pass `.as_millis()` into the human wrapper. |
| Probe labels | `ProbeStatus::as_label` → `ok\|down\|timeout\|error`; `--quick` string `"skipped"` (T247 F19) | JSON uses raw label. Human timeout gets ` (750ms)` suffix. |
| Router lines | `nightly_status.rs` `format_router_status_lines` ~146 | Exact `Router: Running  last result: 267009` (T255 AC6). **Do not restyle.** |
| JSON builder | `build_nightly_status_json` | Frozen keys in `FROZEN_KEYS`. `completion.probe` is the raw label. |
| clap | `main.rs` `Nightly` ~1059 | `after_help` format examples only. Additive honesty. `#[command(after_help = …)]` still clap **4** (`docs.rs/clap/4.6.6` `Command::after_help`). |
| Tests | `nightly.rs` T229/T247 units; `nightly_status.rs` T255 units | Stay green. New units in `nightly_status.rs`. |
| Hotspots | `project.rs` **#1** (4.027); `sync.rs` #2; `daemon.rs` **#10** | Do **not** touch. `nightly.rs` **1964** lines / `nightly_status.rs` **554** — not top-10. New helpers in `nightly_status.rs`. |
| Doctor / embeddings / daemon probe | `doctor.rs` 15; `embeddings.rs` 50 ms; `daemon.rs` TCP | Untouched. |

### 2.4 Dependency / standards research (2026-08-20)

**Snapshot — re-verify at execute.**

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (docs.rs 4.6.6, 2026-08-11). **No clap 5.** `after_help` unchanged. | **No bump.** Additive `after_help` text only. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** JSON keys frozen. |
| `tokio` | workspace **1.52** / lock **1.52.3** | crates.io **1.53.1** | **No bump.** |
| `reqwest` | workspace **0.13** / lock **0.13.4** | models crate only | **No CLI dep.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | — | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| 267009 is success, not failure | [Microsoft Learn — Task Scheduler constants](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-error-and-success-constants) (`SCHED_S_TASK_RUNNING` `0x00041301`) | Human already has T247 hint. Heading is the missing split. Do not “fix” 267009. |
| `/health` can miss a 1 s budget under load | [llama.cpp #20684](https://github.com/ggml-org/llama.cpp/issues/20684) (closed, PR #20817) | `/health` was queued like other HTTP. Raising 750 ms does not fix a busy slot. Label the budget. |
| clap `after_help` | [docs.rs/clap/4.6.6 `Command::after_help`](https://docs.rs/clap/4.6.6/clap/struct.Command.html) | Keep derive `after_help = "…"`. No clap 5 multi-heading. |
| CLI status honesty | Existing T247/T255 SOOT + this machine’s LIST /V | Human chrome only; JSON already honest. |

**N/A:** SQLCipher, schtasks mutate, contracts DTO, Windows service install.

**Could not verify:** whether this machine’s `C:\llm\router.bat` llama.cpp build includes #20817 (operator binary). DoD does not depend on that upgrade.

**ledgerful / ai-brains:** `preflight --summary`; `ledgerful doctor` (5 warn, work root this repo); ledger 0 pending / 0 drift; `index --incremental`; `search --json -- "NIGHTLY_STATUS_PROBE_TIMEOUT"` hits `nightly.rs:13`; `scan --impact` CLEAN at `6825343`; `hotspots` project.rs #1 (do not grow); `ai-brains recall "nightly status Router 267009 probe timeout 750ms" --no-bridge` returned T255 review memories (JSON already split; human Router shipped). Semantic `ask` skipped if Cozo dim mismatch (prior session).

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `7f7f7fd2`. Implement starts a **BUGFIX** TX. |
| **F1 — Nightly heading** | Human status prints `Nightly: AI-Brains-Nightly` immediately after `=== Nightly Status ===` and **before** `format_status_schedule_block`. Same string on Windows and non-Windows (task name is a product constant; Unix still has no schtasks block). Do **not** rename `Last task result:` (T247 substring lock). |
| **F2 — `--quick`** | Still `probe=skipped` (string literal). No HTTP. No `(750ms)` suffix on skipped / ok / down / error. |
| **F3 — Timeout label (human only)** | When the raw probe label is `"timeout"`, human `format_endpoint_line` receives `timeout ({ms}ms)` where `{ms}` is `NIGHTLY_STATUS_PROBE_TIMEOUT.as_millis()` (**750** today). JSON / `--format json` keep the raw token `"timeout"`. |
| **F4 — Do not raise 750 ms** | Affirm T255 F18 / T247 parallel 750 ms. Evidence: llama.cpp #20684; live daemon Open vs HTTP timeout. Run-path probe stays **2s**. |
| **F5 — JSON frozen** | `schema_version` **1**. `FROZEN_KEYS` unchanged. Probe tokens `ok\|down\|timeout\|error\|skipped`. No `probe_budget_ms`. No Router restyle. |
| **F6 — Router line frozen** | `format_router_status_lines` output stays T255 AC6/AC15 (`Router: Running  last result: 267009` / `Router: last result: 267009` / `Router: not scheduled`). Heading work is **Nightly**, not Router. |
| **F7 — `format_status_schedule_block` frozen** | Do not inject the heading inside the helper (T247 `lines[1] == Last task result: 101`). Print heading at the status call site. |
| **F8 — `format_endpoint_line` signature frozen** | Still 4 args. New `format_probe_label_human(label, budget_ms) -> String` in `nightly_status.rs`. Human path wraps; JSON path does not. |
| **F9 — Module** | New helpers + their units live in `nightly_status.rs`. `nightly.rs` prints / dispatches only. **Do not** move T229/T247 tests out of `nightly.rs`. **Do not** grow `project.rs` / `sync.rs` / `daemon.rs`. |
| **F10 — after_help additive** | Keep T255 format examples. Add honesty: Nightly Last Result is `AI-Brains-Nightly`; Router **267009** is `SCHED_S_TASK_RUNNING` (success; ONLOGON keep-alive); `probe=timeout` is HTTP `/health` within **750 ms**; `daemon status` Open is TCP connect. |
| **F11 — Docs** | CAPABILITIES T247/T255 honesty: one additive bullet (not a new section). OPERATIONS: heading + timeout-budget sentence. Root CHANGELOG T269 row. CLI-EXIT-CODES unchanged (status still exit **0**). |
| **F12 — Exit 0** | Unchanged. Timeout / 267009 / down are still success for `--status`. |
| **F13 — Capture independence** | Status/docs only. No events. No `sync_state` probe write (T255 F12). |
| **F14 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.1**. No CLI `reqwest`. |
| **F15 — Contracts** | No DTO. PROTOCOL-COMPAT untouched. |
| **F16 — PATH** | Do not `cargo install` unless the user asks. Tests/manual use `cargo run` / hermetic. |
| **F17 — Stop-before live mutate** | Even after go: do not unschedule/reschedule Nightly, do not write `.cmd`, do not touch Router registration. |
| **F18 — Decline daemon HTTP** | `daemon status` stays T199 TCP. Do not import `probe_health` there. |
| **F19 — Decline doctor 16th / persist probe / embed sleep / wrapper / `--no-vault`** | T255 F11–F15 stand. |
| **F20 — Decline T270 / T272 / T273 F7** | Peers. `bridge_search_args` dash-query stays T273 soft residual. |
| **F21 — Decline JSON budget field** | Human + docs + after_help are the remediator. Soft residual if scripts treat `timeout` as “server dead.” |
| **F22 — Tests** | Naming `function_or_feature__condition__expected_result`. Units for heading const, probe wrapper (timeout vs skipped/ok/down/error), after_help needles. Existing T247/T255 units stay green. Hermetic `--quick` contains heading + `probe=skipped` and does **not** contain `(750ms)`. No `unwrap`/`expect`/`panic` in production. |
| **F23 — Cross-model** | Honesty UX on the status path (easy T255 regression). After Phase-1 review clean, run read-only `codex-review`. |
| **F24 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals go to `conductor/deferred.md`. |
| **F25 — last-PR Cursor** | #185 empty → N/A. #184 Linux Path already declined (no T274). |
| **F26 — Non-Windows** | Heading still prints. Scheduler / Router lines still omitted (T229/T255). JSON `scheduled`/`router` still `null`. |
| **F27 — Helper purity** | `format_probe_label_human` is pure (`&str`, `u128` millis → `String`). No I/O. Compare with `== "timeout"` (the `as_label` token), not `contains`. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `NIGHTLY_TASK_HEADING` (or helper) equals `Nightly: AI-Brains-Nightly` |
| **AC2** | Unit: `format_probe_label_human("timeout", 750) == "timeout (750ms)"`. `"skipped"` / `"ok"` / `"down"` / `"error"` round-trip unchanged (rstest `#[case]`) |
| **AC3** | Existing T247: `format_status_schedule_block` still has `Last task result: 101` as today (do not insert heading into the vec) |
| **AC4** | Existing T255: `format_router_status_lines(true, Some("Running"), Some("267009"))` first line still `Router: Running  last result: 267009`; hint still the following line |
| **AC5** | Existing T255 JSON fixture: `completion.probe == "skipped"` (or whatever the fixture sets); a timeout fixture (if added) serializes `"timeout"` **without** ` (750ms)` |
| **AC6** | Clap: `nightly --help` after_help contains `AI-Brains-Nightly`, `267009` or `SCHED_S_TASK_RUNNING`, and `750` |
| **AC7** | Existing T247: `format_endpoint_line(…, "skipped")` still contains `probe=skipped` and does **not** require the wrapper |
| **AC8** | Hermetic / process: `nightly --status --quick` (temp vault) stdout contains `Nightly: AI-Brains-Nightly` and `probe=skipped` and does **not** contain `(750ms)`. Exit **0**. May contain live `Router:` (host schtasks — do **not** assert 267009) |
| **AC9** | Hermetic: `nightly --status --format json --quick` stdout is JSON, `completion.probe == "skipped"`, no `=== Nightly Status ===`. Frozen keys still present |
| **AC10** | Manual (source or PATH): full `nightly --status` (not `--quick`) shows `Nightly: AI-Brains-Nightly`, `Last task result: 0` (volatile), `Router: … 267009`, and if Completion is timeout then `probe=timeout (750ms)`. `daemon status` may still say Open. Exit **0**. Do **not** mutate tasks |
| **AC11** | Docs: CAPABILITIES additive T247/T255 bullet; OPERATIONS heading + 750 ms vs TCP sentence; root CHANGELOG T269 row |
| **AC12** | No contracts DTO; no pin bumps; `embeddings.rs` / `doctor.rs` 15-check / `daemon.rs` TCP untouched; existing T229/T247/T255 nightly units green |
| **AC13** | `--quick` still does not construct `LlamaCppProvider` (T247 F19). JSON `--quick` `probe` is `"skipped"` |

---

## 5. Design notes

### 5.1 Human layout (after)

```text
=== Nightly Status ===
Nightly: AI-Brains-Nightly
Scheduled: Yes (next run: …)
Last task result: 0
Last scheduled run: …
Last nightly run: …
Unsummarized sessions remaining: 0
Sessions summarized in last run: 8
Errors in last run: []
Completion: 127.0.0.1:8081  model=…  probe=timeout (750ms)
Embedding: 127.0.0.1:8083  model=…  probe=ok
Router: Running  last result: 267009
task still running (SCHED_S_TASK_RUNNING)
Multi-import: …
======================
```

`--quick` Completion/Embedding stay `probe=skipped` (no budget suffix).

### 5.2 Helper sketch (not implement)

```rust
pub(crate) const NIGHTLY_TASK_HEADING: &str = "Nightly: AI-Brains-Nightly";

pub(crate) fn format_probe_label_human(label: &str, budget_ms: u128) -> String {
    if label == "timeout" {
        format!("timeout ({budget_ms}ms)")
    } else {
        label.to_string()
    }
}
```

Status human call site wraps with `NIGHTLY_STATUS_PROBE_TIMEOUT.as_millis()`. JSON path keeps `completion_label.to_string()`.

### 5.3 Why not raise the budget

T247 measured loopback `/health` when idle at **6–57 ms**. 750 ms is already ~10× that. Live timeout is the **busy** server (generation holds the HTTP worker). llama.cpp #20684: `/health` had no fast path. A 2 s status probe reopens the 4–6 s audit without making a 30 s generation look “ok.” Honesty is the remediator; operator llama.cpp upgrade is a soft residual.

### 5.4 Capture independence

No `MemoryPinned`, no probe persistence, no models on `--quick`. Router fetch stays a second `schtasks` spawn.

---

## 6. Non-goals

- Raising `NIGHTLY_STATUS_PROBE_TIMEOUT` or run-path 2 s
- JSON schema / probe token / `probe_budget_ms`
- Restyling the Router line or decoding 267009 as failure
- Doctor 16th model-port check
- Persist probe in `sync_state`
- Product `nightly-run.cmd` / `nightly --schedule` registering Router
- `--quick --no-vault`
- Changing `daemon status` from TCP to HTTP
- T270 retention classify / T272 `safety_ids` / T273 `bridge_search_args`
- clap 5 / pin bumps / new crates / CLI `reqwest`
- Live schtasks mutate / `.env` rewrite / `cargo install`
- T240 F2 silent Scope switch

---

## 7. Verification plan

TDD: failing units **first** (heading const, probe wrapper cases, after_help needles), then wire the call site.

```powershell
# Red → green
cargo nextest run -p ai-brains-cli --lib nightly_status
cargo nextest run -p ai-brains-cli --lib nightly
cargo nextest run -p ai-brains-cli -E "test(nightly)"
cargo clippy -p ai-brains-cli --all-targets -- -D warnings

# Manual (do not schtasks /change)
cargo run -q -p ai-brains-cli -- nightly --status --quick
cargo run -q -p ai-brains-cli -- nightly --status
cargo run -q -p ai-brains-cli -- nightly --status --format json
ai-brains daemon status
cargo run -q -p ai-brains-cli -- nightly --help

# Full gate (before finalize)
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| T247 `Last task result:` substring tests fail | Heading is a separate `println!`, not inside `format_status_schedule_block` (F7) |
| T255 Router exact-line tests fail | Do not restyle Router (F6) |
| JSON probe grows a suffix | Wrapper only on human call site (F3/F8) |
| Hermetic asserts live 267009 | AC8 forbids it (T255 AC9 lesson) |
| Status latency regression | Do not raise 750 ms (F4) |
| `project.rs` / `daemon.rs` hotspot edits | F9 forbid |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-20. `ISSUES.md` does not exist.

| Item | Disposition |
|------|-------------|
| Audit “Nightly human mixes Router 267009; completion probe timeout” | **Absorb** F1–F3 / AC1–AC2 / AC6 / AC8 / AC10 |
| Placeholder F1 Nightly/Router headings | **Absorb** F1 / F6 (Router already headed; Nightly is the hole) |
| Placeholder F2 `--quick` skipped | **Absorb** F2 / AC8 / AC13 |
| Placeholder F3 raise **or** label 750 ms | **Absorb as label** F3 / F4 — do not raise |
| Placeholder F4 read-only Router | **Affirm** T255 F7 / this F6 / F17 |
| Placeholder F5 JSON keys frozen; human-only preferred | **Absorb** F5 / F21 |
| T255 F18 probe timeouts unchanged | **Affirm** F4 |
| T255 closeout: doctor 16th / persist probe / embed sleep / `.cmd` / `--no-vault` | **Decline** F19 (already declined; still true) |
| T255 PATH `cargo install` | **Decline as DoD** F16 |
| T255 live reschedule missing `.cmd` | **Decline** F17 — live Nightly now points at `ai-brains.exe nightly` (Last Result 0); still do not mutate |
| T247 F11–F16 | **Closed by T255** — not reopened |
| T229 F8–F14 | **Closed by T255** except this honesty remainder |
| T270 retention 0 candidates | **Decline** — peer placeholder |
| T272 `safety_ids` over-exclude | **Decline** — peer placeholder |
| T273 F7 `bridge_search_args` dash-query | **Decline** F20 — retrieval crate; not status chrome |
| last-PR Cursor #185 | **N/A** — comments/reviews empty |
| last-PR #184 Linux Path units | **Decline** — already `#[cfg(windows)]`; **no T274** |
| T240 F2 / T255 bag / clap 5 / DTO | **Decline** F14 / F15 / §6 |
| Unify daemon TCP ↔ HTTP `/health` | **Decline** F18 |
| JSON `probe_budget_ms` | **Decline** F21 (soft residual) |
| Raise 750 ms | **Decline** F4 (llama.cpp #20684) |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `changeguard`, R-CI-BRANCH | **Decline** — not status chrome |
| T214 ledgerful-on-global / T266 graph TTY-auto / T253 Claude nightly | **Decline** — peers / T255 F16 |

---

## 10. Implement order (on go)

1. Phase 0: re-verify live hole + deferred rescan + BUGFIX TX.
2. **Red:** AC1 heading const; AC2 rstest cases; AC6 after_help test (fail on current help).
3. **Green:** const + `format_probe_label_human` in `nightly_status.rs`; print heading; wrap human endpoint labels; additive `after_help`.
4. Confirm AC3/AC4/AC5/AC7/AC9/AC13 stay green (no production change to those helpers).
5. Hermetic AC8; manual AC10; docs AC11.
6. Phase-1 review → Codex (F23) → gate → publish (implement-track Phase 6).

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| JSON `probe: "timeout"` has no budget | F21 — scripts must read CAPABILITIES; not a schema bump |
| Operator llama.cpp without #20817 still queues `/health` | Not product DoD; status honesty is the remediator |
| PATH `cargo install` | F16 — operator |
| `--quick --no-vault` | T255 F15 |
| T270 / T272 | Peers |
| T273 F7 recall `bridge_search_args` | Other crate |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/nightly_status.rs` | `NIGHTLY_TASK_HEADING`; `format_probe_label_human`; units AC1/AC2 |
| `crates/ai-brains-cli/src/commands/nightly.rs` | Print heading; wrap human probe labels with timeout millis. **Do not** edit `format_status_schedule_block` / `format_endpoint_line` signature / probe consts |
| `crates/ai-brains-cli/src/main.rs` | Additive Nightly `after_help` (AC6) |
| `Docs/CAPABILITIES.md` | Additive T247/T255 honesty bullet |
| `Docs/OPERATIONS.md` | Heading + 750 ms vs TCP |
| `CHANGELOG.md` | T269 row |
| `conductor/conductor.md` | Planned / Pending (implement sets Completed) |
| `conductor/deferred.md` | This absorption table |
| `conductor/tracks/README-T256-T271-CLI-AUDIT.md` | T269 Planned |

**Do not touch:** `project.rs`, `sync.rs`, `daemon.rs`, `embeddings.rs`, `doctor.rs`, `llama_cpp.rs` `probe_health`, `ai-brains-contracts`, `Cargo.toml` / lock.

---

## 13. Fold-in disposition

Empty until `/review-track` + `/fold-in`. Do not edit `*-review.md` from this planning pass.
