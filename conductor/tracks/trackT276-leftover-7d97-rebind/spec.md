# T276 — Leftover `7d97a456` must stop owning the global rollup

- **Track ID:** T276-Leftover7d97Rebind
- **Status:** **Placeholder** (Pending until `/plan-track 276`)
- **Category:** FEATURE / UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `recall --global` **7/3**; `project list-paths` **8/7**; leftover ~18036 pins; many `C:\dev\*` path aliases
- **Depends on:** T259 ✅ leftover split tools; T258 ✅ path owner; T240 F2 freeze (no silent `.env` switch)
- **F0:** Plan-only until **go**.

## Problem (live)

T259 shipped rebind tools; the leftover project **still** holds ~18k memories and many `C:\dev\crawlx` / `dedupe` / `ledgerful-*` path aliases. `--global` recall returned leftover encoding garbage (`â€™`) as a top hit. `list-paths` makes the ownership obvious; nothing has moved the data.

## How to ≥8 (ideally 10)

Leftover is not the global default brain. Path-aliased repos that have their own project keep their pins. `--global` top hits for an AI-Brains needle are this project (or labeled leftover). Operator remediator is explicit (`rebind-path` / unregister) — **never** T240 F2 silent Scope write.

## Manual DoD (on go)

Print-only unless the owner confirms a write:

```powershell
ai-brains project whoami
ai-brains project list-paths --format human
ai-brains recall "T270 memory_legacy" --global --limit 5 --format pretty --no-bridge
```

Pass: whoami `mismatch: false` for cwd AI-Brains; list-paths still **documents** leftover until a confirmed rebind; after **confirmed** write (or hermetic fixture of leftover+owner), global recall of a unique owner pin is not leftover-first and has no `â€™` mojibake in the top hit. Hermetic is enough if live rebind is Stop-Before.

## Isolation

No silent `.env` (T240 F2). List **sort** of `project list` → **T283**. Ranking of session dumps → **T274**.
