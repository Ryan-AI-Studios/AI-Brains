# Track review: T282-ContextShowLeftover

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT282-context-show-leftover`  
**Date:** 2026-08-22  
**HEAD:** `d370ea1`  

---

## Summary

Track T282 addresses an honesty and security gap in `ai-brains context --show`:
1. **Leftover Shell `PROJECT_ID` Visibility (F1):** While `ai-brains project whoami` reports `shell_project_id` when the inherited parent shell's `AI_BRAINS_PROJECT_ID` differs from the local `.env` file, `ai-brains context --show` only dumped the `.env` contents and `Repository:` path, omitting any indication that a leftover shell ID existed. Because T242’s stderr override warning is session-quiet after its first emission, subsequent `--show` runs gave no indication of the inherited shell ID.
2. **Secret Redaction Litmus (F3):** Previously, `context --show` printed every line in `.env` starting with `AI_BRAINS_` verbatim. If an operator placed an `AI_BRAINS_KEY` or `AI_BRAINS_VAULT_KEY` directly inside their repository `.env`, running `--show` would dump the plaintext secret directly onto the terminal.

T282 resolves both issues:
- **Leftover Line (F1):** When pre-dotenv shell `PROJECT_ID` is present, `.env` contains an `AI_BRAINS_PROJECT_ID`, and the two differ, `--show` prints a dedicated line after `Repository:`:
  ```
  shell leftover PROJECT_ID: <uuid> (.env overrides)
  ```
- **Key Redaction (F3):** Lines beginning with `AI_BRAINS_KEY=` or `AI_BRAINS_VAULT_KEY=` in `.env` are masked as `AI_BRAINS_KEY=(redacted)` and `AI_BRAINS_VAULT_KEY=(redacted)`.
- **Read-Only Invariant (F2):** Affirms T240 F2; `--show` executes strictly as a read-only dump and never modifies or generates `.env` files.

The plan is well-bounded, maintains capture independence, and leaves hotspots (`project.rs`, `sync.rs`, `forget.rs`) untouched.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Parsing `file_project_id` from `.env` in `context.rs` (F1 / AC2):** Ensure the helper extracting `file_project_id` from `.env` properly strips `"AI_BRAINS_PROJECT_ID="` and trims whitespace before calling `leftover_shell_vs_file`.
- **m2: Precise redaction matching in `map_show_env_line` (F3 / AC3):** Ensure `map_show_env_line` targets exact prefix matches (`AI_BRAINS_KEY=` / `AI_BRAINS_VAULT_KEY=`) or exact bare lines to avoid unintentionally masking unrelated future variables.

### Opportunities (O)
- **O1: Secret leakage assertions in hermetic tests (AC6):** In `tests/context_show_leftover.rs`, utilize `ai_brains_crypto::test_support::assert_no_secret_leakage` to verify that dummy keys in `.env` never leak to stdout or stderr.
- **O2: Pure unit tests for helpers (AC1–AC3):** Provide direct unit tests in `context.rs` testing `leftover_shell_vs_file` and `map_show_env_line` across all edge cases (matching IDs, differing IDs, empty IDs, missing IDs, and comments).

---

## What Looks Solid

1. **Alignment with `whoami` Identity Truth:** Brings `context --show` into alignment with `project whoami` without altering whoami's JSON contracts or adding unnecessary dependencies.
2. **12-Factor Security Compliance:** Adheres to 12-factor configuration principles by redacting key material on configuration dumps.
3. **Strictly Additive and Zero-Mutation:** Does not write `.env`, does not add new clap flags, and preserves existing `--show` exit codes.
4. **Hotspot Restraint:** Zero modifications to `project.rs` (calls existing `shell_project_id_captured`), `env_warn.rs`, or `sync.rs`. Edits are strictly isolated to `crates/ai-brains-cli/src/commands/context.rs`.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| `context --show` misses leftover shell vs `.env` | Absorbed into DoD (F1–F4 / AC1–AC5 / AC10) | Solved via conditional stdout leftover line |
| Redact `AI_BRAINS_KEY` on stdout dump | Absorbed into DoD (F3 / AC3 / AC6) | Solved via `map_show_env_line` redaction |
| Path-mismatch line on `--show` | Declined (F10) | Handled by stderr and `whoami`; cwd is `mismatch: false` |
| `--format json` / vault-free `--show` | Declined (F4 / F11) | Preserves existing dump format and dispatch |
| Last-PR Cursor #197 | N/A (empty) | Scanned with 0 findings |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#197](https://github.com/Ryan-AI-Studios/AI-Brains/pull/197) (merged 2026-08-22, T281 `Nightly timeout next-line HTTP vs TCP`).
- **Cursor Comments:** 0 comments (`[]` on PR #197).
- **Disposition:** N/A (no pending findings).

---

## Research / Tools Notes

- **Configuration Output Guidelines:** 12-factor config guidelines and clig.dev recommend masking sensitive credentials whenever configuration files or environments are dumped to terminal output.
- **Dependencies:** `clap` (4.6.1), `serde_json` (1.0.150), `rusqlite` (0.39.0), `chrono` (0.4.44), `uuid` (1.23.1), `tokio` (1.52.3).
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,619 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search shell_project_id_captured`: Located at `crates/ai-brains-cli/src/commands/project.rs:160`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
