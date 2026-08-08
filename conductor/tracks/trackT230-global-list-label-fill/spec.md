# T230 — Global inventory label fill

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Audit — `memory list --summary --global` many blank `label` cells; project list `(no alias)`
- **Category:** UX
- **Depends on:** T212 display_label; T216 F8

## Objective

Under global tables, never show empty label: fallback `(no alias)` or short uuid prefix (reuse `display_label`). Optional stderr: count of unaliased with set-alias hint (T212 footer pattern).

## Non-goals

Auto-create aliases from git slug.
