# T234–T239 — Seamless multi-harness session ingest (placeholders)

**Source:** Research 2026-08-08 — Grok Build, OpenCode, Antigravity 2 CLI (`agy`); retire pure-log reliance on old Antigravity.
**Status:** T234 **Completed** (PR #100 `2ea8897`); T235 **Completed** (PR #101 `b1a0ecc`); T236 **Completed** (PR #102 `d53e4be`); T237 **Completed** (PR #104 `459fc55`); T238 **Completed** (PR #106 `3378a02`); T239 **Completed** (PR #108 `a271a99` multi-harness nightly).
**Prior related:** track033 antigravity import; T48/T49 agy-hook delta; nightly `antigravity-import`; Capture Privacy mandate.
**T234 SOOT path:** `crates/ai-brains-adapters/src/message_only.rs` (`filter_turn`, `classify_antigravity_step`, `filter_grok_history_*`, `filter_opencode_message*`, `extract_user_text`, sole-tool JSON guard).

## Product north star

Make AI-Brains **detect the active coding harness**, **offer (or auto-install) hooks/plugins at preflight**, and **always capture only user prompts + final assistant messages** — never tool calls, tool results, or hidden thinking/reasoning.

Live hooks provide freshness; durable log/export import on nightly provides completeness. Both paths share one **message-only** filter.

## Series map

| Track | Name | Role | Priority |
|-------|------|------|----------|
| **T234** | Message-only capture contract | Shared SOOT filter + capability truth — **Completed** | P0 foundation |
| **T235** | Harness detect + preflight hook UX | **Completed** PR #101 — detect + wiring + `harness *` + preflight/doctor; AGY install ready; Grok/OpenCode/Claude/Codex pending | P0 UX |
| **T236** | AGY 2 seamless ingest | **Completed** PR #102 — wrapper stdout SOOT + step parse + history bind + turn-id + `--force` + re-summarize + AC6 | P1 |
| **T237** | Grok Build seamless ingest | **Completed** — Stop+SessionEnd empty-stdout wrapper + F11 user_query filter + `grok-hook`/`grok-import` + install_ready (not updates; not UserPromptSubmit DoD) | P1 |
| **T238** | OpenCode seamless ingest | **Completed** PR #106 — plugin `session.idle` + nested export + synthetic drop + watermark batch (never SQLite) | P1 |
| **T239** | Nightly multi-harness import | **Completed** PR #108 — multi-source nightly (agy→grok→opencode); per-source skip; fail-open; `last_multi_import` status; SYSTEM keeps skip-import | P1 ops |

## Suggested implement order

1. **T234** — pure filter + tests first (no harness wiring).
2. **T235** — detect + preflight prompt/install dry-run (install can no-op until T236–T238 land).
3. **T236** AGY2 (lowest risk: `agy-hook` + brain scan already exist).
4. **T237** Grok (highest daily value on this machine).
5. **T238** OpenCode.
6. **T239** unify nightly + doctor/preflight “harness wiring” summary.

Parallel after T234+T235: T236/T237/T238 can proceed on non-intersecting adapters if coordinated.

## Hard invariants (all tracks)

| ID | Rule |
|----|------|
| **C1 Capture Privacy** | Store **only** user prompt text and **final** assistant response text. |
| **C2 No tools** | Never ingest tool_call / tool_result / backend_tool / PreToolUse bodies as memory content. |
| **C3 No thinking** | Never ingest `thinking`, `reasoning`, encrypted CoT, planner-internal monologue without user-visible content. |
| **C4 Capture independence** | Hook scripts call CLI/daemon only; capture path must work without models/embeddings/graph. |
| **C5 Dual path** | Hooks/plugins for live; durable scan/export for backfill — not either-or. |
| **C6 Consent** | Preflight may **offer** hook install; auto-write only with prior opt-in or explicit `--yes` / config flag. |
| **C7 No repo pollution** | Default install targets user-global hook paths (`~/.grok/hooks`, `~/.gemini/config`, `~/.config/opencode`), not project trees, unless user chooses project scope. |

## Non-goals of this series

- Capturing full agent traces for debug (use harness-native logs).
- Replacing manual `pin` for decisions mid-session (hooks complement pins).
- Cloud relay of harness transcripts.
- T233 multi-root Ledgerful symbols (orthogonal; nightly may call into T233 later).

## Registry

See `conductor/conductor.md` T234–T239 rows and each `trackT23x-*/spec.md`.
