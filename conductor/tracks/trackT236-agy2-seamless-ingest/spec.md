# T236 — Antigravity 2 CLI seamless ingest

- **Status:** 🚧 **In Progress** (implementation; ledger TX open)
- **Source:** Research 2026-08-08 + live dogfood 2026-08-08; T234/T235 handoffs; official [AGY Hooks docs](https://antigravity.google/docs/hooks); **AI review fold-in 2026-08-08** (§14)
- **Category:** FEATURE
- **Depends on:** T234 (message-only SOOT ✅); T235 (detect/install/wrapper/F34 map ✅)
- **Absorbs:** history.jsonl workspace→project binding; live step-parser wire for `agy-hook` (T234 F13 residual); stale `Docs/antigravity-rule.md` “no hooks”; batch `project_hash: None` UX bug; import stats honesty; quiescence `--force`; soft fullyIdle policy polish; re-summarize re-queue; **AI2 M1–M6 / L1–L5** (wrapper stdout SOOT, unified turn-id, hook normalize/unbound, transcript_full prefer, source_meta path key, dead code, WORKFLOWS honesty, stable unbound name, history tie-break, counter defs)
- **Related:** series [README](../README-T234-T239-HARNESS-INGEST.md); existing `agy-hook`, `antigravity-import`, nightly import
- **Does not absorb:** multi-harness nightly orchestration (T239); Grok/OpenCode (T237/T238); shared message-only module body (T234); preflight JSON harness array (T220); display ASSISTANT: strip (T224); project-scope hook install as default (C7); **re-enable SYSTEM scheduled AGY import** (wrapper today uses `--skip-import` — honesty only; T239)

## Objective

Make **Antigravity 2 CLI (`agy`)** capture **seamless and findable in the right project**:

1. **Live:** Stop (fullyIdle) → T235 wrapper → `agy-hook` correctly parses **step-shaped** `transcript.jsonl` + message-only SOOT + workspace→project binding.
2. **Batch:** `antigravity-import` / nightly brain scan binds each `conversationId` via **`history.jsonl`** workspace map so scoped preflight/recall see AGY work (not only `--global` / wrong default project).
3. **Honesty:** rewrite antigravity docs; import stats include unbound/quiescent counts; capability notes match reality.
4. **Ops polish:** `--force` bypass quiescence; re-queue summarized sessions that receive new turns (query or compensating path — see F17).

## Diagnosis (frozen — 2026-08-08 + code verify)

| ID | Fact |
|----|------|
| **D1** | SOOT content: `~/.gemini/antigravity-cli/brain/<id>/.system_generated/logs/transcript.jsonl` (+ legacy `antigravity` / `antigravity-ide` brains; optional `overview.txt`). |
| **D2** | Live **transcript.jsonl is step-shaped** (`step_index`, `source`, `type`, optional `thinking`/`tool_calls`/`content`) — same as overview — **not** `{role,content}`. Verified on live machine 2026-08-08. |
| **D3** | **`agy-hook` still calls `parse_agy_transcript_message_only`** (`{role,content}` only) → real AGY2 Stop hooks ingest **zero** step lines (F41 skip-all). **P0 for T236** (T234 F13 deferred this wire). |
| **D4** | Batch `import_antigravity_sessions` uses `parse_overview_file` + `extract_turns` (message_only) — **parse OK**; **`project_hash: None`** for all brain sources → **`default_project_id`** (nightly `--no-project-context` / cwd `.env` e.g. `test-alias`). Root “not detecting” UX. |
| **D5** | `~/.gemini/antigravity-cli/history.jsonl` lines: `{display, timestamp, workspace, conversationId?, type?}` — usable **index** for binding; unused today. Case variance exists (`C:\dev\dedupe` vs `C:\dev\Dedupe`). |
| **D6** | T235 shipped: detect, wiring, `harness install --harness agy`, F34 map (`conversationId`→`sessionId`, `workspacePaths[0]`→`projectHash`, else `agy-unbound`), PS wrapper, F35 soft fullyIdle skip, dual-path probe note. **Not** history binding; **not** step parser on hook. |
| **D7** | Official hooks SOOT: `~/.gemini/config/hooks.json` (+ workspace `.agents/hooks.json`); secondary CLI path `~/.gemini/antigravity-cli/hooks.json` (path-misalignment reports). Stop stdin includes `conversationId`, `workspacePaths`, `transcriptPath`, `fullyIdle` (required). Stop stdout: `decision: "continue"` re-enters loop; **any other value allows stop**. |
| **D8** | 5‑minute quiescence hard-skips active files; no `--force` on `antigravity-import`. |
| **D9** | `get_unsummarized_sessions`: `status=completed AND summary_memory_id IS NULL` — new turns after `SessionSummaryCreated` do **not** re-queue today. |
| **D10** | `Docs/antigravity-rule.md` still claims “does not support session hooks” — **stale** vs T235 + CAPABILITIES. |
| **D11** | Official Stop **requires** JSON `decision` on stdout (`continue` re-enters; any other allows stop). T235 wrapper runs `agy-hook` then `exit 0` **without** capturing stdout → human lines (`Successfully ingested…`) leak to AGY. **Elevated hard (AI2 M1 / F8).** |
| **D12** | Dep pins: no intentional bump (workspace `serde`/`serde_json` 1.0, `clap` 4.5, edition 2024). crates.io latest `serde_json` 1.0.151 / `serde` 1.0.229 / clap 4.6.x remain under caret; **do not** bump for T236 unless gate forces. |
| **D13** | Live vs batch **turn-id diverge**: hook `v5(session,"agy-turn-{i}")` vs batch `v5(session,"turn-{i}")` (AI2 M2). Delta-by-count prevents double ingest; identity unstable across paths. |
| **D14** | Live lines may set `truncated_fields:["content"]`; sibling **`transcript_full.jsonl`** exists on this machine and holds full text (AI2 M5). |
| **D15** | `source_meta:{session_id}` shared across `antigravity` + `antigravity-cli` brains with same conversationId → meta overwrite risk (AI2 M6). |
| **D16** | SYSTEM scheduled nightly wrapper uses `--no-project-context --skip-import` — **does not run AGY import**. T236 binding applies to **manual** `nightly` / `antigravity-import` / future T239; do not claim scheduled import fixed (AI2 L6). |
| **D17** | `Docs/WORKFLOWS.md` claims `antigravity-import` prints a **JSON status object** — false today (human text only) (AI2 L2). |

## Frozen direction

### A. Live hook path (parser + binding)

| ID | Decision |
|----|----------|
| **F1** | **`agy-hook` SOOT parser:** detect/step-first: if first non-empty JSONL object has `step_index` or (`source`+`type`) → `AntigravityStep` + `extract_turns` / message_only. Else legacy `{role,content}` → simple filter. **Serde:** `#[serde(default)]` / ignore unknown fields on steps (AI1 #1). Fail-open per line (T234 F41); debug-trace skip, never fail whole file. |
| **F2** | Shared helper **hard:** `parse_transcript_for_ingest(path) -> Vec<…>` used by **hook and batch**. Also owns **deterministic turn-id SOOT** (AI2 M2): prefer `v5(session, "turn-{step_index}")` when `step_index` present; else `v5(session, "turn-{i}")` (retire `agy-turn-{i}`). Same fixture → same turn IDs on both paths (AC19). |
| **F3** | **Project resolve (hook + batch, ordered):** (1) **normalize** `projectHash`/workspace via `ai_brains_path` **before** resolve and **before** `ensure_project_alias` (AI2 M3 — live path today links raw case); (2) alias resolve; (3) repository path-alias if available; (4) `AI_BRAINS_PROJECT_ID` **only** when hash is `agy-unbound` / empty (AI2 M4 — **narrow** current always-on env fallback); (5) path-derived project: alias = normalized path; display name from basename (may be lowercased by normalize — document). Pre-T236 raw-case aliases may not match until re-link — note in docs. |
| **F4** | Keep agy-hook payload schema frozen (`transcriptPath`, `sessionId`, `projectHash`) — no widen unless proven necessary. |
| **F5** | Never populate `IngestRequest.thinking` (T234 F17). |
| **F6** | Delta max-turn-index skip remains (T49). |
| **F7** | **fullyIdle:** keep T235 F35 soft-skip (`false` → exit 0, no ingest). Document; optional stderr count. Hard re-queue / decision:`continue` **out** (soft residual). |
| **F8** | **Wrapper Stop stdout SOOT (AI2 M1 hard):** capture/suppress `agy-hook` stdout; emit **only** JSON to stdout allowing stop — prefer `{"decision":"allow"}` or `{}` (never `"continue"`); all diagnostics on **stderr**. Reinstall via `harness install --harness agy` updates wrapper body. Hermetic test: wrapper body (or pure emit helper) asserts stdout has no human ingest prose. Dual-write SOOT remains `~/.gemini/config/hooks.json`. |

### B. Batch project binding (highest user-visible fix)

| ID | Decision |
|----|----------|
| **F9** | **Load history index** from `~/.gemini/antigravity-cli/history.jsonl` (+ optional legacy `antigravity/history.jsonl`). Pure: `conversationId → workspace`. **Tie-break (AI2 L4 / AI1 #2):** rows with non-empty workspace + conversationId; sort by `(timestamp_ms asc, line_index asc)`; last wins (latest timestamp, then later line). Timestamp is epoch-ms integer on live files; parse fail → treat as 0 + line order only. Normalize path before store. |
| **F10** | At discover/import: for brain sources with `session_id == conversationId`, set workspace/`project_hash` from F9; resolve like F3. |
| **F11** | History is **binding index only** — never content SOOT. |
| **F12** | **Unbound bucket (AI2 L3):** shared stable alias `agy-unbound` with **recognizable project name** e.g. `(unbound AGY)` — hook + batch resolve to **same** project id once alias exists. Increment `unbound_project`. When history miss: use `default_project_id` **only if** `allow_default_project: true` (manual import may). **Normative manual `ai-brains nightly` + default import API for non-interactive:** `allow_default_project: false`. **Scheduled SYSTEM nightly** currently `--skip-import` (D16) — document; do not re-enable import in T236. |
| **F13** | Path case: store **normalized** path as alias key; resolve with normalize-on-lookup (**hook + batch**, AC17 both). |
| **F14** | Discovery remains brain/`transcript.jsonl` (+ overview) primary; tmp ProjectChat keeps dir-name project_hash. |
| **F15** | Soft: `conversations/<id>.db` fallback — **not DoD**. |
| **F29** | **Prefer full transcript (AI2 M5):** when ingest path is `…/logs/transcript.jsonl` and sibling `transcript_full.jsonl` exists and is readable, parse **full** for content SOOT (same message_only filter). If only truncated present, ingest truncated content; docs honesty note. Discovery/mtime still track both for meta (F30). |
| **F30** | **source_meta key (AI2 M6):** key by **normalized source path** (e.g. `source_meta:agy:{sha256_or_stable_path_key}`) — not bare `session_id` alone — so dual-root brains do not clobber each other. |

### C. Import UX / stats / quiescence

| ID | Decision |
|----|----------|
| **F16** | Import summary (stderr human; **no lying JSON**): counters `found`, `imported_turns`, `sessions`, `skipped_quiescent`, `skipped_unchanged_meta`, `unbound_project`, `bound_via_history`, `bound_via_path` (**AI2 L5:** path = resolved from workspace/path-derived alias **without** history hit; drop vague `bound_via_hook_path` name). Optional `--json` **soft residual** only if cheap; else fix WORKFLOWS.md (F31). Human lines must not claim “complete” with only silent skips. |
| **F17** | **Re-summarize re-queue (AI1 #3 affirm):** prefer **query OR** — `summary_memory_id IS NULL OR EXISTS (turns with occurred_at > summarized_at)` — no direct projection wipe. Document if table names differ (`turn_projection`). Else T239 waiver. |
| **F18** | Quiescence: keep 300s default; add `antigravity-import --force` to skip window. |
| **F19** | Hermetic tests: fake `USERPROFILE`/`HOME` + fixture history + brain tree (T205 pattern). |
| **F31** | **Docs honesty (AI2 L2):** fix `Docs/WORKFLOWS.md` JSON claim; antigravity-rule + CAPABILITIES + OPERATIONS. |
| **F32** | **Dead code (AI2 L1):** use or delete unused `filter_recent_sessions` during import refactor. |
| **F33** | **mapping_delta_smoke / env tests (AI2 M4):** update tests that rely on env fallback for non-`agy-unbound` hashes when F3 narrows. |

### D. Docs / capability / series

| ID | Decision |
|----|----------|
| **F20** | Rewrite `Docs/antigravity-rule.md`: hooks **supported** via `harness install --harness agy`; location table AGY2; message-only; history binding; pin still recommended mid-session. |
| **F21** | CAPABILITIES / OPERATIONS: AGY seamless **Partial→Implemented** for live+batch with binding caveats; CAPTURE independence unchanged. |
| **F22** | CHANGELOG + README-T234-T239 T236 row. |
| **F23** | `antigravity_capability` notes: hooks installable + batch binding; Partial only if re-summarize deferred. |
| **F24** | Zero new crates; no dep bump. |
| **F25** | Capture independence: no models/embeddings/graph on ingest path. |
| **F26** | No unwrap/expect in production. |
| **F27** | Contracts: only if import JSON DTO or agy-hook schema changes — prefer keep schema; if CLI gains structured import report as JSON flag, document empty-state. |
| **F28** | Parallel: T237/T238 may reuse path-resolve helpers if extracted; do not block on them. |
| **F34** | **Soft residual (AI2 L7):** per-Stop full re-parse O(n) — optional byte-offset watermark later; not DoD. |

## Non-goals

- SessionStart PreInvocation context injection.
- Desktop IDE-only UI / conversations.db as primary SOOT.
- claude-mem wipe; foreign hooks clobber.
- Grok/OpenCode importers (T237/T238).
- Full multi-harness nightly flags (T239).
- **Re-enable SYSTEM scheduled AGY import** (D16 / T239).
- Project-local `.agents/hooks.json` install as default (C7).
- Widening agy-hook schema without need.
- fullyIdle hard re-queue / decision continue loops.
- MSI packaging.
- Import `--json` machine report (soft only).

## Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | Hermetic **step-shaped** transcript fixture → hook/shared parse ingests **only** user + final assistant; zero tool/thinking memories. |
| **AC2** | Hermetic `{role,content}` fixture still works (legacy path). |
| **AC3** | Tool-heavy step fixture (VIEW_FILE / RUN_COMMAND / thinking) → zero tool/thinking content stored. |
| **AC4** | Discover still finds `antigravity-cli` brain transcripts; missing overview OK. |
| **AC5** | **Binding:** fixture history maps `conversationId` → hermetic workspace path → imported session `project_id` matches that path/alias — **not** accidental `test-alias`. |
| **AC6** | Project-scoped recall/preflight for bound project sees imported AGY turns without `--global`. |
| **AC7** | Missing history/workspace → `unbound_project` ≥1; `allow_default_project: false` does not attach to cwd env project. |
| **AC8** | Import stats print bound/unbound/quiescent/unchanged counters (human); WORKFLOWS no false JSON claim (F31). |
| **AC9** | `--force` imports a file modified within 5 minutes (hermetic mtime). |
| **AC10** | Without `--force`, mtime &lt; 5 min skipped and counted `skipped_quiescent`. |
| **AC11** | Docs: `antigravity-rule.md` no longer claims hooks unsupported; install path documented. |
| **AC12** | CAPABILITIES/OPERATIONS AGY section matches shipped behavior (incl. scheduled nightly skip-import honesty). |
| **AC13** | Re-summarize: session with prior summary + new imported turns appears in unsummarized set (or T239 waiver + ISSUES). |
| **AC14** | T235 install/F34 map regression green; install merges managed key only. |
| **AC15** | Full gate: fmt, clippy `-D warnings`, nextest workspace, deny, audit; no new audit allows. |
| **AC16** | History multi-entry same conversationId → latest by `(timestamp, line_index)` wins. |
| **AC17** | Path case variants resolve same alias after normalize — **batch and live hook** (AI2 M3). |
| **AC18** | **Wrapper stdout (M1):** managed wrapper (or pure function generating its stdout contract) emits only allow-stop JSON on stdout; diagnostics stderr; **no** agy-hook human prose on stdout. |
| **AC19** | **Turn-id SOOT (M2):** same step fixture → identical turn UUIDs via hook path helper and batch path helper. |
| **AC20** | **Live unbound (M4):** hook with real unresolved workspace path does **not** fall back to `AI_BRAINS_PROJECT_ID`; uses path-derived project or `agy-unbound` rules per F3; env fallback only for empty/`agy-unbound`. |
| **AC21** | **transcript_full (M5):** when sibling full file exists, ingested content length/content matches full (not truncated stub) for a fixture with `truncated_fields`. |
| **AC22** | **source_meta (M6):** two sources same session_id different paths keep independent meta (no clobber). |

## Implement order (when go)

1. **F8 + AC18** wrapper stdout SOOT (before live Stop dogfood).  
2. **F9–F13 + AC5–AC7, AC16–AC17** history index + batch bind + stable unbound.  
3. **F1–F3 + F2 turn-id + F29 + AC1–AC3, AC19–AC21** shared parser + hook wire + full prefer.  
4. **F30 + F16–F18 + F32 + AC8–AC10, AC22** meta key, stats, `--force`, dead code.  
5. **F17 + AC13** re-summarize query (or residual).  
6. **F20–F23 + F31 + AC11–AC12** docs (incl. WORKFLOWS + scheduled skip).  
7. Manual dogfood + full gate + review.

## Risks

| Risk | Mitigation |
|------|------------|
| Live step parse miss edge shapes | Fixtures; serde default; fail-open lines |
| history.jsonl lag / missing conversationId | Tie-break F9; F12 unbound |
| Path case / WSL dual | normalize before alias (F3/F13) |
| Manual nightly / env hijack | F12 `allow_default_project: false` |
| Wrapper stdout breaks Stop | **F8 hard** AC18 |
| Divergent turn IDs | **F2 hard** AC19 |
| Hook env hijack | **F3(4) hard** AC20 |
| Truncated memories | **F29** AC21 |
| Dual-root meta clobber | **F30** AC22 |
| Re-summarize projection | Prefer OR query; no raw wipe |
| Scope creep T239 / scheduled import | D16 honesty; out of scope |

## Research notes (2026-08-08)

- Official hooks: [antigravity.google/docs/hooks](https://antigravity.google/docs/hooks) — Stop stdin/stdout contract; config `~/.gemini/config/` or `.agents/`.
- Live: step-shaped transcript; `transcript_full.jsonl` present; history epoch-ms + case variance.
- No new crates; no dep bump for T236.

## Residual / handoff

| Residual | Owner |
|----------|-------|
| Multi-harness nightly + scheduled import re-enable | T239 |
| Grok / OpenCode | T237 / T238 |
| fullyIdle hard continue policy | Soft |
| conversations.db fallback | Soft |
| Per-Stop byte-offset watermark (L7) | Soft F34 |
| Import `--json` | Soft |
| Project-scope hooks install | Later / C7 opt-in |
| Capture refuse thinking Some | T234 soft F24 |
| Display ASSISTANT: strip | T224 |
| Preflight summary JSON harness | T220 |

---

## §14 — AI review fold-in disposition (2026-08-08)

### AI1 (architecture affirm + resilience)

| Item | Verdict | Disposition |
|------|---------|-------------|
| Architecture diagram / F1–F28 affirm | Agree | Affirmed |
| Serde ignore unknown / fail-open lines | Agree | **Elevate** F1 (`#[serde(default)]` + debug skip) |
| History case normalize + timestamp/line order | Agree | **Elevate** F9/F13 + AC16 |
| Re-summarize OR-query | Agree | Affirmed F17 / AC13 |
| AC1–AC17 verification matrix | Agree | Affirmed; extended AC18–AC22 |
| Action table (parser/hook/history/stats/docs/tests) | Agree | Plan phases |

### AI2 (M1–M6 / L1–L7)

| ID | Sev | Verdict | Disposition |
|----|-----|---------|-------------|
| **M1** | high | **Agree** — Stop requires JSON decision; wrapper leaks agy-hook stdout | **F8 hard** + **AC18** |
| **M2** | high | **Agree** — `agy-turn-{i}` vs `turn-{i}` diverge | **F2 hard** turn-id SOOT + **AC19** |
| **M3** | med | **Agree** — hook must normalize before resolve/alias | **F3/F13** + **AC17** covers hook |
| **M4** | med | **Agree** — env fallback on any None contradicts F3(4) | **F3(4)** + **AC20** + **F33** test update |
| **M5** | med | **Agree** — truncated_fields; prefer full sibling | **F29** + **AC21** + D14 |
| **M6** | med | **Agree** — source_meta session-only clobber | **F30** + **AC22** + D15 |
| L1 | low | Agree — dead `filter_recent_sessions` | **F32** |
| L2 | low | Agree — WORKFLOWS JSON lie | **F31** + AC8 |
| L3 | low | Agree — stable unbound name | **F12** `(unbound AGY)` |
| L4 | low | Agree — explicit timestamp+line tie-break | **F9** |
| L5 | low | Agree — define counters | **F16** `bound_via_path` |
| L6 | low | Partial — plan.md **exists** (AI2 stale); scheduled `--skip-import` true | **D16** honesty; **not** re-enable in T236 |
| L7 | low | Agree soft — O(n) re-parse | Soft **F34** not DoD |

### Declined / out of scope

| Item | Why |
|------|-----|
| Re-enable SYSTEM nightly AGY import | Security/ops product decision → T239 |
| Import `--json` as DoD | Soft; fix doc lie first |
| Byte-offset watermark DoD | Soft F34 |
| Widen agy-hook schema | Prefer F3/F8 without schema change |
| fullyIdle hard re-queue | Soft residual |
