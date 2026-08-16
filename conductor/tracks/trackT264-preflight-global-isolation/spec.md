# T264 — Preflight global isolation

- **Track ID:** T264-PreflightGlobalIsolation
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — `preflight --global --pretty --compact` **5/4**; `--global --summary` **7/6**
- **Depends on:** T214 global rollup; T219/T250 pretty
- **Absorbs:** Global pretty mixing coordinator 0022/0023 and a hip-hierarchy review into “Repository Bearings & Safety”; global summary “In context decisions: 20” without saying *which* projects
- **Not absorbed:** Project-scoped pretty (scored well); JSON envelope (T265)

---

## 1. Objective

`--global` remains a **rollup**, not a blender. Safety/session/decision lines must be labeled by project (or omitted in favor of counts). Other repos’ DECISIONs must not appear as *this* repo’s bearings.

## 2. Problem (live 2026-08-16)

`preflight --global --pretty --compact -m 400`:

```
--- Repository Bearings & Safety ---
CONSTRAINT: T170 stop-before — …
# Technical Review: Track 0092 — Hip Hierarchy Polish
DECISION: 0022 project phase timeouts shipped. …   (coordinator)
DECISION: 0023 AlwaysOnServe shipped. …
```

T170 is this vault’s constraint. 0092 / 0022 / 0023 are not AI-Brains. An agent starting with `--global --pretty` would treat coordinator decisions as local law.

`--global --summary` is statistically honest (53 projects, 35,300 pins) but “In context decisions: 20 / constraints: 0” hides the mix.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | Global pretty: every Safety/Session/Memory line prefixed with project label or id. |
| **F2** | Prefer per-project caps over a single blended top-N. |
| **F3** | Summary: keep vault totals; add “in-context spans N projects” or suppress in-context markers when `--global`. |
| **F4** | Project-scoped pretty (no `--global`) unchanged. |

## 4. Verification sketch

- Hermetic two-project vault: global pretty shows both labels; no unlabeled foreign DECISION.
- Summary JSON/human notes project span.
