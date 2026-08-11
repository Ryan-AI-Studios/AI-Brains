# T229 Review Log — Nightly + local router ops

**Track:** T229-NightlyLocalRouterOps  
**Category:** FEATURE / BUGFIX / OPS / DOCS  
**Status:** ✅ **Completed** — product PR #140 `1ec9142`; Codex R2 **PASS**; final closeout

## Reviewers / rounds

| Round | Who | Verdict | Notes |
|-------|-----|---------|-------|
| R1 | Internal subagent (read-only) | **PASS WITH DEFERRED P3** | No P0–P2; product F1–F7/F13 Met |
| — | Orchestrator manual AC7–AC8 | Evidence attached | Live `nightly --status` exit 0 |
| Codex R1 | gpt-5.6-luna high | **FAIL / NOT COMPLETE** | P1 transport Down fallback; P1 query leak; P1 process; P2 log path/hermetic/F5/client |
| Fix loop | Orchestrator | P1/P2 product fixed | classify→Error; host:port strip ?#; hermetic loopback; smart-quote; log path; pure signal units |
| Codex R2 | gpt-5.6-luna high | **PASS** | All R1 P1/P2 product findings verified fixed; no new >low |
| Full gate | orchestrator | **PASS** | nextest 2593; deny ok; audit warnings only; clippy/fmt; ledgerful verify earlier |

## Scope

- `crates/ai-brains-brain/src/embeddings.rs` — F5 `truncate_for_embed`
- `crates/ai-brains-models/src/llama_cpp.rs` + `tests/llama_cpp_probe_health.rs` — F2 probe
- `crates/ai-brains-cli/src/commands/nightly.rs` — F1/F6/F13 + wiring
- `Docs/OPERATIONS.md`, `Docs/CAPABILITIES.md`, root `CHANGELOG.md`

## DoD summary

| Bucket | Result |
|--------|--------|
| F5 UTF-8 truncate | Met — floor_char_boundary; units CJK/emoji |
| F13 nil ProjectId | Met — never random default |
| F1 endpoints on status | Met — host:port + model + probe |
| F2 probe_health | Met — models crate; no CLI reqwest; 2s timeout |
| F6 Last Result | Met — Get-ScheduledTaskInfo primary; CSV next-run only |
| F3/F7 docs | Met — OPERATIONS dual schedule + router.bat + 101 |
| F4 gap-fill | Met — verify-only, no code change |
| AC1–AC6, AC9, AC13–AC15 | Met (automated) |
| AC7–AC8 | Met (manual 2026-08-11) |
| AC12 code constraints | Met |
| Full gate / Codex | Pending orchestrator |

## Findings disposition

| ID | Sev | Status | Disposition |
|----|-----|--------|-------------|
| Internal P3 manual AC7–AC8 | P3 | **verified_fixed** | Live status: Scheduled Yes, Last task result 101, Completion/Embedding lines, probe=timeout (router down), exit 0 |
| Internal P3 full gate / cross-model | P3 | open process | Orchestrator Phase 7 |
| Internal P3 smart-quote unit | P3 | **deferred** | Same floor_char_boundary path as CJK/emoji; optional residual |

## Manual evidence (AC7–AC8)

```text
$ cargo run -q -p ai-brains-cli -- nightly --status
=== Nightly Status ===
Scheduled: Yes (next run: 8/12/2026 3:00:00 AM)
Last task result: 101
Last nightly run: 2026-08-11T07:16:05.381882100+00:00
…
Completion: 127.0.0.1:8081  model=gemma-4-E4B-it-Q6_K.gguf  probe=timeout
Embedding: 127.0.0.1:8083  model=nomic-embed-text-v1.5  probe=timeout
Multi-import: …
======================
EXIT: 0
```

Router endpoints unreachable → probe timeout (Windows connect budget); status still exit **0** (AC8). Last task result **101** from Get-ScheduledTaskInfo (pre-F5 panic residual until next successful run).

## Automated evidence

| Suite | Result |
|-------|--------|
| brain truncate_for_embed | 4 passed |
| models probe_health wiremock | 5 passed |
| cli commands::nightly | 20 passed |
| clippy 3 crates -D warnings | PASS |
| cargo fmt --check | PASS |

## Soft residuals (post-close, not DoD)

F8 doctor matrix; F9 persist probe; F10 schedule registers Router; F11 Router ONLOGON 267014; F12 JSON status; F14 embed 50ms sleep; optional smart-quote unit.

## Final disposition

| Item | Result |
|------|--------|
| Product PR | #140 squash-merged `1ec9142` |
| CI | Win/Linux/macOS green (Linux dead_code fix `3692d26` included) |
| Codex R2 | **PASS** (no P0–P2) |
| Series T217–T232 | **Closed** |
| Soft residuals | F8 doctor ports; F9 persist probe; F10 schedule Router; F11 Router ONLOGON; F12 JSON status; F14 50ms sleep |
| Multi-root | **T233** (unchanged) |
