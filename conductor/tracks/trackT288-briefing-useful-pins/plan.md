# T288 Plan — granted-empty briefing vault-pin stanza (not H2)

**Status:** **Pending** (Planned). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F35 / AC1–AC16
**Category:** FEATURE / UX / HONESTY
**Ledger TX (planning):** `6bf1d41c-a2c6-4b86-8b4b-2dee14690363` (DOCS)
**Ledger TX (implement):** FEATURE on **go**

---

## Preflight (plan time — 2026-08-23)

| Check | Result |
|-------|--------|
| HEAD / tree | `f3f2485` T287 `#203`. CLEAN. `origin/main` = HEAD |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T274. No T285/T286/T287.** Briefing hole is in **source**. **Do not `cargo install`.** |
| `briefing project --format human` | Granted-empty: Decisions `_None_`; Conclusions `_None_`; `empty_authority` + recall next. **No `Pinned:`** |
| `briefing project --format json` | `denied: false`, `decisions: []`, `conclusions: []`, `empty_authority`; no overlay keys |
| `memory list --summary` | `Pinned: 3822` / `Forgotten: 0` (volatile) |
| `preflight --summary` | Pinned **3820**; in-context **0/4/0**; word **1467** |
| Last PR comments | #203 T287 — **empty** (N/A). **No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; GitHub **v4.6.6**; **no clap 5**); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); rusqlite **0.39.0** (0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** — do not touch. `governed_common.rs` **#5** — do not grow. `personal.rs` **#7** — T289. CLI `preflight.rs` **#8** — do not grow. Extend `briefing.rs` + `renderer.rs` |
| Ledger | 0 pending / 0 drift at scan; planning TX `6bf1d41c` |
| `ISSUES.md` | **Does not exist** (F22) |
| ledgerful search | `render_project_markdown` → `renderer.rs` + `briefing.rs` + `preflight.rs:254` |
| Online | clig.dev human-first + JSON stable + next-command; T180 additive extras; T243 CLI overlay analog; T263 F24 COUNT promote; clap 4.6.6 / no clap 5 |
| Skill | CAPABILITIES dual-model sentence (F25). No pin write change |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [ ] Re-read `briefing.rs` `run_project` + `renderer.rs` `render_project_markdown` `:66` + empty_authority footer `:137–141`
- [ ] Confirm `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` still ≤140 / contains `recall` — **do not edit**
- [ ] Confirm `ProjectBriefingPacket` still has no vault-pin fields — **do not add**
- [ ] Confirm `count_pinned_memories` + `list_authority_memories` still on `QueryStore` — **do not** add a third method
- [ ] Confirm `preview_line` still in `memory.rs` — import only
- [ ] Confirm `classify_pin_kind` still Decision/Constraint/Hotspot/Other — **do not edit** `ranking.rs`
- [ ] Confirm `retrieval/src/preflight.rs` still calls `render_project_markdown(packet)` — **do not pass stanza**
- [ ] Confirm `personal.rs` Preferences `_None_` — **T289; do not edit**
- [ ] Rescan `conductor/deferred.md` — T288 rows absorbed; T289/T290/T293/T294 not stolen
- [ ] Confirm #203 comments/reviews still empty (N/A); no mint; Dependabot `#61` still not this track
- [ ] Re-dogfood `briefing project --format human` + `--format json` **read-only**. **Did not** pin production decisions; **did not** write `.env`; **did not** extra `policy bootstrap`
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [ ] FEATURE TX (new)
- [ ] Did **not** `cargo install`; did **not** grow `project.rs` / `preflight.rs` / `governed_common.rs` / `personal.rs` / `query_store.rs`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit granted-empty `_None_` vs 3k pins | **DoD** F2–F5 / AC1–AC4 / AC12 |
| Dual-model friction | **DoD** F1 keep split + F2 labeled stanza |
| T263 F24 soft COUNT | **DoD** F4 inventory COUNT |
| Placeholder Manual `briefing project --format human` | **DoD** AC12 |
| JSON if T180 allows | **DoD** F3 CLI overlay (agents are JSON) |

## Declined (written)

| Item | Why |
|------|-----|
| H2 pin→Approved | F1 / F24 — standing |
| DTO new required keys | F10 |
| Personal `_None_` | **T289** |
| Lists/progressive pin count | **T290** |
| Neighbors CLI | **T293** |
| leftover dest upsert | **T294** |
| Governed preflight stanza | F27 / T170 D21 |
| Lengthen `NEXT_STEP` | F6 |
| Grow `governed_common.rs` | F12 hotspot #5 |
| last-PR #203 Cursor | N/A empty — no T301 |
| clap 5 / rusqlite 0.40 / T240 F2 | Standing |

---

## Phase 1 — Red (required)

- [ ] `render_project_markdown_with_vault_pins__some__inserts_after_empty_authority` (AC3)
- [ ] `should_overlay_vault_pins__rstest_denied_nonempty_empty` (AC14)
- [ ] `briefing_project__granted_with_decision_pin__human_stanza_not_under_decisions` (AC1)
- [ ] `briefing_project__granted_with_decision_pin__json_overlay_count_and_previews` (AC2)
- [ ] Confirm they **fail** (not compile-error-only) before green

## Phase 2 — Green

- [ ] `VaultPinStanza` `{ count: u64, previews: Vec<String> }` + `BRIEFING_VAULT_PINS_HEADING`
- [ ] `render_project_markdown_with_vault_pins`; `render_project_markdown` wraps `None`
- [ ] CLI: parse `Repository:{uuid}`; `count_pinned_memories`; `list_authority_memories` + retain Decision\|Constraint; `preview_line` 80; fail-open
- [ ] JSON overlay keys; E1 omit / zero-empty
- [ ] Export from CP `mod.rs` / `lib.rs` if needed
- [ ] No `unwrap`/`expect`/`panic` in production

## Phase 3 — More ACs

- [ ] AC4 zero-pin granted-empty
- [ ] AC5 denied no stanza
- [ ] AC6 next-step length **stays green**
- [ ] AC7 T227 substance **stays green** (no heading)
- [ ] AC8 DTO serde has no `vault_pin_count`
- [ ] AC9 envelope preview inherit
- [ ] AC13 store units **stay green**
- [ ] AC15 chrome-only COUNT without DECISION preview
- [ ] AC16 Hotspot excluded from previews

## Phase 4 — Docs + gates

- [ ] CAPABILITIES dual-model sentence
- [ ] `briefing project` after_help one sentence (human stanza + JSON extras; authority arrays empty)
- [ ] PROTOCOL-COMPAT briefings: CLI optional keys; daemon packet unchanged
- [ ] CHANGELOG T288
- [ ] `cargo fmt --check` ; `cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings`
- [ ] `cargo nextest run --workspace` (or targeted then full before commit)
- [ ] `cargo deny check` ; `cargo audit`
- [ ] `ledgerful verify --scope full`

## Phase 5 — Manual + closeout

- [ ] Manual AC12 `cargo run -p ai-brains-cli -- briefing project --format human` (not PATH)
- [ ] Same `--format json` `denied: false` + overlay keys when granted-empty
- [ ] Phase-1 review log `review.md` → clean
- [ ] `codex-review` (FEATURE)
- [ ] conductor.md T288 **Completed**; deferred closeout table; README
- [ ] Publish: push `track/T288-*` → PR → `gh run watch --exit-status` CI green → `gh pr merge --squash --delete-branch` → fetch/prune. Never `git push origin main`. Never force-push.

## DoD

- [ ] Human granted-empty names `Pinned:` + `not Approved` + `recall`; Decisions stay `_None_`
- [ ] JSON authority arrays empty; CLI extras present when granted-empty
- [ ] Denied / H2 / DTO required keys / T289–T294 not stolen
- [ ] CI green + squash-merged

## Isolation (every phase)

No `cargo install`. No live pin as implement (hermetic needle SoT). No `.env` write. No extra live `policy bootstrap`. No `migrate governed`. No `retention apply --confirm`. No schtasks mutate.
