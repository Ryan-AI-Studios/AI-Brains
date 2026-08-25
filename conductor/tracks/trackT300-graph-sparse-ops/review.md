# T300 Review Log — graph sparse ops

**Track:** T300-GraphSparseOps  
**Status:** Phase-1 + Codex addressed; full gate green  
**Implementer:** Grok  
**HEAD (work):** `track/T300-graph-sparse-ops`

## Findings

| id | severity | description | source | files | required_fix | status | evidence |
|----|----------|-------------|--------|-------|--------------|--------|----------|
| R1 | medium | Silent rebuild stdout left operators unsure if remediator worked | implementer | `graph.rs` | Emit density after mutate / dry-run | `verified_fixed` | Hermetic AC1/AC2 + live `--dry-run` prints `status:`/`nodes:` + `[dry-run]` |
| R2 | high | Mutating rebuild while daemon Running races LiveGraphHook | implementer / F7 | `graph.rs` | Safety `probe_restore_daemon_busy` fail-closed | `verified_fixed` | Unit AC3 Err + T188 substrings; live dry-run NOTICE while daemon Running |
| R3 | medium | Dual-truth update vs rebuild health JSON | implementer / F27 | `graph.rs` | Shared `graph_health_report` | `verified_fixed` | Unit AC2 status equality; AC5 keys; serde stay-green |
| R4 | low | Live mutate skipped (owner confirmed skip) | F1 / AC14 | plan evidence | Written skip; hermetic DoD | `deferred` | Owner chose skip live mutate; hermetic + dry-run SoT |
| R5 | low | PATH 0.1.2 lacks T300 UX until install | F17 | — | Document residual | `deferred` | Manual via `cargo run --features graph` |
| R6 | low | `read_all_events` full Vec RAM on large vault | F9 / F25 | `rebuild.rs` frozen | Residual only | `deferred` | Engine freeze; after_help minutes honesty |
| R7 | low | Mid-rebuild daemon start / crash TOCTOU | F25 | — | Re-run rebuild; probe≠atomic DELETE | `deferred` | Fold-in OpenCode O1; `rebuild_is_idempotent` stay-green |
| R8 | low | JSON dry-run omits `dry_run` key by design | F10 | — | Human-only extras | `deferred` | AC9 hermetic asserts no `[dry-run]` on JSON stdout |
| C1 | medium | Codex: for-loops in new tests vs rstest | Codex P2 | `graph_rebuild_ops.rs` / `exit_contract.rs` / `graph.rs` | Convert to rstest / split tests | `verified_fixed` | AC9 rstest cases; AC7 split tests; AC10 rstest needles |
| C2 | medium | Codex: AC4 NOTICE stdout not asserted in unit | Codex P2 | `graph_rebuild_ops.rs` | Hermetic assert NOTICE when present | `verified_fixed` | AC1 asserts T188 substrings when `NOTICE:` present |
| C3 | low | Codex: AC2 hermetic early-return when daemon up | Codex P2 | `graph_rebuild_ops.rs` | Unit inject is SoT; keep honest early-return | `deferred` | Not easy without stopping daemon; unit AC2 covers mutate |
| C4 | medium | Codex: AC14 missing doctor agree | Codex P2 | review evidence | Record `doctor --summary` | `verified_fixed` | doctor `graph_density` warn + remediator matches update sparse |

## AC checklist (internal)

| AC | Result |
|----|--------|
| AC1 dry-run density no mutation | PASS hermetic |
| AC2 mutate density + RECALLS + status eq | PASS unit inject + hermetic (daemon-up early-return / CI path) |
| AC3 daemon-up mutate Err | PASS unit |
| AC4 daemon-up dry-run Ok | PASS unit + live NOTICE |
| AC5 JSON keys | PASS unit |
| AC6 T262 stay-green | PASS |
| AC7 feature-off exit 2 | PASS |
| AC8 floors + doctor 15 | PASS (`health_check_order_names`; floors file untouched) |
| AC9 clap tokens | PASS hermetic |
| AC10 inject matrix + busy message | PASS unit |
| AC11 after_help | PASS hermetic |
| AC12 docs | PASS CAPABILITIES/OPERATIONS/WORKFLOWS/CHANGELOG/CLI-EXIT-CODES/PROTOCOL-COMPAT |
| AC13 frozen engines | PASS (`rebuild.rs` / `graph_density.rs` / `doctor.rs` production untouched) |
| AC14 manual | PASS `--dry-run` live; **mutate skipped** (owner) |
| AC15 update human stay-green | PASS live dogfood |
| AC16 peers | PASS targeted T213/T232/T262 |

## Manual evidence (AC14)

```text
cargo run -p ai-brains-cli --features graph -- graph rebuild --dry-run
→ status: sparse … edge_node_ratio: 0.149… remediation: ai-brains graph rebuild
→ NOTICE: … daemon is running … ai-brains daemon stop … sc stop AI-Brains-Daemon
→ [dry-run] would DELETE … replay 57912 events; no mutation.

cargo run -p ai-brains-cli --features graph -- graph update --format human
→ same sparse status / remediator (agree)

cargo run -p ai-brains-cli --features graph -- doctor --summary
→ graph_density warn — sparse E/N 0.149 + remediation: ai-brains graph rebuild
→ (agrees with update / dry-run; other warn recovery_kit_event unrelated)

cargo run -p ai-brains-cli --features graph -- graph rebuild --format auto
→ clap exit 2 invalid value 'auto'
```

Live mutate: **skipped** (owner confirm 2026-08-25). Not a floor lie.

## Gates

- Targeted clippy + nextest graph suite: PASS
- `scripts/dev-check.ps1`: **PASS** (3528 nextest; deny; audit)
- `ledgerful verify --scope full`: **PASS**
- Frozen files `rebuild.rs` / `graph_density.rs` / `doctor.rs`: **CLEAN** (`git diff --exit-code`)

## Cross-model

- Codex `review.codex.md`: initial FAIL on process + P2 test/evidence gaps → fixed C1–C2/C4; C3 deferred (daemon-up hermetic early-return; unit inject SoT).
- Subagent audit: PASS WITH DEFERRED P3 (aligned with R4–R8 / C3).

## Contracts

No `ai-brains-contracts` change. No pin bumps. No new crates.
