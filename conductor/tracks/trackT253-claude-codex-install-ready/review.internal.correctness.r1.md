# T253 Correctness Review

**Reviewer:** Grok (read-only correctness r1)
**Scope:** T253 Claude/Codex `install_ready` writers, wrappers, hook/import CLIs, adapters, schemas
**Date:** 2026-08-15

## Verdict: FAIL

Writers, stdout contracts, merge/JSONC/config.toml, F23 wrapper fail-open, message-only filters, nightly exclusion, C7 hermetic targets, and session-id UUID handling are largely in good shape. The track still ships a dishonest `wiring=ok` probe (F20 / F23) and a live-hook delta-sync swallow that Grok already got right. Both have tests that would still pass on the old/wrong behavior.

## Findings (P0-P3 format with evidence)

### P1 — Probe false-ok: parsed JSON does not require managed `name` (F20); generic `.ai-brains` token collides with Grok merge (F23)

**Where:** `crates/ai-brains-cli/src/harness/wiring.rs` `probe_claude` (182–216), `probe_codex` (218–241); helpers `hooks_json_has_managed_name` / `json_value_contains_token` in `install.rs`.

**Spec:** F20: ok via managed name **or** wrapper path token; **“keep today’s substring probe plus require the managed `name` when JSON parses.”** F23: Grok still merges `~/.claude/settings.json`. F31 honesty: `wiring=ok` must not be fake ready.

**Bug:** When settings/hooks JSON **parses**, status is still

`named || path_tok || legacy`

not “require `name` when parse succeeds.”

Claude additionally treats **unparseable** bytes as `Ok` if the raw file contains `.ai-brains`+`hooks` or `ai-brains-capture`.

Codex, even on a successful parse, treats **any** raw substring `ai-brains` as ok (`raw.to_ascii_lowercase().contains("ai-brains")`).

`path_tok` includes the generic token `.ai-brains` (not only `claude-capture.ps1` / `codex-capture.ps1`). Any Grok/AGY command string under `~\.ai-brains\hooks\…` inside Claude `settings.json` makes **Claude** `wiring=ok` without UPS/Stop/SessionEnd named handlers.

**False-ok examples (no install):**

- `{"theme":"dark","note":"see .ai-brains hooks later"}` → Claude `Ok` (legacy AND on the serialized text).
- `{ not json !! ai-brains-capture` → Claude `Ok` (parse-fail + legacy).
- `{"hooks":{},"author":"ai-brains fan"}` → Codex `Ok`.
- Grok-merged settings whose hook `command` is `…\.ai-brains\hooks\grok-capture.ps1` → Claude `Ok` via `.ai-brains` token walk.

After T253, `install_ready=true` + `wiring=ok` is the operator signal that capture is installed (`next_action` becomes `harness status`; doctor “ready wired”). False-ok hides a missing writer.

**Not a miss:** real `install_claude` / `install_codex` do write `name: ai-brains-capture` and the dedicated wrapper path. Probe after a hermetic install is correctly `Ok` — that is the only case tested.

---

### P2 — Live hook `get_session_turns` swallow vs Grok’s fail-closed delta skip

**Where:** `crates/ai-brains-cli/src/commands/claude_hook.rs` 77–79; `codex_hook.rs` 77–79.

```rust
let existing = query_store
    .get_session_turns(&session_id.to_string())
    .unwrap_or_else(|_| Vec::new());
```

Then skip only when `(role, content)` already exists.

**Contrast:** `grok_hook.rs` 94–97 uses `get_max_turn_index(...).map_err(...)?`. Query failure **does not ingest**. Wrapper still fail-opens stop (F8/F9 `2>&1` + exit 0).

**Effect:** vault lock / projection query errors look like “no turns yet” → re-append the same user/assistant text. Event log is append-only; this is duplicate memory, not a panic. Content-based skip also drops a second genuine turn that happens to reuse the same role+text (`ok` / `thanks`).

Batch import `get_max_turn_index(...).unwrap_or(None)` matches AGY/Grok/OpenCode import and is **not** scored as a T253 regression.

---

### P2 — `deny_unknown_fields` is not on the live `--payload` path (dishonest vs schema / F14)

**Where:**

- Schemas: `Docs/schemas/claude-hook-payload.json`, `codex-hook-payload.json` (`additionalProperties: false`).
- Structs: `ClaudeHookPayload` / `CodexHookPayload` (`deny_unknown_fields`).
- Live CLI: `claude_hook::run` / `codex_hook::run` parse `serde_json::Value` then `map_*_hook_payload`. **`parse_*_hook_payload_strict` is never called** from `main.rs`.

**Spec:** F14: payload `deny_unknown_fields`.

**Effect:** `--schema` advertises a closed object. `--payload` accepts unknown vendor keys (`transcript_path`, `historyPath`, …). That is reasonable for vendor stdin, but the strict parser + tests do not protect the CLI contract. Mid-payload **garbage** still errors (`from_str` fail → CLI 1); extra fields do not.

---

### P3 — Wrappers drop vendor `turn_id` / `uuid` (F15 incomplete; ingest currently ignores `turn_id`)

**Where:** `install.rs` Claude `payloadObj` (1182–1188) and Codex (1393–1399) only forward `sessionId`, `projectHash`, `event`, `prompt`, `lastAssistantMessage`.

F15: live `v5(session, "{event}:{turn_id-or-stable}")`, prefer Claude `uuid` / Codex `turn_id`.

Result: every UPS in a session hashes `{event}:stable` (or `{event}:stable:{role}` if both roles are present). `generate_*_live_turn_id` itself is correct; the wrapper never supplies the stable token.

**Mitigation (do not over-rank):** `build_user_prompt` / `build_assistant_final` do **not** persist `IngestRequest.turn_id`; `turn_projection` keys on auto-increment `turn_index`. Collision is a contract hole, not a last-write-wins overwrite **today**.

---

### P3 — Doctor still hardcodes `T253` on a now-dead pending arm

**Where:** `doctor.rs` `doctor_harness_wiring_message` 822–836. After F2 every `HarnessId` has `install_ready() == true` / `pending_track() == None`, so `pending_present` is empty in production. A future sixth non-ready id would still be labeled `T253`. Leftover, not a live doctor lie when all five are ready (AC13 synthetic path is correct).

---

### P3 — `install.rs` uninstall compare uses `unwrap_or_default` on serialize

**Where:** `uninstall_official_hooks` 1663–1667: `serde_json::to_string(&root).unwrap_or_default()`. Not a panic. If serialize failed, `before == after` would skip the rewrite. Unlikely for `Map<String, Value>`.

---

### Cleared (hunt list — no defect found)

| Hunt item | Result |
|-----------|--------|
| Production `unwrap`/`expect`/`panic!`/`unreachable!` in new hook/install/adapter **prod** paths | None. F4 matches are exhaustive five-variant (no `unreachable!`). `pending_track().unwrap_or` is Option defaulting. Adapter `expect`s are `#[cfg(test)]` only. |
| Wrapper stdout leaks (missing `2>&1`, `Write-Host`, Claude decision JSON, wrong Codex stdout) | Claude: child `2>&1` → stderr; host stdout empty; `exit 0`; no `decision` / `continue`. Codex: same capture then `[Console]::Out.Write('{"continue":true}')`; const `codex_wrapper_continue_stdout()`; no `Write-Host`; no `render_hook_output` / `wrapper_command`. |
| Merge destroying foreign keys | `serde_json::Map` only; inserts matcher groups; replaces only `name == ai-brains-capture`; PreToolUse / top-level `theme`/`keep` preserved (AC3/AC5 tests). |
| JSONC rewrite | `json_has_line_comments` + serde fail → `Refused`, bytes unchanged (AC18). |
| `config.toml` writes | Read-only `codex_features_hooks_disabled`; install/uninstall tests assert bytes unchanged; never writes `codex_hooks`. |
| Grok-shaped stdin blocking stop (F23) | Wrapper: missing `hook_event_name` → skip + `exit 0` (Claude empty / Codex continue). CLI: `map_*` returns `None` → `Ok(())`. Adapter tests cover Grok camelCase-only. |
| `thinking` not `None`; tool/thinking ingested | Hook `IngestRequest.thinking: None`. `filter_turn` + `extract_text_from_json_content` drop thinking/tool parts. Import fixtures assert no `secret-think` / `bash` / `event-noise`. |
| Session id invalid UUID panic | `session_id_from_claude` / `_codex`: parse UUID or `v5(NAMESPACE_URL, raw)`. No `Uuid::parse_str.unwrap`. Empty wrapper session → `claude-unbound` / `codex-unbound`. |
| C7 writes outside temp home | Install/import tests take `tempdir` / `home_override: Some(home)`. Plans `starts_with(home)`. |
| Nightly / `run_multi_harness_import` including Claude/Codex | `multi_import.rs` remains agy → grok → opencode only; no Claude/Codex skip flags or report fields. |

## Tests that are weak

These would **not** fail on the pre-T253 or incorrect behaviors above.

1. **`wiring__claude_after_real_install__ok` / `wiring__codex_after_real_install__ok`** — only assert `Ok` after a real writer. No case: parseable JSON with `.ai-brains` / `ai-brains` substring and **no** `name: ai-brains-capture` must be `Missing`. Old substring probe still passes.

2. **`claude_and_codex_wrapper__capture_then_emit_contract` (AC6)** — string `contains("2>&1")` / `!contains("decision")`. Does not execute PowerShell. Does not assert vendor `turn_id`/`uuid` are forwarded. Does not feed Grok `hookEventName` stdin through the wrapper.

3. **`claude_hook_payload__deny_unknown_fields` / Codex twin** — exercise unused `parse_*_strict`. CLI `--payload` never calls them. Removing `deny_unknown_fields` from the live path would not fail these tests.

4. **`filter_claude_jsonl_lines__thinking_none_on_kept`** — name claims thinking is None; body only asserts `source_ts.is_none()`. No `IngestRequest` is built. AC7 “`thinking` None on ingest request” is untested on the hook CLI.

5. **No `claude_hook` / `codex_hook` unit or hermetic tests** — F23 skip, empty prompt skip, `thinking: None`, and `get_session_turns` **Err** → no ingest are untested. Adapter `map_*` tests would still pass if the CLI swallowed query errors.

6. **Doctor `doctor_harness_wiring_message__separates_ready_from_pending`** — still constructs Claude/Codex with `install_ready=false` and asserts `2 backend pending (T253): claude, codex`. That locks the **dead** formatter arm, not production `HarnessId::install_ready()`. Forgetting F2 on `detect.rs` would still pass this test. AC13 synthetic all-five-ready is the only new lock.

7. **Import hermetics (AC10/AC11)** — good keep/drop lists, `home_override`, dry-run zero writes. They do not lock `allow_default_project: false` against a default-project bind, nor query-error delta behavior.

8. **`install_pending__claude_real__stamps_prefs_no_fake_ok`** — still documents/tests the pending stub after Claude is ready. Does not prove `run_install` never takes that arm.

9. **`install_ready__agy_and_grok`** — now asserts all five ready + `pending_track` is not `T253`/`T239+`. This one **would** fail if F2 were reverted. Keep it; it is not weak.

## Residual risks

- **`preflight --install-hooks` still only loops Agy/Grok/Opencode** (`preflight.rs` ~918–947). Spec isolation said do not grow that hotspot; operators who only run `--install-hooks` will not wire Claude/Codex. Honesty depends on `harness install` / `all-ready`.
- **Codex `/hooks` trust** (F22): `wiring=ok` is files only. Install stdout prints the next-action; live fire is still operator-trust. Not DoD-blocked.
- **Codex Stop JSON on Windows + non-ASCII** (upstream issue #23784): wrapper `catch` fail-opens with `{"continue":true}` and skips ingest. Correct fail-open; missed turns until import.
- **Live SessionEnd ingest** can exceed Claude’s 1.5s shared budget (F34). Primary path is UPS+Stop.
- **`IngestRequest.turn_id` unused by capture payloads** — F15 IDs are currently ornamental. If a later track keys projections on `turn_id`, wrapper omission (P3) becomes data loss.
- **Content-based live skip** can drop a repeated identical user line in one session even when the query succeeds.
- **`codex_features_hooks_disabled`** is a line scanner (`hooks` + value `starts_with("false")`). Quoted `"false"` is missed; `falsehood` is a false positive. Warn-only; no toml write.
- **Capability `Full` + `supports_hooks: true`** remains; notes were rewritten. Still an overclaim vs nightly / `/hooks` trust — accepted F25.
- **No Unix `.sh` wrappers** (F34). Windows DoD only.

## Suggested fix order (not in this review’s scope)

1. When JSON parses: `Ok` only if `hooks_json_has_managed_name` (and optionally a **dedicated** wrapper token `claude-capture.ps1` / `codex-capture.ps1`). Drop raw `contains("ai-brains")` and generic `.ai-brains` on the parse-ok arm. Parse-fail: `Unknown` or `Missing`, not `Ok`.
2. Add hermetic probe tests that fail on substring-only fixtures and on a Grok wrapper path inside Claude settings.
3. Live hooks: on `get_session_turns` / `get_max_turn_index` error, skip ingest (stderr once) and still exit 0 from the wrapper — match Grok. Add a test that injects a failing `QueryStore`.
4. Either run `parse_*_strict` on wrapper-built `--payload`, or stop advertising `additionalProperties: false` as the live contract.
5. Forward `turn_id` / `uuid` from vendor stdin in both wrappers (cheap, unblocks F15 if capture starts persisting it).
