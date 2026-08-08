# T229 — Nightly + local router ops (env / health / schedule)

- **Status:** 📋 Placeholder product + **ops partial done 2026-08-05**
- **Source:** Audit — nightly **not scheduled**; model URLs only in project `.env`; user wants runs against `c:\llm\router.bat` (:8081 LLM, :8083 embed)
- **Category:** OPS / FEATURE
- **Depends on:** T132/T143/T145 schedule wrappers; T85 port status

## Ops done (this machine, agent session)

| Item | State |
|------|--------|
| User-global dotenv model URLs | **Done** — `%USERPROFILE%\.ai-brains\.env` points at `http://127.0.0.1:8081` / `:8083` + models |
| Wrapper with port probes + log | **Done** — `%USERPROFILE%\.ai-brains\nightly-run.cmd` |
| Elevated register script | **Done** — `%USERPROFILE%\.ai-brains\register-nightly-tasks.ps1` |
| Task Scheduler register | **Blocked** — Access denied from agent (need user elevated run) |
| Router ports live | **Verified** — both 8081/8083 HTTP 200 when router up |

**User action required (elevated PowerShell once):**

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File "$env:USERPROFILE\.ai-brains\register-nightly-tasks.ps1"
# or:
ai-brains nightly --schedule --start-time "03:00"
# Ensure c:\llm\router.bat is running before 03:00 (or register AI-Brains-Router ONLOGON via same script)
```

## Product DoD (on go)

| F1 | `nightly --status` shows effective MODEL_URL / EMBEDDING_URL (redacted) + last probe |
| F2 | Optional preflight: if endpoints down, warn non-fatal (summarization may skip) |
| F3 | Document router.bat dependency in OPERATIONS |
| F4 | Schedule path always gap-fills global dotenv model keys (not only project cwd) |

## Non-goals

Bundle router into AI-Brains; start GPU processes from Rust without user policy.

## Split (2026-08-06)

**Multi-root Ledgerful bridge / path aliases / System32 cwd fix** → **[T233](../trackT233-path-alias-multiroot-nightly/spec.md)** (Option B).  
**Upstream:** coordinated **0163-SymbolsInventory** (Ledgerful `symbols` CLI) before T233.  
T229 remains router URLs, schedule wrapper, status probes for `:8081`/`:8083`.
