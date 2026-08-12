# T249 — Scope / daemon / doctor presentation polish

- **Track ID:** T249-ScopeDaemonDoctorPresentation
- **Status:** 📋 **Placeholder** (plan-only until **go**)
- **Category:** UX
- **Source:** Audit — `scope resolve` **Q7** always JSON; `daemon status` **Q7**; no `doctor --summary` (exit 2)

## 1. Objective

1. `scope resolve` human pretty on TTY (JSON via `--format json`).
2. `daemon status` slightly richer (uptime/pid if available; still honest when stopped).
3. Optional `doctor --summary` one-block or map to existing compact path.

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | Scope: pretty default TTY; keep JSON machine. |
| **F2** | Daemon: Status + backends + schedule hint one screen. |
| **F3** | Doctor summary: either real flag or help “use doctor” (no lying flag). |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Scope pretty + json |
| AC2 | Daemon status scannable |
| AC3 | Doctor summary disposition (flag or documented absence) |

---

**Placeholder only.**
