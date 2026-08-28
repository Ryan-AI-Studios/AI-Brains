# T315 — Preflight summary: governed-empty next-step + word-count label

- **Track ID:** T315-PreflightGovernedEmpty
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T315`.
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `preflight --summary` 8/**7**; 0/0/0 decisions/constraints next to 4510 pins; `Total Word Count` opaque. Opportunity (d).
- **Depends on:** T286 ✅ Index/summary pin titles; T220 ✅ summary envelope; T265 ✅ json sections; T263 H1 honesty; T288 ✅ briefing vault-pin stanza (do **not** steal)
- **Blocks / feeds:** Daily at-a-glance. Does **not** populate governed stores.
- **Absorbs:** Audit 0/0/0 + word-count meaning
- **Not absorbed (DoD):** H2 pin→Approved; T286 Index MATCH; growing `preflight.rs` beyond summary renderer (hotspot **#9**); clap 5
- **Research date:** 2026-08-27. `preflight.rs` `render` lines `In context decisions` / `Total Word Count: {word_count}` (budget-window `context.word_count`, not summary size — already in a struct comment). Snapshot — re-verify at execute.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement **FEATURE** TX on go.
- **Isolation:** Do **not** implement until go. Do **not** `cargo install`. Do **not** `policy bootstrap` extra grants. Summary path only; do not rewrite `--pretty` Session. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **0/0/0 is actionable.** When in-context decisions/constraints (and governed emptiness) are zero, summary prints a copy-paste next-step (`decision propose` / `recall "what did we decide"` — full plan picks the exact string) instead of a bare count.
2. **Word count is named.** Label distinguishes budget-window context words from “size of this summary.”
3. **Counts stay honest.** Do not invent in-context decisions from vault pins (H2). Dual-model: pins via recall; Approved via propose.
4. **North star.** Capture independence: CLI overlay. No new events.

---

## 2. Live baseline (mint 2026-08-27)

| Signal | Observation |
|--------|-------------|
| `preflight --summary` | Pinned 4510; decisions 0; constraints 0; hotspots 0; `Total Word Count: 781` |
| Code | `preflight.rs` ~794–796; comment already says budget-window word count |
| Hotspot | `preflight.rs` **#9** — keep the delta small |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- T265 JSON keys for `--format json` full preflight stay unless the plan adds **optional** summary fields (no required-key DTO growth without a contracts note).

---

## 6. Non-goals

H2. Nightly auto-propose. Changing Index MATCH (T286). `--pretty` chrome (T286 residual R1-1).

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| Audit preflight Q=7 | **Absorb** |
| T286 live Index `## Objective` | **Not this DoD** |
| last-PR `#229` | **N/A empty** |

---

## 12. Touch map (sketch)

`crates/ai-brains-cli/src/commands/preflight.rs` summary renderer + existing summary string tests (~1474).
