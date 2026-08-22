# T282 review log — `context --show` leftover shell vs `.env`

**Track:** T282-ContextShowLeftover
**Status:** Completed (full gate green; Phase 6 pending this commit)
**FEATURE TX:** `c93b313e-4270-446f-bce5-eb80f8fec7f0`
**HEAD (implement):** `track/T282-context-show-leftover`

## Reviewers / rounds

| Round | Reviewer | Result |
|-------|----------|--------|
| R1 | Implementer (Grok) vs spec AC1–AC14 / DoD | **PASS** — red then green; F1 leftover after `Repository:`; KEY/VAULT_KEY redact; `--no-project-context` hermetic locks file vs captured shell |
| R1b | Explore subagent (read-only DoD) | **PASS** — AC1–AC14 Met; no P0–P3 product findings; process (gate/closeout) expected open |
| CX1 | Codex gpt-5.6-luna | **Product PASS** — no P0–P3; process (full gate / closeout / Phase 6) open as expected |
| Gate | `scripts/dev-check.ps1` + `ledgerful verify --scope full` | **PASS** nextest **3333** / 1 skipped |

## Finding fields

id, severity, description, source, files, required_fix, status, evidence.

## Findings

| id | severity | description | source | files | required_fix | status | evidence |
|----|----------|-------------|--------|-------|--------------|--------|----------|
| — | — | R1: no product findings | R1 | — | — | — | AC1 prefix 27 / suffix 17 / line 80; AC2 None rstest + padded F33; AC3 KEYRING passthrough; hermetic leftover count==1; dummy KEY leak; no-write; no-env suffix; `--no-project-context` leftover vs file |
| — | — | R1b: no product findings | R1b | — | — | — | AC1–AC14 Met; forbidden files unhooked; F36 rustdoc present |
| P1-1 | high (process) | Full gate / ledger / conductor closure unfinished at CX1 | CX1 | conductor + review.md | Run gate + closeout | `verified_fixed` | `dev-check` nextest **3333** (1 skipped); `ledgerful verify --scope full` exit 0; closeout this commit; Phase 6 remaining |

## DoD matrix (AC1–AC14)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | `format_shell_leftover_line__known_uuid__frozen_80` — prefix `"shell leftover PROJECT_ID: "` 27 chars; suffix `" (.env overrides)"` 17 chars; 36-char UUID line 80 chars; does not start with `Warning:` |
| AC2 | Met | Differ → Some(format); rstest None (same / missing / empty); `file_project_id_from_env_text` padded `"  {uuid}  "` → Some(unpadded) |
| AC3 | Met | KEY / VAULT_KEY / bare names → `(redacted)`; PROJECT_ID passthrough; comment/LEDGERFUL skip; KEYRING / VAULT_KEY_PATH passthrough |
| AC4 | Met | Hermetic `isolate_empty_home`; leftover exact string **once** after `Repository:`; file PROJECT_ID dumped; leftover on stdout |
| AC5 | Met | Shell == file → no prefix, no `(.env overrides)` |
| AC6 | Met | Dummy KEY in file → `AI_BRAINS_KEY=(redacted)`; no `AI_BRAINS_KEY=x'`; `assert_no_secret_leakage`; vault still zero KEY |
| AC7 | Met | `--show` and `--show --new-project` leave `.env` bytes unchanged; header present; no “initialized” / “Local .env updated” |
| AC8 | Met | No `.env` + leftover shell → existing `No .env file found` sentence; no `(.env overrides)` |
| AC9 | Met | `project_identity_convergence` 8 passed including `project_whoami__env_differs_path__mismatch_true` (`shell_project_id`) |
| AC10 | Met | `cargo run -p ai-brains-cli --quiet -- context --show`: `PROJECT_ID=3581317d-…`; leftover `7d97a456-… (.env overrides)`; no `x'`; exit 0. `project whoami` JSON `shell_project_id=7d97a456-…`; `mismatch: false`. Did not write `.env`. Did not `cargo install`. |
| AC11 | Met | CAPABILITIES Show-only + OPERATIONS `--show` leftover + redact; CHANGELOG T282. PROTOCOL-COMPAT untouched. CLI-EXIT-CODES untouched. `.claude` skill leftover one-liner on `:50`/`:57`/`:88`. `.agents` skill no `context` match (unchanged) |
| AC12 | Met | Diff omits `project.rs` / `sync.rs` / `forget.rs` / `env_warn.rs` / `main.rs` / Cargo.toml / lock / contracts. Calls existing `shell_project_id_captured`. F36 rustdoc on `SHOW_REDACTED_VAULT_KEY`. No clap/rusqlite bump |
| AC13 | Met | `cli_help_secret_redaction` 7 passed (`[env: AI_BRAINS_KEY]` without `=`) |
| AC14 | Met | `env_override_session_quiet` 8 passed |

## Targeted gates (R1)

```text
cargo nextest run -p ai-brains-cli leftover_shell map_show_env format_shell_leftover file_project_id
  16 passed (includes helper units)
cargo nextest run -p ai-brains-cli --test context_show_leftover
  6 passed (AC4–AC8 + --no-project-context file-vs-process lock)
cargo nextest run -p ai-brains-cli --test project_identity_convergence --test cli_help_secret_redaction --test env_override_session_quiet
  24 passed
cargo clippy -p ai-brains-cli --all-targets -- -D warnings
  exit 0
cargo fmt --check -p ai-brains-cli
  exit 0
```

## Manual (classify-only)

```text
cargo run -p ai-brains-cli --quiet -- context --show
  --- Current Context ---
  AI_BRAINS_PROJECT_ID=3581317d-601e-44f7-ab84-fde90aa12d3c
  Repository: C:\dev\AI-Brains
  shell leftover PROJECT_ID: 7d97a456-f2f4-43ea-1f13-211af684ad37 (.env overrides)
  exit 0
  no x'
  Did not write .env. Did not cargo install.

cargo run -p ai-brains-cli --quiet -- project whoami
  shell_project_id: 7d97a456-f2f4-43ea-1f13-211af684ad37
  effective/env/path/detect: 3581317d-…
  mismatch: false
```

PATH `ai-brains` remains until `cargo install` (F13). Source/`cargo run` is DoD.
