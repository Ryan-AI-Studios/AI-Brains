---
name: ai-brains
description: "Persistent memory and project context vault. Use this skill whenever the user asks 'what did we decide', mentions past sessions, or when starting work on a repo cold. Trigger when you hear 'remember this', 'don't forget', 'check the vault', or 'what did we decide about'. ALSO trigger on frustration signals like 'I told you last time' or 'we already tried that'. Use even if memory isn't explicitly mentioned if the task involves project history. DO NOT use for generic coding questions, library documentation, or formatting help."
---

# AI-Brains Memory Protocol

Long-term memory vault for project decisions, constraints, and session capture. Prefer vault + preflight over re-deriving architecture from scratch.

## When NOT to use
- Generic “how do I use X in Rust?” knowledge
- Trivial one-line formatting fixes
- Answers already in the current conversation

## Availability
1. `ai-brains --version` — if missing, tell the user to install; fall back to README/Cargo.toml.
2. Vault ops need a **SQLCipher product key** after T187 page encryption (see Key & path below).

## Key & vault path (required for vault commands)

| Item | Where |
|------|--------|
| **Product key** | `AI_BRAINS_KEY` = `x'<64 hex chars>'` (32 random bytes). Never commit. |
| **Daemon key** | `AI_BRAINS_VAULT_KEY` (service / `daemon.env`; same product form). |
| **User-global dotenv** | `%USERPROFILE%\.ai-brains\.env` — CLI merges this for **gaps** (does not override shell or project `.env`), including with `--no-project-context`. Preferred place for `AI_BRAINS_VAULT_PATH` + `AI_BRAINS_KEY`. **Quote values** (e.g. `AI_BRAINS_KEY="x'…'"` and forward-slash path) so dotenvy parses correctly. |
| **Project `.env`** | Prefer **IDs only** via `ai-brains context` (`AI_BRAINS_PROJECT_ID` / `SESSION_ID`). CLI still gap-fills **any** unset keys from project dotenv (including KEY) — do **not** commit secrets; put KEY in global dotenv or shell. |
| **Vault path** | CLI requires `--vault-path` or `AI_BRAINS_VAULT_PATH` (often from global dotenv). **No** silent home default on the CLI. Daemon may fall back to `~/.ai-brains/vault.db`. |
| **This machine’s live vault** | Often set via global dotenv (e.g. `C:/dev/ai-brains/vault.db`) — not the source tree `C:\dev\AI-Brains`. |

**Encryption layers (F8):** plain SQLite (legacy) → **SQLCipher page encrypt** (`ai-brains vault encrypt`) → Content Envelope (payload DEK) → OS ACLs. Setting a key does **not** encrypt a plain file; use `vault encrypt`.

**Missing key:** vault commands fail with `VAULT_KEY_MISSING`. **Wrong key:** `Vault locked`. Doctor: missing → skip open; wrong → fail.

## Multi-repo model
- **One vault + one key** for all repos on a machine.
- **Per-repo** optional project/session identity via `ai-brains context` (local `.env`).
- Without project context, pass `--project-id` or use `--global` on recall/preflight when appropriate.

## Infrastructure
- Daemon may auto-launch; service uses `AI_BRAINS_VAULT_KEY` + path (ProgramData `daemon.env` and/or service Environment).
- Prefer `ai-brains daemon status` for liveness (vault key not required for status).
- Errors: dual envelope — governed JSON often on stdout; generic paths on stderr. See `Docs/CLI-EXIT-CODES.md`.

## Workflow phases (non-destructive first)

### Phase 0: Health (do this first on cold start)
```powershell
ai-brains doctor
ai-brains daemon status
ai-brains context --show   # confirm project id / vault env warnings
```
If `doctor` cannot open the vault: fix `AI_BRAINS_KEY` / global dotenv / path before recall.

### Phase 1: Orient
1. `ai-brains safety sync --dry-run` — Ledgerful hotspots preview (non-mutating).
2. `ai-brains preflight --summary` then `--pretty` or `--format json` if needed.
3. **Verify project:** if warnings say local `.env` overrides shell, trust `context --show`. Wrong `AI_BRAINS_PROJECT_ID` → wrong preflight/recall brain.

### Phase 2: Recall (search before acting)

**Daily “what did we decide?” is `recall` / `search`.** Briefing and `query progressive` read only Approved decisions + Active/Confirmed conclusions. Discovery grants do not turn vault pins into that authority.
```powershell
# Prefer explicit project or global when unsure
ai-brains recall "<topic>" --limit 5 --format pretty
ai-brains recall "<topic>" --project-id <uuid> --limit 5 --format pretty
ai-brains recall "<topic>" --global --limit 5 --format pretty
ai-brains recall "<topic>" --semantic --limit 5 --format pretty   # needs embedding backend
ai-brains search "<topic>" --limit 5 --format pretty              # visible alias of recall
ai-brains sync query "<topic>" --quiet   # vault + Ledgerful ledger
```
- Empty JSON recall includes a **hint** (`--semantic` / `--global`). Pretty empty can look blank except logs — try `--format json` or `--global`.
- Ignore stale DECISION text that contradicts current docs (e.g. pre-T187 “SQLCipher not live”); prefer Ledgerful ledger rows + `Docs/COMPATIBILITY.md` F8.

### Phase 3: Record (mutating)
`ai-brains pin "DECISION: …"` / `CONSTRAINT:` / `INVARIANT:` — dense knowledge only. Tags: `--tag`. Long text: `--stdin`.

### Phase 4: Forget (mutating)
`forget --list-forgotten` (read); `--memory-id` / `--match` + `-f`; `--restore <uuid>`.

### Governed discovery (may POLICY_DENIED)
`scope resolve`, `source list`, `evidence list`, `review list`, `briefing project` need **policy grants** for the principal/scope. Deny + `details.hint` is expected without grants — fall back to preflight/recall. Granted-empty briefing/lists still mean “no Approved/Active authority” — use `recall`, not “seed an Approved decision.” Personal briefing deny is optional continuity, not a required bootstrap. Not a vault-key problem.

## Command summary (agents)

| Goal | Command | Notes |
|------|---------|--------|
| Health | `doctor`, `daemon status` | Best non-destructive start |
| Project identity | `context --show`, `project list`, `project detect` | detect: git slug → vault → env; warns on git/env mismatch |
| Orient | `preflight --summary` / `--pretty` | Scoped by project id |
| Search | `recall` / `search` (alias), `sync query --quiet` | Scope carefully; `search` is vault-first recall, not ledger or progressive |
| Harness capture | `harness install --harness all-ready --dry-run` then `--yes` | Five ready (grok → agy → opencode → claude → codex). Codex live fire needs `/hooks` trust. No nightly Claude/Codex. |
| Hotspots preview | `safety sync --dry-run` | Prefer dry-run until user wants pin |
| Graph health | `graph update` | Needs graph-on install |
| Pin / forget | `pin`, `forget` | Mutating |
| Backup | `backup list` (read); `backup create` (write) | Old plain backups may WARN under new key |

## Maintenance (ops; ask before destructive)
- `nightly`, `backup create`, `recovery export`, `vault encrypt`, `daemon stop` / service control
- Install CLI graph-on: `cargo install --path crates/ai-brains-cli --locked --features graph`

## Normative docs
- `Docs/INSTALL.md` — key form, dotenv order, first vault  
- `Docs/OPERATIONS.md` — daemon/service, env table  
- `Docs/COMPATIBILITY.md` F8 — encryption honesty  
- `Docs/CLI-EXIT-CODES.md` — exits and envelopes  
