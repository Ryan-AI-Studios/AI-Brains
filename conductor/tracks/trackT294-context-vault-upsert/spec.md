# T294 — `context` already-initialized must upsert the `.env` project into the vault

- **Track ID:** T294-ContextVaultUpsert
- **Status:** **Placeholder** (Pending until `/plan-track 294`)
- **Category:** UX / IDENTITY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 leftover 5 roots dest-missing; `context` early-return skips vault ensure
- **Depends on:** T259 ✅ rebind dest must exist; T240 F2 freeze (do **not** rewrite `.env`)
- **F0:** Plan-only until **go**.

## Problem (live)

crawlx/degoo/kinledger have `.env` `PROJECT_ID`s **not** in this vault. `ai-brains context` prints “already initialized” and returns **before** “Ensure project/session exists in the vault.” `rebind-path --to <env-id>` fails dest-missing. Leftover split cannot finish without minting new IDs.

## How to ≥8

If cwd `.env` has `AI_BRAINS_PROJECT_ID` + session, **idempotent** ensure those IDs exist in the open vault (events if missing), **without** rewriting `.env` (T240 F2). Then `rebind-path --to` that id works. gimp/homebrew-tap (no `.env`) still need `context` init (writes `.env`) — document, not silent mint.

## Manual DoD (on go)

```powershell
# hermetic: vault A; .env PROJECT_ID not in vault; context (no --new-project)
ai-brains context
ai-brains project list --format json
ai-brains project rebind-path <path> --to <env-project-id> --format human
```

Pass: hermetic `context` exit **0**, **`.env` bytes unchanged**, `project list` JSON contains the env project_id. Print-only rebind dest exists (not “dest missing”). Live leftover 5: **classify-only** unless owner confirms `--write --yes` per path. Exit **0**.

## Isolation

**T240 F2.** No `--new-project`. No leftover UUID hardcoded. Memories stay (T259 F5).
