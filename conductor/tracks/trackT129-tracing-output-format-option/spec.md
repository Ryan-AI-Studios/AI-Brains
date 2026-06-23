# Track T129: Tracing Output Format Option

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — UX; default tracing format is verbose, no compact/json option.
**Source:** v0.1.1 verification opportunity #8.

---

## Problem Statement

T118 migrated 50 `eprintln!` calls to `tracing::info!`/`warn!` with a scoped EnvFilter. The default `tracing_subscriber::fmt()` format is verbose:

```
2026-06-23T14:50:09.430354Z  INFO ai_brains::commands::backup: Creating vault backup...
```

This is fine for debugging but noisy for normal use. There's no way to control the format without setting `RUST_LOG` env var (which controls level, not format). Pipeline consumers and log aggregation tools would benefit from a JSON format option.

## Acceptance Criteria

**AC1:** A global `--log-format <FORMAT>` CLI flag controls the tracing output format. Values: `compact` (default), `full` (current verbose format), `json` (structured JSON for log aggregation), `off` (suppress all tracing).

**AC2:** `compact` format shows: `INFO backup: Creating vault backup...` — timestamp and target shortened.

**AC3:** `json` format emits one JSON object per line: `{"timestamp":"2026-06-23T14:50:09.430Z","level":"INFO","target":"ai_brains::commands::backup","message":"Creating vault backup..."}`

**AC4:** `off` suppresses all tracing output (useful for scripts that only care about stdout JSON).

**AC5:** `RUST_LOG` env var still controls the level filter regardless of format. `--log-format` only controls formatting.

**AC6:** The default format changes from `full` to `compact` — less verbose for normal use.

## Design Notes

- **File:** `crates/ai-brains-cli/src/main.rs` — tracing subscriber init.
- Add `--log-format` to the global `Cli` struct (not per-subcommand).
- Map format to `tracing_subscriber::fmt` builders:
  - `compact` → `.with_target(false).compact()`
  - `full` → default `.fmt()` (current behavior)
  - `json` → `.json()` (tracing_subscriber's JSON formatter)
  - `off` → set EnvFilter to `off`
- The flag must be parsed BEFORE the tracing subscriber is initialized. Since clap parses all args first, this works — just use the parsed value in the subscriber init.
- `--log-format` and `RUST_LOG` are independent: format controls appearance, `RUST_LOG` controls which levels pass the filter.

## Files

- `crates/ai-brains-cli/src/main.rs` — Add `--log-format` flag, update subscriber init.

## Tests (TDD)

**Red:** `log_format_compact__short_output` — run a command with `--log-format compact`, capture stderr, assert tracing lines don't contain full timestamp (`2026-06-23T14:50:09.430354Z`).

**Red:** `log_format_json__valid_json_lines` — run a command with `--log-format json`, capture stderr, assert each tracing line is valid JSON with `timestamp`, `level`, `target`, `message` fields.

**Red:** `log_format_off__no_tracing_output` — run a command with `--log-format off`, capture stderr, assert no tracing lines.

**Green:** Implement format selection. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains --log-format compact backup create` → short tracing lines.
- Manual: `ai-brains --log-format json backup create` → JSON lines on stderr.
- Manual: `ai-brains --log-format off backup create` → no tracing noise.

## Out of Scope

- Adding structured fields to all tracing calls (T118 focused on migration; structured fields are a separate effort).
- Custom format templates (e.g. user-defined format strings).
- Log file output (tracing goes to stderr only; file logging is a separate concern).
- Changing `println!` (stdout) output format.