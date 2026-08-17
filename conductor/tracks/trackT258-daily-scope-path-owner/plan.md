# T258 Plan — Daily Scope = path owner

**Status:** **Completed** 2026-08-16
**Spec:** [spec.md](./spec.md) F0–F26 / AC1–AC16 + §13 fold-in
**Category:** FEATURE / UX / OPS
**Ledger TX (planning):** `f7b86f91-b914-4a93-b951-217c14157e6c` (DOCS)
**Ledger TX (fold-in):** `f38d51f7-fb9e-4c2b-85cb-379cd76b74a8` (DOCS)
**Ledger TX (implement):** `6962a7b8-ff3a-4c0b-90cb-b3167d993335` (FEATURE)

---

## AI fold-in (2026-08-16) — `opencode-review.md`

No Blockers. One Major (hermetic `--format auto` → JSON). Disposition in spec **§13**.

### Pins locked by fold-in

1. **F26 / AC1–AC6 / AC15:** force `--format human` in hermetic human-chrome fixtures.
2. **AC5 / §5.1:** already-bound human SOOT (`Already bound to path owner` / `No .env write.`).
3. **AC16:** `--no-project-context` uses the **file** PROJECT_ID for already-bound (F7).
4. **F10:** remediations drop `project list`.
5. **§2.4:** `project.rs` is **1547** lines.

---

## Preflight (plan time — 2026-08-16)

| Check | Result |
|-------|--------|
| HEAD / tree | `e055e29` T256 `#170`. CLEAN. `main` = `origin/main`. |
| T258 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe`. Whoami mismatch **true**. |
| Live triangle | env/effective `441837f6` test-alias **592** mem; path/detect `3581317d` **2,700**; shell `7d97a456` **18,028** / `C:\dev\crawlx`. |
| `context.rs` | Existing `.env` writer. Hash UUID tail `000000000000` matches `441837f6`. Not the remediator. |
| `ProjectCommands` | No AdoptPath / Use (`main.rs:2066`). |
| Whoami remediations | Hand-edit + `project list`. No product verb. |
| clap / dotenvy | lock clap **4.6.1** / builder **4.6.0**; dotenvy **0.15.7** read-only (docs.rs). crates.io clap **4.6.6**. **No clap 5.** Snapshot — re-verify at execute. |
| rustc / workspace | 1.95.0 / **0.1.1** |
| Last PR Cursor | #170 empty comments/reviews; HEAD `main`; Dependabot only. **N/A.** |
| `deferred.md` | Full scan. Overlap: audit identity (absorb); T240 F14 path-slice (partial); T240 runbook (absorb); T267 F2 remediations (partial); T257/T259 (point). |
| ai-brains | `preflight --summary` ok (wrong Scope — this track). Recall: T254 path-alias / T240 visibility. No adopt-path pin. |
| ledgerful | doctor ready (hygiene warns). 0 pending at start. Hotspot **#1** `project.rs` — new file `project_adopt.rs`. |
| `ISSUES.md` | **Does not exist** |
| Live `.env` | **Not written** this pass. |

---

## Phase 0 — on go (re-verify)

- [x] Re-read `ProjectCommands` in `main.rs`. Confirm still no `AdoptPath`.
- [x] Re-run `project whoami --format json` (classify IDs only). Confirm mismatch still env ≠ path.
- [x] Re-check lock clap + dotenvy docs.rs: still no write API; still no clap 5 (or this track is not that bump).
- [x] Rescan **entire** `conductor/deferred.md` for new open identity / `.env`-write rows.
- [x] Last merged PR + open HEAD PR Cursor comments. Mint placeholder if a leftover fits nowhere.
- [x] `ledgerful ledger start T258-daily-scope-path-owner --category FEATURE`
- [x] Do **not** `cargo install`, write live `.env`, alias `7d97a456`, or merge projects.

---

## Phase 1 — Red

- [x] Add `crates/ai-brains-cli/tests/project_adopt_path.rs` (hermetic tempdir + `register_path` helpers from T240 tests).
- [x] `project_adopt_path__print_only__names_owner_no_write` (**must red** — unknown subcommand). Fixture uses `--format human` (F26).
- [x] `project_adopt_path__write_env_without_yes__exit_2_no_write` (**must red**; `--format human`)
- [x] `project_adopt_path__write_env_yes__rewrites_only_project_id` (**must red**; `--format human`)
- [x] `project_adopt_path__missing_env__write_creates_project_id_only` (**must red**; `--format human`)
- [x] `cargo nextest run -p ai-brains-cli --test project_adopt_path` **fails** because `adopt-path` does not exist. Do not chase a green by asserting JSON on `auto` or by weakening ACs.

---

## Phase 2 — Green (clap + print-only)

- [x] `commands/project_adopt.rs` + `mod.rs`
- [x] `ProjectCommands::AdoptPath` `{ write_env, yes, format }` with `--yes` `requires = "write_env"`
- [x] Dispatch in `main.rs`
- [x] Print-only path using `resolve_path_alias_for_location` (reuse T240)
- [x] Human chrome + already-bound SOOT (§5.1)
- [x] AC1 / AC2 / AC12 / AC13 green
- [x] AC3 / AC4 still red until Phase 3

---

## Phase 3 — Green (write)

- [x] Pure `rewrite_project_id_line` + unit tests (preserve KEY/SESSION)
- [x] `refuse_if_reparse` before `fs::write`
- [x] `--write-env --yes` AC3 / AC4 / AC5 / AC6 / AC14 / **AC16** (`--no-project-context` file already-bound)
- [x] No events; no `context::run`; no session rotate
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`

---

## Phase 4 — Whoami remediations

- [x] `build_whoami_report` mismatch bullets: F10 / AC7
- [x] Do **not** change `identity_mismatch_warn_line` (AC8)
- [x] `cargo nextest run -p ai-brains-cli --test project_identity_convergence` green
- [x] adopt-path + remediations tests green

---

## Phase 5 — Docs

- [x] `Docs/CAPABILITIES.md` adopt-path row; whoami remediations honesty
- [x] `Docs/WORKFLOWS.md` §0: `project adopt-path` / `--write-env --yes`; drop `7d97a456` + `AI-Brains` example
- [x] Root `CHANGELOG.md` T258 row
- [x] No PROTOCOL-COMPAT / contracts

---

## Phase 6 — Review + gate

- [x] Phase-1 review → `conductor/tracks/trackT258-daily-scope-path-owner/review.md`
- [x] Medium+ not silently dropped
- [x] FEATURE `codex-review` → `review.codex.md`
- [x] Manual AC15: source-bin print-only in this repo names `3581317d-…`; live `.env` hash unchanged
- [x] Full gate: `cargo fmt --check` ; clippy workspace `-D warnings` ; `cargo nextest run --workspace` ; `cargo deny check` ; `cargo audit` ; `ledgerful verify --scope full`

---

## Phase 7 — Close

- [x] conductor T258 → **Completed** with evidence
- [x] `deferred.md`: strike the daily-Scope row; keep F14 remainder + T259/T267 pointers
- [x] FEATURE TX commit
- [x] Optional pin: `DECISION: T258 project adopt-path is the path-owner remediator; default print-only; --write-env --yes touches only AI_BRAINS_PROJECT_ID; T240 F2 stands`
- [x] PR only if owner asks (no push to `main` without owner)

---

## DoD (checkable)

- [x] AC1–AC16 evidenced
- [x] Hermetic human-chrome tests pass `--format human` (F26)
- [x] Live repo `.env` not written by this track unless owner asked
- [x] `context.rs` behavior untouched
- [x] T240 warn SOOT untouched
- [x] No clap 5 / new crate / pin bump
- [x] T240 identity suite still green
- [x] T257 / T259 / leftover alias not stolen
