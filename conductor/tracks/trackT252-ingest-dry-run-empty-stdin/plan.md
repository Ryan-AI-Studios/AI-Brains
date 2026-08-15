# T252 Plan — Ingest dry-run empty stdin honesty

**Status:** ✅ **Completed** (2026-08-15; CX2 PASS)  
**Spec:** [spec.md](./spec.md) F1–F16 / AC1–AC16 + §12 AI fold-in  
**Category:** UX / BUGFIX  
**Ledger TX (planning):** `6cc460e2-6f28-40bb-bb38-0ca787ec4985` (DOCS)  
**Ledger TX (fold-in):** `33e14b3a-8ec3-402d-bfdb-8c40b2460873` (DOCS)  
**Ledger TX (on go):** `ledgerful ledger start T252-ingest-dry-run-empty-stdin --category UX --message "empty/TTY ingest stdin = fail_usage exit 2 + example JSON; mid-payload COMMAND_FAILED frozen"`

---

## AI fold-in (2026-08-15) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 restates planned work. AI2 quoting + wrap are must-pins. Disposition in spec **§12**.

### Pins locked by fold-in

1. `echo '{…}'` single-quoted JSON (not unquoted ScriptBlock).
2. `after_help` multiline; AC8 = keys only.
3. AC14(3) live TTY no-hang.
4. Phase 3 grep hooks / `ingest.ps1` / `COMMAND_FAILED` before BREAKING changelog.
5. Keep F6 `is_tty` param. Const stays in `ingest.rs`.

---

## Preflight (plan time — 2026-08-15)

| Check | Result |
|-------|--------|
| `echo '' \| ai-brains ingest --dry-run` | `COMMAND_FAILED` / `Invalid JSON: EOF while parsing a value at line 2 column 0` → **exit 1** |
| `ai-brains ingest --help` | stdin mentioned; **no** example payload |
| TTY `ingest --dry-run` | hangs on `read_to_string` (not dogfooded to completion) |
| `ingest.rs` | dry-run `from_str` before any empty check; `ctx` unused on dry-run |
| `handle_cli_result` | generic errors → exit **1** JSON; `fail_usage` → exit **2** |
| T86 | TTY + empty query already gated — pattern only |
| T114 / T180 tests | exist; no empty-stdin case |
| clap / serde_json / is-terminal | lock 4.6.1 / 1.0.150 / 0.4.17 — **no bumps** (crates.io clap **4.6.6**, serde_json **1.0.151**) |
| rustc / nextest | 1.95.0 / 0.9.140 |
| Ledger | 0 pending at scan; planning TX `6cc460e2` opened for these docs |
| Hotspots | `ingest.rs` **not** in top 10. `governed_common.rs` rank 9 — **call** `fail_usage`, do not rewrite |
| T243–T251 | Completed — no rewrite |
| `preflight --summary` | Scope `test-alias`; harnesses grok/agy/opencode ok; doctor degraded — unrelated |
| Embedding / completion :8083/:8081 | Capture-independent; not blocking |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| ingest dry-run empty stdin 5/7 | deferred.md / audit P3 | **DoD** F1–F7 |
| Placeholder F1 exit 2 + example | spec draft | **F1 / F5** |
| Placeholder F2 valid dry-run | spec draft | **F2** |
| Placeholder F3 mid-payload envelope | spec draft | **F3 / AC5** |
| Interactive hang | spec “or interactive” | **F4** TTY-before-read |
| Vault-free dry-run | code truth (`ctx` unused) | **Not absorbed** F12 / F9 |
| `ingest --schema` | T83 siblings | **Not absorbed** F12 |
| POSIX unquoted `echo {…}` | AI1 BS5 / AI2 BS1 | **Absorbed** F5 single quotes |
| TTY live dogfood | AI1 O4 | **Absorbed** AC14(3) |
| BREAKING consumer grep | AI1 O5 | **Absorbed** F7 / Phase 3 |
| T86 `read_json_from_stdin` swallow | AI1 BS1 | **Not absorbed** F12 |
| Shared stdin helper | AI1 O2 | **Not absorbed** F12 |
| `events[0]` panic | AI1 BS2 | **Not absorbed** F12 |
| Flatten F6 / const → `governed_common` | AI1 O1 / O3 | **Declined** |
| T253–T255 | peers | **Not absorbed** |

---

## Phase 0 — Ledger + impact (on go)

- [x] `ledgerful ledger status --compact` — 0 pending; implement TX `6f402b33-1e56-474e-a3c8-65aef5a1abbc`
- [x] `ledgerful ledger start T252-ingest-dry-run-empty-stdin --category UX`
- [x] `ledgerful scan --impact` — LOW; ingest.rs not a hotspot; call `fail_usage` only
- [x] Confirm no other agent is editing `ingest.rs` / `main.rs` Ingest / `ingest_reads_json_stdin.rs` / CLI-EXIT-CODES

---

## Phase 1 — Red (TDD)

- [x] Add `ingest_stdin_needs_usage` units (AC6) — compile against missing helper
- [x] Add hermetic AC1 empty `--dry-run` (exit 2 + example, no EOF JSON)
- [x] Add hermetic AC2 whitespace-only
- [x] Add hermetic AC3 live empty stdin
- [x] Add hermetic AC5 mid-payload `{` still COMMAND_FAILED exit 1
- [x] Add AC7 const assertions (seven keys + `ingest --dry-run` + `'{`)
- [x] Add AC8 `ingest --help` contains example **keys** (not exact wrap)
- [x] `cargo nextest run -p ai-brains-cli ingest_reads_json_stdin ingest_stdin` — Red then Green 12/12

---

## Phase 2 — Green

- [x] `INGEST_EMPTY_STDIN_USAGE` const in **`ingest.rs`** (F5 single-quoted `echo '{…}'`)
- [x] `ingest_stdin_needs_usage(is_tty, raw)` (F6 — keep `is_tty`)
- [x] `run()`: TTY → `fail_usage` **before** read; trim-empty → `fail_usage`; else existing dry-run / live paths
- [x] `Commands::Ingest` `after_help` with **multiline** JSON (F5 / AC8 keys only)
- [x] Do **not** touch `DryRunIngestRequest`, `parse_ingest_request`, or `handle_cli_result` generic map
- [x] Targeted: AC4 / AC9 / AC10 / AC11 still green
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`

---

## Phase 3 — Docs

- [x] Grep `COMMAND_FAILED` + empty `ingest` callers: `scripts/*.ps1`, `.agents/skills/ai-brains/scripts/ingest.ps1`, harness wrappers, `Docs/claude-hooks.md` samples. Record hits in this plan (AI1 O5). Expected: hooks pipe a built payload, not empty stdin.
- [x] Re-read `cli_help_ia.rs` — confirm still group-order only (AI1 BS4)
- [x] `Docs/CLI-EXIT-CODES.md` — `fail_usage` footnote for empty/TTY ingest
- [x] `Docs/CAPABILITIES.md` §4 — empty stdin → exit 2 + example
- [x] `Docs/OPERATIONS.md` — one-liner after ingest sample
- [x] `CHANGELOG.md` Unreleased — BREAKING empty-stdin 1→2
- [x] AC12 (docs portion; code ACs remain Phase 1–2)

### Phase 3 BREAKING grep (AI1 O5 / F7) — 2026-08-15

**No empty-stdin consumer of `ai-brains ingest` exists.** No script/doc parses ingest `COMMAND_FAILED` JSON. Hooks that call ingest pipe a built payload (or skip when content/ids are missing).

| File | Note |
|------|------|
| `.agents/skills/ai-brains/scripts/ingest.ps1` | **Absent** in this repo (hooks prefer it if present; README/docs mention it). |
| `scripts/target-claude-hook.ps1` | Builds `ConvertTo-Json` payload → temp file → pipe to `ai-brains ingest`. Returns early if no content / missing ids. |
| `scripts/target-codex-hook.ps1` | Same: built payload via temp file; no empty stdin. |
| `scripts/target-gemini-hook.ps1` | Same: built payload via temp file; no empty stdin. |
| `scripts/target-opencode-hook.ps1` | Same: built payload via temp file; no empty stdin. |
| `scripts/fixed-hook.ps1` | Pipes built `$ingestPayload` hashtable JSON to `ai-brains ingest`. |
| `scripts/Ingest-HermesVault.ps1` | Pipes built `ConvertTo-Json` object per file; never empty stdin. |
| `Docs/claude-hooks.md` | Sample constructs a full JSON hashtable then pipes it (temp-file note). |
| `Docs/codex-hooks.md` | Documents helper-then-CLI fallback; no empty-stdin sample. |
| Harness wrappers (`install.rs` AGY/Grok Stop scripts) | Empty stdin skips the **wrapper** (`Write-Skip 'empty stdin'`; exit 0). They never call `ai-brains ingest` with empty stdin (`agy-hook` / `grok-hook --payload`). |
| `COMMAND_FAILED` + ingest callers | **None** in `scripts/*.ps1` / hook docs. Only `Docs/CLI-EXIT-CODES.md` table + CLI `handle_cli_result` (`main.rs`). |

### Phase 3 `cli_help_ia` confirmation (AI1 BS4)

- `crates/ai-brains-cli/src/help_ia.rs` — group-order appendix only. `ingest` is listed under **Harness**. No ingest `after_help` / body snapshot.
- `crates/ai-brains-cli/tests/cli_help_ia.rs` — group-order only. `ingest` is a Harness **position** lock (`long_help__daily_commands_before_harness_ingest`). Does **not** snapshot `ingest --help` body. Not edited.

---

## Phase 4 — Review convergence

- [x] `conductor/tracks/trackT252-ingest-dry-run-empty-stdin/review.md` created
- [x] Primary review vs spec (completeness, regressions, placeholders) — R1 + R1b PASS
- [x] Resolve until no open critical/high; mediums fixed or justified (cap ≤3)
- [x] Cross-model: user-requested Codex CX1 (process P1 + P3 whitespace); P3 fixed; full gate + CX2 final pending

---

## Phase 5 — Manual + targeted verify

- [x] AC14 live: `echo '' \| target\debug\ai-brains.exe ingest --dry-run` → exit 2 + example
- [x] AC14 live: `echo '{' \| … ingest --dry-run` → exit 1 JSON `COMMAND_FAILED`
- [x] AC14 TTY: Python `CREATE_NEW_CONSOLE` `ingest --dry-run` — usage immediately, exit 2, 8s timeout not hit
- [x] Run pipe cases twice; human usage text byte-stable
- [x] AC15 targeted nextest + clippy
- [x] `ledgerful verify --scope full` — fmt/clippy/nextest ok; deny/audit missing locally (CI residual)

---

## Phase 6 — Full gate + finalize (closeout)

- [x] Workspace fmt + clippy + nextest **PASS** (2856 passed). `dev-check.ps1` / deny / audit missing local binaries (CI residual, T251 same)
- [x] Mark plan tasks complete; conductor T252 → **Completed**
- [x] deferred.md + README-T240-T255: T252 closed
- [x] Append F12 soft residuals to `conductor/deferred.md` (not ISSUES.md — file does not exist)
- [x] `ai-brains pin` memory `36db07a4-be81-4274-9283-be77f0f13f2d`
- [x] `ledgerful ledger commit` FEATURE/UX TX (after CX2 PASS)

---

## Isolation (do not touch)

- `ai-brains-contracts` ingest DTOs
- `ai-brains-capture` `parse_ingest_request`
- T180 deny_unknown_fields policy
- Graph / nightly / harness import paths
- `run_sync_path_free` / vault-path-free set
- T243–T251 product surfaces
- Live vault contents / daemon start
- Pin bumps / clap 5

---

## Proof tests (names)

| Test | AC |
|------|----|
| `ingest_stdin_needs_usage__tty_or_blank__true` | AC6 |
| `ingest_stdin_needs_usage__payload__false` | AC6 |
| `ingest_empty_stdin_usage__contains_example_keys` | AC7 |
| `ingest__dry_run__empty_stdin__usage_exit_2` | AC1 |
| `ingest__dry_run__whitespace_stdin__usage_exit_2` | AC2 |
| `ingest__live__empty_stdin__usage_exit_2` | AC3 |
| `ingest__dry_run__truncated_object__command_failed` | AC5 |
| existing T114 / empty-content / UUID / T180 | AC4 / AC9–AC11 |
| `ingest --help` hermetic or unit on after_help | AC8 |
