# T320 Review Log — Unified `ai-brains status` glance

**Track:** T320-UnifiedStatus  
**Category:** FEATURE / UX  
**FEATURE TX:** `a700986c-41d5-41b6-b417-6cac9153be0e`  
**Branch:** `track/T320-unified-status`  
**Phase:** 1 — Internal implementer DoD (pre cross-model)

## Scope

Unified `ai-brains status` glance: compose daemon / doctor / graph / nightly into one human+JSON envelope, fail-open per section, exit 0 always-report. New sibling `commands/status.rs`; early dispatch before `AppContext`; no doctor growth; no contracts DTO.

## Implemented

- New `crates/ai-brains-cli/src/commands/status.rs` — compose envelope, fail-open, human/JSON
- `Commands::Status { format }` display_order 12; Family A tokens; after_help F26
- Early dispatch before AppContext; unreachable arm
- Probe `DaemonProbePolicy::Status`; `build_report` + `format_doctor_summary` verbatim; graph via `crate::graph_density`; nightly `sync_state` SQL + `pub(crate) fetch_schedule_snapshot`; human `next:` = `daemon::status_next_line(false)`; JSON `next_step` prefix-less const
- `help_ia` Daily + Start-here; `memory_list_inventory` Daily lock
- Hermetic `tests/status_cli.rs`; clap AC2 units in `main.rs`
- Docs: CAPABILITIES, PROTOCOL-COMPAT, OPERATIONS, CLI-EXIT-CODES, CHANGELOG

## Isolation (`git` vs `origin/main`)

| Path | Diff |
|------|------|
| `doctor.rs` / `graph.rs` / `daemon.rs` / `project.rs` / `sync.rs` / `governed_common.rs` / contracts | **empty** |
| `nightly.rs` | visibility-only `pub(crate)` `ScheduleSnapshot` + `fetch` (~+6/−4) |
| `main.rs` | +77/−0 (includes clap unit tests) |
| F32 other-files sum | ~82 physical (main 77 + small visibility/help) — slight over 80; parked as low residual |

## Tests observed

| Gate | Result |
|------|--------|
| `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` | exit 0 |
| status unit tests + clap AC2 + help_ia Daily | PASS |
| hermetic `status_cli` (json envelope, human, help, xml exit 2, auto piped json) | PASS |
| stay-green: `health_check_order_names__fixed_matrix` | PASS |
| stay-green: `status_next_line__stopped__daemon_start` | PASS |
| Full workspace gate | **PASS** — `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo nextest run --workspace` **3622** passed; `cargo deny check`; `cargo audit`; `ledgerful verify --scope full` → Verification passed |

## Manual evidence (AC11)

```text
cargo run -q -p ai-brains-cli -- status --format human
→ daemon Running
→ doctor degraded (graph_density)
→ graph sparse E/N=0.416
→ nightly last 2026-08-28 scheduled Yes
→ exit 0

cargo run -q -p ai-brains-cli -- status --format json
→ schema_version 1 envelope
→ exit 0

cargo run -p ai-brains-cli -- status --format json   # graph-off / no --features graph
→ doctor remediation reinstall SOOT — expected; not FEATURE_UNAVAILABLE exit 2
```

PATH-behind install is not a fail (F22).

## DoD matrix (Phase-1 internal)

| AC | Result | Evidence |
|----|--------|----------|
| AC1 | Met | Stay-green `format_resolve` auto TTY/pipe; clap `status --format` uses Family A resolver (pipe default json) |
| AC2 | Met | Clap AC2 units in `main.rs` — `xml` / `JSON` → InvalidValue exit 2; hermetic xml exit 2 |
| AC3 | Met | Status unit tests — frozen `schema_version: 1` keys `daemon`/`doctor`/`graph`/`nightly`; `next_step` omitted when Running |
| AC4 | Met | Status unit — doctor `Err` → `doctor.error` nonempty; `daemon.state` still present |
| AC5 | Met | Status unit — graph gather fail → `graph.error`; other sections present |
| AC6 | Met | Status unit — never + unscheduled; mapper `snap.next_run.is_some()` |
| AC7 | Met | Status unit — human `next:` equals `daemon::status_next_line(false)`; JSON prefix-less const; stay-green `status_next_line__stopped__daemon_start` |
| AC8 | Met | Status unit — verbatim `format_doctor_summary`; no `=== Nightly Status ===` / `LLM backend` / `probe=` |
| AC9 | Met | Hermetic `status_cli` json envelope — keys only; no host `daemon.state` assert |
| AC10 | Met | Hermetic human lines + pipe `--format auto` → JSON (AC10 pipe-auto hermetic added) |
| AC11 | Met | Manual source bin above (observed-data E/N / last_run); exit 0 |
| AC12 | Met | CAPABILITIES, PROTOCOL-COMPAT, OPERATIONS, CLI-EXIT-CODES, CHANGELOG; after_help F26 |
| AC13 | Met | Isolation empty for doctor/graph/daemon/project/sync/governed_common/contracts; `nightly.rs` visibility-only |
| AC14 | Met | Stay-green `health_check_order_names__fixed_matrix`; `status_next_line__stopped__daemon_start`; help_ia Daily lock updated |
| AC15 | Met | Graph-off `cargo run` includes graph path / remediation SOOT — exit 0, not FEATURE_UNAVAILABLE |
| AC16 | Met | Manual human `E/N=0.416` three decimals; JSON raw f64 (status units) |
| AC17 | Met | Hermetic `status --help` F26; help_ia Daily + `memory_list_inventory` Daily lock contain `status` |

## Findings (R1 — Internal isolation / DoD)

| id | severity | description | source | files | required_fix | status | evidence |
|----|----------|-------------|--------|-------|--------------|--------|----------|
| R1-01 | low | F32 other-file net ~82 physical vs 80 budget (main clap tests dominate) | implementer / F32 | `main.rs` + visibility/help | Park residual; do not grow hotspots | deferred | Isolation of frozen files empty; overage is clap AC2 units in `main.rs` |
| R1-02 | low | PATH `ai-brains` still pre-T320 until owner `cargo install` | F22 | — | Document; do not install | deferred | Manual/hermetic via `cargo run` / test bin |
| R1-03 | low | Live E/N ~0.42 still sparse | F36 | — | Honesty only; no floor retune / rebuild | deferred | AC11 observed sparse; remediator not this DoD |
| R1-04 | low | Doctor Safety vs glance Status probe by design | F9 | `daemon_probe` / status compose | Soft residual | deferred | Status 1×300 ms; doctor Safety unchanged |
| R1-05 | low | Two vault opens (`build_report` + glance `open_read_intent`) | F44 | `status.rs` / doctor | Soft residual; do not grow doctor to return conn | deferred | Documented in spec F44 |
| R1-06 | low | AC10 pipe-auto hermetic coverage gap | implementer | `tests/status_cli.rs` | Add hermetic auto-piped JSON | verified_fixed | Hermetic auto piped json PASS |
| R1-07 | — | No medium/high/critical from internal isolation review | implementer | — | — | — | Frozen-file diffs empty; no behavior bleed into doctor/graph/daemon run_status |
| CX2 | low | `DoctorOptions.summary: true` was inert/misleading (`build_report` ignores it) | OpenCode CX2 | `status.rs` | Set `summary: false` + comment | verified_fixed | OpenCode cross-model; fixed before commit |

## Cross-model

- Codex primary: **usage limit** (retry window later) — not available.
- Claude fallback: **OAuth expired** — failed to authenticate.
- OpenCode `ollama-cloud/glm-5.2` high → `review.codex.md`: **CLEARED** / no P0–P2; P3 CX1 commit hygiene (owner), CX2 inert summary (**fixed**), CX3 F32 interpretation (already R1-01).

## Verdict

**Engineering DoD complete.** AC1–AC17 Met; isolation AC13 empty on frozen files; full workspace gate green (nextest **3622**); OpenCode cross-model PASS WITH DEFERRED P3. Residual lows → `deferred.md`. Proceed to FEATURE commit + Phase 6 publish.
