# T223 — Quiet env override warnings

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Audit — almost every command prints dual “local .env overrides inherited shell value” lines
- **Category:** UX
- **Depends on:** T113 / T139 / T205 dotenv layers

## Objective

Reduce stderr spam without hiding real conflicts.

## Draft decisions

- Collapse to **one line** listing both keys, or only warn when values **differ** and both set
- `--quiet` / `AI_BRAINS_QUIET_ENV_WARN=1` suppress
- Keep warn when git/env project mismatch (T206) separate

## Non-goals

Change precedence rules (only presentation).
