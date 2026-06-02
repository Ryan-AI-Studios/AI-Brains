# AI-Brains Workflows Cookbook

End-to-end recipes for the most common AI-Brains flows. Each recipe
shows the exact PowerShell commands and what you should see. Recipes
assume a default Windows install with `ai-brains` on `$PATH` and a
vault path of `C:\dev\my-project\.ai-brains\vault.db`.

> See [OPERATIONS.md](OPERATIONS.md) for command-by-command reference
> material, and [Deviations.md](Deviations.md) for Windows-specific
> notes.

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
    "DECISION: Use SQLCipher for vault storage; never store raw event payloads unencrypted."

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

- `antigravity-import` prints a JSON status object with the number of
  sessions and turns ingested. If the user's Antigravity history is
  empty, it exits 0 with a no-op status.
- `recall` ranks user prompts, assistant responses, and pinned
  memories, and the `--format pretty` view shows the top hits in
  context.

> **Tip — avoid cross-vault contamination.** When running on an
> isolated, CI, or per-project vault, use
> `ai-brains nightly --skip-import` instead. `antigravity-import`
> always reads the *user's* actual Antigravity history.

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

# 3. List everything currently forgotten, so you can audit later.
ai-brains --vault-path .ai-brains\vault.db forget --list-forgotten

# 4. Restore something you forgot by mistake.
ai-brains --vault-path .ai-brains\vault.db forget --restore <uuid>
```

What you should see:

- `--match` prints a list of matching memories with their UUIDs and
  status (`active` / `forgotten`).
- `--memory-id <unknown>` exits 1 with `Memory <id> not found.`
  (T77 — clear error instead of silent no-op).
- `--list-forgotten` prints a table of `memory_id`, `forgotten_at`,
  and a one-line content excerpt.
- `--restore` flips the projection status back to `active`.

---

## 4. Backup before a risky op

Goal: take a timestamped snapshot, perform the operation, and recover
if anything goes wrong.

```powershell
# 1. Create a backup.
ai-brains --vault-path .ai-brains\vault.db backup

# 2. (Run the risky operation — for example, a graph rebuild.)
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

## 5. Find code that changed

Goal: see what code in this repo was touched recently, and recall any
related memory context.

```powershell
# 1. Pull structured safety/ledger entries from ChangeGuard.
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

## 6. Schedule nightly + daemon

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
> Antigravity history. There is no analogous `--skip-import` for
> `daemon schedule` — the daemon itself does not import.
