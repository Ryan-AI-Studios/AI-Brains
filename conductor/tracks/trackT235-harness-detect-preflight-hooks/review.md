# T235 Review Log — Harness detect + preflight hook install UX

## Scope

Detect harnesses installed on machine; wiring status; `harness status|install|uninstall|reset-decline`; preflight Harness section + consent; AGY install writer + F34 Stop→agy-hook map; doctor soft 12th check; help_ia; capability honesty; docs.

Branch: `feat/T235-harness-detect-preflight-hooks`

## Reviewers / rounds

| Round | Source | Verdict | Notes |
|-------|--------|---------|-------|
| R1 | Internal subagent (explore) | **FAIL** | P1 preflight Refused as success; P2 auto_install non-TTY; P3 wrapper silent skip |
| R1 fix | Orchestrator | — | report_preflight_install; gate reorder; Write-Skip stderr; f34_map_contract_summary |
| R2 | Internal re-review | **PASS** | Prior findings verified fixed; no new P0–P2 |
| CX1 | Codex | **FAIL** | P1 corrupt prefs rewrite; P2 backend_pending status; governance note |
| CX1 fix | Orchestrator | — | save_prefs refuse; stamp backend_pending; probe finalize |
| CX2 | Codex | **FAIL** | P2 install-hooks when AGY absent |
| CX2 fix | Orchestrator | — | gate install-hooks on AGY present+ready |
| CX3 | Codex | **FAIL** | P2 install-hooks refuse still exit 0 |
| CX3 fix | Orchestrator | — | fail_on_error for explicit --install-hooks |
| CX4 | Codex final | **PASS** | No P0–P2; clean cross-model gate |

## DoD matrix (summary)

| AC / Req | Status | Evidence |
|----------|--------|----------|
| AC1–AC4 detect/wiring/dry-run | Met | hermetic unit tests |
| AC5/AC18 never-prompt | Met | should_prompt_install + stdin_mode |
| AC6 consent | Met | interpret + decline prefs |
| AC7 JSON F21 | Met | status_report__json_schema_order |
| AC8 unknown harness exit 2 | Met | smoke + fail_usage |
| AC9/AC22 doctor 12th | Met | health_check_order_names__fixed_matrix |
| AC10–AC11 help/docs/capability | Met | help_ia + antigravity supports_hooks |
| AC12–AC17 install/map/PATH | Met | install + agy_map + detect tests |
| AC19 formatter arity | Met | format_preflight_summary_lines unchanged |
| AC20 vault-free | Met | is_vault_path_free Harness |
| AC21 refuse corrupt | Met | install + preflight report_preflight_install |
| AC23 uninstall prefs | Met | uninstall_agy test |
| F25 auto_install non-TTY | Met | should_prompt__non_tty_auto_install__auto |
| F34 map | Met | pure map + PS mirror + dry-run contract |

## Findings disposition

| ID | Sev | Status | Notes |
|----|-----|--------|-------|
| R1-P1 | P1 | verified_fixed | report_preflight_install |
| R1-P2 | P2 | verified_fixed | auto_install before !is_tty |
| R1-P3 | P3 | verified_fixed | Write-Skip stderr |

## Gates (local)

```
cargo clippy -p ai-brains-cli --all-targets -- -D warnings  # ok
cargo nextest run -p ai-brains-cli -p ai-brains-adapters    # 725 passed
Manual: harness status / install --harness agy --dry-run / unknown foo exit 2
```

## Residual

- Grok/OpenCode/Claude/Codex backends → T237/T238+
- AGY history.jsonl project binding → T236
- fullyIdle hard re-queue → T236
- PS vs Rust F34 dual-impl residual (contract printed on dry-run)

## Completion decision

**Completed.** Codex final CX4 **PASS**. CI gate-windows/linux/macos green (run 31283622244). Squash-merged PR #101 as `b1a0ecc`. Residual backends T236–T239; soft fullyIdle hard policy T236.
