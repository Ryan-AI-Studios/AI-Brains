# T279 review log — Preflight Safety vs live hotspots

**Track:** T279-PreflightSafetyVsHotspots
**Status:** Completed (full gate green; Phase 6 pending this commit)
**FEATURE TX:** `a9c3fdbd-a516-4b8e-b1a9-f3534f057ab4`
**HEAD (implement):** `track/T279-preflight-safety-vs-hotspots`

## Reviewers / rounds

| Round | Reviewer | Result |
|-------|----------|--------|
| R1 | Implementer (Grok) vs spec AC1–AC14 / DoD | **PASS** — red then green; GLOB + live inject + honest empty; JSON keys frozen; no live pin / no `safety sync` without `--dry-run` |
| R1b | Explore subagent (read-only DoD) | **PASS WITH DEFERRED P3** — P3-1 `.agents` skill (untracked); tracked `.claude` already had F26 one-liner. Retrieval fixture skip-env added after workspace nextest |
| CX1 | Codex gpt-5.6-luna | **FAIL** — P0-1 privacy SQL (out of scope / pre-existing); P1-1 F7 substring (fixed); P1-2 process gate; P2-1 cap 5 (fixed); P3 comment |
| R2 | Implementer | P1-1/P2-1 `fixed_pending_verification`; P3-1 `verified_fixed`; P0-1 declined pre-existing; P1-2 residual closeout |
| CX2 | Codex gpt-5.6-luna | **PASS WITH DEFERRED P3** — P1-1/P2-1 verified; P0-1 pre-existing affirmed; P3 agy-review.md trailing whitespace deferred |
| Gate | `dev-check.ps1` + `ledgerful verify --scope full` | **PASS** nextest **3294** / 1 skipped |

## Finding fields

id, severity, description, source, files, required_fix, status, evidence.

## Findings

| id | severity | description | source | files | required_fix | status | evidence |
|----|----------|-------------|--------|-------|--------------|--------|----------|
| P3-1 | low-info | `.agents/skills/ai-brains/SKILL.md` preflight comment still said vault HOTSPOT/DECISION/CONSTRAINT soup | R1b | `.agents/skills/ai-brains/SKILL.md` (gitignored) | One-liner: live hotspots + leading GLOB + dry-run empty | `verified_fixed` locally | Tracked `.claude/skills/ai-brains/SKILL.md` Orient row already named live hotspots + `safety sync --dry-run`. Untracked `.agents` copy updated. |
| P0-1 | critical (claimed) | Safety SQL has no `is_injectable_privacy` | CX1 | `preflight.rs` Safety SELECT | Filter privacy | `out_of_scope` | Pre-existing: LIKE-anywhere SQL also selected `status='pinned'` only. Index already filters. Not a T279 regression; spec F-list does not retune privacy. |
| P1-1 | high | Live inject used substring `HOTSPOT:` and could drop CONSTRAINT bearings | CX1 | `preflight_safety.rs` `suppress_vault_hotspot_row` | Leading `PinKind::Hotspot` for live arm | `verified_fixed` (CX2) | Unit `suppress_vault_hotspot_row__live_inject__leading_only` PASS. Intelligence substring keep (F7). |
| P1-2 | high (process) | Full gate / Phase 6 unfinished at CX1 | CX1 | conductor + review.md | Complete Phase 5–6 | `verified_fixed` (gate) | `dev-check` 3294 passed / 1 skipped; `ledgerful verify --scope full` exit 0. Phase 6 remaining |
| P2-1 | medium | Parser did not cap at 5 | CX1 | `parse_hotspots_json` | Cap `LIVE_HOTSPOT_LIMIT` | `verified_fixed` (CX2) | `parse_hotspots_json__more_than_five__caps` PASS |
| P3-3 | low-info | `agy-review.md` lines 3–6 trailing whitespace | CX2 | `agy-review.md` | Hygiene only | `deferred` | Plan-review artifact; do not edit `*-review.md` |
| P3-2 | low-info | Stale “expected to fail” comment on session-turns test | CX1 | `preflight_includes_session_turns.rs` | Correct comment | `verified_fixed` | Comment updated; assertions already active |

R1: no product findings. Retrieval integration tests now set F13 skip-env (in-process `build_preflight`); privacy tests assert payloads stay out rather than empty text (F3 always-emit).

## DoD matrix (AC1–AC14)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | `safety_marker_glob_sql__includes_constraint_not_decision` — `CONSTRAINT:*` / `INVARIANT:*` / `HOTSPOT:*` / `ASSISTANT: CONSTRAINT:*`; no `DECISION:`; GLOB not LIKE |
| AC2 | Met | `format_safety_hotspot_line__path_and_score__hotspot_prefix` — `HOTSPOT: crates/foo.rs score=0.05`; empty/whitespace path `None`; `{:.2}` |
| AC3 | Met | `preflight__buried_constraint_dump__not_in_safety` — leading needle in Safety; `## Objective` absent |
| AC4 | Met | `preflight__no_bearings__emits_safety_sync_remediator` — header + `safety sync --dry-run` |
| AC5 | Met | T272 `preflight_global_isolation__capped_out_safety__appears_in_index` PASS |
| AC6 | Met | T219 `preflight_pretty__summary_smoke__dual_model_unchanged` + `preflight_pretty__summary_compact__dual_model_unchanged` — no Bearings header |
| AC7 | Met | `preflight__compact_json__required_keys_frozen` — `text` + `word_count`; no `hotspots[]` |
| AC8 | Met | `skip_live_hotspots_env__truthy__no_spawn` — TempEnv `1` does not call spawn |
| AC9 | Met | `parse_hotspots_json__log_then_array__one_path` — log mid-line `[` skipped; array line parsed; missing `[` empty; `displayScore` ignored |
| AC10 | Met | `cargo run -p ai-brains-cli -- preflight --pretty --compact -m 400` → `HOTSPOT: crates/ai-brains-cli/src/commands/project.rs score=0.05` + `sync.rs` + `forget.rs`. **Did not pin. Did not `safety sync` without `--dry-run`.** |
| AC11 | Met | `preflight__global_pretty__no_cwd_project_rs_inject` — `--global` Safety has no `project.rs` |
| AC12 | Met | `preflight__summary_after_bearing__in_context_constraints_ge_1`; empty remediator `in_context_hotspots` stays 0 |
| AC13 | Met | Diff omits `project.rs` / CLI `preflight.rs` / `doctor.rs` / `sync.rs` / `safety.rs`. `main.rs` after_help only |
| AC14 | Met | `safety_empty_const__no_hotspot_marker` — no `HOTSPOT:`; contains `safety sync --dry-run` |

## Targeted gates (R1)

```text
cargo nextest run -p ai-brains-retrieval --lib
  129 passed

cargo nextest run -p ai-brains-cli --test preflight_global_isolation --test preflight_pretty_readability --test preflight_json_envelope --test preflight_safety_vs_hotspots
  23 passed

cargo clippy -p ai-brains-retrieval --all-targets -- -D warnings
  exit 0

cargo clippy -p ai-brains-cli --all-targets -- -D warnings
  exit 0
```

## Manual (classify-only)

```text
cargo run -p ai-brains-cli -- preflight --pretty --compact -m 400
  --- Repository Bearings & Safety ---
  HOTSPOT: crates/ai-brains-cli/src/commands/project.rs score=0.05
  HOTSPOT: crates/ai-brains-cli/src/commands/sync.rs score=0.04
  HOTSPOT: crates/ai-brains-cli/src/commands/forget.rs score=0.02
  +2 more safety entries — ai-brains memory list
  (T250 --compact caps Safety at 3; F16 freeze)
```

**Did not** `safety sync` without `--dry-run`. **Did not** live pin. **Did not** `cargo install`.
