# T323 Plan — conclusion in-force resolver

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** FEATURE
**Ledger (planning):** DOCS `61b188d1-fd07-48e6-9bec-bdce0d197c60`
**Ledger (fold-in):** DOCS `853b18d9-ee2e-4ed9-afe3-01962bab0430`

---

## Preflight (plan time — 2026-08-29)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `0ead377` plan commit CLEAN at fold start; `origin/main` = `766a6c8` (ahead **1**). Plan-write was `766a6c8` / ahead **0** (Agy m2). Branch `track/T323-conclusion-in-force`. Product `src/` = T322 `#244` `766a6c8`. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T311 **on PATH**. T312–T322 **not**. Hole **is** (`conclusion` Propose-only; `in-force` unrecognized exit 1). |
| `preflight --summary` (PATH) | Pinned **4640**; in-context **0/0/0**; `Total Word Count: 777` (PATH-behind T315). |
| PATH `conclusion --help` | `propose` only |
| PATH `conclusion in-force --help` | unrecognized subcommand exit **1** |
| Chain | **Exists** — `correct_conclusion` + projector `superseded_by`. **Walker, not decline.** |
| `conclusion_valid_at` | `briefings/project.rs:688` private — **pub(crate) only** |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; time `"0.3"` / lock **0.3.47** / crates.io **0.3.55**; rusqlite **0.40.2**; serde_json **1.0.150**; uuid ws `"1.13"` / lock **1.23.1**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#244` empty. `#237` → **T326**. `#230` → **T325**. **No T327.** |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX) |
| Hotspots | CLI `project.rs` #1 — do not touch. `governed_common.rs` #3 — do not grow. `in_force.rs` not top 10 — **do not edit**. |
| Line counts | CLI `conclusion.rs` **188**; CP `conclusions.rs` **472**; `in_force.rs` **276**; `project.rs` **969**; store conclusion projector **131**. F22 = go-HEAD diff. |
| `ISSUES.md` | **Does not exist** |
| Planning install / live propose | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| T311 R5 conclusion in-force | **DoD** F1–F12 / AC1–AC13 / AC16 — walker |
| Placeholder “decline if no chain” | **Superseded** — §2.2 chain proof |
| T311 R1 daemon | **Decline** F13 |
| T311 R7 empty TERM | **Not stolen** T324 |
| T322 `--as-of` | **Not stolen** / F30 decline copy |
| last-PR `#244` | **N/A empty** |
| last-PR `#237` / `#230` | **T326** / **T325** — not stolen |
| T322 residuals / T325 / T326 / clap 5 | **Not stolen** / **Decline** |
| T322 uncommitted conductor note | **Plan-write DOCS commit** |
| Agy m2 HEAD snapshot | **Folded** `0ead377` / ahead **1** |
| OpenCode m1 F37 cycle | **Folded** `EventBuilder` single `ConclusionSuperseded` self-hop |
| OpenCode m2 three-hop | **Folded** AC17 |
| OpenCode O2 AC16 | **Folded** live-null expected |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains`
- [ ] Re-read `conclusions.rs` `correct_conclusion` `:355–441` + `confirm_conclusion` `:186–259` + `reject_conclusion` + `activate_conclusion`
- [ ] Re-read projector `store/projections/conclusion.rs:91–112` (`ConclusionSuperseded`)
- [ ] Re-read `conclusion_valid_at` `briefings/project.rs:688–690` (visibility-only)
- [ ] Re-read clap `ConclusionCommands` `main.rs:2852–2885` + dispatch `:4948–4976`
- [ ] Re-read T311 `in_force.rs` as **pattern only** — do **not** edit
- [ ] Re-dogfood `conclusion --help` + `conclusion in-force` still unknown until green
- [ ] Confirm clap lock still **4.6.1**; T322 Completed; T324 / T325 / T326 still Pending (do not steal)
- [ ] Confirm snapshot still `0ead377` / product `766a6c8` or re-cite go HEAD (F22)
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] `ledgerful ledger start T323-conclusion-in-force --category FEATURE`
- [ ] **Do not** `cargo install` / live `conclusion propose` / confirm / correct / `.env` rewrite / clap 5 / grow `governed_common.rs` / edit `in_force.rs` / projector

## Phase 1 — Red

- [ ] `resolve_conclusion_in_force__superseded_root__current_confirmed_in_force` (AC1) — must **fail** (module missing)
- [ ] `resolve_conclusion_in_force__three_hop_chain__tip_ruling_len2` (AC17) — must **fail** (same missing fn)
- [ ] `conclusion_in_force__help__lists_term_scope_format` (AC8) — must **fail** (no subcommand)
- [ ] Confirm T311/T322 stay-green tests still **pass** on this red commit (F24)

## Phase 2 — Green (CP)

- [ ] `pub(crate) fn conclusion_valid_at` (no other `project.rs` edits)
- [ ] `conclusion_in_force.rs` + `lib.rs` export (F1 / F29 new types)
- [ ] F5–F9 walk + Active\|Confirmed ruling
- [ ] AC1–AC7 / AC11 (Active-only) / AC12 (Candidate-only) / AC13 (uncorrected successor) / AC17 (three-hop)
- [ ] F35 evidence on confirm fixtures; F36 AC13; F37 `EventBuilder` self-`ConclusionSuperseded` (not `correct_conclusion`)

## Phase 3 — Green (CLI)

- [ ] `ConclusionCommands::InForce` + dispatch + after_help (F2 / F3 / F21)
- [ ] `run_in_force` `ReadConclusions` (F10)
- [ ] Human F12 / ruling uses `statement`
- [ ] AC8 / AC9 / AC10 / AC5 CLI
- [ ] Stay-green decision in-force tests (untouched)

## Phase 4 — Docs

- [ ] CHANGELOG Unreleased Added
- [ ] CAPABILITIES Family C `conclusion in-force` row (AC15)
- [ ] OPERATIONS one example (AC15)

## Phase 5 — Targeted gate

- [ ] `cargo clippy -p ai-brains-control-plane -p ai-brains-cli --all-targets -- -D warnings` (AC14)
- [ ] nextest those packages + new tests
- [ ] Manual AC16 (`cargo run` help + live `workspace_id` → `ruling: null`) — **no** live propose
- [ ] Implement-track full gate + PR + GHA + squash (never `git push origin main`)

## DoD (after go)

- [ ] AC1–AC17
- [ ] T311/T322 tests stay-green (no `in_force.rs` edit)
- [ ] No `--as-of`; no daemon DTO; no projector `valid_until` close
- [ ] T324 / T325 / T326 / T307 / H2 / clap 5 **not stolen**
- [ ] Medium+ review findings not silently dropped

## Isolation

No daemon DTO. No H2. No `cargo install`. No live vault lifecycle writes. Never `git push origin main`.
