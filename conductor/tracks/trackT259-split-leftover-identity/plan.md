# T259 Plan — Split leftover identity

**Status:** **Completed** (implement-track 2026-08-17)
**Spec:** [spec.md](./spec.md) F0–F26 / AC1–AC18 + §13 fold-in
**Category:** FEATURE / UX / OPS
**Ledger TX (planning):** `49463c65-1759-4110-b1f3-14beda6dfe58` (DOCS)
**Ledger TX (fold-in):** `79be45ed-5222-465b-90d0-ae999ae51d72` (DOCS)
**Ledger TX (implement):** start FEATURE on **go** only

---

## AI fold-in (2026-08-17) — `opencode-review.md` + `agy-review.md`

No Blockers. One Major (unreachable JSON null). Disposition in spec **§13**.

### Pins locked by fold-in

1. **§5.1 / AC10:** `from_project_id` is `"<uuid>"` — no-owner is stderr+exit 1, never JSON.
2. **F6 / AC18:** CP `rebind_path_alias(from, from)` → `InvalidPayload`.
3. **AC16 / F25:** empty filter `No path aliases match.` / `paths: []` / exit 0.
4. **AC17 / F2:** `--project` + `--shared-only` intersection.
5. **F12:** `resolve_project_ref` is `pub(crate)`; do not duplicate.
6. **F23:** Phase 0 reads **source** remediations (`project.rs:823`); PATH-behind is the installed binary.
7. **§2.1:** leftover 18,028 / 11 roots load-bearing; other totals re-counted at Phase 0.

---

## Preflight (plan time — 2026-08-17)

| Check | Result |
|-------|--------|
| HEAD / tree | `049064d` T258 `#171`. CLEAN. `main` = `origin/main`. |
| T259 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` (2026-08-16 08:04). PATH binary remediations pre-T258. **Source** already names adopt-path (`project.rs:823`). Whoami mismatch **true**; shell leftover `7d97a456`. |
| Live leftover | `list-paths` **17** roots / **7** owners. **`7d97a456` owns 11**. Memory **18,028**. Fold-in snapshot: `3581317d` **2,753**; `441837f6` **595**; global pinned **35,561**. Footer `set-alias 7d97… AI-Brains`. Dest projects for leftover folders: **none**. |
| `ProjectCommands` | No `RebindPath`. `ListPaths` format-only (`main.rs:2141`). |
| T254 primitives | `list-paths` / `unregister-path` / `register-path` live. `register_path` uses `process::exit(1)` — rebind must not call it. |
| clap / dotenvy | lock clap **4.6.1** / builder **4.6.0**; crates.io clap **4.6.6**; docs.rs `Arg::requires` current. dotenvy **0.15.7** N/A (no `.env` write). **No clap 5.** Snapshot — re-verify at execute. |
| rustc / workspace | 1.95.0 / **0.1.1** |
| Last PR Cursor | #171 empty comments/reviews/inline; HEAD `main`; Dependabot only. **N/A.** |
| `deferred.md` | Full scan. Overlap: audit leftover (**absorb**); T267 footer (**decline** / F1 partial); T260/T264 (**decline**); T258 `.env` (**decline**); T255 declines stay closed. |
| ai-brains | `preflight --summary` ok (wrong Scope — T258 live rebind out of band). Recall: T254 path-alias decisions. No rebind-path pin. |
| ledgerful | doctor ready (hygiene warns). 0 pending at start. Hotspot **#1** `project.rs` 1549 — new file `project_rebind.rs`. `append_events` is one SQLite tx. |
| `ISSUES.md` | **Does not exist** |
| Live leftover paths | **Not unregistered** this pass. Live `.env` **not written**. |

---

## Phase 0 — on go (re-verify)

- [x] Re-read `ProjectCommands` in `main.rs`. Confirm still no `RebindPath`; `ListPaths` still format-only (or note drift).
- [x] Re-read **source** `project.rs` remediations (~823): adopt-path is present in tree even if PATH binary is old.
- [x] Re-run `project list-paths --format json` (classify IDs + path counts only). Confirm leftover still multi-root; `C:\dev\ai-brains` still `3581317d`. Re-count vault totals (non-load-bearing).
- [x] Re-check lock clap + crates.io: still no clap 5 (or this track is not that bump).
- [x] Rescan **entire** `conductor/deferred.md` for new open leftover / path-alias / footer rows.
- [x] Last merged PR + open HEAD PR Cursor comments. Mint placeholder if a leftover fits nowhere.
- [x] `ledgerful ledger start T259-split-leftover-identity --category FEATURE`
- [x] Do **not** `cargo install`, write live `.env`, `set-alias 7d97a456 AI-Brains`, or `--write --yes` against live leftover roots.

---

## Phase 1 — Red

- [x] Add `crates/ai-brains-cli/tests/project_rebind_path.rs` (hermetic tempdir + register-path helpers from T254 tests).
- [x] `project_rebind_path__print_only__names_from_to_no_events` (**must red** — unknown subcommand). Fixture uses `--format human` (F14).
- [x] `project_rebind_path__write_without_yes__exit_2_no_events` (**must red**; `--format human`)
- [x] `project_rebind_path__write_yes__rebinds_owner_memories_stay` (**must red**; `--format human`)
- [x] `cargo nextest run -p ai-brains-cli --test project_rebind_path` **fails** because `rebind-path` does not exist. Do not chase a green by asserting JSON on `auto` or by weakening ACs.

---

## Phase 2 — Green (list-paths filters)

- [x] `ListPaths { format, project: Option<String>, shared_only: bool }`
- [x] Filter in `project_paths.rs` after join; unfiltered JSON keys **unchanged** (T254 F10)
- [x] `--shared-only` = owner count ≥ 2
- [x] `--project` uses `pub(crate) resolve_project_ref`; unknown → exit **1**
- [x] Empty filter: `No path aliases match.` exit **0**; JSON `paths: []` (**AC16**)
- [x] `--project` + `--shared-only` intersection (**AC17**)
- [x] AC1 / AC2 / AC16 / AC17 green
- [x] T254 `project_path_aliases` stays green

---

## Phase 3 — Green (clap + print-only rebind)

- [x] `commands/project_rebind.rs` + `mod.rs`
- [x] `ProjectCommands::RebindPath { path, to, write, yes, format }` with `--yes` `requires = "write"` and `--to` required
- [x] Dispatch in `main.rs`
- [x] Print-only: resolve owner + dest via `project_paths::resolve_project_ref`; no `append_events`
- [x] Human chrome + already-bound SOOT (§5.1)
- [x] AC3 / AC4 / AC6 / AC7 / AC8 / AC9 / AC10 green
- [x] AC5 still red until Phase 4
- [x] New help/error strings fail AC15 if they recommend leftover `set-alias` + `AI-Brains`

---

## Phase 4 — Green (write)

- [x] CP `rebind_path_alias` → `append_events(&[Removed, Added])` in one tx
- [x] `from == to` → `InvalidPayload` before append (**AC18**)
- [x] Export from `ai-brains-control-plane` lib
- [x] CLI `--write --yes` calls helper only after prechecks (dest exists, owner == from, from != to)
- [x] Do **not** call `project::register_path`
- [x] AC5 green: owner is dest; from `memory_count` unchanged; +2 events
- [x] CP units `rebind_path_alias__appends_removed_then_added` + `rebind_path_alias__from_eq_to__invalid_payload`

---

## Phase 5 — Docs

- [x] CAPABILITIES: list-paths filters + rebind-path row; CONTEXT inventory includes `rebind-path`
- [x] WORKFLOWS leftover runbook (print-only then `--write --yes`; dest via `context`; never `7d97`+`AI-Brains`)
- [x] OPERATIONS: rebind ≠ move memories; pair with unregister honesty
- [x] CLI-EXIT-CODES: rebind 0/1/2; filtered list-paths empty = 0
- [x] CHANGELOG T259
- [x] AC13 / AC15

---

## Phase 6 — Review + close (implement-track, not this pass)

- [x] Internal review vs spec until clean (mediums fixed or justified)
- [x] FEATURE `codex-review` after Phase-1 clean
- [x] Manual AC14 source bin print-only; live leftover row count unchanged
- [x] Targeted clippy/nextest; then full gate on finalize
- [x] conductor **Completed** only after implement-track publish loop
- [x] Soft residuals → `deferred.md` (not ISSUES.md)
- [x] `ai-brains pin` leftover/rebind decisions

---

## Definition of done (checkable)

- [x] AC1–AC18 met with evidence (hermetic names in spec §4)
- [x] F0–F26 honored (especially F5 memories stay, F6 one tx, F11 no `.env`, F16 no live leftover mutate)
- [x] `project.rs` / `context.rs` untouched
- [x] T254 / T240 / T258 suites green
- [ ] No clap 5 / no new crate / no contracts DTO
- [ ] T267 footer still T267 (algorithm unchanged)
- [ ] Medium+ review findings not silently dropped

---

## Stop-before (implement)

- Live `unregister-path` / `rebind-path --write --yes` on leftover roots
- `set-alias 7d97a456 AI-Brains`
- Writing `C:\dev\AI-Brains\.env`
- `cargo install`
- Push to `main` / force-push
- Reopening T240 F2 / T255 declines
