# T324 Plan — PowerShell empty TERM on `in-force`

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** BUGFIX / UX / WINDOWS
**Ledger (planning):** DOCS `3b998d33-ac46-4a8c-9074-aebcc5931e46`

---

## Preflight (plan time — 2026-08-29)

| Check | Result |
|-------|--------|
| HEAD / tree | Product `5b50d56` T323 `#245`. Branch `track/T324-powershell-empty-term`. `origin/main` = `5b50d56`. Dirty conductor T323 Completed + residuals — absorbed here. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T311 **on PATH**. T312–T323 **not**. Decision hole **is** on PATH (5.1). Conclusion in-force **source-only**. |
| `preflight --summary` (PATH) | Pinned **4645**; in-context **0/0/0**; `Total Word Count: 753` (PATH-behind T315). |
| pwsh 7.6 Windows `decision in-force ""` | `term must be non-empty` exit **2** (preserved) |
| powershell.exe **5.1.26100.9168** `decision in-force ""` | clap **missing `<TERM>`** exit **2** — **hole** |
| pwsh Legacy `""` | missing `<TERM>` (drop) |
| `'""'` / `--% ""` (pwsh) | term `"\"\""`, ruling null, exit **0** — **not empty** |
| PATH `--term` | unexpected argument |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; time `"0.3"` / lock **0.3.47**; rusqlite **0.40.2**; serde_json **1.0.150**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#245` empty. `#237` → **T326**. `#230` → **T325**. **No T327.** |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX) |
| Hotspots | CLI `project.rs` #1 — do not touch. `governed_common.rs` #3 — do not grow. |
| Line counts (physical) | `main.rs` **5782**; `decision.rs` **347**; `conclusion.rs` **273**; CLI tests decision **443** / conclusion **306**. F22 = go-HEAD diff. |
| `ISSUES.md` | **Does not exist** |
| Planning install / live propose | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| T311 R7 / I3 | **DoD** F1–F11 / AC1–AC9 / AC15–AC16 |
| T323 same clap positional | **Absorb** F1 / AC9 |
| Placeholder docs-only vs flag | **Pick `--term`** F1 / F30 |
| T322 `--as-of` empty | **Decline** F6 |
| T311 R1 daemon | **Decline** F13 |
| last-PR `#245` | **N/A empty** |
| last-PR `#237` / `#230` | **T326** / **T325** — not stolen |
| T323 residuals / T325 / T326 / clap 5 | **Not stolen** / **Decline** |
| T323 uncommitted conductor note | **Plan-write DOCS commit** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains`
- [ ] Re-dogfood **powershell.exe 5.1** `decision in-force ""` still clap missing `<TERM>`
- [ ] Re-dogfood pwsh Windows `""` still fail_usage
- [ ] Re-read clap `DecisionCommands::InForce` `main.rs:2949–2971` + `ConclusionCommands::InForce` `:2889–2904`
- [ ] Re-read dispatch `:4996–5058`
- [ ] Re-read `fail_usage` `governed_common.rs:342–346` (do **not** edit)
- [ ] Confirm clap lock still **4.6.1**; T323 Completed; T325 / T326 still Pending (do not steal)
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] `ledgerful ledger start T324-powershell-empty-term --category BUGFIX`
- [ ] **Do not** `cargo install` / live propose / `.env` rewrite / clap 5 / `require_equals` / grow `governed_common.rs` / edit CP resolvers

## Phase 1 — Red

- [ ] `decision_in_force__omitted_term__fail_usage_exit_2` (AC1) — must **fail** (clap missing `<TERM>` today)
- [ ] `decision_in_force__term_flag_no_value__fail_usage_exit_2` (AC2) — must **fail** (unexpected `--term`)
- [ ] `decision_in_force__term_flag_equals_empty__fail_usage_exit_2` (AC3) — must **fail**
- [ ] Conclusion mirrors AC1–AC3 (AC9) — must **fail**
- [ ] Confirm existing empty-term hermetics still **pass** on this red commit (F8 / F24)

## Phase 2 — Green (clap)

- [ ] Optional positional + `--term` F1/F2 on **both** InForce structs (copy-not-share)
- [ ] Dispatch merge F2; `InForceOptions.term: String` unchanged
- [ ] `conflicts_with` F37 / AC7
- [ ] after_help F7 (`--term=` empty SOOT; no `'""'` / `--%`)
- [ ] AC4–AC6 / AC8 / AC10–AC12 stay-green or help update
- [ ] AC9 conclusion copy

## Phase 3 — Docs

- [ ] CHANGELOG Unreleased
- [ ] CAPABILITIES Family C `--term` (AC14)
- [ ] OPERATIONS `--term=` empty example
- [ ] CLI-EXIT-CODES: in-force omit / `--term=` is `fail_usage`, not clap missing `<TERM>`

## Phase 4 — Targeted gate

- [ ] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` (AC13)
- [ ] nextest `-p ai-brains-cli --test decision_in_force --test conclusion_in_force`
- [ ] Manual AC15–AC18 (5.1 `""` + `--term=`; pwsh stay-green; source conclusion `--term=`) — **no** live propose
- [ ] Implement-track full gate + PR + GHA + squash (never `git push origin main`)

## DoD (after go)

- [ ] AC1–AC18
- [ ] T311/T322/T323 tests stay-green except AC8 help angle-brackets (F24)
- [ ] No clap 5; no `require_equals`; no CP resolver edits
- [ ] T325 / T326 / T307 / H2 **not stolen**
- [ ] Medium+ review findings not silently dropped

## Isolation

No daemon DTO. No H2. No `cargo install`. No live vault lifecycle writes. Never `git push origin main`.
