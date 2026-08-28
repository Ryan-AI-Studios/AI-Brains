# T316 — `memory list` preview + forget nudge

- **Track ID:** T316-MemoryListPreview
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T316`.
- **Category:** UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `memory list` 6/**6**. Previews are raw first lines (`Let me verify the clap pin...`). Trailing forget nudge reads like an error.
- **Depends on:** T287 ✅ human prefer-fill ORDER (do **not** reopen JSON recency); T216 ✅ inventory; T285 `first_contentful_line`
- **Blocks / feeds:** Daily inventory skim.
- **Absorbs:** Audit preview + F36 stderr nudge
- **Not absorbed (DoD):** T287 ORDER; T299 forgotten-empty; T216 JSON keys; forget match-preview 100
- **Research date:** 2026-08-27. `memory.rs`: list preview max chars; F36 `eprintln!` forget next-step. Snapshot — re-verify at execute.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement **FEATURE** TX on go.
- **Isolation:** Do **not** implement until go. Do **not** grow `forget.rs` production (hotspot **#5**) except inherit-only preview helper. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Human preview is contentful.** After role/TAGS envelope, skip “Let me …” / tool-chrome first lines when a later line is better (full plan names the skip list). JSON preview contract stays T216 unless the plan proves a display-only field.
2. **Forget nudge is not an error.** Move to a `next:` stdout line (T267 style) or drop on default list (full plan). Must not look like a failure after a successful table.
3. **Keep T287 ORDER.** Human prefer-fill stays. JSON recency frozen.
4. **North star.** Capture independence: display only. No new events.

---

## 2. Live baseline (mint 2026-08-27)

| Signal | Observation |
|--------|-------------|
| `memory.rs` F36 | stderr: `Use ai-brains forget --memory-id <id> -f to forget...` |
| T287 | ORDER done; live GLOB 0 may still recency-fill (R1-1) — preview is this track |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- JSON `items[]` recency / keys T216.

---

## 6. Non-goals

Reopening list ORDER. Changing forget match. `--status forgotten` table rewrite (T299).

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| Audit memory list 6/6 | **Absorb** |
| T287 R1-1 live GLOB 0 | **Partial** — ORDER not this DoD; preview still is |
| last-PR `#229` | **N/A empty** |

---

## 12. Touch map (sketch)

`crates/ai-brains-cli/src/commands/memory.rs` `preview_line` + emit footer. Inherit-only if graph/forget already call it (T287 F6).
