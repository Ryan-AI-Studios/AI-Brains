# T266 Plan — Format policy convergence

**Status:** **In Progress** (implement 2026-08-18; FEATURE TX `9aec5831-b7a6-4eb6-85b8-49168cf7b07a`)
**Spec:** [spec.md](./spec.md) F0–F27 / AC1–AC14 + §13 fold-in
**Category:** UX / FEATURE
**Ledger TX (planning):** `201a3883-3053-4487-ba46-8942565eeae5` (DOCS)
**Ledger TX (fold-in):** `0be907dc-a219-45d7-8f73-19043d4e6775` (DOCS)
**Ledger TX (on go):** `ledgerful ledger start T266-format-policy-convergence --category FEATURE --message "Converge inventory --format onto shared resolver; JSON keys frozen; nightly/graph-update defaults stay"`

---

## AI fold-in (2026-08-18) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. OpenCode **m1–m3** folded as named AC4 human-half, F7 arg docs, AC3 no-filter pin. Agy **O1** folded as **F27**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **AC4:** `list_paths__format_json__api_version_1` **and** `list_paths__format_human__table_not_json`.
2. **AC3:** unfiltered empty vault only.
3. **F7 / AC11:** five `--format` arg docs + CAPABILITIES missing-row list.
4. **F27:** `is_json_output` wrapper; do not change `resolve_human_json_format`.
5. **AC7 / AC14:** JSON/Pretty InvalidValue on whoami/adopt/rebind; non-empty pretty ≡ human.
6. **§2.1:** `4088106` vs `8c3b7e1`.

---

## Preflight (plan time — 2026-08-18)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `4088106`. Plan commit `8c3b7e1`. Fold-in docs on that product src. |
| T266 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | **0.1.1**. list-paths `auto` already present. **Do not `cargo install`.** |
| Live hole | Same non-TTY session: list-paths **124-line JSON**; `--format human` **18-line table**; retention/scope/whoami JSON; nightly/doctor/memory/project-list **human**; PATH `graph update` pretty JSON. Token `pretty` clap-rejects on list-paths. |
| SoT | `format_resolve::resolve_human_json_format`; three `use_json_output` forks; whoami inline match in hotspot `project.rs`; clap `value_parser` `auto\|human\|json` on five inventory commands. |
| clap / serde_json | lock clap **4.6.1** / crates.io **4.6.6**; serde_json lock **1.0.150** / crates.io **1.0.151**. rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute. |
| Last PR Cursor | #179 Bugbot Medium `safety_ids` over-exclude Index. **Still true.** **Minted T272** (not this track). No open PR on `main`. |
| `deferred.md` | Full scan. Overlap: audit T266 **absorb**; shared resolver closeout **absorb**; T227 F34 **decline**; T246 F17 **decline**; T255 F2 **affirm**; T265–T271 except T266 **decline**; T240 F2 / T255 **decline**. |
| ai-brains | `preflight --summary` 3581317d / 3022 pins / grants 0 of 3 (T241). Recall: T248/T249 notes; no T266 pin. |
| ledgerful | doctor ready (hygiene warns). 0 pending 0 drift. Hotspot **#1** `project.rs` — format match only. `#5` `governed_common.rs` — do not edit parse. Index incremental completed. |
| Research | clig.dev human-first + `--json` + TTY heuristic + future-proof human; clap 4.6.1 `PossibleValuesParser` case-sensitive; T180 2-key; T246 F6; T255 F2. |
| `ISSUES.md` | **Does not exist** |
| Live `.env` / bootstrap / nightly mutate / pin | **Not written** / **not run** / **not scheduled** / **not pinned** this pass. |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Format maze; list-paths 7/5; retention 6/5 | audit T266 | **DoD** F1–F7 / AC2–AC5 / AC11 / AC13 |
| Shared resolver extracted 2026-08-16 | T255 closeout | **Absorb** F4 — use it; do not fork again |
| Incomplete CAPABILITIES format table | T204 leftover honesty | **Absorb** F1 / AC11 |
| Missing PROTOCOL-COMPAT list-paths rows | §5 inventory | **Absorb** F17 / AC11 |
| T227 F34 OutputFormat surface-wide | T227 / T263 pointer | **Decline** F11 |
| T246 F17 TTY-auto graph update | T246 soft | **Decline** F8 |
| T255 F2 nightly pipes | T255 | **Affirm** F2 / AC10 |
| T265 envelope | series | **Decline** F12 |
| T267 / T268 / T269 / T270 / T271 | series | **Decline** F13 |
| T240 F2 / T255 bag | standing | **Decline** F14 |
| last-PR Cursor #179 | Bugbot Medium | **Mint T272** F15 |
| OpenCode m1–m3 / Agy m2 / O1 / O2 | review | **Absorb** AC3/AC4/AC7/AC11/AC14 / F7 / F27 |

---

## Phase 0 — on go (re-verify)

- [x] Re-read `format_resolve.rs`, three `use_json_output` sites, whoami match, five clap `value_parser`s.
- [x] Confirm `project list-paths --format pretty` is still clap `InvalidValue` on source bin. (red: 5 pretty-parse tests failed with possible values `auto, human, json`)
- [x] Classify-only dogfood: default list-paths still JSON on this agent; `--format human` still a table. **Do not** pin. **Do not** `cargo install`.
- [x] Re-check lock clap **4.6.1** / crates.io **4.6.6**. rustc **1.95.0**. No clap 5.
- [x] Rescan **entire** `conductor/deferred.md`.
- [x] Last merged PR Cursor comments — #179 leftover already **T272**. No new leftover.
- [x] `ledgerful ledger start T266-format-policy-convergence --category FEATURE` → `9aec5831-b7a6-4eb6-85b8-49168cf7b07a`

---

## Phase 1 — Red

- [x] `list_paths__format_pretty__human_empty_copy` (AC3) — **no** `--project` / `--shared-only` (filtered empty is `No path aliases match.`)
- [x] `list_paths__format_json__api_version_1` (AC4 json half) — extend existing json hermetic if needed
- [x] `list_paths__format_human__table_not_json` (AC4 human half — required; do not skip)
- [x] `list_paths__format_pretty__table_not_json` (AC14 non-empty pretty ≡ human)
- [x] `scan_roots__format_pretty__not_json` (AC5)
- [x] Clap units in `main.rs` (AC2 / AC7): list-paths + scan-roots `pretty` parses and `xml` / `JSON` / `Pretty` InvalidValue; whoami / adopt-path / rebind-path `pretty` parses **and** `JSON` / `Pretty` InvalidValue
- [x] `cargo nextest run -p ai-brains-cli` — new tests **fail** (pretty not in parser) — 5 pretty-parse FAIL, InvalidValue already PASS
- [x] Commit red allowed (bundled with green on this implement)

---

## Phase 2 — Green (resolver + tokens)

- [x] Expand five `value_parser` lists to T248/T249 set (F3)
- [x] `format_resolve.rs`: add `is_json_output` (F27) + one unit; do not change `resolve_human_json_format`
- [x] `project_paths.rs` / `project_adopt.rs` / `project_rebind.rs`: delete `use_json_output`; call `is_json_output` (F4 / AC6)
- [x] `project.rs` whoami: replace format `match` only with `is_json_output` (F4 / §5.2). Do not touch `display_label` / detect / report struct
- [x] Do **not** edit `governed_common.rs` `OutputFormat::parse`
- [x] Do **not** edit `graph.rs` / nightly default / recall / retrieval
- [x] Targeted: `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` ; `cargo nextest run -p ai-brains-cli` — clippy 0; 23 clap/resolver + 67 hermetic keep-green
- [x] AC1 / AC8 / AC9 / AC10 still green
- [x] Commit green allowed

---

## Phase 3 — Docs

- [x] CAPABILITIES OutputFormat table = Families A–D; nightly named Family B (F1 / F2 / AC11). **Add missing rows:** list-paths, scan-roots, whoami, adopt-path, rebind-path, graph neighbors/update, nightly, harness status, memory list, project list
- [x] PROTOCOL-COMPAT additive list-paths + scan-roots rows (F17)
- [x] list-paths + scan-roots `after_help`: TTY table / pipe JSON; `--format human` for agents (F7)
- [x] Five clap `--format` arg docs name `auto | pretty | human | text | json | markdown | md` (F7). Add the missing docstring on `ScanRoots`
- [x] Root CHANGELOG T266 row
- [x] Do **not** reorder T204 Start-here groups (F26)

---

## Phase 4 — Review + gate (on go)

- [x] Internal review → `review.md`
- [x] Medium+ not silently dropped
- [x] `codex-review` (F19) — product PASS; process P1 closeout now done
- [x] Manual AC13 (source bin)
- [x] Full gate: `cargo fmt --check` ; clippy workspace `-D warnings` ; nextest workspace ; `cargo deny check` ; `cargo audit` ; `ledgerful verify --scope full` — nextest 3126 + verify passed
- [x] Conductor T266 → Completed in this commit; publish is Phase 6 (push → PR → GHA → squash).

---

## Definition of done

- [x] AC1–AC14 pass with evidence
- [x] F0–F27 honored (declines written)
- [x] No product commits under this planning TX
- [x] T272 placeholder exists for #179 (not silently dropped)
- [x] `conductor/ISSUES.md` not created

---

## Stop-before

- Scope exceeds F1–F7 (graph-update flip, nightly pipe flip, F34 parse, T265 envelope, T272 retrieval)
- Destructive git / push to `main`
- Missing clap/docs pin that would force clap 5
- Broad unrelated failures
