# T237 — Grok Build seamless ingest

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Research 2026-08-08 — Grok hooks + `~/.grok/sessions/**/chat_history.jsonl`
- **Category:** FEATURE
- **Depends on:** T234; T235 install backend
- **Related:** Claude hook pattern (Full capability); docs.x.ai Build hooks

## Objective

Capture **Grok Build** sessions seamlessly:

1. **Live hooks** (primary): `UserPromptSubmit`, `Stop`, `SessionStart`/`SessionEnd` as needed.  
2. **Nightly backfill** (secondary): scan `~/.grok/sessions/<cwd-key>/<session-id>/chat_history.jsonl` + `summary.json` for project binding.  
3. **Message-only** via T234 — drop `reasoning`, `tool_result`, `backend_tool_call`, system chrome.

## Frozen direction (draft)

| ID | Decision |
|----|----------|
| F1 | Hook files: user-global `~/.grok/hooks/ai-brains.json` (or merge managed block); project hooks only if user opts in (trust model) |
| F2 | **UserPromptSubmit** → record user text (prefer content inside `<user_query>` when present) |
| F3 | **Stop** → record final assistant text for turn; do not dump tool transcript |
| F4 | **SessionEnd** → session complete / stop-session equivalent |
| F5 | Optional **SessionStart** → soft preflight nudge only if cheap (no vault lock fights) |
| F6 | New adapter `grok` / harness id; capability Full when hooks+batch land |
| F7 | Batch importer: discover session dirs; parse chat_history; T234 filter; delta by session id + turn fingerprint |
| F8 | Project binding: `summary.json` cwd / git_root_dir → path normalize → project resolve / alias |
| F9 | T235: `harness install --harness grok` |
| F10 | Never read `updates.jsonl` as ingest source (noise) |

## Non-goals

Ingesting Grok cloud session traces; storing encrypted reasoning blobs; auto-approving project hooks trust.

## Acceptance sketch

| AC | Sketch |
|----|--------|
| AC1 | Fixture chat_history (user/assistant/reasoning/tool_result) → only user+assistant in vault |
| AC2 | Hook payload hermetic → CaptureService append |
| AC3 | Batch import idempotent on re-run |
| AC4 | Install merges hooks; `/hooks` still valid JSON |
| AC5 | CAPABILITIES + OPERATIONS Grok section |

## Risks

- Hook stdin schema evolution — pin against docs + golden fixtures.  
- Synthetic user lines (system-reminder) — T234 chrome policy.  
- Large sessions: Stop should not re-upload entire history; prefer turn-scoped or delta like agy-hook.
