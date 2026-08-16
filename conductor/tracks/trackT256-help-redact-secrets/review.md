# T256 Review Log — Help redacts secrets

**Track:** T256-HelpRedactSecrets
**Status:** ✅ **Completed** — product DoD met; full local CI green
**Ledger TX:** `6d57d26e-63d6-4fc6-a4c3-e6b4d949da3c` (SECURITY)

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| Internal completeness R1 | explore | PASS WITH DEFERRED P3 | Two easy P3s |
| Internal correctness R1 | explore | **PASS** | No P0–P3 |
| Fix | orchestrator | — | P3-1 vault-path `=value` lock; P3-2 comment tense |
| Internal completeness R2 | explore | **PASS** | Both P3s `verified_fixed`; no new findings |
| Codex CX1 | gpt-5.6-luna high | product **PASS** | Process P1 closeout only (same class as T255 CX1). No product P0–P3. |

## Findings

| ID | Sev | Status | Description |
|----|-----|--------|-------------|
| IR1-P3-1 | low | verified_fixed | F5 not hermetically locked — root help now asserts `[env: AI_BRAINS_VAULT_PATH=` |
| IR1-P3-2 | low | verified_fixed | Stale pre-F1 comments reworded as historical |
| CX1-P1-1 | process | addressed | Closeout + full gate were still open at CX1 time. Not a product defect. |

## Plan drift (not a finding)

AC3 was labeled a pre-F1 green guard. Unset clap still emits `[env: AI_BRAINS_KEY=]` (empty `=value`). Exact `[env: AI_BRAINS_KEY]` appears only after `hide_env_values`. Leak assert on AC3 was green before F1; slot assert is green-after-F1. Product F1 unchanged.

## Manual AC11 (source bin, dummy key, 2026-08-16) — classify only

| Surface | exact `[env: AI_BRAINS_KEY]` | `AI_BRAINS_KEY=x'` | dummy 64-hex | `AI_BRAINS_VAULT_PATH` shown |
|---------|------------------------------|--------------------|--------------|------------------------------|
| `--help` | true | false | false | true |
| `-h` | true | false | false | true |
| `help` | true | false | false | true |

Did **not** print or paste a live key. Did **not** `cargo install` the product remediator.

## Gate evidence

| Check | Result |
|-------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo nextest run --workspace` | **2984 passed**, 1 skipped |
| Targeted hermetics | `cli_help_secret_redaction` 7/7; `cli_help_ia` 7/7; `key_resolve` 9/9 |
| `cargo deny check` | PASS (0.20.2; wildcard path-dep warnings only) |
| `cargo audit` | PASS (0.22.2; 19 allowlisted warnings) |
| `ledgerful verify --scope full` | **Verification passed** |

First workspace nextest fail was **unrelated**: live daemon blocked `backup_restore__*` (T188 safety). Stopped daemon, re-ran; 2984 green. Not a T256 regression.

## Completion decision

Engineering DoD met. Internal reviews clean (>low resolved). Codex CX1 product PASS. Full gate green including deny/audit. Conductor + deferred absorbed. Soft residual F18 PATH-behind recorded in `conductor/deferred.md`.
