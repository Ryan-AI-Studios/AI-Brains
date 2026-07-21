# AI-Brains — Capabilities & Features

**Version:** 0.1.1  
**Platform:** Windows 11 first (PowerShell); Ubuntu/WSL best-effort  
**Type:** Local-first CLI + optional local daemon (not an MCP server)  
**Related docs:** [OPERATIONS.md](OPERATIONS.md) · [WORKFLOWS.md](WORKFLOWS.md) · [PRD.md](PRD.md) · [status.md](status.md)

---

## 1. Product thesis

AI-Brains is a **durable, project-aware memory layer for AI coding harnesses**. It addresses harness amnesia, parallel-agent isolation, review-agent blindness, and IDE log loss by storing what matters:

```text
User: Do X
AI final response: I did X
```

**Deliberately excluded:** hidden chain-of-thought, tool/action sludge, raw intermediate tool logs.

| Mode | Purpose | When | Works offline (no models/graph)? |
|------|---------|------|----------------------------------|
| **Capture** | Append clean conversation events | Immediate | **Yes** (hard requirement) |
| **Brain** | Summarize, embed, graph, synthesize, inject | Scheduled / on demand | No |

---

## 2. Architectural pillars

| Pillar | Behavior |
|--------|----------|
| **Capture independence** | CLI → daemon → event log works without models, embeddings, or graph DBs |
| **Canonical source of truth** | Every state change is an immutable event in a SQLCipher append-only log |
| **CQRS** | Commands append events; queries read projections only |
| **Capture privacy** | Only user prompts + final assistant responses (no CoT / tool logs) |
| **Privacy inheritance** | Derived memories inherit the strictest privacy of sources |
| **Event sourcing** | No update/delete of raw events; corrections via compensating events |
| **No repo pollution** | Memory defaults to user storage; projects get `.env` IDs only |
| **Path normalization** | Windows drive-case, UNC, WSL `/mnt/c` mappings normalized |
| **Relational graph** | Native SQLite backend (recursive CTEs); Cozo bridge optional/feature-gated |
| **Licensing** | PolyForm Noncommercial 1.0.0 + Small-Entity Commercial Exception; deps stay permissive (see `deny.toml`) |

**Workspace crates:** `core` · `events` · `contracts` · `store` · `crypto` · `path` · `capture` · `retrieval` · `graph` · `models` · `brain` · `scheduler` · `ai-brainsd` · `ai-brains-cli`

---

## 3. CLI surface

```text
init | ingest | recall | preflight | nightly | backup | forget | stop-session
context | pin | safety | sync | antigravity-import | agy-hook | daemon | project | graph
```

**Global options:**

| Flag / env | Purpose |
|------------|---------|
| `--vault-path` / `AI_BRAINS_VAULT_PATH` | Vault database path |
| `--key` / `AI_BRAINS_KEY` | SQLCipher key |
| `--no-project-context` | CI/hooks: do not load project `.env` or clobber inherited IDs |
| `--log-format` | `compact` \| `full` \| `json` \| `minimal` \| `off` |

Failures emit structured JSON error envelopes on stderr.

---

## 4. Capture & ingest

### Manual / programmatic
- **`ingest`** — JSON turn from stdin (`session_id`, `project_id`, `harness_id`, `turn_id`, `role`, `content`, `privacy`)
- **`--dry-run`** — preview without write (relaxed validation on dry-run path)

### Harness integrations

| Integration | Mechanism | Notes |
|-------------|-----------|--------|
| **agy (Antigravity CLI)** | `agy-hook --payload '{...}'` | Real-time; `--schema` prints JSON Schema |
| **Antigravity bulk** | `antigravity-import --days N` | Incremental, idempotent; filters tool/CoT noise |
| **Claude / Codex / Gemini / etc.** | Hooks/scripts → `ingest` | Multi-harness design |
| **Claude hooks** | `Docs/claude-hooks.md` | User-level scripts under `~\.ai-brains\scripts\` |

### Daemon write path
- **`ai-brainsd`** — single-writer queue for concurrent safety
- **Auto-launch** — CLI spawns daemon when the pipe is unreachable
- **Named pipe:** `\\.\pipe\ledgerful-bridge` (aligned with Ledgerful 0064)
- **Windows service:** `daemon install` / `uninstall` (LocalSystem Session 0 + SDDL cross-session access)
- **Deprecated:** `daemon schedule` / `unschedule` (Task Scheduler logon)
- Lifecycle: `start` · `status` · `stop [--force]` · `update`

Most users never need an explicit start: the CLI auto-launches. A Windows service is optional for always-on Session 0 operation.

---

## 5. Project, session & context

| Capability | Command / detail |
|------------|------------------|
| Init project context | `context` — writes local `.env` (`PROJECT_ID`, `SESSION_ID`, `HARNESS_ID`) |
| Show only | `context --show` |
| Rotate project / session | `--new-project` · `--new-session` |
| Ledger linkage | `--tx-id` / `LEDGERFUL_TX_ID` (legacy `CHANGEGUARD_TX_ID` fallback) |
| List projects | `project list` |
| Aliases | `project set-alias` · `project resolve` |
| Auto-detect | `project detect` (git / `.ledgerful` / `.env`) |
| Stop session | `stop-session` |
| Env precedence | Shell env > project `.env` > global `~\.ai-brains\.env` |

Discovery prefers **`.ledgerful/`**, falls back to legacy **`.changeguard/`**.

---

## 6. Dense memory APIs

### Pin
```powershell
ai-brains pin "DECISION: …" --tag architecture
ai-brains pin --stdin --role user --privacy LocalOnly --dry-run
```
- Roles, privacy, tags, tx-id linkage
- Prints projection `memory_id` for later forget
- Emits **`MemoryPinned`** events for live graph edges

### Forget / restore (soft delete)
```powershell
ai-brains forget --memory-id <uuid> -f
ai-brains forget --match "outdated" -f
ai-brains forget --list-forgotten
ai-brains forget --restore <uuid>
ai-brains forget --dry-run …
```
Forgotten items remain in the event log (audit) but drop from FTS / graph / preflight.

---

## 7. Retrieval & orientation

### Recall
```powershell
ai-brains recall "auth flow" --limit 5
ai-brains recall "login" --semantic --graph-boost 0.1
ai-brains recall "query" --global --no-bridge --quiet
ai-brains recall -   # query from stdin
```

| Feature | Detail |
|---------|--------|
| **FTS5** | Default lexical path; sanitized queries |
| **Semantic** | `--semantic` + stored embeddings |
| **Graph boost** | Neighbor score boost (`--graph-boost`) |
| **Substring fallback** | When FTS empty on small vaults |
| **Scope** | Project default; `--global`; `--session` / `--session-prefix` / `--session-last` |
| **Bridge mix** | Ledgerful hits capped so vault memories still surface; `--no-bridge` |
| **Formats** | Pretty on TTY by default; JSON / NDJSON; per-result `session_id` |
| **Hints** | Contextual no-results hints on stdout |

### Preflight (session-start briefing)
```powershell
ai-brains preflight --summary
ai-brains preflight --pretty -m 1500
ai-brains preflight --scope "src/foo.rs" --global
ai-brains preflight --stdin
```
Synthesizes repo safety/hotspots, session turns, memory index, recent dense memories, under a word budget (default 1500). Index titles use Unicode-safe truncation.

### Unified vault + ledger search
```powershell
ai-brains sync query "rust" --format pretty
ai-brains sync query "term" --no-bridge --global --quiet
```

---

## 8. Nightly intelligence (“Brain mode”)

```powershell
ai-brains nightly
ai-brains nightly --status
ai-brains nightly --skip-import
ai-brains nightly --schedule --start-time "03:00"
ai-brains nightly --schedule --run-as-system --dry-run
```

Pipeline includes:
1. Optional Antigravity import  
2. Session summarization (chunked; **38,912-token** context with carryover)  
3. Memory synthesis (batch-limited, e.g. 50 memories/run)  
4. Embedding backfill + stale refresh + WAL checkpoint  
5. Ledgerful **symbol bridge** ingest (functions, routes → code-aware recall)  
6. **`MemorySynthesized`** events for graph edges  
7. Live graph projection updates  

SYSTEM-mode schedules bake vault/model env into a wrapper script so Session 0 has config.

---

## 9. Graph

Requires **`--features graph`** for full backend; default builds may stub with an install hint.

| Command | Purpose |
|---------|---------|
| `graph update` | Health: nodes, edges, live status |
| `graph rebuild` | Full resync (recovery) |
| `graph neighbors <memory_id>` | 1-hop neighbors |
| `graph hierarchy <memory_id>` | Synthesis chain |
| `graph session <session_id>` | Memories in a session |

**Live Graph Hook:** incremental projection on each event append.

---

## 10. Ledgerful integration

| Feature | AI-Brains side |
|---------|----------------|
| Binary | Shells out to `ledgerful` |
| State dirs | `.ledgerful/` preferred; `.changeguard/` legacy |
| Hotspot pin | `safety sync [--limit N] [--dry-run]` |
| Symbol bridge | Nightly ingest → recall returns code structure |
| Unified query | `sync query` |
| NDJSON pull/push | `sync pull` / `sync push` |
| IPC pipe | `\\.\pipe\ledgerful-bridge` |
| Opt-in bridge | Ledgerful default **off**; enable `[bridge] enabled=true` or `LEDGERFUL_BRIDGE=1` |

Explicit `ledgerful bridge export` / `import` remain pure-local without opt-in. Implicit push/IPC paths require opt-in on the Ledgerful side.

---

## 11. Backup & hygiene

```powershell
ai-brains backup
ai-brains backup create --output-dir D:\backups --dry-run
ai-brains backup list
ai-brains backup verify [--full]
ai-brains backup prune --keep N --older-than <dur>
ai-brains backup restore <path> [--force] [--dry-run]
```
SQLCipher-aware backup, metadata headers, integrity checks, restore guarded when daemon is running. Default retention keeps 10 backups.

---

## 12. Privacy & crypto

- Privacy levels from cloud-ok through sealed; pins default to **`LocalOnly`**
- Preflight/recall filter non-injectable / sealed content
- **SQLCipher** vault; busy timeout under concurrent CLI access
- Key via `AI_BRAINS_KEY` / crypto recovery path; `zeroize` for secrets

---

## 13. Models & local AI routing

| Variable | Role |
|----------|------|
| `AI_BRAINS_MODEL_URL` | Completion (default `http://127.0.0.1:8081`) |
| `AI_BRAINS_COMPLETION_MODEL` | Model name |
| `AI_BRAINS_EMBEDDING_URL` | Embeddings (default `http://127.0.0.1:8083`) |
| `AI_BRAINS_EMBEDDING_MODEL` | e.g. `nomic-embed-text-v1.5` |

`daemon status` probes configured host:port with handshake retries. LLM HTTP client uses per-request timeouts.

---

## 14. Configuration hierarchy

1. Process environment  
2. Project-local `.env` (from `context`)  
3. Global `~\.ai-brains\.env` (vault path, model URLs)  

---

## 15. Typical agent workflows

| Intent | Command |
|--------|---------|
| Session start | `preflight --summary` / `--pretty` |
| “What did we decide?” | `recall "…" --semantic` |
| Code + memory | `recall` or `sync query` |
| Persist a decision | `pin "DECISION: …"` |
| Correct a memory | `forget` / `restore` |
| Sync brittle files | `safety sync` |
| Overnight brain | `nightly` (+ schedule) |
| Hygiene | `backup` · `project list` |

End-to-end recipes: [WORKFLOWS.md](WORKFLOWS.md).

---

## 16. Capability map

```text
CAPTURE          ingest · agy-hook · antigravity-import · daemon queue
CONTEXT          context · project list/resolve/detect/set-alias · stop-session
DENSE MEMORY     pin · forget/restore · safety sync
RETRIEVAL        recall (FTS · semantic · graph-boost · bridge) · preflight · sync query
INTELLIGENCE     nightly (summarize · embed · synthesize · symbol bridge)
GRAPH            neighbors · hierarchy · session · update · rebuild · live projector
INTEGRATION      Ledgerful (search/hotspots/bridge/pipe) · multi-harness hooks
OPS              init · backup suite · daemon service · schedule · update
PRIVACY/CRYPTO   SQLCipher · privacy levels · path normalization · no CoT
```

---

## 17. What it is not

- Not a cloud SaaS memory product by default  
- Not a full IDE replacement  
- Not an MCP server (CLI/hooks only)  
- Graph-heavy features need the **graph** build feature and/or healthy local models  
- Capture **must not** depend on intelligence features  
- Ledgerful bridge **push/IPC enrichment is opt-in** on the Ledgerful side  

---

## 18. Install & entry points

```powershell
# From AI-Brains repo
.\scripts\Build-AIBrains.ps1   # → ~\.cargo\bin\ai-brains.exe + ai-brainsd.exe

ai-brains init
ai-brains context
ai-brains preflight --summary
ai-brains pin "DECISION: …"
ai-brains recall "…" --semantic
ai-brains daemon status          # optional always-on: daemon install (elevated)
```
