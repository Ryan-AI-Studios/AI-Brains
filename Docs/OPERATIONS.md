# AI-Brains Operations Guide

This guide covers the day-to-day operations, configuration, and troubleshooting of the AI-Brains system.

> **Current state:** Live track registry and product status live in [`conductor/conductor.md`](../conductor/conductor.md) and the docs index [`Docs/README.md`](README.md). The CLI surface is large (~30+ top-level commands — run `ai-brains --help`); the daemon can auto-launch; nightly can schedule via Windows Task Scheduler; the Ledgerful bridge is live. This guide is an **ops reference** — sections may lag new governed/multi-device commands; prefer `--help` and the docs index when in doubt. Vault encryption: [COMPATIBILITY.md](COMPATIBILITY.md) F8 — **SQLCipher page-level live (T187)** + Content Envelope; not FIPS/Purge.

## 1. Installation and Setup

### Prerequisites
- Rust (Stable, MSVC toolchain) — workspace pin **1.95.0** (`rust-toolchain.toml`)
- PowerShell 7+ (Recommended for Windows)
- `cargo-nextest`, `cargo-deny`, `cargo-audit` — see [ci-tooling.md](ci-tooling.md) for pins
- **Platform support:** Windows 11 x64 is primary (T1). Ubuntu 24.04 / WSL and macOS are tiered — see the normative matrix in **[COMPATIBILITY.md](COMPATIBILITY.md)** before claiming secondary-platform support.

### Build
```powershell
cargo build --release
```

### Initializing a Vault
Every project or user needs a vault. **T73 made `init` safe to re-run.**
- If the vault file does not exist, it is created and migrations are applied.
- If the vault is empty, the command succeeds idempotently.
- If the vault is populated, the command **refuses** with exit 1 unless `--force` is set.

```powershell
ai-brains --vault-path C:\path\to\vault.db init           # create new vault
ai-brains --vault-path C:\path\to\vault.db init           # re-run on empty vault: no-op
ai-brains --vault-path C:\path\to\vault.db init --force   # explicit overwrite
```

When the refused case triggers, the CLI returns a structured error envelope on **stderr** via the generic `handle_cli_result` path (full `ApiResult` shape) and exits **1**. Governed Json failures use a different dual-envelope model (bare `ApiError` on **stdout**); see [CLI-EXIT-CODES.md](CLI-EXIT-CODES.md).

## 2. Ingesting Data

AI-Brains follows an "Ingest-First" philosophy. All conversation data should be piped into the CLI as JSON.

### Manual Ingestion
```powershell
$json = @{
    session_id = "uuid"
    project_id = "uuid"
    harness_id = "uuid"
    turn_id    = "uuid"
    role       = "user"
    content    = "This is a memory."
    privacy    = "CloudOk"
} | ConvertTo-Json -Compress

echo $json | ai-brains --vault-path ./vault.db ingest
```
Empty / whitespace-only / TTY stdin is usage **exit 2** plus a copy-paste example — not EOF `COMMAND_FAILED`. Pipe JSON (the `ConvertTo-Json` sample above).

### Capture Privacy / message-only (T234)

Harness importers and hooks must keep **only** user prompts and final assistant text. Shared SOOT: `ai_brains_adapters::message_only` + `parse_transcript_for_ingest` (step-shaped AGY2 + legacy role/content; prefer `transcript_full.jsonl`). Used by batch import and `agy-hook`. Tool steps (`VIEW_FILE`, `RUN_COMMAND`, tool results), `reasoning`/`thinking`, and system chrome are dropped. The optional `IngestRequest.thinking` DTO field is never populated by adapters.

### Antigravity Import
Bulk-import Antigravity conversation logs from local tool-specific brain dirs.
```powershell
ai-brains antigravity-import --days 30
ai-brains antigravity-import --days 30 --force
```
- `--days <N>`: only import sessions modified in the last N days (default 30).
- `--force`: skip the 5-minute quiescence window for recently modified files.
- Binds `conversationId` → workspace via `history.jsonl` (normalized path alias). Missing history → stable `agy-unbound` / `(unbound AGY)` — **not** cwd `.env` project by default.
- Idempotent: path-keyed `source_meta` + delta turn index; message-only SOOT.
- Human stats on **stderr** (bound/unbound/quiescent/unchanged counters). Not a JSON status object.
- **Scheduled SYSTEM nightly** keeps `--skip-import` by default (T239 D12) — it does **not** run AGY/Grok/OpenCode batch import under Session 0. Manual / user-principal `nightly` without that flag runs multi-harness import (agy → grok → opencode).

### Harness detect + install (T235)

Detect which coding harnesses are **installed on this machine** (not “active this session”) and install message-only capture wiring into **user-global** paths.

**Activation (T245+T253)** — ready backends (grok → agy → opencode → **claude → codex**) in one pass (`all-ready` is five). Non-TTY agents must pass `--yes`.

```powershell
ai-brains harness install --harness all-ready --dry-run
ai-brains harness install --harness all-ready --yes
ai-brains harness status
```

Re-run `harness install` after `cargo install` / a CLI upgrade so baked wrapper and OpenCode `ai-brains` spawn paths update.

```powershell
ai-brains harness status
ai-brains harness status --format json
ai-brains harness install --harness agy --dry-run
ai-brains harness install --harness agy --yes
ai-brains harness install --harness grok --yes
ai-brains harness install --harness opencode --yes
ai-brains harness install --harness claude --yes
ai-brains harness install --harness codex --yes
ai-brains harness uninstall --harness agy --yes
ai-brains harness uninstall --harness grok --yes
ai-brains harness uninstall --harness opencode --yes
ai-brains harness uninstall --harness claude --yes
ai-brains harness uninstall --harness codex --yes
ai-brains harness reset-decline --harness all
```

| Harness | Detect | Install ready |
|---------|--------|---------------|
| grok | PATH `grok` or `~/.grok` | **Yes (T237)** — Stop+SessionEnd → wrapper (**empty** stdout) → `grok-hook` |
| agy | PATH `agy` or `~/.gemini/...` | **Yes** — Stop → wrapper (allow-stop JSON stdout) → `agy-hook` |
| opencode | PATH / `~/.config/opencode` (or `OPENCODE_CONFIG_DIR`) | **Yes (T238)** — managed plugin `session.idle` **or** idle `session.status` → `opencode-hook` |
| claude | PATH / `~/.claude` | **Yes (T253)** — UPS+Stop+SessionEnd → wrapper (empty Stop stdout) → `claude-hook`. No SessionStart injection. |
| codex | PATH / `~/.codex` | **Yes (T253)** — UPS+Stop → wrapper (`{"continue":true}`) → `codex-hook`. Feature key **`hooks`** (not `codex_hooks`). After install run Codex `/hooks` and trust `ai-brains-capture`. |

AGY writer **always** merges managed key `ai-brains-capture` into `%USERPROFILE%\.gemini\config\hooks.json` (creates `config/` only) and writes `%USERPROFILE%\.ai-brains\hooks\agy-stop.ps1`. **Iff** `%USERPROFILE%\.gemini\antigravity-cli` already exists, also stage the CLI plugin bundle `%USERPROFILE%\.gemini\antigravity-cli\plugins\ai-brains-capture\{plugin.json,hooks.json}` (same Stop command as IDE). Never create `antigravity-cli` just to host plugins. Never write undocumented top-level `antigravity-cli\hooks.json`. Uninstall removes the managed IDE key only (foreign keys stay; empty `{}` left if last) and deletes **only** `plugins\ai-brains-capture\` — not `antigravity-cli` or sibling plugins. Foreign IDE hooks are preserved; corrupt JSON refuses rewrite (exit 1). PATH bake: wrappers + OpenCode `ai-brains` spawn use the installing exe absolute path with PATH fallback.

Grok writer creates `%USERPROFILE%\.grok\hooks\ai-brains.json` (dedicated file; sibling `*.json` preserved) and `%USERPROFILE%\.ai-brains\hooks\grok-capture.ps1`. Command line is absolute PowerShell `-File` with **no `$`**. **Grok Stop allow contract:** exit 0 with **empty host stdout** — never emit AGY-style `{"decision":"allow"}` (undefined for Grok Stop). Wrapper captures `grok-hook` stdout to stderr only. Vault is **not** required for `harness` subcommands.

```powershell
ai-brains harness install --harness grok --dry-run
ai-brains harness install --harness grok --yes
ai-brains grok-import --days 30
ai-brains grok-import --days 30 --force
```

**Grok session layout:** `~/.grok/sessions/<percent-encoded-cwd>/<sessionId>/chat_history.jsonl` (+ sibling `summary.json` for bind). Never ingest `updates.jsonl`. User rows kept only when content has non-empty `<user_query>`/`<USER_REQUEST>` body; subagent/worktree sessions skipped by default.

OpenCode writer creates `%USERPROFILE%\.config\opencode\plugins\ai-brains-capture.js` (or `$env:OPENCODE_CONFIG_DIR\plugins\`). Marker must be on the **first non-empty line** (`// AI-Brains managed (T238)` — do not bump the marker). **Never** rewrites `opencode.json(c)`; **never** deletes foreign plugins; refuse overwrite of same-name file without our header marker. Plugin dual-subscribe: `session.idle` **or** `session.status` with `status.type == "idle"` (exact; no aliases). `session.idle` is **not** deprecated — dual-subscribe is resilience. Then `client.session.get` (fail-closed skip if get throws; parentID skip) → shared in-flight guard → `client.session.messages` temp export → `ai-brains opencode-hook` (baked exe path, PATH fallback) → **unlink temp files**. Fail-open into OpenCode (never throw). Batch backstop does not require the plugin.

```powershell
ai-brains harness install --harness opencode --dry-run
ai-brains harness install --harness opencode --yes
ai-brains opencode-import --days 7
ai-brains opencode-import --days 7 --force --dry-run
ai-brains opencode-import --days 7 --max-sessions 100
```

**OpenCode content SOOT:** nested export `{info,messages}` with message-only filter (drop tool/reasoning/step/snapshot/patch/file/subtask/agent/retry/compaction + synthetic/ignored/editor_context parts). **Never open `opencode.db`**. Watermark: `~/.ai-brains/opencode-import-cursor.json` (corrupt JSON → `cursor_corrupt` warn + empty start; optional additive `last_msg_id`). Missing `opencode` binary → soft skip. Child sessions (`parentID`) skipped. List length ≥100 (vendor default) or at requested cap → `list_capped` stderr warn. Export/list subprocesses killed on 120s timeout.

Preflight summary appends a **Harnesses installed on machine:** block when ≥1 harness is not absent. Flags: `--no-hook-prompt`, `--install-hooks`. Doctor soft check: `harness_wiring` (never fails solely for missing hooks). After T253, the pending-backend clause is gone when Claude/Codex are install_ready; next-action is `ai-brains harness install --harness all-ready --dry-run` for any ready-missing wiring. Severity remains soft ok.

### `agy` Hook
Real-time capture from the Antigravity CLI hooks integration:
```powershell
ai-brains agy-hook --payload '{"transcriptPath": "C:\\path\\to\\session.jsonl", ...}'
```
Diagnostics (auto-link / ingest counts) go to **stderr**. Prefer `ai-brains harness install --harness agy` so Stop events are mapped (conversationId→sessionId, workspacePaths[0]→projectHash, fullyIdle soft-skip). Shared step parser + path normalize; env project fallback only for empty/`agy-unbound`. Reinstall after T236 so wrapper stdout is allow-stop JSON only (no human prose leak to AGY).

### `grok` Hook
Real-time capture from Grok Build Stop/SessionEnd:
```powershell
ai-brains grok-hook --payload '{"sessionId":"...","projectHash":"C:\\dev\\AI-Brains","historyPath":"","workspaceRoot":"C:\\dev\\AI-Brains","event":"Stop"}'
ai-brains grok-hook --schema
```
`historyPath` may be empty — Rust resolves via `GROK_HOME`/`~/.grok` + percent-encode + `.cwd` + `summary.info.id` fallbacks. Diagnostics on **stderr**. Env `AI_BRAINS_PROJECT_ID` only when project is `grok-unbound`/empty.

### `opencode` Hook
Real-time capture from the OpenCode managed plugin (`session.idle` **or** idle `session.status`):
```powershell
ai-brains opencode-hook --payload '{"sessionId":"ses_abc","directory":"C:\\dev\\AI-Brains","worktree":"C:\\dev\\AI-Brains","messagesPath":"C:\\Temp\\oc.json","event":"session.idle"}'
ai-brains opencode-hook --schema
```
Prefer `messagesPath` / `exportPath` (export-shaped JSON). When `parentId` is set → **skipped_child_session** (exit 0). Project bind: worktree → directory → unbound `opencode-unbound`. Env `AI_BRAINS_PROJECT_ID` only when unbound (alias not stamped onto env project). Diagnostics on **stderr**.

### `claude` Hook + import
Real-time capture from Claude Code UserPromptSubmit / Stop / SessionEnd (T253). Live path ingests payload text only — **do not** parse `transcript_path`.
```powershell
ai-brains harness install --harness claude --dry-run
ai-brains harness install --harness claude --yes
ai-brains claude-hook --payload '{"sessionId":"...","projectHash":"C:\\dev\\AI-Brains","event":"Stop","lastAssistantMessage":"Done."}'
ai-brains claude-hook --schema
ai-brains claude-import --days 30
ai-brains claude-import --days 30 --force --dry-run
```
`--schema` is vault-path-free. Mid-payload garbage (invalid JSON) exits **1** with JSON. Unrecognized / Grok-shaped stdin exits **0** (no ingest; stderr once). Empty/whitespace prompt or last message skips that role (exit 0). Bind: `cwd` → `ai_brains_path::normalize_project_path` → path alias; `AI_BRAINS_PROJECT_ID` only when unbound. Unbound alias `claude-unbound`. Batch walks `~/.claude/projects/<encoded-cwd>/*.jsonl` (skip `subagents/` / `isSidechain=true`). `--force` skips 300s quiescence. Not in nightly.

### `codex` Hook + import
Real-time capture from Codex UserPromptSubmit / Stop (T253). Feature key is **`hooks`** (not `codex_hooks`). Live fire requires operator **`/hooks` trust** of `ai-brains-capture` — `wiring=ok` is files only.
```powershell
ai-brains harness install --harness codex --dry-run
ai-brains harness install --harness codex --yes
# next: in Codex run /hooks and trust ai-brains-capture
ai-brains codex-hook --payload '{"sessionId":"...","projectHash":"C:\\dev\\AI-Brains","event":"Stop","lastAssistantMessage":"Looks good.","turnId":"turn_1"}'
ai-brains codex-hook --schema
ai-brains codex-import --days 30
ai-brains codex-import --days 30 --force --dry-run
```
`--schema` is vault-path-free. Mid-payload garbage exits **1** with JSON. Missing fields exit **0**. Empty/whitespace prompt or last message skips that role. Bind same as Claude; unbound `codex-unbound`. Batch walks `~/.codex/sessions/**/rollout-*.jsonl` (or `CODEX_HOME`); keep only `response_item` + `payload.type=message` + user/assistant; drop `event_msg` / `session_meta` / unknown; malformed line skipped. Format is **not vendor-stable** — soft-skip rather than fake complete. Never edit `config.toml`. Not in nightly.

## 3. Retrieving Memories

### Lexical Recall
```powershell
ai-brains --vault-path ./vault.db recall "authentication logic" --limit 5
```
Options worth knowing:
- `--format pretty` for human-readable scores
- `--semantic` for vector (embedding) search alongside FTS5
- With `--semantic`, JSON carries additive `embedding.status` (`ok` / `unreachable` / `error` / `no_stored_embeddings`). Embed backend failure is soft-fail (exit 0; FTS/bridge still return). Model: `AI_BRAINS_EMBEDDING_MODEL` (default `nomic-embed-text-v1.5`); URL: `AI_BRAINS_EMBEDDING_URL` (default `http://127.0.0.1:8083`)
- `--graph-boost <0.0–1.0>` to weight graph-neighbor hits
- `--project-id` / `--session-id` to scope

### Unified Search (AI-Brains + Ledgerful)
The T70 bridge lets a single command search both your memory vault and the Ledgerful ledger.
```powershell
ai-brains sync query "rust" --format pretty
```
Output has two sections — `--- AI-Brains Recall ---` (vault FTS hits) and `--- Ledgerful Ledger Search ---` (ledger entries). `--no-bridge` skips the ledger pane. `--quiet` omits never-ran/failed ledger lines (hits and ran-empty still print).

### Generating Preflight Context
```powershell
ai-brains preflight --max-words 1500
```
- `--summary` for a concise statistical summary
- `--pretty` / `--format human` for human-readable text
- `--pretty --compact` for a tighter skim (JSON and `--summary` ignore `--compact`)
- `--scope "src/foo.rs,src/bar.rs"` for contextual risk analysis on a specific path set

Default `--pretty` Session/Recent lines are capped at 140 Unicode chars; Safety is full-length unless `--compact`.

### Governed briefings & progressive query (T152)

Set `AI_BRAINS_GOVERNED_BRIEFING=1` (or `true`/`yes`) to route `preflight` through the typed
`ProjectBriefingPacket` path (policy + scope authority + budget). Default remains the legacy
string-scrape preflight. The principal used for grant checks is
`AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` (UUID) when set; otherwise a well-known System principal.
Both must be **registered** and hold `ReadDecisions` / `ReadConclusions` grants for the
resolved repository scope, or authority sections are empty (`denied` / warnings).

**Empty-state contract**
- Unresolved / global preflight → empty project packet + warning (not a crash).
- Policy denial → `denied=true` (or empty sections) with a `denied` warning; never injects
  high-authority claims without a grant.
- Non-authoritative scope (Low/Ambiguous) → empty current decisions/conclusions +
  `low_confidence` warning.

**Packet shape (JSON)**
- Project: `api_version`, `briefing_id`, `kind="Project"`, `scope`, `decisions[]`,
  `conclusions[]`, `constraints[]`, `warnings[]`, `freshness`, `evidence_handles[]`,
  `budget`, optional `denied` / `denial_reason`. Personal fields are never nested.
- Personal: separate packet (`kind="Personal"`) with `preferences`, `continuity`,
  `open_review_items`, `grants_applied` — never embedded inside Project.
- Progressive query: `results[]` (handles + ranking), `query_trace_id`, `freshness_summary`,
  optional `denied` / `denial_reason` / **`denial_hint`** (bootstrap when denied; omitted otherwise).

**CLI surface (dry-run)**
- **Briefing** format (T227): markdown on TTY, json otherwise; explicit `--format` wins.
  - Aliases `human` / `pretty` / `text` / `markdown` / `md` → markdown; only `json` → JSON.
  - Unknown `--format` → exit **2** + accepted list on stderr (no silent JSON).
  - Preflight pins ≠ briefing authority (dual model); empty allowed packets surface next-steps.
  - **T288:** granted-empty CLI human prints a **Vault pins (not Approved)** stanza (`Pinned: N`); JSON may add `vault_pin_count` / `vault_pin_previews`. Authority arrays stay empty.
- **Progressive / expand / trace**: JSON only (not TTY-markdown).

```powershell
ai-brains briefing project --project-id <uuid> --format human
ai-brains briefing project --project-id <uuid> --format json
ai-brains briefing personal --format human
ai-brains briefing personal --format markdown
ai-brains query progressive "authority order" --project-id <uuid>
ai-brains query expand <handle-id> --project-id <uuid>
ai-brains query trace <trace-id>
```
Missing `--project-id` / `AI_BRAINS_PROJECT_ID` on `query progressive` and `query expand` exits **2** (`EXIT_USAGE`) with a copy-paste example on stderr. `query trace` is excluded (missing/unauthorized empty-success envelope, exit **0**).

**Progressive / expand policy walls (T221):** `query progressive` with policy deny prints the pretty packet on **stdout** (including `denied: true` and `denial_hint`) and exits **3** — not exit 0 empty-knowledge. Same for `--dry-run`. `query expand` with `kind: "Denied"` exits **3**; `kind: "Unknown"` stays exit **0**. **`Denied` may mean capability miss and/or cross-scope** — exit 3 does not prove which. stderr carries `POLICY_DENIED: …` then bootstrap remediation. First-run: `policy bootstrap --dry-run` then `policy bootstrap` (omit `--scope` when project context is authoritative; `--scope Repository:<uuid>` remains valid for no-context CI). Omit `--principal-id` to grant the default System principal used by progressive/expand.

### Governed command surface (T160)

Thin CLI over control-plane (local default for reads) and named-pipe daemon (preferred for mutations). JSON default for new commands.

**Scope resolve (T249):** default `--format auto` is TTY human (`scope:` / `confidence:` / evidence) and pipe JSON. Scripts should pass `--format json`. Tokens are case-sensitive (`JSON` / `Pretty` exit **2**). When the human report is not authoritative, last line may be `next: ai-brains project whoami`.

**Exit codes (normative 0–7):** see **[CLI-EXIT-CODES.md](CLI-EXIT-CODES.md)**. Quick map: 0 success / status / doctor ok|degraded; 1 internal / vault key / doctor fail; 2 clap usage + `FEATURE_UNAVAILABLE`; 3 `POLICY_DENIED` / `APPROVAL_REQUIRED`; 4 `NOT_FOUND`; 5 daemon unavailable; 6 `INVALID_PAYLOAD`; 7 hard-gate failed. Missing required `--scope` on clap-required commands exits **2** (not 6).

**Error envelopes (format-dependent):** governed Json → bare `ApiError` on **stdout**; governed Human → `CODE: message` on **stderr**; generic failures → full `ApiResult` on **stderr**.

```powershell
ai-brains scope resolve                 # TTY: human
ai-brains scope resolve --format json   # scripts: pretty JSON
# Discovery (T203) — list before show; soft-fill --scope when context is authoritative
ai-brains source list --format json
ai-brains source list --scope Repository:<uuid> --format json
ai-brains evidence list --scope Repository:<uuid>
ai-brains evidence list --query keyword --scope Repository:<uuid>
ai-brains evidence search --query keyword --scope Repository:<uuid>
ai-brains evidence show <id> --scope Repository:<uuid> --format json
ai-brains source show <id> --scope Repository:<uuid>
ai-brains review list --format json                    # soft-default scope or fail_usage exit 2
ai-brains review list --scope Repository:<uuid>
ai-brains conclusion propose --claim "..." --evidence <id> --scope Repository:<uuid> --local
ai-brains decision propose --statement "..." --scope Repository:<uuid>
ai-brains decision in-force workspace_id --format json
ai-brains review resolve <id> --resolution approved --scope Repository:<uuid>
ai-brains erasure request --id <id> --scope Repository:<uuid>   # daemon-required; ticket only (not CE)
ai-brains erasure wipe --content-key-id <uuid> --scope Repository:<uuid>           # dry-run (default)
ai-brains erasure wipe --content-key-id <uuid> --scope Repository:<uuid> --confirm # execute CE
ai-brains policy bootstrap --scope Repository:<uuid>   # T210: discovery grants (ReadEvidence/Conclusions/Decisions)
ai-brains policy bootstrap --scope Repository:<uuid> --dry-run
ai-brains policy bootstrap   # omit --scope when project context is authoritative
ai-brains policy show --scope Repository:<uuid>
ai-brains policy show --format json   # T226: soft-fill --scope when authoritative
ai-brains policy check --capability ProposeConclusion --scope Repository:<uuid>
ai-brains policy check --capability ReadEvidence --format human   # T292: TTY-friendly allow/deny lines
ai-brains policy check --capability ReadEvidence --format json   # omit --scope when authoritative
# default --format auto = TTY human / pipe JSON; scripts that previously parsed TTY default JSON must pass --format json
```

**Governed policy bootstrap (T210 + T241 cold-start):** After vault open, discovery lists (`source list` / `evidence list` / `review list`) and briefing sections need grants. **Discoverability surfaces** (T241): `doctor` soft check **`policy_grants`** (warn when discovery active_count < 3 under authoritative scope), `preflight --summary` post-hoc grants/next line when project-scoped incomplete, `policy show` empty human SOOT + JSON `next_step`, `policy check` without `--capability` → exit **2** capability catalog (not clap “required arguments”), briefing denied JSON **`denial_hint`**. Cold-start sequence: open vault → `policy bootstrap --dry-run` → `policy bootstrap` → `policy show` / `briefing project` / `evidence list`. Run **`ai-brains policy bootstrap --scope Repository:<uuid>`** (or omit `--scope` when project context is authoritative) once per principal+scope. Issues exactly `ReadEvidence`, `ReadConclusions`, `ReadDecisions` with `Privacy::LocalOnly`; registers the principal if missing; idempotent re-run. Does **not** issue Propose*/Approve*/Export/Erase. Does **not** auto-run on `init` (deny-by-default until explicit opt-in). `--dry-run` / `-n` reports the plan with zero event appends.

**Empty vs deny:** list/show hard deny = exit **3** + `details.hint` (mentions bootstrap first; human also prints hint after `CODE: message`). **Progressive** hard deny = exit **3** + packet on stdout (`denied: true`, `denial_hint`) + stderr CODE/hint — **not** “no knowledge.” Authorized empty progressive = exit **0**, `denied: false`, empty `results`, `next_step` is copy-paste `recall` of the operator query plus `(Pinned: N)` when COUNT succeeds. Expand `Denied` = exit **3**; expand `Unknown` = exit **0** with a non-empty `preview` SOOT. `query trace` missing/unauthorized = stdout JSON envelope (`found: false` + `next_step` copy-paste `query progressive … --dry-run false`) + exit **0** (`--format human` is two lines: `No trace` + `next:`). Not the token `null`. Briefing **granted-empty** (`empty_authority`) next names `recall` / `search` (vault pins are not Approved authority). Briefing **project** soft deny = exit **0** + bootstrap `denial_hint`. **T275 grant-wall:** Denied project markdown is not `_None_` empty — it says this is a grant wall and that pins remain via `recall` / `search`; run `policy bootstrap` then `briefing project` / `evidence list`. Briefing **personal** deny names `recall` (Personal is optional; not a required bootstrap). Authorized-empty `evidence`/`source`/`review` list JSON `next_step` is copy-paste `recall "what did we decide"` plus `(Pinned: N)` when local COUNT succeeds; human prints that line after `(none)`. Bootstrap clears discovery walls when grants are present for the **same principal** progressive uses (default System); it does **not** fill briefing authority from pins. Capture/recall/legacy preflight stay grant-independent.

**Discovery workflow (T203):** Prefer `source list` / `evidence list` (bounded, Active-only, optional FTS `--query`) to find ids, then `show`. When `--scope` is omitted, CLI soft-fills only if `scope resolve` is **authoritative** (e.g. `AI_BRAINS_PROJECT_ID` set → High). Non-authoritative context → **exit 2** with a `fail_usage` template (`--scope Repository:<uuid>`, `ai-brains scope resolve`); never reintroduces exit **6** for missing scope on these CLI paths.

- Mutations auto-generate `--command-id` when omitted; local propose uses shared CP `id_from_command` (same pre-assigned domain id as daemon).
- After a mutation is **sent** to the daemon, timeout → non-zero + "outcome unknown; retry same --command-id" (no silent local fallback).
- Erasure ticket and wipe are always daemon-required (`--local` rejected). Ticket honesty only — never claims content-envelope wipe. Wipe is a **separate** command.

### Erasure honesty (ticket vs cryptographic erasure)

Operators and docs must keep these mechanisms distinct. Normative design is [ADR-0016](DECISIONS/ADR-0016-content-envelope-cryptography.md) (Accepted 2026-07-28). **T165 shipped** governed CE wipe for envelope-backed content (with residuals below).

| Mechanism | What it does today | Cryptographic erasure (CE)? |
|-----------|--------------------|-----------------------------|
| `ai-brains erasure request` → `ErasureTicketAccepted` | Durable **ticket / intent** accepted by the daemon | **No** — ticket ≠ CE; does **not** write CE tables |
| `ai-brains forget` → `MemoryForgotten` | Soft hide/filter in projections | **No** — plaintext remains in the **append-only event log**; does **not** touch CE tables |
| `ai-brains erasure wipe` → `ContentErasureRequested` → destroy content DEK wrap → purge FTS/embeddings → `ContentErased` | **Implemented for envelope-backed** (`content_key_store` row required); dry-run default; execute needs `--confirm` | **Yes** under ADR-0016 §12 assumptions (live vault; wrap destroyed; derived indexes purged; AES-256 unbroken; no offline pre-erase copy) |

**Schema + crypto primitives (T163/T164):**

- Vault migration **`0026_content_envelopes_erasure`** creates side stores `content_key_store` / `encrypted_content_blob` and event projections `erasure_request_projection` / `tombstone_projection`.
- **Side stores are not event-sourced ciphertext:** `rebuild_projections` retains wrap + blob rows; it is **not** a backup restore of sealed content from the event log. Only erasure-request and tombstone projections are truncated and re-applied from events.
- Seal/open/wrap/destroy primitives (T164) + governed wipe (T165) form the product CE path.

**Post-wipe WAL (E16):** after successful wipe commit, the daemon runs `PRAGMA wal_checkpoint(TRUNCATE)` on the single-writer connection (BUSY → one retry → warn `pending_passive`). This reduces uncheckpointed deleted FTS/embedding pages in the `-wal` file. It is **not** NIST Purge, not free-page zeroization, and **not** `VACUUM`.

**Honest limits (do not over-claim):**

- **Pre-envelope / legacy content:** plaintext already written to the append-only log **cannot** be cryptographically erased without rewriting history (forbidden by event-sourcing invariants). Soft forget is the only mechanism for that class. Wipe refuses non-envelope keys with `NOT_ENVELOPE_BACKED` (no silent soft-forget fallback).
- **CE (ADR-0016):** for envelope-backed content only — per content-unit DEK wrapped under vault `DataKey`; AES-256-GCM; CE = destroy DEK wrap + purge derived FTS/embeddings; verify wrap absent via store re-query (not a fake AEAD `open_fails`).
- **Dependents:** conclusion/decision invalidation runs only when a blob subject is source-like **and** the `subject_id` is a registered `SourceId` (E15). Memory-only subjects purge indexes only.
- **Non-claims:** not NIST media **Purge**/**Destroy** (RustCrypto is not a FIPS-/NIST-validated module); not destruction of offline copies, exports, or **pre-erase backups**; not “SQLCipher vault lock = per-item CE”; WAL TRUNCATE is not media sanitization.
- CLI and HTTP surfaces must **never** present `ErasureTicketAccepted` or soft forget as content-envelope wipe.

### Class-based retention (T166 / P8.4)

Retention is a **class-and-risk matrix**, not a single global clock. Dry-run **before** destroy is mandatory.

```powershell
ai-brains retention plan                 # TTY: human class matrix
ai-brains retention plan --format json   # pretty JSON (scripts)
ai-brains retention apply --confirm --format json
```

| Rule | Behavior |
|------|----------|
| Dry-run default | `retention plan` is report-only; `retention apply` **refuses** without `--confirm` |
| One CE path | Envelope classes use the same T165 `wipe_content_envelope` path — **no** parallel destroy |
| Production apply dual path | **Projection** deletes run in-process. **CE** (`ce_wipe`) rows **require the daemon** (parity T165 E8); if any CE candidate exists and the daemon is down, apply fails with `DAEMON_UNAVAILABLE` before disposal. Projection-only apply may run without the daemon. |
| Legacy ≠ CE | Stream A projection `DELETE` is **never** labeled cryptographic erasure |
| Reports | Counts, class, mechanism, truncated sample ids only — **no** plaintext bodies |
| Approved decisions | Active `Approved` decisions are **not** age-wiped; only terminal `Revoked`/`Superseded` after cooldown |
| Nightly CE | **Never auto-applied.** `AI_BRAINS_RETENTION_APPLY_CE` / `APPLY_CE_ON_NIGHTLY` only **log intent**; they do **not** enable nightly CE. Nightly runs class dry-run log + raw-turn projection cleanup. Class CE remains confirm-gated CLI + daemon. |
| Pin hold (R11) | If **any** memory subject linked to a content key is pinned, the key is `held` (not age CE-wiped). |
| R15 cascade residual | Hierarchy cascade may mark a parent `stale` for resynthesis even if that parent was `pinned` (pin superseded by synthesis staleness after child CE). |
| Audit | Apply appends `RetentionApplied` (class counts/mechanisms; no bodies). Dry-run does not. `sample_ids` prefer dispose identities (`content_key:` / `turn:`) when CE or projection work exists; overlay pin ids are inventory-only (T284). Human **Work** is the due slice (dispose counts), not the class-matrix dominant mechanism. |

**Class matrix (v1 defaults)**

| Class | Stream | Horizon | Mechanism |
|-------|--------|---------|-----------|
| `raw_turn` | A (projection) | 90d | `projection_delete` (`delete_old_turns` / equivalent) |
| `evidence` | B (envelope) | 365d | `ce_wipe` if `content_key` present |
| `decision_approved` | A | revoked/superseded + 30d cooldown | projection cleanup of terminal rows only |
| `secret` | B | 7d | `ce_wipe` |
| `review_trace` | A | 90d from terminal `updated_at` | projection cleanup if closed |
| `query_trace` | A | 30d | projection delete by `recorded_at` |
| `memory_legacy` | A | none auto | Inventory overlay (T270): `held` for pinned rows, `skip` for other statuses. COUNT + ≤5 sample ids. Plan does not auto-forget. Zero-row (empty vault) still displays `skip`. |
| `orphaned_envelope` | B | 7d (active wrap, **0** blobs) | CE destroy wrap only |
| `unclassified` | either | skip apply | listed in dry-run only |

**Streams (R13):** Stream A identities are projection keys (turns, traces, decisions, …). Stream B is `content_key_id`. Plan never double-counts the same `content_key_id`. When a turn↔envelope join is known (`subject_kind=turn`, `subject_id={session_id}:{turn_index}`), CE wins and stream A projection delete for that turn is skipped. **Until capture writes that join, streams are independent** (documented residual: projection delete may run without CE of a sealed turn, and vice versa).

**Env knobs** (`AI_BRAINS_RETENTION_*`)

| Variable | Default |
|----------|---------|
| `AI_BRAINS_RETENTION_RAW_TURN_DAYS` | 90 |
| `AI_BRAINS_RETENTION_EVIDENCE_DAYS` | 365 |
| `AI_BRAINS_RETENTION_SECRET_DAYS` | 7 |
| `AI_BRAINS_RETENTION_QUERY_TRACE_DAYS` | 30 |
| `AI_BRAINS_RETENTION_REVIEW_TRACE_DAYS` | 90 |
| `AI_BRAINS_RETENTION_DECISION_REVOKED_COOLDOWN_DAYS` | 30 |
| `AI_BRAINS_RETENTION_ORPHAN_ENVELOPE_DAYS` | 7 |
| `AI_BRAINS_RETENTION_APPLY_CE` / `AI_BRAINS_RETENTION_APPLY_CE_ON_NIGHTLY` | false — **intent log only**; does **not** enable nightly CE. CE apply is `retention apply --confirm` + daemon only |

Horizon day overrides must be integers in **`1..=36500`** (~100y). Non-integer, ≤0, or oversized values **fall back to the class default** (never panic; never apply a negative horizon that would push cutoffs into the future). Cutoffs use checked `chrono::Duration` arithmetic.

**Honesty on every plan/apply with CE candidates:** not NIST Purge; pre-erase backups residual; ticket/soft forget ≠ CE; stream independence until subject join; legacy projection delete ≠ CE.

### Legacy → governed classification import (T167 / P9.1)

Maps historical AI-Brains events into governed Evidence / Candidate Conclusions / Proposed Decisions **without** promoting to Confirmed or Approved authority.

| Rule | Behavior |
|------|----------|
| Surface | Control-plane API (`classify_legacy` / `apply_legacy_import`). **CLI is live (T168):** `ai-brains migrate governed`. |
| Dry-run default | Classify builds a full plan + `plan_hash`; apply requires `ApplyOpts.confirm = true`. |
| No live vault | Module never opens `%USERPROFILE%\.ai-brains` itself; callers supply event stream + destination ports. |
| Under-promote | Pins → Evidence; synth → Candidate only; decisions → Proposed + ReviewItemOpened. Never auto-approve. |
| Forgotten | Final `forgotten` status excludes Evidence; synth with forgotten sources → `unsupported` + reason. |
| CE honesty | Import does **not** claim content-envelope cryptography (legacy plaintext ≠ CE). |
| Source | One `SourceRegistered` (`SourceKind::LegacyAiBrains`) per destination vault; **not** via `observe_source`. |
| Idempotency | uuid v5 natural keys; second apply → zero new aggregates (`has_evidence` / conclusion / decision probes). |
| Reports | Counts, ids, reason codes, `plan_hash` only — no full plaintext bodies by default. |
| Audit | Successful apply appends `LegacyImportApplied` (plan_hash + counts). Dry-run does not. |

There is **no** `RecordEvidence` capability; bulk import uses raw `build_event` appends (same discipline as invalidation reviews). Production operators must not point apply at the live vault without explicit T168 flags.

### Governed migrate CLI (T168 / P9.2)

Shadow-style **replay + differential report** for legacy → governed import. Does **not** cut over the live vault (that remains **T170**).

```powershell
# Dry-run (default): classify + write report; no dest vault / no manifest
ai-brains migrate governed `
  --source .\fixture-or-shadow.db `
  --destination .\migrated.db `
  --report .\migrate-report.json

# Confirm: materialize dest (copy envelopes when empty) + T167 apply + migrate-manifest.json
ai-brains migrate governed `
  --source .\fixture-or-shadow.db `
  --destination .\migrated.db `
  --report .\migrate-report.json `
  --confirm

# Optional: default scope for events without project_id (T167 L19)
ai-brains migrate governed --source .\s.db --destination .\d.db --report .\r.json `
  --default-scope "Repository:<project-uuid>" --confirm
```

| Rule | Behavior |
|------|----------|
| Dry-run default | Omit `--confirm` (or pass `--dry-run`). Report always written when `--report` is set. Both `--dry-run` and `--confirm` → `INVALID_PAYLOAD` exit **6**. |
| Confirm | Creates dest (if needed), dest-only `migrate()`, optional envelope copy, T167 apply, **mandatory** `migrate-manifest.json` beside dest. |
| Source integrity | Source (and dry-run dest peek) open via pure RO `VaultConnection::open_read_intent` (`SQLITE_OPEN_READ_ONLY` only — **no** R/W fallback; key pragmas only, **no** `journal_mode=WAL`). Never `migrate()` source; no intentional event writes (T147 residual **#12**). After all output writes (report + migrate-manifest), source content fingerprint is re-verified; mismatch aborts hard. |
| Dest safety | Reuses T147 `refuse_unsafe_destination`: refuse source==dest, dest==live, dest inside live parent, reparse dest/parent. Additionally refuses multi-link (hardlink) destination so confirm cannot R/W-open a dest that shares an inode with source/live. |
| Live source | Source == live vault refused unless `--allow-live-source` (still refuses live dest). Prefer `shadow create` first. |
| Report path | Refuse reparse/symlink; refuse existing hardlinked report path (prevents truncate of shared inode / source); refuse report path same location as source, dest vault file, or sibling `migrate-manifest.json` (would overwrite mandatory manifest). |
| Manifest path | Sibling `migrate-manifest.json` refused early if it collides with source or dest vault path (e.g. source/dest named `migrate-manifest.json`), or if the path exists as reparse/symlink/hardlink. Pre-write hardlink re-check retained. |
| Missing source | `NOT_FOUND` with exit **4**. |
| Copy events | Default **on** for empty dest (`--copy-events`); `--no-copy-events` for import-only. Re-apply into non-empty dest is **import-only** even if `--copy-events` (no duplicate source envelopes). |
| Re-apply | Non-empty dest requires matching `migrate-manifest.json` `source_fingerprint` (content-based, not mtime). Missing/mismatch → refuse unless `--force-overwrite`. |
| Force overwrite | With `--confirm`: delete dest db (+ WAL/SHM) and migrate-manifest, then recreate fresh (still subject to live/reparse refuse). |
| Keys | `--source-key` / `--destination-key` with `--key` / `AI_BRAINS_KEY` fallback (T197). `--key` may be placed **after** `migrate governed` or as a root flag before `migrate`. Resolution: source_key → shared key → `AI_BRAINS_KEY` → **Missing** (`VAULT_KEY_MISSING`; no silent zero); same for destination. Product form `x'<64 hex>'` only. Production DPAPI unlock of arbitrary live vaults is **out of scope** (T168). |
| Report contents | Schema v1 JSON: counts, classification, unresolved reason codes, content hashes (payload_hash samples), `plan_hash` / `report_hash`, CE honesty (`claims_cryptographic_erasure: false`), rollback (`source_modified: false`). **No plaintext bodies.** |
| Progress | Confirm copy of ≥1000 events emits stderr progress; batch append size 5000. |

**Rollback:** discard destination vault + report; do not point `AI_BRAINS_VAULT_PATH` at the destination until T170 dogfood passes. Live vault is never modified by this command.

### Governed evaluate CLI (T169 / P9.3)

Hermetic **trust-gate** harness over synthetic scenarios (tempfile vaults only). Full metric definitions, catalog, and limitations: [EVALUATION/GOVERNED-MEMORY-MVP.md](EVALUATION/GOVERNED-MEMORY-MVP.md).

```powershell
ai-brains evaluate governed --fixtures fixtures/governed-memory/scenarios
ai-brains evaluate governed --fixtures fixtures/governed-memory/scenarios --report .\evaluate-report.json
```

| Exit | Meaning |
|------|---------|
| **0** | Hard gates passed |
| **1** | Internal / path refuse (tool broke) |
| **6** | Invalid scenario schema/payload |
| **7** | `HARD_GATE_FAILED` (trust regression) or `--strict-soft` quality fail |

Scenario 10 (circularity) runs in `ai-brains-sources` nextest (`runner=sources_tests`); the CP CLI report marks it skipped with that reason when not aggregated.

### Shadow dogfood gate (T170 / P9.4)

Progressive dogfood before any live governed enablement. Full runbook (Stages A–D, D1–D26, compare schema, human checklist): [EVALUATION/SHADOW-DOGFOOD-GATE.md](EVALUATION/SHADOW-DOGFOOD-GATE.md). Checklist template: [EVALUATION/templates/dogfood-human-checklist.md](EVALUATION/templates/dogfood-human-checklist.md).

```powershell
# Orchestrator (WorkDir only; never Stage D; never User-level env)
.\scripts\dogfood-shadow.ps1 -WorkDir C:\temp\ai-brains-dogfood

# Compare CLI (pure JSON in → dogfood-compare.json)
ai-brains dogfood compare `
  --governed .\governed-packet.json `
  --legacy .\legacy-preflight.json `
  --out .\dogfood-compare.json `
  --stage B
```

| Rule | Behavior |
|------|----------|
| Progressive order | A (T169 evaluate exit **0**) → B (fixture vault) → C (redacted shadow) → D (**approval only**) |
| **D26** | Compare with global **`--vault-path <shadow-or-migrated.db>`**. **Never** set `AI_BRAINS_VAULT_PATH` to a shadow/migrated path (breaks live refuse). |
| Flag enable (session only, after Stage D approval) | `$env:AI_BRAINS_GOVERNED_BRIEFING = "1"` |
| Flag rollback (primary) | `$env:AI_BRAINS_GOVERNED_BRIEFING = "0"` or `Remove-Item Env:AI_BRAINS_GOVERNED_BRIEFING` |
| Rollback verify | `preflight --format json` `(governed)` probe + `briefing project --format json` for authority — **never** `preflight --summary` for governed |
| **D24** | Live vault SHA-256 pre/post must match when a live vault exists; locked/unreadable live path is **fail-closed** (not N/A pass) |
| Stage D | Scripts **refuse**; explicit user approval required; observation min 1 session or ≥3 governed invocations |

**Emergency User-env clear (manual only — D23).** Scripts never set User scope. Only if an operator previously set persistent User env by hand:

```powershell
[Environment]::SetEnvironmentVariable("AI_BRAINS_GOVERNED_BRIEFING", $null, "User")
# Open a new shell after User clear so process does not inherit a stale value.
```

**Thin shadow wrapper:** `scripts/shadow-vault.ps1` (forwards to `ai-brains shadow`). Dogfood orchestrator: `scripts/dogfood-shadow.ps1`.

## 4. Project & Session Management

### Project Setup
```powershell
ai-brains context
```
First-init (no `.env`) generates a deterministic `PROJECT_ID` based on your directory and a fresh `SESSION_ID`, storing them in a local `.env` file. Subsequent operations (recall, ingest) automatically use these env values. When `.env` already has a session (and you did not pass `--new-project` / `--new-session`), `context` ensures those `.env` project/session IDs exist in the open vault and does **not** rewrite `.env` (prints `Vault: project and session present.`).

- `--show` — print current context without modifying `.env` and without vault ensure. When pre-dotenv shell `PROJECT_ID` differs from the file, the next line after `Repository:` is `shell leftover PROJECT_ID: <uuid> (.env overrides)`. `AI_BRAINS_KEY` / `VAULT_KEY` file lines print `(redacted)` (T256 `--help` hide stays separate).
- `--new-project` — force a fresh project ID
- `--new-session` — rotate the session ID (useful for long sessions)
- `--tx-id <uuid>` — link the context to a Ledgerful transaction (T37)

### Listing Projects
```powershell
ai-brains project list
ai-brains project list --format json
```
Human table (T212 columns): `label` | `project_id` | `memories` | `last_activity` | `path`. The first data row is the cwd path-owner when that id is registered; remaining rows stay memory-desc. `--format json` keeps T212 size-desc array order (`memory_count DESC, project_id ASC`) — not cwd-first. Star `*` still marks process `AI_BRAINS_PROJECT_ID` (not the sort key).

### Resolving Aliases
```powershell
ai-brains project resolve ai-brains          # exact alias match, falls back to fuzzy
ai-brains project detect --export            # path_alias → git_slug → env PROJECT_ID; export comments include source=
ai-brains project whoami                     # all identity signals (TTY human / piped JSON)
ai-brains project whoami --format json
```

### Identity SOOT (T240)

- **Daily Scope** = effective `AI_BRAINS_PROJECT_ID` after local `.env` force-set (CLI flags / `--global` unchanged). **Never** silently rewritten to path-alias.
- **`project detect` order:** (1) path alias of git toplevel else cwd (2) git slug exact-first (T206; ambiguous exit **1** when no path) (3) env post-dotenv if in vault (4) miss exit **1**. Path owner **always** wins over unique slug hit; stderr notes the slug project.
- **`project whoami`:** shell vs env vs path vs detect + remediations; `--no-project-context` nulls env fields but still resolves path/detect. On mismatch, remediations name `project adopt-path` (print-only / `--write-env --yes`).
- **`project adopt-path` (T258):** print-only remediator for the path-alias owner. `--write-env --yes` rewrites only `AI_BRAINS_PROJECT_ID` in cwd `.env`. Never silent auto-switch. `context` is not the remediator.
- **`project rebind-path` (T259):** print-only remediator that would move **one** path alias to an existing dest. `--write --yes` appends Removed+Added in one transaction. **Does not move memories.** Does not write `.env`. Does not mint dest. Unregister/rebind ≠ forget / CE wipe. **T276:** `--global` recall prefer-fills the current project and pretty-tags leftover vs owner; it does **not** rebind leftover roots. Memories stay until an operator confirms `--write --yes`.
- **Mismatch warn (once/process):** env ≠ path-alias owner → non-fatal stderr + whoami hint. Skip: `--no-project-context`, argv `--global`, no path, empty env.
- **`set-alias` vs `register-path`:** label vs filesystem root — never conflate (see multi-root section below).

## 5. Background Intelligence & Scheduling

### Daemon Lifecycle
The `ai-brainsd` daemon is a single-writer queue that serializes event writes for concurrency safety.
```powershell
ai-brains daemon start             # start in background (console process)
ai-brains daemon status            # IPC liveness (Running|Stopped); no vault key required
ai-brains daemon stop              # graceful shutdown (use --force if it hangs)
ai-brains daemon install           # install as Windows service (requires elevation)
ai-brains daemon uninstall         # remove the Windows service (requires elevation)
```
- **`daemon status` vault independence (T199):** Status answers “is the daemon process / IPC up?” via a single-shot Ping→Pong probe. It does **not** require `--key` / `AI_BRAINS_KEY`, does **not** open or migrate the vault for liveness, and exits **0** for both Running and Stopped. Optional `--vault-path` when Running prints path + size (filesystem metadata only); pinned memory count is attempted only if a key is available — otherwise `Memories: skipped (vault key missing or vault not openable)`.
- **`daemon status` next-step (T249):** When Stopped, last line is `next: ai-brains daemon start`. When Running, `next:` is omitted. No `--format`. Exit **0** both states. Status remains keyless liveness — it does **not** start or stop the daemon and does **not** detect the Windows service.
- **`daemon status` backend Open contrast (T297):** When Stopped and at least one configured LLM/Embedding backend TCP port is Open, the line immediately above `next:` is `backend TCP Open ≠ daemon` (model-process TCP ≠ AI-Brains daemon IPC). Running omits that line. This is **not** the nightly `--status` contrast (`HTTP /health 750ms ≠ daemon TCP`, T281) — status Open remains a TCP connect to the model process, not HTTP `/health`.
- **Governed IPC (T159/T165):** propose/resolve/erasure-ticket/wipe mutations go through the writer queue; scope/briefing/query/inspect are off-queue reads. Spool durable crash recovery for governed mutations **only** when the request includes `command_id` (filename `{op}_{sanitized_command_id}.json`). Briefings over the daemon are dry-run. Ticket erasure returns `accepted` only after a durable `ErasureTicketAccepted` event (still **not** CE). Content-envelope wipe is `WipeContentEnvelope` / HTTP `POST /v1/erasure/wipe` (dry-run default; execute needs `confirm=true`). Principal: wire `principal_id` UUID → System if well-known CLI System UUID, else Human; if wire omitted, `AI_BRAINS_DAEMON_PRINCIPAL_ID`, else System default. CLI always passes resolved `principal_id` on daemon wire (T160). Production policy only (`production_policy`).
- **Loopback HTTP API (T161):** optional authenticated REST surface under `/v1` that reuses the same `handle_daemon_request` path as named-pipe IPC (no second writer). **Default off.** Enable with `AI_BRAINS_HTTP=1` or `ai-brainsd --http`. Binds `127.0.0.1:7432` by default (`AI_BRAINS_HTTP_PORT`). Bearer token lives at `%USERPROFILE%\.ai-brains\http.token` (created on first enable; path logged, token never printed). Owner-only Windows ACL `D:P(A;;FA;;;OW)`. All data routes require `Authorization: Bearer <token>`; `GET /health` and `GET /v1/health` are unauthenticated liveness only (`{"status":"ok"}`). Non-loopback bind requires **both** `--http-bind <addr>` and `AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK=1`. CORS is deny-by-default (no `Access-Control-Allow-Origin: *`). Body limit 1 MiB → HTTP 413. Loopback is **not** zero-trust — always use bearer auth; do not expose without reverse-proxy/mTLS planning (out of scope for v1). Mutations accept optional `X-Command-Id` when the JSON body omits `command_id`. If HTTP is explicitly enabled and bind/token start fails, the daemon **hard-fails** (does not continue as if HTTP were up). Post-spawn serve death is logged only (bind already succeeded). Interactive `ai-brainsd --http` is unchanged by T195.
- **HTTP under Windows LocalSystem service (T195 F10 / R-HTTP-SYS):** When `ai-brainsd` runs as the **Windows service** (`LocalSystem` / Session 0), the service host **refuses to start HTTP** unless `AI_BRAINS_HTTP_SERVICE` is truthy (`1`/`true`/`yes` — same set as `AI_BRAINS_HTTP`). If `AI_BRAINS_HTTP` would enable HTTP but the opt-in is missing, the service **logs a warn, skips HTTP, and continues named-pipe IPC**. When opted in, `%USERPROFILE%` is the **SYSTEM profile**, so `http.token` is SYSTEM-owned with an owner-only ACL — **Interactive desktop/CLI clients cannot read that token.** HTTP under the service host is **not** intended for Session 1 local clients and is **not** claimed “ready for desktop.” Prefer an **interactive** daemon (`ai-brainsd --http` or `ai-brains daemon start` with `AI_BRAINS_HTTP=1`) for desktop clients. A multi-session shared token path is **out of scope**. If HTTP is opted in and bind/token setup fails (or other fatal startup fails), the service **does not report `Running` to SCM** — `StartPending` until failure, then **`Stopped` with `ServiceSpecific(1)`**. Operators should treat `sc start` / Services MMC failures as a real start failure.
- **Unix domain socket path (T195 F7):** Daemon and CLI share `ai_brains_daemon_api::resolve_daemon_socket_path`. Order: absolute `AI_BRAINS_DAEMON_SOCKET` (relative → fail closed) → valid `$XDG_RUNTIME_DIR` (absolute, mode `0700`, owned by euid; **not** created by AI-Brains) → `/tmp/ledgerful-bridge.sock` with a runtime warning. Socket file name is always `ledgerful-bridge.sock`. Post-bind mode **0o600**. Pre-bind and shutdown unlink only **owned sockets** (refuse regular files/dirs/foreign owners). macOS often has no XDG → `/tmp` fallback residual.
- The CLI auto-launches the daemon if it is unreachable, so most users never need `daemon start` explicitly.
- **Windows service (recommended for persistent daemon):** `daemon install` registers `AI-Brains-Daemon` as a Windows service running as `LocalSystem` in Session 0. Default named-pipe SDDL is `D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)` — **Local System + Built-in Administrators + Interactive** (not World/Everyone). That grants Session 1 interactive CLI clients cross-session access while excluding null-session/Everyone. **Residual (R-PIPE-IU):** any interactive logon on a multi-user host can open the default pipe; pipe traffic has no HTTP-style bearer (see SECURITY-LIMITS §7 / ADR-0022). **Opt-in tighter ACL:** set `AI_BRAINS_PIPE_ACL=service-only` in the service environment → SDDL `D:(A;;GA;;;SY)(A;;GA;;;BA)` (no IU). In that mode an **interactive (non-elevated) CLI will typically see `daemon status` → Status: Stopped / NotRunning and cannot open the SYSTEM service pipe even if `sc query AI-Brains-Daemon` reports Running** — use `sc query AI-Brains-Daemon`, an elevated Administrators client, or run an interactive daemon + HTTP+bearer instead (T195 AC11 + T199 F17 honesty). Pipe name remains `\\.\pipe\ledgerful-bridge`. Env vars (vault path, model URLs) are written to `%ProgramData%\AI-Brains\daemon.env` with a restrictive ACL (`SYSTEM:F` + `Administrators:F` only — same model as the nightly SYSTEM wrapper; T145). Requires an elevated PowerShell session.
- **Deprecated:** `daemon schedule` / `unschedule` (Task Scheduler ONLOGON) still work but are deprecated in favor of `install` / `uninstall`. The Task Scheduler approach had a cross-session pipe access issue where Session 0 daemons were unreachable from Session 1.

#### Unix service units (reference only — T196)

**Windows SCM remains the only product-managed service install** (`daemon install` / `AI-Brains-Daemon`). Linux/macOS operators who want a user session daemon can **copy-paste** reference templates from [`packaging/reference/`](../packaging/reference/README.md):

| Template | Role |
|----------|------|
| `packaging/reference/systemd/ai-brainsd.user.service` | Primary Linux: systemd **user** unit |
| `packaging/reference/systemd/ai-brainsd.system.service` | Secondary system unit (honesty; not recommended primary) |
| `packaging/reference/launchd/dev.ledgerful.ai-brainsd.plist` | Primary macOS: LaunchAgent |
| `packaging/reference/launchd/ai-brainsd.wrapper.sh.example` | Secrets via 0600 env + `exec` (never system-wide plist secrets) |
| `packaging/reference/daemon.env.example` | Sample env (absolute `AI_BRAINS_VAULT_PATH`; no real keys) |

These are **reference / operator templates**, not a product Unix installer and **not** T1 multi-OS service parity. Read the packaging README for linger tradeoff, XDG/UDS honesty, KeepAlive/suspend risk, foreground process model, and SIGTERM graceful stop. Soft check: `scripts/check-reference-units.sh`. There is **no** Unix `daemon install` CLI.

### Nightly Intelligence Sweep
```powershell
ai-brains --vault-path ./vault.db nightly
```
The nightly job does:
- **Multi-harness session import (T239):** AGY → Grok → OpenCode (message-only; never opens `opencode.db`). Flags: `--skip-import` (all), `--skip-import-agy`, `--skip-import-grok`, `--skip-import-opencode`. Fail-open per source; `last_multi_import` sync_state + `nightly --status` Multi-import block. Claude/Codex **not** in nightly batch (T253 live + `claude-import` / `codex-import` only). Adapter progress may print non-JSON lines on stderr even when `--log-format json` is set (SYSTEM wrapper).
- Soft model-endpoint probe (T229) after multi-import / before summarize — non-fatal `warn` if completion/embedding endpoints are down
- Summarization of unsummarized sessions (with T34 chunking for sessions over 38,912 tokens)
- Memory synthesis (RAPTOR-style clustering + CRAG factual verification)
- Embedding backfill (UTF-8-safe truncate, T229 F5 — no mid-character panic)
- **Phase 2 multi-root bridge (T233):** MADR + symbol inventory per **registered path alias** (not process cwd)
- MemoryPinned / MemorySynthesized event emission (T67, T68) for the live graph

### Multi-root path aliases (T233)

Nightly **Phase 1** (summarize / embed / synthesize) runs once against the vault. **Phase 2** walks every row in `repository_path_alias_projection` (sorted by normalized path ASC) and, for each existing disk root, runs Ledgerful with **explicit `current_dir(root)`**:

1. `ledgerful bridge export --ledger` → MADR decisions (empty record `project_id` → **alias owner**)
2. `ledgerful symbols --pub --json --limit N --auto-index` → symbol memories (`source_tag=ledgerful:symbol`)

This is independent of Task Scheduler **System32** cwd: roots come from vault path aliases, not from where `schtasks` started the process.

#### `set-alias` vs `register-path` (SOOT — do not conflate)

| Command | What it stores | Used by |
|---------|----------------|---------|
| `project set-alias <uuid> <label>` | Human **label** only | Display, resolve, detect **git slug** name/alias match |
| `project register-path <uuid\|alias> <path>` | **Filesystem root** (normalized Win/WSL) | Detect **step 1**, nightly Phase 2 bridge, whoami path, mismatch warn |
| `project list-paths` | **All** registered roots | Operator inventory (not just `project list` first-path). `--project` / `--shared-only` filter leftover multi-root IDs |
| `project unregister-path <path>` | Compensating **Removed** event | Frees the path for another project; symbols stay |
| `project rebind-path <path> --to <dest>` | Removed (from) + Added (to) in **one tx** | Confirmable per-path split; memories stay on from |
| `project scan-roots [path]` / `--root DIR` | Dry-run `.ledgerful` discovery | Unregistered hits suggest `register-path`; registered suggested is empty; never writes |

Putting a path string into `set-alias` does **not** register a path alias. `project list` **path** column shows a registered path alias when present; it is never invented from cwd/git. Labels like `C:\dev\foo` in the label column are **not** path aliases unless you also ran `register-path`.

```powershell
# Once per repo root (examples)
ai-brains project register-path <id-or-alias> C:\dev\AI-Brains
ai-brains project register-path <id-or-alias> C:\dev\ledgerful
# Dual-checkout optional second form (same project):
# ai-brains project register-path <id-or-alias> /mnt/c/dev/AI-Brains
```

- **Conflict (F21):** the same normalized path can only belong to one project — second owner gets **exit 1** + ownership message naming `ai-brains project unregister-path <path>`. Same project re-register is idempotent OK. Projection **refuses to steal** if a raced other-owner `Added` is applied.
- **Correct a wrong bind:** prefer `project rebind-path <path> --to <dest>` (print-only, then `--write --yes`) so unregister+register cannot be half-applied. The two-step `unregister-path` then `register-path` remains available but is not the happy path. Neither command forgets ingested symbols; Phase 2 simply stops walking the path for the from-project.
- **Discover roots:** `project scan-roots C:\dev` or `project scan-roots --root C:\dev` lists immediate children (plus the scan root) that contain `.ledgerful`. `--root` XOR positional (both set → exit **2**). Default is cwd — not the parent. Dry-run — never registers, never writes `.env`. `.changeguard` leftover dirs are **not** hits. Already-registered rows list the owner and leave `suggested` empty (human `—`). From inside a git worktree, implicit-cwd human output with no unregistered hits may print `next: ai-brains project scan-roots --root <parent-of-toplevel>` so sibling roots under that parent can be scanned.
- **Zero aliases:** Phase 2 is a no-op + stderr hint to run `register-path` (Phase 1 still runs). `project list-paths` prints the empty next-step.
- **Missing root / Ledgerful failure:** per-root warn + continue (non-fatal). Nightly logs `bridge_roots_failed` on symbol ingest error so totals add up.
- **Env caps:** `AI_BRAINS_NIGHTLY_MAX_ROOTS` (optional list truncate); `AI_BRAINS_NIGHTLY_MAX_SYMBOLS` (default **5000**, per-root ingest cap).

#### `ledgerful init` once per root

Path registration alone does not create a Ledgerful ledger. For each root you want symbols from:

```powershell
cd C:\dev\<repo>
ledgerful init   # once per root, when missing
ledgerful symbols --pub --json --limit 5 --auto-index   # smoke
```

Without prior init / usable index, Phase 2 skips that root with a warn (`indexStatus` unusable or empty inventory).

### Local dynamic router (T229)

Overnight brain is designed to talk to a **local** llama.cpp-style router rather than cloud APIs:

| Port | Role | Env (defaults) |
|------|------|----------------|
| **:8081** | Completion / chat | `AI_BRAINS_MODEL_URL` → `http://127.0.0.1:8081`; `AI_BRAINS_COMPLETION_MODEL` |
| **:8083** | Embeddings | `AI_BRAINS_EMBEDDING_URL` → `http://127.0.0.1:8083`; `AI_BRAINS_EMBEDDING_MODEL` |

- **Operator script (this machine):** `c:\llm\router.bat` starts the dynamic multi-model router (completion + embedding servers). A separate Task Scheduler entry **`AI-Brains-Router`** (ONLOGON) can keep the router available after reboot; register it with your ops script (e.g. `register-nightly-tasks.ps1`) — `nightly --schedule` does **not** register the router task.
- **Global dotenv:** put MODEL/EMBED URLs and model names in `%USERPROFILE%\.ai-brains\.env` (merged by T205 before every subcommand). SYSTEM schedule wrappers bake the **process env at schedule time** (including values from that global file).
- **Health:** llama.cpp prefers `GET /health`; if 404, clients try `GET /v1/models`. Nightly **run** pre-summarize probe timeout is **2s** (not the 120s LLM completion timeout). Default `nightly --status` probes run **in parallel** at **750 ms**; `--status --quick` skips HTTP.
- **Logs:** operator wrapper typically appends to `%USERPROFILE%\.ai-brains\nightly-run.log` (this machine’s `nightly-run.cmd`). SYSTEM schedule JSON goes to the Task Scheduler history / wrapper stdout capture under `%ProgramData%\AI-Brains\` when using `--run-as-system`. Prefer `ai-brains nightly --status` for schedule + last result + endpoint probes.

### Dual schedule paths (user multi-import vs SYSTEM skip-import)

| Path | Who | Multi-import | Env / project context |
|------|-----|--------------|------------------------|
| **User-principal** (`nightly --schedule`, no `--run-as-system`) | Logged-in user | **ON** by default (AGY → Grok → OpenCode) | Inherits user env + global dotenv; harness homes readable |
| **SYSTEM** (`nightly --schedule --run-as-system`) | Session 0 | **`--skip-import` baked into wrapper** (T239 D12) | Wrapper bakes vault + model env; no user-profile harness homes |

**Completeness path:** run interactive `ai-brains nightly` or a **user-principal** scheduled task when you need multi-harness import. SYSTEM is for headless summarize/embed when nobody is logged in.

### Scheduling Nightly
```powershell
ai-brains nightly --schedule --start-time "03:00"
ai-brains nightly --status             # schedule + Last Result + endpoints/probe + Multi-import
ai-brains nightly --status --quick     # same, skip HTTP probes (probe=skipped)
ai-brains nightly --status --format json   # machine object (default human; pipes stay human)
ai-brains nightly --unschedule
ai-brains nightly --skip-import        # skip all harness importers
ai-brains nightly --skip-import-opencode
```

`nightly --status` (T247 / T269 / T281) prints additive human lines:

- **Nightly heading** — `Nightly: AI-Brains-Nightly` on every OS, immediately after `=== Nightly Status ===`, so Last Result is not mixed with Router
- **Scheduled** next run — **LIST /V primary** (one `schtasks /FO LIST /V` spawn: next-run + last-run + last-result + Task To Run). CSV next-run is fallback only (3 columns — never Last Result). LIST /V non-zero (task missing) → `Scheduled: No`; no PowerShell.
- **Last task result** — from that LIST /V parse. PowerShell `Get-ScheduledTaskInfo` is Last Result fallback **only when LIST /V succeeded but last_result parse missed (locale)**.
- **Last scheduled run** (Task Scheduler Last Run Time) is printed separately from vault **Last nightly run**. They can disagree (e.g. the task fired but the action target was missing, so the vault never advanced).
- Last nightly run / unsummarized counts / last-run errors
- **Completion** / **Embedding** host:port + model + soft probe (`ok` / `down` / `timeout` / `error` on default `--status`; **`--quick` prints `probe=skipped`** — no HTTP). Human `probe=timeout` is labeled `timeout (750ms)` (HTTP `/health` budget). On Completion human timeout, the next line is `HTTP /health 750ms ≠ daemon TCP`. `daemon status` Open is TCP connect, not `/health`. Credentials in URLs are redacted; vault keys never printed
- **Router** (T255 / T296, read-only) — Status for `AI-Brains-Router`. Human omits scheduler-success decimals (`267014` → `last run: terminated`; `267009` → Status-only). JSON still has raw `last_result` + `SCHED_S_*` hints. Does **not** register, start, or repair that task
- **Multi-import** block (T239)
- Missing action: if Task To Run’s first quoted `.cmd` / `.bat` / `.exe` does not exist → `Action target missing: <path>` + `next: ai-brains nightly --schedule --dry-run`

`--quick` requires `--status`. Without `--status` → clap exit **2**. `--quick` still opens the vault and still prints schedule + last-run.

`--format json` emits a CLI-local machine object. Default `--format` is **human**; piped `nightly --status` stays human. `doctor` is not the model-port matrix; `nightly --status` is.

Default status probes run **in parallel** with a **750 ms** timeout (not sequential 2s+2s). Nightly **run** pre-summarize probe stays **2s**. The 750 ms budget is **not** raised; human timeout is labeled `(750ms)` so it is not read as “backend down.” On Completion human timeout, the next line is `HTTP /health 750ms ≠ daemon TCP` (HTTP `/health` budget ≠ daemon TCP Open). Embedding-only timeout is not this line.

Status **exit 0** when probes are down / timeout / missing action / nonzero Last Result.

#### Last Result **1** vs **101** vs Event ID **101**

These are three different tokens:

| Token | Meaning |
|-------|---------|
| Last Result **1** | Process exit 1 or scheduler “Incorrect function” — missing/bad action, CLI `fail_api`, `.cmd` not found. **Live residual on this class of machine.** |
| Last Result **101** | Child process exit 101 = Rust panic/abort. T229 F5 UTF-8 truncate panic is **cleared**. Do **not** treat Last Result 101 as Task Scheduler Event ID 101. |
| Task Scheduler **Event ID 101** | Operational log: task failed to start (permissions / principal). Different namespace. |

Do **not** wait for the next schedule to “clear 101.” If Last Result is **1** because Task To Run points at a missing `nightly-run.cmd`, the next schedule will fire the same missing path and stay at **1**.

When status prints `Action target missing: <path>` + `next: ai-brains nightly --schedule --dry-run`, that dry-run is **non-mutating**. Product user-principal schedule is `'<exe>' nightly` (not a `.cmd`). Do **not** write `%USERPROFILE%\.ai-brains\nightly-run.cmd` as the product remediator. Recreating a historical ops `.cmd` is out of scope (T255/F14). `--status` does not unschedule, reschedule, or write wrappers.

#### Running the nightly as SYSTEM (`--run-as-system`)
By default `--schedule` registers a task under the current user, which inherits that user's environment variables. The optional `--run-as-system` flag registers the task with `/RU SYSTEM` so it runs without anyone logged in (T132). Because the `SYSTEM` account does **not** inherit User-level environment variables, the CLI handles this specially (T143 + T145):

- It generates a **wrapper `.bat` script** that bakes in the current values of `AI_BRAINS_VAULT_PATH`, `AI_BRAINS_MODEL_URL`, `AI_BRAINS_COMPLETION_MODEL`, `AI_BRAINS_EMBEDDING_URL`, and `AI_BRAINS_EMBEDDING_MODEL` from your environment (or global `%USERPROFILE%\.ai-brains\.env` via T205). The scheduled task runs that wrapper instead of the bare executable, so SYSTEM gets the same config you have.
- **Wrapper location (T145):** `%ProgramData%\AI-Brains\nightly-task.bat` — not the vault parent or `%TEMP%`. Creation refuses symlink/reparse/junction targets at the file path, refuses hardlinks (`nlink > 1`), **and** refuses if the parent directory (e.g. `%ProgramData%\AI-Brains`) exists as a junction/reparse point. Regular single-link existing files may be replaced on re-schedule.
- **ACL (T145):** after write, the CLI applies an **absolute** DACL via Win32 SDDL/`SetNamedSecurityInfo` (`D:P(A;;FA;;;SY)(A;;FA;;;BA)` — protected, SYSTEM + Administrators full only). This replaces the entire DACL so session leftovers (e.g. `LogonSessionId`) cannot remain. The CLI then verifies with `icacls` query (fail closed). Check with:
  ```powershell
  icacls "$env:ProgramData\AI-Brains\nightly-task.bat"
  ```
  Expect only `SYSTEM` and `Administrators` with full control. If ACL apply or verify fails, scheduling aborts — `schtasks /Create` is not called.
- The wrapper appends `--no-project-context --skip-import` to the `ai-brains.exe nightly` invocation. SYSTEM has no user-profile harness homes (AGY/Grok/OpenCode) and no `.env` to auto-discover, so project-context discovery and multi-harness import would both be wrong or empty under Session 0; these flags skip them by default (T239 D12). **Completeness path:** run `nightly` as the interactive user (or user-principal scheduled task) so harness homes are readable.
- `--run-as-system` **requires Administrator rights** (ProgramData ACL + `/RU SYSTEM`). From a normal shell the CLI **prompts for UAC** and re-launches itself elevated (approve the dialog). You can still use an already-elevated PowerShell if you prefer. If UAC is cancelled or disabled, re-run from an Administrator shell. `--dry-run` does not elevate.
- **Residual risk (accepted, T145):** the invoked binary typically lives under `%USERPROFILE%\.cargo\bin\` (user-writable by design for `cargo install`). Copying binaries into `ProgramData` is packaging/installer scope, not done here. The primary hijack vector on the *script* path is closed by the ProgramData + ACL model above. The same residual applies to `ai-brainsd.exe` used by `daemon install` / deprecated `daemon schedule --run-as-system`. `daemon.env` uses the same ACL model under `%ProgramData%\AI-Brains\`.

To preview the registration without writing it, add `--dry-run`:

```powershell
ai-brains nightly --schedule --run-as-system --start-time "03:00" --dry-run
```

`--dry-run` prints the `schtasks` command and the generated wrapper script to stdout without registering the task, so you can verify the baked-in env vars and flags before committing.

> **Migration (T143 + T145):** Existing `AI-Brains-Nightly` SYSTEM tasks may still point at a vault-parent or `%TEMP%` wrapper without restrictive ACL. Re-schedule after T145 to pick up `%ProgramData%\AI-Brains\nightly-task.bat`: `ai-brains nightly --unschedule` then `ai-brains nightly --schedule --run-as-system` from an elevated shell. The same treatment applies to deprecated `daemon schedule --run-as-system` and to re-running `daemon install` for `daemon.env` ACL hardening.

## 6. Memory Hygiene

### Soft-Delete + inventory (T216)
```powershell
ai-brains memory list                         # skim pinned (human prefer-fills authority; JSON recency; default limit 50)
ai-brains memory list --summary               # Pinned + Forgotten counts
ai-brains memory list --status forgotten -l 5
ai-brains forget --memory-id <uuid>           # prompt; -f to skip
ai-brains forget --match "outdated fact" -f   # find by content; -f to forget
ai-brains forget --list-forgotten --limit 5   # soft-deleted rows (bounded; not CE wipe)
ai-brains forget --restore <uuid>             # undo with a compensating event
```
Forgotten memories remain in the event log for audit but are excluded from FTS, graph, and preflight. Soft-forget ≠ CE wipe / NIST Purge. Empty human `forget --list-forgotten` / `memory list --status forgotten` keeps `No forgotten memories.`, then prints `Pinned: N` matching `--summary` and last-line `next: ai-brains memory list` (add `--global` on that next when the list was global).

### Backup
```powershell
ai-brains backup                                # create with timestamped default path
ai-brains backup create --output-dir D:\backups # custom directory
ai-brains backup create --no-prune              # keep full fleet (no default keep-10 prune)
ai-brains backup list
ai-brains backup verify
ai-brains doctor --backup-max-age 7d
ai-brains doctor --summary           # T249: compact 15-check skim
```
Backups include an integrity check; corrupt backups are rejected at creation time. After write, create classifies the file under the current key and deletes a non-usable snapshot (T277) — `Backup created and verified:` means doctor-usable, not merely `integrity_check` ok.

**Recoverability green path (T244 / T277 / T295):** after encrypt, KEY change, or when doctor/list show zero usable encrypted backups (legacy plain wall, Incomplete shells missing `events`/`memory_projection`, wrong key), create a **new** snapshot under the current key in the **default vault sibling `backups/`** (no `--output-dir`) then prove recovery. Custom `--output-dir` is a manual export only — `backup list` (default dir) and doctor `backup_recent` scan the sibling `backups/` directory alone.

```powershell
ai-brains --no-project-context backup create --no-prune   # this vault; keep residuals
ai-brains --no-project-context backup verify              # expect ≥1 OK (exit 1 is OK if residuals FAIL)
ai-brains --no-project-context doctor                     # backup_recent should ok (or age-warn only)
```

After `AI_BRAINS_KEY` change, old `.bak` stay KeyMismatch. Do not transcode. Exhibit: `vault-2026-08-12T15-50-06.db.bak` (T244; unreadable under a later key). Do not treat “backup file exists” or list timestamp as recovery proof — verify must pass core-table checks.

**List honesty (T209 / T244):** `ai-brains backup list` labels residual plain / incomplete / wrong-key / corrupt (`(legacy plain)` / `(no core tables)` / `(unreadable key)` / `(corrupt)`), sorts **usable-first**, warns only on short corrupt files, and prints one residual summary (`not recoverable under current key`); use `--verbose` for per-file detail or `--quiet` to suppress the summary (see CAPABILITIES §11 decision table).

**Verify quiet default (T225 / T244):** `ai-brains backup verify` prints counts + first 5 FAIL reasons (use `--verbose` for the full per-file stream). Both core tables required (`missing core tables`). Doctor `backup_recent` ages only usable encrypted backups (Readable/PreT109 with cores); Incomplete/plain fleets warn + nudge `ai-brains backup create` only.

**Recovery drills (T181):** operator playbook, CE pre-erase honesty, RecoveryKit residual, and automated drill matrix live in [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md). Run restore + content smoke before releases — not “backup exists” alone.

### Restore
```powershell
ai-brains backup restore <path>               # interactive confirm + overwrite
ai-brains backup restore <path> --force       # non-interactive (CI/automation); does NOT override daemon probe
ai-brains backup restore <path> --dry-run     # verify integrity, report, no changes (allowed while daemon up)
```
`--dry-run` runs the integrity check, prints the planned destination, and exits 0 without touching the vault. Use it in scripts before a real restore.

**Restore safety (T188):** mutating restore **hard-fails** (non-zero, no vault overwrite) when a robust IPC probe finds the daemon/service (`timeout ≥1s`, ≥2 retries / 3 total attempts). Message includes `daemon is running` plus both `ai-brains daemon stop` and `sc stop AI-Brains-Daemon`. Dry-run while daemon is up prints a **live restore will fail** notice and still exits 0. Probe residual: detects our named-pipe/UDS only (not third-party lockers).

### Recovery kit export (T188)
```powershell
ai-brains recovery export --output E:\offline\kit.json --passphrase-file $SecurePw
# kit JSON only to file; stdout: path + dpapi: present|absent
# no --passphrase argv; min passphrase 8 bytes; export skips migrate while daemon up
# passphrase-file and kit output (incl. existing parents) refuse reparse/symlink/junction
# preflight: vault+key must match (hard-fail wrong key/missing vault); event soft-fail only when daemon write blocked
```
`ai-brains doctor` is **shipped (T192)** as a read-only health report. Use it for vault open/cipher/backup age / optional kit unlock checks; it does **not** replace RECOVERY-DRILLS or invent a default kit path. **`--summary` (T249)** is the skim path: an opt-in compact view of the same 15-check report (warn+fail attention, or `No issues.`). Default `doctor` stays the full listing. `--summary --json` / `--format json --summary` still emit the full `DoctorReport` (`schema_version=1`).

### DataKey rotation (T189 / ADR-0020)

Operator ceremony to rotate the vault **DataKey** (KEK) and SQLCipher page key together. Closes the wrap-nonce budget residual under ceremony controls (not automatic).

```text
1. ai-brains backup create                 # required by default gate (or verified recent backup)
2. ai-brains daemon stop                   # and service stop if applicable
3. ai-brains vault rotate-datakey --dry-run
4. ai-brains vault rotate-datakey --confirm --kit-output ./kit-new.json --passphrase-file ./pw.txt
5. Unlock-verify NEW kit (passphrase/DPAPI) BEFORE retiring old kits (F32)
6. Update AI_BRAINS_KEY / secrets store / .env to NEW key (CLI prints STALE warning)
7. ai-brains daemon start
8. Verify: capture, recall, open vault with new key
9. Only then: retire/destroy old kit copies under operator policy
```

```powershell
# Prefer export path (default, crash-safe sqlcipher_export)
ai-brains --vault-path $Vault --key $OldKey vault rotate-datakey --dry-run
ai-brains --vault-path $Vault --key $OldKey vault rotate-datakey `
  --confirm --kit-output E:\offline\kit-new.json --passphrase-file $SecurePw `
  --i-have-backup "I have a backup"   # or rely on a ≤24h verified backup in sibling backups/
```

| Rule | Detail |
|------|--------|
| Daemon | Mutating rotate **hard-fails** if daemon is up (same robust probe as restore) |
| Backup gate | Default ON: recent non-empty backup opens with **current** key and mtime ≤24h, or exact phrase `--i-have-backup "I have a backup"` (audited `backup_bypassed`) |
| Kit | `--kit-output` required; **verify unlock of NEW kit before retiring old kits** — if new kit fails, restore from pre-rotation backup with **old** key |
| Method | Primary = export; opt-in `--accept-rekey-risk` for in-place `PRAGMA rekey` (snapshot + auto-restore; mid-crash residual) |
| Multi-device | Each device runs its own ceremony; peer wraps untouched |
| Stale env | Success stdout always includes a **STALE key WARNING** |
| Windows replace | Exclusive source lock held through export/rewrap/verify; released only immediately before `MoveFileEx` (OS cannot replace an open DB). Tiny concurrent-open residual remains — keep daemon stopped. |

Normative: [ADR-0020](DECISIONS/ADR-0020-datakey-rotation.md). Residual honesty: offline backups/old kits remain decryptable under the **old** key until operators destroy them (not NIST Purge).

## 7. Safety & Hotspot Sync

Ledgerful scans the codebase for hotspots (frequently-edited, complex files). The bridge re-pins these as AI-Brains memories so they appear in preflight and recall.

```powershell
ai-brains safety sync                # sync top 5 hotspots
ai-brains safety sync --limit 20     # sync top 20
ai-brains safety sync --dry-run      # preview what would be synced
```

`preflight --pretty` Safety lists those live paths (project-scoped) without pinning. `safety sync` without `--dry-run` writes `HOTSPOT:` pins into the vault.

## 8. Troubleshooting

### `cargo audit` appears to hang
Plain `cargo audit` 0.22.x **exits 0 with no final summary line** on a clean run — the visible output ends with `Scanning Cargo.lock for vulnerabilities …`. This is a CLI behavior change, not a hang. To confirm a clean run:
```powershell
cargo audit --json
# => {"vulnerabilities":{"found":false,"count":0,"list":[]}, ...}
```
See [ci-tooling.md](ci-tooling.md#behavior-notes) for more.

### `init` refuses on a populated vault
Caused by T73's safety gate. The vault at the given path already contains projects. Re-run with `--force` to acknowledge the overwrite:
```powershell
ai-brains init --force
```

### `daemon schedule` reports "Access is denied"
The Task Scheduler registration requires elevation. Open PowerShell **as Administrator** and retry. The CLI prints the exact `schtasks` command it tried to run, which you can also paste manually. **Note:** `daemon schedule` is deprecated — use `ai-brains daemon install` for a proper Windows service.

### `Failed to create named pipe instance: Access is denied (os error 5)`
This means another `ai-brainsd` instance already owns the `\\.\pipe\ledgerful-bridge` pipe and the security descriptor denies your user access. This is the cross-session pipe issue (T144). The fix is to:
1. Stop any existing daemon: `ai-brains daemon stop --force`
2. Install as a service: `ai-brains daemon install` (from elevated PowerShell)
3. The service creates the pipe with SDDL `D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)` (SYSTEM + Administrators + Interactive) so Session 1 interactive clients can connect — not World/Everyone (see SECURITY-LIMITS §7).

If you see this on a second manual launch, the daemon now detects the existing instance and exits cleanly with "Daemon already running" instead of looping.

### Ledgerful bridge is off by default
Ledgerful (engine tracks 0064/0065) renamed the IPC pipe to `\\.\pipe\ledgerful-bridge` and made the bridge **opt-in**. AI-Brains listens on that pipe. Explicit `ledgerful bridge export` / `import` still work without enabling the bridge (pure-local I/O). Implicit push paths (`verify` outcomes, `watch` risk alerts, `ask` enrichment, `bridge query` IPC) require either:

```toml
# .ledgerful/config.toml
[bridge]
enabled = true
provider_command = "ai-brains"
```

or for the current shell:

```powershell
$env:LEDGERFUL_BRIDGE = "1"
```

### Recalls return only code files, not session memories
This is correct FTS5 behavior when no session context has been pinned. After a few ingest+recall cycles, the relevant session memory will surface. The `safety sync` command intentionally pins file paths as memories so the same query can return both kinds of result.

### Graph health check (T213 density honesty)
```powershell
# Requires graph-on binary (`cargo install --path crates/ai-brains-cli --locked --features graph`)
ai-brains graph update
```
Success JSON shape (pretty):

```text
{
  "nodes": …,
  "edges": …,
  "pinned_memories": …,
  "memory_nodes": …,          // kind = 'memory' only
  "edge_node_ratio": …,       // edges/nodes (0.0 if nodes==0); fraction, not NetworkX density
  "density": "ok|warn|skip",  // never "fail" — query errors use the error path
  "status": "live|sparse|empty",
  "note": "…human one-liner…",
  "remediation": "ai-brains graph rebuild"   // omitted when null
}
```

| `status` | Meaning |
|----------|---------|
| `live` | Density assessor Ok (or small empty vault still reporting live with `density=skip`) |
| `sparse` | Under-linked (E/N below typed-lineage floor **0.50**), orphan nodes (many nodes, zero edges), or severe memory projection lag |
| `empty` | Many pinned memories but graph tables empty (empty lag) |

**When to rebuild (T232 / T308 — capability-aware):** if doctor/`graph update` `density` is `warn`, pick the next action that matches **this binary** and the **primary verdict**:

| Capability + verdict | Primary remediation |
|----------------------|---------------------|
| Graph-on **empty_lag / orphan_nodes / projection_lag** | `ai-brains graph rebuild` |
| Graph-on **sparse** (typed-lineage E/N below floor **0.50**) | **Omit** remediator (T308) — note may still say `rebuild if projection lag suspected`; a second rebuild will not raise typed E/N |
| Graph-off any density warn (including sparse) | Install a graph-capable binary first: `cargo install --path crates/ai-brains-cli --locked --features graph` (`GRAPH_REINSTALL_SOOT`) — rebuild is a dead-end on graph-off |

```powershell
# Graph-on only — prefer dry-run first (allowed while daemon is Running):
ai-brains graph rebuild --dry-run
# Mutating rebuild: stop the daemon first (LiveGraphHook races DELETE+replay).
ai-brains daemon stop
# or: sc stop AI-Brains-Daemon
ai-brains graph rebuild
# Optional: same keys as update for scripts
ai-brains graph rebuild --format json
```

Mutating rebuild fail-closes with exit **1** while the daemon is up (message names `ai-brains daemon stop` / `sc stop AI-Brains-Daemon`). Success prints the density report on stdout (human default) and exits **0** even when still `sparse` — rebuild replays the same projector; typed-lineage floor **0.50** is not retuned. `--dry-run` never calls `GraphRebuilder`; JSON dry-run is the health object only (no `[dry-run]` line).

Do **not** treat non-zero `nodes` alone as healthy — live dogfood historically showed ~1300 nodes / ~95 edges (`E/N ≈ 0.07`) while still reporting `live` before T213.

**Graph-off lag:** default / GitHub Release binaries (no `--features graph`) never run the incremental LiveGraphHook. `graph update` exits **2** (`FEATURE_UNAVAILABLE`). Use doctor for capability + SQL density on any binary:

```powershell
ai-brains doctor --summary           # skim (human-only; warn+fail or No issues.)
ai-brains doctor --format json
# --summary --json / --format json --summary still emit full DoctorReport
# checks include:
#   name=graph_feature  (soft info: available|unavailable via compile-time cfg; never alone fail/degraded)
#   name=graph_density  (soft warn → overall degraded; never hard-fail alone;
#                        remediator: graph-on empty/orphan/projection_lag → rebuild;
#                        graph-on sparse → omit (T308); graph-off → GRAPH_REINSTALL_SOOT)
```

**Local graph-on rebuild:** `scripts/Build-AIBrains.ps1` and `scripts/build.ps1` build CLI with `--features graph` and probe `graph_feature=available` before finishing (T222). Primary source install SOOT remains `cargo install --path crates/ai-brains-cli --locked --features graph`. Cargo `default = []` is unchanged (slim / Release may stay graph-off).

Thresholds (soft env, invalid→default): `AI_BRAINS_GRAPH_MIN_PINNED` (100), `AI_BRAINS_GRAPH_MIN_NODES` (50), `AI_BRAINS_GRAPH_MIN_EDGE_RATIO` (0.50), `AI_BRAINS_GRAPH_MIN_MEMORY_COVERAGE` (0.10 severe floor).

**Reading neighbors / hierarchy / session (T246 / T262 / T278 / T293):** on a TTY (or `--format pretty` / `human` / `text`) these print a human table; piped stdout or `--format json` stays **compact** JSON for scripts. Session-kind neighbor PREVIEW is `{n} memories · first line` (human-only; JSON keys unchanged). T293: human neighbors prefer-fills authority 1-hop (`DECISION:` / `CONSTRAINT:` / `INVARIANT:` / `HOTSPOT:` memory or session caption) ahead of dump sessions; JSON array order stays direction→label→id. `graph update` is a health check — it does **not** create nodes and is not a rebuild. `graph rebuild` is recovery replay; it cannot invent a `turn_id` that was never logged on old events. After T262, `pin` prints the ingest turn id, which **is** `memory_projection.memory_id` and the graph memory node (`RECALLS` to the session) without a rebuild. Missing-node pretty says `next: ai-brains graph rebuild` **only** when that id exists in the vault; unknown ids get `(not a vault memory id)` and no remediator. Honest leaf / no-neighbors / empty-session print no `next:`. T246 JSON keys stay frozen. `graph update --format human` is labeled lines; default JSON is unchanged.

**Cozo init quiet by default (T208):** graph-on CLI paths construct the Cozo proxy but do **not** print `CozoProxyBackend initialized` under the product default log filter. To see lifecycle/debug for the graph crate only:

```powershell
$env:RUST_LOG = 'ai_brains_graph=debug'   # =info is not enough after demote
```

`--log-format off` silences all tracing. Unset `RUST_LOG` (do not set it to empty string) to restore product defaults.

### Vault Locked / Missing Key (T197)
| Symptom | Cause | Operator action |
|---------|--------|-----------------|
| `Vault key missing:` / JSON `VAULT_KEY_MISSING` | Neither `--key` nor `AI_BRAINS_KEY` after trim | Set key (see [INSTALL.md](INSTALL.md) bootstrap) or re-run `ai-brains init` to generate |
| `Vault key invalid format:` / `VAULT_KEY_FORMAT` | Not product form `x'<64 hex>'` | Fix quoting; bare 64-hex is **not** auto-wrapped |
| `Vault key refused:` / `VAULT_KEY_ZERO` | Explicit all-zero without allow | Use a non-zero key; tests may set `AI_BRAINS_ALLOW_ZERO_KEY=1` |
| `Vault locked:` / `VAULT_LOCKED` | Wrong key or cannot decrypt | Fix key material; do not expect multi-line native hmac dumps |
| Doctor `vault_open` **skipped** | Key missing | Report still emits; exit 1 |
| Doctor `vault_open` **fail** | Wrong key | Report emits; exit 1 |

CLI dotenv: project `.env` when `--no-project-context` is unset, then **always** merge user-global `~/.ai-brains/.env` for **gaps** only (non-override; still under `--no-project-context`). Prefer KEY + path in the global file. Never commit keys.

### Missing Graph Database
If the graph features are missing on Windows, verify that the `graph` feature was enabled during build and that the MSVC 4GB image size limit was not exceeded. If it was, the system will gracefully fall back to Lexical search.

## 9. Environment Variables

| Variable | Description |
|---|---|
| `AI_BRAINS_VAULT_PATH` | Default path to the vault database. |
| `AI_BRAINS_KEY` | SQLCipher vault key as product form `x'<64 hex>'` (67 chars; T187/T197). Required for vault-backed commands when `--key` omitted. Missing → `VAULT_KEY_MISSING` (not silent zero). CLI gap-fill: shell env > project `.env` > always-merge global `~/.ai-brains/.env` (non-override; still under `--no-project-context`). Never commit. |
| `AI_BRAINS_ALLOW_ZERO_KEY` | When `1`/`true`/`yes` (case-insensitive), allow all-zero SQLCipher keys (hermetic tests / legacy dogfood only). Production should omit. Explicit zero without this → `VAULT_KEY_ZERO`. |
| `AI_BRAINS_VAULT_KEY` | **Daemon only** (`ai-brainsd`): vault key env name used by the daemon process (not the CLI resolver). Prefer documenting daemon secrets in a 0600 env file; do not conflate with CLI `AI_BRAINS_KEY` without ensuring both are set when CLI and daemon share a vault. |
| `AI_BRAINS_PROJECT_ID` | Default `project_id` for capture/recall (set by `ai-brains context`). Local `.env` force-sets this over a different shell value (T80/T223). |
| `AI_BRAINS_SESSION_ID` | Default `session_id` (set by `ai-brains context`). Local `.env` force-sets this over a different shell value; session-only override is demoted to debug (no stderr). |
| `AI_BRAINS_QUIET_ENV_WARN` | When `1`/`true`/`yes` (case-insensitive, trim), suppress stderr for local `.env` project-context ID override warnings (collapsed debug only). **Must be in shell env or project `.env`** — global `~/.ai-brains/.env` alone does **not** work (global loads after the warning is emitted). **Quiet wins over** `AI_BRAINS_FORCE_ENV_WARN`. Does not affect T206 `git/env project mismatch` or T240 identity mismatch. |
| `AI_BRAINS_FORCE_ENV_WARN` | When `1`/`true`/`yes` (case-insensitive, trim), always emit the project-context override stderr line even when a T242 session marker already exists for the fingerprint. **Quiet wins over force.** Does not affect session-only demote (still debug). |
| Session override-warn markers (T242) | Cross-process suppress after first project-differ warn: empty files under `%USERPROFILE%\.ai-brains\cache\env-override-warn\` (fingerprint hex names). Re-warn on fingerprint change. Manual reset: `Remove-Item -Recurse -Force "$env:USERPROFILE\.ai-brains\cache\env-override-warn" -ErrorAction SilentlyContinue`. No auto-TTL. Never written into the git worktree. |
| `LEDGERFUL_TX_ID` | Ledgerful transaction ID for ledger cross-linking (preferred; T142). |
| `CHANGEGUARD_TX_ID` | Deprecated alias for `LEDGERFUL_TX_ID` (warns and falls back; T142). |
| `AI_BRAINS_MODEL_URL` | Endpoint for the local LLM completion server (default: `http://127.0.0.1:8081`). |
| `AI_BRAINS_EMBEDDING_URL` | Endpoint for the local embedding server (default: `http://127.0.0.1:8083`). |
| `AI_BRAINS_EMBEDDING_MODEL` | Name of the embedding model (default: `nomic-embed-text-v1.5`). |
| `AI_BRAINS_COMPLETION_MODEL` | Name of the completion model (default: `gemma-4-E4B-it-Q6_K.gguf`). |
| `AI_BRAINS_SCOPE` | Comma-separated paths for preflight contextual risk analysis. |
| `AI_BRAINS_GOVERNED_BRIEFING` | When `1`/`true`/`yes`, `preflight` uses typed Project briefing (policy + authority). Default off. |
| `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` | UUID principal for governed preflight / briefing / query CLI grant checks. |
| `AI_BRAINS_HTTP` | When `1`/`true`/`yes`, enable in-daemon loopback HTTP `/v1` (T161). Default off. Also `ai-brainsd --http`. Interactive path only for desktop clients. |
| `AI_BRAINS_HTTP_PORT` | HTTP listen port (default `7432`). Bind remains `127.0.0.1` unless non-loopback double opt-in. |
| `AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK` | Must be `1` **and** `--http-bind` explicit to bind non-loopback (dangerous; not recommended). |
| `AI_BRAINS_HTTP_SERVICE` | **Windows service only (T195).** When `1`/`true`/`yes`, allow service host to start HTTP if `AI_BRAINS_HTTP`/`--http` would enable it. Unset/false → service **skips HTTP**, logs warn, continues IPC. Token under SYSTEM profile is not for Session 1 desktop clients. |
| `AI_BRAINS_PIPE_ACL` | **Windows pipe (T195).** `interactive` (default/unset) = SDDL SY+BA+IU; `service-only` = SY+BA (no IU). Unknown values fail closed. **service-only honesty:** interactive CLI may see NotRunning against a healthy SYSTEM service pipe — use `sc query AI-Brains-Daemon`, elevated BA, or interactive daemon + HTTP. |
| `AI_BRAINS_DAEMON_SOCKET` | **Unix UDS (T195).** Absolute path override for daemon bind **and** CLI connect (must match). Relative paths fail closed. When unset: valid `$XDG_RUNTIME_DIR/ledgerful-bridge.sock`, else `/tmp/ledgerful-bridge.sock` + warn. Prior `/tmp`-hardcoded external clients need this set on both sides if the daemon uses XDG. |

## 10. Command Summary

| Action | Command |
|---|---|
| Initialize Vault | `ai-brains init` (use `--force` to overwrite populated vault) |
| Show Context | `ai-brains context --show` |
| Sync Safety Signals | `ai-brains safety sync` (use `--dry-run` to preview) |
| Unified Search | `ai-brains sync query "<topic>"` (searches vault + Ledgerful). Dash needles: `sync query -- --limit` |
| Get Orientation | `ai-brains preflight` (use `--pretty` for full text, `--summary` for stats) |
| Typed Project/Personal Briefing | `ai-brains briefing project\|personal` (TTY markdown / non-TTY json; T227 aliases human\|pretty\|text\|md → markdown; unknown format exit 2; see T152/T202/T227) |
| Progressive Query / Expand / Trace | `ai-brains query progressive\|expand\|trace` (progressive/expand require project id; missing → exit **2**) |
| Scope / Evidence / Source / Review | `ai-brains scope resolve` · `evidence list\|search\|show` · `source list\|show` · `review list\|resolve` (T160/T203) |
| Propose Conclusion / Decision | `ai-brains conclusion propose` · `decision propose` (daemon prefer; `--local` OK) |
| In-force decision (term) | `ai-brains decision in-force <TERM>` (local projection; default JSON; `ReadDecisions`) |
| Erasure ticket (daemon-only) | `ai-brains erasure request --id … --scope …` (no CE wipe claim) |
| Policy show / check / bootstrap | `ai-brains policy show\|check` (read-only); `policy bootstrap` (discovery grants, T210) |
| Deep Search | `ai-brains recall` (use `--format pretty` for readable results) |
| Pinned Record | `ai-brains pin` (use `--tag` for categories, `--stdin` piped) |
| Forget Memory | `ai-brains forget` (use `--match` for search, `--restore` undo, `-f` to skip confirm) |
| Antigravity Capture Hook | `ai-brains agy-hook --payload "{...}"` (used by agy CLI hooks) |
| Import Antigravity | `ai-brains antigravity-import --days 30` (incremental scan) |
| Grok Capture Hook | `ai-brains grok-hook --payload "{...}"` (Stop/SessionEnd wrapper) |
| Import Grok | `ai-brains grok-import --days 30` (chat_history scan; never updates.jsonl) |
| OpenCode Capture Hook | `ai-brains opencode-hook --payload "{...}"` (plugin session.idle) |
| Import OpenCode | `ai-brains opencode-import --days 7` (list+export; never opencode.db) |
| Nightly Sweep | `ai-brains nightly` (summarization + graph + bridge) |
| Schedule Nightly | `ai-brains nightly --schedule --start-time "03:00"` |
| Daemon Control | `ai-brains daemon start/status/stop/schedule/unschedule` |
| Backup Vault | `ai-brains backup` |
| Restore Vault | `ai-brains backup restore <path>` (use `--force` non-interactive, `--dry-run` to preview; hard-fails if daemon up — T188) |
| Recovery kit export | `ai-brains recovery export --output <path> [--passphrase-file] [--dry-run] [--force]` (T188) |
| Doctor (health) | `ai-brains doctor [--summary] [--json] [--kit-path] [--passphrase-file] [--fail-on-degraded] [--backup-max-age 7d] [--full]` (T192/T249; `--summary` skim; JSON still full report) |
| Manage Projects | `ai-brains project list/resolve/detect` |
| Graph Health | `ai-brains graph update` (`live`\|`sparse`\|`empty`; graph-on rebuild for empty/orphan/projection_lag; sparse omits remediator — T308) + doctor `graph_density` (capability-aware remediation — T232/T308) |

## Desktop thin client (T172 + T173 security)

Operator notes for the Tauri desktop adapter. Deep dive, architecture diagrams, and residual detail live in [apps/desktop/README.md](../apps/desktop/README.md).

### Runtime (T172)

- **Invoke-first:** UI never uses webview `fetch` to loopback `/v1`; Rust holds the user-session bearer.
- **Prereqs for live screens:** daemon on `AI_BRAINS_HTTP_PORT` (default 7432) + `%USERPROFILE%\.ai-brains\http.token`.
- **Offline/denied:** paint promptly (QueryClient `retry: false`); no fake full-grant empty states.
- **Unavailable by design:** connectors UI, retention plan UI, grants inventory.
- **Single-instance:** second launch focuses/unminimizes the existing `main` window (plugin registered first).

### Security locks (T173 / SU15)

- **Dual-layer open:** external open is **Rust-only** via `open_url` / `reveal_path`. Effective gates on the custom-command path: **Layer 1 validators** (https-only; path empty/`..` refuse) + **Layer 2 capability-mirror** (independent allowlist matching `https://*` / path globs; kept in sync with `capabilities/default.json`). Scoped capability objects also constrain plugin IPC if invoked. Then `tauri-plugin-opener` via `OpenerExt`. **Forbidden:** `opener:default`, `opener:allow-default-urls`, bare unscoped `opener:allow-open-path`, and the JS package `@tauri-apps/plugin-opener` (never in FE `package.json`).
- **Isolation Pattern (mandated):** classic single-file isolation app (`apps/desktop/isolation/`). The isolation hook is **hygiene/audit only** — pass-through; it **cannot claim denylist** enforcement (C13 residual).
- **CSP prod vs `devCsp`:** production `app.security.csp` is strict (IPC + Isolation `frame-src 'self' customprotocol: asset:`; no `unsafe-inline` / `unsafe-eval` / HMR hosts). `devCsp` relaxes for Vite localhost only — **never ship `devCsp` as prod**.
- **Typed WIPE:** execute wipe requires typing exact phrase **`WIPE`** in the confirm dialog (checkbox confirm removed). Dry-run does not require the phrase. Enter focuses Confirm (no auto-submit); Escape cancels.
- **Focus a11y:** `:focus-visible` outlines on interactive controls; `scroll-padding-top` so focus is not hidden under the sticky topbar.
- **No analytics / crash phone-home by default:** no Sentry, PostHog, or similar in the production tree. Opt-in would need an ADR + track.


## Multi-device sync residuals

> **Scope:** Optional multi-device encrypted event replication (ADR-0018 / Phase 11 fake-relay path). Local-only remains the default. This section is the canonical home for residual honesty claims (T178 F26).

### What multi-device sync does **not** claim

- **Not a ZK relay.** The fake relay stores opaque `wire_v1` bodies plus public routing metadata (device ids, sequence numbers, content-type codes, sizes). Operators and a compromised relay can observe the **device graph**, envelope **counts**, **sizes**, and **timing** of put/pull. This is **not** a metadata-private channel.
- **Not post-quantum.** Device keys and envelope crypto are classical (Ed25519, X25519, HKDF-SHA256, AES-256-GCM). There is **no** post-quantum KEM/signature in the protocol; do not market the path as PQ-secure or post-quantum-ready.
- **Not NIST SP 800-88 Purge / remote wipe.** Content Erasure (CE) is best-effort multi-device destroy of content-key wraps with peer **ErasureAck** rows. An ACK is a **signed attestation** that a peer applied the tombstone locally — **not wipe proof** of peer disks, backups, or offline devices. Do not claim multi-device NIST Purge as a product property.
- **Padding is not metadata-private.** Size-bucket padding (`PAD_BUCKETS` 256 / 4096 / 65536) is best-effort traffic shaping only. It does **not** make the relay metadata-private; sizes, counts, timing, and the enrolled device graph remain observable (**pad is not metadata-private**).
- **DataKey rotation is local-ceremony only (T189 / ADR-0020).** Per-seal content-DEK wraps improve multi-device nonce budget relative to a single long-lived content key, but they **do not** rotate each peer’s vault DataKey. **Each enrolled device** must run its own `ai-brains vault rotate-datakey` ceremony. Peer content wraps (`peer_content_key_wrap`) are **not** mutated by local rotation.

### Operational residuals operators should expect

| Residual | Operator expectation |
|----------|----------------------|
| Metadata leakage | Relay sees sizes, counts, timing, device graph, content-type codes |
| ACK attestation residual | `acked` / `failed` / `unreachable` are peer attestations, not wipe proof |
| Offline CE lag | Offline peers stay `pending` then may become `unreachable` after ACK timeout cycles; not silent full wipe |
| Classical crypto | Ed25519 / X25519 / AES-GCM — not PQ |
| Gap / reorder | Sync gaps buffer until fill or signed `GapSkipAudit`; no corrupt apply past gap |
| Capture independence | `ai-brains-capture` has **no** dependency on `ai-brains-sync` |

**Discoverability:** `ai-brains device status` prints the enrolled roster (same as `device list`), names this machine (`{hostname} (not enrolled)` or hyphen fingerprint), prints short honesty `local-only; not PQ; not remote wipe`, and always ends with `next: ai-brains replicate status`. Human `replicate status` adds the same this-machine label after `enrolled_count` (JSON keys unchanged).

### Related docs

- [ADR-0018](DECISIONS/ADR-0018-encrypted-event-replication-protocol.md) — protocol normative
- Threat model §7 (T175) — claim matrix L1–L16 / residuals
- T178 security suite — executable acceptance gates for the claims above
