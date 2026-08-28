# T319 — Governed handle vs vault memory ID namespace

- **Track ID:** T319-HandleVsMemoryId
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T319`.
- **Category:** UX / HONESTY / CONTRACTS (overlay only unless plan proves)
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `evidence show` / `source show` on a memory UUID → `Handle not found` / `NOT_FOUND`. Two UUID namespaces look identical.
- **Depends on:** T160/T203 governed show; T263 H1; T290 granted-empty lists; `UNKNOWN_HANDLE_PREVIEW` / `EXIT_NOT_FOUND`
- **Blocks / feeds:** Operators who paste `recall` ids into governed show.
- **Absorbs:** Audit UUID namespace confusion
- **Not absorbed (DoD):** H2 auto-resolve memory → evidence; fabricating evidence rows; DTO required-key growth without a contracts note; T290 list empty copy
- **Research date:** 2026-08-27. `governed_common.rs` `UNKNOWN_HANDLE_PREVIEW = "Handle not found."`; `EXIT_NOT_FOUND = 4`. Snapshot — re-verify at execute.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement **FEATURE** TX on go.
- **Isolation:** Do **not** implement until go. Do **not** grow hotspot `governed_common.rs` (#3) beyond a small formatter if callers can own the probe. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Wrong-namespace is named.** If the UUID exists as a vault `memory_id` but not as a governed handle, human + JSON `next_step` say so and point at `recall` / `memory list` (exact string in full plan). Still exit **4** / `NOT_FOUND` unless the plan proves a distinct code is required (CLI-EXIT-CODES).
2. **Unknown-unknown stays E1.** Truly missing ids keep the empty-state contract.
3. **Do not coerce.** Showing a memory as evidence is H2-adjacent — decline.
4. **North star.** Capture independence: CLI overlay + optional `QueryStore` EXISTS. No new events.

---

## 2. Live baseline (mint 2026-08-27)

| Signal | Observation |
|--------|-------------|
| `evidence show` / `source show` | Same UUID shape as `memory_id` |
| Expand | `query expand` also takes handle ids — same hole |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- Exit 4 / `NOT_FOUND` unless plan + docs update CLI-EXIT-CODES together.
- Arrays on list stay empty (T290).

---

## 6. Non-goals

H2. Alias table mapping memory→handle. Changing progressive search to FTS.

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| Audit handle vs memory | **Absorb** |
| T290 empty lists | **Not stolen** |
| last-PR `#229` | **N/A empty** |

---

## 12. Touch map (sketch)

`evidence.rs` / `source.rs` show miss path; optional `query expand`. Tests hermetic: memory EXISTS + handle miss.
