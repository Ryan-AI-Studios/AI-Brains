# T232 — Graph density remediation path

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Audit/doctor — `graph_density` sparse remediates `graph rebuild` but binary graph-off → FEATURE_UNAVAILABLE
- **Category:** UX / OPS
- **Depends on:** T213 density; T222 install path

## Objective

Doctor remediation text must match **effective binary capabilities**:

- Graph-off: reinstall `--features graph` (not rebuild)
- Graph-on sparse: rebuild or accept sparse
- Optional: `doctor --json` field `graph_feature: available|unavailable`

## Non-goals

Auto rebuild; change density floors (T213).
