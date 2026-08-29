# T321 Plan — `safety sync` write honesty

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** UX / SAFETY
**Ledger (planning):** DOCS `956c8463-c577-44cf-a614-169d77117446`

---

## Preflight (plan time — 2026-08-29)

| Check | Result |
|-------|--------|
| HEAD / tree | `16edc3f` T318 Completed note `#242` CLEAN. Branch `track/T321-safety-sync-honesty` off `main` = `origin/main`. Ahead **0** at plan start. Product `src/` = T318 `#241` `3bac49e`. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T279 **on PATH**. T312–T318 **not**. Hole **is** (write default + Scanning + JSON `[` finder). |
| `preflight --summary` (PATH) | Pinned **4616**; in-context **0/0/0**; `Total Word Count: 701` (PATH-behind T315). 0 hotspots = envelope hole. |
| PATH `safety sync --help` | `Synchronize…`; `--dry-run` “synced”; **no** after_help |
| PATH `safety sync --dry-run` | Scanning + text-mode JSON miss + `would sync 5` + scores **3.65…** |
| `ledgerful hotspots --json --limit 5` | `{schemaVersion:1, files:[…]}` raw score ~0.037 / displayScore ~3.65 |
| `run` / fetch | `safety.rs:11–128` scan + `[` finder |
| Retrieval parse | `preflight_safety.rs:29–62` `[` + cap 5 |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; serde_json **1.0.150**; uuid ws `"1.13"` / lock **1.23.1**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#242` empty. `#241` empty. `#237` → **T326**. `#230` → **T325**. **No T327.** |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan |
| Hotspots | `project.rs` #1 **3.648** — do not touch. CLI `preflight.rs` #7 — do not grow. `safety.rs` not top 10. |
| Line counts | `safety.rs` **168** nonblank / **187** physical; `preflight_safety.rs` **227** physical; `pin.rs` **138**. F22 = go-HEAD diff. |
| `ISSUES.md` | **Does not exist** |
| Planning install / live pin | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit safety 5/5 write surprise + chatter | **DoD** F1–F6 / AC1–AC4 / AC7 / AC14 |
| Placeholder dry-run-default vs banner | **Banner** F1 — default stays write |
| T279 remediator `--dry-run` | **Freeze** F8 |
| Live JSON `{files[]}` | **DoD** F7 / AC5 / AC6 |
| T279 F29 parser drift | **Copy-not-share** F29 |
| WORKFLOWS LedgerEntry lie | **Docs** F10 / AC13 |
| antigravity-rule session-start write | **Docs** F33 / AC13 |
| last-PR `#242`/`#241` | **N/A empty** |
| last-PR `#237` / `#230` | **T326** / **T325** — not stolen |
| T322–T326 / clap 5 | **Not stolen** / **Decline** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains`
- [ ] Re-read `safety.rs` `run` `:11–100` + `fetch_hotspots_json` `:102–128` + `fetch_hotspots_text` + `render_hotspots`
- [ ] Re-read clap `SafetyCommands::Sync` `main.rs:3716–3725` + dispatch `:5427–5430`
- [ ] Re-read retrieval `parse_hotspots_json` `:29–62` + T279 units `:151–178` + `SAFETY_EMPTY`
- [ ] Re-read `pin.rs` `run` (do **not** edit production)
- [ ] Re-dogfood `safety sync --help` + `safety sync --dry-run` (**never omit `--dry-run`**)
- [ ] Re-run `ledgerful hotspots --json --limit 5` — confirm envelope still `{schemaVersion, files[]}`
- [ ] Confirm clap lock still **4.6.1**; T325 / T326 / T322 still Pending (do not steal)
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] `ledgerful ledger start T321-safety-sync-honesty --category FEATURE`
- [ ] **Do not** `cargo install` / live `safety sync` without `--dry-run` / `.env` rewrite / clap 5 / grow `project.rs` / `pin.rs` production

## Phase 1 — Red

- [ ] `safety_sync_help__after_help__names_pin_and_dry_run` (AC1) — must **fail** today
- [ ] `safety_sync_clap__default__dry_run_false` (AC2) — lock freeze (green-on-arrival ok)
- [ ] `format_write_banner__names_pinning_and_count` (AC3) — must **fail** (helper absent)
- [ ] `format_dry_run_header__would_pin_not_sync` (AC4)
- [ ] `parse_hotspots_json__envelope_v1_files__raw_score` (AC5) — must **fail** (empty on object)
- [ ] CLI `…envelope_v1_files__raw_score` (AC6)
- [ ] AC7 no `Scanning for Ledgerful Hotspots` — must **fail** today

## Phase 2 — Green

- [ ] Envelope parse in CLI `fetch_hotspots_json` + retrieval `parse_hotspots_json` (F7 / F21 / F29)
- [ ] Drop Scanning / scan-complete / text-mode stdout; JSON-err → `tracing::warn!` (F4)
- [ ] `format_write_banner` / `format_dry_run_header`; human rows path + raw `{:.2}` (F2 / F5)
- [ ] Drop `Safety synchronization complete…`; do not edit `pin.rs` (F6)
- [ ] Sync about + after_help (F3 / F23)
- [ ] Empty path unchanged (F36)

## Phase 3 — Stay-green + docs

- [ ] T279 array + cap-5; SAFETY_EMPTY; preflight help remediator (AC8/AC9/AC16)
- [ ] AC11/AC12 empty diff `pin.rs` / CLI `preflight.rs` / `help_ia.rs` / `project.rs` / `doctor.rs` production
- [ ] CAPABILITIES; OPERATIONS §7; WORKFLOWS; antigravity-rule; CHANGELOG (AC13)

## Phase 4 — Manual + gate + publish

- [ ] AC14 `cargo run -p ai-brains-cli -- safety sync --dry-run` only (no Scanning; would pin; scores &lt; 1; exit 0)
- [ ] Targeted `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` ; `-p ai-brains-retrieval` ; nextest named tests
- [ ] FEATURE cross-model (`codex-review`)
- [ ] Full gate; conductor Completed; deferred residuals; implement-track Phase 6 publish

## Manual AC14 evidence (fill on go)

```text
cargo run -p ai-brains-cli --quiet -- safety sync --dry-run
  (record stdout/stderr/exit; scores; no Scanning; no pin)
```

---

## DoD (checkable)

- [ ] Default still writes (`dry_run` false) — F1
- [ ] Write banner + `--dry-run` `would pin` — F2/F5
- [ ] No Scanning / no text-mode stdout — F4/AC7
- [ ] Envelope JSON path + legacy array stay-green — F7/AC5
- [ ] T279 remediator exact — F8
- [ ] `pin.rs` / `help_ia.rs` / CLI `preflight.rs` untouched — F6/F11
- [ ] No live pin as proof — F12
- [ ] Docs AC13
- [ ] FEATURE TX committed; conductor Completed only after go + gate + publish

## Isolation

No live hotspot pins as planning or implement proof. No `cargo install`. Never `git push origin main`.
