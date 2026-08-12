# T241 — Policy cold-start bootstrap

- **Track ID:** T241-PolicyColdStartBootstrap
- **Status:** 📋 **Placeholder** (plan-only until **go**)
- **Category:** FEATURE / UX / GOVERNED
- **Source:** Audit — briefing **4–5**, progressive **3**, evidence/source/review **3**, policy show **6/5**, policy check **5**; P0 first-run grants
- **Depends on:** T210 policy bootstrap exists; T221 deny exit; T226 soft-resolve
- **Absorbs:** Guided bootstrap from doctor/preflight when `grants: []`; richer denied briefing; policy show empty guidance; policy check discoverability
- **Not absorbed:** Full admin grant UI; progressive ranking quality (→ T243)

## 1. Objective

First-run (and this machine’s empty-grant vault) can unlock **ReadDecisions / ReadConclusions / ReadEvidence** without archaeology so governed surfaces are usable after one clear step.

## 2. Problem (live)

- `policy show` → `grants: []`
- `briefing project` → Denied authority, empty decisions
- `query progressive` → exit **3** POLICY_DENIED
- `evidence|source|review list` → exit **3** with bootstrap hint (good hint, still dead-end)

## 3. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | Doctor check or preflight section: **grants_empty** → next: `policy bootstrap --dry-run` then apply. |
| **F2** | Optional `preflight --install-grants` / non-interactive bootstrap with confirm (consent). |
| **F3** | `policy show` empty state: human one-liner + bootstrap example (not only `[]`). |
| **F4** | `policy check` without `--capability` → usage lists common capabilities. |
| **F5** | Briefing denied: keep packet; strengthen next-step (T227 residual). |
| **F6** | Capture independence: bootstrap never required for recall/preflight ungoverned path. |

## 4. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Empty grants discoverable from doctor and/or preflight summary |
| AC2 | After bootstrap (hermetic + live dogfood), progressive + briefing non-denied for discovery reads |
| AC3 | Deny path still exit **3** when denied (T221) |
| AC4 | Docs + CHANGELOG |

---

**Placeholder only. Say go to expand plan + implement.**
