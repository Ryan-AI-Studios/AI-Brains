# T191 Review Log — Hygiene Batch (Ledgerful + Hermetic Long-tail)

- **Track:** T191-HygieneLedgerfulHermetic
- **Branch:** `agent/T191-hygiene-ledgerful-hermetic`
- **Commits:** `dadd75d` (implement), `9e57365` (mixed-tag + cosmetics)
- **Ledger TX:** `c3db2dce-8a6b-4bfe-8a97-af7805c5491e`

## Review rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| Internal R1 completeness | subagent | **PASS WITH DEFERRED P3** | Process P3s only (deferred strike, full gate, cosmetic cg_, plan banner) |
| Internal R1 correctness | subagent | **PASS** | Dual-read, hermetic, capture independence OK |
| Codex R1 | gpt-5.6-luna high | **PASS WITH P3 CLOSEOUT GAPS** | No P0–P2; mixed-tag test + gate/closeout |
| Mixed-tag fix | orchestrator | **fixed_pending_verification** | `symbol_dedup__mixed_legacy_and_new_tags__no_double_ingest` |
| Final Codex | (pending gate green) | — | Fresh re-review after fixes |

## Findings disposition

| ID | Sev | Status | Disposition |
|----|-----|--------|-------------|
| Codex R1 P3-1 full gate | P3 | process | Local fmt+clippy green; nextest requires `AI_BRAINS_ALLOW_ZERO_KEY=1` (T187; CI sets it). Environmental, not T191 regression. |
| Codex R1 P3-2 AC8 deferred | P3 | **fixed_pending_verification** | Struck T142 #1–2 + T186 L13 in `deferred.md` on branch |
| Codex R1 P3-3 mixed-tag test | P3 | **verified_fixed** (unit) | Test added; 5/5 symbol_dedup/ingest tests pass |
| Internal cg_ locals | P3 | **verified_fixed** | Renamed to `lf_*` |
| Plan/spec banners | P3 | fixed | Status → Implemented on branch |

## DoD matrix (engineering)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | No production `ChangeGuardHotspot` / `ChangeGuardVerificationBackend` |
| AC2 | Met | F23 greps clean for forbidden query/ingest/refresh names |
| AC3 | Met | dual-read + write `ledgerful:symbol` + 4 proof tests |
| AC4 | Met | T167 legacy_import preserve green |
| AC5 | Met | 5 long-tail files use `common::hermetic_*` |
| AC6 | Met | 0 bare `Command::cargo_bin` in five files |
| AC7 | Met when gate green | fmt/clippy green; nextest+deny+audit with ALLOW_ZERO_KEY |
| AC8 | Met on branch | deferred strikes |
| AC9 | Met | briefings/intervention/fixtures/schema |
| AC10 | Met | denylist TX_ID keys |
| AC11 | Met | F23 clean |

## Residual out of scope

- doctor CLI (#2)
- archive purge (T142 #4)
- vault-wide offline tag backfill (soft; dual-read sufficient)
- `.changeguard/` discovery fallback (kept)
