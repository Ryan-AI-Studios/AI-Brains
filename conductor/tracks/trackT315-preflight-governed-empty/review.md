# T315 Review Log — Preflight empty-decisions next-step

**Track:** T315-PreflightGovernedEmpty  
**Category:** UX / HONESTY (FEATURE)  
**Branch:** `track/T315-preflight-governed-empty`  
**FEATURE TX:** `a38a0cba-b820-4f36-8924-c13bff46b50a`

## Scope

- `crates/ai-brains-cli/src/commands/preflight.rs` — F7 label, F2/F8 helpers, F5 JSON fill, units
- `crates/ai-brains-cli/tests/preflight_summary_json.rs` — AC5/AC6 scope-none; AC7 not-T315-SOOT
- `Docs/CAPABILITIES.md`, `Docs/PROTOCOL-COMPAT.md`, `CHANGELOG.md` — AC10

## Commits

| SHA | Role |
|-----|------|
| `0ea95c5` | Red — failing stubs + AC assertions |
| `0a38e1e` | Green — production + docs |
| `6648974` | P3 — DTO comment + legacy insert unit |

## DoD matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | verified_fixed | `format_summary_empty_decisions_next__zero__exact_soot` |
| AC2 | verified_fixed | `Budget window words: 100`; no `Total Word Count` |
| AC3 | verified_fixed | arity-9 unit |
| AC4 | verified_fixed | insert after budget + legacy `Total Word Count:` unit |
| AC5 | verified_fixed | scope-none human hermetic |
| AC6 | verified_fixed | scope-none JSON SOOT |
| AC7 | verified_fixed | tagged + legacy not T315 SOOT |
| AC8 | verified_fixed | bootstrap wins unit |
| AC9 | verified_fixed | T180 non-summary path untouched (full gate) |
| AC10 | verified_fixed | CAPABILITIES + PROTOCOL-COMPAT + CHANGELOG |
| AC11 | verified_fixed | `cargo run -- preflight --summary` → SOOT + Budget window words + decisions 0 |
| AC12 | verified_fixed | retrieval `preflight.rs` diff empty |
| AC13 | verified_fixed | SOOT ≤140 in AC1 |
| AC14 | verified_fixed | hotspots=5 still inserts |
| AC15 | verified_fixed | no new required JSON keys |

## Findings

### R1 — Internal DoD (PASS, no >low open)

| id | severity | description | status |
|----|----------|-------------|--------|
| R1-01 | low-info | SOOT + F7 + F8 wired | verified_fixed |
| R1-02 | low-info | T241 JSON precedence | verified_fixed |
| R1-03 | low-info | F38 scope-none + AC7 | verified_fixed |
| R1-04 | low-info | Docs AC10 | verified_fixed |
| R1-05 | low-info | Isolation (retrieval / list overlay / arity) | verified_fixed |
| R1-06 | low-info | Optional after_help skipped | out_of_scope |

### R2 — Codex cross-model

| id | severity | description | status |
|----|----------|-------------|--------|
| P3-001 | low | `next_step` DTO comment stale (complete/global omit) | verified_fixed (`6648974`) |
| P3-002 | low | AC4 missing legacy `Total Word Count:` lock | verified_fixed (`6648974`) |

Fresh re-review: see `review.codex.md` (overwrite after P3).

## Manual evidence (AC11)

```text
cargo run -q -p ai-brains-cli -- preflight --summary
… In context decisions: 0
Budget window words: 708
next: ai-brains recall "what did we decide"
…

cargo run -q -p ai-brains-cli -- preflight --summary --format json
… "word_count": 708, "next_step": "next: ai-brains recall \"what did we decide\""
```

PATH install not required (F18).

## Residuals → deferred.md (non-easy / not this DoD)

| Residual | Note |
|----------|------|
| T286 Index `## Objective` (R1-1) | Decline steal F11 — needs Index SQL track |
| PATH until owner `cargo install` | F18 |
| T325 F8 PreferRecency | Minted; not stolen |
| In-context still 0 after T315 | By design — next-step is the product |
| T220 soft / `is-terminal` | Prior soft residuals |

## Gates

| Gate | Result |
|------|--------|
| Targeted nextest ACs | PASS |
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo nextest run --workspace` | PASS (3575 passed, 1 skipped) — prior load flake on `cross_repo_bridge_smoke` timed out under contention; alone PASS ~92s; quiet full suite PASS |
| `cargo deny check` | PASS (warnings only) |
| `cargo audit` | PASS (20 allowed warnings) |
| Codex | PASS (`review.codex.md`; P3-001/P3-002 verified_fixed) |
| `ledgerful verify --scope full` | **PASS** (fmt/clippy/nextest/deny/audit) |
