---
name: ai-brains
description: >
  Persistent memory vault for project decisions, constraints, and session
  capture. Use when the user asks what we decided, mentions past sessions,
  starts a repo cold, says remember this / check the vault, or signals
  frustration ('I told you last time'). Skip generic library questions and
  one-line formatting.
---

# AI-Brains Memory Protocol

Local-first vault. Prefer **preflight + recall** over re-deriving architecture.
Never print `AI_BRAINS_KEY` / `AI_BRAINS_VAULT_KEY`. Never pin chain-of-thought,
tool logs, or model reasoning.

## When NOT to use

- Generic “how does crate X work?”
- Trivial formatting already answered in this chat

## Availability

1. `ai-brains --version` — missing → tell the user to install; fall back to README.
2. Vault commands need `AI_BRAINS_VAULT_PATH` + `AI_BRAINS_KEY` (usually `%USERPROFILE%\.ai-brains\.env`, quoted). CLI has **no** silent home vault. Wrong key → `Vault locked`.

## Session start (cold repo)

Run **this first** — it is the briefing:

```powershell
ai-brains preflight --summary --bind
```

Expect pins, in-context DECISION/CONSTRAINT/HOTSPOT counts, harness wiring, and a `next:` line. `--bind` registers an unowned git toplevel (same helper as `context`). Default `--summary` without `--bind` stays query-only. Use `--pretty --compact` only if you need the Index. **~several seconds is normal.**

Then, only if Scope looks wrong (`(no alias)`, leftover UUID, git slug ≠ label):

```powershell
ai-brains project whoami
ai-brains context --show
```

- `preflight` = vault content for **this** `AI_BRAINS_PROJECT_ID`.
- `context --show` = **dotenv dump** (IDs, redacted keys, leftover shell `PROJECT_ID`). It does **not** open the vault. Empty `--show` ≠ empty vault.
- `project detect` / `whoami` = identity repair. Remediations name `set-alias` and `register-path`. `context` (not `--show`) may auto-bind the git toplevel and a unique slug alias.
- `briefing project` / `decision in-force` need `policy bootstrap --dry-run` then `policy bootstrap`. **POLICY_DENIED is a grant wall, not an empty vault** — use recall.

## Search (what did we decide?)

```powershell
ai-brains recall "<topic>" --limit 5 --format pretty
ai-brains search "<topic>" --limit 5 --format pretty
ai-brains sync query "<topic>" --quiet
```

`search` is an alias of vault recall. If FTS returns nothing, **do not conclude the vault is empty** — re-read `preflight --summary` / `--pretty --compact` Index, or retry with concrete terms (`graft`, `SQLCipher`, a track number). `--semantic` needs the embed server; a threshold miss falls back to lexical (extra RTT, often the same list). `--global` only when the user wants other projects.

## Record (mutating — ask unless the owner already said pin)

`pin` needs `AI_BRAINS_SESSION_ID` — `ai-brains context` if unset (`--show` does not write `.env`).

```powershell
ai-brains pin "DECISION: <what + why>"
ai-brains pin "CONSTRAINT: <rule>"
ai-brains pin "INVARIANT: <must-hold>"
```

First contentful line after optional `TAGS:` must be one of those prefixes (or T336 skips as `Other`). `--tx-id` / `LEDGERFUL_TX_ID` optional. Dense knowledge only.

## Forget / coverage / nightly

- `forget --list-forgotten` is read-only; `--memory-id` / `--match` mutate.
- `capture coverage --days 7` is **machine-wide** until T348 — do not treat grok-disk vs this-project vault as “capture broken.”
- `nightly --status --quick` is the operator card. **Do not** run live `nightly` without `--status`.
- `safety sync --dry-run` before pin. Vendored `deps_src/` + `score=0.00` rows are low-signal (T347).
- `doctor --summary` is the 4-line health glance. Full `doctor` is ~15 checks, not a dump.

## Identity (do not conflate)

| Command | Role |
|---------|------|
| `context` | Write/ensure `.env` IDs + vault project/session rows |
| `context --show` | Print `.env` IDs only |
| `project set-alias` | Human **label** |
| `project register-path` | Filesystem **root** (nightly Phase 2, Cursor slug bind) |
| `project adopt-path` | Print-only Scope fix; `--write-env --yes` to rewrite **only** `PROJECT_ID` |

Path strings in `set-alias` do **not** register a path. One path → one project (conflict exit 1).

## Key placement

| Item | Where |
|------|--------|
| KEY + vault path | `%USERPROFILE%\.ai-brains\.env` (gap-fill; does not override shell) |
| Project/session IDs | cwd `.env` via `ai-brains context` — **no secrets** in repo |
| Live vault on this machine | Often `C:/dev/ai-brains/vault.db` via global dotenv, not the source tree |

## Docs

`Docs/INSTALL.md`, `Docs/OPERATIONS.md`, `Docs/COMPATIBILITY.md` (F8), `Docs/CLI-EXIT-CODES.md`.
