# T283 Plan — `project list` cwd-first (human only)

**Status:** **Planned** (Pending until **go**)
**Spec:** [spec.md](./spec.md) F0–F36 / AC1–AC14
**Category:** UX / HONESTY
**Ledger TX (planning):** `0535063a-dd76-454e-8c1b-bae350a5d7bd` (DOCS)
**Ledger TX (implement):** FEATURE on **go**

---

## Preflight (plan time — 2026-08-22)

| Check | Result |
|-------|--------|
| HEAD / tree | **Plan dogfood:** `6d3cbc5` T282 `#198`. CLEAN. `origin/main` = HEAD (`0 0`) after fetch |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-22 14:49, 25 443 840 bytes. Pre-T282 install. List hole is in **source**. **Do not `cargo install`.** |
| `preflight --summary` | Pinned **3633** (volatile); in-context 5/0/0; grants **0 of 3**; Scope `3581317d` |
| PATH `project list` | First data row leftover `7d97a456` **18043** `C:\dev\crawlx`. Cwd `*C:\dev\ai-brains` `3581317d` **fourth** (3633). Footer `set-alias 33ec90e0 … my-project` (T267 already honest) |
| PATH `project whoami` | JSON `shell_project_id=7d97a456-…`; `mismatch: false`; remediations `[]` |
| Last PR comments | #198 T282 — **empty** (N/A). #188 closed by T284. No T285 |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); rusqlite **0.39.0** (0.40.2); uuid lock **1.23.1** (1.25.0); tokio **1.52.3** (1.53.1) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** (1472 lines) — do not grow helpers. New sibling `project_list_order.rs` |
| Ledger | 0 pending / 0 drift at scan; planning TX `0535063a` |
| `ISSUES.md` | **Does not exist** (F23) |
| ledgerful search | `list_projects_detail` `query_store.rs:584` + `project.rs:27` |
| Online | clig.dev human-first + JSON-stable; kubectl contexts mark `*` not current-first; clap 4.6.6 no clap 5 |
| Skill | `.agents` no `project list` match (no-op). `.claude` **`:89`** table — F19 one-liner on go |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [ ] Re-read `project.rs` `:26–102` list + JSON
- [ ] Re-read `query_store.rs` `:567` / `:611` — **do not edit ORDER BY**
- [ ] Re-read `project_list_footer.rs` `:21–41` — **do not edit**; pass original vec
- [ ] Re-read clap List `main.rs` `:2636–2643` — **no new flags**
- [ ] Re-read `resolve_path_alias_for_location` `project.rs` `:226–237` — **call existing**
- [ ] Confirm T212 JSON tests find by id not `[0]`
- [ ] Rescan `conductor/deferred.md` — T283 rows absorbed; no new mint
- [ ] Confirm #198 comments/reviews still empty (N/A); no mint; Dependabot `#61` still not this track
- [ ] Re-dogfood `project list` + `project whoami` **read-only**. **Did not** write `.env`
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [ ] FEATURE TX
- [ ] Did **not** `cargo install`; did **not** grow `query_store.rs` / `project_list_footer.rs` / `sync.rs` / `forget.rs`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit leftover-first human table | **DoD** F1 / AC1–AC6 / AC10 |
| Placeholder JSON freeze vs human-only | **DoD** F2 JSON freeze; F1 human permute |
| T276 / T282 list leftover-first pointer | This track |

## Declined (written)

| Item | Why |
|------|-----|
| T267 footer restyle | F3 / AC7 |
| T240 F2 write | F4 |
| JSON reorder / `cwd_first` key | F2 / F32 |
| `--sort` / star-as-sort | F5 / F10 |
| Store `ORDER BY` | F11 / F36 |
| last-PR #198 Cursor | N/A empty |
| Dependabot rusqlite `#61` | F12 — no T285 |
| T276 live 11-root rebind | F16 / F17 |

---

## Phase 1 — Red (TDD)

- [ ] `promote_cwd_owner__middle_id__becomes_first` — AC1
- [ ] rstest None/empty/missing — AC2
- [ ] Commit red allowed

---

## Phase 2 — Green

- [ ] `project_list_order.rs` + `mod.rs`
- [ ] `list()` human loop uses `promote_cwd_owner`; JSON + `print_unaliased_footer` keep original
- [ ] Hermetic `tests/project_list_cwd_first.rs` AC3–AC6 (`hermetic_bin` + **must** `isolate_empty_home`; leftover more pins; cwd `register-path`; `.current_dir`)
- [ ] AC7–AC9 / AC13 stay green
- [ ] Commit green

---

## Phase 3 — Docs

- [ ] CAPABILITIES List + List JSON: cwd-first human; JSON size-desc
- [ ] OPERATIONS Listing Projects: T212 columns + cwd-first (replace stale T76)
- [ ] PROTOCOL-COMPAT: no new required keys
- [ ] CLI-EXIT-CODES: list still exit 0
- [ ] Root CHANGELOG T283
- [ ] List `after_help` F35 one sentence
- [ ] `.claude/skills/ai-brains/SKILL.md` `:89` one sentence (F19). **Skip** `.agents/skills/ai-brains/SKILL.md` (no `project list` match)

---

## Phase 4 — Gate + publish (on go)

- [ ] Classify-only AC10 (`cargo run -p ai-brains-cli -- project list` and `--format json`). **No** `.env` write
- [ ] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [ ] Review log `review.md`; FEATURE cross-model (F22)
- [ ] `scripts/dev-check.ps1` (not repo-root `dev-check.ps1`)
- [ ] implement-track Phase 6: push `track/T283-*` → PR → watch GHA `CI` green → squash-merge → prune. Never `git push origin main`

## DoD

- [ ] Human first data row is cwd path-owner when registered (F1)
- [ ] JSON array still memory-desc (F2)
- [ ] Footer still T267 (F3)
- [ ] No T240 F2 write (F4)
- [ ] Conductor **Completed** only after merge + hygiene
