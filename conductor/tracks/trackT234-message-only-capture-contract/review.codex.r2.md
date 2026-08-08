**Findings**

- `P1` `crates/ai-brains-adapters/src/message_only.rs:225`, `:304`, `:352`: the sole-tool-JSON guard added in `filter_turn_with_ts` is real, but it is not applied in all assistant-producing paths. `classify_antigravity_step` keeps any non-empty `PLANNER_RESPONSE` text at [message_only.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/src/message_only.rs:225), `filter_grok_history_record` keeps any non-empty assistant `content` at [message_only.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/src/message_only.rs:304), and `filter_opencode_message` does the same at [message_only.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/src/message_only.rs:352). The actual guard only exists in [message_only.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/src/message_only.rs:61). That means assistant payloads like `{"tool_calls":[...]}` or `{"type":"tool_result",...}` are still ingestable through those entry points, which violates the track’s message-only contract and means the prior P1 is not fully closed across the shared SOOT surface. Coverage also misses this case: the new P1 test only exercises `filter_turn` at [message_only.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/src/message_only.rs:656), while the Grok/OpenCode fixture tests are happy-path only at [message_only_fixtures.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/tests/message_only_fixtures.rs:66) and [message_only_fixtures.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/tests/message_only_fixtures.rs:84).

**Prior Findings Verification**

- `P1` fixed locally for the simple role/content path: `is_sole_tool_json_payload` is implemented in [message_only.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/src/message_only.rs:61) and covered by [message_only.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/src/message_only.rs:656). AGY simple filtering and `agy-hook` now route through that path at [agy.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/src/agy.rs:50) and [agy_hook.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/agy_hook.rs:49). ProjectChat also routes through `filter_turn` at [antigravity.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/src/antigravity.rs:293). But because the harness-specific assistant constructors above still bypass the guard, I cannot verify P1 as fully closed for T234.
- `P2` malformed AGY lines fail-open: verified fixed in [agy.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/src/agy.rs:25) with coverage at [agy_parsing.rs](C:/dev/AI-Brains/crates/ai-brains-adapters/tests/agy_parsing.rs:64).
- Governance/marker closeout: not treated as an engineering gap for this review, per scope.

**Fresh Sweep**

- No second independent regression stood out beyond the incomplete P1 closure above.
- I could not rerun repo tooling in this read-only session: `ai-brains preflight --summary`, `ledgerful doctor`, and `ledgerful ledger status --compact` all failed with `unable to open database file`, and I did not run cargo gates.

**Verdict**

FAIL