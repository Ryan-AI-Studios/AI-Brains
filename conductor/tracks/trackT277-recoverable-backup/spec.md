# T277 — At least one usable encrypted backup under the current key

- **Track ID:** T277-RecoverableBackup
- **Status:** **Placeholder** (Pending until `/plan-track 277`)
- **Category:** OPS / UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `backup list` **8/8** honest FAIL fleet; `backup verify` **8/9** 0 OK / 22 FAIL; doctor `backup_recent` warn
- **Depends on:** T209 ✅ list honesty; T225 ✅ verify quiet; T244 ✅ usable class
- **F0:** Plan-only until **go**.

## Problem (live)

Honesty is done; **recoverability is not**. Every listed backup is legacy plain / incomplete / wrong key. Doctor remediator is `backup create`. No encrypted OK file exists under the current key.

## How to ≥8 (ideally 10)

`backup list` shows ≥1 **Readable** (usable) row after a create; `backup verify` reports ≥1 OK; doctor `backup_recent` is not “no usable encrypted backup.” Effectiveness of list/verify is already ≥8 — this track is **fleet state**, not restyle.

## Manual DoD (on go)

**Hermetic vault** (do not fill the live backup dir as the only proof):

```powershell
ai-brains --no-project-context --vault-path <tmp> backup create
ai-brains --no-project-context --vault-path <tmp> backup list
ai-brains --no-project-context --vault-path <tmp> backup verify
```

Pass: list has a usable/Readable file; verify ≥1 OK; exit 0 on verify when that file is OK. Live vault `backup create` only if the owner confirms (writes next to real backups).

## Isolation

No restore while daemon Running (T188). No CE. No key print.
