# T314 Plan — clap flag unify (`--format` / `--dry-run`)

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** UX / CLI
**Ledger (planning):** DOCS `23da7568-f134-4dde-8a9a-3842eb213cb7`
**Ledger (fold-in):** DOCS `0d3c2e80-a309-41c0-b49b-08627ec2d373`

---

## Preflight (plan time — 2026-08-28)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `2a1eb35` plan commit CLEAN; `origin/main` = `ae6615d` (ahead **1**). Plan-write was `ae6615d` / ahead **0** (Agy m1). Branch `track/T314-clap-flag-unify`. Product `src/` = T315 `#231`. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T312/T315 **not** on PATH. T314 clap holes **are**. |
| `preflight --summary` (PATH) | Pinned **4536**; in-context **0/0/0**; `Total Word Count: 705` (PATH-behind T315) |
| `query expand --format json` | unexpected `--format` (tip `--log-format`) |
| `query progressive "…" --dry-run` | value required for `--dry-run <DRY_RUN>` |
| `briefing project --dry-run` / `personal` | same required-value |
| `project scan-roots --dry-run` | unexpected `--dry-run` |
| Clap structs | Progressive `:2238` Set; Briefing `:2197/:2213` Set; Expand no format `:2245`; ScanRoots no dry_run `:3153`; Trace tokens `:2263` |
| T291 SOOT | `governed_query.rs:29–34` `--dry-run false` exact |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#231` `mergedAt` **2026-08-28T04:49:26Z**; comments/reviews **[]** — **N/A empty**. `#230` → **T325** already. |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan; this TX `23da7568` |
| `ISSUES.md` | **Does not exist** |
| Planning install / live pin | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit 5 clap errors | **DoD** F1/F7/F11 / AC1–AC6 / AC10–AC14 |
| Briefing same Set trap | **F5** / AC3 |
| T290 F10 progressive no `--format` | **F6** / AC7 |
| T291 `--dry-run false` | **F3** / AC8 |
| T268 scan-roots already dry-run | **F11** no-op flag |
| last-PR `#231` Cursor | **N/A empty** F21 |
| last-PR `#230` F8 recency | **T325** — not stolen |
| T319 / T321 / T324 / clap 5 | **Not stolen** / **Decline** |
| OpenCode m1 AC7 absent | **F6 / AC7** new Phase 1 green-on-arrival unit |
| OpenCode O1 found newlines | **F9 / AC16** |
| OpenCode O2 AC14 `--project-id` | **AC14** |
| OpenCode O3 PROTOCOL-COMPAT add expand row | **F25 / AC13** |
| OpenCode O4 help-shape AC | **Decline** as DoD |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [x] Confirm cwd `C:\dev\AI-Brains` (not Helping Hands)
- [x] Re-read `GovernedQueryCommands::{Progressive,Expand,Trace}` and `BriefingCommands` dry_run attrs
- [x] Re-read `ProjectCommands::ScanRoots` + dispatch `:5251`
- [x] Re-read `TRACE_MISSING_NEXT_STEP` / `TRACE_PROGRESSIVE_PERSIST` (`governed_query.rs:29–34`)
- [x] Re-read `run_expand` `:169–219` and `missing_trace_is_human` `:242–248`
- [x] Confirm clap lock still **4.6.1** (or note drift); `#5912` still in-tree (`num_args 0..=1` legal)
- [x] Rescan `deferred.md` open overlapping rows
- [x] Confirm T325 placeholder still Pending (do not steal F8 recency)
- [x] Confirm T290 F10 still no progressive `--format`
- [x] `ledgerful ledger start T314-clap-flag-unify --category FEATURE`
- [x] **Do not** `cargo install` / live production `pin` / `.env` rewrite / clap 5 bump in Phase 0

## Phase 1 — Red

- [x] `query_progressive__dry_run_bare__parses_true` (AC1)
- [x] `query_progressive__dry_run_false__parses_false` + omitted default true (AC2)
- [x] `briefing_project__dry_run_bare__parses_true` + `briefing_personal__dry_run_bare__parses_true` (AC3)
- [x] `query_expand__format_json__parses` + human + default json (AC4)
- [x] `query_expand__format_JSON__clap_invalid_value` + xml (AC5)
- [x] `scan_roots__dry_run__parses` (AC6)
- [x] Hermetic AC10 expand `--format human` two lines
- [x] Hermetic AC11 progressive bare `--dry-run` deny
- [x] Confirm those tests **fail** on current tree (Set requires value; expand has no `--format`; scan-roots unknown arg)
- [x] **Write AC7** `query_progressive__format_json__unexpected_argument` (green-on-arrival — passes on HEAD; OpenCode m1: do **not** skip as stay-green)
- [x] AC8 T291 freeze + AC9 expand JSON unknown are **stay-green** (not Phase-1 red)

## Phase 2 — Green

- [x] F1 attribute block on Progressive + Briefing Project + Personal (no `require_equals`)
- [x] F7 Expand `format: String` default `json` + Trace `value_parser`; thread through `ExpandHandleOptions` + dispatch
- [x] F32 `query_format_is_human`; F9 human `kind` then preview verbatim (two-line count Unknown/Denied only); JSON `emit_json` default
- [x] F11 ScanRoots `dry_run: bool`; discard in match; after_help sentence
- [x] Do **not** edit `project_paths.rs` / `project.rs` / `briefing.rs` / `governed_common.rs` / contracts
- [x] Do **not** add progressive `--format`
- [x] Do **not** change T291 consts

## Phase 3 — Stay-green + docs

- [x] AC7 (now written) / AC8 / AC9 / AC12 scan-roots JSON keys
- [x] T266 scan-roots format tokens stay
- [x] T268 no writes / `--root` XOR
- [x] T290 progressive JSON-only
- [x] T291 missing-trace envelope + `--dry-run false` persist hermetic
- [x] `governed_first_run_deny_exit.rs:128` explicit `--dry-run true` still green
- [x] CAPABILITIES split progressive vs expand; scan-roots `--dry-run` accepted (AC13)
- [x] PROTOCOL-COMPAT: **add** `query expand` CLI row (HandlePreviewDto + `applied_scope`; human not a wire contract); scan-roots keys frozen
- [x] CHANGELOG Unreleased T314
- [x] after_help examples F25

## Phase 4 — Manual + gate

- [x] AC14 `cargo run` parse of expand `--format json --project-id <uuid>`, progressive `--dry-run`, scan-roots `--dry-run` (PATH-behind not a fail)
- [x] AC15 empty diff on forbidden files
- [x] Review log `review.md`; medium+ not dropped
- [x] `codex-review` after Phase-1 clean (F23)
- [x] Full gate: `cargo fmt --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run --workspace` ; `cargo deny check` ; `cargo audit` ; `ledgerful verify --scope full`
- [x] Conductor **Completed**; `deferred.md` closeout row; FEATURE TX commit
- [x] implement-track Phase 6: push `track/T314-*`, PR, watch GHA `CI` green, squash-merge, prune. Never `git push origin main`.

## DoD (checkable)

- [x] `query expand --format json` parses; `--format human` two lines not JSON
- [x] `query progressive "q" --dry-run` parses true; `--dry-run false` parses false; omitted true
- [x] Briefing project/personal bare `--dry-run` parses true
- [x] `project scan-roots --dry-run` parses; command still writes nothing
- [x] Progressive still has no `--format`
- [x] T291 `--dry-run false` string exact
- [x] No clap 5; no `project.rs` growth; no new DTO keys
- [x] Status Completed only after merge hygiene

## Isolation

No clap 5. No `cargo install` as planning/execute unless owner asks. Never `git push origin main`. No T325 steal. No T319 steal.
