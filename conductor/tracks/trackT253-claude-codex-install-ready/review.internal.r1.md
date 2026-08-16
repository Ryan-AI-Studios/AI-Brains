# Track Completion Audit — T253 (internal completeness)

Audit date: 2026-08-15. Scope: working tree + HEAD vs `spec.md` F0–F34 / AC1–AC20. No production edits. Tests and live dogfood were not re-executed.

Sources: Agent A commit `0587cb9` (writers, `install_ready` flip, exhaustive match, wrappers, doctor AC13, probes). Agent B working-tree slice (hook/import CLIs, adapters, schemas, product docs, help_ia). Coordinator flip of `harness_wiring_activation.rs` so `all-ready` includes five plans.

## Verdict: FAIL

Code for AC1–AC19 and F1–F32 is present and wired. The track is **not complete**: AC20 live dogfood is unchecked and unrecorded, and F30’s skill one-liner still tells Claude agents that Claude/Codex are pending.

## Requirement and DoD Matrix (one row per AC + each F that is independently verifiable)

| ID | Status | Evidence |
|----|--------|----------|
| **AC1** | Met | `detect.rs` `install_ready()` true for Claude/Codex; `pending_track()` `None`; tests 296–308 assert both plus `assert_ne!(…, Some("T253"/"T239+"))`. |
| **AC2** | Met | `install_claude__dry_run__zero_writes` / `install_codex__dry_run__zero_writes` — `DryRun`, walk_files unchanged, no `config.toml`. |
| **AC3** | Met | `install_claude__real__merges_foreign_exec_form_idempotent` — `theme` + `PreToolUse` kept; UPS+Stop+SessionEnd named `ai-brains-capture`; `command`+`args` exec-form; second run one UPS group; `probe_wiring` Ok. |
| **AC4** | Met | `install_codex__real__hooks_ups_stop_no_config_toml` + `install_codex__hooks_false_config__writes_hooks_leaves_toml` — UPS+Stop named; no SessionEnd; no `config.toml` create/edit; probe Ok. |
| **AC5** | Met | `uninstall_claude__removes_managed_keeps_foreign` / `uninstall_codex__removes_managed_keeps_foreign` — wrapper gone; foreign groups stay; prefs `installed_at` cleared; toml bytes unchanged. |
| **AC6** | Met | `claude_and_codex_wrapper__capture_then_emit_contract` — both `2>&1`, no `Write-Host`, no `decision`, no `render_hook_output`/`wrapper_command`; Claude empty const; Codex `codex_wrapper_continue_stdout()` = `{"continue":true}`. |
| **AC7** | Met | `claude_filter__*` / `codex_filter__*` keep user+assistant, drop tool/thinking/system/sidechain/`event_msg`; `append_*_turns` hardcodes `thinking: None`. |
| **AC8** | Met | `claude_map__ups_prompt_and_stop_last_message` + `claude_map__grok_shaped_stdin__none`; wrapper skips missing `hook_event_name`. |
| **AC9** | Met | `codex_map__ups_prompt_and_stop_last_message` + `codex_map__missing_fields__none`. |
| **AC10** | Met | `crates/ai-brains-adapters/tests/claude_import_t253.rs` hermetic project JSONL — user/assistant kept; `isSidechain` + `subagents/` skipped. |
| **AC11** | Met | `codex_import_t253.rs` rollout with `session_meta` + `event_msg` + `response_item` — only message roles; malformed counted fail-open. |
| **AC12** | Met | `resolve_harness_list__all_ready__returns_grok_agy_opencode` (five ids) + `harness_wiring_activation__all_ready_dry_run__zero_writes` (stdout names five + `/hooks`). |
| **AC13** | Met | `doctor_harness_wiring_message__all_five_ready_ok__no_t253_pending` — `5/5 ready wired`; no `T253` / `backend pending`. |
| **AC14** | Met | `StatusReport::SCHEMA_VERSION = 1`; `all_ready_yes` asserts `schema_version==1` and `install_ready=true` for all five. |
| **AC15** | Met | `help_ia.rs` Harness inventory + `root_after_long_help__harness_inventory_includes_harness_cmd`. |
| **AC16** | Met | CAPABILITIES table + §8; OPERATIONS path/activation; WORKFLOWS Activate (five + `/hooks`); CHANGELOG Unreleased T253; four research docs bannered. |
| **AC17** | Met | Parse/filter units live in `ai-brains-adapters` (no `models`/`graph` deps). Detect/install remain vault-free. |
| **AC18** | Met | Corrupt settings/hooks + JSONC `//` refuse; bytes unchanged; wrapper not written. |
| **AC19** | Met | Plans/targets/`all-ready --yes` writes asserted under temp home; no repo `.claude/settings.json` / `.codex/hooks.json`. |
| **AC20** | Unmet | Plan Phase 5 still unchecked. No recorded dry-run/`--yes`/`harness status`/`preflight` /hooks note. Cannot treat live dogfood as done. |
| **F0** | N/A | Plan-only until go; go recorded 2026-08-15. |
| **F1** | Met | Claude/Codex `install_ready=true` with real writers (not a pending stamp). |
| **F2** | Met | Flip + `pending_track()=None` + summary footer drop `T239+` + doctor AC13. |
| **F3** | Met | `HARNESS_ORDER` grok → agy → opencode → claude → codex; `all-ready` = that list. |
| **F4** | Met | `harness.rs` install/uninstall exhaustive five-variant match; no `_ => unreachable!`. |
| **F5** | Met | Map-only `settings.json` merge; UPS+Stop+SessionEnd; exec-form `args`; PATH bake; JSONC refuse. |
| **F6** | Met | `hooks.json` only; UPS+Stop; warn if `[features].hooks=false`; never writes `codex_hooks` / `config.toml`. |
| **F7** | Met | User-global paths under resolved home; hermetic tests; consent `--yes`/TTY unchanged. |
| **F8** | Met | Claude wrapper captures child `2>&1`, host stdout empty, `exit 0`. |
| **F9** | Met | Codex wrapper captures then `[Console]::Out.Write('{"continue":true}')`; no `render_hook_output` / `wrapper_command`. |
| **F10** | Met | Live map uses `prompt` / `last_assistant_message` + `filter_turn`; no `transcript_path` parse on live path. |
| **F11** | Met | Same for Codex; empty role skipped. |
| **F12** | Met | `codex-import` walks `sessions/**/rollout-*.jsonl`; keep-list + fail-open. |
| **F13** | Met | `claude-import` walks `projects/<encoded-cwd>/*.jsonl`; decode + unbound; skip sidechain. |
| **F14** | Met | clap `claude-hook`/`codex-hook` `--payload`/`--schema`; import `--days`/`--force`/`--dry-run`; `include_str!` 2020-12 schemas; `deny_unknown_fields` on payload structs. |
| **F15** | Met | Live `v5(session,"{event}:{id}")`; batch uuid/`turn_id` else `turn-{i}`; `thinking: None`; hook skips existing role+content. |
| **F16** | Met | cwd → normalize → alias; env project only when unbound; `allow_default_project: false`; `source_meta:claude|codex:…`. |
| **F17** | Met | `run_multi_harness_import` still agy → grok → opencode only; no skip-import-claude/codex. |
| **F18** | Met | SessionStart / PreCompact / StopFailure / Subagent not in managed event lists. |
| **F19** | Met | New wrappers under `~/.ai-brains/hooks/`; `scripts/target-*-hook.ps1` not installed; research banners. |
| **F20** | Met | Probe named **or** path token **or** legacy substring; `targets_for` lists settings/hooks.json + wrapper. |
| **F21** | Met | Ready next = `--dry-run` without `# backend pending`; status footer lists Claude/Codex ready lines; AC13. |
| **F22** | Met | Dry-run/install print `next: in Codex run /hooks and trust ai-brains-capture`. |
| **F23** | Met | Wrapper + `map_claude_hook_payload` skip Grok camelCase-only / missing Claude fields; exit 0. |
| **F24** | Met | Detect/install/map have no model/graph; hook CLIs open vault like `grok-hook`. |
| **F25** | Met | `CapabilityLevel::Full` kept; notes rewritten; `parse_claude_stop_payload` routes `filter_turn`; Codex parsers added. |
| **F26** | Met | Uninstall removes only `ai-brains-capture` handlers + wrapper. |
| **F27** | Met | `atomic_write_str` + T190 reparse refuse reused. |
| **F28** | Met | Wrappers do not embed keys; no `AI_BRAINS_KEY` print in new paths. |
| **F29** | Met | Workspace clap `4.5`, version `0.1.1`; adapters add no new crates. |
| **F30** | Partial | Product docs + CHANGELOG + research banners done. `.claude/skills/ai-brains/SKILL.md` still says Claude/Codex pending. |
| **F31** | Met (code) | No fake-ready-without-writers, no SessionStart injection, no Codex `decision:block`, no nightly schema/clap 5/`unreachable!` in dispatch. |
| **F32** | Met | No `parse_ingest_request` / T252 ingest rewrite; T254/T255 untouched. |
| **F33/F34** | Met (excluded) | Soft residuals remain out of DoD (nightly sources, SessionEnd ingest, Unix wrappers, unified PS1). |

## Findings in P0-P3 format:

### [P1] AC20 live dogfood is not recorded

Confidence: High

Requirement: AC20 / plan Phase 5 / spec §10 “Manual on go: AC20”

Location: `C:\dev\AI-Brains\conductor\tracks\trackT253-claude-codex-install-ready\plan.md` Phase 5 (all boxes unchecked); no AC20 note in the track dir

Problem: DoD requires dry-run then `--yes` for claude and codex (or `all-ready`), `harness status` both `ok`, preflight with no pending next, zero new files under `C:\dev\AI-Brains`, and a recorded `/hooks` trust next-action. None of that is checked or written down.

Evidence: Plan lines 140–146 remain `- [ ]`. Track directory contains only `spec.md` and `plan.md` (plus this review). Hermetic tests prove writers under temp home, not live user-global install.

Failure scenario: Track marked complete while this machine still has `wiring=missing` / no user-global hooks, or a live install silently wrote repo-local files that nobody checked.

Correction: On go, run the Phase 5 commands; confirm `config.toml` untouched and no new repo files; paste `harness status` + preflight harness lines and the `/hooks` trust note into the plan.

Verification: Plan Phase 5 checked with recorded stdout excerpts; live `~\.claude\settings.json` + `~\.codex\hooks.json` + wrappers present; repo `git status` shows no `.claude/settings.json` / `.codex/hooks.json`.

Deferrable: No

### [P2] Claude project skill still claims Claude/Codex are pending

Confidence: High

Requirement: F30 skill one-liner; honesty after F1/F2 (`install_ready` true)

Location: `C:\dev\AI-Brains\.claude\skills\ai-brains\SKILL.md` line 89

Problem: Command summary still says `Ready grok/agy/opencode only; Claude/Codex pending (T253)` after writers flipped ready.

Evidence:

```89:89:C:\dev\AI-Brains\.claude\skills\ai-brains\SKILL.md
| Harness capture | `harness install --harness all-ready --dry-run` then `--yes` | Ready grok/agy/opencode only; Claude/Codex pending (T253) |
```

Contrast: `.agents/skills/ai-brains/SKILL.md` line 145 already has the T253 ready one-liner. CAPABILITIES/OPERATIONS/WORKFLOWS match ready-five.

Failure scenario: A Claude Code session following the project skill will not install Claude/Codex and will treat T253 as still fenced.

Correction: Change the Notes cell to five-ready language (same as WORKFLOWS: `all-ready` is grok → agy → opencode → claude → codex; Codex still needs `/hooks` trust; no nightly).

Verification: Grep `.claude/skills` and `.agents/skills` for `pending (T253)` / `T239+` returns none except historical CHANGELOG.

Deferrable: No

## Completeness Sweep

Placeholders / stubs: `install_pending` remains as a dead helper (Claude is ready, so `run_install` never calls it). Not a fake-ready path. Doctor copy still contains a `backend pending (T253)` string for hypothetical `install_ready=false` rows; after F2 that bucket is empty.

Fake ready: No. Flip landed with `install_claude` / `install_codex`, wrappers, probes, and hook CLIs.

SessionStart injection: Not wired. Managed Claude events are UPS+Stop+SessionEnd only. Capability notes and banners say no injection.

Nightly source growth: `run_multi_harness_import` still three sources. CAPABILITIES §8 unchanged order.

`config.toml` rewrite: Install/uninstall tests assert bytes unchanged / file not created.

Reachable `unreachable!`: None in `commands/harness.rs` install/uninstall matches.

`render_hook_output` as Codex Stop stdout: Not called. Wrapper emits `{"continue":true}` only after `2>&1` capture.

Isolation: No T252 ingest rewrite; clap remains 4.5; workspace 0.1.1; no pin bumps; no new crates; no repo-local hook JSON; nightly `last_multi_import` schema untouched.

Uncommitted slice: Agent B hook/import/docs/adapters are in the working tree and were audited; they are not sufficient for completion until committed after AC20 + skill fix.

Process: `conductor.md` T253 still In Progress. Plan Phases 1–6 checkboxes still open (implementation exists anyway). `deferred.md` absorb line not closed. Full gate / `ledgerful verify` / cross-model review not in this audit.

## Wiring and Regression Review

Install → files: `install_claude` merges `~/.claude/settings.json` hooks (UPS+Stop+SessionEnd, name `ai-brains-capture`, exec-form) + `~/.ai-brains/hooks/claude-capture.ps1`. `install_codex` merges `~/.codex/hooks.json` (UPS+Stop only) + `codex-capture.ps1`. PATH bake via `ps_resolve_ai_brains(resolve_cli_exe_for_wrapper())`.

Probe: After hermetic install, `probe_claude` / `probe_codex` return Ok (`wiring.rs` tests). `targets_for` lists JSON + wrapper.

Hook CLI: Wrappers invoke `claude-hook` / `codex-hook --payload`. Commands map via `filter_turn` then `CaptureService::ingest_request` (`thinking: None`). Unrecognized / empty roles exit 0. Mid-payload garbage is `?` → `handle_cli_result` exit 1 JSON. Wrapper still fail-open (catch + F8/F9 stdout).

Import: `claude-import` / `codex-import` walk vendor trees, filter, bind path/unbound, 300s quiescence unless `--force`, dry-run lists without writes.

Tests vs old pending backends: AC1/AC12/AC13/AC3–AC6/AC14/AC15 would fail if Claude/Codex were still `install_ready=false` or `install_pending`-only. Import/filter tests would fail if adapters were still capability-only / naive NeutralEvent.

Docs: Product docs match ready-five + `/hooks` + no nightly. Stale skill is the remaining honesty hole. Historical CHANGELOG T238/T239 lines still say T239+ (immutable history; fine).

## Completion Decision

**Do not mark T253 Completed.**

Required before clearance:

1. Fix `.claude/skills/ai-brains/SKILL.md` (P2).
2. Execute and record AC20 (P1).
3. Commit the Agent B + skill + any AC20 plan note as the implement slice (this review does not commit).
4. Then: fmt/clippy/nextest/deny/audit, `ledgerful verify`, cross-model `codex-review` (FEATURE), absorb deferred line, ledger commit.

No P0. No invented findings. No P3 proposed for deferral (the only extras are process checkboxes, not product defects).
