# T147 Review Log — Governed Memory Baseline + Edition 2024 + Shadow

## Scope

- **Track:** T147-GovernedMemoryBaseline
- **Branch:** `feature/governed-memory-baseline`
- **Ledger tx:** `46bcef1f-4a50-49f0-b175-b2d76d07da77`
- **Excluded from stage:** `.agents/skills/codex-review/SKILL.md` (unrelated)

## Implementation summary

1. Workspace edition **2024** (Cargo.toml + rustfmt); toolchain pin **1.95.0** (≥1.85 floor documented).
2. `TempEnv` in `ai_brains_core::temp_env`; production env mutation uses `unsafe` + SAFETY at CLI startup / elevate handoff.
3. Path helpers: `resolve_best_effort`, `paths_refer_to_same_location`, `path_is_same_or_inside`.
4. Synthetic fixture replay + golden projections under `fixtures/governed-memory/`.
5. `ai-brains shadow create` with live-vault safety, dry-run, default turn redaction, `shadow-manifest.json`.
6. Tool pins: nextest 0.9.140, deny 0.20.2, audit 0.22.2. deferred.md #6 struck.

## Review rounds

### Internal review 1

- Verdict: **PASS WITH DEFERRED** (Phase 0–3 product complete; mediums F1–F3, F5 open for tests/source-migrate).
- Findings F1–F3, F5 medium; F4/F6/F7 low.

### Fix round 1

- F5: no `migrate()` on source.
- F1/F2/F3: integration tests for live-parent, reparse, happy-path redact+manifest.
- F6: golden regen gated by `UPDATE_GOVERNED_GOLDEN=1`.

### Internal re-review

- Verdict: **PASS**
- F1–F3, F5, F6: `verified_fixed`
- F4, F7: accepted residual (memory_id non-determinism; TempEnv public API)

### Cross-model review (Codex primary / Claude fallback)

- **Codex (primary):** Attempted `codex exec -s read-only -m gpt-5.4` 2026-07-24 — **blocked by account usage limit** until ~2026-07-28. No `review.codex.md` produced.
- **Claude (skill fallback, round 1):** `review.claude.md` — **PASS WITH DEFERRED P3** (AUD-F1 onboarding pins; AUD-F2 dead `fixture_sql_key`).
- **Fix:** Updated onboarding pin table; removed dead `fixture_sql_key`.
- **Claude (round 2):** `review.claude.round2.md` — **PASS** (prior P3s verified_fixed; no P0–P2).

## Gate evidence (2026-07-24)

| Command | Result |
|---------|--------|
| `cargo fmt --check` | GREEN |
| `cargo clippy --workspace --all-targets -- -D warnings` | GREEN |
| `cargo nextest run --workspace` | **426 passed**, 0 skipped |
| `cargo deny check` | GREEN (exit 0) |
| `cargo audit` | GREEN (exit 0; 1 allowed RUSTSEC-2026-0190 anyhow) |
| `ledgerful verify --scope full` | GREEN (all 5 steps SUCCESS) |
| `powershell.exe -NoProfile -File scripts\dev-check.ps1 -CheckOnly` | GREEN — nextest 0.9.140, deny 0.20.2, audit 0.22.2 |

## Manual acceptance

1. `rustc --version` → `1.95.0`; `Cargo.toml` `edition = "2024"`.
2. Fixture tests green via nextest (`governed_fixture_replay__*`).
3. Shadow same-path refuse (manual):  
   `ai-brains shadow create --source X --destination X --dry-run` →  
   `refusing shadow create: source and destination refer to the same location`.
4. Full gate table above.

## Findings disposition

| ID | Severity | Status | Notes |
|----|----------|--------|-------|
| T147-F1 | medium | verified_fixed | live-parent refuse test |
| T147-F2 | medium | verified_fixed | reparse/junction refuse test |
| T147-F3 | medium | verified_fixed | happy-path redact + manifest |
| T147-F5 | medium | verified_fixed | no source migrate |
| T147-F6 | low | verified_fixed | UPDATE_GOVERNED_GOLDEN gate |
| T147-F4 | low | accepted residual | omit non-deterministic memory_id from golden |
| T147-F7 | low | accepted residual | TempEnv public for integration tests |
| T147-AUD-F1 | low | verified_fixed | onboarding SKILL.md tool pins synced |
| T147-AUD-F2 | low | verified_fixed | removed unused `fixture_sql_key` |

## Residual

- Turn-derived `MemoryId::new()` non-determinism (future track if needed).
- Shadow TOCTOU / soft-canonicalize (P6).
- Dry-run still opens source for event count (no migrate).
