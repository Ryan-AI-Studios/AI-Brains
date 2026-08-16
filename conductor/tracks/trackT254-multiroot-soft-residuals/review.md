# T254 Review Log — Multi-root soft residuals

**Track:** T254-MultiRootSoftResiduals
**Status:** ✅ **Completed** — product DoD met; CX2 PASS is the fresh final gate
**Ledger TX:** `4e796ab0-aefd-4de4-9023-d62fbd278dd0` (FEATURE)

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| Internal completeness R1 | explore | PASS | Product F/AC wired. P3s only. |
| Internal correctness R1 | explore | PASS | No P0/P1. Invariants hold. |
| Fix | orchestrator | — | Easy P3s: CLI-EXIT-CODES `--format` wording; stale F21 steal comment; hermetics for marked scan-root, grandchild depth, empty unregister. |
| Internal completeness R2 | explore | PASS | Prior P3s verified in tree; no new P0–P2 |
| Codex CX1 | gpt-5.6-luna high | product PASS / process P1 | 0 product P0–P2. Process P1 = closeout pending (T251/T252/T253 pattern). P3 trailing space fixed. |
| Codex CX2 | gpt-5.6-luna high | **PASS** | Fresh final gate. No P0–P2. No qualifying P3. Prior CX1 P3 verified_fixed. |

## Findings

| ID | Sev | Status | Description |
|----|-----|--------|-------------|
| IR1-P3a | low | verified_fixed | CLI-EXIT-CODES attributed unknown `--format` to `unregister-path` |
| IR1-P3b | low | verified_fixed | `register_path` comment still claimed UPSERT would steal |
| IR1-P3c | low | verified_fixed | AC10 scan-root-if-marked untested |
| CR1-P3a | low | verified_fixed | F21 depth-1 untested |
| CR1-P3b | low | verified_fixed | F35 empty-path unregister untested |
| IR1-P3d | low | deferred | AC3 `--format auto` TTY/pipe not hermetic |
| CR1-P3c | low | deferred | AC13 helper-vs-loop wiring |
| CR1-P3d | low | deferred | F16 no pin/symbol assertion after unregister |
| CR1-P3e | low | deferred | Scan/dry-run alias-count vs event-log length |
| CX1-P1 | high | process | Closeout pending at CX1 — not a product defect |
| CX1-P3 | low | verified_fixed | plan.md trailing whitespace |

## Gate evidence

| Check | Result |
|-------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo nextest run --workspace` | **2935 passed**, 1 skipped |
| Targeted hermetics | **21 passed** (`project_path_aliases` + `project_register_path`) |
| `cargo deny check` / `cargo audit` | not on PATH (same T251/T252 residual) |
| `ledgerful verify --scope full` | fmt/clippy/nextest ok; deny/audit fail missing binaries |
| Live list-paths empty | PASS — `No path aliases registered.` + next-step; JSON `{api_version:"1",paths:[]}` |
| Live `scan-roots C:\dev --format json` | PASS — 18 roots (C:\dev + 17 children), all `registered_project_id: null`, 0 writes |
| Codex CX2 | **PASS** |

## Completion decision

Engineering DoD met. Internal reviews clean. Findings greater than low resolved. Fresh Codex CX2 **PASS**. Conductor + deferred absorbed. Soft residuals F12 + deferred P3s recorded in `conductor/deferred.md`.
