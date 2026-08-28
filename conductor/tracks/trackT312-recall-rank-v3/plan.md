# T312 Plan — Recall rank v3 (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full F-list / ACs / red names on `/plan-track T312`.
**Category:** FEATURE
**Ledger (planning):** series DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit ranking + T285 live dumps | **DoD** after full plan |
| T218 floors / `candidate_depth` / H2 | **Declined** spec §6/§9 |

---

## Phase 0 (on `/plan-track T312`, not go)

- [ ] Re-read `ranking.rs` `rerank_hits`, `session_chrome.rs` detector, `lexical.rs` two-pass
- [ ] Re-dogfood `recall "graph backend"` and `recall "GPU driver fix"` with `--format pretty --no-bridge`
- [ ] Confirm lock clap **4.6.1**, rusqlite **0.40.2**
- [ ] Write F-list, ACs, failing test names (`function__condition__expected`)

## DoD (after full plan + go)

Live topical query: leading hit is not a `# Review of Track` / `## Objective` dump when an in-scope authority pin MATCH-hits. Hermetic needle still SoT.

## Isolation

No `cargo install`. No live vault pins as implement SoT. Never `git push origin main`.
