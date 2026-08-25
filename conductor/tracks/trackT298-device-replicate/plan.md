# T298 Plan — device/replicate useful empty

**Status:** **Pending** (Planned; not Placeholder). Implement only on **go**.
**Spec:** [spec.md](./spec.md) F0–F25 / AC1–AC16
**Category:** UX / HONESTY
**Ledger TX (planning):** `839a62a1-2881-4fbb-b918-4ce5673d721c` (DOCS)
**Ledger TX (implement):** FEATURE on **go**

---

## Preflight (planning dogfood 2026-08-25)

| Check | Result |
|-------|--------|
| HEAD / tree | `01fb0db` T297 `#213`. CLEAN. `origin/main` = HEAD (`0 0`). Branch `main`. |
| PATH `ai-brains` | **0.1.2** 2026-08-22 19:41. Has T251 empty+next. **No** T298 this-machine. Do **not** `cargo install`. |
| `device status` | T198 + `next:`. Exit 0. No this-machine. No `local-only`. |
| `replicate status` | `enrolled_count: 0` + honesty + hint. No `this machine:`. JSON 6 keys. |
| Live OS | `COMPUTERNAME=DESKTOP`. `HOSTNAME` unset. Hermetic injects `T298-HOST`. |
| Live vault | **Zero** enrolled. Do **not** bootstrap. |
| Pins | clap lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.39.0** / crates.io **0.40.2**; no clap 5; `hostname` **0.4.2** not added. **Snapshot — re-verify at execute.** |
| last-PR Cursor | **#213** empty. **No T301.** Dependabot remotes only. |
| Ledger | 0 pending / 0 drift at scan. This DOCS TX `839a62a1`. |
| Hotspots | `project.rs` #1 — do not touch. `device.rs` / `replicate.rs` not top-10. |
| `ISSUES.md` | Does not exist. |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; branch `track/T298-device-replicate`
- [ ] `ledgerful doctor` ; ledger 0 pending / 0 drift before FEATURE TX
- [ ] Re-read `device.rs` `emit_device_roster` / `run_status` / `EMPTY_ENROLL_HINT` / `DEVICE_STATUS_NEXT`
- [ ] Re-read `replicate.rs` `run_status` human + JSON + `--quiet`
- [ ] Re-dogfood `device status` + `replicate status` read-only — **do not bootstrap**
- [ ] Pins clap **4.6.1** / rusqlite **0.39.0** — no bump; no `hostname` crate
- [ ] FEATURE TX
- [ ] Did **not** `cargo install`; did **not** grow hotspots / bootstrap live vault
- [ ] Rescan `conductor/deferred.md` — T298 absorbed; T299–T300 / T240 F2 / 750 ms not stolen

---

## Absorbed deferred

- [ ] Audit device/replicate U=5 empty → F1–F8 / AC1–AC9 / AC14
- [ ] Placeholder Manual `device status` + `replicate status` / no bootstrap → AC14 / F13
- [ ] Placeholder hostname/fingerprint + short honesty + existing `next:` → F1 / F4 / F5
- [ ] Placeholder replicate `none` → **rewrite** F20 `{hostname} (not enrolled)`
- [ ] T251 F14 partial lift → F4
- [ ] last-PR #213 Cursor N/A → F18 no T301

---

## Phase 1 — TDD red

- [ ] `os_hostname` rstest (AC10) — `TempEnv`; COMPUTERNAME wins / HOSTNAME fallback / `unknown`
- [ ] `this_machine_label` rstest (AC11) — empty / local 32-byte / bad len fail-open
- [ ] Hermetic empty `device status` four-line body (AC1) — inject `COMPUTERNAME=T298-HOST`
- [ ] Hermetic empty `replicate status` this-machine `(not enrolled)` (AC6)
- [ ] JSON key-set freeze + no `this_machine` (AC7)

---

## Phase 2 — green

- [ ] Const `DEVICE_STATUS_HONESTY` + helpers in `device.rs` (`pub(crate)`)
- [ ] `run_status` (device): after emitter, print label + honesty + `next:` (F1/F5/F6)
- [ ] Optional: `emit_device_roster` returns `Vec` so status does not double-list; empty **copy** unchanged
- [ ] `replicate.rs` human line after `enrolled_count`; JSON / `--quiet` untouched
- [ ] Enrolled hermetic AC2 / AC9 (fingerprint shared)
- [ ] Stay-green AC3–AC5 / AC8 / AC13 / AC16 / T176 / T198 fingerprint

---

## Phase 3 — docs

- [ ] CAPABILITIES `:112` last-line `next:` kept; additive this-machine + honesty
- [ ] CAPABILITIES `:113` additive human this-machine; JSON keys frozen
- [ ] OPERATIONS `:1082` additive
- [ ] INSTALL `:197` tip additive
- [ ] PROTOCOL-COMPAT `:107` additive; `:109` keys unchanged
- [ ] Root CHANGELOG T298 Unreleased
- [ ] Optional Status about text (F22)

---

## Phase 4 — verify

- [ ] Targeted nextest `-p ai-brains-cli` `device_status` + `device_replicate` + `empty_states`
- [ ] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [ ] Manual AC14 live empty — **no** bootstrap; record PATH vs source if they differ
- [ ] `scripts/dev-check.ps1`
- [ ] Phase-1 `review.md` + Codex `review.codex.md`

---

## Phase 5 — closeout

- [ ] conductor T298 **Completed**
- [ ] deferred.md T298 closeout table (unresolved lows)
- [ ] README-T285-T300 T298 Completed
- [ ] `ai-brains pin` DECISION
- [ ] FEATURE TX commit
- [ ] 0 pending / 0 drift (after commit)

---

## Phase 6 — publish (standing)

- [ ] Push `track/T298-*`
- [ ] Open PR to `main` if none
- [ ] `gh run watch --exit-status` until GHA `CI` every job green
- [ ] `gh pr merge --squash --delete-branch`
- [ ] Hygiene: fetch/prune; point local `main` at `origin/main`

Never `git push origin main`. Never force-push.

---

## DoD

- [ ] Empty `device status`: T198 + this-machine `(not enrolled)` + short honesty + `next:` last; exit 0
- [ ] Empty `replicate status`: `enrolled_count: 0` + this-machine + honesty; does not claim sync running; JSON 6 keys frozen
- [ ] List / fingerprint / `--quiet` / `--format json` on device unchanged-as-frozen
- [ ] Manual AC14 recorded; **did not** bootstrap live vault
- [ ] Full gate green; contracts/pins unchanged
- [ ] Published (Phase 6)

---

## Declined (not this track)

| Item | Why |
|------|-----|
| Live `device bootstrap` | F13 |
| `--format` on `device` / JSON DTO | F11 |
| Replicate JSON `this_machine` key | F9 |
| Crate `hostname` / Win32 / `hostname.exe` | F3 / F14 |
| Combined dashboard / doctor 16th | F16 |
| T299 forget-list / T300 graph sparse | F24 |
| clap 5 / rusqlite 0.40 / T240 F2 | F14 / F24 |
