# AI-Brains Operations Guide

This guide covers the day-to-day operations, configuration, and troubleshooting of the AI-Brains system.

> **Current state (June 2026):** Phase 15 (Cross-Agent Memory Synthesis) plus T44–T71 are shipped. The CLI has 17 top-level subcommands, the daemon auto-launches, nightly schedules via Windows Task Scheduler, and the Ledgerful bridge is live. The Operations surface is significantly larger than the pre-T44 era this document originally described.

## 1. Installation and Setup

### Prerequisites
- Rust (Stable, MSVC toolchain)
- PowerShell 7+ (Recommended for Windows)
- `cargo-nextest`, `cargo-deny`, `cargo-audit` — see [ci-tooling.md](ci-tooling.md) for pins

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

When the refused case triggers, the CLI returns a structured JSON error envelope on stderr (the same shape used by every other failure path) and exits 1.

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

### Antigravity Import
Bulk-import Antigravity conversation logs from local tool-specific brain dirs.
```powershell
ai-brains antigravity-import --days 30
```
- `--days <N>`: only import sessions modified in the last N days (default 30).
- Idempotent: skips sessions already in the vault.
- Tool-only and hidden-thinking entries are filtered out (Mandate #4).

### `agy` Hook
Real-time capture from the Antigravity CLI hooks integration:
```powershell
ai-brains agy-hook --payload '{"transcriptPath": "C:\\path\\to\\session.jsonl", ...}'
```
A well-formed payload returns `{"ok":true,"status":"success",...}`. A malformed payload (e.g. missing `transcriptPath`) returns `{"ok":false,"status":"error","message":"..."}` — the harness hook treats this as a non-fatal failure.

## 3. Retrieving Memories

### Lexical Recall
```powershell
ai-brains --vault-path ./vault.db recall "authentication logic" --limit 5
```
Options worth knowing:
- `--format pretty` for human-readable scores
- `--semantic` for vector (embedding) search alongside FTS5
- `--graph-boost <0.0–1.0>` to weight graph-neighbor hits
- `--project-id` / `--session-id` to scope

### Unified Search (AI-Brains + Ledgerful)
The T70 bridge lets a single command search both your memory vault and the Ledgerful ledger.
```powershell
ai-brains sync query "rust" --format pretty
```
Output has two sections — `--- AI-Brains Recall ---` (vault FTS hits) and `--- Ledgerful Ledger Search ---` (ledger entries). Use `--quiet` to suppress the second section if you only want the vault view.

### Generating Preflight Context
```powershell
ai-brains preflight --max-words 1500
```
- `--summary` for a concise statistical summary
- `--pretty` / `--format human` for human-readable text
- `--scope "src/foo.rs,src/bar.rs"` for contextual risk analysis on a specific path set

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
  optional `denied`.

**CLI surface (dry-run JSON by default)**
```powershell
ai-brains briefing project --project-id <uuid> --format json
ai-brains briefing personal --format markdown
ai-brains query progressive "authority order" --project-id <uuid>
ai-brains query expand <handle-id> --project-id <uuid>
ai-brains query trace <trace-id>
```

### Governed command surface (T160)

Thin CLI over control-plane (local default for reads) and named-pipe daemon (preferred for mutations). JSON default for new commands; exit codes: 3=POLICY_DENIED, 4=NOT_FOUND, 5=DAEMON_UNAVAILABLE, 6=INVALID_PAYLOAD.

```powershell
ai-brains scope resolve --format json
ai-brains evidence show <id> --scope Repository:<uuid> --format json
ai-brains source show <id> --scope Repository:<uuid>
ai-brains review list --scope Repository:<uuid>
ai-brains conclusion propose --claim "..." --evidence <id> --scope Repository:<uuid> --local
ai-brains decision propose --statement "..." --scope Repository:<uuid>
ai-brains review resolve <id> --resolution approved --scope Repository:<uuid>
ai-brains erasure request --id <id> --scope Repository:<uuid>   # daemon-required; ticket only (not CE)
ai-brains erasure wipe --content-key-id <uuid> --scope Repository:<uuid>           # dry-run (default)
ai-brains erasure wipe --content-key-id <uuid> --scope Repository:<uuid> --confirm # execute CE
ai-brains policy show --scope Repository:<uuid>
ai-brains policy check --capability ProposeConclusion --scope Repository:<uuid>
```

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
ai-brains retention plan --format json
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
| Audit | Apply appends `RetentionApplied` (class counts/mechanisms; no bodies). Dry-run does not. |

**Class matrix (v1 defaults)**

| Class | Stream | Horizon | Mechanism |
|-------|--------|---------|-----------|
| `raw_turn` | A (projection) | 90d | `projection_delete` (`delete_old_turns` / equivalent) |
| `evidence` | B (envelope) | 365d | `ce_wipe` if `content_key` present |
| `decision_approved` | A | revoked/superseded + 30d cooldown | projection cleanup of terminal rows only |
| `secret` | B | 7d | `ce_wipe` |
| `review_trace` | A | 90d from terminal `updated_at` | projection cleanup if closed |
| `query_trace` | A | 30d | projection delete by `recorded_at` |
| `memory_legacy` | A | none auto | pinned → `held` |
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
| Keys | `--source-key` / `--destination-key` with `--key` fallback. `--key` may be placed **after** `migrate governed` or as a root flag before `migrate`. Resolution: source_key → key → zero-key; destination_key → key → zero-key. Raw SQLCipher key strings only (fixture/shadow pattern). Production DPAPI unlock of arbitrary live vaults is **out of scope** (T168). |
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
This command generates a deterministic `PROJECT_ID` based on your directory and a fresh `SESSION_ID`, storing them in a local `.env` file. Subsequent operations (recall, ingest) automatically use these env values.

- `--show` — print current context without modifying `.env`
- `--new-project` — force a fresh project ID
- `--new-session` — rotate the session ID (useful for long sessions)
- `--tx-id <uuid>` — link the context to a Ledgerful transaction (T37)

### Listing Projects
```powershell
ai-brains project list
```
Output (post-T76): a table with `project_id`, `name (alias|UUID)`, `alias`, and `memories` columns.

### Resolving Aliases
```powershell
ai-brains project resolve ai-brains          # exact alias match, falls back to fuzzy
ai-brains project detect --export            # auto-detect from current git repo
```

## 5. Background Intelligence & Scheduling

### Daemon Lifecycle
The `ai-brainsd` daemon is a single-writer queue that serializes event writes for concurrency safety.
```powershell
ai-brains daemon start             # start in background (console process)
ai-brains daemon status            # show PID + listening ports
ai-brains daemon stop              # graceful shutdown (use --force if it hangs)
ai-brains daemon install           # install as Windows service (requires elevation)
ai-brains daemon uninstall         # remove the Windows service (requires elevation)
```
- **Governed IPC (T159/T165):** propose/resolve/erasure-ticket/wipe mutations go through the writer queue; scope/briefing/query/inspect are off-queue reads. Spool durable crash recovery for governed mutations **only** when the request includes `command_id` (filename `{op}_{sanitized_command_id}.json`). Briefings over the daemon are dry-run. Ticket erasure returns `accepted` only after a durable `ErasureTicketAccepted` event (still **not** CE). Content-envelope wipe is `WipeContentEnvelope` / HTTP `POST /v1/erasure/wipe` (dry-run default; execute needs `confirm=true`). Principal: wire `principal_id` UUID → System if well-known CLI System UUID, else Human; if wire omitted, `AI_BRAINS_DAEMON_PRINCIPAL_ID`, else System default. CLI always passes resolved `principal_id` on daemon wire (T160). Production policy only (`production_policy`).
- **Loopback HTTP API (T161):** optional authenticated REST surface under `/v1` that reuses the same `handle_daemon_request` path as named-pipe IPC (no second writer). **Default off.** Enable with `AI_BRAINS_HTTP=1` or `ai-brainsd --http`. Binds `127.0.0.1:7432` by default (`AI_BRAINS_HTTP_PORT`). Bearer token lives at `%USERPROFILE%\.ai-brains\http.token` (created on first enable; path logged, token never printed). Owner-only Windows ACL `D:P(A;;FA;;;OW)`. All data routes require `Authorization: Bearer <token>`; `GET /health` and `GET /v1/health` are unauthenticated liveness only (`{"status":"ok"}`). Non-loopback bind requires **both** `--http-bind <addr>` and `AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK=1`. CORS is deny-by-default (no `Access-Control-Allow-Origin: *`). Body limit 1 MiB → HTTP 413. Loopback is **not** zero-trust — always use bearer auth; do not expose without reverse-proxy/mTLS planning (out of scope for v1). Mutations accept optional `X-Command-Id` when the JSON body omits `command_id`. If HTTP is explicitly enabled and bind/token start fails, the daemon **hard-fails** (does not continue as if HTTP were up). Post-spawn serve death is logged only (bind already succeeded).
- **HTTP under Windows LocalSystem service (residual):** When `ai-brainsd` runs as the **Windows service** (`LocalSystem` / Session 0), `%USERPROFILE%` is the **SYSTEM profile**, so `http.token` is created with SYSTEM as owner and an owner-only ACL. **Interactive desktop/CLI clients cannot read that token.** HTTP under the service host is **not** intended for Session 1 local clients. Prefer an **interactive** daemon (`ai-brainsd --http` or `ai-brains daemon start` with `AI_BRAINS_HTTP=1`) for desktop clients. A multi-session shared token path is **out of scope** residual. The service path logs a strong warning when HTTP is enabled. If HTTP is enabled and bind/token setup fails (or other fatal startup fails), the service **does not report `Running` to SCM** — it stays in `StartPending` until failure, then reports **`Stopped` with a service-specific non-zero exit code** (`ServiceSpecific(1)` = startup failed). Operators should treat `sc start` / Services MMC failures as a real start failure, not a healthy daemon without HTTP.
- The CLI auto-launches the daemon if it is unreachable, so most users never need `daemon start` explicitly.
- **Windows service (recommended for persistent daemon):** `daemon install` registers `AI-Brains-Daemon` as a Windows service running as `LocalSystem` in Session 0. The pipe security descriptor grants the interactive user cross-session access, so CLI clients in Session 1 can connect. Env vars (vault path, model URLs) are written to `%ProgramData%\AI-Brains\daemon.env` with a restrictive ACL (`SYSTEM:F` + `Administrators:F` only — same model as the nightly SYSTEM wrapper; T145). Requires an elevated PowerShell session.
- **Deprecated:** `daemon schedule` / `unschedule` (Task Scheduler ONLOGON) still work but are deprecated in favor of `install` / `uninstall`. The Task Scheduler approach had a cross-session pipe access issue where Session 0 daemons were unreachable from Session 1.

### Nightly Intelligence Sweep
```powershell
ai-brains --vault-path ./vault.db nightly
```
The nightly job does:
- Antigravity session import (T33)
- Summarization of unsummarized sessions (with T34 chunking for sessions over 38,912 tokens)
- Memory synthesis (RAPTOR-style clustering + CRAG factual verification)
- Symbol-bridge ingestion from Ledgerful (T70)
- MemoryPinned / MemorySynthesized event emission (T67, T68) for the live graph

### Scheduling Nightly
```powershell
ai-brains nightly --schedule --start-time "03:00"
ai-brains nightly --status             # show last run timestamp + pending work
ai-brains nightly --unschedule
```

#### Running the nightly as SYSTEM (`--run-as-system`)
By default `--schedule` registers a task under the current user, which inherits that user's environment variables. The optional `--run-as-system` flag registers the task with `/RU SYSTEM` so it runs without anyone logged in (T132). Because the `SYSTEM` account does **not** inherit User-level environment variables, the CLI handles this specially (T143 + T145):

- It generates a **wrapper `.bat` script** that bakes in the current values of `AI_BRAINS_VAULT_PATH`, `AI_BRAINS_MODEL_URL`, `AI_BRAINS_COMPLETION_MODEL`, `AI_BRAINS_EMBEDDING_URL`, and `AI_BRAINS_EMBEDDING_MODEL` from your environment (or `.env`). The scheduled task runs that wrapper instead of the bare executable, so SYSTEM gets the same config you have.
- **Wrapper location (T145):** `%ProgramData%\AI-Brains\nightly-task.bat` — not the vault parent or `%TEMP%`. Creation refuses symlink/reparse/junction targets at the file path, refuses hardlinks (`nlink > 1`), **and** refuses if the parent directory (e.g. `%ProgramData%\AI-Brains`) exists as a junction/reparse point. Regular single-link existing files may be replaced on re-schedule.
- **ACL (T145):** after write, the CLI applies an **absolute** DACL via Win32 SDDL/`SetNamedSecurityInfo` (`D:P(A;;FA;;;SY)(A;;FA;;;BA)` — protected, SYSTEM + Administrators full only). This replaces the entire DACL so session leftovers (e.g. `LogonSessionId`) cannot remain. The CLI then verifies with `icacls` query (fail closed). Check with:
  ```powershell
  icacls "$env:ProgramData\AI-Brains\nightly-task.bat"
  ```
  Expect only `SYSTEM` and `Administrators` with full control. If ACL apply or verify fails, scheduling aborts — `schtasks /Create` is not called.
- The wrapper appends `--no-project-context --skip-import` to the `ai-brains.exe nightly` invocation. SYSTEM has no `.env` to auto-discover and cannot reach your Antigravity session DB, so project-context discovery and the Antigravity import would both fail; these flags skip them.
- `--run-as-system` **requires Administrator rights** (ProgramData ACL + `/RU SYSTEM`). From a normal shell the CLI **prompts for UAC** and re-launches itself elevated (approve the dialog). You can still use an already-elevated PowerShell if you prefer. If UAC is cancelled or disabled, re-run from an Administrator shell. `--dry-run` does not elevate.
- **Residual risk (accepted, T145):** the invoked binary typically lives under `%USERPROFILE%\.cargo\bin\` (user-writable by design for `cargo install`). Copying binaries into `ProgramData` is packaging/installer scope, not done here. The primary hijack vector on the *script* path is closed by the ProgramData + ACL model above. The same residual applies to `ai-brainsd.exe` used by `daemon install` / deprecated `daemon schedule --run-as-system`. `daemon.env` uses the same ACL model under `%ProgramData%\AI-Brains\`.

To preview the registration without writing it, add `--dry-run`:

```powershell
ai-brains nightly --schedule --run-as-system --start-time "03:00" --dry-run
```

`--dry-run` prints the `schtasks` command and the generated wrapper script to stdout without registering the task, so you can verify the baked-in env vars and flags before committing.

> **Migration (T143 + T145):** Existing `AI-Brains-Nightly` SYSTEM tasks may still point at a vault-parent or `%TEMP%` wrapper without restrictive ACL. Re-schedule after T145 to pick up `%ProgramData%\AI-Brains\nightly-task.bat`: `ai-brains nightly --unschedule` then `ai-brains nightly --schedule --run-as-system` from an elevated shell. The same treatment applies to deprecated `daemon schedule --run-as-system` and to re-running `daemon install` for `daemon.env` ACL hardening.

## 6. Memory Hygiene

### Soft-Delete
```powershell
ai-brains forget --memory-id <uuid>           # prompt; -f to skip
ai-brains forget --match "outdated fact" -f   # find by content; -f to forget
ai-brains forget --list-forgotten             # show everything soft-deleted
ai-brains forget --restore <uuid>             # undo with a compensating event
```
Forgotten memories remain in the event log for audit but are excluded from FTS, graph, and preflight.

### Backup
```powershell
ai-brains backup                                # create with timestamped default path
ai-brains backup create --output-dir D:\backups # custom directory
```
Backups include an integrity check; corrupt backups are rejected at creation time.

### Restore
```powershell
ai-brains backup restore <path>               # interactive confirm + overwrite
ai-brains backup restore <path> --force       # non-interactive (CI/automation)
ai-brains backup restore <path> --dry-run     # verify integrity, report, no changes
```
`--dry-run` runs the integrity check, prints the planned destination, and exits 0 without touching the vault. Use it in scripts before a real restore.

## 7. Safety & Hotspot Sync

Ledgerful scans the codebase for hotspots (frequently-edited, complex files). The bridge re-pins these as AI-Brains memories so they appear in preflight and recall.

```powershell
ai-brains safety sync                # sync top 5 hotspots
ai-brains safety sync --limit 20     # sync top 20
ai-brains safety sync --dry-run      # preview what would be synced
```

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
3. The service creates the pipe with a security descriptor that grants the current user cross-session access.

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

### Graph health check
```powershell
ai-brains graph update
```
Reports `{ nodes, edges, status: "live", note }`. If `status` is not `"live"` or counts are unexpectedly zero, run:
```powershell
ai-brains graph rebuild
```

### Vault Locked
If the vault cannot be opened, ensure the `AI_BRAINS_KEY` environment variable is set or the correct `--key` argument is provided.

### Missing Graph Database
If the graph features are missing on Windows, verify that the `graph` feature was enabled during build and that the MSVC 4GB image size limit was not exceeded. If it was, the system will gracefully fall back to Lexical search.

## 9. Environment Variables

| Variable | Description |
|---|---|
| `AI_BRAINS_VAULT_PATH` | Default path to the vault database. |
| `AI_BRAINS_KEY` | Hex-encoded SQLCipher key (or dummy in degraded mode). |
| `AI_BRAINS_PROJECT_ID` | Default `project_id` for capture/recall (set by `ai-brains context`). |
| `AI_BRAINS_SESSION_ID` | Default `session_id` (set by `ai-brains context`). |
| `LEDGERFUL_TX_ID` | Ledgerful transaction ID for ledger cross-linking (preferred; T142). |
| `CHANGEGUARD_TX_ID` | Deprecated alias for `LEDGERFUL_TX_ID` (warns and falls back; T142). |
| `AI_BRAINS_MODEL_URL` | Endpoint for the local LLM completion server (default: `http://127.0.0.1:8081`). |
| `AI_BRAINS_EMBEDDING_URL` | Endpoint for the local embedding server (default: `http://127.0.0.1:8083`). |
| `AI_BRAINS_EMBEDDING_MODEL` | Name of the embedding model (default: `nomic-embed-text-v1.5`). |
| `AI_BRAINS_COMPLETION_MODEL` | Name of the completion model (default: `gemma-4-E4B-it-Q6_K.gguf`). |
| `AI_BRAINS_SCOPE` | Comma-separated paths for preflight contextual risk analysis. |
| `AI_BRAINS_GOVERNED_BRIEFING` | When `1`/`true`/`yes`, `preflight` uses typed Project briefing (policy + authority). Default off. |
| `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` | UUID principal for governed preflight / briefing / query CLI grant checks. |
| `AI_BRAINS_HTTP` | When `1`/`true`/`yes`, enable in-daemon loopback HTTP `/v1` (T161). Default off. Also `ai-brainsd --http`. |
| `AI_BRAINS_HTTP_PORT` | HTTP listen port (default `7432`). Bind remains `127.0.0.1` unless non-loopback double opt-in. |
| `AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK` | Must be `1` **and** `--http-bind` explicit to bind non-loopback (dangerous; not recommended). |

## 10. Command Summary

| Action | Command |
|---|---|
| Initialize Vault | `ai-brains init` (use `--force` to overwrite populated vault) |
| Show Context | `ai-brains context --show` |
| Sync Safety Signals | `ai-brains safety sync` (use `--dry-run` to preview) |
| Unified Search | `ai-brains sync query "<topic>"` (searches vault + Ledgerful) |
| Get Orientation | `ai-brains preflight` (use `--pretty` for full text, `--summary` for stats) |
| Typed Project/Personal Briefing | `ai-brains briefing project\|personal` (JSON packet; see T152 section) |
| Progressive Query / Expand / Trace | `ai-brains query progressive\|expand\|trace` |
| Scope / Evidence / Source / Review | `ai-brains scope resolve` · `evidence show` · `source show` · `review list\|resolve` (T160) |
| Propose Conclusion / Decision | `ai-brains conclusion propose` · `decision propose` (daemon prefer; `--local` OK) |
| Erasure ticket (daemon-only) | `ai-brains erasure request --id … --scope …` (no CE wipe claim) |
| Policy show / check | `ai-brains policy show\|check` (read-only grants) |
| Deep Search | `ai-brains recall` (use `--format pretty` for readable results) |
| Pinned Record | `ai-brains pin` (use `--tag` for categories, `--stdin` piped) |
| Forget Memory | `ai-brains forget` (use `--match` for search, `--restore` undo, `-f` to skip confirm) |
| Antigravity Capture Hook | `ai-brains agy-hook --payload "{...}"` (used by agy CLI hooks) |
| Import Antigravity | `ai-brains antigravity-import --days 30` (incremental scan) |
| Nightly Sweep | `ai-brains nightly` (summarization + graph + bridge) |
| Schedule Nightly | `ai-brains nightly --schedule --start-time "03:00"` |
| Daemon Control | `ai-brains daemon start/status/stop/schedule/unschedule` |
| Backup Vault | `ai-brains backup` |
| Restore Vault | `ai-brains backup restore <path>` (use `--force` non-interactive, `--dry-run` to preview) |
| Manage Projects | `ai-brains project list/resolve/detect` |
| Graph Health | `ai-brains graph update` (use `graph rebuild` if stale) |

## Desktop thin client (T172)

See [apps/desktop/README.md](../apps/desktop/README.md) for the Tauri adapter shell.

- **Invoke-first:** UI never uses webview fetch to loopback `/v1`; Rust holds the user-session bearer.
- **Prereqs for live screens:** daemon on `AI_BRAINS_HTTP_PORT` (default 7432) + `%USERPROFILE%\.ai-brains\http.token`.
- **Offline/denied:** paint promptly (QueryClient `retry: false`); no fake full-grant empty states.
- **Unavailable by design on this track:** connectors UI, retention plan UI, grants inventory.
