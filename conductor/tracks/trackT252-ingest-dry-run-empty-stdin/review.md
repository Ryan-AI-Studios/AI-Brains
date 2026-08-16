# T252 Review Log — Ingest dry-run empty stdin honesty

**Track:** T252-IngestDryRunEmptyStdin  
**Category:** UX / BUGFIX  
**Ledger TX:** `6f402b33-1e56-474e-a3c8-65aef5a1abbc`

## Reviewers / rounds

| Round | Reviewer | Verdict |
|-------|----------|---------|
| Internal R1 | completeness vs spec | **PASS** (0 findings) — `review.internal.r1.md` |
| Internal R1b | correctness / tests | **PASS** (0 findings) — `review.internal.r1b.md` |
| Completeness gate | product+docs vs F/AC | **COMPLETE** — `review.completeness.md` |
| Codex CX1 | gpt-5.6-luna high | **FAIL** P1 closeout evidence + P3 trailing whitespace — `review.codex.cx1.md` |
| Codex CX2 | gpt-5.6-luna high **fresh final** | **PASS** (0 P0–P3) — `review.codex.cx2.md` |

## CX1 dispositions

| ID | Classification | Action |
|----|----------------|--------|
| P1 Track closeout / full-gate evidence incomplete | Partly valid process; **not a product defect** | Addressed: plan checkboxes, this `review.md`, P3 whitespace, full gate recorded below. Conductor/deferred Completed after gate. Ledger commit on closeout. |
| P3 `Docs/CLI-EXIT-CODES.md:109` trailing whitespace | **Validated** easy P3 | **verified_fixed** — `git diff --check` clean |

## Final DoD

F1–F15 and AC1–AC15 met on product (AC16 N/A after go). Soft F12 residuals remain (not implemented). Isolation honored: no `IngestRequest`/`IngestResponse`/`parse_ingest_request`/T180/T114 rewrite, no pin bumps, no vault-free dry-run, `fail_usage` called not rewritten.

## Gates (orchestrator-observed)

| Gate | Result |
|------|--------|
| `cargo fmt --check` (touched Rust) | PASS |
| `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` | PASS |
| Targeted nextest `ingest_reads_json_stdin` + ingest units | **12/12 PASS** |
| Targeted nextest `protocol_compat_cli` + `cli_help_ia` | **11/11 PASS** (AC9 T180 + AC8 group-order) |
| `git diff --check` | PASS after P3 fix |
| AC14(1) empty pipe | exit **2**, usage example, no EOF JSON; repeat byte-stable |
| AC14(2) `echo '{'` | exit **1** `COMMAND_FAILED` / `Invalid JSON` |
| AC14(3) TTY `CREATE_NEW_CONSOLE` | exit **2** immediately; no hang |
| `cargo fmt --check` (workspace via `ledgerful verify --scope full`) | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo nextest run --workspace` | **2856 passed**, 1 skipped (78.7s targeted run; 86.1s in verify) |
| `ledgerful verify --scope full` | fmt/clippy/nextest **ok**; `cargo deny` / `cargo audit` **missing local binaries** (exit 101 `no such command`) — same residual as T251; CI jobs run both |

## Soft residuals (F12)

Vault-free `--dry-run`; `ingest --schema`; `std::io::IsTerminal`; clap 4.6 workspace pin; T86 `read_json_from_stdin` swallow; shared stdin helper; `outcome.events[0]` panic if `events` empty; T253–T255.

## Completion decision

Product engineering **clear** after CX2 fresh **PASS**. Conductor/deferred already mark T252 Completed. Ledger commit is the remaining closeout action.
