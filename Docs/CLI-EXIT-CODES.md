# CLI exit codes (normative)

**Source of truth** for `ai-brains` process exit codes. Linked from [OPERATIONS.md](OPERATIONS.md), [CAPABILITIES.md](CAPABILITIES.md), and root [CONTRIBUTING.md](../CONTRIBUTING.md).

Implementation map: `crates/ai-brains-cli/src/commands/governed_common.rs` (`EXIT_*`, `exit_code_for_api_error`, `fail_api`, `fail_usage`).

## Normative table (product codes 0–7)

| Exit | Constant | When |
|------|----------|------|
| **0** | `EXIT_SUCCESS` | Success; empty-success; `daemon status` report (Running or Stopped); `doctor` **ok** or **degraded** (unless `--fail-on-degraded`) |
| **1** | `EXIT_INTERNAL` | Internal / catch-all; `PATH_REFUSED`; `COMMAND_FAILED`; `INVALID_TRANSITION`; vault/key codes (`VAULT_KEY_*` / `VAULT_LOCKED`); `doctor` **fail** |
| **2** | `EXIT_USAGE` | Clap missing/invalid usage (e.g. missing required `--scope`); `FEATURE_UNAVAILABLE` (e.g. default-build `graph *`); `fail_usage` for `query progressive` / `query expand` missing project id |
| **3** | `EXIT_POLICY_DENIED` | `POLICY_DENIED`; `APPROVAL_REQUIRED` |
| **4** | `EXIT_NOT_FOUND` | `NOT_FOUND` |
| **5** | `EXIT_DAEMON_UNAVAILABLE` | Daemon required / unreachable for a daemon-required path |
| **6** | `EXIT_INVALID_PAYLOAD` | `INVALID_PAYLOAD`; `NOT_ENVELOPE_BACKED`; malformed provided values (ids, unknown capability, bad JSON, dual-flag wipe, dogfood missing file, …) |
| **7** | `EXIT_HARD_GATE_FAILED` | Evaluate trust hard gates failed (harness ran; product blocked) |

**No product exit codes 8+.** Scripts must not invent higher codes as contract.

### FEATURE_UNAVAILABLE → 2

Optional features not compiled into this binary (notably default-build `graph *` without `--features graph`) exit **2** with a human `FEATURE_UNAVAILABLE:` prefix (and reinstall hint). Same numeric class as clap usage.

### fail_usage → 2 (T202)

`query progressive` and `query expand` require `--project-id` or `AI_BRAINS_PROJECT_ID`. When both are unset, `fail_usage` writes a copy-paste example to **stderr** and exits **2** via `GovernedCliError` / `EXIT_USAGE` (not clap-required; not exit 1). `query trace` is excluded.

### Doctor (footnote)

| Outcome | Exit |
|---------|------|
| status **ok** or **degraded** | **0** |
| **`--fail-on-degraded`** when status is degraded | **1** (promotes degraded) |
| status **fail** | **1** |
| Clap usage on doctor flags | **2** |

### Daemon status

`ai-brains daemon status` exits **0** for both Running and Stopped (liveness report, not a failure).

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

On **`policy check`** deny (and local **`review list`** deny), Json envelopes carry a non-empty structured **`details.hint`** string (e.g. point operators at `ai-brains policy show --scope …`). Other deny sites may still emit bare `POLICY_DENIED` without `details` (not yet universal). Message remains terse. Exit stays **3**.

## Missing required `--scope` (F4 / F35)

After T201, CLI commands that always need a scope use **clap-required** `--scope: String` so forgetting the flag exits **2** (English clap usage on stderr), not **6**.

Flipped in T201: `policy show`, `review list`, `erasure request` (and peers already required, e.g. `policy check`, `erasure wipe`).

**F35 — daemon / raw IPC honesty:** HTTP, named-pipe, or other non-CLI callers that omit scope may still receive **`INVALID_PAYLOAD` (exit/map 6)** from defensive daemon arms. The CLI always sends scope after F4.

Malformed *provided* scope values (bad identity key shape, unknown capability labels, etc.) remain **6** / control-plane class errors — only the *missing required flag* class is usage **2**.

## Related

- [OPERATIONS.md](OPERATIONS.md) — operator CLI reference  
- [CAPABILITIES.md](CAPABILITIES.md) — feature inventory  
- T160 governed surface; T192 doctor; T197 vault key; T198 `FEATURE_UNAVAILABLE`→2  
