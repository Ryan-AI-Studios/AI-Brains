# AI-Brains Workflows Cookbook

End-to-end recipes for the most common AI-Brains flows. Each recipe
shows the exact PowerShell commands and what you should see. Recipes
assume a default Windows install with `ai-brains` on `$PATH` and a
vault path of `C:\dev\my-project\.ai-brains\vault.db`.

> See [OPERATIONS.md](OPERATIONS.md) for command-by-command reference
> material, and [Deviations.md](Deviations.md) for Windows-specific
> notes.

---

## 0. Project identity triangle (T240 / T258)

Three signals can disagree — **daily Scope is always the effective env
`AI_BRAINS_PROJECT_ID` after local `.env` force-set** (never auto-switched
to path). Use `project whoami` when Scope looks wrong. Rebind with
`project adopt-path` (print-only) / `--write-env --yes`.

```text
                    git slug (detect step 2)
                           │
                           ▼
              vault name/alias match ──► may be empty or a thin project

  repo .env PROJECT_ID ──► daily Scope (*)  ← force-set over shell

  path register-path  ──► path_alias owner  ← detect step 1 (wins over slug)
```

| Signal | What sets it | Used by |
|--------|--------------|---------|
| **Env / daily Scope** | Project `.env` `AI_BRAINS_PROJECT_ID` (force-set) or shell / `--project-id` | recall, pin, preflight, most commands |
| **Path alias** | `project register-path <id\|alias> <path>` | `project detect` (first), nightly multi-root, whoami, mismatch warn |
| **All roots** | `project list-paths` | See every registered filesystem root (`project list` stays first-path-only) |
| **Unbind root** | `project unregister-path <path>` | Free a mistaken bind; symbols stay |
| **Rebind root** | `project rebind-path <path> --to <dest>` | Move one path to dest (print-only; `--write --yes`). Memories stay |
| **Shared leftover inventory** | `project list-paths --shared-only` / `--project <id>` | See multi-root leftover IDs without scrolling every root |
| **Discover roots** | `project scan-roots [path]` | Dry-run `.ledgerful` children; copy suggested `register-path` |
| **Git slug** | `origin` remote name (else toplevel dir) | `project detect` when no path owner |

### Operator runbook (identity confusion)

```powershell
cd C:\dev\your-repo
ai-brains project whoami --format json
ai-brains project detect
ai-brains project list   # label + first path column
ai-brains project list-paths   # every registered root
ai-brains project scan-roots C:\dev   # dry-run .ledgerful discovery; never writes

# Prefer the real work project for this repo (the path-alias owner):
# ai-brains project set-alias <path-owner-uuid> MyRepoLabel   # human label only
# ai-brains project register-path <path-owner-uuid> C:\dev\your-repo
# ai-brains project unregister-path --dry-run C:\dev\wrong-root
# ai-brains project unregister-path C:\dev\wrong-root
ai-brains project adopt-path --format human
ai-brains project adopt-path --write-env --yes   # confirmable; only AI_BRAINS_PROJECT_ID
ai-brains preflight --summary   # Scope should match the path owner
```

If you see `Warning: project identity mismatch…`, Scope and path disagree —
whoami shows both. Default `adopt-path` is print-only; `--write-env --yes`
is the confirmable write (T240 F2: no silent auto-switch). `context` is
not the remediator.

### Leftover identity split (T259)

A leftover dump UUID (historically `7d97a456-…`) can own many unrelated
`C:\dev\*` roots. Inventory, then rebind **one path at a time**. Do **not**
`set-alias` that leftover UUID as `AI-Brains`. Rebind does **not** move
historical memories.

```powershell
ai-brains project list-paths --shared-only --format human
ai-brains project list-paths --project <leftover-uuid> --format human

# In the leftover repo, ensure dest first (already-initialized context upserts
# the .env PROJECT_ID into the open vault without rewriting .env):
cd C:\dev\crawlx
ai-brains context

ai-brains project rebind-path C:\dev\crawlx --to <dest-uuid> --format human
ai-brains project rebind-path C:\dev\crawlx --to <dest-uuid> --write --yes
```

Repos without a `.env` (e.g. gimp / homebrew-tap) still need first-init `context`, which writes `.env`. Do **not** rebind `C:\dev\ai-brains` off its real path owner as leftover cleanup.

---

## 1. First-time setup

Goal: get a fresh vault, register the project, and pin your first
decision.

```powershell
# 1. Create the project dir and step into it.
mkdir C:\dev\my-project
cd C:\dev\my-project

# 2. Initialize the vault.
ai-brains --vault-path .ai-brains\vault.db init

# 3. Initialize the project context (writes AI_BRAINS_* to .env).
ai-brains --vault-path .ai-brains\vault.db context

# 4. Pin a high-level decision so the next recall will surface it.
ai-brains --vault-path .ai-brains\vault.db pin `
    "DECISION: Prefer Content Envelope encryption for sensitive payloads; never store raw secrets in plain vault fields. Vault page-level SQLCipher is live (T187; COMPATIBILITY F8) — not FIPS/Purge; zero key refused unless AI_BRAINS_ALLOW_ZERO_KEY=1."

# 5. Verify the pin comes back from recall.
ai-brains --vault-path .ai-brains\vault.db recall "vault storage decision"
```

What you should see:

- `init` prints `Vault initialized successfully`.
- `context` prints `Context initialized for project: my-project`
  and creates `.env` with `AI_BRAINS_PROJECT_ID`, `AI_BRAINS_SESSION_ID`,
  and `AI_BRAINS_HARNESS_ID`.
- `pin` prints `Memory pinned successfully: <memory-id>`.
- `recall` returns the pin in JSON or pretty form, depending on
  `--format`.

---

## 2. Capture an Antigravity session

Goal: pull a session from your local Antigravity conversation history
and recall from it.

```powershell
# 1. Initialize a vault for this project (if you haven't already).
cd C:\dev\my-project
ai-brains --vault-path .ai-brains\vault.db init

# 2. Import Antigravity history from the last 30 days.
ai-brains --vault-path .ai-brains\vault.db antigravity-import --days 30

# 3. Recall across the imported content.
ai-brains --vault-path .ai-brains\vault.db recall "what did we discuss about the auth flow?"
```

What you should see:

- `antigravity-import` prints **human** status lines on **stderr**
  (found / imported_turns / sessions / skipped_quiescent /
  skipped_unchanged_meta / unbound_project / bound_via_history /
  bound_via_path). It does **not** emit a JSON status object today.
  Empty history exits 0 with a no-op message.
- `recall` ranks user prompts, assistant responses, and pinned
  memories, and the `--format pretty` view shows the top hits in
  context. Prefer project-scoped recall after history binding.

> **Tip — avoid cross-vault contamination.** When running on an
> isolated, CI, or per-project vault, use
> `ai-brains nightly --skip-import` (skips AGY + Grok + OpenCode batch
> importers). Manual `*-import` commands always read the *user's*
> actual harness homes. SYSTEM scheduled nightly keeps `--skip-import`
> by default (T239); use user-context `nightly` for multi-harness
> completeness.

---

## 3. End-of-day memory hygiene

Goal: review what was learned today, prune the noise, and surface
the keepers.

```powershell
# 1. Search for candidate memories by content.
ai-brains --vault-path .ai-brains\vault.db forget --match "temp scaffolding"

# 2. Forget the ones that are clearly throwaway. Use --force in
#    non-interactive shells.
ai-brains --vault-path .ai-brains\vault.db forget --memory-id <uuid> --force

# 3. List forgotten rows (bounded, Scope-honest; same backend as memory list).
ai-brains --vault-path .ai-brains\vault.db forget --list-forgotten --limit 20
# Or skim pinned inventory without a recall query:
ai-brains --vault-path .ai-brains\vault.db memory list --limit 20
ai-brains --vault-path .ai-brains\vault.db memory list --summary

# 4. Restore something you forgot by mistake (soft restore — not CE wipe).
ai-brains --vault-path .ai-brains\vault.db forget --restore <uuid>
```

What you should see:

- `--match` prints a list of matching memories with their UUIDs and
  a one-line preview.
- `--memory-id <unknown>` exits 1 with `Memory <id> not found.`
  (T77 — clear error instead of silent no-op).
- `--list-forgotten` / `memory list --status forgotten` print **Scope**,
  a bounded table (`memory_id`, optional `project` under `--global`,
  `updated`, preview), and `Showing N of T` when truncated (default
  limit 50). Soft-forget ≠ CE wipe / NIST Purge. When the forgotten list
  is empty, human output keeps `No forgotten memories.`, then `Pinned: N`
  (same COUNT as `memory list --summary`) and last-line
  `next: ai-brains memory list` so operators still see live pins.
- `--restore` flips the projection status back to pinned (soft restore).

---

## 4. Backup before a risky op

Goal: take a timestamped snapshot, perform the operation, and recover
if anything goes wrong.

```powershell
# 1. Create a backup.
ai-brains --vault-path .ai-brains\vault.db backup

# 2. (Run the risky operation — for example, a graph rebuild.)
# Stop the daemon first so LiveGraphHook cannot race DELETE+replay.
ai-brains daemon stop
ai-brains --vault-path .ai-brains\vault.db graph rebuild --dry-run
ai-brains --vault-path .ai-brains\vault.db graph rebuild

# 3. If you want to roll back, dry-run the restore first.
ai-brains --vault-path .ai-brains\vault.db backup restore `
    --path .ai-brains\backups\vault-2026-06-02T18-30-00.db `
    --dry-run

# 4. If the dry-run output looks right, force the restore.
ai-brains --vault-path .ai-brains\vault.db backup restore `
    --path .ai-brains\backups\vault-2026-06-02T18-30-00.db `
    --force
```

What you should see:

- `backup` creates `.ai-brains/backups/vault-<RFC3339>.db` and prints
  the path.
- `backup restore --dry-run` prints the actions it *would* take and
  exits 0 without writing.
- `backup restore --force` prints the actions, swaps the vault file,
  and exits 0.

---

## Activate harness capture

Goal: wire ready coding harnesses (grok, agy, opencode, claude, codex) for
message-only capture. User-global only — never repo-local hooks.

```powershell
# 1. Preview writes (zero files created).
ai-brains harness install --harness all-ready --dry-run

# 2. Install (non-TTY agents must pass --yes).
ai-brains harness install --harness all-ready --yes

# 3. Confirm grok / agy / opencode / claude / codex report wiring=ok.
ai-brains harness status

# 4. In Codex, review and trust the managed hook (required for live fire).
#    /hooks  →  trust ai-brains-capture

# 5. Doctor should not list those five as ready-missing / T253-pending.
ai-brains doctor
```

What you should see:

- Dry-run prints **five** plans (grok → agy → opencode → claude → codex)
  and writes nothing.
- After `--yes`, `harness status` shows those five **ok** when the binaries
  are present. `wiring=ok` for Codex means files exist — live fire still
  needs `/hooks` trust of `ai-brains-capture`.
- `doctor` `harness_wiring` (soft ok) has **no** T253 pending clause when
  Claude/Codex are install_ready. After a successful install it should not
  list the five as ready-missing; next-action is `all-ready --dry-run` only
  while ready backends are still unwired.
- **C7:** writes stay user-global (`~/.grok/hooks`, `~/.gemini/config`,
  optional `~/.gemini/antigravity-cli/plugins/ai-brains-capture/` iff
  that CLI home already exists, `~/.config/opencode/plugins`,
  `~/.claude/settings.json`, `~/.codex/hooks.json`,
  `~/.ai-brains/hooks`). No repo `.claude/` / `.codex/` hooks.

Re-run install after `cargo install` so baked `ai-brains` paths update.

---

## 5. Find something

Goal: pick the right search command for vault-only vs vault+ledger vs governed conclusions, human vs agent.

```powershell
# Human, vault only (TTY pretty; or force --format pretty). `search` is an alias of `recall`.
ai-brains recall "auth middleware decision" --format pretty
ai-brains search "auth middleware decision" --format pretty

# Agent / pipe / scripts (JSON default when non-TTY)
ai-brains recall "auth middleware decision" --limit 5

# Human: vault memories + Ledgerful ledger pane in one view
ai-brains sync query "auth middleware" --format pretty

# Governed conclusions / decisions (needs discovery grants; not vault FTS)
ai-brains query progressive "why was graph backend replaced?" --project-id <uuid>

# Semantic / hybrid (embeddings) — recall only, not sync query
ai-brains recall "auth middleware" --semantic --format pretty

# Machine stream of vault hits
ai-brains recall "auth middleware" --format json
# or:
ai-brains sync query "auth middleware" --format ndjson --no-bridge
```

What you should see:

- `recall` / `search` pretty: `Scope:` + hits (or empty next-step including a `sync query` tip when you need the ledger pane). `--format text` is pretty (same as `sync query --format text`).
- `sync query` pretty: `--- AI-Brains Recall ---` vault block + optional `--- Ledgerful Ledger Search ---`.
- `query progressive`: JSON `ProgressiveQueryResponse`. Deny → exit **3**, `denied: true`, bootstrap + ungoverned `recall` hint. Authorized empty → exit **0**, `denied: false`, `results: []`, `next_step` names `recall` (empty governed ≠ empty vault).
- Missing/invalid project on `sync query` → `Scope: project=(none)` (vault-wide), not a random UUID.
- Invalid `AI_BRAINS_PROJECT_ID` on `recall` / `search` → clap exit **2** (env parse); on `sync query` → exit **0** with `project=(none)`.

> **Note.** Prefer `sync query` when comparing plan vs shipped / ledger context.
> Prefer `recall` / `search` for agents, `--semantic`, and **“what did we decide?”** (vault pins).
> Prefer `query progressive` only when you need *typed* Approved decisions / Active-Confirmed conclusions with evidence handles. Discovery grants do **not** turn pins into that authority.
> Full decision table: [CAPABILITIES.md §15](CAPABILITIES.md#15-typical-agent-workflows).

---

## 6. Find code that changed

Goal: see what code in this repo was touched recently, and recall any
related memory context.

```powershell
# 1. Pull structured safety/ledger entries from Ledgerful.
ai-brains --vault-path .ai-brains\vault.db safety sync --limit 50

# 2. Recall semantically across the same vault (combines FTS5 + embeddings).
ai-brains --vault-path .ai-brains\vault.db recall --semantic `
    "what did we change in the auth middleware last week?"
```

What you should see:

- `safety sync` prints a JSON array of `LedgerEntry` records
  (file path, tx id, risk score, etc.). With `--dry-run`, it lists
  pending entries without applying them.
- `recall --semantic` mixes keyword hits and embedding hits; the
  `graph_boost` adds a small bonus to graph-neighbor results.

> **Note.** `recall --semantic` requires a configured local model
> (Ollama). Without one, the FTS5 path still works; you'll see a
> warning that semantic search was disabled.

---

## 7. Schedule nightly + daemon

Goal: register the nightly sweep and the local daemon so they run
automatically when you log in.

```powershell
# 1. Schedule the nightly sweep (Windows Task Scheduler).
ai-brains --vault-path .ai-brains\vault.db nightly `
    --schedule --start-time 03:00

# 2. Register the daemon to auto-start at logon.
ai-brains --vault-path .ai-brains\vault.db daemon schedule

# 3. Verify both are healthy.
ai-brains --vault-path .ai-brains\vault.db nightly --status
ai-brains --vault-path .ai-brains\vault.db daemon status
```

What you should see:

- `nightly --schedule` prints `Nightly task registered for 03:00` and
  the schtasks command it ran. (T78 fixed a trailing-backslash bug
  in this command.)
- `daemon schedule` prints `Logon task registered: ai-brainsd`.
- `--status` commands print read-only summaries without starting
  anything.
- `daemon schedule` requires an elevated shell on first run; the
  command will tell you if it needs elevation.

> **Heads-up.** On isolated/CI/per-project vaults, register
> nightly with `--skip-import` to avoid reading the user's real
> harness history (AGY/Grok/OpenCode). There is no analogous
> `--skip-import` for `daemon schedule` — the daemon itself does not
> import.
