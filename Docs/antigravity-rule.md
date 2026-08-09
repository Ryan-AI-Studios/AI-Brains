# Antigravity / AGY 2 Integration with AI-Brains

AI-Brains supports **Antigravity 2 CLI (`agy`)** live capture via Stop hooks and **batch import** of brain logs, with **history.jsonl workspace→project binding** so turns land under the right project.

## Live hooks (supported)

Install (user-global, message-only):

```powershell
ai-brains harness install --harness agy --yes
# After T236: reinstall once so the Stop wrapper emits allow-stop JSON only on stdout
ai-brains harness install --harness agy --yes
```

| Location | Role |
|----------|------|
| `~/.gemini/config/hooks.json` | Official AGY hooks SOOT — managed key `ai-brains-capture` |
| `~/.ai-brains/hooks/agy-stop.ps1` | Wrapper: maps Stop → `agy-hook` payload; **stdout = only** `{"decision":"allow"}`; diagnostics on stderr |
| Brain logs | `~/.gemini/antigravity-cli/brain/<id>/.system_generated/logs/transcript.jsonl` (+ optional `transcript_full.jsonl`, legacy `overview.txt`) |

**Message-only SOOT:** user prompts + final assistant text only. Tool steps (`VIEW_FILE`, `RUN_COMMAND`, …), thinking/reasoning, and system chrome are dropped.

**Project binding (live):** `workspacePaths[0]` → normalized path alias (or `agy-unbound`). `AI_BRAINS_PROJECT_ID` is used **only** when the hash is empty/`agy-unbound`.

**fullyIdle:** when `false`, the wrapper soft-skips ingest (exit 0 + allow-stop JSON).

## Batch import

```powershell
ai-brains antigravity-import --days 7
ai-brains antigravity-import --days 30 --force   # skip 5-minute quiescence
```

- Discovers brains under `~/.gemini/{antigravity,antigravity-cli,antigravity-ide}/brain/` and project chats under `~/.gemini/tmp/…/chats/`.
- Binds `conversationId` → workspace via `~/.gemini/antigravity-cli/history.jsonl` (optional legacy `antigravity/history.jsonl`). Latest `(timestamp_ms, line)` wins; paths are normalized.
- Missing history → stable project alias **`agy-unbound`** / display **`(unbound AGY)`** — does **not** attach unbound brains to the cwd `.env` project by default (`allow_default_project: false`).
- Prefers sibling **`transcript_full.jsonl`** when present (truncated `transcript.jsonl` may list `truncated_fields`).
- Human stats on **stderr** (found, imported_turns, sessions, skipped_quiescent, skipped_unchanged_meta, unbound_project, bound_via_history, bound_via_path). **Not** a JSON status object unless a future `--json` is added.
- **`--force`** skips the 300s quiescence window for recently modified files.

### Nightly honesty

- Manual / user-principal `ai-brains nightly` (without `--skip-import`) runs **multi-harness** import: AGY → Grok → OpenCode (message-only; unbound anti-hijack). Per-source skips: `--skip-import-agy` / `--skip-import-grok` / `--skip-import-opencode`.
- **SYSTEM scheduled nightly** keeps `--skip-import` by default (T239 D12). Do not assume Session 0 Task Scheduler jobs import AGY (or Grok/OpenCode) history.

## Mid-session: pin is still recommended

Hooks capture turns after Stop; they do not replace pinning decisions mid-session:

```
ai-brains pin "DECISION: …"
ai-brains pin "CONSTRAINT: …"
```

## Orientation at session start

```powershell
ai-brains safety sync
ai-brains preflight --max-words 1000
```

## Commands reference

| Action | Command | Notes |
|---|---|---|
| Install Stop hook | `ai-brains harness install --harness agy --yes` | Reinstall after T236 for wrapper stdout SOOT |
| Status | `ai-brains harness status` | Detect + wiring |
| Real-time hook | `agy-hook --payload '{…}'` | Prefer install wrapper; diagnostics on stderr |
| Import recent | `ai-brains antigravity-import --days 7` | History bind + message-only |
| Force import | `ai-brains antigravity-import --force` | Skip 5-minute quiescence |
| Nightly | `ai-brains nightly` | Multi-harness import (agy→grok→opencode) unless `--skip-import` |
| Pin | `ai-brains pin "…"` | Mid-session decisions |
| Recall | `ai-brains recall "topic"` | Project-scoped by default |

## What NOT to do

- **Do not** expect human ingest lines on AGY Stop **stdout** (wrapper allows stop with JSON only).
- **Do not** assume scheduled SYSTEM nightly imports AGY/Grok/OpenCode (keeps `--skip-import`).
- **Do not** rely on env project for non-unbound workspaces (path-derived / history bind).
- **Do not** skip pinning mid-session for decisions you need immediately.
