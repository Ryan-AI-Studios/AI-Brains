# T270 review log — Retention live `memory_legacy` inventory overlay

**Track:** `conductor/tracks/trackT270-retention-live-classification`
**Category:** FEATURE / UX / HONESTY
**FEATURE TX:** `20ad711c-ff41-4262-9dd0-887a6eb54c03`
**Date:** 2026-08-21

## Scope

Read-only `memory_legacy` COUNT overlay on `retention plan` (and apply prepare/in-process apply reports): `memory_projection` pinned → `held`, other statuses → `skip`. SQL `COUNT` + `ORDER BY memory_id ASC LIMIT 5` samples. Merge **after** `build_report` (not inside `collect_candidates`). `Nothing to dispose.` iff `would_ce_wipe + would_projection_delete == 0`. Work table is dispose-only. JSON keys frozen (`api_version` 1). No `classify_legacy` / `migrate governed` / live apply / `cargo install` / `project.rs`.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC17 / F0–F31 | PASS |
| R1b | Independent explore (read-only) | **PASS** (no P0–P3 product findings) |
| CX1 | Codex `gpt-5.6-luna` high | FAIL (P1 process + P2 apply-test) → P2 **verified_fixed**; P1 orchestrator-owned |
| CX2 | Codex `gpt-5.6-luna` high (after P2 fix) | **PASS** product (process P1 closed by Phase 5–6) |

## Findings

### R1

No product findings. Required red observed on HEAD: CP pinned-only had no `memory_legacy` bucket; pretty inventory-only printed `Work` and omitted `Nothing to dispose.`; clap after_help lacked `inventory`/`none_auto`. Green: overlay COUNT+LIMIT 5, pretty F8/F9, honesty short, Plan after_help.

Noted out of DoD (not findings): nightly `candidates=` one-liner includes held (F20); `active` lumped in other skip; `list_pinned_memory_ids` still loads all ids for R11; PATH until `cargo install` (F16); leftover project 18k pins (T259); doctor retention check (T248 F16).

### R1b

No product findings. Independently traced COUNT + SQL `LIMIT 5` (no unbounded `.take`), merge after `build_report` at all three call sites (`plan_retention` / `prepare_retention_apply` / `apply_retention`), empty-vault early return, pretty dispose-work + Work CE/PD-only, honesty const + short, Plan `after_help`, F31 notes const, class sort, apply loop still iterating `candidates` only (inventory cannot enqueue CE). Envelope R11 test still present. Nightly left unstyled (F20).

### CX1

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| CX1-P1-1 | high (process) | Phase 5 gate / TX commit / publish not done at CX1 | orchestrator-owned | same class as T269/T272 CX1 process-timing |
| CX1-P2-1 | medium | Apply/prepare overlay untested on non-empty inventory | `verified_fixed` | `retention_apply__pinned_inventory__held_in_report_no_delete` — prepare report held/skip split, honesty const, empty CE/turn queues, pins unchanged, no body leak |

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 store COUNT + LIMIT 5 | met | `memory_legacy_inventory__pinned_and_other__counts_and_limit_5` (3 pinned + 2 active + 1 forgotten); pinned=0 other samples; empty zeros; 7-other SQL LIMIT 5 |
| AC2 empty CP | met | `retention_plan__empty_vault__zero_counts` still green |
| AC3 pinned-only held | met | `retention_plan__pinned_memories__memory_legacy_held_inventory` — required red then green; JSON denylist of pin body |
| AC4 forgotten/active skip | met | rstest `#[case]` `retention_plan__non_pinned_status__memory_legacy_skip`; samples non-empty; F31 notes |
| AC5 mixed one bucket | met | `retention_plan__mixed_pinned_and_other__one_bucket_split_totals` — candidates=5, held=3, skip=2, mechanism held |
| AC6 pretty inventory-only | met | `format_retention_pretty__held_inventory_only__nothing_to_dispose_no_work_no_next` — required red (`Work` header) then green |
| AC7 empty pretty | met | `format_retention_pretty__empty__nothing_to_dispose_matrix_skip_no_next` — zero-row `memory_legacy` still **skip** |
| AC8 empty JSON hermetic | met | `retention_plan__format_json__frozen_keys_empty_classes` |
| AC9 hermetic pin + plan | met | `retention_plan__pinned_memory__human_held_inventory_nothing_to_dispose` — human held + non-zero + Nothing to dispose; JSON `memory_legacy` + `would_held >= 1` + `api_version` 1 |
| AC10 plan does not append | met | same hermetic: `memory list --summary` stdout identical before/after plan; JSON `mode` `dry_run` |
| AC11 apply without confirm | met | `retention_apply__without_confirm__invalid_payload_exit_6` still present (`#[test]` restored) |
| AC12 after_help | met | `retention_plan__help__names_inventory_or_none_auto` — required red then green |
| AC13 live | met | `cargo run` human: `Nothing to dispose.`, `memory_legacy` **held** 38313/38342, no `next:`; JSON `would_held=38313`, `api_version=1` |
| AC14 docs | met | CAPABILITIES T248 row; OPERATIONS matrix; PROTOCOL-COMPAT §5; CHANGELOG T270 |
| AC15 existing suite | met | CP `class_based_retention` 35/35; CLI pretty units 9/9; hermetic `retention_plan` 6/6 |
| AC16 SQL LIMIT 5 | met | store helper SQL contains `COUNT(*)` and `LIMIT 5` (not unbounded SELECT + Rust slice) |
| AC17 class sort | met | `retention_plan__raw_turn_and_pinned__classes_sorted_memory_legacy_before_raw_turn` |
| F1 overlay not migrate | met | no `classify_legacy` / `migrate` calls |
| F2 held vs skip | met | pinned→held; other→skip; never soft_forget/ce_wipe from overlay |
| F3 none_auto | met | horizon unchanged; overlay rows have no turn/content_key_id |
| F4 apply gated | met | overlay on prepare report only; loop still no-ops held/skip |
| F5 COUNT+LIMIT 5 | met | store helper; not per-row Candidate |
| F6 merge after build_report | met | `plan_retention` / `prepare_retention_apply` / `apply_retention` |
| F7 JSON keys frozen | met | live JSON; empty vault `classes: []` |
| F8 pretty empty-check | met | dispose-work (`ce_wipe + projection_delete`) |
| F9 Work dispose-only | met | mechanism filter CE / projection_delete |
| F10 honesty const | met | contracts const + CLI short `memory_legacy inventory ≠ auto-forget` |
| F11 samples no bodies | met | AC3 JSON denylist; live samples are ids |
| F12 plan writes nothing | met | AC10 |
| F13 capture independence | met | SQL + pretty only |
| F14 pins | met | clap lock 4.6.1; rusqlite 0.39.0; rstest 0.25 already in lock (CP dev-dep only) |
| F15 no new DTO fields | met | additive warning const + optional class bucket |
| F16 no cargo install | met | tests/manual used `cargo run` / hermetic |
| F17 no live apply | met | not run |
| F23 cross-model | CX2 product **PASS** | P2-1 closed; no new P0–P2 |
| F29 no project.rs | met | diff does not touch `project.rs` / `preflight.rs` / `sync.rs` / `nightly.rs` |
| F30 classes sort | met | AC17 |
| F31 notes const | met | `NOTE_MEMORY_LEGACY_INVENTORY` |
| No production unwrap/expect/panic | met | clippy `-D warnings` |

## Full gate (observed)

- `.\scripts\dev-check.ps1` EXIT 0 — nextest **3233 passed** (1 skipped)
- `ledgerful verify --scope full` EXIT 0 — fmt 2.5s / clippy 17.4s / nextest 143.8s / deny 2.6s / audit 3.0s

## Targeted gates (observed)

- Red: CP 5/5 new tests FAIL (no `memory_legacy` bucket); pretty AC6 FAIL (`Work` + no `Nothing to dispose.`); clap AC12 FAIL
- Green: store inventory 4/4; CP `class_based_retention` 35/35; CLI bins pretty+help 9/9; hermetic `retention_plan` 6/6
- `cargo fmt` applied
- `cargo clippy -p ai-brains-store -p ai-brains-control-plane -p ai-brains-cli -p ai-brains-contracts --all-targets -- -D warnings` PASS (assemble helper dropped to avoid `too_many_arguments`; merge inlined at 3 call sites)

## Manual evidence (AC13)

| Command | Result |
|---------|--------|
| `cargo run -p ai-brains-cli --quiet -- retention plan --format human` | EXIT 0. `Nothing to dispose.` `memory_legacy none_auto held 38342`. Totals `candidates=38342 … skip=29 held=38313`. Honesty includes `memory_legacy inventory ≠ auto-forget`. No `Work`. No `next:`. |
| `cargo run -p ai-brains-cli --quiet -- retention plan --format json` | `api_version=1`, `mode=dry_run`, one `memory_legacy` class, `totals.would_held=38313`, `would_ce_wipe=0`, `would_projection_delete=0`, 5 sample ids, honesty const in `warnings`. No new keys. |
| Live apply | **not run** (F17) |
