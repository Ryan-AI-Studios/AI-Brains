# T284 Plan — Retention Work + apply samples

**Status:** **Pending** (Planned — not In Progress)
**Spec:** [spec.md](./spec.md) F0–F41 / AC1–AC17 + §13 AI fold-in
**Category:** BUGFIX / HONESTY
**Ledger TX (planning):** `d2010eda-264a-449b-9f37-f3e7687e9fe1` (DOCS)
**Ledger TX (fold-in):** `9c454170-57a9-405a-b6e6-ace0b177b472` (DOCS)
**Ledger TX (implement):** start **BUGFIX** on **go**

---

## AI fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F37 / AC5:** zero-dispose class JSON keys exactly five (`class`, `candidate_count`, `mechanism`, `sample_ids`, `notes`).
2. **F41 / AC16:** `audit_sample_ids` same-file units (overlay pins OK; mixed dispose-only, cap 5, de-duped).
3. **AC17:** pretty fallback when `dispose_sample_ids` empty.
4. **F27:** helper is `pub(crate)`, not `pub`.
5. **Already:** F6 fallback (Agy m1); F7 de-dupe (Agy m2); F28 no Default (OpenCode m2); F38 comment (Agy O2 / OpenCode m1).
6. **Decline:** `#[derive(Default)]`; public helper for `tests/`; pin bumps.

---

## Preflight (plan time — 2026-08-22)

| Check | Result |
|-------|--------|
| HEAD / tree | **Plan dogfood:** `abaab31` T277 `#192`. **This fold-in:** `da6f316` (plan docs; product crates identical). CLEAN; `main` ahead of `origin/main` by planning docs |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-21 05:55. **T270** on PATH. Work/samples hole is T270-era. **Do not `cargo install`.** |
| `preflight --summary` | Pinned **3429**; in-context 0/0/0; grants **0 of 3**; Scope `3581317d` |
| `whoami` | `mismatch: false`; shell leftover `7d97a456` (T282 / T258 — not this track) |
| `memory list --summary --global` | Pinned **39118** / Forgotten 29; leftover `7d97a456` **18039** |
| PATH `retention plan --format human` | `Nothing to dispose.` `memory_legacy held 39147`; skip 29; **no Work**; **no next:** |
| PATH JSON class keys | `class`, `candidate_count`, `mechanism`, `sample_ids`, `notes` only (inventory omits extras) |
| Last PR comments | #192 T277 — **empty** (N/A). #188 Mediums stay **this track**. No T285 |
| Open PR on HEAD | none (Dependabot remotes only) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); rusqlite **0.39.0** (0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** (3.962) — do not grow. `preflight.rs` #7 — do not grow. `doctor.rs` **1855** — do not grow. CLI `retention.rs` **981**; CP `class_based_retention.rs` **1292** |
| Ledger | 0 pending / 0 drift at scan; planning TX `d2010eda` |
| `ISSUES.md` | **Does not exist** (F23) |
| ledgerful search | `dominant_mechanism` `:686`; `append_retention_applied` `:1248`; Work filter `:434` |
| Online | ISO 27001 A.8.10 record deletion; GDPR deletion log = class/count/method not bodies; clap 4.6.6; rusqlite 0.40.2 **not** bumped |

---

## Phase 0 — on go (re-verify)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before BUGFIX TX)
- [ ] Re-read Work filter `retention.rs` ~`:420–445`, `next:` ~`:514`, `dominant_mechanism` ~`:686`, `build_report` ~`:610`, `merge_memory_legacy_inventory` ~`:711`, `append_retention_applied` ~`:1263`
- [ ] Re-read clap `RetentionCommands::Plan` ~`:2298` / `Apply` ~`:2312`
- [ ] Confirm T270 inventory pretty `:682` and overlay apply `:1031` still present
- [ ] Rescan `conductor/deferred.md` — T284 rows already absorbed; no new overlapping open rows
- [ ] Confirm #192 comments/reviews still empty (N/A); #188 two Mediums still the hole; no mint
- [ ] Re-dogfood `retention plan --format human` / `--format json` only. **Did not** live apply
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [ ] BUGFIX TX on go
- [ ] Did **not** `cargo install`; did **not** grow `doctor.rs` / `project.rs`; did **not** live `retention apply --confirm`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| #188 Work hides CE when held dominates | **DoD** F1/F6 / AC1/AC3 |
| #188 apply samples prefer overlay ids | **DoD** F7 / AC2 |
| T270 F9 Work dispose-only | **Lift** F1 — class dispose counts |
| T270 F8 empty-check | **Affirm** AC4 |

## Declined (written)

| Item | Why |
|------|-----|
| Live `retention apply --confirm` | F16 |
| Change `dominant_mechanism` / split buckets | F2/F3 |
| T270 overlay removal | F9 |
| Doctor / nightly restyle / HTTP / desktop | F17 |
| last-PR #192 Cursor | N/A empty |
| T278–T283 / leftover rebind / T240 F2 / clap 5 / rusqlite 0.40 | F18/F19 |

---

## Phase 1 — Red (TDD)

- [ ] `retention_plan__mixed_held_and_ce_secret__held_dominant_dispose_counts` (rstest: 1+1 tie, 2+1 majority) — AC1
- [ ] `format_retention_pretty__held_dominates_ce_same_class__work_shows_dispose_row` — AC3
- [ ] `retention_apply__overlay_plus_raw_turn__applied_samples_include_turn` — AC2
- [ ] Commit red allowed

## Phase 2 — Green

- [ ] F4 optional fields on `RetentionClassBucket` (`skip_serializing_if` zero/empty) + `class_dispose_count`
- [ ] F5 `build_report` fills dispose counts + `dispose_sample_ids` (CE first); overlay merge does not
- [ ] F6 Work rows from class dispose counts (two rows if both mechanisms)
- [ ] F7 `audit_sample_ids` — dispose-only when totals dispose > 0
- [ ] F28 update ~6 struct literals
- [ ] F38 stale comment
- [ ] F11 Plan `after_help` additive
- [ ] F37 / AC5 exact 5-key omit; roundtrip still equal
- [ ] F41 / AC16 `audit_sample_ids` same-file units
- [ ] AC17 pretty fallback when `dispose_sample_ids` empty
- [ ] AC4/AC6/AC11/AC12/AC15 stay green
- [ ] Commit green

## Phase 3 — Docs

- [ ] CAPABILITIES T248/T270 row: Work = dispose identities
- [ ] OPERATIONS Audit: `RetentionApplied` samples prefer dispose ids
- [ ] PROTOCOL-COMPAT: optional class-bucket extras; absent = 0
- [ ] CHANGELOG T284
- [ ] conductor Completed only on implement closeout — **not** this planning pass

## Phase 4 — Verify

- [ ] Targeted nextest: `-p ai-brains-contracts retention` ; `-p ai-brains-control-plane --lib` ; `-p ai-brains-control-plane --test class_based_retention` ; `-p ai-brains-cli --test retention_plan_human`
- [ ] `cargo clippy -p ai-brains-contracts -p ai-brains-control-plane -p ai-brains-cli --all-targets -- -D warnings`
- [ ] `cargo fmt --check`
- [ ] Primary review → `review.md`; mediums not silently dropped
- [ ] Cross-model `codex-review` (F22)
- [ ] Full workspace gate at closeout only
- [ ] Optional PATH `retention plan --format human` still inventory-only (AC13). **No** live apply

## DoD (checkable)

- [ ] Mixed CE+held class still appears under Work when `would_ce_wipe>0` (AC1/AC3)
- [ ] `RetentionApplied` samples include dispose ids when overlay is large (AC2)
- [ ] Inventory-only vault still `Nothing to dispose.` (AC4)
- [ ] Zero-dispose class JSON keys exactly five (AC5)
- [ ] `audit_sample_ids` helper units (AC16)
- [ ] Pretty fallback when `dispose_sample_ids` empty (AC17)
- [ ] No live `retention apply --confirm`
- [ ] No `cargo install`
- [ ] implement-track Phase 6: push `track/T284-*` → PR → watch GHA `CI` green → squash-merge → prune (never `git push origin main`)

## Stop-before

- Live apply / CE / `.env` rewrite / schtasks mutate / `cargo install`
- Scope exceeds T284 (do not steal T278–T283, T277 live create, T275 bootstrap)
- Ambiguous spec vs src after Phase 0 — halt and ask
