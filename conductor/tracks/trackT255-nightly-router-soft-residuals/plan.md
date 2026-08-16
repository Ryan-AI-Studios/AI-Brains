# T255 Plan — Nightly / router soft residuals (T229+ / T247+)

**Status:** ✅ **Completed** (2026-08-16)
**Spec:** [spec.md](./spec.md) F0–F37 / AC1–AC16 + §12 AI fold-in
**Category:** OPS / POLISH / UX
**Ledger TX (planning):** `5d3d182d-f689-4673-9a03-733f5a178f3c` (DOCS)
**Ledger TX (fold-in):** `6a1380e9-3c04-4d0f-b8d9-69f2dcb1a265` (DOCS)
**Ledger TX (implement):** `646bd578-95ab-4220-9c05-306996ae6930` (FEATURE)

---

## AI fold-in (2026-08-16) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI2 affirms F2/F10/F11/F12/F21. AI1 seven blind spots: six folded, one declined as stated (`first_quoted` already `pub(crate)`). Disposition in spec **§12**.

### Pins locked by fold-in

1. **AC3/F5:** one key list — include `action_target` + `errors_last_run_unreadable`.
2. **F20/F35:** thread raw `Option` from `get_sync_state`; do not coalesce `"0"`/`"[]"` before the JSON builder.
3. **F34:** snapshot `{ found, snap }`. Nightly `scheduled` = `next_run.is_some()`. Router `scheduled` = `found` (ONLOGON).
4. **F21:** pass precomputed action fields; helper stays where it is.
5. **AC9:** hermetic asserts only stable keys (not host `schtasks`).
6. **F25/F37:** Nightly `after_help` is required.
7. **AC15/AC16:** blank-status Router line; non-Windows JSON `null`.

---

## Preflight (plan time — 2026-08-15)

| Check | Result |
|-------|--------|
| HEAD / tree | `1f7b014` CLEAN; ahead of origin/main by **10** (T252–T254 local, not pushed) |
| T254 | ✅ Completed in source. T255 was Placeholder. |
| PATH `ai-brains` | Pre-T247 (`--quick` unknown). **OOS** F29 |
| Source `--status --quick` | T247 surface green: Last Result **1** + missing `.cmd` + `probe=skipped` |
| `AI-Brains-Nightly` | Ready; Last Result **1**; action `"…\nightly-run.cmd"` **missing** |
| `AI-Brains-Router` | Running; Last Result **267009**; Task To Run unquoted `C:\llm\router.bat` |
| :8081 / :8083 | Completion down; embedding **ok** |
| `nightly --status --format` | **Does not exist** |
| Doctor | Frozen **15** checks — do not add model ports |
| Embed sleep | 50 ms; no live nightly since 2026-08-02 — decline F13 |
| Last PR comments | #167 / #168 — **none** (no Cursor) |
| Pins | clap lock **4.6.1**, tokio **1.52.3**, serde_json **1.0.150**, reqwest **0.13.4** models-only — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1**. `nightly.rs` 1819 lines, **not** top-10 → new code in `nightly_status.rs` |
| Ledger | 0 pending at scan; planning TX `5d3d182d` |
| `ISSUES.md` | **Does not exist** — debt is `deferred.md` |
| T167 / T240 F13–F14 / T253 F34 | Peers — **not absorbed** |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| JSON `nightly --status` | T229 F12 / T247 F11 | **DoD** F1–F6 / F19–F20 / AC1–AC4 / AC9–AC10 / AC14 |
| Router ONLOGON 267009/267014 | T229 F11 / T247 F15 | **DoD** as **read-only** display F7–F10 / AC5–AC7 / AC11 |
| Doctor model-port matrix | T229 F8 / T247 F12 | **Declined** F11 |
| Persist probe in `sync_state` | T229 F9 / T247 F13 | **Declined** F12 |
| Register Router from `--schedule` | T229 F10 | **Declined** F14 |
| Product `nightly-run.cmd` | T247 F16 | **Declined** F14 |
| 50ms embed sleep | T229 F14 / T247 F14 | **Declined** F13 |
| `--quick --no-vault` | T247 O12 | **Declined as DoD** F15 (soft residual) |
| T253 Claude/Codex nightly | T253 F34 / T239 D16 | **Not absorbed** F16 |
| T167 importer | conductor | **Not absorbed** F33 |
| T240 F13/F14 | T240 residual | **Not absorbed** F33 |
| T254 F12 leftovers | T254 closeout | **Not absorbed** |
| clap 5 / pin bumps | series | **Not absorbed** F23 |
| Shared `resolve_*_format` | T249 F12 | **Declined** F3 / F33 |
| Live reschedule missing `.cmd` | T247 F10 | **Stop-before** F30 — not DoD |
| PATH reinstall | live | **Not absorbed** F29 |
| AC3/§5.1 key mismatch | AI1 BS1 | **Absorbed** F5 / AC3 |
| Coalesced `"0"`/`"[]"` | AI1 BS2 | **Absorbed** F20 / F35 |
| Router ONLOGON scheduled | AI1 BS3 + O1 | **Absorbed** F34 / F9 |
| `first_quoted` “private” | AI1 BS4 | **Declined as stated** — already `pub(crate)`; **absorbed intent** F21 precompute |
| AC9 host schtasks flake | AI1 BS5 | **Absorbed** AC9 wording |
| Blank-status Router line | AI1 BS6 | **Absorbed** AC15 |
| Nightly `after_help` | AI1 BS7 | **Absorbed** F37 |
| Shared format helper | AI1 (implicit) / T249 | **Declined** F3 |

---

## Architecture SoT (files on go)

| Area | Path | Today | T255 change |
|------|------|-------|-------------|
| clap | `main.rs` `Commands::Nightly` | `--status` / `--quick` | **+ `--format`** requires status, default `human` |
| Dispatch | `main.rs` ~3713 | `nightly::run(…, quick)` | Thread `format: String` |
| Status branch | `nightly.rs` `if status` | Human only | Call `nightly_status`; additive Router lines; JSON emit |
| **New** | `commands/nightly_status.rs` | — | resolver + JSON types + Router formatter + units |
| LIST /V | `SchtasksListV` in `nightly.rs` | 4 fields | Additive `status: Option<String>` (F8). Keep parser here (T247 tests) |
| Fetch | `fetch_schedule_snapshot(task_name)` | `SchtasksListV`; miss ≡ default | **F34** `{ found, snap }`; second call `"AI-Brains-Router"` |
| Sync state | `nightly.rs:42-47` | `unwrap_or "0"` / `"[]"` | **F20/F35** thread `Option` into builder; human may still print 0/[] |
| Multi-import | `multi_import.rs` | print human | Reuse view/report in JSON (no schema fork) |
| Doctor / embeddings / models | — | 15 checks / 50 ms / probe | **No** |
| Docs | CAPABILITIES / OPERATIONS / CLI-EXIT-CODES / CHANGELOG | T247 human | F25 |

---

## Phase 0 — Ledger + impact (on go)

- [x] `ledgerful ledger status --compact` — 0 pending / 0 drift
- [x] `ledgerful ledger start T255-nightly-router-soft-residuals --category FEATURE --message "JSON nightly --status + read-only Router line; no doctor/embed/wrapper"` — TX `646bd578-95ab-4220-9c05-306996ae6930`
- [x] `ledgerful scan --impact` — tree CLEAN at start; expect `nightly.rs` / `main.rs` / new `nightly_status.rs`
- [x] Confirm `embeddings.rs` and `doctor.rs` are **not** in the touch set

## Phase 1 — Red → Green: format clap + resolver (F3 / F4 / F37 / AC1–AC2)

- [x] Units in `nightly_status.rs` for `resolve_nightly_status_format` (AC1)
- [x] clap `--format` on `Nightly`: default `human`, `requires = "status"`, conflicts schedule/unschedule, T248 token set
- [x] **Required** Nightly `after_help` (F37): `--format json` + “default human; pipes stay human”
- [x] Clap tests next to existing `nightly_quick__*` : no-status, `xml`, `JSON` (copy T248/T249 template)
- [x] Targeted: `cargo nextest run -p ai-brains-cli --bin ai-brains -E "test(nightly)"` (crate is bin-only; 71 passed) ; clippy `-p ai-brains-cli`

## Phase 2 — Red → Green: JSON builder (F1 / F5 / F6 / F19 / F20 / F35 / F36 / AC3–AC4 / AC9 / AC14 / AC16)

- [x] `NightlyStatusJson` + `EndpointJson` + `RouterJson` + `MultiImportJson` in `nightly_status.rs`
- [x] Pure builder from already-fetched fields (no HTTP inside the builder)
- [x] Thread raw `Option` for last_count / last_errors (F20/F35); AC3 includes `action_target` + `errors_last_run_unreadable`
- [x] Same `host_port` / model / probe tuple as human (F36); `--quick` fixture `probe: "skipped"`
- [x] `multi_import` never/unreadable/ok (F19)
- [x] Hermetic AC9: `--format json` one object, no `=== Nightly Status ===`; assert **only** `schema_version` / endpoints / `multi_import`
- [x] Hermetic: omitted `--format` still human (AC10)
- [x] AC16: non-Windows / no-scheduler builder → `scheduled` + `router` JSON `null`

## Phase 3 — Red → Green: Router line + Status: parse + foundness (F7–F10 / F8 / F34 / AC5–AC7 / AC15)

- [x] Additive `SchtasksListV.status`; update English fixture unit (AC5)
- [x] `fetch_schedule_snapshot` returns `{ found, snap }` (F34)
- [x] `format_router_status_lines`: running+267009; `found == false` → `not scheduled` (no next:); ONLOGON `found` + no next_run → still scheduled
- [x] AC15: status missing + last_result → `Router: last result: 267009` + following hint
- [x] `nightly.rs` status branch: second snapshot `"AI-Brains-Router"`; print after endpoints; precompute action fields for JSON
- [x] JSON `router.scheduled = found`; non-Windows `null`
- [x] Do **not** run T247 F6 missing-action on Router (F10)

## Phase 4 — Docs (F25 / AC12)

- [x] CAPABILITIES: **one additive** T247 honesty bullet (format + default human + Router)
- [x] OPERATIONS: `--format json` example + “doctor is not the model-port matrix”
- [x] CLI-EXIT-CODES nightly status exit **0** footnote
- [x] Root CHANGELOG T255 row (**must** say piped default stays **human**)
- [ ] conductor + deferred closeout **only at track complete** (not this fold-in commit)

## Phase 5 — Review / gate (F27 / AC11 / AC13)

- [x] Internal review vs spec until clean (mediums fixed or ≤3 justified) — R1 P3s fixed; R2 PASS
- [x] `codex-review` FEATURE — CX1 product **PASS** (0 findings)
- [x] Manual AC11 on **source** bin (no live mutate, no PATH reinstall)
- [x] Full gate: fmt PASS; clippy workspace `-D warnings` PASS; nextest workspace **2962 passed** / 1 skipped; deny/audit not on PATH (T251 residual)
- [x] Mark conductor Complete; append leftover softs to `deferred.md`; pin after gate

---

## Manual AC (record on go)

| AC | Command | Expected |
|----|---------|----------|
| AC10 | `cargo run -q -p ai-brains-cli -- nightly --status` | Human header; Last Result 1; missing `.cmd`; **new** Router 267009; exit 0 |
| AC11 | `… nightly --status --format json` | JSON object; `action_target_missing: true`; `router.last_result: "267009"`; exit 0 |
| AC14 | `… nightly --status --quick --format json` | `completion.probe` / `embedding.probe` == `"skipped"` |
| AC8 live | `… nightly --status --quick` | T247 lines unchanged except additive Router |

Do **not** `schtasks /change`, do **not** write `nightly-run.cmd`.

---

## Stop-before

- Destructive git / push to `main`
- Live task mutate / `.cmd` write / Router register
- Doctor 16th check or contracts DTO
- Embed sleep retune without timings
- T253 nightly Claude/Codex
- PATH `cargo install` unless user asks
