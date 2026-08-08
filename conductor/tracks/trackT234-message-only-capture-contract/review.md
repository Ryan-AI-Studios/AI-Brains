# T234 Review Log — Message-only capture contract

## Scope

Shared pure SOOT `ai_brains_adapters::message_only`; migrate `extract_turns` + ProjectChat + agy-hook; fixtures AGY/Grok/OpenCode; docs CAPABILITIES/OPERATIONS/CHANGELOG; UTF-8-safe strip; never populate `IngestRequest.thinking`; F15 sole-tool JSON guard on all assistant paths; F41 fail-open AGY parse.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Internal subagent | PASS WITH DEFERRED P3 — ProjectChat bypass P2; fixture/docs P3s |
| R1 fix | Implementer | ProjectChat → `filter_turn`; thinking fixture; emoji user_query; synthetic dead branch; OPERATIONS honesty |
| R2 | Internal re-review | PASS WITH DEFERRED P3 (F24 only) |
| Codex r1 | gpt-5.4 high | **FAIL** — P1 sole tool JSON on simple path only; P2 AGY fail-open; governance timing |
| Codex r1 fix | Implementer | `is_sole_tool_json_payload` on `filter_turn`; AGY skip bad lines |
| Codex r2 | gpt-5.4 high | **FAIL** — sole-tool guard not on AGY/Grok/OpenCode assistant constructors |
| Codex r2 fix | Implementer | Guard on `classify_antigravity_step`, `filter_grok_history_record`, `filter_opencode_message` + unit |
| **Codex final** | gpt-5.4 high | **PASS WITH DEFERRED P3** (soft F24 only) — fresh clean final gate |

## Findings disposition

| ID | Sev | Status | Disposition |
|----|-----|--------|-------------|
| R1-P2 ProjectChat | P2 | verified_fixed | `parse_project_chat_file` → `filter_turn` |
| R1-P3 fixtures/docs | P3 | verified_fixed | thinking fixture, emoji user_query, synthetic, OPERATIONS |
| Codex-P1 sole tool JSON | P1 | verified_fixed | Guard on all assistant SOOT paths + tests |
| Codex-P2 AGY fail-open | P2 | verified_fixed | `parse_agy_transcript` skips bad lines |
| Codex-P2 governance timing | P2 | out_of_scope | Closeout after PR merge |
| F24 soft | P3 | deferred | Capture refuse `thinking: Some` — soft residual |

## Gates (local)

- `cargo fmt` clean
- `cargo clippy -p ai-brains-adapters --all-targets -- -D warnings` green
- `cargo nextest run -p ai-brains-adapters` **42** passed
- `cargo nextest run --workspace` **2238** passed (pre-final sole-tool path expand; re-run adapters green after)
- `cargo deny check` ok
- `cargo audit` allowed warnings only (pre-existing)
- CI: pending PR

## Soft residual (allowed)

- **F24:** capture warn+clear if `IngestRequest.thinking` is Some (adapters always set None)
- Full Grok synthetic taxonomy (T237)
- Live OpenCode export schema (T238)
- T235–T239 harness install/wire

## Completion decision

Engineering DoD met for AC1–AC16 / F1–F47 (soft F24 deferred). Cross-model final **PASS WITH DEFERRED P3**.  
PR #100 squash-merged `2ea8897` after CI Win/Linux/macOS green. Conductor **Completed**.
