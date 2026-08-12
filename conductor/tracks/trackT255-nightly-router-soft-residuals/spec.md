# T255 — Nightly / router soft residuals (T229+)

- **Track ID:** T255-NightlyRouterSoftResiduals
- **Status:** 📋 **Placeholder** (plan-only until **go**)
- **Category:** OPS / POLISH
- **Source:** T229 soft F8–F12/F14 — doctor model ports; JSON status; embed sleep retune
- **Related:** T247 latency/101 (prefer T247 for 101/latency)

## 1. Objective

Optional polish batch for nightly/router ops without reopening T229 DoD.

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | Soft: doctor model endpoint matrix (:8081/:8083). |
| **F2** | Soft: `nightly --status --format json`. |
| **F3** | Soft: embed 50ms sleep retune (F14) only with evidence. |
| **F4** | Decline freely if not worth it — append deferred. |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Each item fixed or declined with one-line deferred |
| AC2 | No regression T229 status/probe |

---

**Placeholder only.**
