# T255 Plan — Nightly / router soft residuals (T229+ / T247+)

**Status:** 📋 **Planning** (plan-only until **go**)
**Spec:** [spec.md](./spec.md) F0–F33 / AC1–AC14
**Category:** OPS / POLISH / UX
**Ledger TX (planning):** `5d3d182d-f689-4673-9a03-733f5a178f3c` (DOCS)
**Ledger TX (implement):** open on **go** (`ledgerful ledger start T255-nightly-router-soft-residuals --category FEATURE`)

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

---

## Architecture SoT (files on go)

| Area | Path | Today | T255 change |
|------|------|-------|-------------|
| clap | `main.rs` `Commands::Nightly` | `--status` / `--quick` | **+ `--format`** requires status, default `human` |
| Dispatch | `main.rs` ~3713 | `nightly::run(…, quick)` | Thread `format: String` |
| Status branch | `nightly.rs` `if status` | Human only | Call `nightly_status`; additive Router lines; JSON emit |
| **New** | `commands/nightly_status.rs` | — | resolver + JSON types + Router formatter + units |
| LIST /V | `SchtasksListV` in `nightly.rs` | 4 fields | Additive `status: Option<String>` (F8). Keep parser here (T247 tests) |
| Fetch | `fetch_schedule_snapshot(task_name)` | Nightly only | Second call `"AI-Brains-Router"` |
| Multi-import | `multi_import.rs` | print human | Reuse view/report in JSON (no schema fork) |
| Doctor / embeddings / models | — | 15 checks / 50 ms / probe | **No** |
| Docs | CAPABILITIES / OPERATIONS / CLI-EXIT-CODES / CHANGELOG | T247 human | F25 |

---

## Phase 0 — Ledger + impact (on go)

- [ ] `ledgerful ledger status --compact`
- [ ] `ledgerful ledger start T255-nightly-router-soft-residuals --category FEATURE --message "JSON nightly --status + read-only Router line; no doctor/embed/wrapper"`
- [ ] `ledgerful scan --impact` — expect `nightly.rs` / `main.rs` / new `nightly_status.rs`
- [ ] Confirm `embeddings.rs` and `doctor.rs` are **not** in the touch set

## Phase 1 — Red → Green: format clap + resolver (F3 / F4 / AC1–AC2)

- [ ] Units in `nightly_status.rs` for `resolve_nightly_status_format` (AC1)
- [ ] clap `--format` on `Nightly`: default `human`, `requires = "status"`, conflicts schedule/unschedule, T248 token set
- [ ] Clap tests next to existing `nightly_quick__*` : no-status, `xml`, `JSON`
- [ ] Targeted: `cargo nextest run -p ai-brains-cli --lib nightly` ; clippy `-p ai-brains-cli`

## Phase 2 — Red → Green: JSON builder (F1 / F5 / F6 / F19 / F20 / AC3–AC4 / AC9 / AC14)

- [ ] `NightlyStatusJson` + `EndpointJson` + `RouterJson` + `MultiImportJson` in `nightly_status.rs`
- [ ] Pure builder from already-fetched fields (no HTTP inside the builder)
- [ ] `--quick` fixture `probe: "skipped"`
- [ ] `errors_last_run` parse (F20); `multi_import` never/unreadable/ok (F19)
- [ ] Hermetic: `--format json` one object, no `=== Nightly Status ===`, exit 0
- [ ] Hermetic: omitted `--format` still human (AC10)

## Phase 3 — Red → Green: Router line + Status: parse (F7–F10 / F8 / AC5–AC7)

- [ ] Additive `SchtasksListV.status`; update English fixture unit (AC5)
- [ ] `format_router_status_lines` : running+267009; missing → `not scheduled` (no next:)
- [ ] `nightly.rs` status branch: second `fetch_schedule_snapshot("AI-Brains-Router")`; print after endpoints
- [ ] JSON `router` object / `scheduled: false` / non-Windows `null`
- [ ] Do **not** run T247 F6 missing-action on Router (F10)

## Phase 4 — Docs (F25 / AC12)

- [ ] CAPABILITIES T247 honesty + `--format json` + default human + Router read-only
- [ ] OPERATIONS examples + “doctor is not the model-port matrix”
- [ ] CLI-EXIT-CODES nightly status exit **0** footnote
- [ ] Root CHANGELOG T255 row (note: piped default stays **human**, not a silent JSON break)
- [ ] conductor + deferred closeout **only at track complete** (not this planning commit)

## Phase 5 — Review / gate (F27 / AC11 / AC13)

- [ ] Internal review vs spec until clean (mediums fixed or ≤3 justified)
- [ ] `codex-review` FEATURE
- [ ] Manual AC11 on **source** bin (no live mutate, no PATH reinstall)
- [ ] Full gate: fmt ; clippy `-D warnings` ; nextest workspace ; deny ; audit ; `ledgerful verify --scope full`
- [ ] Mark conductor Complete; append leftover softs to `deferred.md`; pin decisions

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
