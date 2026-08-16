# T263 — Governed surface: honesty + vault-pin authority

- **Track ID:** T263-GovernedVaultPinAuthority
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** FEATURE / UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — governed empty; briefing **4/6** + **3/6**; `query progressive` **3/5**; expand **6/6**; trace **5/4**; evidence/source/review **3/5**; opportunity “connect governed to vault pins or stop advertising”
- **Depends on:** T152/T160/T203/T210/T221/T227/T241 (surface exists and is granted)
- **Absorbs:** 3 discovery grants, 0 evidence, 0 conclusions, 0 approved decisions; `briefing project` “No current authority”; progressive `results: []` + next-step `recall`; personal denied on hardcoded-looking `Personal:a1b2a1b2-…`; expand missing → empty preview exit 0; trace missing → literal `null`
- **Not absorbed:** Policy bootstrap (T241 closed); Scope default (T258); format maze (T266)

---

## 1. Objective

The governed CLI must not look like the place to “ask what we decided” unless it can answer. Pick **one** at plan time (both allowed as phases):

- **H1 Honesty:** Daily help / CAPABILITIES / empty packets name `recall` / `preflight` as the decision path. Progressive/briefing stay for *Approved/Active governed* rows only. Tighten expand/trace empty (no bare `null`).
- **H2 Promotion (opt-in):** A dry-run classifier that can propose vault `DECISION:` / `CONSTRAINT:` pins as governed drafts for review — never silent Approved. Capture-independence: classifier is SQL + rules, not an LLM, unless the user later enables a flagged path.

Default recommendation: **H1 in this track; H2 only if plan-time research shows a lossless pin→proposal mapping without a new event soup.**

## 2. Problem (live 2026-08-16)

Policy `show` listed ReadEvidence / ReadConclusions / ReadDecisions on `Repository:441837f6`. Doctor `policy_grants` ok (3 of 3). Then:

| Command | Result |
|---------|--------|
| `briefing project --format human` | Decisions _None_; Conclusions _None_; `empty_authority`; Ledgerful degraded |
| `briefing personal` | **Denied:** Personal scope read denied without grant |
| `query progressive "why was graph backend replaced?"` | `results: []` |
| `evidence list` / `search nightly` / `source list` / `review list` | `items: []` |
| `query expand` missing UUID | `kind: Unknown`, `preview: ""`, exit 0 |
| `query trace` missing | `null` |

Meanwhile `memory list` on the same Scope showed dozens of `DECISION: T255…` / `CONSTRAINT:` pins from the last two days. Two products, one binary. T241 unblocked *grants*. It did not unblocked *authority*.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. Choose H1 vs H1+H2 in plan.md before code. |
| **F1** | Empty progressive/briefing already point at `recall` — make top-level help and skill match (do not list progressive as “what did we decide”). |
| **F2** | `query trace` missing: JSON `null` is a contract — if frozen, document; if not, emit `{ "trace": null }` / empty object. Decide at plan (T180 honesty). |
| **F3** | Personal briefing denied: next-action is `policy bootstrap --scope Personal:…` only if Personal is a supported product; otherwise say Personal is unused. |
| **F4** | H2 if taken: `--dry-run` proposals only; no auto-approve; no live vault migrate (T170 stop-before). |
| **F5** | Do not enable `AI_BRAINS_GOVERNED_BRIEFING` on production preflight without explicit user approval (T170 CONSTRAINT still live). |

## 4. Verification sketch

- Help/CAPABILITIES decision table matches H1.
- Trace/expand empty documented + hermetic.
- If H2: dry-run fixture pin → one proposal row; `--confirm` refused without `--yes`.
