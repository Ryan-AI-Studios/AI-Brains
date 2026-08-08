# T225 — Backup verify quiet + encrypted backup nudge

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Audit — `backup verify` exit 1 with INFO flood; all listed backups legacy plain / unreadable
- **Scores:** usefulness 7 · quality **6**
- **Category:** UX / OPS
- **Depends on:** T209 list honesty; T187 SQLCipher

## Objective

1. Quiet-by-default verify summary (counts + first N failures); `--verbose` full stream.  
2. Doctor / verify human path nudge: `ai-brains backup create` under live SQLCipher when newest backup is legacy or older than N days.

## Non-goals

Auto-delete legacy backups; force migrate vault.
