# Track review: T279-PreflightSafetyVsHotspots

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT279-preflight-safety-vs-hotspots`  
**Date:** 2026-08-22  
**HEAD:** `448ef47`  

---

## Summary

Track T279 resolves a major preflight brief corruption issue observed during the 2026-08-21 CLI audit:
When running `ai-brains preflight --pretty --compact`, the `--- Repository Bearings & Safety ---` section displayed review-track skill prompts (`## Objective`, `## Important Details`) rather than actionable repository bearings or hotspots.

The root cause was twofold:
1. **Substring Matching in Safety SQL:** `preflight.rs` used `LIKE '%CONSTRAINT:%'` and `LIKE '%INVARIANT:%'`, causing any session transcript or harness log containing the word "CONSTRAINT:" anywhere in its body to match. Because `## Objective` was the first line of the document, it was rendered as the bearing heading.
2. **Missing Live Hotspot Integration:** While `ai-brains safety sync --dry-run` queries live hotspots via `ledgerful hotspots --json --limit 5`, `preflight` relied strictly on static vault pins, which are rarely pinned in daily development.

T279 fixes this with a clean, layered approach:
- **Leading-Line Safety GLOB (F1):** Replaces `LIKE '%...%'` with `safety_marker_glob_sql` (`CONSTRAINT:*`, `INVARIANT:*`, `HOTSPOT:*`, and `ASSISTANT:` prefixes at the start of content), preventing buried mentions in session dumps from matching.
- **Live Hotspot Injection (F2):** For project-scoped preflight, dynamically queries `ledgerful hotspots --json --limit 5` (fail-open) and renders live `HOTSPOT: {path} score={score:.2}` lines before leading constraint pins.
- **Honest Empty Display (F3):** When no live hotspots or leading constraint pins exist, always emits the Safety header with `SAFETY_EMPTY` (`No in-context hotspots. next: ai-brains safety sync --dry-run`), eliminating confusion while preserving summary badge counts.
- **Test Isolation (F13):** Adds `AI_BRAINS_PREFLIGHT_SKIP_LIVE_HOTSPOTS=1` to `hermetic_bin` so ambient workspace hotspots do not leak into hermetic integration test assertions.

The plan is well-bounded, maintains capture independence, and leaves top hotspots (`project.rs`, CLI `preflight.rs`, `doctor.rs`) untouched.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Robust log-line stripping in `preflight_safety::fetch_live_hotspots` (F35 / AC9):** Ensure the JSON parser locates the first line starting with `[` to cleanly skip preceding terminal logs or ANSI formatting from `ledgerful hotspots`.
- **m2: Word budget truncation on assembled Safety section (F3 / AC4):** Verify that `trim_to_word_budget_no_sentinel` is applied to the combined Safety text (live hotspots, leading constraint pins, or `SAFETY_EMPTY`) within `onboarding_budget`.

### Opportunities (O)
- **O1: Summary badge safety in `SAFETY_EMPTY` (AC14):** Ensure `SAFETY_EMPTY` strictly avoids the substring `"HOTSPOT:"` so that summary keyword matching (`matches("HOTSPOT:")`) does not count an empty state as an active hotspot.
- **O2: Pure unit tests for hotspot parsing (AC9 / AC2):** Provide direct unit tests for `format_safety_hotspot_line` and JSON array extraction in `preflight_safety.rs` independent of the live `ledgerful` binary.

---

## What Looks Solid

1. **Elimination of Captured Prompts in Safety:** Switching to leading-line GLOB matching permanently stops review prompts and session transcripts from hijacking the Bearings & Safety section.
2. **Dynamic Live Bearings Alignment:** Injecting live `ledgerful hotspots` paths brings preflight output into direct alignment with `safety sync --dry-run`.
3. **Fail-Open Design:** If Ledgerful is not installed, fails, or returns unexpected output, preflight gracefully falls back to vault pins or `SAFETY_EMPTY` without crashing.
4. **Hotspot Restraint:** Zero hunks in CLI `preflight.rs`, `project.rs`, `sync.rs`, or `doctor.rs`. Code is modularized into retrieval `preflight_safety.rs` and `session_chrome.rs`.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| Safety = review-track Objective dump | Absorbed into DoD (F1–F3 / AC3–AC4 / AC10) | Solved via leading-line GLOB + live hotspot injection |
| T274 F23 Safety SQL leftover | Absorbed (F1) | Replaced with `safety_marker_glob_sql` |
| T274 AC6 buried CONSTRAINT Safety-steal | Absorbed (AC3) | Buried mentions excluded from Safety |
| T250 F12 float formatting | Partial (F15) | Applied as `score={:.2}` on live hotspot lines |
| Live `safety sync` vault pinning | Declined (F21) | Read-only injection is sufficient DoD |
| Last-PR Cursor #194 | N/A (empty) | Scanned with 0 findings |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#194](https://github.com/Ryan-AI-Studios/AI-Brains/pull/194) (merged 2026-08-22, T278 `Session neighbor PREVIEW captions`).
- **Cursor Comments:** 0 comments (`[]` on PR #194).
- **Disposition:** N/A (no pending findings).

---

## Research / Tools Notes

- **SQLite Query Optimization:** SQLite `GLOB 'PREFIX*'` queries take advantage of indexes and restrict matches to string prefixes, unlike `LIKE '%...%'` which forces substring table scans.
- **CLI Empty State Standards:** clig.dev guidelines advise against omitting expected sections entirely when empty; displaying actionable next steps (`next: ai-brains safety sync --dry-run`) improves operator discoverability.
- **Dependencies:** `clap` (4.6.1), `serde_json` (1.0.150), `rusqlite` (0.39.0), `chrono` (0.4.44), `uuid` (1.23.1).
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,516 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search safety_sql`: Located at `crates/ai-brains-retrieval/src/preflight.rs:290`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
