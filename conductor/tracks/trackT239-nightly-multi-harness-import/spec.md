# T239 — Nightly multi-harness import orchestration

- **Status:** ✅ **Completed** (PR #108 squash `a271a99`)
- **Source:** Series dual-path completeness (T234–T238 shipped); deferred S9 / SYSTEM skip-import honesty
- **Category:** FEATURE / OPS
- **Depends on:** T234 ✅; T236 AGY import ✅; T237 `grok-import` ✅; T238 `opencode-import` ✅
- **Related:** T79 `--skip-import`; T143/T145 SYSTEM wrapper; T135 nightly status; doctor/preflight harness summary (T235); T233 multi-root symbols (orthogonal)
- **Absorbs:** deferred multi-harness nightly (S9); SYSTEM `--skip-import` **product decision** (not silent re-enable); per-source skip flags; multi-import stats on `nightly --status`; docs honesty for CAPABILITIES pipeline step 1; **AI review 2026-08-09** (M1 hermetic seam; M2 path-in-error + per-source honesty; M3 per-source sinks; M4 OpenCode health counters; M5 corrupt status; M6–M10 honesty)
- **Does not absorb:** Claude/Codex `install_ready` (**T239+** / follow-up — labels stay pending); live hooks; synthesis batch limits; raw `opencode.db`; default child ingest; import `--json` on individual importers (soft); BrainLog harness-id analytics (soft); opt-in SYSTEM import (S-SYS)

## Objective

Extend `ai-brains nightly` so harness session import is **multi-source**, **message-only**, **skippable per source**, **fail-open**, and **observable**:

1. **AGY / Antigravity** batch (`import_antigravity_sessions`).
2. **Grok** `chat_history` batch (`import_grok_sessions`).
3. **OpenCode** list+export+watermark (`import_opencode_sessions`) — **never** `opencode.db`.
4. Persist + display a **last multi-import** summary on `nightly --status` (and optionally soft doctor/preflight lines).
5. Freeze honesty on **SYSTEM scheduled** import (default remains skip; user-context schedule is the completeness path).

## Research notes (2026-08-09)

| Topic | Finding | Pin for T239 |
|-------|---------|--------------|
| Live OpenCode | CLI **1.18.15**; list `-n` default 100, export, `--pure`, `OPENCODE_CONFIG_DIR` ([CLI docs](https://opencode.ai/docs/cli/)) | Reuse adapter; cap + 120s timeout fail-open |
| Grok / AGY homes | Live machine has `~/.grok`, `~/.gemini`, `grok` on PATH | User-scheduled nightly can read homes; SYSTEM cannot assume user profile |
| Task Scheduler identity | Task runs in the **principal’s security context**; SYSTEM ≠ interactive profile ([MS docs](https://learn.microsoft.com/en-us/windows/win32/taskschd/security-contexts-for-running-tasks)) | **Do not** re-enable import under SYSTEM by default |
| Dep pins | Workspace `clap` 4.5, `serde`/`serde_json` 1.0; crates.io clap 4.6.6 / serde 1.0.229 / serde_json 1.0.151 | **No intentional dep bumps** |
| Current code gap | `nightly.rs` only calls `antigravity_import::run` when `!skip_import` | Core body |
| Hermetic seams | Adapter options already have inject fields (`home_override`, OpenCode list/export/cursor overrides); CLI commands do **not** expose them | **Orchestrator `MultiImportOptions` threads overrides** (AI1 M1) |
| Stats | All three `*ImportStats` derive `Default` and return from adapters | Call adapters directly (D5); no wrapper-required path for tests |
| SYSTEM wrapper | Hardcodes `--no-project-context nightly --skip-import` | Keep; status will show `skipped (skip_import)` |

## Frozen decisions

| ID | Decision |
|----|----------|
| **D1** | Nightly phase order: **multi-harness import →** retention dry-run → summarization/embed/synthesis/MADR/symbols (existing tail unchanged). |
| **D2** | Sources (fixed order): **agy → grok → opencode**. Deterministic logs only (no inter-source dependency). |
| **D3** | `--skip-import` skips **all** harness importers. Help text must say multi-harness, not “Antigravity only”. |
| **D4** | Per-source flags: `--skip-import-agy`, `--skip-import-grok`, `--skip-import-opencode`. Global skip wins; per-source flags still accepted when global set. |
| **D5** | Orchestrator calls **adapter `import_*_sessions` directly** — not shelling out. **`StoreSink` is per-source** (AI1 M3): one sink per harness so `last_error` cannot contaminate the next source. Prefer small `make_sink(ctx)` helper. |
| **D6** | Typed `MultiImportReport` + `SourceImportReport` (`Serialize`/`Deserialize`, `#[serde(default)]` where needed). Per source: `status` (`ok` \| `error` \| `skipped`), skip reason, counters (`sessions`, `imported_turns`, `unbound`, source-specific), optional `error` string. OpenCode **must** include health counters: `list_capped`, `export_errors`, `timed_out`, `skipped_missing_binary` (AI1 M4). Aggregate order always agy → grok → opencode. |
| **D7** | **Fail-open is per-source** (not per-session by default): one harness `Err` → that source `status=error`, continue. Never abort nightly for import failure. Capture independence: no LLM for import phase. |
| **D8** | Persist `sync_state` key **`last_multi_import`** as compact JSON (`v:1`, stable field order). **`at` = `chrono::Utc::now().to_rfc3339()`** (same as `last_nightly_run`) (AI1 M6). Hermetic tests treat `at` as volatile (assert shape / ignore exact value). |
| **D9** | `nightly --status` prints a **Multi-import** block from `last_multi_import`, or **“Multi-import: never”** if missing. Missing key is non-fatal (AI2 blind spot 2). |
| **D10** | Defaults: `days = 30`; `force = false`; OpenCode `max_sessions` = adapter default (100) unless env `AI_BRAINS_NIGHTLY_OPENCODE_MAX` (optional). |
| **D11** | T234 message-only already enforced inside adapters — **no second filter** in nightly. |
| **D12** | **SYSTEM keeps `--skip-import` by default.** Completeness path: user-principal `nightly --schedule` or manual `nightly`. Do not claim SYSTEM imports harness history. |
| **D13** | Opt-in SYSTEM import out of DoD (**S-SYS**). |
| **D14** | `allow_default_project = false` on all nightly import paths. |
| **D15** | Missing binary / empty homes: soft counters, not hard error when adapters already soft-skip. **Honesty:** empty-home under SYSTEM-without-skip would look like “0 sessions found” — wrapper prevents by default; doc residual (**S-HOME**, AI1 M9). |
| **D16** | Claude/Codex remain **T239+** — nightly multi-import supports **only** AGY, Grok, OpenCode. Docs must say so (AI2 blind spot 4). |
| **D17** | No intentional dep bumps. Edition 2024; no `unwrap`/`expect` in production. |
| **D18** | Doctor/preflight last-import line optional soft (**S-DOC**). |
| **D19** | Re-summarize OR already T236 F17 — no synthesis expansion in T239. |
| **D20** | **Hermetic injection seam (AI1 M1 hard):** `MultiImportOptions` carries production-`None` override fields: `agy_home_override`, `grok_home_override`, `opencode_list_json_override`, `opencode_export_dir_override`, `opencode_cursor_path_override`, `opencode_config_dir_override` (and any needed to mirror adapter options). **AC2 uses malformed-fixture vector** (e.g. bad OpenCode list JSON → adapter `Err`) — zero env mutation, zero test-only production flags. |
| **D21** | **Partial import (AI1 M3):** after each source, capture **stats first**, then check `sink.last_error` / adapter `Err`. Report `status=error` with **actual** `sessions`/`imported_turns` already written (vault keeps partial success). |
| **D22** | **Error path capture (AI1 M2 minimum):** when a source fails, `error` string should include **failing path or session id** when available from the error chain (or adapter message). Document that one corrupt file may still abort the rest of **that** source under current adapters unless soft-skip lands. |
| **D23** | **Status corrupt tolerance (AI1 M5):** unreadable / invalid `last_multi_import` → warn + print **“last multi-import: unreadable”** (or equivalent); no panic, no serde stack on stdout. |
| **D24** | Import progress `eprintln!` may interleave non-JSON under SYSTEM `--log-format json` (AI1 M7). Accept + document; prefer `tracing` for new orchestrator logs; do not rewrite all adapter progress in this track. |
| **D25** | `list_capped` semantics: report the adapter counter as returned; soft residual to tighten false-positive when `max_sessions` > default cap (AI1 M10) — fix only if touching that code path cheaply. |

## Functional requirements

| ID | Requirement |
|----|-------------|
| **F1** | Module `commands/multi_import.rs`: `run_multi_harness_import(ctx, opts) -> MultiImportReport`. |
| **F2** | Nightly: if `!skip_import`, run multi-import; else all sources `skipped` reason `skip_import`. Always persist report after phase. |
| **F3** | Honor per-source skip flags when global skip is false. |
| **F4–F6** | AGY / Grok / OpenCode import via adapters with opts (days/force/max_sessions/overrides). |
| **F7** | Per-source sink (`make_sink`); fail-open; `tracing::error!`; partial counters + `status=error` (D21). |
| **F8** | Persist `last_multi_import` JSON (`v:1`, `at` RFC3339). |
| **F9** | `nightly --status` Multi-import block: timestamp, per-source status/turns/sessions/errors; OpenCode health counters; surface **“OpenCode import capped — may be incomplete”** (or short equivalent) when `list_capped > 0`. |
| **F10** | Expand **all** “Antigravity-only” copy (AI1 M8 touch-list): clap help `main.rs`; `nightly.rs` skip `tracing::info!`; OPERATIONS SYSTEM wrapper prose + nightly import section; CAPABILITIES §8 pipeline; WORKFLOWS; antigravity-rule. |
| **F11** | Clap per-source skip flags; plumb `main.rs` → `nightly::run`. |
| **F12** | SYSTEM wrapper tests still assert `--skip-import`. |
| **F13** | CAPABILITIES: multi-harness import step; Claude/Codex **not** in nightly batch (T239+). |
| **F14** | OPERATIONS: flags, D12 SYSTEM, user-schedule completeness; stderr/JSON-log note (D24). |
| **F15** | WORKFLOWS + antigravity-rule multi-harness language. |
| **F16** | Series README + conductor closeout. |
| **F17** | help_ia if long-help samples need flags. |
| **F18** | Hermetic tests use **D20 overrides only** (no live homes, no bare env set_var). |
| **F19** | Smoke: skip flags accepted; status missing key → “never”; status corrupt value → “unreadable”. |
| **F20** | Ship notes in CAPABILITIES/OPERATIONS. |
| **F21** | `MultiImportOptions` override fields wired into adapter options (D20). |
| **F22** | Error strings include path/session when available (D22). Prefer: if AGY/Grok loop can soft-skip one corrupt file with a counter in ≤small patch, do it; else document per-source fail-open + soft **S-SESSION**. |
| **F23** | Status parse: missing → never; invalid JSON → unreadable (D23). |
| **F24** | Typed report structs with stable serde (AI2 #3). |

## Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | Hermetic multi-import via **override inject** (D20): AGY+Grok+OpenCode fixtures → three `ok` (or soft-skip with counters); turns when non-empty. |
| **AC2** | **Malformed-fixture** error on one source (e.g. bad OpenCode list JSON) → other sources still run; failed source `status=error` only; no env-based inject. |
| **AC3** | `--skip-import` → no importer vault writes; all sources `skipped`. |
| **AC4** | `--skip-import-grok` alone → AGY+OpenCode run; Grok skipped (plus at least one other per-source case). |
| **AC5** | Message-only regression suite green (tool-heavy fixtures). |
| **AC6** | After a run, `nightly --status` multi-import block matches stored report; missing key → “never”. |
| **AC7** | SYSTEM wrapper still embeds `--skip-import`. |
| **AC8** | CAPABILITIES + OPERATIONS: multi-import + SYSTEM + Claude/Codex not in batch. |
| **AC9** | Full CI gate green; no dep bumps. |
| **AC10** | Claude/Codex still pending (not claimed shipped). |
| **AC11** | Corrupt `last_multi_import` value → status “unreadable”, exit 0, no panic (AI1 M5). |
| **AC12** | OpenCode source report includes `list_capped` / `export_errors` / `timed_out` / `skipped_missing_binary` when available; status surfaces cap warning when `list_capped > 0` (AI1 M4). |
| **AC13** | Per-source sink isolation: forced `last_error` / partial import on source A does not flip source B to error without B failing (AI1 M3). |

## Non-goals

- Claude / Codex / Cursor install backends.
- Re-enabling SYSTEM import by default (D12).
- Live hook install or plugin changes.
- Changing NightlyService summarization, batch limits, or graph.
- JSON status for individual `*-import` CLIs (soft S-JSON).
- Full rewrite of adapter `eprintln!` progress to tracing (D24 accept).
- T233 multi-root; npm plugin; OpenCode SQLite.

## Soft residuals

| ID | Item | Disposition |
|----|------|-------------|
| **S-SYS** | Opt-in SYSTEM import + baked USERPROFILE | Soft — not DoD |
| **S-JSON** | Per-importer `--json` | Soft (T238 S7) |
| **S-DOC** | Doctor/preflight last-import one-liner | Soft |
| **S-BRAINLOG** | BrainLog harness id vs live agy | Soft |
| **S-BUDGET** | Wall-clock hard cap beyond list/timeout | Soft |
| **S-CLAUDE** | Claude/Codex install_ready | **T239+** |
| **S-FORCE** | Shared `--import-force` on nightly | Soft |
| **S-SESSION** | Full per-session soft-skip counters for AGY/Grok (beyond path-in-error) | Soft if not cheap in F22 |
| **S-HOME** | Explicit “no user profile” counter under SYSTEM empty home | Soft (AI1 M9); wrapper default prevents |
| **S-CAP** | Tighten `list_capped` when `max_sessions` > default | Soft (AI1 M10) |

## Risks

| Risk | Mitigation |
|------|------------|
| OpenCode export runtime | List cap + 120s timeout + watermark; health counters visible |
| Nightly duration 3× | days=30; per-source skip; watermarks |
| SYSTEM expects import | D12 + status `skipped (skip_import)` |
| Cross-vault contamination | `--skip-import`; `allow_default_project=false` |
| Schema drift / corrupt state | `v:1` + AC11 unreadable |
| Hermetic tests flaky on env | D20 overrides only |
| One corrupt file kills source | D22 path capture; S-SESSION if full soft-skip deferred |

## Files likely touched

| Area | Path |
|------|------|
| Orchestrator | `commands/multi_import.rs` (new), `nightly.rs`, `mod.rs` |
| CLI | `main.rs` (flags + help), maybe `help_ia.rs` |
| Adapters | only if F22 soft-skip path capture needs small loop change |
| Tests | multi_import hermetic; nightly status corrupt/missing; SYSTEM wrapper; smoke flags |
| Docs | CAPABILITIES, OPERATIONS (incl. ~512 + SYSTEM ~535), WORKFLOWS, antigravity-rule, series README |

## Definition of Done

- AC1–AC13 met; plan checklist complete.
- Internal review clean of critical/high; mediums fixed or justified ≤3.
- Full gate green.
- Manual: multi-import status block, skip flags, SYSTEM dry-run still skip-import, corrupt status path.
- Pins: multi-source order; SYSTEM default skip; `last_multi_import`; hermetic overrides; per-source sinks.
- Conductor Completed; S9 closed; Claude remains T239+.

## Stop-before

- Code / ledger TX until **go**.
- Force-push / push main without approval.
- Re-enable SYSTEM import by default.
- Claiming Claude/Codex install shipped.
- Opening `opencode.db` as import SOOT.
- Dep bumps “while we’re here”.
- Bare `std::env::set_var` in tests (use overrides / TempEnv only if unavoidable).

## AI review fold-in (2026-08-09)

| Item | Verdict | Disposition |
|------|---------|-------------|
| **AI1 M1** hermetic `MultiImportOptions` + malformed AC2 | **Agree hard** | **D20**, F18/F21, AC1/AC2 rewrite |
| **AI1 M2** per-session vs path-in-error | **Agree minimum** | **D22**/F22 path-in-error hard; full soft-skip **S-SESSION** if not cheap |
| **AI1 M3** per-source sink + partial counters | **Agree hard** | **D5 rewrite**, **D21**, F7, **AC13** |
| **AI1 M4** OpenCode health counters + status | **Agree hard** | **D6**, F9, **AC12** |
| **AI1 M5** corrupt JSON status | **Agree hard** | **D23**, F19/F23, **AC11** |
| **AI1 M6** `at` RFC3339 | **Agree** | **D8** |
| **AI1 M7** non-JSON stderr ×3 | **Agree document** | **D24**, F14 |
| **AI1 M8** Antigravity-only touch-list | **Agree** | **F10** expanded list |
| **AI1 M9** SYSTEM empty-home honesty | **Agree soft** | **D15**, **S-HOME** |
| **AI1 M10** list_capped false-positive | **Agree soft** | **D25**, **S-CAP** |
| **AI1 confirms** D12, D5 adapters, sync_state, status cheap, order, deps | Affirmed | Keep |
| **AI2** architecture summary / AC table | Affirmed | Matches; AC extended |
| **AI2** OpenCode budget | Affirmed | Already D10 + T238 |
| **AI2** status never | Affirmed | D9 |
| **AI2** typed report serde | **Agree** | **F24** |
| **AI2** Claude docs honesty | **Agree** | F13/AC8/D16 |
