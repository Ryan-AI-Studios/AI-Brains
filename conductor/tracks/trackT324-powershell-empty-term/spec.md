# T324 — PowerShell empty TERM on `decision in-force`

- **Track ID:** T324-PowershellEmptyTerm
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T324`.
- **Category:** BUGFIX / UX / WINDOWS
- **Owner:** Grok
- **Source:** `conductor/deferred.md` T311 residual **R7** — PowerShell `""` drops empty TERM. Hermetic `.arg("")` + whitespace `fail_usage` already exist; live `ai-brains decision in-force ""` is not empty.
- **Depends on:** T311 ✅ empty-term usage exit 2 (hermetic)
- **Blocks / feeds:** Windows operators who follow docs using `""`.
- **Absorbs:** T311 R7
- **Not absorbed (DoD):** T314 progressive `--dry-run` Set vs SetTrue (different clap hole); clap 5
- **Research date:** 2026-08-27. PowerShell 5.1 / 7: unquoted `""` after a command can vanish from argv (native command argument passing). Microsoft: pass `--%` or `--term ''` / `--term \`""\``. Snapshot — re-verify at execute on this machine.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement **BUGFIX** TX on go.
- **Isolation:** Do **not** implement until go. Do **not** change T311 JSON keys. Prefer `--term` named flag + after_help PowerShell example over a parser hack. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Empty term is reachable from PowerShell.** Either a named `--term` (optional positional) or documented argv that survives PS quoting — full plan picks one. Missing/whitespace still usage exit **2**.
2. **Hermetic `.arg("")` stays green.** Do not break the T311 unit.
3. **Help shows the Windows invocation.** `after_help` example that actually works in PowerShell.
4. **North star.** Capture independence: clap/usage only.

---

## 2. Live baseline (mint 2026-08-27)

| Signal | Observation |
|--------|-------------|
| T311 | Empty positional → fail_usage; PS `""` may omit the argv slot entirely → clap missing-arg, not usage-empty |
| Shell | This repo: PowerShell (AGENTS.md) |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- T311 `value_parser` format list stays.

---

## 6. Non-goals

Fixing every positional across the CLI. Changing TERM matching semantics.

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| T311 R7 | **Absorb** |
| T314 flag unify | **Not stolen** |
| last-PR `#229` | **N/A empty** |

---

## 12. Touch map (sketch)

`main.rs` `DecisionCommands` + T311 in-force parse tests + after_help.
