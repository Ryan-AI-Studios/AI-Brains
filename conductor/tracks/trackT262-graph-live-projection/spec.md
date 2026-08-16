# T262 — Graph live projection + neighbors / hierarchy

- **Track ID:** T262-GraphLiveProjection
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** FEATURE / BUGFIX
- **Owner:** —
- **Source:** Audit 2026-08-16 — graph unused; `graph neighbors` **4/5**; `graph hierarchy` **3/4**; opportunity “rebuild or fix live projection”
- **Depends on:** T69 live hook; T213 density doctor; T232 remediations; T246 human CLI
- **Absorbs:** 21,100–21,118 nodes / 918–945 edges (E/N ≈ 0.044 vs floor 0.50); 35,300 pinned vs 19,751 memory nodes; 4-hour-old pin `46d88c87` **no graph node**; hierarchy `synthesized_from: []`; neighbors of a T255 decision only incoming `RECALLS` from this audit
- **Not absorbed:** Density threshold retune; graph default-on Cargo feature; Cozo INFO (T208 closed)

---

## 1. Objective

A memory that exists in the vault (especially a just-pinned DECISION) must be queryable via `graph neighbors` / `graph hierarchy` without a manual `graph rebuild`. If live projection cannot catch up, doctor/neighbors must say **why** (lag vs never-projected vs unknown id), not only `next: graph update`.

## 2. Problem (live 2026-08-16)

`graph update`: sparse, E/N 0.044, remediation `graph rebuild`. Doctor agrees.

`graph neighbors 46d88c87-…` (pinned ~4h, `TAGS: daemon, wsl`):

```
No graph node for 46d88c87-…
next: ai-brains graph update
```

`graph update` is a **health check**, not a rebuild. The next-action is wrong (T232 already taught doctor to say `rebuild` when graph-on). Neighbors still points at `update`.

`graph hierarchy` on the same id: empty JSON `synthesized_from: []` (no human “no synthesis chain” line).

T255 decision `aa0a75da` *did* have neighbors — only `RECALLS` from the audit sessions. No `IN_SESSION` / `SYNTHESIZED_FROM` / pin lineage.

T69 claimed incremental projection on every append. Live data says many pins never became nodes, and new pins can miss entirely.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. Diagnose before prescribing a full rebuild as DoD. |
| **F1** | Neighbors next-action: `graph rebuild` when density/capability says so; never `graph update` as the remediator for a missing node. |
| **F2** | Hierarchy empty-state human: “no SYNTHESIZED_FROM chain” (JSON keys frozen). |
| **F3** | Find why T69 hook skipped the 4h pin (event kind? hook off? feature? error swallowed?). Fix or document fail-open honestly. |
| **F4** | Do not make `graph rebuild` the nightly default without a plan-time cost estimate (21k nodes). |
| **F5** | Capture independence: graph failures stay non-fatal to event append. |

## 4. Verification sketch

- Hermetic: pin → neighbors finds a node without rebuild (or documented fail with rebuild next).
- Neighbors missing-node copy names `rebuild` when graph-on.
- Hierarchy empty pretty.
- Doctor density still warn until projection actually adds edges (do not fake live).
