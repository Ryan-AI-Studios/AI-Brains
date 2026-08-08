# T238 — OpenCode seamless ingest

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Research 2026-08-08 — OpenCode plugins + `session list` / `export` (avoid raw multi-GB SQLite)
- **Category:** FEATURE
- **Depends on:** T234; T235 install backend
- **Related:** stub `opencode` adapter capability Partial / wrapper

## Objective

Capture **OpenCode** sessions seamlessly:

1. **Live:** TypeScript/JS plugin on `session.idle` (and optionally settled `message.updated`) → call `ai-brains` with message-only payload.  
2. **Batch:** `opencode session list` + `opencode export <id>` for sessions updated since cursor — **not** direct schema coupling to `opencode.db`.  
3. **Message-only** via T234.

## Frozen direction (draft)

| ID | Decision |
|----|----------|
| F1 | Ship managed plugin under user config: `~/.config/opencode/plugins/ai-brains-capture.*` (or npm later); register via `opencode.json` plugin list if required |
| F2 | Primary event: **`session.idle`** — export or SDK-read session messages → filter → ingest |
| F3 | Prefer **CLI export** subprocess over linking OpenCode internals (version resilience) |
| F4 | Nightly: list sessions (`--format json`), filter by `updated` watermark stored in vault/meta or `~/.ai-brains/opencode-import-cursor.json` |
| F5 | Project binding: export/session `directory` → path normalize → project resolve |
| F6 | Capability: Partial→Full when plugin+batch work; `supports_hooks: true` meaning plugin events |
| F7 | T235: `harness install --harness opencode` |
| F8 | Explicit non-goal: open 10GB SQLite as primary API |

## Non-goals

OpenCode TUI theming; ACP bridge; reading tool-output cache dirs.

## Acceptance sketch

| AC | Sketch |
|----|--------|
| AC1 | Fixture export JSON → only user/assistant turns ingested |
| AC2 | Plugin install dry-run paths correct on Windows |
| AC3 | Batch respects watermark (no full re-export every night) |
| AC4 | Missing `opencode` binary → soft skip with clear status |
| AC5 | Docs: plugins + export path |

## Risks

- Export shape changes — golden fixtures + tolerant serde.  
- Plugin runtime (Bun) availability — document prerequisite; fail soft.  
- session.idle frequency — debounce / delta by message id.
