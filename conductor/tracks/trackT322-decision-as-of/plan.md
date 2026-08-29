# T322 Plan — `decision in-force --as-of`

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** FEATURE
**Ledger (planning):** DOCS `d8e6e556-cfb8-4cd6-84cc-3f5b1599532c`

---

## Preflight (plan time — 2026-08-29)

| Check | Result |
|-------|--------|
| HEAD / tree | Product `0eef80b` T321 `#243`. Branch `track/T322-decision-as-of`. `origin/main` = `0eef80b`. Working tree had uncommitted T321 conductor Completed + residuals — included in this DOCS commit. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T311 **on PATH**. T312–T321 **not**. Hole **is** (no `--as-of`). |
| `preflight --summary` (PATH) | Pinned **4630**; in-context **0/0/0**; `Total Word Count: 728` (PATH-behind T315). |
| PATH `decision in-force --help` | TERM / `--scope` / `--format` json; **no** `--as-of` |
| PATH `decision in-force workspace_id --format json` | `ruling: null`, `chain: []`, exit 0 |
| PATH `--as-of 2026-01-01T00:00:00Z` | clap unexpected argument exit **2** |
| Projector | `DecisionSuperseded` overwrites `updated_at`; does **not** close `valid_until`; `DecisionApprovedPayload.approved_at` **unprojected** |
| `decision_valid_at` | `briefings/project.rs:695` already takes `at` — **do not edit** |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; time `"0.3"` / lock **0.3.47** / crates.io **0.3.55**; rusqlite **0.40.2**; serde_json **1.0.150**; uuid ws `"1.13"` / lock **1.23.1**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#243` empty. `#237` → **T326**. `#230` → **T325**. **No T327.** |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX) |
| Hotspots | `project.rs` #1 **3.640** — do not touch. `governed_common.rs` #3 — do not grow. `in_force.rs` not top 10. |
| Line counts | `in_force.rs` **204** physical; `decision.rs` **299**; CP tests **267**; CLI tests **252**. F22 = go-HEAD diff. |
| `ISSUES.md` | **Does not exist** |
| Planning install / live propose | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| T311 R2 `--as-of` | **DoD** F1–F7 / AC1–AC7 / AC11 / AC14 / AC15 |
| T311 R4 `approved_at` column | **Decline** F9 — hop-stop on superseded/revoked `updated_at` |
| T311 R1 daemon | **Decline** F11 |
| T311 R3 sibling Approved | **Decline** F8 / T311 F7 |
| T311 R5 / R7 | **Not stolen** T323 / T324 |
| last-PR `#243` | **N/A empty** |
| last-PR `#237` / `#230` | **T326** / **T325** — not stolen |
| T321 residuals / T325 / T326 / clap 5 | **Not stolen** / **Decline** |
| T321 uncommitted conductor note | **This DOCS commit** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains`
- [ ] Re-read `in_force.rs` `resolve_in_force` `:52–96` + `walk_chain` `:148–189` + `ruling_from_row`
- [ ] Re-read clap `DecisionCommands::InForce` `main.rs:2925–2944` + dispatch `:5002–5015`
- [ ] Re-read `decision_valid_at` `briefings/project.rs:695–698` (do **not** edit)
- [ ] Re-read projector `DecisionApproved` / `DecisionSuperseded` / `DecisionRevoked` (`store/.../decision.rs:83–116`)
- [ ] Re-read T311 tests `control-plane/tests/in_force.rs` + `cli/tests/decision_in_force.rs`
- [ ] Re-dogfood `decision in-force --help` + `decision in-force workspace_id --format json` + `--as-of` still unknown until green
- [ ] Confirm clap lock still **4.6.1**; time lock **0.3.47**; T323 / T324 / T325 / T326 still Pending (do not steal)
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] `ledgerful ledger start T322-decision-as-of --category FEATURE`
- [ ] **Do not** `cargo install` / live `decision propose` / approve / supersede / `.env` rewrite / clap 5 / grow `governed_common.rs` / `project.rs` / projector

## Phase 1 — Red

- [ ] `decision_in_force_help__after_help__names_as_of` (AC1) — must **fail** today
- [ ] `resolve_in_force_at__as_of_before_supersede__prior_approved` (AC3) — must **fail** (`resolve_in_force_at` missing)
- [ ] `parse_as_of_rfc3339__date_only__err` + `parse_as_of_rfc3339__zulu__ok` (AC6) — must **fail** (helper absent)
- [ ] Confirm T311 stay-green tests still **pass** on this red commit (AC8 / AC9) — 4-arg untouched

## Phase 2 — Green (CP)

- [ ] `resolve_in_force_at` + 4-arg wrapper (F1)
- [ ] F5 hop-stop in `walk_chain`
- [ ] F6 Some-path ruling (Approved / Superseded-stopped / Revoked-before)
- [ ] Additive JSON `as_of` skip_serializing_if (F4)
- [ ] AC3 / AC4 / AC5 / AC11 / AC15
- [ ] `pub use resolve_in_force_at`

## Phase 3 — Green (CLI)

- [ ] `parse_as_of_rfc3339` in `decision.rs` (F29)
- [ ] clap `--as-of` + dispatch + InForce after_help (F2 / F23)
- [ ] `run_in_force` passes `Option<OffsetDateTime>`
- [ ] Human `As of:` line when set (F12)
- [ ] AC1 / AC2 / AC6 / AC7 / AC10
- [ ] Stay-green AC8

## Phase 4 — Docs

- [ ] CHANGELOG Unreleased Added
- [ ] CAPABILITIES Family C row names `--as-of` (AC13)
- [ ] OPERATIONS one example (AC13)

## Phase 5 — Targeted gate

- [ ] `cargo clippy -p ai-brains-control-plane -p ai-brains-cli --all-targets -- -D warnings` (AC12)
- [ ] nextest those packages + new tests
- [ ] Manual AC14 (`cargo run` help + live as-of 2020 → `ruling: null`) — **no** live propose
- [ ] Implement-track full gate + PR + GHA + squash (never `git push origin main`)

## DoD (after go)

- [ ] AC1–AC15
- [ ] T311 4-arg / JSON-without-`as_of` / deny exit 3 / empty term exit 2 stay-green
- [ ] No `approved_at` column; no projector `valid_until` close
- [ ] T323 / T324 / T325 / T326 / T307 / H2 / clap 5 **not stolen**
- [ ] Medium+ review findings not silently dropped

## Isolation

No daemon DTO. No H2. No `cargo install`. No live vault lifecycle writes. Never `git push origin main`.
