# T257 Plan — Warning + JSON stdout hygiene

**Status:** **Pending** (requirements written; spec **Planned** + fold-in)
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

- [ ] Re-read `maybe_warn_identity_mismatch` (`project.rs`) and the `:3275` call. Confirm still `eprintln!` before the command `match` (or note drift).
- [ ] Re-read `emit_json` — still `to_string_pretty` + one `println!`.
- [ ] Re-read `scope.rs` **both** `emit_json(&wire)` sites (`run_resolve_local` + `run_resolve_daemon`). Confirm line numbers vs F24 (or note drift).
- [ ] Re-read `handle_cli_result` — still the Ok/Err join for `run` and `run_sync_path_free`.
- [ ] Confirm product warn/`emit_json`/`scope.rs` still match plan dogfood `ed329b1` (fold-in `2b3f859` was docs-only).
- [ ] Re-read **source** whoami remediations (~823): adopt-path present even if PATH is old.
- [ ] Split-stream dogfood `scope resolve --format json` (classify only). Confirm stderr SOOT + empty `warnings[]` still, or note drift.
- [ ] Re-check lock clap + crates.io: still no clap 5 (or this track is not that bump).
- [ ] Rescan **entire** `conductor/deferred.md` for new open warn / JSON-interleave rows.
- [ ] Last merged PR + open HEAD PR Cursor comments. Mint placeholder if a leftover fits nowhere.
- [ ] `ledgerful ledger start T257-warning-json-stdout-hygiene --category FEATURE`
- [ ] Do **not** `cargo install`, write live `.env`, `set-alias 7d97a456 AI-Brains`, or mutate scheduled tasks.

---

## Phase 1 — Red

- [ ] Add `crates/ai-brains-cli/src/commands/identity_warn.rs` as a thin move **or** write units first against `pub(crate)` fns still in `project.rs` if extract is Phase 2. Prefer extract in Phase 2; Phase 1 may add units next to existing skip tests **or** in the new module if created empty-with-tests.
- [ ] Add `crates/ai-brains-cli/tests/warning_json_stdout_hygiene.rs` (hermetic tempdir + T240 register-path helpers).
- [ ] `scope_resolve_json__mismatch__stdout_parses_token_no_soot` (**must red**)
- [ ] `scope_resolve_json__mismatch__concat_streams_parse` (**must red** — AC9)
- [ ] `whoami_json__mismatch__no_stderr_soot` (**must red**)
- [ ] `whoami_human__mismatch__no_stderr_soot` (**must red**)
- [ ] `nightly_status_json__mismatch__no_soot_no_warnings_key` (**must red** on stderr SOOT)
- [ ] `scope_resolve_human__mismatch__stderr_soot_stdout_clean` (may be **green** today — guard)
- [ ] Re-run T240 `project_list__env_differs_path__mismatch_warn` — **must stay green** after green phase (F7)
- [ ] Units: `should_skip` whoami/adopt-path (**must red**); `identity_mismatch_json_token` (**green ok** if written first)
- [ ] `cargo nextest run -p ai-brains-cli --test warning_json_stdout_hygiene` **fails** on AC3/AC5/AC9 (do not chase red by asserting “stderr still has warn”).

---

## Phase 2 — Green (record / skip / flush)

- [ ] Extract warn helpers → `identity_warn.rs` (F11). `project.rs` calls stay thin.
- [ ] `record_identity_mismatch` at `main.rs:3275` (no eprintln).
- [ ] Extend `should_skip` for `project whoami` / `project adopt-path` (AC1).
- [ ] `flush_identity_mismatch_warn` from `handle_cli_result` (Ok and Err).
- [ ] AC4 / AC6 / AC13 / AC14 path compiles; list still warns.

---

## Phase 3 — Green (JSON silence + token)

- [ ] `print_json_stdout` + `note_machine_stdout` (F8).
- [ ] Wire `emit_json` / JSON `emit_error`.
- [ ] Shared inject helper; call it in `run_resolve_local` **and** `run_resolve_daemon` before `emit_json` (F3/F24/F25/**AC17**). Do not wire local only.
- [ ] `print_json_stdout` lives in `identity_warn.rs` (F8/F11) — not `format_resolve.rs`.
- [ ] whoami / nightly_status / remaining F8 stdout pretty sites call the helper.
- [ ] AC3 / AC5 / AC7 / AC9 green. Nightly keys unchanged (F15).
- [ ] Dry-run AC8: stdout preview has no SOOT.

---

## Phase 4 — Docs + gate

- [ ] CAPABILITIES mismatch-warn row (JSON-effective silent + token + remediator skip)
- [ ] PROTOCOL-COMPAT scope row: keys unchanged; additive token
- [ ] Root CHANGELOG T257
- [ ] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [ ] `cargo nextest run -p ai-brains-cli --test warning_json_stdout_hygiene`
- [ ] `cargo nextest run -p ai-brains-cli --test project_identity_convergence`
- [ ] Targeted T249 scope format + T255 nightly status tests
- [ ] Phase-1 review → FEATURE `codex-review`
- [ ] Manual AC16 classify-only (source bin; no live `.env`; no key paste)
- [ ] Full workspace gate on finalize
- [ ] conductor **Completed**; deferred closeout; pin

---

## Definition of Done

- [ ] F0–F26 honored (F0 lifted only after go)
- [ ] AC1–AC17 evidenced (AC16 manual classify; AC17 both scope emit sites)
- [ ] T240 AC4 still green
- [ ] No contracts field; no clap 5; no new crates
- [ ] `project.rs` did not grow (extract allowed)
- [ ] No live `.env` write; no `cargo install`; no leftover path mutate
- [ ] FEATURE TX committed; conductor Completed; deferred T257 row closed

---

**Planning + fold-in 2026-08-17.** Still **plan-only until go**.
