# T299 Plan — forget list useful empty

**Status:** **Pending** (Planned; not Placeholder). Implement only on **go**.
**Spec:** [spec.md](./spec.md) F0–F33 / AC1–AC16
**Category:** UX / HONESTY
**Ledger TX (planning):** `4516432b-edbf-49b4-a11a-2e682db985c0` (DOCS)
**Ledger TX (implement):** FEATURE on **go**

---

## Preflight (planning dogfood 2026-08-25)

| Check | Result |
|-------|--------|
| HEAD / tree | `5323034` T298 `#214`. CLEAN. `origin/main` = HEAD (`0 0`). Branch `main`. |
| PATH `ai-brains` | **0.1.2** 2026-08-22 19:41. Has T216 empty one-liner. **No** T299 `Pinned:` / `next:`. Do **not** `cargo install`. |
| `forget --list-forgotten --limit 5` | Scope + `status=forgotten` + `No forgotten memories.` Exit 0. PATH **and** `cargo run` identical. |
| `--format json` | Nine keys. `items: []`. `total: 0`. No `next_step`. |
| `memory list --summary` | `Pinned: 4152` (volatile). `Forgotten: 0`. |
| Live vault | **Forgotten: 0**. Do **not** auto-forget. |
| Pins | clap lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.39.0** / crates.io **0.40.2**; no clap 5; `rstest` **0.25**. **Snapshot — re-verify at execute.** |
| last-PR Cursor | **#214** empty. **No T301.** Dependabot remotes only. |
| Ledger | 0 pending / 0 drift at scan. This DOCS TX `4516432b`. |
| Hotspots | `project.rs` #1 — do not touch. `forget.rs` #5 — **do not grow.** Implement in `memory.rs`. |
| `ISSUES.md` | Does not exist. |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; branch `track/T299-forget-list-useful`
- [ ] `ledgerful doctor` ; ledger 0 pending / 0 drift before FEATURE TX
- [ ] Re-read `memory.rs` `run_inventory` / `emit_list_human` empty arm / `run_summary` COUNT / `MemoryListJson`
- [ ] Re-read `forget.rs` list-forgotten wrapper (must stay thin)
- [ ] Re-locate doc anchors CAPABILITIES Empty row / OPERATIONS / WORKFLOWS / CHANGELOG / after_help (do not trust plan-time `:274`/`:745`/`:195`/`:1597`/`:2985` if those files moved)
- [ ] Re-dogfood `forget --list-forgotten --limit 5` + `memory list --summary` read-only — **do not forget live pins**
- [ ] Pins clap **4.6.1** / rusqlite **0.39.0** — no bump; no new crate
- [ ] FEATURE TX
- [ ] Did **not** `cargo install`; did **not** grow `forget.rs` / hotspots / live-forget
- [ ] Rescan `conductor/deferred.md` — T299 absorbed; T300 / T240 F2 / 750 ms not stolen

---

## Absorbed deferred

- [ ] Audit forget-list empty U=6 → F1–F6 / AC1–AC6 / AC14
- [ ] Placeholder Manual `forget --list-forgotten` + `memory list --summary` / no auto-forget → AC14 / F13
- [ ] Placeholder keep empty const + `Pinned: N` + `next: memory list` → F1 / F3 / F4
- [ ] Placeholder JSON `next_step` → **rewrite** F20 human-only
- [ ] T216 F36 partial lift → F27
- [ ] T287 F7 empty next parked here → absorb
- [ ] last-PR #214 Cursor N/A → F18 no T301

---

## Phase 1 — TDD red

- [ ] `forgotten_empty_remediator` rstest (AC10) — Some(n) / None / 0 / global
- [ ] Hermetic empty forgotten + ≥1 pin: `Pinned:` matches `--summary`; last line `next: ai-brains memory list` (AC1)
- [ ] Hermetic `memory list --status forgotten` matches forget list (AC2)
- [ ] JSON key absence `next_step` / `pinned` (AC5)
- [ ] Hermetic `--global` last line includes `--global` (AC6)

---

## Phase 2 — green

- [ ] Const-free helper `forgotten_empty_remediator` in `memory.rs` (`pub(crate)`)
- [ ] `emit_list_human` Forgotten empty arm: COUNT via `count_memories` (Pinned, same project/tag/global) + print remediator; still `return Ok(())` before F36
- [ ] Pass `tag` into `emit_list_human` (F31 / AC11)
- [ ] Pinned-empty / summary / JSON / nonempty forgotten **untouched-as-frozen**
- [ ] `forget.rs` production **unchanged**
- [ ] Stay-green AC3–AC4 / AC7–AC9 / AC16 / T216 / T287

---

## Phase 3 — docs

- [ ] CAPABILITIES Empty row additive (`Pinned: N` + last-line `next:`)
- [ ] OPERATIONS `:745` additive
- [ ] WORKFLOWS empty-forgotten case
- [ ] Root CHANGELOG T299 Unreleased
- [ ] Forget after_help + memory list after_help one sentence each
- [ ] CLI-EXIT-CODES empty forgotten still 0 (sentence if missing)

---

## Phase 4 — verify

- [ ] Targeted nextest `-p ai-brains-cli` `memory_list_inventory`
- [ ] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [ ] Manual AC14 live empty — **no** live forget; record PATH vs source if they differ
- [ ] `scripts/dev-check.ps1`
- [ ] Phase-1 `review.md` + Codex `review.codex.md`

---

## Phase 5 — closeout

- [ ] conductor T299 **Completed**
- [ ] deferred.md T299 closeout table (unresolved lows)
- [ ] README-T285-T300 T299 Completed
- [ ] `ai-brains pin` DECISION
- [ ] FEATURE TX commit
- [ ] 0 pending / 0 drift (after commit)

---

## Phase 6 — publish (standing)

- [ ] Push `track/T299-*`
- [ ] Open PR to `main` if none
- [ ] `gh run watch --exit-status` until GHA `CI` every job green
- [ ] `gh pr merge --squash --delete-branch`
- [ ] Hygiene: fetch/prune; point local `main` at `origin/main`

Never `git push origin main`. Never force-push.

---

## DoD

- [ ] Empty `forget --list-forgotten`: `No forgotten memories.` + `Pinned: N` matching `--summary` + last-line `next: ai-brains memory list`; exit 0
- [ ] `memory list --status forgotten` empty matches (shared backend)
- [ ] JSON nine keys frozen; no `next_step`; nonempty forgotten / pinned-empty / summary unchanged-as-frozen
- [ ] Manual AC14 recorded; **did not** forget live pins
- [ ] Full gate green; contracts/pins unchanged; `forget.rs` production unchanged
- [ ] Published (Phase 6)

---

## Declined (not this track)

| Item | Why |
|------|-----|
| Live auto-forget / `--match -f` | F13 |
| JSON `next_step` / `pinned` key | F10 / F20 |
| `--summary` on `forget` | F9 / T216 F28 |
| Forgotten human mix | F7 / T287 F7 |
| Grow `forget.rs` | F6 / hotspot #5 |
| T300 graph sparse | F24 |
| clap 5 / rusqlite 0.40 / T240 F2 | F14 / F24 |
