# CLI exit codes (normative)

**Source of truth** for `ai-brains` process exit codes. Linked from [OPERATIONS.md](OPERATIONS.md), [CAPABILITIES.md](CAPABILITIES.md), and root [CONTRIBUTING.md](../CONTRIBUTING.md).

Implementation map: `crates/ai-brains-cli/src/commands/governed_common.rs` (`EXIT_*`, `exit_code_for_api_error`, `fail_api`, `fail_usage`).

## Normative table (product codes 0–7)

| Exit | Constant | When |
|------|----------|------|
| **0** | `EXIT_SUCCESS` | Success; empty-success; `daemon status` report (Running or Stopped); `device status` report (empty or enrolled); `doctor` **ok** or **degraded** (unless `--fail-on-degraded`); **T254** empty `project list-paths`; **T259** filtered-empty `list-paths` (`No path aliases match.`); not-registered `project unregister-path`; `project scan-roots` (including truncated / no hits); `project rebind-path` print-only / write / already-bound |
| **1** | `EXIT_INTERNAL` | Internal / catch-all; `PATH_REFUSED`; `COMMAND_FAILED`; `INVALID_TRANSITION`; vault/key codes (`VAULT_KEY_*` / `VAULT_LOCKED`); `doctor` **fail**; **T254** `register-path` conflict (other owner) and `unregister-path --project` owner mismatch; **T259** `rebind-path` no owner / dest missing; `list-paths --project` unknown dest |
| **2** | `EXIT_USAGE` | Clap missing/invalid usage (e.g. missing required `--scope` on erasure); `FEATURE_UNAVAILABLE` (e.g. default-build `graph *`); `fail_usage` for `query progressive` / `query expand` missing project id; **T241** `policy check` omit `--capability` → capability catalog (not clap “required arguments”); **T203/T226** soft-resolve failure on `source`/`evidence`/`review` list|show and `policy show|check|bootstrap` when `--scope` omitted and context is not authoritative; **T252** empty / whitespace-only / TTY stdin on `ingest` / `ingest --dry-run` → `fail_usage` (not EOF `COMMAND_FAILED`); **T254** empty path after normalize; unknown `--format` on `list-paths` / `scan-roots`; **T259** `rebind-path --write` without `--yes`; clap `--yes` without `--write`; missing `--to` |
| **3** | `EXIT_POLICY_DENIED` | `POLICY_DENIED`; `APPROVAL_REQUIRED`; **`query progressive`** when packet `denied: true` (T221 — pretty `ProgressiveQueryResponse` still on **stdout**); **`query expand`** when preview `kind` is exact **`Denied`** |
| **4** | `EXIT_NOT_FOUND` | `NOT_FOUND` |
| **5** | `EXIT_DAEMON_UNAVAILABLE` | Daemon required / unreachable for a daemon-required path |
| **6** | `EXIT_INVALID_PAYLOAD` | `INVALID_PAYLOAD`; `NOT_ENVELOPE_BACKED`; malformed provided values (ids, unknown capability, bad JSON, dual-flag wipe, dogfood missing file, …) |
| **7** | `EXIT_HARD_GATE_FAILED` | Evaluate trust hard gates failed (harness ran; product blocked) |

**No product exit codes 8+.** Scripts must not invent higher codes as contract.

### FEATURE_UNAVAILABLE → 2

Optional features not compiled into this binary (notably default-build `graph *` without `--features graph`) exit **2** with a human `FEATURE_UNAVAILABLE:` prefix (and reinstall hint). Same numeric class as clap usage.

### fail_usage → 2 (T202 / T203)

`query progressive` and `query expand` require `--project-id` or `AI_BRAINS_PROJECT_ID`. When both are unset, `fail_usage` writes a copy-paste example to **stderr** and exits **2** via `GovernedCliError` / `EXIT_USAGE` (not clap-required; not exit 1). `query trace` is excluded (no project-id). Missing/unauthorized: exit **0** + pretty JSON envelope (`found: false`, `next_step` copy-paste `query progressive … --dry-run false`) or `--format human` two lines. Optional `--format` (`auto|pretty|human|text|json|markdown|md`; case-sensitive). Not exit **4** `NOT_FOUND`. Not the token `null`.

**T203/T226 soft-resolve:** `source list|show`, `evidence list|search|show`, `review list`, and **`policy show|check|bootstrap`** accept optional `--scope`. When omitted, CLI runs `resolve_scope` (cwd + `AI_BRAINS_PROJECT_ID`) and fills only if **authoritative**. Otherwise `fail_usage` on **stderr** (template includes example `--scope Repository:<uuid>`, `ai-brains scope resolve`, and “non-authoritative context is not filled silently”) and exit **2** — **not** clap “required arguments were not provided”, and **not** exit **6** `INVALID_PAYLOAD`.

**T252 ingest empty/TTY:** `ingest` and `ingest --dry-run` treat empty, whitespace-only, or interactive TTY stdin as `fail_usage` — human text on **stderr** (copy-paste example JSON + `ai-brains ingest --dry-run`), **zero stdout**, exit **2**. Do **not** emit `COMMAND_FAILED` / `Invalid JSON: EOF`. Mid-payload parse (`{`, truncated object, non-empty garbage) stays exit **1** `COMMAND_FAILED` / `Invalid JSON` via the generic `handle_cli_result` envelope on stderr.

### Doctor (footnote)

| Outcome | Exit |
|---------|------|
| status **ok** or **degraded** | **0** |
| **`--fail-on-degraded`** when status is degraded | **1** (promotes degraded) |
| status **fail** | **1** |
| Clap usage on doctor flags | **2** |

### Daemon status

`ai-brains daemon status` exits **0** for both Running and Stopped (liveness report, not a failure).

### Device status

`ai-brains device status` exits **0** for empty and enrolled vaults (roster report, not a failure — like `daemon status`). Unexpected extra args / unknown `--format` stay generic clap **2**.

### Nightly status

`ai-brains nightly --status` (human or `--format json`) exits **0** when probes are down / timeout / missing action / nonzero Last Result / Router 267009 (status report, not a failure — like `device status`). `--format` without `--status`, unknown `--format`, or `JSON`/`Pretty` → clap exit **2**.

### Path aliases (T254)

| Outcome | Exit |
|---------|------|
| `project list-paths` empty or populated | **0** |
| `project list-paths --project` / `--shared-only` empty filter | **0** |
| `project list-paths --project` unknown dest | **1** |
| `project unregister-path` missing path (idempotent) / `--dry-run` | **0** |
| `project scan-roots` (hits, empty, truncated) | **0** |
| `project rebind-path` print-only / `--write --yes` / already-bound | **0** |
| `project rebind-path` no owner / dest missing | **1** |
| `project register-path` other-owner conflict | **1** |
| `project unregister-path --project` owner mismatch | **1** |
| Empty path after normalize / unknown `--format` / clap usage / `rebind-path --write` sans `--yes` | **2** |

### Exit 130 (OS footnote)

**130** is the conventional shell/OS code for termination by SIGINT (Ctrl-C). It is **not** a product-defined code in the 0–7 table and is not mapped by `exit_code_for_api_error`.

### Vault / key codes (handle_cli_result exception)

Missing, invalid-format, zero-without-allow, and locked vault paths emit structured codes such as `VAULT_KEY_MISSING`, `VAULT_KEY_FORMAT`, `VAULT_KEY_ZERO`, `VAULT_LOCKED` and exit **1**.

These are handled in `main::handle_cli_result` with a **hardcoded exit 1** path. They **do not** go through `exit_code_for_api_error` (that map would also yield 1 via catch-all for unknown codes, but the vault path is an explicit exception so codes stay stable independent of the map).

## Dual error envelopes and streams

AI-Brains does **not** force a single error envelope shape. Format and path matter:

| Path | Shape | Stream |
|------|-------|--------|
| Governed **Json** (`fail_api` / `emit_error`) | bare `ApiError` (`code`, `message`, optional `details`) | **stdout** |
| Governed **Human** / Markdown | `CODE: message` | **stderr** |
| Generic `handle_cli_result` (non-governed failures) | full `ApiResult` error wrapper | **stderr** always |

Do **not** document “all failures always emit JSON on stderr” — that is false for governed Json mode.

### POLICY_DENIED remediation

On **`policy check`** deny and local **list** denies (`review list`, `source list`, `evidence list`), Json envelopes carry a non-empty structured **`details.hint`** string. Prefer **`ai-brains policy bootstrap --dry-run`** then **`ai-brains policy bootstrap`** (omit `--scope` when project context is authoritative) to register the principal (if needed) and issue discovery grants (`ReadEvidence`, `ReadConclusions`, `ReadDecisions`). Explicit `--scope Repository:<uuid>` remains valid for no-context CI (`--no-project-context`). Soft: source/evidence **show** deny also attaches the hint when touched by T203. Other deny sites may still emit bare `POLICY_DENIED` without `details`. Message remains terse. Exit stays **3**.

**T292 `policy check` human deny:** `--format human` / TTY `auto` prints two **stdout** lines (`denied: {cap}` + bootstrap SHORT) and exits **3** with empty stderr (skips `emit_error`). JSON deny (`--format json` / pipe `auto`) stays one ApiError document on stdout.

**Human / Markdown** governed deny (`emit_error`): after the `CODE: message` stderr line, a non-empty `details.hint` is printed on the next stderr line (T221 F5).

### Progressive / expand deny (T221)

| Command | Deny signal | Exit | Streams |
|---------|-------------|------|---------|
| `query progressive` | packet `denied: true` (incl. `--dry-run`) | **3** | **stdout:** pretty `ProgressiveQueryResponse` (keeps `denied` / `denial_reason` / additive **`denial_hint`** bootstrap string plus ungoverned `recall` fallback). **stderr:** `POLICY_DENIED: …` then bootstrap hint then `Ungoverned vault search: ai-brains recall "…"`. `next_step` omitted. |
| `query expand` | preview `kind == "Denied"` (capability miss **and/or** cross-scope — not disambiguated) | **3** | **stdout:** preview JSON. **stderr:** `POLICY_DENIED: …` then bootstrap hint. |
| `query expand` | `kind == "Unknown"` (handle not found) | **0** | not a policy wall |
| `briefing project` / `personal` | soft packet `denied: true` | **0** | unchanged (T210 F28) — do not treat like progressive |
| `briefing project` / `personal` | unknown `--format` (not human/pretty/text/markdown/md/json) | **2** | **T227:** `fail_usage` on stderr with accepted list; **zero stdout** (no silent JSON) |

Authorized progressive with grants and zero hits stays **`denied: false`**, empty `results`, exit **0** (T221 F14 — true empty *governed* knowledge). **T243 / T290:** that packet includes additive **`next_step`** that is copy-paste `ai-brains recall "<operator query>"` plus `(Pinned: N)` when COUNT succeeds (`denial_hint` omitted). Hits omit `next_step`. Denied stderr still prints the T243 ellipsis const. Empty governed ≠ empty vault. Authorized-empty `evidence` / `source` / `review` list is the same exit **0** + informative `next_step` (lists use `what did we decide`).

Invalid `AI_BRAINS_PROJECT_ID`: **`recall` / `search`** → clap **exit 2**; **`sync query`** → exit **0** with `Scope: project=(none)` (T231 F36 — not converged).

## Missing required `--scope` (F4 / F35 / T203)

After T201, CLI commands that always need a scope use **clap-required** `--scope: String` so forgetting the flag exits **2** (English clap usage on stderr), not **6**.

Still clap-required after T226: `erasure request`, `erasure wipe`, `review resolve` (destructive / mutate / CE). **T241:** `policy check --capability` is **optional at clap**; omit → runtime **`fail_usage` exit 2** with discovery-first capability catalog (not clap “required arguments were not provided”). Only `--scope` softens via soft-resolve; capability omission is a separate catalog usage path.

**T203/T226 soft-default:** `--scope` is **optional** on `review list`, `source list|show`, `evidence list|search|show`, and **`policy show|check|bootstrap`**. Missing + non-authoritative → runtime **`fail_usage` exit 2** (template class, not clap text). Authoritative context (e.g. `AI_BRAINS_PROJECT_ID`) may soft-fill. **Do not** reintroduce exit-6 missing-scope on these CLI paths.

**F35 — daemon / raw IPC honesty:** HTTP, named-pipe, or other non-CLI callers that omit scope may still receive **`INVALID_PAYLOAD` (exit/map 6)** from defensive daemon arms. The CLI always sends a filled scope after soft-resolve (or fails before send).

Malformed *provided* scope values (bad identity key shape, unknown capability labels, etc.) remain **6** / control-plane class errors — only the *missing / non-authoritative soft-resolve* class is usage **2**.

## Related

- [OPERATIONS.md](OPERATIONS.md) — operator CLI reference  
- [CAPABILITIES.md](CAPABILITIES.md) — feature inventory  
- T160 governed surface; T192 doctor; T197 vault key; T198 `FEATURE_UNAVAILABLE`→2; T252 ingest empty/TTY `fail_usage`
