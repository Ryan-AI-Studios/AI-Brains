# T226 — Policy show/check soft-resolve scope

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Audit — `policy show` / `policy check` exit 2 clap-required `--scope` while source/evidence soft-resolve
- **Scores:** usefulness **5** · quality **6**
- **Category:** UX / CONSISTENCY
- **Depends on:** T203 soft-resolve; T210 bootstrap

## Objective

When authoritative project scope exists (`AI_BRAINS_PROJECT_ID` / context), `policy show` and `policy check` soft-fill scope; else fail_usage exit 2 with same template as discovery lists.

## Non-goals

Change grant evaluation semantics.
