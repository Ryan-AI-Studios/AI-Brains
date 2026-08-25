# T298 Plan — device/replicate useful empty

**Status:** **Pending** (Planned; not Placeholder). Implement only on **go**.
**Spec:** [spec.md](./spec.md) F0–F27 / AC1–AC16
**Category:** UX / HONESTY
**Ledger TX (planning):** `839a62a1-2881-4fbb-b918-4ce5673d721c` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `b206dce2-6324-4c49-97f3-b3328d15db16` (DOCS)
**Ledger TX (implement):** FEATURE on **go**

---

## AI fold-in (2026-08-25) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 0**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F2/AC11:** malformed enrolled fingerprint → `{hostname} (enrolled; fingerprint unavailable)`, never `(not enrolled)`.
2. **F26:** `emit_device_roster` **must** return `Vec` (no second SQL list).
3. **F27/AC10:** no `serial_test` / no `#[serial(env)]`; nextest isolation.
4. **AC6/AC9:** same `COMPUTERNAME=T298-HOST` + `HOSTNAME` removed as AC1.
5. **AC11:** rstest includes active-without-local.
6. **F8:** exact 19-char `  this machine:    `.
7. **Phase 0:** re-locate doc line anchors.

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

- [x] `git fetch --all --prune` ; branch `track/T298-device-replicate`
- [x] `ledgerful doctor` ; ledger 0 pending / 0 drift before FEATURE TX
- [x] Re-read `device.rs` `emit_device_roster` / `run_status` / `EMPTY_ENROLL_HINT` / `DEVICE_STATUS_NEXT`
- [x] Re-read `replicate.rs` `run_status` human + JSON + `--quiet`
- [x] Re-locate doc anchors CAPABILITIES / OPERATIONS / INSTALL / PROTOCOL-COMPAT (OpenCode O1) — do not trust plan-time `:112`/`:113`/`:1082`/`:197`/`:107`/`:109` if those files moved
- [x] Re-dogfood `device status` + `replicate status` read-only — **do not bootstrap**
- [x] Pins clap **4.6.1** / rusqlite **0.39.0** — no bump; no `hostname` crate; **no** `serial_test` crate
- [x] FEATURE TX
- [x] Did **not** `cargo install`; did **not** grow hotspots / bootstrap live vault
- [x] Rescan `conductor/deferred.md` — T298 absorbed; T299–T300 / T240 F2 / 750 ms not stolen

---

## Absorbed deferred

- [x] Audit device/replicate U=5 empty → F1–F8 / AC1–AC9 / AC14
- [x] Placeholder Manual `device status` + `replicate status` / no bootstrap → AC14 / F13
- [x] Placeholder hostname/fingerprint + short honesty + existing `next:` → F1 / F4 / F5
- [x] Placeholder replicate `none` → **rewrite** F20 `{hostname} (not enrolled)`
- [x] T251 F14 partial lift → F4
- [x] last-PR #213 Cursor N/A → F18 no T301
- [x] Fold-in OpenCode m3 fail-open; F26 Vec return; F27 no serial_test; AC6 env inject; AC11 4-case rstest

---

## Phase 1 — TDD red

- [x] `os_hostname` rstest (AC10) — `TempEnv`; **no** `#[serial(env)]`; COMPUTERNAME wins / HOSTNAME fallback / `unknown` / trim
- [x] `this_machine_label` rstest (AC11) — empty / local 32-byte / active-without-local / 31-byte → `enrolled; fingerprint unavailable`
- [x] Hermetic empty `device status` four-line body (AC1) — inject `COMPUTERNAME=T298-HOST` + remove `HOSTNAME`
- [x] Hermetic empty `replicate status` this-machine `(not enrolled)` (AC6) — **same env inject as AC1**
- [x] JSON key-set freeze + no `this_machine` (AC7)

---

## Phase 2 — green

- [x] Const `DEVICE_STATUS_HONESTY` + helpers in `device.rs` (`pub(crate)`)
- [x] `run_status` (device): after emitter, print label + honesty + `next:` (F1/F5/F6)
- [x] **Required:** `emit_device_roster` returns `Vec` (F26); `run_list` discards; empty **copy** unchanged
- [x] `replicate.rs` human line after `enrolled_count` with exact 19-char prefix; JSON / `--quiet` untouched
- [x] Enrolled hermetic AC2 / AC9 (fingerprint shared)
- [x] Stay-green AC3–AC5 / AC8 / AC13 / AC16 / T176 / T198 fingerprint

---

## Phase 3 — docs

- [x] CAPABILITIES `:112` last-line `next:` kept; additive this-machine + honesty
- [x] CAPABILITIES `:113` additive human this-machine; JSON keys frozen
- [x] OPERATIONS `:1082` additive
- [x] INSTALL `:197` tip additive
- [x] PROTOCOL-COMPAT `:107` additive; `:109` keys unchanged
- [x] Root CHANGELOG T298 Unreleased
- [x] Optional Status about text (F22)

---

## Phase 4 — verify

- [x] Targeted nextest `-p ai-brains-cli` `device_status` + `device_replicate` + `empty_states`
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [x] Manual AC14 live empty — **no** bootstrap; record PATH vs source if they differ
- [x] `scripts/dev-check.ps1`
- [x] Phase-1 `review.md` + Codex `review.codex.md`

---

## Phase 5 — closeout

- [x] conductor T298 **Completed**
- [x] deferred.md T298 closeout table (unresolved lows)
- [x] README-T285-T300 T298 Completed
- [x] `ai-brains pin` DECISION
- [x] FEATURE TX commit
- [x] 0 pending / 0 drift (after commit)

---

## Phase 6 — publish (standing)

- [x] Push `track/T298-*`
- [x] Open PR to `main` if none
- [x] `gh run watch --exit-status` until GHA `CI` every job green
- [x] `gh pr merge --squash --delete-branch`
- [x] Hygiene: fetch/prune; point local `main` at `origin/main`

Never `git push origin main`. Never force-push.

---

## DoD

- [x] Empty `device status`: T198 + this-machine `(not enrolled)` + short honesty + `next:` last; exit 0
- [x] Empty `replicate status`: `enrolled_count: 0` + this-machine + honesty; does not claim sync running; JSON 6 keys frozen
- [x] List / fingerprint / `--quiet` / `--format json` on device unchanged-as-frozen
- [x] Manual AC14 recorded; **did not** bootstrap live vault
- [x] Full gate green; contracts/pins unchanged
- [x] Published (Phase 6)

---

## Declined (not this track)

| Item | Why |
|------|-----|
| Live `device bootstrap` | F13 |
| `--format` on `device` / JSON DTO | F11 |
| Replicate JSON `this_machine` key | F9 |
| Crate `hostname` / Win32 / `hostname.exe` | F3 / F14 |
| Crate `serial_test` / `#[serial(env)]` | F27 |
| Combined dashboard / doctor 16th | F16 |
| T299 forget-list / T300 graph sparse | F24 |
| clap 5 / rusqlite 0.40 / T240 F2 | F14 / F24 |
