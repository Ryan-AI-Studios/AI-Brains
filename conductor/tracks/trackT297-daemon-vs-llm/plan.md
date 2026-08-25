# T297 Plan — daemon status Stopped vs backend TCP Open

**Status:** **Completed** 2026-08-24
**Spec:** [spec.md](./spec.md) F0–F36 / AC1–AC14
**Category:** UX / HONESTY
**Ledger TX (planning):** `3f147d91-b4f9-42b2-a8c3-8ea01dd1292d` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `338d0a5b-9dba-446a-8e07-89e0af81f610` (DOCS)
**Ledger TX (implement):** BUGFIX `a3c47213-69bf-4d30-85ea-bd6758e7022b`

---

## AI fold-in (2026-08-24) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 1**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F28/AC8:** keep-bound `TcpListener` hermetic is required `run_status` wiring proof (OpenCode M1).
2. **F35/AC1:** rstest 8 triples; include `(false, true, true)`.
3. **F30/AC6:** both-Open tail is exactly `[const, next:]`.
4. **F36:** capture by backend `name`, not both-bools-from-one-state.
5. **F20/AC7:** exact after_help uses `TCP connect`; in-process `try_parse_from`.
6. **F19:** docs distinguish T281 HTTP contrast vs T297 IPC vs TCP.

---

## Phase 0 — on go (re-verify)

- [x] `git fetch --all --prune` ; branch `track/T297-daemon-vs-llm`
- [x] `ledgerful doctor` ; ledger 0 pending / 0 drift before BUGFIX TX
- [x] Re-read `run_status` / TCP loop / `status_next_line` / early-route / Status clap
- [x] Re-dogfood `daemon status` read-only — **Running** + 8081/8083 Open; no contrast pre-change
- [x] Pins clap **4.6.1** / rusqlite **0.39.0** — no bump
- [x] BUGFIX TX `a3c47213`
- [x] Did **not** `cargo install`; did **not** grow hotspots / stop daemon

---

## Absorbed deferred

- [x] Audit Stopped vs LLM Open → F1–F6 / AC1–AC6 / AC10
- [x] Placeholder Manual `daemon status` / do not start daemon → AC10 / F11
- [x] T281 F27 Stopped + port Open → this track
- [x] T296 F11 / OpenCode m3 → this track
- [x] last-PR #212 Cursor N/A → F14 no T301
- [x] Fold-in OpenCode M1 keep-bound AC8; F35 rstest; F36 name-match; F30 both-Open

---

## Phase 1 — TDD red / Phase 2 — green

- [x] Const + helpers + F36 name-match + `status_report_tail` print
- [x] Units AC1–AC6 + AC5 U+2260 + AC7 help + unknown `--format` clap
- [x] AC8 keep-bound listener hermetic
- [x] Status exact F20 `after_help`

---

## Phase 3 — docs

- [x] CAPABILITIES `:110` last-line `next:` kept; additive contrast; T281 vs T297
- [x] OPERATIONS `:558` additive; T281 vs T297
- [x] Root CHANGELOG T297 Unreleased
- [x] PROTOCOL-COMPAT untouched

---

## Phase 4 — verify

- [x] Targeted nextest 25+ (contrast/tail/next/help/keep_bound/T85/T94/T128)
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [x] Manual AC10 Running+Open → contrast absent (no daemon stop)
- [x] Unrelated live-daemon force-restore soft-skip (recovery_drills + smoke)
- [x] `scripts/dev-check.ps1` **SUCCESS**
- [x] Phase-1 `review.md` + Codex `review.codex.md` (product PASS; process P1 closed by closeout; P2 fixed)

---

## Phase 5 — closeout

- [x] conductor T297 **Completed**
- [x] deferred.md T297 closeout table (unresolved lows)
- [x] README-T285-T300 T297 Completed
- [x] `ai-brains pin` DECISION
- [x] BUGFIX TX commit
- [ ] 0 pending / 0 drift (after commit)

---

## Phase 6 — publish (standing)

- [ ] Push `track/T297-*`
- [ ] Open PR to `main` if none
- [ ] `gh run watch --exit-status` until GHA `CI` every job green
- [ ] `gh pr merge --squash --delete-branch`
- [ ] Hygiene: fetch/prune; point local `main` at `origin/main`

---

## DoD

- [x] Stopped+Open prints frozen contrast; `next:` still last; Running+Open omits contrast
- [x] Units AC1–AC6; U+2260 locked; AC8 keep-bound
- [x] Manual AC10 recorded; **did not** start/stop daemon
- [x] Full gate green; contracts/pins unchanged
- [ ] Published (Phase 6)
