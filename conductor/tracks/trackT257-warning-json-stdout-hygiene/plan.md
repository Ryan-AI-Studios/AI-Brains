# T257 Plan — Warning + JSON stdout hygiene

**Status:** **Completed** 2026-08-17 (FEATURE TX `d086c5f3-6918-49e6-a1fd-377a743ee7fc`)
**Spec:** [spec.md](./spec.md) F0–F26 / AC1–AC17 + §13 fold-in
**Category:** UX / CONTRACTS-adjacent
**Ledger TX (planning):** `b033f134-fb4a-4eb4-bf07-b46087a83a71` (DOCS)
**Ledger TX (fold-in):** `886450fe-3c49-4b44-9073-f5d598297d5a` (DOCS)
**Ledger TX (implement):** start **FEATURE** on **go** only

---

## AI fold-in (2026-08-17) — `agy-review.md` only

No Blockers / Majors. One informational SHA note folded. Dual-site inject and `print_json_stdout` location already in F3/F8/F24 — tightened. Disposition in spec **§13**.

### Pins locked by fold-in

1. **§2.1:** plan dogfood `ed329b1` vs fold `2b3f859`; product src unchanged.
2. **F24 / AC17:** `run_resolve_local` **and** `run_resolve_daemon` call the shared inject helper before `emit_json`.
3. **F8 / F11:** `print_json_stdout` lives in `identity_warn.rs` (not `format_resolve.rs`).

---

## Preflight (plan time — 2026-08-17)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `ed329b1`. Fold-in `2b3f859`. Product src unchanged. CLEAN. `main` ahead of origin by **1**. |
| T257 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` (2026-08-16 08:04). Split: warn **stderr only**; JSON stdout **parses**. `2>&1` concat **fails** (`Additional text… W`). PATH whoami remediations pre-T258. |
| Source / debug | `target\debug\ai-brains.exe` (2026-08-16 22:26). Remediations name **adopt-path**. Same stderr SOOT. |
| Warn site | `project.rs:332` `eprintln!` + `Once`. `main.rs:3275` **before** command match. `emit_json` = `to_string_pretty` + one `println!`. |
| Scope `warnings[]` | **Empty** on live `scope resolve --format json` while mismatch is true. |
| Doctor JSON `2>&1` | **Parses** (early-route; no warn). |
| clap / serde_json | lock clap **4.6.1** / builder **4.6.0** / crates.io **4.6.6**; serde_json lock **1.0.150** / crates.io **1.0.151**. **No clap 5.** Snapshot — re-verify at execute. |
| rustc / workspace | 1.95.0 / **0.1.1** |
| Last PR Cursor | #172 empty comments/reviews/inline/issue; HEAD `main`; Dependabot only. **N/A.** |
| `deferred.md` | Full scan. Overlap: audit warn/JSON (**absorb**); T240 AC4 (**keep**); T258/T259/T266/T265/T267 (**point**); T223/T242 (**decline**); T255 declines stay closed. |
| ai-brains | `preflight --summary` ok (wrong Scope — T258 live rebind out of band). Recall: T240 once/process; T258 adopt-path. No JSON-hygiene pin. |
| ledgerful | doctor ready (hygiene warns). 0 pending at start. Hotspot **#1** `project.rs` **1549** — new file `identity_warn.rs`. |
| Research | clig.dev stdout=data / stderr=messaging (fetched). Mid-object interleave **not reproduced**; merge is JSON-then-Warning. |
| `ISSUES.md` | **Does not exist** |
| Live `.env` / leftover paths | **Not written** / **not rebound** this pass. |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Identity warn / JSON interleave | T256–T271 audit T257 | **DoD** F1–F8 / AC3–AC9 / AC14 |
| T240 once/process + F3b | T240 F3 | **Keep** F6; extend skip F5 / AC1 |
| T240 list hermetic SOOT | T240 AC4 | **Keep** F7 / AC4 |
| T259 “T257 owns JSON interleave” | T259 closeout | **Absorb** identity/`2>&1` half; rebind no-owner JSON stays T259 |
| T258 daily Scope | T258 | **Not absorbed** F14 |
| T259 leftover | T259 Completed | **Not absorbed** |
| T266 format maze | T266 | **Not absorbed** F16 |
| T265 preflight envelope | T265 | **Not absorbed** |
| T267 list footer | T267 | **Not absorbed** |
| T223/T242 env-override | T223/T242 | **Declined** F17 |
| T255 declined bag | T255 | **Stay declined** |
| clap 5 / pin bumps | series | **Not absorbed** F10 |
| last-PR Cursor | #172 | **N/A** — no leftover to mint |
| Live `.env` rebind | T258 | **Stop-before** F14 |

---

## Phase 0 — on go (re-verify)

- [x] Re-read `maybe_warn_identity_mismatch` (`project.rs`) and the `:3275` call. Confirm still `eprintln!` before the command `match` (or note drift). **Confirmed** `eprintln!` + `Once` at `project.rs:332`; `main.rs:3275` before `match`.
- [x] Re-read `emit_json` — still `to_string_pretty` + one `println!`. **Confirmed** (`governed_common.rs:197`).
- [x] Re-read `scope.rs` **both** `emit_json(&wire)` sites (`run_resolve_local` + `run_resolve_daemon`). Confirm line numbers vs F24 (or note drift). **Confirmed** local `:94` / daemon `:128`.
- [x] Re-read `handle_cli_result` — still the Ok/Err join for `run` and `run_sync_path_free`. **Confirmed** `:2868`.
- [x] Confirm product warn/`emit_json`/`scope.rs` still match plan dogfood `ed329b1` (fold-in `2b3f859` was docs-only). **Match** at go HEAD `ca75751`.
- [x] Re-read **source** whoami remediations (~823): adopt-path present even if PATH is old. **Present** `:823`.
- [x] Split-stream dogfood `scope resolve --format json` (classify only). Confirm stderr SOOT + empty `warnings[]` still, or note drift. **Confirmed** debug bin: stdout `{` + `"warnings": []`; stderr T240 SOOT; concat `ConvertFrom-Json` fails `Additional text… W`.
- [x] Re-check lock clap + crates.io: still no clap 5 (or this track is not that bump). lock **4.6.1** / builder **4.6.0** / crates.io **4.6.6**. No clap 5.
- [x] Rescan **entire** `conductor/deferred.md` for new open warn / JSON-interleave rows. T257 Planned row + T259 soft pointer only.
- [x] Last merged PR + open HEAD PR Cursor comments. Mint placeholder if a leftover fits nowhere. **#172** comments/reviews/inline empty; open PRs Dependabot only. **N/A.**
- [x] `ledgerful ledger start T257-warning-json-stdout-hygiene --category FEATURE` — TX `d086c5f3-6918-49e6-a1fd-377a743ee7fc`
- [x] Do **not** `cargo install`, write live `.env`, `set-alias 7d97a456 AI-Brains`, or mutate scheduled tasks.

---

## Phase 1 — Red

- [x] Units first against `pub(crate)` skip in `project.rs`; extract in Phase 2. Commit `4c5a718`.
- [x] Add `crates/ai-brains-cli/tests/warning_json_stdout_hygiene.rs` (hermetic tempdir + T240 register-path helpers).
- [x] `scope_resolve_json__mismatch__stdout_parses_token_no_soot` (**red** — stderr SOOT)
- [x] `scope_resolve_json__mismatch__concat_streams_parse` (**red** — trailing `Warning:`)
- [x] `whoami_json__mismatch__no_stderr_soot` (**red**)
- [x] `whoami_human__mismatch__no_stderr_soot` (**red**)
- [x] `nightly_status_json__mismatch__no_soot_no_warnings_key` (**red** on stderr SOOT)
- [x] `scope_resolve_human__mismatch__stderr_soot_stdout_clean` (**green** today — guard)
- [x] Re-run T240 `project_list__identity_mismatch__warn_on_stderr` — **stayed green** after green (F7)
- [x] Units: `should_skip` whoami/adopt-path (**red** at `project.rs:1406`); token units written with green module
- [x] `cargo nextest run -p ai-brains-cli --test warning_json_stdout_hygiene` **failed 5/9** (AC3/AC5/AC6/AC7/AC9). Guards AC8/AC13/AC14 already green.

---

## Phase 2 — Green (record / skip / flush)

- [x] Extract warn helpers → `identity_warn.rs` (F11). `project.rs` **1514** lines (was ~1549).
- [x] `record_identity_mismatch` at `main.rs` vault-open (no eprintln).
- [x] Extend `should_skip` for `project whoami` / `project adopt-path` (AC1).
- [x] `flush_identity_mismatch_warn` from `handle_cli_result` (Ok and Err).
- [x] AC4 / AC6 / AC13 / AC14 path compiles; list still warns.

---

## Phase 3 — Green (JSON silence + token)

- [x] `print_json_stdout` + `note_machine_stdout` (F8).
- [x] Wire `emit_json` / JSON `emit_error`.
- [x] Shared inject helper; `run_resolve_local` **and** `run_resolve_daemon` before `emit_json` (AC17).
- [x] `print_json_stdout` lives in `identity_warn.rs` (F8/F11) — not `format_resolve.rs`.
- [x] whoami / nightly_status / remaining F8 stdout pretty sites call the helper. Compact JSON sites (`recall`/`backup`/`ingest`/preflight full) call `note_machine_stdout`.
- [x] AC3 / AC5 / AC7 / AC9 green. Nightly keys unchanged (F15).
- [x] Dry-run AC8: stdout preview has no SOOT.

---

## Phase 4 — Docs + gate

- [x] CAPABILITIES mismatch-warn row (JSON-effective silent + token + remediator skip)
- [x] PROTOCOL-COMPAT scope row: keys unchanged; additive token
- [x] Root CHANGELOG T257
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` — exit 0
- [x] `cargo nextest run -p ai-brains-cli --test warning_json_stdout_hygiene` — **9 passed**
- [x] `cargo nextest run -p ai-brains-cli --test project_identity_convergence` — **all passed** (AC4 green)
- [x] Targeted T249 scope format + T255 nightly status tests — **all passed**
- [x] Phase-1 review → FEATURE `codex-review` — CX1 product PASS; P2 process-timing (closeout after gate)
- [x] Manual AC16 classify-only (source bin; no live `.env`; no key paste) — token present; stderr empty; concat parses
- [x] Full workspace gate on finalize — `dev-check.ps1` exit 0 (3026 passed, 1 skipped); `ledgerful verify --scope full` exit 0
- [x] conductor **Completed**; deferred closeout; pin

---

## Definition of Done

- [x] F0–F26 honored (F0 lifted only after go)
- [x] AC1–AC17 evidenced (AC16 manual classify; AC17 both scope emit sites)
- [x] T240 AC4 still green
- [x] No contracts field; no clap 5; no new crates
- [x] `project.rs` did not grow (1514; extract allowed)
- [x] No live `.env` write; no `cargo install`; no leftover path mutate
- [x] FEATURE TX committed; conductor Completed; deferred T257 row closed

---

**Planning + fold-in 2026-08-17.** Still **plan-only until go**.
