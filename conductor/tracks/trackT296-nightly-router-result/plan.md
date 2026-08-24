# T296 Plan — Nightly Router last-result honesty

**Status:** **Completed** (BUGFIX TX `388b9f76-dd66-4978-9a8e-3964d4fb372a`). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F34 / AC1–AC14 + §13 AI fold-in
**Category:** UX / HONESTY
**Ledger TX (planning):** `3b6532dc-54eb-4313-bdf8-477f4348a694` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `314aa590-c779-4c0a-9889-81681319e950` (DOCS)
**Ledger TX (implement):** BUGFIX TX on **go**

---

## AI fold-in (2026-08-24) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 0**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F33/AC3:** hex `0x41306` / `0X41306` / `0x41301` rstest required red.
2. **F34/AC3:** whitespace-only Status ≡ blank (live trim `:195`).
3. **F19/AC10:** CLI-EXIT-CODES both 267009 and 267014 as `SCHED_S_*` success.
4. **F7/AC6:** after_help **does** add 267014 (decline OpenCode “no help change”).
5. **F9:** zero production `nightly.rs`.

---

## Preflight (plan time — 2026-08-24; fold-in refresh)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in on `c7d6e3e` (`main`, T296 plan). Parent `8b95181` T295 `#211`. CLEAN at fold-in start. `origin/main` = `8b95181` until this commit. |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T269/T281.** No T285–T295. Hole is **human Router 267014**. **Do not `cargo install`.** |
| `preflight --summary` | Pinned **4119** (volatile; plan 4102 / OpenCode 4102). In-context **0/0/0**. Word **536** (plan 367 / OpenCode 428). Scope `3581317d`. |
| PATH `nightly --status --quick` | Heading + Last Result **0** + `Router: Ready  last result: 267014` + `task terminated (SCHED_S_TASK_TERMINATED)`. `probe=skipped`. Exit **0**. |
| PATH JSON `--quick` | `last_task_result` `"0"`; `router.last_result` `"267014"`; hint SCHED_S_TASK_TERMINATED; probes `"skipped"`. |
| LIST /V Router | Ready; Last Result **267014**; Last Run **8/19/2026 2:40:07 PM**; Next **N/A**; `C:\llm\router.bat` |
| `nightly --help` after_help | 267009 success sentence; **no 267014** |
| `daemon status` | Stopped — **T297**, not this track |
| Last PR comments | #211 T295 — Cursor/Bugbot/reviews/issue **empty**. **N/A. No T301.** |
| Open PR on HEAD | none (Dependabot remotes: rusqlite `#61`, chrono `#62`, tokio `#59`, thiserror `#60`, tower-http `#58`, actions `#68–#72`) |
| Pins | clap lock **4.6.1** (crates.io **4.6.6**; **no clap 5**); rusqlite **0.39.0** (0.40.2); chrono **0.4.44**; serde_json **1.0.150**; thiserror **2.0.18**; tokio **1.52.3** — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** (**3.897** fold-in; plan 3.906) — do not touch. `nightly.rs` **2128** / `nightly_status.rs` **760** — not top-10. |
| Ledger | 0 pending / 0 drift at scan |
| `ISSUES.md` | **Does not exist** (F23) |
| ledgerful search | `format_router_status_lines` `nightly_status.rs:187` / `nightly.rs:210`; `explain_last_task_result` `nightly.rs:958` |
| Online | Microsoft Learn SCHED_S_TASK_TERMINATED `0x41306` = 267014 **success**; clig.dev just-enough; clap 4.6.6 after_help |
| Skill | `--status` allowed (not a nightly **run**). Did **not** mutate schtasks. |

---

## Phase 0 — on go (re-verify)

- [x] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before BUGFIX TX)
- [x] Re-read `format_router_status_lines` `nightly_status.rs` **`:187–216`** — this is the DoD edit; confirm trim `:195` (F34) and that helper still has no hex parse (F33 is new)
- [x] Re-read `explain_last_task_result` `nightly.rs` **`:958–973`** — **do not edit** (F6)
- [x] Re-read JSON `router_json_from_input` **`:218–230`** — hints stay `explain_last_task_result` (F5)
- [x] Re-read `FROZEN_KEYS` **`:289–308`** — **do not add**
- [x] Re-read after_help `main.rs` **`:1432–1434`** + T269 AC6 **`:952–980`** — additive 267014 only (F7)
- [x] Re-read `NIGHTLY_STATUS_PROBE_TIMEOUT` — **do not edit** (F8)
- [x] Confirm hermetic `--quick` `tests/nightly_status.rs` **`:77`**
- [x] Rescan `conductor/deferred.md` — T296 absorbed; T297–T300 / T240 F2 / 750 ms not stolen
- [x] Confirm `#211` still empty Cursor; no mint; Dependabot `#61` still not this track
- [x] Re-dogfood `nightly --status --quick` **read-only**. Record live Router status + last_result (plan-time Ready / 267014). **Did not** mutate schtasks
- [x] Re-check clap lock **4.6.1**, rusqlite **0.39.0** — **no bump**
- [x] BUGFIX TX (new) `388b9f76-dd66-4978-9a8e-3964d4fb372a`
- [x] Did **not** `cargo install`; did **not** grow `project.rs` / `doctor.rs` / `daemon.rs`

---

## Absorbed deferred

- [x] Audit Router 267014 / TASK_TERMINATED → F1–F7 / AC1–AC3 / AC9
- [x] Placeholder Manual `nightly --status --quick` → AC9
- [x] T255 AC6/AC15 human numeric → rewrite AC2 / AC3
- [x] last-PR #211 Cursor N/A → F18 no T301

---

## Declined (written)

| Item | Why |
|------|-----|
| JSON last_result / hint rewrite | F5 |
| `explain_last_task_result` edit | F6 |
| Raise 750 / daemon HTTP | F8 / F11 → T297 |
| Doctor 16th / persist / `.cmd` | F10 |
| T297–T300 / leftover `--write` | F17 |
| clap 5 / rusqlite 0.40 | F12 |
| T240 F2 | F17 |

---

## Phase 1 — TDD red

- [x] `format_router_status_lines__ready_267014__status_then_terminated_no_numeric` fails
- [x] Rewrite Running+267009 unit → status-only (fails)
- [x] Rewrite blank-status 267009 unit → `Router: running` (fails)
- [x] `format_router_status_lines__blank_status_267014__terminated_phrase` + whitespace `"   "` (F34) fails
- [x] `format_router_status_lines__hex_0x41306__same_as_267014` (F33) fails
- [x] `nightly__help__names_router_267014_success` fails
- [x] Red commit allowed

---

## Phase 2 — green

- [x] Const `ROUTER_LAST_RUN_TERMINATED` + helper body (F1–F4)
- [x] JSON units still raw `"267014"` / `"267009"` + existing hints
- [x] after_help additive sentence; T269 AC6 stay-green
- [x] Hermetic AC8: human `--quick` has no `267014` / no `SCHED_S_TASK_TERMINATED`
- [x] Green commit allowed

---

## Phase 3 — docs

- [x] `Docs/CAPABILITIES.md` T269/T281 bullet
- [x] `Docs/OPERATIONS.md` Router bullet
- [x] `Docs/CLI-EXIT-CODES.md` 267014 next to 267009
- [x] `CHANGELOG.md` T296 Unreleased
- [x] PROTOCOL-COMPAT untouched

---

## Phase 4 — verify

- [x] Targeted: `cargo nextest run -p ai-brains-cli -- nightly_status nightly__help format_router explain_last_task`
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [x] Manual AC9 (`cargo run`, `--no-project-context`, `--status --quick` + json). Record transcript in `review.md`. **No** schtasks mutate
- [x] `scripts/dev-check.ps1`
- [x] Phase-1 review → `review.md`
- [x] `codex-review` (F22) → `review.codex.md`

---

## Phase 5 — closeout

- [x] conductor T296 **Completed**
- [x] deferred.md T296 closeout table
- [x] README-T285-T300 T296 Completed
- [x] `ai-brains pin` DECISION (human omits 267014; JSON frozen) — `1ec245ad-9b85-47f5-8fcc-e4d5dacc19df`
- [ ] BUGFIX TX commit
- [ ] 0 pending / 0 drift

---

## Phase 6 — publish (standing)

- [ ] Push `track/T296-*` (never `git push origin main`)
- [ ] PR to `main` if none
- [ ] `gh run watch --exit-status` until GHA `CI` every job green
- [ ] `gh pr merge --squash --delete-branch`
- [ ] `git fetch --all --prune`; point local `main` at `origin/main`; delete merged local `track/T296-*` only

---

## DoD (checkable)

- [x] AC1 Ready+267014 → `Router: Ready` + `last run: terminated`; no numeric; no SCHED_S
- [x] AC2 Running+267009 → `Router: Running` only
- [x] AC3 hex `0x41306` (F33) + whitespace Status (F34)
- [x] AC5 JSON still `"267014"` + existing hint
- [x] AC6 after_help 267014 success; T269 needles stay
- [x] AC7 `explain_last_task_result` units green
- [x] AC8 hermetic `--quick` heading + no 267014 on human
- [x] AC9 live Manual: Nightly `Last task result: 0` still present; Router human not `267014`
- [x] AC11 engine/doctor/daemon/`explain_last_task_result` untouched
- [x] AC13 full gate green
- [x] F0 was respected (no product commits as planning)

---

## Stop-before (even after go)

- Live `schtasks /End` / `/Change` / `/Create` / `/Run` on Nightly or Router
- `cargo install`
- `.env` rewrite (T240 F2)
- `retention apply --confirm`
- `graph rebuild`
- leftover `rebind-path --write --yes`
- `safety sync` without `--dry-run`
- clap 5 / rusqlite 0.40
- Growing `project.rs` / `doctor.rs` / `daemon.rs`
