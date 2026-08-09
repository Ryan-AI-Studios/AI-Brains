# T237 — Grok Build seamless ingest

- **Status:** 📋 **Planning** (plan-only until go; no production code / ledger TX)
- **Source:** Research 2026-08-08 — live `~/.grok` (Grok Build **1.0.0**), `~/.grok/docs/user-guide/10-hooks.md` + `17-sessions.md`, T234 fixture SOOT, T235 install marker, T236 dual-path lessons; crates.io pins (serde 1.0.229 / serde_json 1.0.151 / clap 4.6.x under caret — **no intentional bump**); **AI review fold-in 2026-08-08** (§14)
- **Category:** FEATURE
- **Depends on:** T234 (message-only SOOT ✅); T235 (detect/wiring/`harness *` ✅; Grok marker + `backend_pending`); T236 lessons (wrapper stdout, path normalize, unbound anti-hijack, path-keyed meta, turn-id SOOT)
- **Absorbs:** deferred “Grok Build hooks + chat_history batch”; T234 “wire T237” for `filter_grok_history_*`; T235 Grok install backend; **AI2 M1–M6** (empty Stop stdout, user_query-only user keep, subagent skip, percent encode helper, turn-id risk honesty, no `$` in command); **AI2 M7–M9 / AI1 path scan + multipart affirm**
- **Related:** series [README-T234-T239](../README-T234-T239-HARNESS-INGEST.md); Claude hooks (`Docs/claude-hooks.md`) as pattern only — **not** Claude/Codex install in this track
- **Does not absorb:** OpenCode (T238); multi-harness nightly orchestration / SYSTEM `--skip-import` re-enable (T239); Claude/Codex `install_ready` (remain pending / follow-up labels until a later track); T224 display strip; T220 preflight JSON harness array; project-scope hooks as default (C7); **optional** fingerprint turn-ids (soft residual if filter churn forces it)

## Objective

Make **Grok Build** capture **seamless and findable in the right project**:

1. **Live:** user-global hooks (`Stop` + `SessionEnd`) → thin wrapper → `grok-hook` ingests **message-only** turns from `chat_history.jsonl` with workspace→project binding.
2. **Batch:** `grok-import` discovers `~/.grok/sessions/**/chat_history.jsonl`, binds via `summary.json` (`cwd` / `git_root_dir`), same filter + turn-id SOOT as live.
3. **Install honesty:** `harness install --harness grok` writes real hooks (no `backend_pending` stamp); wiring probe `ok` when managed marker present.
4. **Docs:** CAPABILITIES / OPERATIONS / CHANGELOG / series README reflect Grok Full (with listed caveats).

## Diagnosis (frozen — 2026-08-08 live + docs)

| ID | Fact |
|----|------|
| **D1** | **Session tree:** `~/.grok/sessions/<URL-encoded-cwd>/<session-id>/` (GROK_HOME overrides `~/.grok`). Live AI-Brains group: `C%3A%5Cdev%5CAI-Brains`. Long encoded names may use slug+hash + `.cwd` file (docs). |
| **D2** | **Content SOOT:** `chat_history.jsonl` — typed records (`type`: `user` \| `assistant` \| `reasoning` \| `tool_result` \| `backend_tool_call` \| `system`; user `content` often array of `{type:text,text}`; real prompts wrapped in `<user_query>`). T234 `filter_grok_history_*` fixture-ready. |
| **D3** | **Binding index:** `summary.json` — `info.id`, `info.cwd`, `git_root_dir`, timestamps, model. **Not** content SOOT. |
| **D4** | **Noise (never content SOOT):** `updates.jsonl` (docs: resume authority — full of tools), `events.jsonl`, `rewind_points.jsonl`, `*.lock`, `system_prompt.txt`, subagent trees unless explicit follow-up. |
| **D5** | **Chrome user rows:** `synthetic_reason` e.g. `compaction_meta`, `system-reminder` inject large AGENTS/skills blobs as `type:user`. Must **not** land as user memories (empty after `extract_user_text` is insufficient for full system-reminder bodies). |
| **D6** | **Hooks (Grok 1.0.0 user-guide):** global `~/.grok/hooks/*.json` always trusted; project `.grok/hooks` needs trust. Events: `UserPromptSubmit`, `Stop`, `SessionEnd`, `SessionStart`, … stdin **camelCase** (`sessionId`, `cwd`, `workspaceRoot`, `hookEventName`, `timestamp`, `permissionMode`). Env: `GROK_SESSION_ID`, `GROK_WORKSPACE_ROOT`, `GROK_HOOK_EVENT`. |
| **D7** | **Stop gate:** can block stop (`decision: block`) or allow (exit 0 / empty). Input includes `reason` (`end_turn` vs session-end observe fires), `lastAssistantMessage`, `stopHookActive`, `backgroundTasks`. Fail-open on hook failure. **Capture path must never block stop.** |
| **D8** | **UserPromptSubmit:** documented as non-blocking; **prompt text field not clearly specified** in 10-hooks.md for reliable SOOT. Prefer **chat_history** for both user + assistant (parity live/batch). |
| **D9** | **T235 today:** `probe_grok` = file `~/.grok/hooks/ai-brains.json` → `ok`; real install calls `install_pending` → stamps prefs `backend_pending`; `install_ready()` **false** for Grok (`pending_track=T237`). |
| **D10** | **T236 lessons that apply:** (a) wrapper stdout never leaks human ingest prose; (b) path normalize before alias; (c) env project id **only** for unbound; (d) path-keyed `source_meta`; (e) unified turn-id live+batch; (f) `--force` for quiescence; (g) scheduled SYSTEM nightly may still `--skip-import` — honesty only (T239). |
| **D11** | **Dep pins:** workspace `serde`/`serde_json` 1.0, `clap` 4.5, `uuid` 1.13, edition 2024. crates.io latest under caret — **do not bump** for T237 unless gate forces. |
| **D12** | Live type histogram (sample session): tool_result ≫ reasoning/assistant/user; many assistant rows empty (filter drops empty); reasoning may carry `encrypted_content` (still type-drop). |
| **D13** | Claude/Codex detect still `T237+` pending — **out of T237 implement scope** (Claude already has separate `Docs/claude-hooks.md` scripts). |
| **D14** | **Grok Stop allow contract ≠ AGY (AI2 M1):** official 10-hooks.md — *“Allow the stop: exit 0 with no output (or any non-JSON output).”* Documented Stop decisions are `block` / `hookSpecificOutput` / `continue:false` only. Emitting `{"decision":"allow"}` is PreToolUse vocabulary and **undefined for Stop** (valid JSON may win over exit code). **Must not reuse `agy_wrapper_allow_stop_stdout()`.** |
| **D15** | **Chrome without `synthetic_reason` (AI2 M2):** live rows include `<user_info>`/`<git_status>` with **empty** `synthetic_reason`; `synthetic_reason` values seen: `compaction_meta`, `project_instructions`, `system_reminder`, `task_completed`. Today’s `extract_user_text` keeps non-empty chrome when no `<user_query>` → **leaks**. Live real prompts are **100%** `<user_query>`-wrapped in probed sessions. |
| **D16** | **No per-row timestamps/ids on user/assistant** (AI2 M5): only stable index after filter; `source_ts` always `None` for Grok → `occurred_at` = ingest time (batch backfill stamps “now”). |
| **D17** | **Subagent sessions live in normal sessions tree** (AI2 M3): e.g. worktree/subagent groups with real `chat_history.jsonl` — naive `**` walk ingests them unless skipped. |
| **D18** | **Grok expands `$VAR`/`${VAR}` in hook `command`** (AI2 M6). Absolute wrapper path must contain **no `$`**. Grok also merges `~/.claude/settings.json` + `~/.cursor/hooks.json` by default (AI2 M8) — future Claude hooks fire in Grok too. |

## Frozen direction

### A. Live hook path

| ID | Decision |
|----|----------|
| **F1** | **Managed install artifact:** write user-global `~/.grok/hooks/ai-brains.json` (T235 marker SOOT) + wrapper `~/.ai-brains/hooks/grok-capture.ps1`. Create `~/.grok/hooks/` if missing. **No** project `.grok/hooks` by default (C7). |
| **F2** | **Events wired:** `Stop` + `SessionEnd` only for capture DoD. Same wrapper command for both. |
| **F3** | **Stop filter:** process only when `reason` is missing **or** `reason == "end_turn"` **or** event is `SessionEnd`. Soft-skip other Stop observe fires (`channel_closed` / `shutdown`). Never block. |
| **F4** | **Content path:** resolve `chat_history.jsonl` from `sessionId` + `workspaceRoot`/`cwd` (env fallbacks). Parse with **`filter_grok_history_*` + F11**. Prefer file over `lastAssistantMessage` alone (live==batch). **Multipart (AI1 #3 affirm):** keep existing text-part join; discard tool_call parts; empty after strip → drop. Fail-open per malformed JSONL line. |
| **F5** | **CLI:** `ai-brains grok-hook --payload '{...}'`. Schema `Docs/schemas/grok-hook-payload.json`. Payload: `historyPath`, `sessionId`, `projectHash` (`workspace` or `grok-unbound`). Optional `event`. |
| **F6** | **Wrapper Stop stdout SOOT (AI2 M1 hard):** capture/suppress `grok-hook` stdout; host **stdout must be empty** (zero bytes preferred); **exit 0**; **never** emit `decision`, `continue`, `hookSpecificOutput`, or exit 2. Diagnostics **stderr only**. **Do not** call or copy `agy_wrapper_allow_stop_stdout()`. Dedicated `grok_wrapper_*` helpers + hermetic body test (AC12). |
| **F7** | **Session path resolve (elevated AI1 #1 + AI2 M4):** pure helpers in `grok.rs` (not PS): (1) `GROK_HOME` or `~/.grok`; (2) **hand-rolled percent-encode** workspaceRoot then cwd (RFC-3986 unreserved `A-Za-z0-9-._~`, uppercase `%XX`, space→`%20` — match live `C%3A%5Cdev%5CAI-Brains`); try `sessions/<enc>/<sessionId>/chat_history.jsonl`; (3) scan groups for `.cwd` match; (4) scan `summary.json` for `info.id == sessionId`; (5) fail-open skip. **Percent-decode** for group-name → path fallback. Hermetic fixtures from live encodings. |
| **F8** | **Delta + turn-id:** max-turn-index / existing-turn skip on **kept** index. Turn ids: **`v5(session, "turn-{i}")`** on kept `i` (unified live+batch). Document filter-version stability risk (F35); no per-row source_ts (D16). |
| **F9** | **UserPromptSubmit:** **not DoD**. Soft residual S1. |
| **F10** | Never populate `IngestRequest.thinking`. Capture independence: no models/embeddings/graph. |
| **F11** | **User keep rule (AI2 M2 hard):** for Grok `type:user`, **keep only** when content yields a non-empty body from **`<user_query>` or `<USER_REQUEST>`** (after extract). **Drop** when: any `synthetic_reason` present; pure chrome (`<user_info>`, `<git_status>`, system-reminder, project_instructions, task_completed, compaction continuation prose without user_query); empty after extract. **Tradeoff (documented):** bare-text user prompts without those tags are dropped until Grok changes shape — live probe 100% user_query. Fixture matrix AC2. |

### B. Batch import

| ID | Decision |
|----|----------|
| **F12** | **Command:** `ai-brains grok-import [--days N] [--force] [--dry-run]`. Discovery: walk sessions tree (GROK_HOME / hermetic home). **Skip subagent sessions by default (AI2 M3 hard):** path/group contains `subagent-` **or** `\.grok\worktrees\` / `/worktrees/` segment **or** summary `agent_name` is a subagent role (not main). Counter `skipped_subagent`. Opt-in include = soft residual S2 only. |
| **F13** | **Binding:** sibling `summary.json` → prefer `git_root_dir` then `info.cwd` → **normalize** → project alias. Missing → decode group name if possible → else `grok-unbound` / `(unbound Grok)`. |
| **F14** | **Unbound anti-hijack:** env `AI_BRAINS_PROJECT_ID` only when `grok-unbound`/empty. Default non-interactive `allow_default_project: false`. |
| **F15** | **Quiescence:** 300s on `chat_history.jsonl` mtime; `--force` skips. **Ignore `*.lock`** in discovery (AI1 #4). Fail-open incomplete JSON lines. |
| **F16** | **source_meta:** path-keyed (`source_meta:grok:{stable_path_key}`). |
| **F17** | **Stats (stderr honesty):** `found`, `imported_turns`, `sessions`, `skipped_quiescent`, `skipped_unchanged`, `skipped_subagent`, `unbound_project`, `bound_via_summary`, `bound_via_path`. |
| **F18** | **Never** ingest updates/events/rewind as content. |
| **F19** | Reuse T236 unsummarized OR-query (harness-agnostic). |
| **F20** | Hermetic tests: fake home + encoded cwd + summary + history (+ subagent negative). |

### C. Install / detect / wiring

| ID | Decision |
|----|----------|
| **F21** | `HarnessId::Grok.install_ready() → true`; `pending_track` → `None`. |
| **F22** | `install_grok`: write **only** our `ai-brains.json` + wrapper; **never** delete sibling hook JSON (AI1 #2 / AC11). Uninstall removes only managed pair; clear backend_pending prefs. |
| **F23** | Hooks JSON shape (official Quick Start). **timeout: 120** for Stop/SessionEnd (AI2 M7: 60 may be tight on first large-session reparse; fail-open on timeout; dogfood may retune). **command:** absolute path to wrapper; **no `$` / `${` characters (AI2 M6 / AC19)**. |
| **F24** | Dry-run zero writes; idempotent reinstall; corrupt our file → refuse. |
| **F25** | Wiring marker-file `ok` when present. |
| **F26** | No PreflightContextResponse growth. |
| **F34** | Hermetic assert: installed command string contains no `$` (AC19). |

### D. Docs / capability / series

| ID | Decision |
|----|----------|
| **F27** | CAPABILITIES: Grok **Implemented** with caveats: user_query-only user keep; subagent skip default; **source_ts none / occurred_at=ingest-time**; empty Stop stdout; no UserPromptSubmit; SYSTEM skip-import honesty; **vendor-compat note (AI2 M8):** Grok may also load Claude/Cursor hooks — dual-fire possible later. |
| **F28** | OPERATIONS: install + `grok-import` + session layout + Stop empty-stdout contract. |
| **F29** | CHANGELOG + series README. |
| **F30** | Prefer OPERATIONS/CAPABILITIES over new long doc. |
| **F31** | Zero new crates; no dep bump; **hand-rolled percent codec in grok.rs** (not new dep). |
| **F32** | Schema + `--schema` for grok-hook payload. |
| **F33** | Claude/Codex `install_ready=false`; pending labels **not** “T237” (use follow-up / T238+). |
| **F35** | **Turn-id stability honesty (AI2 M5):** document that kept-index ids shift if filter taxonomy changes → risk of duplicates; pin filter contract in CAPABILITIES; optional fingerprint residual **S8**. |

### E. Soft residuals (not DoD)

| ID | Residual |
|----|----------|
| **S1** | UserPromptSubmit live prompt field if proven. |
| **S2** | **Opt-in** subagent session ingest (default skip is hard F12). |
| **S3** | Byte-offset watermark vs O(n) re-parse. |
| **S4** | Import `--json` machine report. |
| **S5** | Project-local hooks install opt-in. |
| **S6** | Wire Claude/Codex install_ready (later track). |
| **S7** | Nightly multi-harness orchestration (T239). |
| **S8** | Fingerprint turn-id if filter churn forces re-key (F35). |

## Non-goals

- Ingesting `updates.jsonl` / full tool traces / encrypted reasoning.
- Blocking Grok Stop for tests/linters (capture is observe-only).
- Emitting AGY-style `{"decision":"allow"}` on Grok Stop (wrong contract — F6).
- Auto-trust of project hooks.
- Cloud Grok.com session import.
- OpenCode / multi-harness nightly (T238/T239).
- Claude/Codex seamless install rewrite.
- Re-enable SYSTEM scheduled import (T239).
- Default subagent session import (skipped unless S2 opt-in).
- Dep version bumps for their own sake.
- MSI packaging.

## Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | Fixture `chat_history` (user/assistant/reasoning/tool_result/backend_tool_call/system) → **only** user+assistant text; zero tool/reasoning/system. |
| **AC2** | **Live chrome matrix (AI2 M2):** (a) `synthetic_reason` set; (b) `<user_info>`+`<git_status>` **without** synthetic_reason; (c) system-reminder / project_instructions / task_completed; (d) compaction continuation prose without `<user_query>` → **zero** user memories. Real `<user_query>` row kept. |
| **AC3** | Hermetic `grok-hook` → appends kept turns; thinking None. |
| **AC4** | Same fixture → same turn ids live vs batch (`turn-{i}`). |
| **AC5** | Batch summary bind → correct project alias (not env hijack). |
| **AC6** | Unbound → `grok-unbound`; env only when unbound. |
| **AC7** | Re-import unchanged → zero duplicate turns. |
| **AC8** | Quiescence skip; `--force` imports. |
| **AC9** | Real install writes marker + wrapper; `install_ready`; wiring `ok`; no backend_pending. |
| **AC10** | Dry-run zero writes. |
| **AC11** | Uninstall managed only; **foreign sibling hooks preserved**. |
| **AC12** | **Grok Stop stdout empty** (AI2 M1): wrapper body asserts empty allow path (no `decision`/`continue`/`hookSpecificOutput` keys); exit 0; grok-hook human stdout not forwarded; **not** AGY allow JSON. |
| **AC13** | Path case variants → same project alias. |
| **AC14** | `updates.jsonl` never content SOOT. |
| **AC15** | CAPABILITIES/OPERATIONS/CHANGELOG; Grok pending cleared; honesty F27/F35. |
| **AC16** | Corrupt managed file → refuse. |
| **AC17** | Capture independence (no models/graph on path). |
| **AC18** | **Subagent skip (AI2 M3):** fixture subagent/worktree session not imported by default; `skipped_subagent` increments. |
| **AC19** | **No `$` in installed command (AI2 M6);** percent-encode live path shape (AI2 M4): `C:\dev\AI-Brains` → group `C%3A%5Cdev%5CAI-Brains` (hermetic). |
| **AC20** | Resolve fallback: `.cwd` group and/or `summary.info.id` finds session when direct encode miss (AI1 #1). |

## Risks

| Risk | Mitigation |
|------|------------|
| Wrong Stop JSON blocks or confuses Grok | **F6/AC12** empty stdout pin |
| Chrome leak without synthetic_reason | **F11/AC2** user_query whitelist |
| Subagent pollution | **F12/AC18** default skip |
| Encode mismatch | **F7/AC19–AC20** hand-rolled codec + multi fallback |
| Turn-id shift after filter change | **F35** docs; S8 fingerprint later |
| `$` expansion in command | **F34/AC19** |
| Large first Stop timeout | timeout 120; fail-open; dogfood |
| Hook stdin schema drift | ignore unknown keys |
| Mid-write chat_history | fail-open lines; locks ignored; quiescence |

## Verification plan

1. **Red/Green unit:** filter synthetic; path resolve; install merge/uninstall; wrapper body assertions.  
2. **Hermetic import/hook:** temp home + fixture tree.  
3. **Targeted:** `cargo nextest` adapters + cli harness packages; clippy package.  
4. **Manual (go day):** `harness install --harness grok`; `/hooks-list` or Hooks tab shows Stop/SessionEnd; one dogfood turn; `grok-import --days 1`; scoped `recall` sees content under AI-Brains project.  
5. **Full gate** before finalize.  
6. **Review:** internal + codex-review (FEATURE / capture surface).

## Dependency research (2026-08-08)

| Item | Pin / source | Action |
|------|----------------|--------|
| Grok Build CLI | **1.0.0** stable live (`grok --version`) | Target |
| Hooks docs | `~/.grok/docs/user-guide/10-hooks.md` | SOOT for events/stdout |
| Sessions docs | `17-sessions.md` | Layout; chat_history vs updates |
| serde / serde_json | workspace 1.0; crates.io 1.0.229 / 1.0.151 | No bump |
| clap | workspace 4.5; crates.io 4.6.6 | No bump |
| uuid | 1.13 v4/v5 | Keep |
| T234 filter | `ai_brains_adapters::message_only` | Wire only; extend F11 if needed in same module or grok adapter thin layer |

## Files likely touched

| Area | Path |
|------|------|
| Filter extend | `crates/ai-brains-adapters/src/message_only.rs` (+ live chrome fixtures) |
| Grok lib | `crates/ai-brains-adapters/src/grok.rs` (new) — discover, **percent encode/decode**, path resolve, import, subagent skip |
| Hook CLI | `crates/ai-brains-cli/src/commands/grok_hook.rs` |
| Import CLI | `grok_import.rs` + `main.rs` + help_ia |
| Install | `harness/{install,detect,wiring}.rs` — **Grok empty-stdout wrapper**, not AGY allow JSON |
| Schema | `Docs/schemas/grok-hook-payload.json` |
| Docs | CAPABILITIES, OPERATIONS, CHANGELOG, series README |
| Conductor | this track, `conductor.md`, `deferred.md` |

## §14 AI review fold-in (2026-08-08)

| Item | Severity | Disposition |
|------|----------|-------------|
| **AI2 M1** Grok Stop = empty stdout, not AGY allow JSON | High | **F6 hard** / **AC12** |
| **AI2 M2** chrome without synthetic_reason; user_query-only keep | High | **F11 hard** / **AC2** matrix; Phase 1 blocking Red |
| **AI2 M3** subagent walk pollution | Medium | **F12 hard** / **AC18** (S2 = opt-in only) |
| **AI2 M4** percent encode/decode helper | Medium | **F7 hard** / **AC19** |
| **AI2 M5** turn-id + source_ts honesty | Medium | **F8/F35** / CAPABILITIES; fingerprint **S8** |
| **AI2 M6** no `$` in command | Medium | **F23/F34** / **AC19** |
| **AI2 M7** timeout 60 vs 600 | Low | **F23** default **120**; dogfood retune |
| **AI2 M8** Claude/Cursor vendor merge | Low | **F27** caveat |
| **AI2 M9** Phase 1 Red = live chrome | Low | plan Phase 1 reorder |
| **AI1 #1** path resolve steps | — | absorbed **F7/AC20** |
| **AI1 #2** foreign hooks isolation | — | already F22/AC11 (affirmed) |
| **AI1 #3** multipart assistant | — | affirm **F4** (existing filter) |
| **AI1 #4** lock + fail-open lines | — | **F15** affirmed |

**Declined / deferred:** fingerprint turn-ids as DoD (S8 only); UserPromptSubmit DoD (S1); include subagents by default (default skip instead).
