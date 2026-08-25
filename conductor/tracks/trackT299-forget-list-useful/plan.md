# T299 Plan — forget list useful empty

**Status:** **Pending** (Planned; not Placeholder). Implement only on **go**.
**Spec:** [spec.md](./spec.md) F0–F33 / AC1–AC16
**Category:** UX / HONESTY
**Ledger TX (planning):** `4516432b-edbf-49b4-a11a-2e682db985c0` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `81ff640f-110b-4e12-872c-e4f468e016de` (DOCS)
**Ledger TX (implement):** FEATURE on **go**

---

## AI fold-in (2026-08-25) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 0**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **§7 / AC16:** JSON absence lock (AC5) is stay-green, not red.
2. **AC14:** Manual SoT is `cargo run -p ai-brains-cli`.
3. **AC2 / F6:** same flags → byte-identical stdout (`assert_eq!`).
4. **F19:** CAPABILITIES Empty row `:275`; CLI-EXIT-CODES sentence **required**.
5. **Phase 0:** re-locate doc anchors (including `:275`).

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
- [ ] Re-locate doc anchors CAPABILITIES Empty row (`:275` at plan) / OPERATIONS / WORKFLOWS / CHANGELOG / after_help (do not trust plan-time `:275`/`:745`/`:195`/`:1597`/`:2985` if those files moved)
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
- [ ] Fold-in OpenCode m1 stay-green JSON; m2 `cargo run` Manual; Agy m3 AC2 `assert_eq!`; O1 `:275`; O2 CLI-EXIT-CODES required

---

## Phase 1 — TDD red

- [ ] `forgotten_empty_remediator` rstest (AC10) — Some(n) / None / 0 / global
- [ ] Hermetic empty forgotten + ≥1 pin: `Pinned:` matches `--summary`; last line `next: ai-brains memory list` (AC1)
- [ ] Hermetic `forget --list-forgotten` stdout **equals** `memory list --status forgotten` **and** has AC1 markers (AC2)
- [ ] Hermetic `--global` last line includes `--global` (AC6)

---

## Phase 2 — green

- [ ] Const-free helper `forgotten_empty_remediator` in `memory.rs` (`pub(crate)`)
- [ ] `emit_list_human` Forgotten empty arm: COUNT via `count_memories` (Pinned, same project/tag/global) + print remediator; still `return Ok(())` before F36
- [ ] Pass `tag` into `emit_list_human` (F31 / AC11)
- [ ] Pinned-empty / summary / JSON / nonempty forgotten **untouched-as-frozen**
- [ ] `forget.rs` production **unchanged**
- [ ] Stay-green AC3–AC5 / AC7–AC9 / AC16 / T216 / T287 (AC5 JSON absence is **not** red)

---

## Phase 3 — docs

- [ ] CAPABILITIES Empty row additive (`Pinned: N` + last-line `next:`)
- [ ] OPERATIONS `:745` additive
- [ ] WORKFLOWS empty-forgotten case
- [ ] Root CHANGELOG T299 Unreleased
- [ ] Forget after_help + memory list after_help one sentence each
- [ ] CLI-EXIT-CODES **add** empty forgotten still exit 0

---

## Phase 4 — verify

- [ ] Targeted nextest `-p ai-brains-cli` `memory_list_inventory`
- [ ] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [ ] Manual AC14 live empty **via `cargo run -p ai-brains-cli`** — **no** live forget; optionally record PATH-behind diff
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
