# T146 Review — Governed Memory Control Plane Architecture

**Reviewed:** 2026-07-23  
**Unblocked / re-verified:** 2026-07-24  
**Branch:** `docs/memory-control-plane-architecture`  
**Ledgerful transaction:** `0115f19b-6637-4d75-aa05-bd0871e3d15f` (Architecture)  
**Outcome:** **Complete** — documentation deliverables complete; full repository and Ledgerful gates green on native Windows

## 1. Deliverables reviewed

### Product and domain

- `CONTEXT.md`
- `Docs/MEMORY-CONTROL-PLANE-VISION.md`
- `Docs/DECISIONS/ADR-0010-evolve-ai-brains-into-successor.md`
- `Docs/DECISIONS/ADR-0011-separate-evidence-conclusions-decisions.md`
- `Docs/DECISIONS/ADR-0012-local-first-control-plane-and-public-protocol.md`
- `Docs/DECISIONS/ADR-0013-distinct-briefings-and-scope-hierarchy.md`
- `Docs/DECISIONS/ADR-0014-source-aligned-freshness-and-explicit-conflict.md`
- `Docs/DECISIONS/ADR-0015-event-ledger-erasure-and-encrypted-replication.md`

### Research

- `Docs/RESEARCH/memory-systems-comparison-2026-07.md`
- Cerebras four-file mechanism checked against both a timestamped transcript and a focused source-grounded NotebookLM query over video `TeGsFFNqRLA`.
- Karpathy's LLM Wiki pattern is separated from the independent Obsidian plugin implementation.
- OB1 claims are based on repository README, provenance policy, API, and schema sources.
- Mem0 OSS and managed-platform claims are distinguished; vendor benchmark claims are caveated with independent LoCoMo audit concerns.
- Scores define `10` as better in every column; overhead `10` means least burden.

### Implementation handoff

- `.hermes/plans/2026-07-23_204630-memory-control-plane-successor.md`
- 1,478 lines / 55,839 bytes after the post-fan-out codebase-map audit.
- Covers phases P0–P12.
- Names additive migrations `0020`–`0026`.
- Includes exact existing/new crate and file targets, RED/GREEN tests, rollout flags, migration/rollback constraints, branch/track boundaries, mandatory stop conditions, and MVP acceptance evidence.

### Navigation/governance

- `README.md`
- `conductor/conductor.md`
- this track's `spec.md`, `plan.md`, and `review.md`

## 2. Documentation validation evidence

A deterministic local validator checked 14 authored/modified Markdown files for:

- duplicate heading paths
- unresolved local Markdown targets
- incorrect next-day date strings relative to the 2026-07-23 decision date
- required implementation-plan sections and existing-file references

Result:

```text
files_checked: 14
errors: 0
```

Implementation-plan structural result:

```text
lines: 1478
phases: 0 through 12
migration mentions: 0020 through 0026
required existing paths missing: 0
mandatory stop conditions: present
MVP definition: present
errors: 0
```

`git diff --check` also returned exit code 0.

## 3. Changed-file scope

Intended T146 files:

```text
.hermes/plans/2026-07-23_204630-memory-control-plane-successor.md
CONTEXT.md
Docs/DECISIONS/ADR-0010-evolve-ai-brains-into-successor.md
Docs/DECISIONS/ADR-0011-separate-evidence-conclusions-decisions.md
Docs/DECISIONS/ADR-0012-local-first-control-plane-and-public-protocol.md
Docs/DECISIONS/ADR-0013-distinct-briefings-and-scope-hierarchy.md
Docs/DECISIONS/ADR-0014-source-aligned-freshness-and-explicit-conflict.md
Docs/DECISIONS/ADR-0015-event-ledger-erasure-and-encrypted-replication.md
Docs/MEMORY-CONTROL-PLANE-VISION.md
Docs/RESEARCH/memory-systems-comparison-2026-07.md
README.md
conductor/conductor.md
conductor/tracks/trackT146-governed-memory-control-plane-architecture/plan.md
conductor/tracks/trackT146-governed-memory-control-plane-architecture/review.md
conductor/tracks/trackT146-governed-memory-control-plane-architecture/spec.md
```

Pre-existing unrelated file, inspected but untouched:

```text
.agents/skills/codex-review/SKILL.md
```

It must not be staged or committed with T146.

### Separate gate-hygiene chore (not T146 architecture content)

Required to make the repository verification gate parse and pass under Windows PowerShell 5.1 / rustfmt. Landed as a separate commit from T146 docs:

```text
scripts/dev-check.ps1
crates/ai-brains-cli/src/artifact_security.rs
crates/ai-brains-cli/src/commands/daemon.rs
crates/ai-brains-cli/src/commands/nightly.rs
crates/ai-brains-cli/src/elevation.rs
crates/ai-brains-cli/src/main.rs
```

- `dev-check.ps1`: replace Unicode em-dashes (U+2014) with ASCII `-` so Windows PowerShell 5.1 can parse the UTF-8 file without a BOM.
- Rust files: pure `cargo fmt` (import order, line wrapping, Windows newline style). No behavioral changes.

## 4. Post-fan-out primary-source audit

A three-agent research fan-out completed after the initial review. Its summaries were treated as leads, not authority. Two summary claims conflicted with primary artifacts and were rejected:

- The subagent placed the Cerebras four-file segment at 14:30–15:40. The locally retained timestamped transcript shows the external-memory explanation beginning at 15:50, file names beginning at 15:57, and `VERIFY.md` completing at 16:37. The comparison retains the transcript-backed timestamps.
- The subagent dated Obsidian plugin v1.25.3 to 2026-07-22. GitHub's release API reports `published_at: 2026-07-23T15:44:59Z`; the comparison retains 2026-07-23.

The Mem0 fan-out correctly highlighted an OSS/platform boundary that was too loose in the comparison. Primary Mem0 docs confirm that OSS v3 removed `enable_graph` and external `graph_store` support, while the managed platform provides native always-on Graph Memory. The comparison was corrected so platform graph/temporal features are not attributed to OSS.

A later read-only codebase-mapping fan-out was checked against the canonical implementation plan. Its scratch map was not adopted wholesale because it was incomplete, proposed reusing already-applied migration numbers `0011`–`0019`, and suggested replacing legacy `Memory` rather than preserving additive compatibility. Two verified omissions were incorporated into P2:

- structured action/verification Evidence now explicitly reuses `ai-brains-capture/src/verification_gate.rs`, while distinguishing fail-open capture from a passed verification;
- governed provenance and conflict events now explicitly extend the existing `ai-brains-graph/src/projector.rs`, `rebuild.rs`, and `queries.rs`, with deterministic rebuild and degraded-mode tests.

## 5. Full repository gate evidence (2026-07-24 unblock)

**Gate environment:** native Windows, PowerShell 7.6 / Windows PowerShell 5.1, Cargo 1.95.0.

### Prior blockers and resolution

| Blocker | Prior evidence | Resolution (2026-07-24) |
|---------|----------------|-------------------------|
| `dev-check.ps1` parse fail under PS 5.1 | Unexpected `}` / unterminated string around lines 45/61/84 | UTF-8 em-dashes (U+2014) misdecoded by PS 5.1 without BOM. Replaced with ASCII `-`. `powershell.exe -File … -CheckOnly` now exit 0. |
| `cargo fmt --check` | Diffs in elevation/nightly/main | Applied `cargo fmt` only (also sorted imports in artifact_security/daemon). Separate chore commit. |
| Clippy | WSL-reported dead_code / unused PIPE_NAME | On native Windows, `cargo clippy --workspace --all-targets -- -D warnings` exit 0 without code changes. Prior WSL report was environment-skewed (Windows-only modules). |
| nextest / deny / audit missing | WSL Cargo lacked subcommands | Present on native Windows: nextest 0.9.140, deny 0.20.2, audit 0.22.2. Gate is Windows-first; WSL tool install remains optional follow-up if WSL is used as a gate host. |
| Ledgerful migration mismatch | `scan --impact` failed with migration number too high | No longer reproduces. `ledgerful scan --impact` exit 0; `ledgerful verify --scope full` exit 0. |

### Observed green results

```text
cargo fmt --check                                          exit 0
cargo clippy --workspace --all-targets -- -D warnings      exit 0
cargo nextest run --workspace                              exit 0  (405 passed, 0 skipped)
cargo deny check                                           exit 0  (advisories/bans/licenses/sources ok)
cargo audit                                                exit 0  (1 allowed warning: RUSTSEC-2026-0190 anyhow)
ledgerful verify --scope full                              exit 0  (fmt, clippy, nextest, deny, audit all SUCCESS)
powershell.exe -File scripts\dev-check.ps1 -CheckOnly      exit 0
git diff --check                                           exit 0
ledgerful scan --impact                                    exit 0  (overall risk MEDIUM; docs-dominated dirty tree)
ledgerful ledger status                                    1 pending tx (T146 Architecture), 0 unaudited drift
```

## 6. Decision

T146 documentation deliverables and the mandatory full repository gate are green on the approved native Windows environment.

- Mark T146 **Complete** after the Ledgerful transaction is committed.
- Stage only intended T146 files for the docs commit; keep gate-hygiene files in a separate chore commit.
- Do **not** stage `.agents/skills/codex-review/SKILL.md`.
- Stop before merge or push to `main` pending explicit approval.

## 7. Recovery reference

Prior machine-local action note:

`C:\Users\RyanB\Documents\Hermes\Today\BLOCKED - AI-Brains T146 full repository verification.md`

Superseded by this review section 5.

## 8. Post-unblock closure checklist

- [x] Fix/approve the supported `scripts/dev-check.ps1` runtime and parsing (PS 5.1 ASCII fix).
- [x] Resolve unrelated Rust formatting failures via separate fmt chore (clippy already green on Windows).
- [x] Provide `cargo-nextest`, `cargo-deny`, and `cargo-audit` in the approved gate environment (native Windows; already installed).
- [x] Confirm Ledgerful impact/verify work (migration mismatch no longer reproduces).
- [x] Re-run all six gates and append exact results here.
- [x] Stage only the intended T146 files (after separate chore commit).
- [x] Commit the Ledgerful transaction (`0115f19b-6637-4d75-aa05-bd0871e3d15f`).
- [x] Mark T146 Complete only after evidence exists.
- [x] Stop before merge or push to `main` pending explicit approval.
