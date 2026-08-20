# T272 review log — Preflight `--global` Safety skip vs Index

**Track:** `conductor/tracks/trackT272-preflight-global-safety-index-skip`
**Category:** BUGFIX / UX
**BUGFIX TX:** `56cb6203-46da-473c-8ab5-c73f0a9df8c3`
**Date:** 2026-08-20

## Scope

Under `--global`, Index/Recent skip `safety_ids` is rebuilt from **emitted** Safety rows (post HOTSPOT-suppress + `dedup_hotspots_keyed` + `take_round_robin`), not the pre-cap LIMIT 40 fetch. `memory_id` is carried as extra `T = (Option<String>, String)`. Fetch-loop `insert` is gone. T264 caps / LIKE / tags / span formula / T180 keys / T265 `sections[]` stay frozen. No CLI hotspot growth, no `project.rs`, no clap 5, no `cargo install`.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC11 / F0–F28 | PASS |
| R1b | Independent explore (read-only) | **PASS** (no P0–P3 product findings) |
| CX1 | Codex `gpt-5.6-luna` high | **PASS** product (process P1 = gate/publish not yet; orchestrator-owned) |

## Findings

### R1 / R1b

No product findings. R1b independently traced HOTSPOT-if-cg `continue` before `push`, extra `(project_id, memory_id)`, RR key `|(_, (pid, _))|`, F28 rebuild comment, Index `contains` still on the post-pipeline HashSet, `safety_for_skip` still post-cap, `GLOBAL_*` constants untouched, CLI `preflight.rs` / `preflight_json.rs` / `project.rs` not in the diff.

Noted out of DoD (not findings): live `-m` default may hide Memory Index behind a large Safety pin (F26 hermetic `-m 1500` is the proof; AC10 pass-with-observed-data); session `HOTSPOT:` content skip (F18); Index fetch 80 leftover-heavy (F17); PATH `cargo install` (F14).

### CX1

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| CX1-P1-gate | high (process) | Full workspace gate / TX / publish not done at CX1 time | `verified_fixed` (local gate) / remaining Phase 6 | `dev-check` 3220 + `ledgerful verify --scope full` |

Product implementation: PASS. No P0–P3 product findings. Process P1 is orchestrator-owned (same class as T269 CX1 process-timing). No product re-review required.

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 extras-through-dedup skip set | met | `dedup_hotspots_keyed__duplicate_path__skip_set_omits_dropped_id` — remaining extras keep `keep-id`, omit `drop-id` |
| AC2 capped-out A-one in Index | met | `preflight_global_isolation__capped_out_safety__appears_in_index` (`-m 1500`, `Memory Index`, A-one absent Safety / present Index; A-two/A-three/B-only in Safety). **Required red** then green |
| AC3 project-scoped shown skip | met | `preflight_global_isolation__project_scoped__shown_safety_not_in_index` — guard; already green on LIMIT 10 (F27) |
| AC4 T264 AC10 still green | met | `preflight_global_isolation__three_a_one_b__b_appears_a_capped` (`a_count <= 2`, B in Safety) |
| AC5 T264 labels + continuation | met | `preflight_global_isolation__two_projects__pretty_labels_and_no_unlabeled_safety` |
| AC6 T265 compact JSON 2-key | met | `preflight_global_isolation__compact_json__two_keys_and_hex_tags` |
| AC7 summary span line | met | `preflight_global_isolation__summary_span_and_json_key` (`In context spans`, `N >= 2` not frozen) |
| AC8 docs | met | CAPABILITIES T264 skip-emitted clause; CHANGELOG Unreleased T272 row |
| AC9 no DTO / pins / CLI / constants | met | contracts untouched; clap lock 4.6.1; `GLOBAL_*` 2/8/40, 3/15/80, 1/3, 1/40; CLI preflight/project.rs not edited |
| AC10 manual | met | see Manual evidence |
| AC11 session CONSTRAINT skip post-cap | met | `safety_for_skip` filled from emitted bodies; session loop uses that vec, not pre-cap ids |
| F1 skip emitted ids only | met | rebuild after pipeline; `safety_ids.insert` absent in repo |
| F2 project-scoped post-dedup | met | RR gated `if global`; LIMIT 10 SQL unchanged |
| F3 carry memory_id | met | extra `(Option<String>, String)`; `dedup_hotspots` still `T=()` |
| F4 T264 freeze | met | LIKE / LIMIT 40 / tags / span formula / leftover-recall-drop untouched |
| F5 `safety_for_skip` post-cap | met | filled after rebuild from remaining entries |
| F6 HOTSPOT suppress before push | met | `continue` then `safety_raw.push` |
| F7 T265 / T180 | met | CLI splitters not edited |
| F8 no new flags | met | no `--include-capped` |
| F9 module / hotspots | met | retrieval `preflight.rs` + isolation tests + docs; no helper file |
| F10 / F11 pins / contracts | met | no lock bumps; no DTO |
| F12 capture independence | met | SQL + HashSet only |
| F13 tests / no unwrap | met | naming convention; production change is collect/map |
| F14 no cargo install | met | tests/manual used `cargo run` / hermetic |
| F26 AC2 `-m 1500` | met | explicit args + `Memory Index` assert |
| F27 AC3 guard | met | already green; kept |
| F28 rebuild comment | met | exact line above HashSet collect |
| No production unwrap/expect/panic | met | clippy `-D warnings` |

## Targeted gates (observed)

- Red: AC2 panicked `AC2 Memory Index header present under -m 1500` — Safety = A-three / B-only / A-two; Index absent because all 4 fetch ids were in pre-cap `safety_ids`
- AC3 already PASS (guard)
- AC1 PASS against remaining extras
- Green: same AC2 PASS; isolation **8/8** PASS; retrieval dedup units **3/3** PASS
- `cargo fmt --check` PASS
- `cargo clippy -p ai-brains-retrieval --all-targets -- -D warnings` PASS
- `ledgerful verify --scope fast` PASS (fmt 2.0s / clippy 22.5s / workspace nextest 237.1s / deny 17.8s / audit 4.0s)

## Manual evidence (AC10)

```powershell
cargo run -q -p ai-brains-cli -- preflight --global --pretty --no-hook-prompt
# EXIT=0
# Safety first line tagged `[C:\dev\ai-brains]`
# Memory Index header absent in this live window (Safety pin ate the budget)
# pass-with-observed-data (same class as T264 AC14); hermetic AC2 is the skip proof
```

Did **not** pin, `cargo install`, rewrite `.env`, or mutate schtasks.

## Full gate (observed)

- `.\scripts\dev-check.ps1` **SUCCESS** — nextest **3220 passed** (1 slow), **1 skipped**; deny 0.20.2; audit 0.22.2 (19 allowed warnings)
- `ledgerful verify --scope full` **PASS** (fmt 2.4s / clippy 1.9s / nextest 102.9s / deny 2.5s / audit 2.6s)

## Residual (append to deferred.md at closeout)

- Session `HOTSPOT:` content skip (F18)
- T264 Index fetch 80 leftover-heavy (F17)
- Live `-m` windows without an Index header (word budget; hermetic is DoD)
- PATH `cargo install` (F14)
- T270 / T273 F7 peers
