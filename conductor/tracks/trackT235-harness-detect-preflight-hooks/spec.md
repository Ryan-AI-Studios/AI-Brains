# T235 — Harness detect + preflight hook install UX

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Research 2026-08-08 — seamless install; “detect harness and add hooks at preflight (or ask)”
- **Category:** FEATURE / UX
- **Depends on:** T234 (contract exists; install stubs may no-op until T236–T238)
- **Blocks / pairs with:** T236–T238 install backends
- **Related:** T214 preflight summary; T192 doctor; adapter capability report

## Objective

At session start (`preflight`, optionally `doctor`), AI-Brains:

1. **Detects** which harness(es) are present / likely active.  
2. **Reports** whether capture wiring is installed (hooks/plugins + last import signal if known).  
3. **Offers** to install (interactive) or installs with explicit consent flags — **message-only** hooks only.

Seamless default: detect + ask once; remember preference in user-global config (`%USERPROFILE%\.ai-brains\`).

## Frozen direction (draft)

| ID | Decision |
|----|----------|
| F1 | **Detect signals (read-only, best-effort):** PATH binaries (`grok`, `agy`/`antigravity`, `opencode`, `claude`, `codex`); home dirs (`~/.grok`, `~/.gemini/antigravity-cli`, `~/.local/share/opencode`, `~/.claude`, `~/.codex`); optional parent-process / env markers if reliable (`GROK_*`, etc.) — document false-positive risk |
| F2 | **Wiring status per harness:** `missing` \| `partial` \| `ok` \| `unknown` — check expected hook/plugin files and managed markers (AI-Brains-owned comment/id in hooks JSON) |
| F3 | **Preflight integration:** after normal summary (or under `--summary`), print short **Harness** section when any detected: name, wiring status, one-line next action |
| F4 | **Consent model:** default **ask** on TTY when `missing`/`partial` and not previously declined; non-TTY / CI: never prompt, only report; `--install-hooks` / `--yes` for non-interactive install; `--no-hook-prompt` suppress |
| F5 | **Remember:** user-global `harness_hooks.json` (or dotenv keys) — `auto_install`, per-harness `installed_at` / `declined_at` / `version` |
| F6 | **Install scope default:** **user-global** hooks (C7); project-local only with `--scope project` |
| F7 | **CLI surface:** `ai-brains harness status` (read-only); `ai-brains harness install [--harness agy\|grok\|opencode\|all] [--yes] [--dry-run]`; optional `harness uninstall` |
| F8 | **Doctor soft check:** list harness detect + wiring; do not fail doctor solely for missing hooks |
| F9 | **Install backends** may be stubs until T236–T238 land — dry-run must show target paths; real write gated by track readiness flag or backend existence |
| F10 | Never write secrets into hook files; hooks shell out to `ai-brains` with paths/payloads only |

## Preflight UX sketch

```
--- AI-Brains Preflight Summary ---
...
Harness: grok=ok  agy=missing  opencode=partial
  → Install Antigravity 2 capture hooks? [Y/n]
```

Or non-interactive:

```
Harness: agy=missing (run: ai-brains harness install --harness agy)
```

## Non-goals

Parsing transcripts (T234/T236+); nightly import (T239); auto-pin decisions.

## Acceptance sketch

| AC | Sketch |
|----|--------|
| AC1 | Detect fixtures: each harness home layout → correct presence flags |
| AC2 | Preflight TTY with missing wiring + no prior decline → prompt once |
| AC3 | Decline persisted → no re-prompt until `harness install` or reset |
| AC4 | `--dry-run` install prints paths, writes nothing |
| AC5 | Non-TTY no prompt; exit 0 |
| AC6 | `harness status --format json` machine-readable |
| AC7 | CAPABILITIES / OPERATIONS / help_ia Daily or Setup group mention |

## Risks

- False “active harness” from leftover home dirs — label as **installed on machine**, not **this session**.  
- Windows UAC / file locks on hook config — fail soft with path + manual instructions.  
- Competing tools (claude-mem) editing same AGY settings — use namespaced hook entry ids; never wipe foreign hooks.
