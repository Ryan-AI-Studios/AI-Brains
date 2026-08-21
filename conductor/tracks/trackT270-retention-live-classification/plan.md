# T270 Plan — Retention live `memory_legacy` inventory overlay

**Status:** **Pending** (requirements written; not In Progress)
**Spec:** [spec.md](./spec.md) F0–F31 / AC1–AC17 + §13 AI fold-in
**Category:** FEATURE / UX / HONESTY
**Ledger TX (planning):** `3ebebd1f-58e1-4663-b559-75f900edfc95` (DOCS)
**Ledger TX (fold-in):** `56696e5a-9104-46c6-9313-447d2bacb7d1` (DOCS)
**Ledger TX (implement):** start **FEATURE** on **go**

---

## AI fold-in (2026-08-20) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F5 / AC1:** `pinned==0` → sample SQL `status != 'pinned' ORDER BY memory_id ASC LIMIT 5`.
2. **F5 / AC16:** `LIMIT 5` in SQL (not unbounded SELECT + Rust slice).
3. **F30 / AC17:** `classes` sorted by `class` after merge (green-phase guard).
4. **F31:** `NOTE_MEMORY_LEGACY_INVENTORY` const.
5. **F6:** merge after `build_report`, not inside `collect_candidates`.
6. **F8:** dispose-work empty-check — do **not** revert to `candidates==0`.

---

## Preflight (plan time — 2026-08-20)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `fdd4924`; fold-in `70d61cd` (docs-only; product tree identical). CLEAN; `main` ahead of `origin/main` by planning+fold docs |
| T166 / T248 | ✅ engine + human matrix in source. Stream-A `memory_legacy` **not** scanned |
| PATH `ai-brains` | `0.1.1`. `retention plan --format human`: `Nothing to dispose.` / `memory_legacy skip 0`. JSON `classes=[]` `candidates=0` |
| Live vault | `memory list --summary --global`: **Pinned 38,208** at plan (review **38,210** volatile) / Forgotten 29 |
| `whoami` | `mismatch: false`; effective `3581317d`; shell leftover `7d97a456` overridden |
| Last PR comments | #187 T272 — **empty** (N/A). **No T274** |
| Open PR on HEAD | none (Dependabot remotes only) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150**; chrono **0.4.44**; rusqlite **0.39.0** — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** (4.008 plan / 3.999 review) — do not grow. CLI `retention.rs` **902**; CP `class_based_retention.rs` **1204**; store `retention.rs` **450** — not top-10 |
| Ledger | 0 pending at scan; planning TX `3ebebd1f`; fold-in TX `56696e5a` |
| `ISSUES.md` | **Does not exist** (F24) |
| ledgerful search | `collect_candidates` at `:234/:269/:771/:990` (OpenCode also `:239/:240`) |
| Online | ISO 27001 A.8.10 record-against-schedule; clap 4.6.6 `after_help`; restic dry-run analogy |

---

## Phase 0 — on go (re-verify)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact`
- [ ] Re-read `collect_candidates` / `build_report` / `prepare_retention_apply` / `format_retention_pretty` empty-check
- [ ] Rescan `conductor/deferred.md` for new open rows that overlap
- [ ] Confirm #187 still empty / no new Cursor leftover that needs a mint
- [ ] Re-dogfood `retention plan --format human` and `--format json` vs `memory list --summary --global` (do **not** apply)
- [ ] Re-check clap/rusqlite/chrono lock vs crates.io (**no bump** unless execute proves otherwise)
- [ ] FEATURE TX start

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit 0 candidates on ~35k (now 38,208) pins | **DoD** F1–F11 / AC1–AC13 |
| Placeholder F1 honesty sentence | Overlay **plus** F10 warning (sentence-only not enough) |
| Placeholder F2 overlay | **DoD** COUNT + ≤5 samples |
| T166 §5.1.5 never coded | Inventory, not age-wipe |
| T248 empty-check / Work-all-classes | **Lift** F8 / F9 |

## Declined (written)

| Item | Why |
|------|-----|
| `classify_legacy` / `migrate governed` remediator | F1 / F18 — T167 importer |
| Live `retention apply --confirm` | F17 |
| Auto-forget / soft_forget pins | F3 |
| Doctor 16th / HTTP / desktop / nightly restyle | F20 |
| Identity mismatch `7d97a456` vs `fcb8a40f` | F19 — T240/T258; **no T274** |
| last-PR #187 Cursor | N/A empty |
| T273 F7 / T240 F2 / leftover rebind | Peers / standing |
| clap 5 / pin bumps / DTO new keys / `cargo install` | F14 / F15 / F16 |

---

## Phase 1 — Red (failing tests first)

- [ ] AC1 store unit: `memory_legacy_inventory__pinned_and_other__counts_and_limit_5` **plus** pinned=0 other&gt;0 sample case
- [ ] AC3 CP: `retention_plan__pinned_memories__memory_legacy_held_inventory`
- [ ] AC4 rstest `#[case]` forgotten/active → skip + non-empty samples
- [ ] AC5 mixed pinned+other → one bucket + split totals
- [ ] AC6 unit: `format_retention_pretty__held_inventory_only__nothing_to_dispose_no_work_no_next`
- [ ] AC9 hermetic: pin then `retention plan --format human` / `json`
- [ ] AC12: `retention plan --help` contains `none_auto` or `inventory`
- [ ] **Required red:** AC3 + AC6 + AC9 (HEAD has skip 0 / empty-check on `candidates==0` / no helper)
- [ ] AC2 / AC7 / AC8 / AC10 / AC11 may already be green (regression guards)
- [ ] AC17 class-sort is a **green-phase guard** (not Phase-1 required red)

## Phase 2 — Green (helper + merge + pretty)

- [ ] `MemoryLegacyInventory` + `memory_legacy_inventory` in `projections/retention.rs` (`COUNT` + SQL `ORDER BY memory_id LIMIT 5`; pinned vs `status != 'pinned'` fallback)
- [ ] `NOTE_MEMORY_LEGACY_INVENTORY` + `merge_memory_legacy_inventory` in `class_based_retention.rs`; call from `plan_retention`, `prepare_retention_apply`, `apply_retention`; **sort `classes` after merge** (F30)
- [ ] Contracts honesty const; pretty F8/F9; `honesty_short_label` F10
- [ ] Additive Plan `after_help`
- [ ] Do **not** push per-memory `Candidate`; do **not** edit `collect_candidates` retain for this; do **not** call `classify_legacy`
- [ ] Do **not** grow `project.rs` / `preflight.rs` / `sync.rs` / `nightly.rs` / `legacy_import.rs`

## Phase 3 — Regressions + hermetic

- [ ] AC2 empty CP; AC7 empty pretty; AC8 empty JSON hermetic
- [ ] AC10 plan does not append events
- [ ] AC11 apply without `--confirm` still exit-6 class
- [ ] AC17: old turn + pinned → `classes` sorted (`memory_legacy` before `raw_turn`)
- [ ] Existing envelope R11 `would_held >= 1` + no plaintext body
- [ ] `cargo nextest run -p ai-brains-control-plane class_based_retention`
- [ ] `cargo nextest run -p ai-brains-cli -E "test(retention_plan)"`
- [ ] `cargo clippy -p ai-brains-store -p ai-brains-control-plane -p ai-brains-cli -p ai-brains-contracts --all-targets -- -D warnings`

## Phase 4 — Docs + manual

- [ ] CAPABILITIES T248 row: inventory overlay sentence
- [ ] OPERATIONS `memory_legacy` none_auto + held pins
- [ ] PROTOCOL-COMPAT §5: live vaults may emit `memory_legacy` bucket; keys frozen
- [ ] Root CHANGELOG T270 row
- [ ] AC13 live human+json (source or PATH); **do not apply**

## Phase 5 — Review + publish (on go)

- [ ] `review.md` primary loop until clean
- [ ] `codex-review` (honesty UX)
- [ ] Full AGENTS.md gate + `ledgerful verify --scope full`
- [ ] FEATURE TX commit; conductor **Completed**; deferred residuals
- [ ] implement-track Phase 6: push `track/T270-*` → PR → watch GHA `CI` green → squash-merge → prune. Never `git push origin main`

---

## Definition of Done (checkable)

- [ ] Live/hermetic `retention plan` shows `memory_legacy` held count ≥ 1 on a pinned vault
- [ ] Same run still prints `Nothing to dispose.` when ce_wipe+projection_delete == 0
- [ ] No `Work` table / no `next: apply` for inventory-only
- [ ] JSON `api_version` 1; no new required keys; pin bodies absent
- [ ] `retention plan` appends **zero** events
- [ ] Store helper uses `COUNT` + `LIMIT 5` (AC16)
- [ ] Empty vault T248/CP tests still green
- [ ] Docs F27 landed
- [ ] No live apply; no `cargo install`; no `.env` rewrite; no `project.rs` growth
- [ ] Ledger FEATURE TX committed; 0 pending / 0 drift

---

## Manual evidence (fill on go)

| Command | Result |
|---------|--------|
| `cargo run -p ai-brains-cli -- retention plan --format human` | |
| `cargo run -p ai-brains-cli -- retention plan --format json` (totals/classes only) | |
| `ai-brains memory list --summary --global` (counts) | |
| Event count before/after plan (hermetic) | |
