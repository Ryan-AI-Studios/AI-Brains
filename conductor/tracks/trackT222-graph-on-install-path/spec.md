# T222 — Graph-on install path

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Audit — PATH `ai-brains` graph-off; `graph update` FEATURE_UNAVAILABLE (use **3**); docs push graph
- **Category:** INFRA / DOCS / RELEASE
- **Depends on:** T200 graph install honesty

## Objective

Operators who follow INSTALL get **graph-capable** CLI by default *or* an unmistakable one-command upgrade; doctor remediations that say `graph rebuild` must not dead-end.

## Draft options (pick on go)

| A | Flip cargo default feature `graph` on for `ai-brains-cli` (product decision; CI cost) |
| B | Keep default-off; ship `scripts/Install-AIBrains.ps1` / OPERATIONS that **always** `--features graph` |
| C | `ai-brains doctor` soft check `graph_feature` with reinstall line when tables exist but binary graph-off |

## Non-goals

Force Cozo/INFO noise regression (T208); auto rebuild.
