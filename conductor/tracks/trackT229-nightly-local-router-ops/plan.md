# T229 Plan — Nightly + local router ops

**Status:** Planning + **AI fold-in 2026-08-11** (plan-only until **go**)  
**Spec:** [spec.md](./spec.md) §15 disposition  
**Category:** FEATURE / BUGFIX / OPS / DOCS  
**Ledger TX (on go):** `ledgerful ledger start T229-nightly-local-router-ops --category FEATURE --message "Nightly probe+LastResult+UTF8 truncate+nil project; no CLI reqwest"`

## Preflight (done at plan time)

- [x] Live dogfood: schedule Ready, Last Result **101**, router 200, panic `embeddings.rs:69`
- [x] Live schtasks **CSV = 3 cols only** (no Last Result) — F6 revised
- [x] Confirmed CLI has **no reqwest**; models has reqwest + wiremock
- [x] Confirmed `ProjectId::default()` = random v4; nightly warning dead
- [x] AI1 + AI2 fold-in → spec §15
- [x] Pin planning decision to vault (earlier)

## Phase 0 — Ledger + impact (on go)

- [ ] `ledgerful ledger status --compact`
- [ ] `ledgerful ledger start T229-nightly-local-router-ops --category FEATURE --message "…"`
- [ ] `ledgerful scan --impact` on embeddings / llama_cpp / nightly

## Phase 1 — Red → Green: UTF-8 embed truncate (F5 / AC1–AC2)

- [x] Red unit multi-byte @ ~4000
- [x] Green: `truncate_for_embed` with `floor_char_boundary(max.min(len))`
- [x] `cargo nextest run -p ai-brains-brain --lib`

## Phase 2 — Red → Green: project resolve (F13 / AC13–AC14)

- [x] Pure `resolve_nightly_project_id` → nil UUID, never random
- [x] Fix dead warning (`== nil`)
- [x] Units AC13–AC14

## Phase 3 — Red → Green: status endpoints + Last Result (F1 / F6 / F6b / AC3–AC5 / AC7)

- [x] `format_endpoint_line` host:port + credential redaction
- [x] Next-run from CSV cols 0–2 (quote-safe)
- [x] Last Result from **Get-ScheduledTaskInfo** (not CSV col 5)
- [x] Units AC3–AC5; manual AC7

## Phase 4 — Soft probe (F2 / AC6 / AC8) — **models crate**

- [x] `LlamaCppProvider::probe_health(&self, timeout: Duration) -> ProbeStatus`
- [x] `/health` then `/v1/models`; timeout **not** LLM 120s
- [x] wiremock tests in **ai-brains-models** (AC6)
- [x] Call from `--status` and **after multi-import / before summarize**
- [x] Non-fatal warn SOOT; status exit 0 on down
- [x] **Do not** add reqwest to CLI

## Phase 5 — F4 gap-fill verify

- [x] Confirm global dotenv → schedule wrapper keys (expect no code change)
- [x] Fix only if broken — verified: `generate_nightly_wrapper_script` reads process env after T205 global dotenv merge; comment added; no code change required

## Phase 6 — Docs (F3 / F7 / AC10–AC11 / AC15)

- [x] `Docs/OPERATIONS.md` — router.bat, ports, dual schedule, log, Last Result 101
- [x] `Docs/CAPABILITIES.md` — status endpoint/probe/Last Result
- [x] **`CHANGELOG.md` (repo root)** — T229 row only

## Phase 7 — Review + gate

- [x] Internal review vs spec — CLEAN (PASS WITH DEFERRED process P3 only)
- [x] **Cross-model** Codex R1 FAIL → fix → R2 **PASS** (F5 + all product)
- [x] Full gate: fmt, clippy `-D warnings`, nextest **2593**, deny, audit
- [x] `ledgerful verify --scope full` (pre-fix) + ledger TX open
- [ ] Conductor Completed; deferred + series README strike T229 (closeout PR)
- [ ] `ai-brains pin "DECISION: T229 closed — …"` (closeout)

## Explicit non-goals

- T233 multi-root
- Bundle router / add reqwest to CLI / TCP-only primary probe
- Doctor model matrix; JSON status; embed 50ms sleep retune
- Change `run_nightly` signature to `Option<ProjectId>`

## Evidence log (fill on implement)

| AC | Command / proof | Result |
|----|-----------------|--------|
| AC1–AC2 F5 | `cargo nextest run -p ai-brains-brain --lib` (4 truncate tests) | PASS (35/35 package) |
| AC13–AC14 F13 | `cargo nextest run -p ai-brains-cli -E 'test(commands::nightly)'` resolve units | PASS |
| AC3–AC6 F1/F2/F6 | CLI format/CSV units + `ai-brains-models` probe_health wiremock (5) | PASS |
| AC7–AC8 manual | `cargo run -q -p ai-brains-cli -- nightly --status` — Scheduled Yes, Last task result 101, Completion/Embedding probe lines, exit 0 (router down → probe=timeout) | PASS 2026-08-11 |
| AC10–AC11 / AC15 docs | OPERATIONS + CAPABILITIES §8 + root CHANGELOG | done |
| Full gate | clippy `-D warnings` on 3 crates + `cargo fmt --check` | PASS |
| F4 | Wrapper reads process env post T205 global dotenv; comment only | verify-only, no code change |
