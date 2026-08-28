# T319 Plan — handle vs memory id namespace

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** UX / HONESTY
**Ledger (planning):** DOCS `844bdbed-7295-4635-a04f-968d224e41ec`

---

## Preflight (plan time — 2026-08-28)

| Check | Result |
|-------|--------|
| HEAD / tree | `fa353c7` T317 `#234` CLEAN. Branch `track/T319-handle-vs-memory-id`. `origin/main` even (ahead **0**). |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T263 overlay **on PATH**. T314 expand `--format` **not**. T319 hole **is**. |
| `preflight --summary` (PATH) | Pinned **4554**; in-context **0/0/0**; `Total Word Count: 740` (PATH-behind T315) |
| `evidence show 431f6505-… --format json` | `kind=Unknown` `preview=""` exit **0** |
| `query expand 431f6505-…` (PATH JSON) | `kind=Unknown` `preview="Handle not found."` exit **0** — **same as random UUID** |
| `source show 431f6505-… --format json` | `NOT_FOUND` `source {id}` exit **4** |
| `evidence list` / `source list` | T290 empty + `Ungoverned vault search:` — stay-green |
| `expand_handle` | `query.rs:538–669` — **do not edit** |
| `memory_exists` | `query_store.rs:735` |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; serde_json **1.0.150**; uuid ws `"1.13"` / lock **1.23.1**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#234` `mergedAt` **2026-08-28T23:23:15Z**; comments/reviews **[]** — **N/A empty**. `#230` → **T325** already. |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan; this TX `844bdbed` |
| Hotspots | `project.rs` #1 **3.715** / `sync.rs` #2 **3.519** — do not touch. `governed_common.rs` #3 **3.389** **1029** lines — sibling module. |
| `ISSUES.md` | **Does not exist** |
| Planning install / live pin / migrate | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit evidence/source show vault UUID | **DoD** F1–F8 / AC5–AC8 / AC11 |
| Audit query expand same hole | **DoD** F3 / AC5–AC6 |
| Evidence empty Unknown preview | **F2** T263 overlay on evidence |
| T263 Handle not found / exit 0 | **Stay-green** F11 / AC9 |
| T290 empty lists | **Not stolen** F12 |
| T263 H2 | **Decline** F9 |
| last-PR `#234` Cursor | **N/A empty** F19 |
| last-PR `#230` F8 recency | **T325** — not stolen |
| T316 / T317–T318 / T320–T325 / clap 5 | **Not stolen** / **Decline** |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains`
- [ ] Re-read `expand_handle` `:538–669` (must still return Unknown empty — do not edit)
- [ ] Re-read `run_expand` `:174–243` + `apply_unknown_expand_preview` `:68–86`
- [ ] Re-read evidence `run_show_local` `:112–164` + daemon `:167–201`
- [ ] Re-read source `run_show_local` `:110–169` + `fail_api` NOT_FOUND
- [ ] Confirm `memory_exists` still `query_store.rs:735` and `graph.rs` `vault_memory_present` still feature-gated
- [ ] Re-read T263 AC10 `governed_vault_pin_honesty.rs:220–255`
- [ ] Re-read clap Evidence/Source Show + Expand after_help
- [ ] Re-dogfood `431f6505-50d7-5176-8cda-f8ba2534fe14` on evidence/expand/source (record preview/exit)
- [ ] Confirm clap lock still **4.6.1**; `HandlePreviewDto` still has no `next_step`
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] Confirm T325 placeholder still Pending (do not steal F8 recency)
- [ ] `ledgerful ledger start T319-handle-vs-memory-id --category FEATURE`
- [ ] **Do not** `cargo install` / live `policy bootstrap` / `migrate governed` / live production `pin` / `.env` rewrite / clap 5

## Phase 1 — Red

- [ ] `apply_unknown_handle_overlay__memory_exists__preview_and_next_step` (AC1)
- [ ] `apply_unknown_handle_overlay__unknown_unknown__handle_not_found_no_next` (AC2)
- [ ] `apply_unknown_handle_overlay__non_unknown__unchanged` (AC3)
- [ ] `wrong_namespace_source_hint__contains_preview_and_next` (AC4)
- [ ] Hermetic AC5 `query_expand__memory_id__json_names_namespace` — seed pin + grants; id from `memory list --format json` (AC17)
- [ ] Hermetic AC6 `query_expand__memory_id__human_three_lines`
- [ ] Hermetic AC7 `evidence_show__memory_id__names_namespace`
- [ ] Hermetic AC8 `source_show__memory_id__not_found_hint_exit_4`
- [ ] Confirm red tests **fail** on current tree (expand preview still `Handle not found.` / evidence preview empty / source no hint)
- [ ] Confirm T263 AC10 **still passes** (stay-green on this commit)

## Phase 2 — Green

- [ ] Add `governed_namespace.rs` + `mod.rs` (`F14`)
- [ ] EXISTS-result mapper (never `?`; F1) — do **not** import `graph.rs`
- [ ] `apply_unknown_handle_overlay` replace-not-stack (F6)
- [ ] Wire `run_expand` (keep `applied_scope`; F3 exit 0; F13 three-line human)
- [ ] Wire evidence local **and** daemon emit via `Value` (F2 / F30)
- [ ] Wire source local **and** daemon NOT_FOUND `with_details` (F4)
- [ ] Production net: sibling + call sites; do not grow `governed_common.rs` / `project.rs` / `sync.rs`

## Phase 3 — Stay-green + docs

- [ ] AC9 T263 two-line expand unknown-unknown
- [ ] AC14 T290 list empty / T221 Denied / T314 `JSON` InvalidValue
- [ ] AC15 found-kind unit (AC3) — no H2 evidence seed
- [ ] AC10 CAPABILITIES `:346` ; PROTOCOL-COMPAT expand `:112` + **add** evidence show + source show rows ; OPERATIONS ; CLI-EXIT-CODES footnote ; CHANGELOG ; after_help F26
- [ ] AC13 empty diff: `query.rs` `expand_handle`, `HandlePreviewDto`, `project.rs`, `sync.rs`

## Phase 4 — Manual + gate

- [ ] AC11 `cargo run -p ai-brains-cli -- evidence show 431f6505-50d7-5176-8cda-f8ba2534fe14 --format human` (PATH-behind not a fail)
- [ ] AC11 `cargo run -p ai-brains-cli -- query expand 431f6505-… --format json`
- [ ] AC11 `cargo run -p ai-brains-cli -- source show 431f6505-… --format json` still exit 4 + hint
- [ ] AC12 random UUID still T263-only
- [ ] `cargo fmt --check` ; `clippy -p ai-brains-cli --all-targets -- -D warnings` ; targeted nextest
- [ ] Phase-1 review → `review.md` ; FEATURE cross-model `codex-review` (F24)
- [ ] Full gate before Complete; `ledgerful verify --scope full`
- [ ] conductor **Completed** only after squash-merge hygiene (implement-track Phase 6). Never `git push origin main`.

---

## Definition of Done (checkable)

- [ ] AC1–AC8 red-then-green; AC5–AC8 hermetic seed uses `memory list` JSON id
- [ ] AC9 / AC14 / AC15 stay-green
- [ ] AC10 docs + after_help dual-truth
- [ ] AC13 control-plane `expand_handle` and DTO untouched
- [ ] AC11–AC12 manual recorded
- [ ] Exit 0 Unknown (expand/evidence) and exit 4 source miss unchanged
- [ ] No H2 / no `kind: "Memory"` / no new clap flag / no clap 5
- [ ] Status stays **Pending** until implement Complete + PR merge

---

## Isolation (on go)

No `cargo install`. No live bootstrap / migrate / production pin. No `governed_common.rs` hotspot growth. No `project.rs` / `sync.rs`. No `git push origin main`.
