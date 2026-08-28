# T313 Plan — `sync query` rescued heading

**Status:** **Implementing** (FEATURE `a58ee509`). Spec [spec.md](./spec.md).
**Category:** UX / HONESTY
**Ledger (planning):** DOCS `bdf8fddd-84f9-4d9d-9b7d-64887dd834e2`
**Ledger (fold-in):** DOCS `5fa5626e-ce2f-42df-97f4-744053ba09a5`
**Ledger (implement):** FEATURE `a58ee509-ed84-420b-9fd0-c4112782289d`

---

## Preflight (plan time — 2026-08-28)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `2bec83e` plan commit CLEAN; `origin/main` = `cd7bfde` (ahead **1**). Plan-write was `cd7bfde` / ahead **0** (Agy m1). Product `src/` = T314 `#232`. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T271 banner **is** on PATH. T312/T315/T314 **not**. |
| `preflight --summary` (PATH) | Pinned **4544**; in-context **0/0/0**; `Total Word Count: 737` (PATH-behind T315) |
| Phrase `"graph backend"` | `ledgerful ledger search --json -- "graph backend"` → `[]` |
| Token `graph` | JSON hits (~10 KB) |
| Phrase `T314` | JSON **3** hits (AC12 control) |
| PATH `sync query "graph backend" --limit 3 --quiet` | Generic heading + F7 banner + `10 matching entries for 'graph':` |
| `print_ledger` | `sync.rs:563–571` hardcoded `--- Ledgerful Ledger Search ---` |
| Rescue arm | `sync_query_ledger.rs:307–312` sets `banner` only |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#232` `mergedAt` **2026-08-28T11:05:01Z**; comments/reviews **[]** — **N/A empty**. `#230` → **T325** already. |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan |
| Hotspots | `project.rs` #1 / `sync.rs` #2 (3.420) — do not grow / `governed_common.rs` #3 |
| `ISSUES.md` | **Does not exist** |
| Planning install / live pin | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit phrase→fuzzy opacity | **DoD** F1 / AC1–AC3 / AC8 / AC11 |
| T271 F7 looks like a phrase hit | **Heading** F1; banner exact F2 / AC4 |
| T271 F6 first-seen | **Affirm** F4 — no scoring |
| T273 `--` | **Affirm** F6 / AC5 |
| T231 always-pretty / ndjson | **Affirm** F7 / AC14 |
| T124 `--no-bridge` | **Affirm** AC6 |
| last-PR `#232` Cursor | **N/A empty** F20 |
| last-PR `#230` F8 recency | **T325** — not stolen |
| T312 / T314 / T315 / T316–T324 / clap 5 | **Not stolen** / **Decline** |
| OpenCode m1 / Agy m2 heading `trim()` | **F1 / F25 / AC3** `Some("   ")` |
| OpenCode m2 WORKFLOWS.md `:316` | **F14 / AC10** |
| OpenCode m3 AC13 out-of-repo path | **AC13** in-repo name-only |
| OpenCode m4 ndjson heading guard | **AC14** new Phase 1 green-on-arrival |
| OpenCode O2 three `println!` | **§5.2 SoT** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [x] Confirm cwd `C:\dev\AI-Brains` (not Helping Hands)
- [x] Re-read `probe_ledger_search` rescue arm + `LedgerProbeResult`
- [x] Re-read `sync.rs` `print_ledger` `:563–571` + T211 ledger-first `:573–587` (call sites `:576` / `:585`)
- [x] Confirm `SyncCommands::Query` still `:3629–3647` (enum `:3590`)
- [x] Re-read `ledger_rescue_banner` + unit `:663–667`
- [x] Confirm Ledgerful `search.rs` still phrase-wraps
- [x] Re-dogfood `sync query "graph backend"` — generic heading still the hole (pre-green)
- [x] Confirm `T314` (or Phase 0 pick) is still a phrase hit for AC12
- [x] Rescan `deferred.md` open overlapping rows
- [x] Confirm T325 placeholder still Pending (do not steal F8 recency)
- [x] `ledgerful ledger start T313-sync-query-provenance --category FEATURE`
- [x] **Do not** `cargo install` / live production `pin` / `.env` rewrite / Ledgerful edits in Phase 0

## Phase 1 — Red

- [x] `ledger_section_heading__rescued_token__names_token` (AC1)
- [x] `ledger_section_heading__phrase_hit__generic` (AC2)
- [x] `ledger_section_heading__empty_token__generic` (AC3 — `Some("")` **and** `Some("   ")`)
- [x] `format_ledger_section_lines__rescued__heading_then_banner` (AC8)
- [x] Confirm those tests **fail** while heading is hardcoded generic / helper missing
- [x] **Write AC14** `sync_query__format_ndjson__no_ledger_heading` (green-on-arrival — passes on HEAD; OpenCode m4: do **not** skip as stay-green)
- [x] AC4 T271 banner + AC5 T273 argv + AC6 T124 no-bridge are **stay-green** (not Phase-1 red)

## Phase 2 — Green

- [x] F1 `ledger_section_heading` with `!tok.trim().is_empty()` + F10 `format_ledger_section_lines` (AC8) / `print_ledger_section` (**three `println!`**, not `join`)
- [x] F3 `rescued_token: Option<String>` on `LedgerProbeResult`; F6 hit arm `Some(token.clone())`; other arms `None`
- [x] `sync.rs` delete closure; call `print_ledger_section` (file must not grow)
- [x] Match today’s spacing (heading `println!("\n{…}")` then banner then display)
- [x] Do **not** edit `project.rs` / retrieval / contracts / Ledgerful
- [x] Do **not** change F7 banner string
- [x] Do **not** pass `--limit` to ledger argv
- [x] Do **not** add `--format json` combined envelope

## Phase 3 — Stay-green + docs

- [x] AC4 banner exact
- [x] AC5 T273 argv
- [x] AC6 T124 `--no-bridge`
- [x] AC7 isolation vault header
- [x] AC14 (now written) ndjson no ledger heading without `--no-bridge`
- [x] `ledger_json_non_empty` units
- [x] CAPABILITIES pane bullet + OPERATIONS two-section sentence + **WORKFLOWS.md `:316`** (AC10)
- [x] CHANGELOG Unreleased T313
- [x] No PROTOCOL-COMPAT DTO row

## Phase 4 — Manual + gate

- [x] AC11 `cargo run` `sync query "graph backend"` rescued heading + F7 + `matching entries for 'graph'` (PATH-behind not a fail)
- [x] AC12 phrase-hit control (`T314` or Phase 0 pick) generic heading, no banner
- [x] AC13 `git diff --name-only -- crates/` allow-list (`sync_query_ledger.rs` / `sync.rs` / optional `tests/smoke.rs`); **not** `C:\dev\Ledgerful`
- [x] Review log `review.md`; medium+ not dropped
- [x] Cross-model optional (F21) — Codex `review.codex.md` PASS (product); finalize pending
- [x] Full gate: `cargo fmt --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run --workspace` ; `cargo deny check` ; `cargo audit` ; `ledgerful verify --scope full`
- [x] Conductor **Completed**; `deferred.md` closeout row; FEATURE TX commit
- [ ] implement-track Phase 6: push `track/T313-*`, PR, watch GHA `CI` green, squash-merge, prune. Never `git push origin main`.

## DoD (checkable)

- [x] Rescued pane heading is `--- Ledgerful Ledger Search (rescued token: '<tok>') ---`
- [x] Phrase-hit / miss heading stays `--- Ledgerful Ledger Search ---`
- [x] F7 banner sentence exact
- [x] Rescue still runs (F6 first-seen, cap 3)
- [x] `--no-bridge` still skips the ledger section
- [x] ndjson still has no ledger pane (**AC14 named test exists**)
- [x] `sync.rs` did not grow (print extracted)
- [x] Status **Completed** after full gate (Phase 6 merge hygiene follows)

## Manual evidence

```text
# AC11
cargo run -q -p ai-brains-cli -- sync query "graph backend" --limit 3 --quiet --no-project-context
→ --- Ledgerful Ledger Search (rescued token: 'graph') ---
  Note: no phrase match for 'graph backend'; showing hits for 'graph'.
  10 matching entries for 'graph':

# AC12
cargo run -q -p ai-brains-cli -- sync query "T314" --limit 3 --quiet --no-project-context
→ --- Ledgerful Ledger Search ---
  3 matching entries for 'T314':
  (no F7 banner)

# AC13
git diff --name-only -- crates/
→ crates/ai-brains-cli/src/commands/sync.rs
  crates/ai-brains-cli/src/commands/sync_query_ledger.rs
  crates/ai-brains-cli/tests/smoke.rs
```

## Isolation

Do not grow `sync.rs` / `project.rs`. No `cargo install`. Never `git push origin main`.
