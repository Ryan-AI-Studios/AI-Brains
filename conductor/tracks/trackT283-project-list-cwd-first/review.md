# T283 review log — `project list` cwd-first (human only)

**Track:** T283-ProjectListCwdFirst
**Status:** Completed (full gate green; Phase 6 pending this commit)
**FEATURE TX:** `b0f00cbc-63a7-4cce-be0c-3f4edbd1c7b5`
**HEAD (implement):** `track/T283-project-list-cwd-first`

## Reviewers / rounds

| Round | Reviewer | Result |
|-------|----------|--------|
| R1 | Implementer (Grok) vs spec AC1–AC14 / DoD | **PASS** — red then green; human permute cwd-first; JSON + T267 footer original; hermetic AC3–AC6; classify-only AC10 |
| R1b | Explore subagent (read-only DoD) | **PASS** — AC1–AC14 Met; no P0–P3 product findings |
| CX1 | Codex gpt-5.6-luna | **Product PASS** — no P0–P3; P1 process (full gate / closeout / Phase 6) open as expected |
| Gate | `scripts/dev-check.ps1` + `ledgerful verify --scope full` | **PASS** nextest **3341** / 1 skipped |

## Finding fields

id, severity, description, source, files, required_fix, status, evidence.

## Findings

| id | severity | description | source | files | required_fix | status | evidence |
|----|----------|-------------|--------|-------|--------------|--------|----------|
| — | — | R1: no product findings | R1 | — | — | — | AC1 last/middle/already-first + len/once; AC2 rstest None/empty/missing; hermetic nth(1) cwd; JSON [0] leftover; star leftover env; F8 no-owner; footer original |
| — | — | R1b: no product findings | R1b | — | — | — | AC1–AC14 Met; forbidden files unhooked; F26 `?`; F35 after_help exact |
| P1-1 | high (process) | Full gate / ledger / conductor closure unfinished at CX1 | CX1 | conductor + review.md | Run gate + closeout | `verified_fixed` | `dev-check` nextest **3341** (1 skipped); `ledgerful verify --scope full` exit 0; closeout this commit; Phase 6 remaining |

## DoD matrix (AC1–AC14)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | `promote_cwd_owner__middle_id__becomes_first` — last `[c,a,b]`; middle `[b,a,c]`; already-first `[a,b,c]`; `len == 3`; promoted id `count == 1`. F37 `with_capacity`. |
| AC2 | Met | rstest `None` / `Some("")` / `Some("missing")` clone including empty vec |
| AC3 | Met | `project_list__human__cwd_owner_smaller_count__first_data_row` — `isolate_empty_home`; leftover 2 pins; cwd `register-path`; `lines().nth(1)` cwd id not leftover |
| AC4 | Met | `project_list__json__still_memory_desc` — `projects[0]` leftover; cwd later; keys `api_version` / `projects` / `unaliased_count` |
| AC5 | Met | `project_list__human__star_on_leftover_env__cwd_still_first` — `.env(AI_BRAINS_PROJECT_ID, leftover)` after `hermetic_bin`; nth(1) still cwd |
| AC6 | Met | `project_list__human__no_path_owner__memory_desc` — no cwd `register-path`; first data row leftover |
| AC7 | Met | `next_action_honesty` footer tests 3/3 project_list footer cases passed (original vec) |
| AC8 | Met | `project_list__format_json__shape_and_unaliased_count` passed |
| AC9 | Met | `project_list__empty_vault__t198_no_footer` passed |
| AC10 | Met | `cargo run -q -p ai-brains-cli -- project list`: nth(1) `3581317d-601e-44f7-ab84-fde90aa12d3c` (3647). JSON `projects[0]` leftover `7d97a456-…` 18043 = max. Footer `set-alias 33ec90e0-… my-project` (not leftover + `AI-Brains`). Exit 0. `.env` SHA256 unchanged. Did not `cargo install`. |
| AC11 | Met | CAPABILITIES List + List JSON cwd-first / size-desc; OPERATIONS Listing Projects T212 columns + cwd-first; CHANGELOG T283; `.claude` `:89` one sentence; `.agents` skill no `project list` (unchanged). PROTOCOL-COMPAT no new keys. CLI-EXIT-CODES untouched. |
| AC12 | Met | Diff omits `query_store.rs` / `project_list_footer.rs` / `sync.rs` / `forget.rs` / contracts / Cargo.toml / lock. `project.rs` no new named helpers (call-site + F39 comment only; +13 lines vs planning HEAD). No clap/rusqlite bump. No production unwrap/expect/panic. |
| AC13 | Met | Default `project list` still table (`label` header; not `{`) in AC3 hermetic + AC10 |
| AC14 | Met | AC3 stdout cwd id `matches` count == 1 |

## Targeted gates (R1)

```text
cargo nextest run -p ai-brains-cli promote_cwd_owner
  4 passed
cargo nextest run --test project_list_cwd_first --test project_list_labels --test next_action_honesty
  20 passed (AC3–AC9 / T267 footer / T212 JSON / T198 empty)
cargo clippy -p ai-brains-cli --all-targets -- -D warnings
  exit 0
cargo fmt --check
  exit 0 (after rustfmt on list() call)
```

## Manual (classify-only)

```text
cargo run -q -p ai-brains-cli -- project list
  label … path
  *C:\dev\ai-brains  3581317d-601e-44f7-ab84-fde90aa12d3c  3647  …
  (no alias)         7d97a456-f2f4-43ea-1f13-211af684ad37 18043  …  C:\dev\crawlx
  stderr: 27 project(s) have no alias.
  Example: ai-brains project set-alias 33ec90e0-be74-4159-0000-000000000000 my-project
  exit 0
  .env SHA256 CB6E1F0ECAF00C8C749F6B59693BF198A690B8188F1DBAA0DF2C64DD4512702D unchanged
  Did not cargo install.

cargo run -q -p ai-brains-cli -- project list --format json
  api_version 1; keys api_version, projects, unaliased_count
  projects[0] 7d97a456-… 18043 (max); cwd idx 3
```

PATH `ai-brains` remains until `cargo install` (F13). Source/`cargo run` is DoD.
