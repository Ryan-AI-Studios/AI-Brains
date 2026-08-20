# T269 Plan — Nightly vs Router status split + probe honesty

**Status:** **Completed**
**Spec:** [spec.md](./spec.md) F0–F27 / AC1–AC13 + §13 AI fold-in
**Category:** OPS / UX / BUGFIX
**Ledger TX (planning):** `7f7f7fd2-5ce1-4892-94d0-451699366dd0` (DOCS)
**Ledger TX (fold-in):** `6c22c5b1-463f-492f-a656-4514742b412f` (DOCS)
**Ledger TX (implement):** `50098557-f967-4281-ab52-bb19c095719c` (BUGFIX)

---

## AI fold-in (2026-08-20) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F1:** heading `println!` is **outside** `#[cfg(windows)]`.
2. **F27 / AC2:** suffix iff `label == "timeout"`; `"TIMEOUT"` / `"timeout-ish"` / `""` pass through.
3. **AC8:** all-OS hermetic in `tests/nightly_status.rs` (not `cfg(windows)`).
4. **AC6:** after_help needles include `TCP` and `/health`.
5. Line counts: `nightly.rs` **2124** total / `nightly_status.rs` **593** total.

---

## Preflight (plan time — 2026-08-20)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `6825343`; fold-in `5bfc088` (docs-only; product tree identical). CLEAN; `main` == `origin/main` |
| T229 / T247 / T255 | ✅ in source: JSON, Router line, 750 ms, `--quick` |
| PATH `ai-brains` | `0.1.1`. `--status --quick`: `Last task result: 0` (no Nightly heading) + `Router: Running  last result: 267009`. Full JSON: `completion.probe=timeout` vs daemon **Open** |
| `AI-Brains-Nightly` | Ready; Last Result **0**; `/tr` quoted `ai-brains.exe nightly` |
| `AI-Brains-Router` | Running; ONLOGON; Last Result **267009**; `/tr` unquoted `C:\llm\router.bat` |
| Last PR comments | #185 T273 — **empty** (N/A). #184 Linux Path already `cfg(windows)` — no T274 |
| Open PR on HEAD | none (Dependabot remotes only) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150**; tokio **1.52.3** — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** — do not grow. `nightly.rs` **2124** total / **1964** non-blank, not top-10 → helpers in `nightly_status.rs` (**593** / **554**) |
| Ledger | 0 pending at scan; planning TX `7f7f7fd2` |
| `ISSUES.md` | **Does not exist** |
| ledgerful search | `NIGHTLY_STATUS_PROBE_TIMEOUT` at `nightly.rs:13` |
| Online | Microsoft Learn `SCHED_S_TASK_RUNNING` 0x41301=267009; llama.cpp #20684 `/health` queue; clap 4.6.6 `after_help` |

---

## Phase 0 — on go (re-verify)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact`
- [x] Re-read `nightly.rs` status branch + `format_endpoint_line` + `NIGHTLY_STATUS_PROBE_TIMEOUT` and `nightly_status.rs` Router/JSON helpers
- [x] Rescan `deferred.md` for new open rows that overlap
- [x] Confirm #185 still empty / no new Cursor leftover that needs a mint
- [x] Re-dogfood `nightly --status --quick` and full `--status` vs `daemon status` (do not mutate tasks)
- [x] Re-check clap lock vs crates.io (**no bump** unless execute proves otherwise)
- [x] BUGFIX TX start `50098557-f967-4281-ab52-bb19c095719c`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Human mixes Nightly 0 with Router 267009 | **DoD** F1 / AC1 / AC8 / AC10 |
| Completion `probe=timeout` vs daemon Open | **DoD** F3 / F4 / AC2 / AC6 / AC10 — **label**, do not raise 750 ms |
| `--quick` still skipped | **DoD** F2 / AC8 / AC13 |
| JSON already split / keys frozen | **Affirm** F5 |
| T255 Router read-only | **Affirm** F6 / F17 |

## Declined (written)

| Item | Why |
|------|-----|
| Raise 750 ms | F4 — llama.cpp #20684; T247 latency |
| JSON `probe_budget_ms` / token suffix | F5 / F21 |
| Daemon TCP → HTTP `/health` | F18 — T199 |
| Doctor 16th / persist probe / embed sleep / `.cmd` / `--no-vault` | T255 F11–F15 |
| T270 / T272 / T273 F7 | Peers |
| last-PR #185 Cursor | N/A empty |
| #184 Linux Path | Already fixed; no T274 |
| clap 5 / pin bumps / DTO / `cargo install` | F14 / F15 / F16 |
| T240 F2 / live schtasks mutate | Standing |

---

## Phase 1 — Red (failing tests first)

- [x] AC1 unit: heading const `Nightly: AI-Brains-Nightly`
- [x] AC2 rstest `#[case]`: timeout → `timeout (750ms)`; skipped/ok/down/error/`""`/`"TIMEOUT"`/`"timeout-ish"` unchanged
- [x] AC6 clap: `nightly --help` contains `AI-Brains-Nightly`, `267009` or `SCHED_S_TASK_RUNNING`, `750`, `TCP`, and `/health`
- [x] Red proven on current tree (heading `""`; timeout identity; after_help missing needles; hermetic AC8 missing heading)

## Phase 2 — Green (helpers + call site)

- [x] `NIGHTLY_TASK_HEADING` + `format_probe_label_human` in `nightly_status.rs`
- [x] Status human: print heading after banner **outside** `#[cfg(windows)]`; wrap Completion/Embedding labels with `NIGHTLY_STATUS_PROBE_TIMEOUT.as_millis()`
- [x] JSON path: raw `as_label` / `"skipped"` unchanged
- [x] Additive Nightly `after_help` (keep T255 format examples)
- [x] Do **not** edit `format_status_schedule_block` / Router formatter / probe Duration consts / `format_endpoint_line` signature

## Phase 3 — Regressions + hermetic

- [x] AC3/AC4/AC5/AC7/AC9/AC13 existing units green
- [x] AC8 all-OS hermetic in `tests/nightly_status.rs` (not `cfg(windows)`): heading + `probe=skipped`, no `(750ms)`
- [x] `cargo nextest run -p ai-brains-cli -E "test(nightly)"` **90 passed** (crate is bins-only; `--lib` is N/A)
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`

## Phase 4 — Docs + manual

- [x] CAPABILITIES additive honesty bullet
- [x] OPERATIONS heading + 750 ms vs TCP
- [x] CHANGELOG T269 row
- [x] AC10 manual: full `--status` heading + 267009 + `probe=timeout (750ms)`; `daemon status` Open; **did not** mutate tasks
- [x] AC11 docs present

## Phase 5 — Review + gate + publish

- [x] Phase-1 review → `review.md` until clean
- [x] Codex read-only (F23) — CX1 FAIL P2 adjacency (fixed); CX2 product **PASS**
- [x] Full gate: `.\scripts\dev-check.ps1` nextest **3217** passed, 1 skipped; `ledgerful verify --scope full` passed (fmt 2.3s / clippy 1.8s / nextest 114.5s / deny 2.5s / audit 2.7s)
- [x] Conductor **Completed**; deferred closeout; pin
- [ ] implement-track Phase 6: push `track/T269-*` → PR → watch GHA `CI` green → squash-merge → prune. Never `git push origin main`

---

## Definition of done

- [x] AC1–AC13 proven (units + hermetic + manual AC10 + docs)
- [x] T247/T255 nightly units still green
- [x] JSON keys / probe tokens / 750 ms / `--quick` skipped unchanged
- [x] No `cargo install`; no schtasks mutate; no `.env`; no contracts DTO
- [x] Medium+ review findings not silently dropped (CX1 P2 adjacency fixed; CX2 PASS)
- [ ] Ledger BUGFIX TX committed; 0 pending / 0 unaudited drift
- [ ] PR merged; local `main` at `origin/main`
