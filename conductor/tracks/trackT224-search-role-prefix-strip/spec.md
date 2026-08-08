# T224 — Search/display role-prefix strip

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Audit — `memory list` strips USER/ASSISTANT/SYSTEM; recall/sync/forget dry-run still dump `ASSISTANT:`
- **Category:** UX
- **Depends on:** T216 `preview_line` SOOT

## Objective

Share **display-only** role-prefix strip across recall pretty, sync query pretty, forget match dry-run previews. Stored content unchanged.

## Draft decisions

- Extract pure helper (core or CLI) reused by T216 + search paths
- JSON raw content may keep prefix (document) **or** strip preview field only
- Unit: multibyte-safe; case-sensitive token match like T216 F9
