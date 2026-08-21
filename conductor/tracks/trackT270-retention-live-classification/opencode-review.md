# Track T270 Review — Retention Live Classification

**Reviewer:** opencode (cross-model, read-only harness)
**Date:** 2026-08-20
**Scope:** Plan audit only — no implementation, no folding, no edits to spec/plan/conductor/deferred/product code.
**Track:** T270 — `memory_legacy` COUNT overlay for `retention plan` (Category FEATURE/UX/HONESTY; owner Grok; Status **Planned**)
**Spec:** F0–F29 / AC1–AC16

---

## Summary

T270 is a **read-only** enhancement to `ai-brains retention plan`: it adds a live pinned-memory
COUNT overlay (`memory_legacy` inventory) so operators can see how many memories are being held
under the retention policy. Behavior spec: pinned → `held`; other → `skip`; when nothing is
disposable the plan prints `Nothing to dispose.` and performs **no CE/projection work** (dry-run
only). The plan explicitly scopes `retention apply` mutation, CE wipe, and `classify_legacy` out
of the DoD.

**Verdict basis:** every line-count, symbol, and behavior claim in `plan.md` was re-verified against
live `src/` at HEAD `70d61cd` (the plan's own docs commit, parent `fdd4924`). Zero blocker/medium
findings. Findings below are minor (`m`) and opportunity (`O`) only. **Verdict: Planned.**

---

## Findings

### Blocking (B)
None.

### Medium (M)
None. All plan preflight claims verified accurate against live code:

| Claim | Live verification |
| :--- | :--- |
| `run_plan` read-only at `commands/retention.rs:64` | Confirmed — query-only path, no writer |
| Empty-check `classes.is_empty() \|\| totals.candidates == 0` at `:417` | Confirmed (`format_retention_pretty` `:403`) |
| `Nothing to dispose.` empty state present | Confirmed via live `retention plan --format human` |
| `zero_row_mechanism` (`memory_legacy`→`skip`) at `:314` | Confirmed |
| `run_apply` uses `prepare_retention_apply` at `:146` | Confirmed (`:145-146`) |
| `plan_retention` `:234` / `collect_candidates` `:269` / retain filter `:414` | Confirmed (CP `class_based_retention.rs`, 1204 lines) |
| `build_report` `:604`; `would_skip` via `_ =>` `:648-650` | Confirmed |
| **SOOT for merge**: `prepare_retention_apply` `:755` → `collect_candidates` `:771` | Confirmed — correct single point of truth |
| `classify_envelope` `:427`; HELD at `:498` | Confirmed |
| Store `list_pinned_memory_ids` `:242` (`WHERE status = 'pinned'`); `memory_status` `:255` | Confirmed — **no memory COUNT helper exists** (new store helper required, correctly planned) |
| **Stream-A `memory_legacy` scan absent** | Confirmed — `collect_candidates` scans envelopes, raw turns, query traces, closed reviews, disposable decisions only |
| Contracts `API_VERSION "1"` `:10`, `CLASS_MEMORY_LEGACY` `:22`, horizon `none_auto` `:212`, honesty consts `:59-75` | Confirmed — **no `RETENTION_HONESTY_MEMORY_LEGACY_INVENTORY` const yet** (new const needed, correctly planned) |
| `truncate_sample_ids` `:232` (`.take(5)`); `truncate_id` `:240` (MAX 36) | Confirmed |
| `RetentionCommands::Plan` `main.rs:2288`; after_help `:2281/:2286` (no inventory/none_auto needle — red, correct) | Confirmed (`main.rs` 4819 lines; `--format` value_parser excludes `xml`) |
| Nightly dry-run `plan_retention` call | File is `crates/ai-brains-cli/src/commands/nightly.rs` (NOT daemon — plan correct); calls at `:511-535` (see m2) |
| Tests: `insert_memory` `:126` (cols `memory_id, content, privacy, status, level, created_at, updated_at`); empty-vault zero-count `:163`; pinned-held `:275` | Confirmed (`retention_plan__pinned_memory__held` asserts `would_held >= 1` + held mechanism + no secret leak in JSON) |
| CLI unit `format_retention_pretty__empty__nothing_to_dispose_matrix_skip_no_next` `:676` | Confirmed — asserts exact `Totals candidates=0 ce_wipe=0 projection_delete=0 skip=0 held=0`, `projection delete ≠ CE`, NIST short, 90d, all canonical classes |
| Hermetic `tests/retention_plan_human.rs`; frozen-keys JSON test `:49` | File exists (Test-Path True); test asserts empty `classes` |
| Dependabot open PRs (chrono 0.4.45 #62, rusqlite 0.40.2 #61, thiserror, tokio, tower-http) | Confirmed via `gh pr list` — **no bumps per plan, correct** |

### Minor (m)
1. **HEAD staleness (m-note, same class as T272 review).** Plan preflight claims source HEAD
   `fdd4924` (T272 #187), CLEAN, `main == origin/main`. Live: HEAD = `70d61cd` (the T270 plan's own
   docs commit `docs(conductor): plan T270 memory_legacy inventory overlay`), parent `fdd4924`;
   `origin/main` = `fdd4924` (docs commit unpushed). Plan predates its own docs commit — cosmetic,
   no content drift.
2. **Nightly.rs line drift.** Plan cites `nightly.rs:509-531`; actual calls at `:511-535`
   (`plan_retention` import `:511`, call `:514`, totals eprintln `:525`, failure eprintln `:534-535`).
   Symbols match; line drift only.

### Opportunity (O)
1. **Pinned count volatile.** Live `memory list --summary --global` = **Pinned 38,210 / Forgotten 29**
   vs plan's 38,208. Plan labels the figure volatile — confirmed correct handling.
2. **Hotspot score drift.** `project.rs` hot score 3.999 live vs plan's 4.008 (still #1; `sync.rs`
   #2 3.693; `preflight.rs` #7 2.307). Retention files not in top-10 — correct, plan does not grow
   them.
3. **Ledgerful search line drift.** Plan cited `collect_candidates` hits `:234/:269/:771/:990`;
   live hits `:239/:240/:269/:990`. Symbols match; call-site line drift only.
4. **`ISSUES.md` absence.** `conductor/ISSUES.md` does NOT exist (Test-Path False). Missing scan —
   would be M for the fold phase; the plan's deferred.md absorption (`:235`, `:615`) covers the known
   deferrals, so no live gap for this track.

---

## What looks solid

- **Honest, minimal scope.** Read-only overlay; `retention apply` mutation and CE wipe explicitly
  out of DoD; `classify_legacy`/`migrate governed` never called. Respects Capture Privacy
  (pinned-count only, no body/index leakage — R11 asserts no secret in JSON).
- **Correct single point of truth.** Overlay merges into `prepare_retention_apply`→`collect_candidates`
  (`:755`/`:771`), the exact spot where plan-side candidate enumeration converges — additive, no
  double counting, no second scan path.
- **Contract discipline.** `none_auto` horizon (`:212`) + `truncate_sample_ids`/`truncate_id`
  (`:232`/`:240`) already constrain the human output; new honesty const only. No clap 5 / pin bumps.
- **Determinism + fixtures.** Frozen-keys JSON test and exact-value human assertions; volatile
  `generated_at` handled; no bare env vars in tests.
- **Empty state already implemented** (`Nothing to dispose.` at `:417`, T248) — T270 lifts the
  empty-check condition without touching the pretty printer's contract.
- **Dependency research correct.** Cargo.lock pins clap **4.6.1**, serde_json **1.0.150**,
  chrono **0.4.44**, rusqlite **0.39.0**; crates.io current clap **4.6.6** (no clap 5), rusqlite
  **0.40.2** — plan's "no bumps" stance is right and Dependabot PRs are open for the interested
  crates.
- **ISO 27001 A.8.10 citation verified** (deletion when no longer required; recording deletion as
  evidence) — plan's governance hook is real.

---

## Deferred fold-in table

| Source | Claim | T270 disposition |
| :--- | :--- | :--- |
| `deferred.md:235` | "`retention plan` 0 candidates on 35,300 memories" | **Absorbed** — F1 placeholder → F0–F29 (inventory overlay) |
| `deferred.md:615` | T270 planning absorption block (placeholders F1–F4) | **Absorbed** — spec F0–F29 covers |
| T166 §5.1.5 | Stream-A `memory_legacy` scan never coded | **Absorbed** — T270 read-only overlay is the coded replacement |
| T248 | Empty-check `classes.is_empty() \|\| totals.candidates == 0` | **Absorbed** — T270 lifts to `totals.candidates == 0` |
| — (out of scope, DoD) | `retention apply` mutation, CE wipe, `classify_legacy`, `migrate governed`, auto-forget pins, T166 horizon retune, T248 format tokens / apply JSON default, doctor 16th, clap 5 / pin bumps, contracts new keys | **Not absorbed** — correctly excluded |
| Live | Shell leftover `7d97a456` vs effective `3581317d-601e-44f7-ab84-fde90aa12d3c` | **Not absorbed** — `project whoami` shows leftover overridden, mismatch false; outside T270 scope |

---

## Last-PR Cursor comments

PR **#187** (T272) — **MERGED** 2026-08-20T23:09:31Z. `gh pr view 187 --json comments,reviews`
returned `comments: []`, `reviews: []` → **Cursor comments N/A**. No T274 PR exists (open PRs are
Dependabot-only: actions 68–72, cargo 58–62). Nothing to fold from last merged PR.

---

## Research/tools notes

- **Pins (Cargo.lock):** clap 4.6.1, serde_json 1.0.150, chrono 0.4.44, rusqlite 0.39.0 — exact
  match with plan. crates.io current (via `ConvertFrom-Json` extraction): clap 4.6.6 (no clap 5
  line), rusqlite 0.40.2 → no bumps per plan, correct.
- **Ledgerful:** `ledger status --compact` = 0 pending / 0 unaudited drift; `doctor` = 10 hygiene
  findings collapsed + timings-0 warn (22,888 rows >10k threshold); `hotspots` = `project.rs` #1
  (3.999), `sync.rs` #2 (3.693), `preflight.rs` #7 (2.307); `search collect_candidates` →
  `:239/:240/:269/:990`. `ISSUES.md` absent (Test-Path False).
- **Live dogfood (read-only):** `ai-brains retention plan --format human` → `Nothing to dispose.`
  + matrix `memory_legacy none_auto skip 0` + `Totals candidates=0 ce_wipe=0 projection_delete=0
  skip=0 held=0`; `--format json` → `api_version "1"`, `classes=[]`, `candidates=0`, 4 warnings;
  `memory list --summary --global` → Pinned 38,210 / Forgotten 29.
- **AI-Brains self-usage:** `preflight --summary` → project `3581317d-…`, Pinned 3,255, 3 active
  sessions, 0 hotspots/decisions/constraints in context; `recall "retention plan inventory
  memory_legacy overlay"` → returns T248 review memory + no contradictory prior decision.
- **Isolation honored:** no `retention apply --confirm`, no `classify_legacy`, no `migrate
  governed`, no `cargo install`, no `.env` rewrite, no schtasks mutate, no `AI_BRAINS_KEY`
  exposure. No edits to `.ledgerful/` state files.

---

## Verdict

**Planned.** No blocker or medium findings; minor (`m`) notes are cosmetic (plan-predates-own-docs-
commit HEAD, nightly.rs line drift). All load-bearing claims (SOOT `:755`/`:771`, no COUNT helper,
absent stream-A scan, empty-check `:417`, frozen JSON keys, dependency pins, absent honesty const)
verify exactly. Ready to proceed to `/fold-in T270`.
