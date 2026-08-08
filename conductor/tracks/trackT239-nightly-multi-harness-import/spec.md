# T239 — Nightly multi-harness import orchestration

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Research 2026-08-08 — dual-path completeness for AGY/Grok/OpenCode
- **Category:** FEATURE / OPS
- **Depends on:** T234; preferably T236–T238 importers (can no-op missing backends)
- **Related:** T79 `--skip-import`; nightly pipeline; T233 multi-root (orthogonal symbols)

## Objective

Extend `ai-brains nightly` so harness session import is **multi-source**, **message-only**, **skippable**, and **observable**:

1. AGY/Antigravity brain scan (existing, fixed parsers).  
2. Grok `chat_history` backfill (T237).  
3. OpenCode export watermark import (T238).  
4. Status: what ran, counts, errors, skip reasons.  
5. Preflight/doctor can show “last multi-import” summary if cheap.

## Frozen direction (draft)

| ID | Decision |
|----|----------|
| F1 | Nightly phase order: multi-harness import → existing summarization/embed/… |
| F2 | Per-source flags: `--skip-import` (all) remains; add `--skip-import-agy` / `--skip-import-grok` / `--skip-import-opencode` (names TBD) |
| F3 | Each importer returns structured stats: sessions_seen, turns_ingested, skipped_delta, errors[] |
| F4 | Aggregate into nightly status / log; `nightly --status` shows last multi-import block |
| F5 | All importers **must** call T234 filter |
| F6 | Fail-open per source: one harness error must not abort others (record error, continue) |
| F7 | Capture independence: import works offline without LLM |
| F8 | Optional: `harness status` shows last import timestamps from same stats store |

## Non-goals

Live hooks (T235–T238); changing synthesis batch limits (T61); Ledgerful symbol phase (T70/T233).

## Acceptance sketch

| AC | Sketch |
|----|--------|
| AC1 | Nightly with all three fixtures → three sources report stats |
| AC2 | One source forced error → other sources still run |
| AC3 | `--skip-import` skips all harness importers |
| AC4 | Message-only: tool-heavy fixtures add zero tool memories |
| AC5 | `nightly --status` includes multi-import summary |
| AC6 | OPERATIONS.md nightly section updated |

## Risks

- Runtime budget: OpenCode export of many sessions — cap per run + watermark.  
- Path encoding (Grok URL-encoded cwd dirs) — use existing path crate.
