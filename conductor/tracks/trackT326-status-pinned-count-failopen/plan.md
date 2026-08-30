# T326 Plan — status/graph PinnedCountFailed fail-open + workspace 0.1.4

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** BUGFIX / UX / CHORE
**Ledger (planning):** DOCS `5fd70b52-1a16-4971-ab0f-684c553a4c17`
**Ledger (mint):** T316 DOCS `66b597f7-faf9-4f3e-bb06-6af72811bdc6`

---

## Preflight (plan time — 2026-08-29)

| Check | Result |
|-------|--------|
| HEAD / tree | Product `9119c74` T325 `#247`. Dirty conductor T325 Completed notes → this plan DOCS. Branch `track/T326-status-pinned-count-failopen`. `origin/main` = `9119c74` (ahead **0** at plan start). |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T312–T325 **not** on PATH. `status` unrecognized. |
| `preflight --summary` (PATH) | Pinned **4674**; in-context **0/0/0**; `Total Word Count: 717` (PATH-behind T315). |
| Bugbot `#237` `3885361601` | **Still true** — `status.rs:329–340` invents `pinned=0` + assesses |
| graph.rs same arm | `:445–458` fake 0 |
| Doctor skip | `:901–904` **SOOT** |
| Assessor skip arm | `graph_density.rs:260–270` empty graph + `pinned<100` → `live`/`skip` |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; serde_json **1.0.150**; workspace **0.1.3** → **0.1.4 this track** |
| Last PR Cursor | `#247` empty. `#237` → **this**. **No T327.** |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX) |
| Impact | **LOW** (conductor-only dirty) |
| Hotspots | CLI `project.rs` #1 — do not touch. |
| Line counts (physical) | `status.rs` **808**; `graph.rs` **1731**; `graph_density.rs` **733**. |
| `ISSUES.md` | **Does not exist** |
| Planning install / live mutate | **Not run** |
| git tags `v0.*` | **None** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| `#237` Bugbot fake `pinned=0` | **DoD** F1 / AC1 |
| `graph.rs` same fake 0 | **DoD** F2 / AC2 |
| T304 R6 Docs 0.1.2 banners | **DoD** F25 / AC11 |
| Owner version bump | **DoD** F23–F28 / AC9–AC11 |
| T320 F4 envelope | **Reuse** |
| T325 residuals / Completed note | **Not stolen** / absorb dirty conductor |
| T307 / clap 5 / H2 / floors | **Not stolen** / **Decline** |
| last-PR `#247` | **N/A empty** |
| Desktop 0.1.2 / git tag | **Decline** F26/F28 |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains`
- [ ] Re-read `status.rs:329–340` vs doctor `:901–904` vs `graph.rs:445–458`
- [ ] Re-read assessor skip arm `graph_density.rs:260–270`
- [ ] Confirm Bugbot `#237` still true (fake 0 + assess)
- [ ] Confirm workspace still **0.1.3**; rusqlite **0.40.2**; clap lock **4.6.1**
- [ ] Confirm T307 still Blocked; T325 Completed; no new Cursor leftover
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] `ledgerful ledger start T326-status-pinned-count-failopen --category BUGFIX`
- [ ] **Do not** `cargo install` / live table drop / `.env` rewrite / retune floors / edit `doctor.rs` / git tag

## Phase 1 — Red

- [ ] `graph_section_from_gather__pinned_count_failed__error_not_fake_zero` (AC1) — **must fail** (today Ok `live`/`pinned=0` on empty-graph COUNT fail)
- [ ] Second case `nodes=100, edges=10` still would be Ok sparse with `pinned=0` today
- [ ] `graph_health_from_gather__pinned_count_failed__err` (AC2, `--features graph`) — **must fail**
- [ ] Confirm AC3/AC4/AC5/AC6 still **pass** (stay-green)

## Phase 2 — Green (fail-open)

- [ ] `PINNED_COUNT_FAILED_MSG` const on `graph_density.rs` (F4)
- [ ] `graph_section_from_gather` — `PinnedCountFailed → Err` (F1/F8)
- [ ] `graph_health_from_gather` — `PinnedCountFailed → Err` (F2/F8/F35)
- [ ] AC1/AC2/AC7/AC8 pass
- [ ] AC3–AC6 / AC12–AC14 / AC16 stay-green
- [ ] `git diff` `doctor.rs` empty (AC5/AC13)

## Phase 3 — Version 0.1.4

- [ ] `Cargo.toml` `workspace.package.version = "0.1.4"` (F23)
- [ ] Cargo rewrite of lock workspace package versions (**no** `cargo update` of third-party crates)
- [ ] CHANGELOG: T326 Fixed bullet + insert `## [0.1.4] — <date>` after Unreleased (F24 / AC10)
- [ ] Docs F25 headers **0.1.4** (CAPABILITIES / INSTALL / README / RELEASE-CHECKLIST currently-line / RELEASE-CLAIMS header / SECURITY-LIMITS / ci-tooling example / generate-sbom comment)
- [ ] CAPABILITIES F36 COUNT-fail honesty (AC11)
- [ ] AC9 source `--version` / `CARGO_PKG_VERSION` is **0.1.4**
- [ ] `check-version-banners.ps1` sees `## [0.1.4]`
- [ ] Desktop **0.1.2** untouched (F26)
- [ ] Path-dep `"0.1.0"` untouched (F27)

## Phase 4 — Docs / isolation

- [ ] PROTOCOL-COMPAT: no new required keys
- [ ] AC14 no new clap flags
- [ ] AC15 PATH honesty (no install)
- [ ] Conductor / deferred Completed notes **only after** implement-track publish

## DoD (checkable)

- [ ] AC1 red-then-green: empty-graph COUNT fail is glance `error`, not `live`/`pinned=0`
- [ ] AC2: `graph update` COUNT fail is `Err`, not `pinned_memories: 0`
- [ ] Doctor skip + 15-check matrix untouched
- [ ] Floors 0.50 frozen
- [ ] Workspace **0.1.4** + CHANGELOG section + Docs headers
- [ ] No clap 5 / no rusqlite bump / no T307 / no H2 / no desktop bump / no git tag
- [ ] Full implement-track gate + PR squash-merge (never `git push origin main`)

## Isolation

Do not implement from this plan until **go**. No floor retune. No `cargo install`. No live vault mutate. Never `git push origin main`. Never force-push.
