# AI-Brains — Capabilities & Features

**Version:** 0.1.1
**Platform:** Windows 11 first (PowerShell); Ubuntu 24.04 / WSL and macOS are tiered — see **[COMPATIBILITY.md](COMPATIBILITY.md)** (not a blanket “best-effort” claim)
**Type:** Local-first CLI + optional local daemon (not an MCP server)
**Related docs:** [README.md](README.md) (index) · [INSTALL.md](INSTALL.md) · [SECURITY-LIMITS.md](SECURITY-LIMITS.md) · [OPERATIONS.md](OPERATIONS.md) · [WORKFLOWS.md](WORKFLOWS.md) · [PRD.md](PRD.md) · [COMPATIBILITY.md](COMPATIBILITY.md) · [PROTOCOL-COMPAT.md](PROTOCOL-COMPAT.md) · [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md) · [status.md](status.md) (historical) · [ADR-0019 connector sandbox](DECISIONS/ADR-0019-connector-sandbox-execution-model.md)

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
| **Canonical source of truth** | Every state change is an immutable event in an append-only log (SQLCipher page-level vault + CE — [COMPATIBILITY.md](COMPATIBILITY.md) F8 / T187) |
| **CQRS** | Commands append events; queries read projections only |
| **Capture privacy** | Only user prompts + final assistant responses (no CoT / tool logs) |
| **Privacy inheritance** | Derived memories inherit the strictest privacy of sources |
| **Event sourcing** | No update/delete of raw events; corrections via compensating events |
| **No repo pollution** | Memory defaults to user storage; projects get `.env` IDs only |
| **Path normalization** | Windows drive-case, UNC, WSL `/mnt/c` mappings normalized |
| **Relational graph** | Native SQLite backend (recursive CTEs); Cozo bridge optional/feature-gated |
| **Licensing** | PolyForm Noncommercial 1.0.0 + Small-Entity Commercial Exception; deps stay permissive (see `deny.toml`) |

**Workspace crates:** `core` · `events` · `contracts` · `store` · `crypto` · `path` · `capture` · `retrieval` · `graph` · `models` · `brain` · `scheduler` · `ai-brains-daemon-api` · `ai-brains-api-server` · `ai-brainsd` · `ai-brains-cli`

**Loopback HTTP (T161):** optional authenticated `/v1` REST on `ai-brainsd` (default off; `AI_BRAINS_HTTP=1` / `--http`); same `DaemonRequest`/`DaemonResponse` contracts as named-pipe IPC via `HttpDispatch` → `handle_daemon_request`; bearer token + owner-only ACL; bind loopback-only by default.

### Provenance (user view)

Governed memory separates **what was observed** from **what we conclude** and **what we decide**:

| Kind | Operator meaning | CLI (T160 / T203) |
|------|------------------|-------------------|
| **Evidence / source** | Observed material with fingerprints and origin | `evidence list\|search\|show`, `source list\|show` |
| **Conclusion** | Derived claim that still needs review when required | `conclusion` |
| **Decision** | Accepted commitment (often after review) | `decision`, `review list\|resolve` |

**Rules of thumb:** evidence ≠ conclusion; circular or unrooted promotion stays **Unknown** (not silently “Independent”). Corrections use **compensating events**, not silent rewrite of the log. See [ADR-0011](DECISIONS/ADR-0011-separate-evidence-conclusions-decisions.md) and [OPERATIONS.md](OPERATIONS.md) governed command surface.

---

## 3. CLI surface

```text
Setup:     init
Daily:     recall | preflight | doctor | project | pin | context | stop-session | daemon
Operator:  backup | recovery | vault | retention | device | replicate | nightly | safety
Governed:  scope | briefing | query | evidence | source | review | policy | conclusion | decision
Dangerous: forget | erasure  (+ dual-ops: retention apply, vault encrypt|rotate-datakey, migrate governed --confirm, daemon install|uninstall|update)
Harness:   ingest | antigravity-import | agy-hook | sync | shadow | evaluate | dogfood | graph | migrate
```

Canonical inventory is also on `ai-brains --help` (T204 groups). Partial historical one-liners are obsolete.

**Help information architecture (T204):** `ai-brains --help` appends role groups (Setup / Daily / Operator / Governed / Dangerous / Harness), a short Start-here block, and docs pointers. Names are unchanged; `display_order` only reorders the flat Commands list. Dangerous ops carry a `[dangerous]` about marker at the depth where mutation lives (e.g. `forget`, `erasure wipe`, `retention apply`, `vault rotate-datakey`). Short `-h` keeps a one-line tip only.

### OutputFormat defaults by command (consolidated)

Honesty matrix — **no** blanket TTY default flip for governed JSON surfaces.

| Surface | Default TTY | Default non-TTY | Notes |
|---------|-------------|-----------------|-------|
| `recall` | pretty | json | Explicit `--format` wins |
| `preflight` | human | json | TTY default is `human`; `--pretty` / `--format pretty` also human-mode |
| `briefing` | markdown | json | T202 F9; explicit `--format` wins |
| `query progressive` / `expand` / `trace` | json | json | No TTY flip |
| list/show (governed evidence/source/review/...) | json (Human if `--format human`) | json | `OutputFormat::parse` → Json bare |
| `doctor` | human | human | `--json` / `--format json` override |

**Global options:**

| Flag / env | Purpose |
|------------|---------|
| `--vault-path` / `AI_BRAINS_VAULT_PATH` | Vault database path |
| `--key` / `AI_BRAINS_KEY` | Vault SQLCipher key (`x'<64 hex>'`). Zero key refused unless `AI_BRAINS_ALLOW_ZERO_KEY=1` (T187) |
| `--no-project-context` | CI/hooks: do not load project `.env` or clobber inherited IDs |
| `--log-format` | `compact` \| `full` \| `json` \| `minimal` \| `off` |

Failures use a **dual envelope** model (see [CLI-EXIT-CODES.md](CLI-EXIT-CODES.md)):

| Path | Shape | Stream |
|------|-------|--------|
| Governed **Json** | bare `ApiError` | **stdout** |
| Governed **Human** / Markdown | `CODE: message` | **stderr** |
| Generic non-governed (`handle_cli_result`) | full `ApiResult` | **stderr** |

Normative exit codes **0–7** (including `FEATURE_UNAVAILABLE`→**2**, doctor footnotes, vault key exit **1**): [CLI-EXIT-CODES.md](CLI-EXIT-CODES.md).

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
| **Semantic** | `--semantic` + stored embeddings; honors `AI_BRAINS_EMBEDDING_URL` (default `http://127.0.0.1:8083`) and `AI_BRAINS_EMBEDDING_MODEL` (default `nomic-embed-text-v1.5`) |
| **Embedding status** | With `--semantic`, JSON includes additive `embedding: { status, endpoint?, detail? }`. Closed statuses: `ok` \| `unreachable` \| `error` \| `no_stored_embeddings` \| `skipped`. Soft-fail: embed down never aborts FTS/bridge recall (exit **0**). Pretty TTY prints one status line when `status != ok`. |
| **Graph boost** | Neighbor score boost (`--graph-boost`) |
| **Substring fallback** | When FTS empty on small vaults |
| **Scope** | Project default; `--global`; `--session` / `--session-prefix` / `--session-last` |
| **Bridge mix** | Ledgerful hits capped so vault memories still surface; `--no-bridge` |
| **Formats** | Pretty on TTY by default; JSON / NDJSON; per-result `session_id` |
| **Hints** | Contextual no-results hints on stdout (next-action only when embedding status already explains cause) |

### Briefing + progressive query (T202)
```powershell
ai-brains briefing project --project-id <uuid>
ai-brains briefing personal --format json
ai-brains query progressive "why was graph backend replaced?" --project-id <uuid>
```

| Feature | Detail |
|---------|--------|
| **Briefing format** | TTY default **markdown**; non-TTY **json**; explicit `--format` wins (dogfood always passes `--format json`) |
| **Denied packets** | `denied=true` always seeds `warnings[]` with `kind: "denied"`; markdown includes `> **Denied:** …` one-liner |
| **Progressive / expand** | Require project id (`--project-id` or `AI_BRAINS_PROJECT_ID`); missing → exit **2** + copy-paste example on stderr |
| **Trace** | No project-id gate; missing/unauthorized → `null` exit **0** |

### Governed discovery lists (T203)
```powershell
ai-brains source list --scope Repository:<uuid> --format json
ai-brains source list --format json   # soft-fill when AI_BRAINS_PROJECT_ID / context is authoritative
ai-brains evidence list --scope Repository:<uuid>
ai-brains evidence list --query keyword --scope Repository:<uuid>
ai-brains evidence search --query keyword --scope Repository:<uuid>
ai-brains review list --format json   # soft-default scope (authoritative) or fail_usage exit 2
```

| Feature | Detail |
|---------|--------|
| **Commands** | `source list`, `evidence list` (optional `--query` FTS), `evidence search` (requires `--query`) |
| **Bounds** | Default limit **50**, hard clamp **200**; `more_available` via LIMIT+1 |
| **Empty** | E1 `items: []` (never null); human `(none)`; exit **0** when policy allows |
| **Policy** | `ReadEvidence` for source/evidence list; `ReadConclusions` for review list; deny → exit **3** + `details.hint` |
| **Soft-resolve** | Omitted `--scope` fills only when `scope resolve` is authoritative; else **exit 2** `fail_usage` (never exit **6**) |
| **Show** | `source show` / `evidence show` use the same soft-resolve helper |
| **Status filter** | Default Active-only on source/evidence projections |
| **Capture independence** | Projection reads only — no models/embeddings; no control-plane→retrieval dependency |

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

Requires a build with **`--features graph`** for the live backend. Recommended source install (INSTALL SOOT):

```powershell
cargo install --path crates/ai-brains-cli --locked --features graph
```

**Feature-off honesty (T198):** on default / slim / GitHub Release `ai-brains.exe` builds (no graph feature), every `ai-brains graph *` subcommand exits **2** with a `FEATURE_UNAVAILABLE:` prefix and a reinstall hint for the command above. `graph --help` remains exit **0**. GitHub Release `ai-brains.exe` is currently graph-off; see [INSTALL.md](INSTALL.md).

| Command | Purpose |
|---------|---------|
| `graph update` | Health: nodes, edges, live status |
| `graph rebuild` | Full resync (recovery) |
| `graph neighbors <memory_id>` | 1-hop neighbors |
| `graph hierarchy <memory_id>` | Synthesis chain |
| `graph session <session_id>` | Memories in a session |

> **Feature-off:** all rows above exit **2** + `FEATURE_UNAVAILABLE` when the binary was built without `--features graph`.

**Live Graph Hook:** incremental projection on each event append (graph-on builds only).

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
ai-brains recovery export --output <path> [--passphrase-file] [--dry-run] [--force|--overwrite]
```
Backup suite with metadata headers, integrity checks, and restore **hard-fail** when the daemon/service is reachable via robust IPC probe (T188; `--force` never overrides). SQLCipher-encrypted vaults and backups (T187). Default retention keeps 10 backups. Plain→encrypted migrate: `ai-brains vault encrypt` (`sqlcipher_export`).

**Recovery export (T188 / T194):** writes RecoveryKit JSON (`schema_version: 1`) to a restricted file path only (never kit JSON on stdout). Passphrase via file or zero-echo TTY (`rpassword`). Kits embed Argon2id params in `passphrase.kdf` (algorithm=argon2id, version=19, m=19456, t=2, p=1); pre-T194 kits without `kdf` dual-read fixed legacy constants.

**Doctor (T192):** `ai-brains doctor` is a **read-only** operator health surface.

```text
ai-brains doctor
  [--format human|json]           # default human
  [--json]                        # force JSON (overrides --format)
  [--fail-on-degraded]            # exit 1 when status=degraded
  [--kit-path <path>] [--passphrase-file <path>]
  [--backup-max-age <Nd|Nh|Nw>]   # default 7d
  [--full]                        # PRAGMA integrity_check
```

Check matrix (fixed order): `vault_exists`, `vault_open` (`open_read_intent` only — never migrates), `schema_readable`, `cipher_page`, `daemon_reachable` (info: up/down never fails alone), `backup_recent` (soft), `recovery_kit_event` (soft; event ≠ offline file proof), `recovery_kit_file` (hard when `--kit-path` set; skip otherwise — no default kit path search), `zero_key_escape` (soft / R-ZERO-KEY), `integrity` (only with `--full`). Overall: fail ≻ degraded ≻ ok. Exit 0 for ok|degraded (default); 1 for fail; clap usage 2. Never creates vault or `backups/`; never prints secrets. Residual: offline kit without `--kit-path` remains operator responsibility (see RECOVERY-DRILLS).

---

## 12. Privacy & crypto

- Privacy levels from cloud-ok through sealed; pins default to **`LocalOnly`**
- Preflight/recall filter non-injectable / sealed content
- Vault open with SQLCipher page encryption + Content Envelope — [COMPATIBILITY.md](COMPATIBILITY.md) F8 / T187; busy timeout under concurrent CLI access
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
PRIVACY/CRYPTO   vault+CE (F8) · privacy levels · path normalization · no CoT
LEGACY IMPORT    classify_legacy / apply_legacy_import (T167) + `migrate governed` CLI (T168)
EVALUATION       `evaluate governed` (T169) + shadow dogfood gate / `dogfood compare` (T170)
```

### Legacy → governed classification import (T167 / P9.1) + migrate CLI (T168)

**Control-plane API** plus operator CLI. Callers (or `ai-brains migrate governed`) supply a legacy event stream + destination ports; the importer never opens the live vault itself.

| API / CLI | Role |
|-----------|------|
| `classify_legacy` | Dry-run plan + `plan_hash` (default); under-promotes pins→Evidence, synth→Candidate, decisions→Proposed+Review |
| `apply_legacy_import` | Confirm-gated append via raw `build_event`; no `observe_source` / no `RecordEvidence` capability |
| `plan_report_json` | Operator report (ids/counts/hash; optional truncated snippets; never full bodies by default) |
| `ai-brains migrate governed` | T168: dry-run differential report; `--confirm` dest materialize + T167 apply; live/reparse refuse; no plaintext in report |

See [OPERATIONS.md](OPERATIONS.md#legacy--governed-classification-import-t167--p91) and [OPERATIONS.md](OPERATIONS.md#governed-migrate-cli-t168--p92) for operator rules (idempotency, forgotten cascade, CE honesty, re-apply, force-overwrite).

### Governed evaluation harness (T169 / P9.3)

| API / CLI | Role |
|-----------|------|
| `ai-brains evaluate governed` | Run scenario corpus; JSON report + exit 0/1/6/7; never mutates live vault |
| CP `evaluation/` | Pure metrics, seed programs 1–9, hermetic runner |
| sources nextest | Scenario 10 circularity hard gates |

See [EVALUATION/GOVERNED-MEMORY-MVP.md](EVALUATION/GOVERNED-MEMORY-MVP.md) and [OPERATIONS.md](OPERATIONS.md#governed-evaluate-cli-t169--p93).

### Shadow dogfood gate (T170 / P9.4)

| API / CLI / script | Role |
|--------------------|------|
| `scripts/dogfood-shadow.ps1` | Stage A evaluate + D24 live hash + shadow/migrate under WorkDir + compare capture; **never** Stage D / User env / `AI_BRAINS_VAULT_PATH`→shadow |
| `ai-brains dogfood compare` | Pure-serde compare packet (`dogfood-compare.json`); fingerprints + `compare_hash`; D15 seed + warning_refs |
| Human checklist | [EVALUATION/templates/dogfood-human-checklist.md](EVALUATION/templates/dogfood-human-checklist.md) |

See [EVALUATION/SHADOW-DOGFOOD-GATE.md](EVALUATION/SHADOW-DOGFOOD-GATE.md) and [OPERATIONS.md](OPERATIONS.md#shadow-dogfood-gate-t170--p94).

---

## 17. What it is not

- Not a cloud SaaS memory product by default
- Not a full IDE replacement
- Not an MCP server (CLI/hooks only)
- Graph-heavy features need a **`--features graph`** build (recommended: `cargo install --path crates/ai-brains-cli --locked --features graph`). Feature-off binaries exit **2** with `FEATURE_UNAVAILABLE` on `graph *` (T198). GitHub Release `ai-brains.exe` is currently graph-off. Healthy local models may still be needed for semantic/embedding paths.
- Capture **must not** depend on intelligence features
- Ledgerful bridge **push/IPC enrichment is opt-in** on the Ledgerful side
- Not a third-party connector plugin host — release connectors are first-party **`TrustedBuiltin` only** ([ADR-0019](DECISIONS/ADR-0019-connector-sandbox-execution-model.md))


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
