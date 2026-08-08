# T236 — Antigravity 2 CLI seamless ingest

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Research 2026-08-08 — AGY 2 hooks + `transcript.jsonl`; live diagnosis 2026-08-08 (location OK; project binding + timing feel like “not detecting”)
- **Category:** FEATURE
- **Depends on:** T234 (message-only filter); T235 (install UX; install backend can land here)
- **Absorbs:** Stale `Docs/antigravity-rule.md` “no hooks”; capability `supports_hooks: false`; unify `agy.rs` simple JSONL vs `antigravity.rs` step format; **workspace→project binding**; history index for batch project resolve
- **Related:** existing `agy-hook`, `antigravity-import`, nightly import; series [README](../README-T234-T239-HARNESS-INGEST.md)
- **Does not absorb:** multi-harness nightly orchestration (T239); Grok/OpenCode (T237/T238); shared message-only module itself (T234)

## Objective

Make **Antigravity 2 CLI (`agy`)** capture seamless and **findable in the right project**:

1. **Live:** `Stop` (optional throttled `PostInvocation`) → `ai-brains agy-hook` with `transcriptPath` + session id + **workspace-derived project**.  
2. **Batch:** keep scan of `~/.gemini/antigravity-cli/brain/**/transcript.jsonl` (legacy antigravity / ide paths too).  
3. **Both paths:** T234 message-only turns only.  
4. **Fix the “not detecting” UX:** bind sessions to real workspaces (Orca/STL/…) so project-scoped preflight/recall see AGY work — not only `--global` / wrong default project.

## Diagnosis (frozen facts — 2026-08-08)

| Fact | Detail |
|------|--------|
| D1 | Old `~/.gemini/antigravity/brain` + `overview.txt` largely gone on this machine |
| D2 | SOOT content path is already scanned: `~/.gemini/antigravity-cli/brain/<id>/.system_generated/logs/transcript.jsonl` (+ optional `transcript_full.jsonl`, not primary) |
| D3 | Nightly **does** discover brains (e.g. 36 sessions); import counts are non-zero on some nights |
| D4 | **Root UX bug:** batch brain sources set `project_hash: None` → all turns land on **default project** (nightly `--no-project-context` / cwd `.env` e.g. `test-alias`) |
| D5 | `history.jsonl` has `workspace` + `conversationId` (usable project map) but is **unused** today |
| D6 | `conversations/<id>.db` exists; **out of scope as primary SOOT** unless transcript missing (optional fallback later) |
| D7 | Live gap: no Stop hook → same-day AGY work missing until next nightly or manual import |
| D8 | 5‑minute quiescence skip delays manual catch-up right after a session |
| D9 | extract_turns already drops tool-only `PLANNER_RESPONSE` (message-only intent); keep via T234 SOOT |
| D10 | New turns on already-summarized sessions may not re-queue synthesis (observe; fix here if AGY-only, else T239) |

## Frozen direction (draft)

### Capture + hooks

| ID | Decision |
|----|----------|
| F1 | Official hook config: user-global `~/.gemini/config/hooks.json` (and documented AGY paths); namespaced entry e.g. `ai-brains-capture` — never wipe foreign hooks |
| F2 | Hook event primary: **`Stop`** when `fullyIdle` (if present) → capture; never block stop (`decision` allow/stop-safe) |
| F3 | Payload: extend/reuse `agy-hook` — `transcriptPath`, `sessionId`, **`workspacePaths` or resolved `projectId`**, optional `projectHash` |
| F4 | **Parser SOOT:** all AGY transcript ingest uses step parser + **T234** filter; thin/remove simple `{role,content}` path if live files are step-shaped |
| F5 | Delta ingest: keep max-turn-index skip (T49); meta mtime/size cache remains |
| F6 | `supports_hooks: true` (or Partial with honest notes) after install path works |
| F7 | Rewrite `Docs/antigravity-rule.md` — location table, hooks + transcript, pin still recommended for decisions |
| F8 | T235 backend: `harness install --harness agy` merges hooks idempotently |
| F9 | Message-only: never store tool names/args/thinking as memory content (T234) |

### Project binding (why AGY work “vanishes” under preflight)

| ID | Decision |
|----|----------|
| F10 | **Batch project resolve (required):** for each brain `conversationId`, resolve workspace via **`~/.gemini/antigravity-cli/history.jsonl`** (`conversationId` → latest/most-specific `workspace`) → `ai_brains_path` normalize → project resolve / path alias / set-alias fallback |
| F11 | **Hook project resolve:** prefer hook `workspacePaths[0]` (or first existing path) over history; then same resolve chain as F10 |
| F12 | If resolve fails: import under explicit **unscoped/default** bucket **and** log/warn with workspace path + “run project set-alias”; do **not** silently attribute to unrelated cwd `.env` project when nightly uses `--no-project-context` |
| F13 | Discovery remains brain/`transcript.jsonl`-primary; history is **index for binding**, not content SOOT |
| F14 | Optional soft: if transcript missing but `conversations/<id>.db` exists, document residual — implement only if needed after F10–F13 |

### Operator honesty

| ID | Decision |
|----|----------|
| F15 | Import summary stderr/json: `found`, `imported_turns`, `sessions`, `skipped_quiescent`, `skipped_unchanged_meta`, `unbound_project` counts |
| F16 | Revisit 5‑minute quiescence: keep for concurrent write safety; document; consider `--force` / shorter window for manual import |
| F17 | If AGY import appends turns to a session already marked summarized, **mark session unsummarized again** (or equivalent) so next nightly re-summarizes — unless T239 owns multi-harness re-queue (then implement once in shared helper and call from here) |

## Non-goals

- SessionStart context injection via PreInvocation (optional later / T235 preflight).  
- Desktop IDE-only UI.  
- claude-mem compatibility beyond not clobbering.  
- Opening multi-GB conversations.db as default path.  
- Grok/OpenCode importers (T237/T238).  
- Full multi-harness nightly CLI flags (T239) — AGY path stays callable from nightly as today.

## Acceptance sketch

| AC | Sketch |
|----|--------|
| AC1 | Hermetic Stop-like payload + fixture transcript → only user/assistant in vault |
| AC2 | Tool-heavy fixture → zero tool/thinking memories |
| AC3 | Discover includes `antigravity-cli` brain (regression); missing `overview.txt` still OK |
| AC4 | Install dry-run + real install merge without deleting foreign hooks |
| AC5 | Docs/capability no longer claim “hooks unsupported”; location table matches AGY2 |
| AC6 | **Binding:** fixture history maps `conversationId` → `C:\dev\Orca` → vault project matches Orca/alias — **not** AI-Brains `test-alias` by accident |
| AC7 | Project-scoped `recall`/`preflight` for that project sees imported AGY turns without requiring `--global` |
| AC8 | Import stats include `unbound_project` when history/workspace missing |
| AC9 | Manual import after >quiescence window picks up same-day session (document or `--force`) |
| AC10 | Re-summarize path: new turns on prior-summarized session re-enter unsummarized set (or tracked T239 waiver with test) |

## Implement notes (when go)

1. Land F10–F12 + AC6/AC7 first if shipping without hooks — **highest user-visible fix**.  
2. Then F4/T234 wire + docs.  
3. Then F1–F3 hooks + T235 install backend.  
4. F17 re-queue with or after T239 shared helper.

## Risks

- Hook event name drift (Stop vs SessionEnd vs settings.json BeforeAgent) — probe live `agy` at implement time.  
- Windows command quoting for hook `command` — PowerShell-safe wrapper under `%USERPROFILE%\.ai-brains\`.  
- history.jsonl may lag or omit workspace on some entries — fall back F12.  
- Multiple workspaces per conversation over life — prefer latest non-empty workspace or longest path under known roots.
