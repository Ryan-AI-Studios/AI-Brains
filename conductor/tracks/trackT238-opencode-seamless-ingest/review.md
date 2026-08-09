# T238 Internal Review

**Reviewer:** Grok Build (read-only internal)  
**Date:** 2026-08-09  
**Scope:** Spec AC1–AC23 + hard F decisions vs code/tests/docs  
**Method:** Spec/plan read; `message_only.rs`, `opencode.rs`, `opencode_hook.rs`, `opencode_import.rs`, harness install/detect/wiring, fixtures, adapter/CLI tests, CAPABILITIES/OPERATIONS/CHANGELOG/Opencode-Hooks-Research; placeholder + wiring greps. No code changes; no cargo fix.

## Final verdict (orchestrator): **CLEAN** — Codex final **PASS**

| Round | Result |
|-------|--------|
| Internal R1 | FAIL (P1 F12 fallback, P2 help/honesty) |
| Internal R2 | CLEAN after fix |
| Codex R1 | FAIL (temp cleanup, child fail-closed, marker header, timeout kill, list_capped, probe, …) |
| Codex R2 | FAIL (probe_opencode still non-header) |
| Codex final | **PASS** (`review.codex.final.md`) — no open P0–P2 |

**Local gates:** `cargo fmt --check` clean; workspace clippy `-D warnings` clean; nextest **2373 passed** (1 skipped); `cargo deny check` ok; `cargo audit` warnings only (pre-existing allowed). Full PR CI still required before squash-merge.

## Verdict: CLEAN (post-fix 2026-08-09)

Core message-only filter, install_ready, child skip, synthetic drop, watermark batch, and never-`opencode.db` are substantially implemented with non-vacuous tests.

### Fix pass (after internal R1 FAIL)

| Finding | Disposition |
|---------|-------------|
| **P1** F12 live CLI export fallback | **verified_fixed** — plugin `exportViaCli` (`opencode export` 120s) when SDK messages fail; hook `export_session_via_cli` when no path; install test asserts exportViaCli + 120000 |
| **P2** capability honesty | **verified_fixed** — claim now matches implementation (fallback wired) |
| **P2** help_ia inventory | **verified_fixed** — `opencode-hook` / `opencode-import` in ROOT_AFTER_LONG_HELP + test |
| **P3** targets_for OPENCODE_CONFIG_DIR | **verified_fixed** — uses `opencode_config_dir` |
| **P3** harness status OpenCode tip | **verified_fixed** |
| Remaining P3s (CLI hook vault test, AC16 timeout sim, list_capped vs vendor 100, parent get fail-open) | soft / optional |

Targeted: adapters message_only + opencode_import 17/17; CLI install_opencode/help/wiring/install_ready 10/10; clippy adapters+cli clean.

## Requirement matrix (AC1–AC23): Met/Partial/Unmet + evidence

| AC | Status | Evidence |
|----|--------|----------|
| **AC1** nested export → non-synthetic text only | **Met** | `filter_opencode_export` + denylist; unit `filter_opencode__nested_export__only_non_synthetic_text`; fixture test `opencode_export_live__nested_part_union__only_text` (`tests/fixtures/message_only/opencode_export_live.json`) |
| **AC2** flat `opencode_messages.jsonl` | **Met** | `filter_opencode_message_lines`; `opencode_fixture__only_user_assistant` |
| **AC3** hermetic hook path, thinking None, `msg_*` ids | **Partial** | SOOT: `append_opencode_turns` always `thinking: None`; `generate_opencode_turn_id` + `filter_opencode_message_with_id`; test `append_opencode_turns__thinking_none_msg_id_stable`. Hook CLI `commands/opencode_hook.rs` loads `messagesPath`/`exportPath`. **No** hermetic CLI-level `opencode-hook --payload` vault test (library path only) |
| **AC4** live==batch turn ids | **Met** | Shared `generate_opencode_turn_id` + `append_opencode_turns` used by hook + import; unit proves `msg_id` wins over index |
| **AC5** worktree/directory bind | **Met** | `resolve_opencode_project` prefer worktree→directory; tests `import_opencode__directory_bind__project_matches`, `import_opencode__worktree_prefer__over_directory` |
| **AC6** unbound + env anti-hijack | **Met** | `OPENCODE_UNBOUND_ALIAS`; batch `allow_default_project: false`; hook env only when unbound + alias not stamped; `import_opencode__unbound__not_env_default` |
| **AC7** watermark zero dupes | **Met** | Cursor `~/.ai-brains/opencode-import-cursor.json` + skip `updated <= cursor`; `import_opencode__watermark__second_run_zero_dupes` |
| **AC8** `--force` / dry-run | **Met** | Force clears watermark skip; dry_run no vault writes (`sessions`/`imported_turns` 0); `import_opencode__force_and_dry_run` |
| **AC9** install marker + ready + wiring | **Met** | `install_ready()→true` for Opencode; `install_opencode` marker + prefs installed; `wiring__opencode_after_real_install__ok`; no `backend_pending` after real install |
| **AC10** dry-run install zero writes | **Met** | `install_opencode__dry_run__zero_writes` |
| **AC11** uninstall managed only / foreign preserved | **Met** | `uninstall_opencode__removes_managed_keeps_foreign`; does not rewrite `opencode.json(c)` |
| **AC12** missing binary soft skip | **Met** | `force_missing_binary` / `ExportErr::Binary` → `skipped_missing_binary`, exit path soft; `import_opencode__missing_binary__soft_skip` |
| **AC13** path case normalize | **Met** | `normalize_opencode_project_hash` + unit case fold on Windows paths |
| **AC14** never `opencode.db` | **Met** | Design comments + list/export only; `import_opencode__never_references_opencode_db` asserts no `join("opencode.db")` |
| **AC15** docs honesty F34 + Claude/Codex not T238 | **Partial** | CAPABILITIES/OPERATIONS/CHANGELOG present; F37 banner on `Docs/Opencode-Hooks-Research.md`; Claude/Codex `T239+`. **Overclaim:** CAPABILITIES + `opencode_capability` notes say live “CLI export fallback, 120s” but plugin does not implement it (see P1/P2). Series README still “Implementing” (expected pre-close). `help_ia` omits opencode commands |
| **AC16** export timeout fail-open | **Partial** | `OPENCODE_EXPORT_TIMEOUT_SECS=120`, `timed_out` + continue; only constant presence test (`parse_export__source_has_no_db_open`), no hermetic timeout simulation |
| **AC17** capture independence | **Met** | Pure filter + CaptureService ingest; no models/embeddings on adapter path. Graph hook only under existing CLI `cfg(feature="graph")` parity with other harnesses |
| **AC18** managed marker + idempotent reinstall | **Met** | `// AI-Brains managed (T238)`; reinstall overwrite when marker present; foreign refuse |
| **AC19** part type `tool` dropped | **Met** | denylist + `filter_opencode__part_type_tool_with_stray_text__dropped` |
| **AC20** hermetic list+export inject | **Met** | `list_json_override` + `export_json_override_dir`; `import_opencode__hermetic_inject__no_network` |
| **AC21** child `parentID` skip | **Met** | List + export info parent skip; plugin early return; hook parentId skip; `import_opencode__child_session__skipped` asserts `skipped_child_session==1` and zero turns |
| **AC22** synthetic chrome matrix | **Met** | `is_synthetic_or_ignored_part` (synthetic/ignored/editor_context); unit + fixture `opencode_synthetic_chrome__zero_user_memories_real_kept` |
| **AC23** list_capped warn | **Met** | When `len >= list_cap`/`max_sessions` → stderr `list_capped` + stats; `import_opencode__list_capped__warns` |

## Findings

### [P1] Live F12 export fallback not implemented

**F12 (hard):** prefer `client.session.messages` → temp export → `opencode-hook`; **fallback** CLI `opencode export <sessionId>` (120s) if SDK fetch fails.

**Actual** (`harness/install.rs` `opencode_plugin_js_body`):
- On `client.session.messages` failure, sets `messagesPath = null` and still spawns `opencode-hook` with no path.
- Hook `load_turns` then returns empty document → zero turns (silent live no-op for that idle).
- No `opencode export` spawn in plugin or hook.

Batch path correctly uses CLI export only (F19). Live completeness then depends entirely on later `opencode-import` (batch backstop), which is weaker than F12.

**Fix direction:** In plugin `catch` after messages fail (or when list empty after hard error): run `opencode export <sessionID>` (or `$`/client equivalent) with 120s, write temp file, pass as `exportPath`/`messagesPath`. Optionally mirror fallback in hook when paths missing + sessionId present.

### [P2] Capability honesty overclaims live export fallback (AC15 / F34)

`Docs/CAPABILITIES.md` OpenCode row and `opencode_capability().notes` advertise **“CLI export fallback, 120s”** for live. OPERATIONS is more accurate (SDK messages → hook; batch independent). Until P1 is fixed, docs/capability notes must not claim live export fallback.

### [P2] `help_ia` harness inventory omits OpenCode commands

Plan Phase 3: CLI + `help_ia`.  
`crates/ai-brains-cli/src/help_ia.rs` Harness line lists `agy-hook`, `grok-hook`, `grok-import` but **not** `opencode-hook` / `opencode-import`. Commands are registered in `main.rs` and work, but root `--help` IA is incomplete vs Grok/AGY parity.

### [P3] No hermetic CLI `opencode-hook` integration test

AC3 behavior is covered via shared library (`parse_export_*` + `append_opencode_turns`) and schema file, not an end-to-end `opencode_hook::run` with temp vault + fixture path (parentId skip, bind, thinking). Recommend one hermetic test mirroring T237 style.

### [P3] AC16 timeout fail-open untested behaviorally

Timeout branch exists (`recv_timeout` → `timed_out` + continue) but only constant presence is asserted. Optional: inject timeout error enum in hermetic path without real process hang.

### [P3] `targets_for(Opencode)` ignores `OPENCODE_CONFIG_DIR`

Install + `probe_opencode` honor F40 via `opencode_config_dir`. Status `targets_for` hardcodes `home/.config/opencode/plugins/...`, so status/plan text can disagree with install path when env relocates config.

### [P3] `harness status` human text still only prints AGY/Grok “ready” tips

`commands/harness.rs` `run_status` omits an OpenCode ready line (install path works via `--harness opencode`).

### [P3] list_capped when `--max-sessions` > vendor hard cap 100

CLI sets `list_cap = max_sessions`. If user passes e.g. 500 and OpenCode still returns 100, `len >= list_cap` is false → no `list_capped` warn despite possible silent vendor cap. Default 100 path is fine; consider warn when `len >= OPENCODE_LIST_DEFAULT_CAP`.

### [P3] Plugin parentID `session.get` fail-open continues ingest

If `client.session.get` throws, plugin continues without parent check (fail-open). Could ingest child sessions on get failure. Soft risk; batch list is roots-oriented.

## Codex R1 dispositions

Cross-model findings from `review.codex.md` (2026-08-09). Fix pass applied; track **not** marked Completed (no ledger commit / Phase 6 closure).

| Finding | Validated? | Disposition |
|---------|------------|-------------|
| **P1** Live plugin leaves raw tool/reasoning exports in `%TEMP%` | Validated | **fixed** — plugin `unlinkQuiet` in try/finally after hook spawn (and outer finally); install + `opencode_plugin_js_body__seam_contract` assert `unlink`/`unlinkQuiet` |
| **P1** Child-session exclusion not guaranteed (`--roots` + get fail-open) | Partly valid | **fixed** (plugin fail-closed) + **false positive** on `--roots` (OpenCode CLI has no `--roots` flag; rely on `parentID` from list JSON + plugin fail-closed on `session.get` throw). `parse_session_list_json` already extracts `parentID`/`parentId` |
| **P1** F14 message-ID delta not implemented | Partly valid | **fixed** (honesty) — CAPABILITIES + capability notes disclose turn ids use `msg_*` for stability; delta is max turn_index + watermark (Grok-class residual). Optional additive `last_msg_id` in cursor JSON. **No** QueryStore rewrite |
| **P1** Phase 6 / completion verification missing | Validated (process) | **deferred / out of scope** for this fix pass — no ledger commit, no conductor Completed, no full workspace gate here |
| **P2** Managed-marker not header-scoped | Validated | **fixed** — `has_opencode_managed_marker_header` (first non-empty line); install/uninstall use it; body-only marker refuse test |
| **P2** `OPENCODE_CONFIG_DIR` detection incomplete | Validated | **fixed** — `first_existing_home_root` treats existing `OPENCODE_CONFIG_DIR` as Opencode home present; unit test |
| **P2** Rust export timeout leaks child | Validated | **fixed** — `run_opencode_command_blocking` spawn + poll + `child.kill()` on timeout; list uses same path; stdout/stderr drained on threads |
| **P2** List-cap honesty when max-sessions > 100 | Validated | **fixed** — `list_capped` when `len >= OPENCODE_LIST_DEFAULT_CAP` (100) even if user max higher; hermetic test |
| **P2** Seam behavior not behaviorally tested | Partly valid | **fixed** (minimum) — extended install unit + `opencode_plugin_js_body__seam_contract` for exportViaCli, unlink, fail-closed parent get, 120000 (no full JS runtime) |
| **P2** Corrupt cursor silent fallback | Validated | **fixed** — `cursor_corrupt` eprintln once + empty start; unit covers parse-fail path |
| **Bonus** `config_dir_override` dead field | Validated | **fixed** — wired: sets `OPENCODE_CONFIG_DIR` on list/export subprocesses |

## Completeness sweep

### message_only (F1–F7, AC1/2/19/22)

| Decision | Status |
|----------|--------|
| F1 `normalize_opencode_export_message` nested→flat | **Met** |
| F2 non-synthetic text only; bare user kept | **Met** |
| F3 full denylist + synthetic/ignored/editor_context | **Met** (`tool`…`file`, steps, compaction, etc.) |
| F4 `filter_opencode_export` fail-open per msg | **Met** |
| F5 `source_ts` from `info.time.created` ms → RFC3339 | **Met** (unit asserts `Z`/`T`) |
| F6 thinking never populated | **Met** on `IngestRequest` |
| F7 fixtures nested + synthetic + flat jsonl | **Met** |

No TODO/FIXME/`unimplemented!` in OpenCode filter path.

### opencode.rs (import, watermark, bind, capability)

| Decision | Status |
|----------|--------|
| Import watermark cursor atomic-ish | **Met** |
| Never open `opencode.db` | **Met** |
| List cap + `projectId`/`projectID` tolerant | **Met** |
| Child skip list + export info | **Met** |
| Worktree prefer bind (F20) | **Met** |
| Unbound anti-hijack (F21) | **Met** |
| Turn ids `msg_*` (F13) | **Met** |
| thinking None (F6) | **Met** |
| Capability Full + hooks notes (F34/F39) | **Met** structure; **notes overclaim fallback** (P2) |
| Stats F23 (incl. list_capped, skipped_child_session) | **Met** (optional skipped_synthetic omitted — OK) |
| F25 unsummarized | **Met by inheritance** — store-level unsummarized is harness-agnostic; no OpenCode-specific gap |

### opencode-hook / opencode-import CLI

| Item | Status |
|------|--------|
| `opencode-hook --payload` / `--schema` | **Met** (`main.rs` + schema include) |
| Payload schema `Docs/schemas/opencode-hook-payload.json` | **Met** |
| parentId skip + worktree/directory bind | **Met** |
| `opencode-import --days/--force/--dry-run/--max-sessions` | **Met** |
| Soft missing binary | **Met** |

### install_opencode plugin (F8–F15, F27–F33, F40)

| Item | Status |
|------|--------|
| Managed `.js` marker T238 | **Met** |
| `session.idle` only | **Met** |
| parentID child skip | **Met** (plugin + Rust) |
| in-flight Map guard | **Met** |
| SDK `client.session.messages` | **Met** |
| CLI export fallback | **Unmet** (P1) |
| `install_ready` true / pending None | **Met** |
| OPENCODE_CONFIG_DIR install+probe | **Met** |
| No opencode.json rewrite; foreign refuse | **Met** |
| PATH `ai-brains` (no hardcoded profile) | **Met** |
| Claude/Codex T239+ labels | **Met** |

### Docs

| Item | Status |
|------|--------|
| CAPABILITIES OpenCode Implemented + caveats | **Partial** (fallback overclaim) |
| OPERATIONS install/hook/import/never db | **Met** (more accurate than CAPABILITIES) |
| CHANGELOG T238 entry | **Met** |
| F37 research banner | **Met** |
| Series README Completed | **Pending ship** (still Implementing — OK pre-PR close) |

### Placeholders / incorrect readiness

| Check | Result |
|-------|--------|
| TODO/FIXME/`unimplemented!` in opencode modules | **None found** |
| OpenCode `install_ready` false | **False** — correctly **true** |
| OpenCode `backend_pending` after real install | **Cleared** (prefs `mark_installed`) |
| Claude/Codex still pending T239+ | **Correct** |

## Wiring

| Check | Status |
|-------|--------|
| `harness` install match `HarnessId::Opencode => install_opencode` | **Yes** |
| uninstall match `uninstall_opencode` | **Yes** |
| `detect::install_ready` includes Opencode | **Yes** |
| `probe_opencode` / wiring after install | **Yes** |
| `main.rs` `OpencodeHook` / `OpencodeImport` | **Yes** |
| `commands/mod.rs` modules | **Yes** |
| `ai_brains_adapters` lib exports (`filter_opencode_*`, import, capability) | **Yes** |
| `adapter.rs` `AdapterKind::OpenCode => opencode_capability()` | **Yes** |
| help_ia OpenCode commands | **Missing** (P2) |

## Soft residuals only (S*)

Aligned with spec F soft list — not DoD failures:

| ID | Residual | Notes |
|----|----------|-------|
| **S1** | Min-interval debounce beyond in-flight | In-flight Map only |
| **S2** | Extra compaction_continue keying | Synthetic already dropped |
| **S3** | `message.updated` incremental | Not implemented |
| **S4** | Pure-export live if SDK shape drifts | Fallback itself is F12 hard (P1), not residual |
| **S5** | npm plugin publish | Not done |
| **S6** | Project-local plugin opt-in | Global only |
| **S7** | Import `--json` report | Human stderr stats only |
| **S8** | Claude/Codex install_ready | Correctly deferred T239+ |
| **S9** | Nightly multi-harness | T239 |
| **S10** | session.compacting archive | Non-goal |
| **S11** | Opt-in child ingest | Default skip hard |
| **S12** | Dual-subscribe session.status idle | Documented risk only |

## AGY / Grok regression notes

- AGY/Grok `install_ready` still true; OpenCode added without flipping them.
- `message_only` fixture tests still green for AGY/Grok matrices (same module, extended OpenCode helpers only).
- Claude/Codex pending labels correctly moved to **T239+** (not left as T238).
- No evidence of AGY wrapper stdout contract or Grok empty-Stop contract changes in this track surface.
- Shared harness status path now treats three ready backends; pending path unchanged for Claude/Codex.

## Recommended fix order before clearance

1. **P1** — implement live CLI export fallback in plugin (and/or hook) when SDK messages fail.  
2. **P2** — fix CAPABILITIES + capability notes to match reality (or keep claim only after P1).  
3. **P2** — add `opencode-hook` / `opencode-import` to `help_ia` Harness inventory (+ test assert).  
4. **P3** (optional) — hermetic hook CLI test; list_cap vs 100; targets_for OPENCODE_CONFIG_DIR; status ready tip.

## Summary

T238 is **feature-complete on the critical privacy/bind/install axes** (nested normalize, synthetic/tool denylist, child skip, watermark import, install_ready, docs banner). Verdict is **FAIL** until **F12 live export fallback** is implemented or the hard decision is explicitly waived, and capability/help honesty is aligned.

## Internal R2

**Reviewer:** Grok Build (read-only re-review after fix pass)  
**Date:** 2026-08-09  
**Scope:** Verify P1 F12 + P2 help_ia/honesty; spot-check P3 targets_for/status tip; no production edits.

### Verdict: **CLEAN** (no open P0–P2)

| Prior finding | R2 disposition | Evidence |
|---------------|----------------|----------|
| **P1** F12 live CLI export fallback | **verified_fixed** | Plugin `exportViaCli` spawns `opencode export` + 120000ms kill; on SDK messages fail sets `exportPath`; **`if (!messagesPath && !exportPath) return`** — never spawns `opencode-hook` with empty paths. Hook `load_turns` calls `export_session_via_cli` when no messagesPath/exportPath. Adapter `export_session_via_cli` (120s, never db). Install test asserts `exportViaCli` / export argv + `120000`. |
| **P2** help_ia inventory | **verified_fixed** | `help_ia.rs` Harness line includes `opencode-import, opencode-hook`; test asserts both. |
| **P2** capability honesty | **verified_fixed** | `Docs/CAPABILITIES.md` OpenCode row + `opencode_capability().notes` claim “CLI export fallback, 120s” / “export fallback, 120s” — matches plugin + hook + adapter. |
| **P3** targets_for OPENCODE_CONFIG_DIR | **verified_fixed** (spot) | `wiring::targets_for(Opencode)` uses `opencode_config_dir(home)`. |
| **P3** harness status OpenCode tip | **verified_fixed** (spot) | `run_status` prints `OpenCode ready: … --harness opencode --dry-run`; install notes for Opencode present. |

### F12 control-flow (no empty-path hook)

1. SDK `client.session.messages` → temp `messagesPath`; on throw → null.  
2. If no `messagesPath` → `exportViaCli(sessionID)` → temp `exportPath` or null.  
3. Both null → early `return` (no `ai-brains opencode-hook`).  
4. Hook path-miss → `export_session_via_cli` fail-open empty (no hard crash).

### Spot-check / non-regressions

- Child skip (`parentID` plugin + hook `parent_id`), marker, never-`opencode.db` design, install_ready Opencode still present in tree as before.  
- Soft residuals (hermetic hook CLI test, AC16 timeout sim, list_capped vs vendor 100, parent get fail-open) remain **P3 / optional** — not DoD blockers.  
- **No new P0–P2 findings.**

Orchestrator may run **codex-review** (cross-model) for clearance.
