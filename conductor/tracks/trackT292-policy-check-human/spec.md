# T292 — `policy check` needs a human allowed/denied line

- **Track ID:** T292-PolicyCheckHuman
- **Status:** **Placeholder** (Pending until `/plan-track 292`)
- **Category:** UX
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `policy check --capability ReadEvidence` **7/8** JSON-only
- **Depends on:** T226 ✅ soft-resolve; T241 ✅ catalog; T266 Family D
- **F0:** Plan-only until **go**.

## Problem (live)

TTY/`auto` still dumps JSON `{allowed:true,...}`. Operators cannot scan allowed vs denied. U=7.

## How to ≥8

`--format auto`: TTY human `allowed: true` / `denied: <cap> — next: …`; pipe JSON frozen keys. `--format json` unchanged. `--format human` forces the line.

## Manual DoD (on go)

```powershell
ai-brains policy check --capability ReadEvidence --format human
ai-brains policy check --capability ReadEvidence --format json
```

Pass: human stdout contains `allowed` and `ReadEvidence` and is **not** a JSON object (no leading `{`). JSON still has `allowed` boolean + `capability`. Hermetic deny cap shows `denied` + bootstrap/recall next, not empty. Exit **0** on allow; deny exit stays current (document).

## Isolation

No new capabilities. JSON keys frozen. No clap 5.
