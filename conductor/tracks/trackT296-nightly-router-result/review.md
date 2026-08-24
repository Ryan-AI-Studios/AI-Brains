# T296 Review — Nightly Router last-result honesty

**Track:** `conductor/tracks/trackT296-nightly-router-result`  
**Status:** Phase-1 clean → cross-model BUGFIX  
**BUGFIX TX:** `388b9f76-dd66-4978-9a8e-3964d4fb372a`  
**Branch:** `track/T296-nightly-router-result`

---

## Scope landed

| Surface | Change |
|---------|--------|
| `crates/ai-brains-cli/src/commands/nightly_status.rs` | `ROUTER_LAST_RUN_TERMINATED` + `format_router_status_lines` F1–F4 + unit rewrite/add (F33/F34) |
| `crates/ai-brains-cli/src/main.rs` | Nightly `after_help` additive 267014 sentence + AC6 help unit |
| `crates/ai-brains-cli/tests/nightly_status.rs` | AC8 human contains-not `267014` / `SCHED_S_TASK_TERMINATED` |
| `Docs/CAPABILITIES.md` | T269/T281 bullet → T296 human omits decimals |
| `Docs/OPERATIONS.md` | Router bullet human vs JSON |
| `Docs/CLI-EXIT-CODES.md` | both 267009 and 267014 as `SCHED_S_*` success → exit 0 |
| `CHANGELOG.md` | T296 Unreleased |

**Untouched (freeze):** `explain_last_task_result` / Nightly schedule block in `nightly.rs`; `doctor.rs`; `daemon.rs`; `project.rs`; `llama_cpp.rs`; `embeddings.rs`; contracts; `Cargo.lock`; PROTOCOL-COMPAT.

---

## Phase 0 dogfood

| Check | Result |
|-------|--------|
| HEAD at start | `1694a87` (fold-in); `origin/main` `8b95181` (`left-right` `0 2`) |
| PATH hole | `Router: Ready  last result: 267014` + `SCHED_S_TASK_TERMINATED` under Nightly `Last task result: 0` |
| Pins | clap **4.6.1**, rusqlite **0.39.0** — no bump |
| `#211` Cursor | empty comments/reviews — no T301 |
| Hotspot #1 | `project.rs` — not touched |
| Ledger | 0 pending / 0 drift before BUGFIX TX |
| Did not | `cargo install`; schtasks mutate; grow doctor/daemon/project |

---

## AC9 live evidence

**Command:** `cargo run -p ai-brains-cli --quiet -- --no-project-context nightly --status --quick`  
**Exit:** 0

```
Nightly: AI-Brains-Nightly
…
Last task result: 0
…
Router: Ready
last run: terminated
…
```

Human stdout contains **neither** `267014` **nor** `SCHED_S_TASK_TERMINATED`.

**JSON:** `--format json --quick` → `router.last_result` `"267014"`, `last_result_hint` `"task terminated (SCHED_S_TASK_TERMINATED)"`, `status` `"Ready"`, probes `"skipped"`. Exit 0. Did **not** mutate schtasks.

---

## Findings (Phase-1)

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| — | — | No >low findings | — | AC1–AC6 red→green; AC7 decode stay-green; AC8 hermetic; AC9 live; clippy `-D warnings` exit 0 |

fmt only: rstest hex helper lines reformatted via `cargo fmt` after first `ledgerful verify --scope fast` (fmt was the sole fail; clippy/nextest/deny/audit already ok in that pass).

---

## DoD matrix

| Item | Status | Evidence |
|------|--------|----------|
| AC1 Ready+267014 | Met | unit red→green; const lock |
| AC2 Running+267009 | Met | status-only; JSON half frozen |
| AC3 blank/whitespace/hex | Met | terminated/running phrases + rstest F33/F34 |
| AC4 Ready+0/+1 / not scheduled | Met | unit + existing not-found |
| AC5 JSON frozen | Met | unit + live JSON |
| AC6 after_help + T269 needles | Met | new help unit + existing AC6 stay-green |
| AC7 explain decode | Met | `explain_last_task_result__267014/267009` green |
| AC8 hermetic | Met | contains-not 267014 / SCHED_S |
| AC9 manual | Met | transcript above |
| AC10 docs | Met | CAPABILITIES + OPERATIONS + CLI-EXIT-CODES + CHANGELOG |
| AC11 freeze / pins | Met | nightly.rs explain untouched; no lock bump |
| AC12 `--quick` no Llama | Met | freeze; hermetic probe=skipped |
| AC13 full gate | Met | `scripts/dev-check.ps1` SUCCESS (fmt/clippy/nextest/deny/audit) |
| AC14 capture independence | Met | status/docs only |

---

## Phase-1 internal

Orchestrator + live src audit vs every AC/F: **PASS**. No >low product findings. Freeze surfaces (`explain_last_task_result`, doctor, daemon, project, Cargo.lock) absent from product diff. Explore subagent hung mid-audit; evidence above is sufficient for Phase-1 clearance.

---

## Cross-model (`review.codex.md`)

| Finding | Disposition |
|---------|-------------|
| — | Codex **PASS** — no P0–P3 findings |
| Freezes (`explain_last_task_result`, doctor, daemon, project, Cargo.lock) | Confirmed untouched |
| T255 AC6/AC15 human supersede | Confirmed intentional human-only |

Product DoD judged implemented. Procedural closeout + publish remain.
