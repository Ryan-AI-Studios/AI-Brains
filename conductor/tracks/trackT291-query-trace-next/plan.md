# T291 Plan — query trace missing envelope + human next

**Status:** ✅ **Completed**. Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F26 / AC1–AC13 + §13 AI fold-in
**Category:** FEATURE / UX / HONESTY
**Ledger TX (planning):** `c59e5bb6-adf1-40c5-9288-66403d208aca` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `627d3871-b5c6-4e03-8b11-9588a61777d1` (DOCS)
**Ledger TX (implement):** FEATURE on **go**

---

## AI fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0**. OpenCode M-1 hotspot **folded**. M-2/M-3 volatile snapshot. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F3/AC7 (Agy m1 + OpenCode o-1):** clap `value_parser` T266 set — **not** `OutputFormat::parse`.
2. **AC5 (OpenCode o-2):** System bootstrap omit `--principal-id`; `--dry-run false`; not `progressive_cmd`.
3. **AC10 (OpenCode o-6 + Agy O1):** OPERATIONS both null phrases; after_help `:1589` / `:1892` / `:1924`; `query trace --help` forbids `JSON token null`.
4. **F16 (Agy m2):** interpolators = space boundary; no double-space.
5. **Hotspot (OpenCode M-1):** `governed_common.rs` **#2** (3.806).

---

## Preflight (plan time — 2026-08-23; fold-in refresh)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `ff61399` (`track-t95-in-force`). T291 plan `e81a1a2`. `origin/main` = `37012fe` T290 `#206`. CLEAN. T95 did not touch `run_trace`. |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T274. No T285–T290.** Hole is in **source**. **Do not `cargo install`.** |
| `query trace missing-id` | stdout token `null`; exit **0** |
| `query trace missing-id --format human` | clap unexpected `--format` (tip: `--log-format`) |
| `query progressive` `--dry-run` | default **true** — no persist |
| `preflight --summary` | Pinned **4019** (volatile); in-context **0/0/0**; word **689** |
| Last PR comments | #206 T290 — Cursor Bugbot **1 Low** sanitizer collapse. **Absorb F16.** **No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; GitHub **v4.6.6**; **no clap 5**); serde_json **1.0.150** (crates.io 1.0.151); chrono **0.4.44**; rusqlite **0.39.0** (0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** (3.941) — do not touch. `governed_common.rs` **#2** (3.806) — sanitizer collapse only. `governed_query.rs` not top-10 — envelope here. CLI `preflight.rs` **#8** — do not grow. |
| Ledger | 0 pending / 0 drift at scan; planning TX `c59e5bb6` |
| `ISSUES.md` | **Does not exist** (F22) |
| ledgerful search | `run_trace` → `governed_query.rs:205`; `get_query_trace` → `query.rs:413` |
| Online | clig.dev human-first + next-command; T180 type-change documented; clap 4.6.6 / no clap 5 |
| Skill | CAPABILITIES Trace row (F14) |

---

## Phase 0 — on go (re-verify)

- [x] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [x] Re-read `governed_query.rs` `run_trace` `:205` None arm `println!("null")`
- [x] Confirm `QueryTraceDto` still has **no** `found` / `next_step` — **do not add**
- [x] Confirm progressive `--dry-run` default still **true** — **do not flip** (F9)
- [x] Confirm no `--trace` on progressive
- [x] Confirm `get_query_trace` still `Ok(None)` for miss / cross-principal / no grant — **do not change CP**
- [x] Confirm `sanitize_recall_query` still sets `prev_space = false` on `$`/backtick — F16 still true
- [x] Confirm hermetic `query_trace__unknown__stdout_null_exit_0` still exact `"null"`
- [x] Confirm `OutputFormat::parse` still case-insensitive unknown→Json — AC7 **must** use clap `value_parser` (F3)
- [x] Confirm `progressive_cmd` still hardcodes `"x"` + default dry-run — AC5 must not reuse it
- [x] Re-scan hotspots — `governed_common.rs` still sanitizer-only
- [x] Rescan `conductor/deferred.md` — T291 absorbed + #206; T292–T300 / T290 not stolen
- [x] Confirm #206 Bugbot still the only Cursor finding; no mint; Dependabot `#61` still not this track
- [x] Re-dogfood `query trace missing-id` **read-only**. **Did not** pin production decisions; **did not** write `.env`; **did not** extra `policy bootstrap`
- [x] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [x] FEATURE TX (new)
- [x] Did **not** `cargo install`; did **not** grow `project.rs` / `preflight.rs` / `briefing.rs` / `personal.rs` / CP `persist_trace`

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

- [x] `trace_missing_next_step__frozen__exact_string` (AC1) — F8 const exact
- [x] `query_trace__unknown__stdout_envelope_exit_0` (AC2) — fail while stdout is `null`
- [x] `sanitize_recall_query` rstest `a $ b` → `a b` (AC8) — fail on current helper
- [x] Commit red allowed

## Phase 2 — green missing path

- [x] `TRACE_MISSING_NEXT_STEP` + missing envelope struct in `governed_query.rs`
- [x] `run_trace` None arm: json → `emit_json` envelope; human → two lines (F2)
- [x] clap `--format` on `Trace` with T266 `value_parser` (not `OutputFormat::parse`); dispatch; after_help `:1589` / `:1892` / `:1924`
- [x] `sanitize_trace_id` (F15); share collapse with F16
- [x] Fix `sanitize_recall_query` interpolator-as-whitespace + final trim (F16)
- [x] AC3 human hermetic; AC4 no-project; AC7 InvalidValue `JSON` via clap `value_parser`
- [x] AC9 unit id sanitize

## Phase 3 — found + stay-green

- [x] AC5 hermetic: System bootstrap omit `--principal-id` + `query progressive "what did we decide" --dry-run false` (not `progressive_cmd`) then `query trace <id>`
- [x] Found `--format human` still QueryTraceDto (F10)
- [x] AC6 T290/T221 stay green; ellipsis const exact
- [x] AC12 `QueryTraceDto` no new fields

## Phase 4 — docs + gate

- [x] CAPABILITIES Trace row; OPERATIONS; CLI-EXIT-CODES; PROTOCOL-COMPAT §5 + §3.1; clap after_help; CHANGELOG
- [x] `cargo fmt --check` ; `cargo clippy --workspace --all-targets -- -D warnings`
- [x] Targeted nextest then workspace gate (`dev-check` / nextest + deny + audit)
- [x] Manual AC11 `cargo run -p ai-brains-cli -- query trace missing-id` and `--format human`
- [x] `ledgerful verify --scope full`

## Phase 5 — review + publish

- [x] `conductor/tracks/trackT291-query-trace-next/review.md` phase-1
- [x] Codex/cross-model when FEATURE
- [x] Mark conductor **Completed**; append closeout residuals to `deferred.md`
- [x] Push `track/T291-*` ; PR ; `gh run watch --exit-status` ; `gh pr merge --squash --delete-branch`
- [x] Fetch prune; point local `main` at `origin/main`; delete merged local `track/T291-*` only
- [x] Never `git push origin main`. Never force-push.

---

## DoD (checkable)

- [x] Default `query trace missing-id` is **not** the token `null`; JSON `found: false` + F8 `next_step`
- [x] `--format human` has `No trace` and `next:` and `--dry-run false`
- [x] Exit **0**; no project-id required
- [x] Found DTO unchanged; no invented traces
- [x] #206 `a $ b` collapses to `a b`
- [x] Docs no longer say “JSON token null (not an object)”
- [x] Status stays **Pending** until go; this file’s implement boxes stay unchecked through planning
