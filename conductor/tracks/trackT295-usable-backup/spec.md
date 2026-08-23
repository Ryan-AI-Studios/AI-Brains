# T295 — Operator vault must have ≥1 usable encrypted backup

- **Track ID:** T295-UsableBackup
- **Status:** **Placeholder** (Pending until `/plan-track 295`)
- **Category:** OPS / RECOVERY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — backup list/verify **8/8** honest; **not working:** 0 usable; doctor `backup_recent` warn
- **Depends on:** T277 ✅ fail-closed create (hermetic); live `--no-prune` still skipped
- **F0:** Plan-only until **go**.

## Problem (live)

`backup verify` = `0 OK, 22 FAIL` + nudge `backup create`. T277 DoD was hermetic. This track is the **operator file**.

## How to ≥8

Product already creates usable snapshots. This track: **owner-confirm** live `backup create --no-prune` (keep-10 would prune 12 residuals — do not). After that, `backup list` shows ≥1 Readable/current-key; doctor `backup_recent` **ok**. No transcode of T244 `.bak`.

## Manual DoD (on go)

```powershell
ai-brains backup create --no-prune   # ONLY if owner confirmed at go
ai-brains backup list
ai-brains backup verify
ai-brains doctor --summary
```

Pass: `backup list` has ≥1 row **not** `(unreadable key)`/`(legacy plain)`/`(no core tables)` classified usable. `backup verify` **OK ≥ 1** (may still exit 1 if residuals FAIL — document). `doctor --summary` does **not** warn `backup_recent` **or** warn is gone. Hermetic T277 still green without live create. If owner does **not** confirm live create: hermetic-only + written skip (same as T277) — **not** Completed until live file **or** owner skip recorded.

## Isolation

**`--no-prune`** on live. No restore while daemon up (T188). No `retention apply`.
