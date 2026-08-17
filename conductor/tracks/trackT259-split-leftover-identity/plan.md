# T259 Plan — Split leftover identity

**Status:** **Pending** (requirements **Planned**; not In Progress)
**Spec:** [spec.md](./spec.md) F0–F26 / AC1–AC15
**Category:** FEATURE / UX / OPS
**Ledger TX (planning):** `49463c65-1759-4110-b1f3-14beda6dfe58` (DOCS)
**Ledger TX (implement):** start FEATURE on **go** only

---

## Preflight (plan time — 2026-08-17)

| Check | Result |
|-------|--------|
| HEAD / tree | `049064d` T258 `#171`. CLEAN. `main` = `origin/main`. |
| T259 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` (2026-08-16 08:04). PATH-behind T258. Whoami mismatch **true**; shell leftover `7d97a456`. |
| Live leftover | `list-paths` **17** roots / **7** owners. **`7d97a456` owns 11** (`crawlx` … `wondermaker`). Memory **18,028**. Global pinned **35,520**. Footer `set-alias 7d97… AI-Brains`. Dest projects for those folders: **none**. |
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

- [ ] Re-read `ProjectCommands` in `main.rs`. Confirm still no `RebindPath`; `ListPaths` still format-only (or note drift).
- [ ] Re-run `project list-paths --format json` (classify IDs + path counts only). Confirm leftover still multi-root; `C:\dev\ai-brains` still `3581317d`.
- [ ] Re-check lock clap + crates.io: still no clap 5 (or this track is not that bump).
- [ ] Rescan **entire** `conductor/deferred.md` for new open leftover / path-alias / footer rows.
- [ ] Last merged PR + open HEAD PR Cursor comments. Mint placeholder if a leftover fits nowhere.
- [ ] `ledgerful ledger start T259-split-leftover-identity --category FEATURE`
- [ ] Do **not** `cargo install`, write live `.env`, `set-alias 7d97a456 AI-Brains`, or `--write --yes` against live leftover roots.

---

## Phase 1 — Red

- [ ] Add `crates/ai-brains-cli/tests/project_rebind_path.rs` (hermetic tempdir + register-path helpers from T254 tests).
- [ ] `project_rebind_path__print_only__names_from_to_no_events` (**must red** — unknown subcommand). Fixture uses `--format human` (F14).
- [ ] `project_rebind_path__write_without_yes__exit_2_no_events` (**must red**; `--format human`)
- [ ] `project_rebind_path__write_yes__rebinds_owner_memories_stay` (**must red**; `--format human`)
- [ ] `cargo nextest run -p ai-brains-cli --test project_rebind_path` **fails** because `rebind-path` does not exist. Do not chase a green by asserting JSON on `auto` or by weakening ACs.

---

## Phase 2 — Green (list-paths filters)

- [ ] `ListPaths { format, project: Option<String>, shared_only: bool }`
- [ ] Filter in `project_paths.rs` after join; unfiltered JSON keys **unchanged** (T254 F10)
- [ ] `--shared-only` = owner count ≥ 2
- [ ] `--project` uses `pub(crate) resolve_project_ref`; unknown → exit **1**
- [ ] Empty filter: `No path aliases match.` exit **0**; JSON `paths: []`
- [ ] AC1 / AC2 / AC (empty filter) green
- [ ] T254 `project_path_aliases` stays green

---

## Phase 3 — Green (clap + print-only rebind)

- [ ] `commands/project_rebind.rs` + `mod.rs`
- [ ] `ProjectCommands::RebindPath { path, to, write, yes, format }` with `--yes` `requires = "write"` and `--to` required
- [ ] Dispatch in `main.rs`
- [ ] Print-only: resolve owner + dest; no `append_events`
- [ ] Human chrome + already-bound SOOT (§5.1)
- [ ] AC3 / AC4 / AC6 / AC7 / AC8 / AC9 / AC10 green
- [ ] AC5 still red until Phase 4
- [ ] New help/error strings fail AC15 if they recommend leftover `set-alias` + `AI-Brains`

---

## Phase 4 — Green (write)

- [ ] CP `rebind_path_alias` → `append_events(&[Removed, Added])` in one tx
- [ ] Export from `ai-brains-control-plane` lib
- [ ] CLI `--write --yes` calls helper only after prechecks (dest exists, owner == from, from != to)
- [ ] Do **not** call `project::register_path`
- [ ] AC5 green: owner is dest; from `memory_count` unchanged; +2 events
- [ ] CP unit `rebind_path_alias__appends_removed_then_added`

---

## Phase 5 — Docs

- [ ] CAPABILITIES: list-paths filters + rebind-path row; CONTEXT inventory includes `rebind-path`
- [ ] WORKFLOWS leftover runbook (print-only then `--write --yes`; dest via `context`; never `7d97`+`AI-Brains`)
- [ ] OPERATIONS: rebind ≠ move memories; pair with unregister honesty
- [ ] CLI-EXIT-CODES: rebind 0/1/2; filtered list-paths empty = 0
- [ ] CHANGELOG T259
- [ ] AC13 / AC15

---

## Phase 6 — Review + close (implement-track, not this pass)

- [ ] Internal review vs spec until clean (mediums fixed or justified)
- [ ] FEATURE `codex-review` after Phase-1 clean
- [ ] Manual AC14 source bin print-only; live leftover row count unchanged
- [ ] Targeted clippy/nextest; then full gate on finalize
- [ ] conductor **Completed** only after implement-track publish loop
- [ ] Soft residuals → `deferred.md` (not ISSUES.md)
- [ ] `ai-brains pin` leftover/rebind decisions

---

## Definition of done (checkable)

- [ ] AC1–AC15 met with evidence (hermetic names in spec §4)
- [ ] F0–F26 honored (especially F5 memories stay, F6 one tx, F11 no `.env`, F16 no live leftover mutate)
- [ ] `project.rs` / `context.rs` untouched
- [ ] T254 / T240 / T258 suites green
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
