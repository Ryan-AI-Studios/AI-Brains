# T269 review log — Nightly vs Router status split + probe honesty

**Track:** `conductor/tracks/trackT269-nightly-router-status-split`
**Category:** OPS / UX / BUGFIX
**BUGFIX TX:** `50098557-f967-4281-ab52-bb19c095719c`
**Date:** 2026-08-20

## Scope

Human `nightly --status` prints `Nightly: AI-Brains-Nightly` immediately after `=== Nightly Status ===` (outside `#[cfg(windows)]`) so Last Result **0** is not mixed with `Router: … 267009`. Human `probe=timeout` is labeled `timeout (750ms)` iff the raw token is exactly `"timeout"`. JSON probe tokens, 750 ms budget, `--quick` `probe=skipped`, Router line format, and `format_endpoint_line` / `format_status_schedule_block` stay frozen. No DTO, no clap 5, no live schtasks mutate, no `cargo install`.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC13 / F0–F27 | PASS |
| R1b | Independent explore (read-only) | **PASS** (no P0–P3 product findings) |
| CX1 | Codex `gpt-5.6-luna` high | **FAIL** — P1 process (gate/publish not yet); **P2** AC8 heading adjacency (fixed) |
| CX2 | Codex `gpt-5.6-luna` high | **PASS** (no remaining product findings; CX1 P2 verified) |

## Findings

### R1 / R1b

No product findings. R1b independently traced heading print **outside** `#[cfg(windows)]`, JSON early-return using raw `as_label` / `"skipped"`, `format_probe_label_human` exact `== "timeout"`, 750 ms const unchanged, `--quick` still skipping `LlamaCppProvider`, hotspots / daemon TCP / doctor 15 / embeddings 50 ms untouched. `rstest = "0.25"` is a cli **dev-dep only** (lock already had `0.25.0` via `ai-brains-path`; no version bump).

Noted out of DoD (not findings): JSON `probe: "timeout"` has no budget field (F21); PATH `ai-brains` 0.1.1 until operator `cargo install` (F16); daemon stays TCP (F18).

### CX1

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| CX1-P1-gate | high (process) | Full workspace gate / TX / publish not done at CX1 time | `verified_fixed` (gate) / remaining Phase 6 | `dev-check` 3217 + `ledgerful verify --scope full` |
| CX1-P1-publish | high (process) | Conductor/deferred still Planned; uncommitted | `fixed_pending_verification` until merge | this implement-track Phase 6 |
| CX1-P2-ac8 | medium | Hermetic AC8 used `contains` not banner+1 adjacency | `verified_fixed` | `tests/nightly_status.rs` banner at `n`, heading at `n+1`; CX2 PASS |

### CX2

No product findings. P2 adjacency verified. T247/T255 regression sweep clean.

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 heading const | met | `nightly_task_heading__equals_nightly_ai_brains_nightly` |
| AC2 timeout suffix + passthrough | met | `format_probe_label_human__timeout__budget_suffix`; rstest 7 cases (`skipped`/`ok`/`down`/`error`/`""`/`TIMEOUT`/`timeout-ish`) |
| AC3 schedule block `Last task result:` | met | `format_status_schedule_block__order__result_hint_then_last_scheduled` `lines[1] == "Last task result: 101"` |
| AC4 Router line frozen | met | `format_router_status_lines__running_267009__router_and_hint_on_following_line` |
| AC5 JSON timeout raw token | met | `build_nightly_status_json__timeout_probe__raw_token_no_budget_suffix`; quick fixture still `"skipped"` |
| AC6 after_help needles | met | `nightly__help__names_nightly_heading_and_probe_budget`; live `--help` contains `AI-Brains-Nightly`, `267009`, `SCHED_S_TASK_RUNNING`, `750`, `TCP`, `/health` |
| AC7 endpoint line skipped | met | `format_endpoint_line__quick__probe_skipped` |
| AC8 all-OS hermetic heading | met | `nightly_status__default_format__human_header_even_if_piped` (file not `cfg(windows)`): heading + `probe=skipped`, no `(750ms)`, exit 0 |
| AC9 JSON `--quick` | met | hermetic `nightly_status__format_json_quick__probe_skipped` + `…one_object_no_human_header` |
| AC10 manual | met | see Manual evidence |
| AC11 docs | met | CAPABILITIES additive T269 bullet; OPERATIONS heading + 750 ms vs TCP; CHANGELOG T269 row |
| AC12 no DTO / pins / doctor/daemon/embed | met | contracts untouched; clap lock 4.6.1; doctor 15; daemon TCP; T229/T247/T255 units green |
| AC13 `--quick` no LlamaCppProvider | met | `if quick { ("skipped", "skipped") }` else `join!`; JSON `"skipped"` |
| F1 heading outside windows cfg | met | `nightly.rs` after banner, before `#[cfg(windows)]` schedule block |
| F3/F27 exact `== "timeout"` | met | helper; `"TIMEOUT"` / `"timeout-ish"` / `""` pass through |
| F4 750 ms not raised | met | `Duration::from_millis(750)` |
| F5 JSON frozen | met | raw token in JSON path; `schema_version` 1 live |
| F6 Router frozen | met | `format_router_status_lines` not edited |
| F7 schedule helper frozen | met | heading is caller `println!` |
| F8 endpoint signature frozen | met | still 4 args; wrap at call site |
| F9 helpers in `nightly_status.rs` | met | units there; `nightly.rs` prints only |
| F12 exit 0 | met | AC8 + AC10 EXIT=0 |
| F13 capture independence | met | status/docs only |
| F14 no clap 5 / no new production crates | met | `rstest` **dev-dep** only, lock `0.25.0` unchanged |
| F17 no live mutate | met | no schtasks `/change` / `/create` |
| No production unwrap/expect/panic | met | new helper is `if`/`format!`; clippy `-D warnings` |
| Hotspots not grown | met | `project.rs` / `sync.rs` / `daemon.rs` not in diff |

## Targeted gates (observed)

- Red: AC1 heading `""`; AC2 `"timeout"` vs `"timeout (750ms)"`; AC6 after_help missing `AI-Brains-Nightly`; AC8 hermetic missing heading
- `cargo fmt` applied; `cargo fmt --check` on that pass
- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` exit 0
- `cargo nextest run -p ai-brains-cli -E "test(nightly)"` **90 passed**

## Full gate (observed)

- `.\scripts\dev-check.ps1` **SUCCESS** — nextest **3217** passed, 1 skipped (first run fail-fast: live daemon blocked T188 `backup_restore__daemon_down_force__succeeds`; re-run after temporary `daemon stop`/`start`)
- `ledgerful verify --scope full` **passed** (fmt 2.3s / clippy 1.8s / nextest 114.5s / deny 2.5s / audit 2.7s)

## Manual evidence

```text
AC10 cargo run -q -p ai-brains-cli -- nightly --status --quick
     Nightly: AI-Brains-Nightly
     Last task result: 0
     Completion … probe=skipped
     Embedding … probe=skipped
     Router: Running  last result: 267009
     EXIT=0. No (750ms).

AC10 cargo run -q -p ai-brains-cli -- nightly --status
     Nightly: AI-Brains-Nightly
     Last task result: 0
     Completion … probe=timeout (750ms)
     Embedding … probe=ok
     Router: Running  last result: 267009
     EXIT=0.

AC10 cargo run -q -p ai-brains-cli -- nightly --status --format json
     completion.probe = "timeout" (no " (750ms)")
     embedding.probe = "ok"
     router.last_result = "267009"
     schema_version = 1
     EXIT=0.

AC10 ai-brains daemon status
     LLM backend 127.0.0.1:8081 … Open
     Embedding backend 127.0.0.1:8083 … Open
     (TCP Open vs HTTP timeout — expected; F18)

AC6  cargo run -q -p ai-brains-cli -- nightly --help
     after_help names AI-Brains-Nightly, 267009, SCHED_S_TASK_RUNNING, 750, TCP, /health
     T255 format examples kept.
```

## Deferred candidates (Phase 5)

- PATH `ai-brains` until `cargo install` (F16)
- JSON `probe: "timeout"` has no budget field (F21)
- Operator llama.cpp without #20817 still queues `/health`
- `--quick --no-vault` (T255 F15)
- T270 / T272 peers
- T273 F7 recall `bridge_search_args`
