# T291 Plan — query trace missing envelope + human next

**Status:** **Pending** (Planned). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F26 / AC1–AC13
**Category:** FEATURE / UX / HONESTY
**Ledger TX (planning):** `c59e5bb6-adf1-40c5-9288-66403d208aca` (DOCS)
**Ledger TX (implement):** FEATURE on **go**

---

## Preflight (plan time — 2026-08-23)

| Check | Result |
|-------|--------|
| HEAD / tree | `37012fe` T290 `#206`. CLEAN. `origin/main` = HEAD |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T274. No T285–T290.** Hole is in **source**. **Do not `cargo install`.** |
| `query trace missing-id` | stdout token `null`; exit **0** |
| `query trace missing-id --format human` | clap unexpected `--format` (tip: `--log-format`) |
| `query progressive` `--dry-run` | default **true** — no persist |
| `preflight --summary` | Pinned **3976**; in-context **0/0/0**; word **381** |
| Last PR comments | #206 T290 — Cursor Bugbot **1 Low** sanitizer collapse. **Absorb F16.** **No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; GitHub **v4.6.6**; **no clap 5**); serde_json **1.0.150** (crates.io 1.0.151); chrono **0.4.44**; rusqlite **0.39.0** (0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** — do not touch. `governed_common.rs` **#3** — sanitizer collapse only. `governed_query.rs` not top-10 — envelope here. CLI `preflight.rs` **#8** — do not grow. |
| Ledger | 0 pending / 0 drift at scan; planning TX `c59e5bb6` |
| `ISSUES.md` | **Does not exist** (F22) |
| ledgerful search | `run_trace` → `governed_query.rs:205`; `get_query_trace` → `query.rs:413` |
| Online | clig.dev human-first + next-command; T180 type-change documented; clap 4.6.6 / no clap 5 |
| Skill | CAPABILITIES Trace row (F14) |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [ ] Re-read `governed_query.rs` `run_trace` `:205` None arm `println!("null")`
- [ ] Confirm `QueryTraceDto` still has **no** `found` / `next_step` — **do not add**
- [ ] Confirm progressive `--dry-run` default still **true** — **do not flip** (F9)
- [ ] Confirm no `--trace` on progressive
- [ ] Confirm `get_query_trace` still `Ok(None)` for miss / cross-principal / no grant — **do not change CP**
- [ ] Confirm `sanitize_recall_query` still sets `prev_space = false` on `$`/backtick — F16 still true
- [ ] Confirm hermetic `query_trace__unknown__stdout_null_exit_0` still exact `"null"`
- [ ] Rescan `conductor/deferred.md` — T291 absorbed + #206; T292–T300 / T290 not stolen
- [ ] Confirm #206 Bugbot still the only Cursor finding; no mint; Dependabot `#61` still not this track
- [ ] Re-dogfood `query trace missing-id` **read-only**. **Did not** pin production decisions; **did not** write `.env`; **did not** extra `policy bootstrap`
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [ ] FEATURE TX (new)
- [ ] Did **not** `cargo install`; did **not** grow `project.rs` / `preflight.rs` / `briefing.rs` / `personal.rs` / CP `persist_trace`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit `query trace` `null` U=3 | **DoD** F1–F8 / AC1–AC4 / AC11 |
| Placeholder Manual two commands | **DoD** AC11 |
| T263 F6 scalar freeze | **Lift** F1 / F7 envelope |
| #206 Bugbot sanitizer collapse | **DoD** F16 / AC8 |

---

## Phase 1 — red (required first)

- [ ] `trace_missing_next_step__frozen__exact_string` (AC1) — F8 const exact
- [ ] `query_trace__unknown__stdout_envelope_exit_0` (AC2) — fail while stdout is `null`
- [ ] `sanitize_recall_query` rstest `a $ b` → `a b` (AC8) — fail on current helper
- [ ] Commit red allowed

## Phase 2 — green missing path

- [ ] `TRACE_MISSING_NEXT_STEP` + missing envelope struct in `governed_query.rs`
- [ ] `run_trace` None arm: json → `emit_json` envelope; human → two lines (F2)
- [ ] clap `--format` on `Trace`; dispatch; after_help
- [ ] `sanitize_trace_id` (F15); share collapse with F16
- [ ] Fix `sanitize_recall_query` interpolator-as-whitespace + final trim (F16)
- [ ] AC3 human hermetic; AC4 no-project; AC7 InvalidValue `JSON`
- [ ] AC9 unit id sanitize

## Phase 3 — found + stay-green

- [ ] AC5 hermetic persist (`--dry-run false`) then `query trace <id>`
- [ ] Found `--format human` still QueryTraceDto (F10)
- [ ] AC6 T290/T221 stay green; ellipsis const exact
- [ ] AC12 `QueryTraceDto` no new fields

## Phase 4 — docs + gate

- [ ] CAPABILITIES Trace row; OPERATIONS; CLI-EXIT-CODES; PROTOCOL-COMPAT §5 + §3.1; clap after_help; CHANGELOG
- [ ] `cargo fmt --check` ; `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Targeted nextest then workspace gate (`dev-check` / nextest + deny + audit)
- [ ] Manual AC11 `cargo run -p ai-brains-cli -- query trace missing-id` and `--format human`
- [ ] `ledgerful verify --scope full`

## Phase 5 — review + publish

- [ ] `conductor/tracks/trackT291-query-trace-next/review.md` phase-1
- [ ] Codex/cross-model when FEATURE
- [ ] Mark conductor **Completed**; append closeout residuals to `deferred.md`
- [ ] Push `track/T291-*` ; PR ; `gh run watch --exit-status` ; `gh pr merge --squash --delete-branch`
- [ ] Fetch prune; point local `main` at `origin/main`; delete merged local `track/T291-*` only
- [ ] Never `git push origin main`. Never force-push.

---

## DoD (checkable)

- [ ] Default `query trace missing-id` is **not** the token `null`; JSON `found: false` + F8 `next_step`
- [ ] `--format human` has `No trace` and `next:` and `--dry-run false`
- [ ] Exit **0**; no project-id required
- [ ] Found DTO unchanged; no invented traces
- [ ] #206 `a $ b` collapses to `a b`
- [ ] Docs no longer say “JSON token null (not an object)”
- [ ] Status stays **Pending** until go; this file’s implement boxes stay unchecked through planning
