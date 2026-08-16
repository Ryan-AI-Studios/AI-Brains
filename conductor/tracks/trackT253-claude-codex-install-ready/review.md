# T253 Review Log — Claude / Codex install_ready

**Track:** T253-ClaudeCodexInstallReady
**Status:** ✅ **Completed** — product DoD met; CX3 PASS is the fresh final gate
**Ledger TX:** `bf495396-a8b4-4014-932b-f40c181df28f` (FEATURE)

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| Internal completeness R1 | explore | FAIL | P1 AC20 unrecorded; P2 skill still pending |
| Internal correctness R1 | explore | FAIL | P1 probe false-ok; P2 query swallow; P2 deny_unknown_fields; P3 wrapper ids |
| Fix | general-purpose + orchestrator | — | Probe name/wrapper-token only; skip-on-query-error; accept_*_live_payload; wrappers forward uuid/turnId; skill five-ready; AC20 live dogfood |
| Internal completeness R2 | explore | PASS | Prior findings verified_fixed; no remaining P0–P2 |
| Codex CX1 | gpt-5.6-luna high | FAIL | Process P1 closeout + product P2 import query swallow |
| CX1 fix | orchestrator | — | Import `get_sync_state` / `get_max_turn_index` errors skip + `skipped_query` |
| Codex CX2 | gpt-5.6-luna high | FAIL | P2 stale Install help “AGY ready; others pending.” |
| CX2 fix | orchestrator | — | Help text five-ready; plan.md trailing space |
| Codex CX3 | gpt-5.6-luna high | **PASS** | Fresh final gate. No P0–P2. Prior findings verified. |

## Findings

| ID | Sev | Status | Description |
|----|-----|--------|-------------|
| IR1-P1 | high | verified_fixed | AC20 live dogfood unrecorded → dry-run + `--yes` both backends |
| IR1-P2 | medium | verified_fixed | `.claude/skills/ai-brains/SKILL.md` still said pending |
| CR1-P1 | high | verified_fixed | Probe Ok on generic `.ai-brains` / `ai-brains` (Grok merge false-ok) |
| CR1-P2a | medium | verified_fixed | Live hook `get_session_turns` swallow |
| CR1-P2b | medium | verified_fixed | `--payload` accepted unknown fields |
| CR1-P3 | low | verified_fixed | Wrappers omitted uuid/turnId |
| IR2-P3a | low | deferred | Doctor helper still has `backend pending (T253)` for synthetic `install_ready=false` rows |
| IR2-P3b | low | deferred | uninstall `serde_json::to_string(...).unwrap_or_default()` |
| IR2-P3c | low | deferred | Historical research banners mention `codex_hooks` as the stale claim |
| CX1-P1 | high | process | Phase 6 closeout still open during CX1 (T251/T252 pattern) |
| CX1-P2 | medium | verified_fixed | Import query `unwrap_or(None)` → skip + `skipped_query` |
| CX2-P2 | medium | verified_fixed | `HarnessCommands::Install` help “AGY ready; others pending.” |
| CX2-P3 | low | verified_fixed | plan.md trailing whitespace |

## Gate evidence

| Check | Result |
|-------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo nextest run --workspace` | **2907 passed**, 1 skipped |
| Import tests after CX1-P2 | 7 passed (`claude_import_t253` + `codex_import_t253`) |
| `cargo deny check` / `cargo audit` | not on PATH (same T251/T252 residual) |
| `ledgerful verify --scope full` | fmt/clippy/nextest ok; deny/audit fail missing binaries |
| AC20 live | PASS 2026-08-15 — see plan.md Phase 5 |
| Codex CX3 | **PASS** |

## AC20 excerpt

```
claude wiring=ok (ready)  install_ready=true
codex  wiring=ok (ready)  install_ready=true
preflight: all five wiring=ok (ready); no backend pending next
config.toml SHA256 630F5B5E…FA53FE unchanged
next: in Codex run /hooks and trust ai-brains-capture
```

## Completion decision

Engineering DoD met. Internal reviews clean. Findings greater than low resolved. Fresh Codex CX3 **PASS**. Conductor + deferred absorbed. Soft residuals F34 + deferred P3s recorded in `conductor/deferred.md`.
