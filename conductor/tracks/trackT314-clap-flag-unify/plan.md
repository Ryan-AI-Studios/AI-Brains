# T314 Plan — clap flag unify (`--format` / `--dry-run`)

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** UX / CLI
**Ledger (planning):** DOCS `23da7568-f134-4dde-8a9a-3842eb213cb7`

---

## Preflight (plan time — 2026-08-28)

| Check | Result |
|-------|--------|
| HEAD / tree | `ae6615d` T315 `#231` CLEAN. Branch `track/T314-clap-flag-unify`. `origin/main` = `ae6615d` (ahead **0**). |
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

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains` (not Helping Hands)
- [ ] Re-read `GovernedQueryCommands::{Progressive,Expand,Trace}` and `BriefingCommands` dry_run attrs
- [ ] Re-read `ProjectCommands::ScanRoots` + dispatch `:5251`
- [ ] Re-read `TRACE_MISSING_NEXT_STEP` / `TRACE_PROGRESSIVE_PERSIST` (`governed_query.rs:29–34`)
- [ ] Re-read `run_expand` `:169–219` and `missing_trace_is_human` `:242–248`
- [ ] Confirm clap lock still **4.6.1** (or note drift); `#5912` still in-tree (`num_args 0..=1` legal)
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] Confirm T325 placeholder still Pending (do not steal F8 recency)
- [ ] Confirm T290 F10 still no progressive `--format`
- [ ] `ledgerful ledger start T314-clap-flag-unify --category FEATURE`
- [ ] **Do not** `cargo install` / live production `pin` / `.env` rewrite / clap 5 bump in Phase 0

## Phase 1 — Red

- [ ] `query_progressive__dry_run_bare__parses_true` (AC1)
- [ ] `query_progressive__dry_run_false__parses_false` + omitted default true (AC2)
- [ ] `briefing_project__dry_run_bare__parses_true` + `briefing_personal__dry_run_bare__parses_true` (AC3)
- [ ] `query_expand__format_json__parses` + human + default json (AC4)
- [ ] `query_expand__format_JSON__clap_invalid_value` + xml (AC5)
- [ ] `scan_roots__dry_run__parses` (AC6)
- [ ] Hermetic AC10 expand `--format human` two lines
- [ ] Hermetic AC11 progressive bare `--dry-run` deny
- [ ] Confirm those tests **fail** on current tree (Set requires value; expand has no `--format`; scan-roots unknown arg)
- [ ] AC7 progressive `--format` unexpected + AC8 T291 freeze + AC9 expand JSON unknown are **stay-green** (not Phase-1 red)

## Phase 2 — Green

- [ ] F1 attribute block on Progressive + Briefing Project + Personal (no `require_equals`)
- [ ] F7 Expand `format: String` default `json` + Trace `value_parser`; thread through `ExpandHandleOptions` + dispatch
- [ ] F32 `query_format_is_human`; F9 two-line human; JSON `emit_json` default
- [ ] F11 ScanRoots `dry_run: bool`; discard in match; after_help sentence
- [ ] Do **not** edit `project_paths.rs` / `project.rs` / `briefing.rs` / `governed_common.rs` / contracts
- [ ] Do **not** add progressive `--format`
- [ ] Do **not** change T291 consts

## Phase 3 — Stay-green + docs

- [ ] AC7 / AC8 / AC9 / AC12 scan-roots JSON keys
- [ ] T266 scan-roots format tokens stay
- [ ] T268 no writes / `--root` XOR
- [ ] T290 progressive JSON-only
- [ ] T291 missing-trace envelope + `--dry-run false` persist hermetic
- [ ] `governed_first_run_deny_exit.rs:128` explicit `--dry-run true` still green
- [ ] CAPABILITIES split progressive vs expand; scan-roots `--dry-run` accepted (AC13)
- [ ] PROTOCOL-COMPAT: expand human not a wire contract; scan-roots keys frozen
- [ ] CHANGELOG Unreleased T314
- [ ] after_help examples F25

## Phase 4 — Manual + gate

- [ ] AC14 `cargo run` parse of expand `--format json`, progressive `--dry-run`, scan-roots `--dry-run` (PATH-behind not a fail)
- [ ] AC15 empty diff on forbidden files
- [ ] Review log `review.md`; medium+ not dropped
- [ ] `codex-review` after Phase-1 clean (F23)
- [ ] Full gate: `cargo fmt --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run --workspace` ; `cargo deny check` ; `cargo audit` ; `ledgerful verify --scope full`
- [ ] Conductor **Completed**; `deferred.md` closeout row; FEATURE TX commit
- [ ] implement-track Phase 6: push `track/T314-*`, PR, watch GHA `CI` green, squash-merge, prune. Never `git push origin main`.

## DoD (checkable)

- [ ] `query expand --format json` parses; `--format human` two lines not JSON
- [ ] `query progressive "q" --dry-run` parses true; `--dry-run false` parses false; omitted true
- [ ] Briefing project/personal bare `--dry-run` parses true
- [ ] `project scan-roots --dry-run` parses; command still writes nothing
- [ ] Progressive still has no `--format`
- [ ] T291 `--dry-run false` string exact
- [ ] No clap 5; no `project.rs` growth; no new DTO keys
- [ ] Status Completed only after merge hygiene

## Isolation

No clap 5. No `cargo install` as planning/execute unless owner asks. Never `git push origin main`. No T325 steal. No T319 steal.
