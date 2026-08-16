# Track Completion Audit — T253 (internal completeness r2)

Audit date: 2026-08-15. Scope: working tree vs `spec.md` F0–F34 / AC1–AC20 after the claimed post-r1 fixes. Read-only. Tests and live dogfood were **not** re-executed; AC20 is judged from the Phase 5 record, not a fresh `harness status` run.

Sources: `spec.md`, `plan.md` Phase 5, `review.internal.r1.md`, `review.internal.correctness.r1.md`, and the current code/docs at the paths cited below.

## Verdict: PASS

Prior completeness P1 (AC20 unrecorded) and P2 (skill pending copy) are fixed. Prior correctness P1 (probe false-ok) and both P2s (query swallow, live `deny_unknown_fields`) plus the wrapper-id P3 are fixed in code with tests that would fail on the old behavior. Fresh sweep found no new P0–P2. Leftover P3s are the same dead doctor-arm / serialize-default nits plus a research-banner copy-paste.

Product DoD for this track is met. Conductor **Completed** still waits on Phase 6 (gate, `ledgerful verify`, cross-model, deferred absorb, ledger commit) — those are process, not product defects.

## Prior finding dispositions

### Completeness r1

| Finding | Disposition | Evidence |
|---------|-------------|----------|
| **P1 AC20 unrecorded** | **verified_fixed** | `plan.md` Phase 5 is checked (2026-08-15, `target\debug\ai-brains.exe`): dry-run then `--yes` for claude and codex; `harness status` both `present=yes wiring=ok install_ready=true`; `preflight --summary` all five `wiring=ok (ready)`, no pending next; `~\.codex\config.toml` SHA256 `630F5B5EC418B51CA3A451DFAE79CB2BEC6AA3C00D7C5F16B844582208FA53FE` unchanged; no repo-local `.claude/settings.json` / `.codex/hooks.json`; install printed `next: in Codex run /hooks and trust ai-brains-capture`. Live paths listed under user-global home only. Optional Stop fire skipped (F34). This r2 did not re-run those commands. |
| **P2 skill still pending** | **verified_fixed** | `.claude/skills/ai-brains/SKILL.md` line 89: “Five ready (grok → agy → opencode → claude → codex). Codex live fire needs `/hooks` trust. No nightly Claude/Codex.” Grep of `*SKILL.md` for `pending (T253)` / `T239+` is empty. `.agents/skills/ai-brains/SKILL.md` already had the T253 ready one-liner. |

### Correctness r1

| Finding | Disposition | Evidence |
|---------|-------------|----------|
| **P1 probe false-ok** | **verified_fixed** | `wiring.rs` `probe_claude` / `probe_codex`: parsed JSON is `Ok` only if `hooks_json_has_managed_name` **or** dedicated wrapper token (`claude-capture.ps1` / `codex-capture.ps1`). No generic `.ai-brains` / raw `ai-brains` substring. Claude parse-fail → `Missing` (not Ok). Codex parse-fail → `Unknown` (not Ok). New tests: `wiring__claude_settings_grok_ai_brains_path_only__missing` (Grok wrapper path + name `grok-capture` → Missing); `wiring__codex_hooks_generic_ai_brains_substring__missing` (`{"note":"see ai-brains docs"}` → Missing). Post-install Ok tests still pass. Matches F20 “managed name **or** wrapper path token.” |
| **P2 `get_session_turns` swallow** | **verified_fixed** | `claude_hook.rs` / `codex_hook.rs`: no `get_session_turns(...).unwrap_or_else(|_| Vec::new())`. Query health is `skip_on_query_error(get_max_turn_index(...))` → `Ok(())` no ingest on Err. Vendor-id skip uses `get_sync_state` the same way (not content-match). Helper unit tests lock skip-on-Err. Wrapper still fail-opens stop (F8/F9). |
| **P2 `deny_unknown_fields` unused on `--payload`** | **verified_fixed** | `main.rs` `--payload` calls `claude_hook::run` / `codex_hook::run` → `accept_*_live_payload`. That maps first (F23 skip), then `parse_*_hook_payload_strict`; `unknown field` → `Err` (CLI 1). Tests: `claude_live_payload__unknown_field_on_valid__err`, `codex_live_payload__unknown_field_on_valid__err`; Grok-shaped still `Ok(None)`. Schemas remain `additionalProperties: false`. |
| **P3 wrappers drop uuid / turnId** | **verified_fixed** (was P3) | Both wrappers copy `$ev.uuid` and `$ev.turn_id`/`$ev.turnId` onto the official payload when non-empty. AC6 unit asserts `$ev.uuid` + `turnId`. Live skip keys `live-turn:{session}:{turn_id}` only when a vendor id is present (F15). |
| **P3 doctor hardcodes `T253`** | **still open** | Dead arm after F2. See remaining P3. |
| **P3 uninstall `unwrap_or_default`** | **still open** | Not a panic. See remaining P3. |

## Remaining findings (P0-P3)

No P0. No P1. No P2.

### [P3] Doctor pending formatter still hardcodes `T253`

Confidence: High

Requirement: F2 / AC13 honesty of doctor copy (not live lie)

Location: `crates/ai-brains-cli/src/commands/doctor.rs` `doctor_harness_wiring_message` ~822–836; test `doctor_harness_wiring_message__separates_ready_from_pending` still constructs Claude/Codex with `install_ready=false` and asserts `2 backend pending (T253): claude, codex`.

Problem: After F2 every `HarnessId` has `pending_track() == None`, so `pending_present` is empty in production. A future sixth non-ready id would still be labeled `T253`. AC13 all-five-ready path is correct and is the live lock.

Deferrable: Yes (same leftover as correctness r1; not a live doctor lie).

### [P3] Uninstall serialize compare uses `unwrap_or_default`

Confidence: High

Location: `install.rs` `uninstall_official_hooks` ~1673–1677.

Problem: `serde_json::to_string(&root).unwrap_or_default()`. If serialize failed, `before == after` would skip the rewrite. Unlikely for `Map<String, Value>`. Not a panic.

Deferrable: Yes.

### [P3] Claude research banners mention Codex feature-key copy

Confidence: High

Requirement: F30 research-doc banners (honesty, not rewrite-as-spec)

Location: `Docs/claude-hooks.md` and `Docs/Claude-Hooks-Research.md` opening banners: “Hooks default-on via `features.hooks` (not `codex_hooks`)” on **Claude** docs.

Problem: Banner exists and points at T253 wrappers + `harness install --harness claude`. The `codex_hooks` clause is copy-paste from the Codex banners. Misleading, not a product-path lie.

Deferrable: Yes.

## AC matrix updates (especially AC20 / F20 / F30)

Unchanged from completeness r1 except the rows below. AC1–AC19 remain **Met** on code/tests cited in r1; this r2 re-checked the probe, hook, wrapper, schema, skill, and dispatch sites.

| ID | r1 | r2 | Evidence |
|----|----|----|----------|
| **AC20** | Unmet | **Met** | Phase 5 recorded 2026-08-15: dry-run + `--yes` both harnesses; status both `ok` + `install_ready=true`; preflight five `wiring=ok (ready)`; config.toml hash unchanged; zero repo-local hook JSON; `/hooks` next-action printed. Live fire optional (F34) skipped. |
| **F14** | Met (structs only) | **Met** (live path) | `accept_*_live_payload` is the `--payload` gate; unknown keys error; unrecognized / Grok skip `Ok(None)`. |
| **F15** | Met (generator only) | **Met** (wrapper + skip) | Wrappers forward uuid/turnId; live `v5(session,"{event}:{id}")`; skip via `get_sync_state` when vendor id present; `thinking: None`; no content-match on `{event}:stable`. |
| **F20** | Met (old substring probe) | **Met** (honest) | Parsed Ok only on managed `name` or dedicated wrapper token; tests lock Grok-path and generic-`ai-brains` false-ok. `targets_for` lists JSON + wrapper. |
| **F30** | Partial | **Met** | Product docs + CHANGELOG + four research banners + **both** skill files five-ready. Claude banners still mention `codex_hooks` (P3 only). |
| **F21** | Met | Met | Doctor AC13; ready next is `--dry-run` without `# backend pending`. `--install-hooks` still loops Agy/Grok/Opencode only (`preflight.rs` ~918–920) — spec isolation / F32; not a regression. Honesty remains `harness install` / `all-ready`. |
| **F22** | Met | Met | Install/dry-run print `/hooks` trust; AC20 recorded it. |
| **F23** | Met | Met | Wrapper skip on missing `hook_event_name`; `accept_claude_live_payload` Grok-shaped → None. |
| **F17** | Met | Met | `run_multi_harness_import` still agy → grok → opencode. |

### Completeness sweep (r2)

- Fake ready: no. Flip is writers + wrappers + probes + hook/import CLIs.
- SessionStart / additionalContext: not in managed event lists.
- Nightly source growth: none.
- `config.toml` rewrite: still read-only warn.
- Reachable `unreachable!` in harness install/uninstall: none (five-variant match).
- `render_hook_output` / `wrapper_command` as Codex Stop stdout: not used.
- Isolation: no T252 ingest rewrite; clap 4.5; workspace 0.1.1; no new crates; C7 user-global / hermetic temp home.
- Official Codex stdin includes `hook_event_name` (2026 docs); wrapper requirement is not a skip-all bug.
- Weak tests remain (hook `run()` not hermetic; AC6 is string-contains; `get_max_turn_index` is health-only). Not product defects.

## Completion Decision

**Product completeness: PASS.** Prior r1 completeness and correctness blockers are fixed. No remaining P0–P2.

**Do not flip `conductor.md` T253 to Completed in this review.** Phase 6 is still open: fmt/clippy/nextest/deny/audit, `ledgerful verify`, cross-model `codex-review` (FEATURE), absorb `deferred.md` “Claude/Codex install_ready”, then ledger commit. Those are the publish/closeout steps, not reopen-the-track defects.

No invented findings. Leftover P3s may defer to `conductor/ISSUES.md` at closeout.
