# T294 Plan — context vault upsert (already-initialized)

**Status:** **Completed** (gates green; publish Phase 6). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F39 / AC1–AC15 + §13 AI fold-in
**Category:** UX / IDENTITY / FEATURE
**Ledger TX (planning):** `dd3a3998-4754-49e8-9558-524c7b1761c3` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `d578953e-0409-47cd-b323-2c4a6faca842` (DOCS)
**Ledger TX (implement):** `b61c69ee-23ab-4bb1-8d0b-3586cd6d4b3f` (FEATURE)

---

## AI fold-in (2026-08-24) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 0**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F3/AC1:** `strip_prefix("AI_BRAINS_SESSION_ID=")` ; `_EXTRA=` → `None`.
2. **AC15:** second `context` `event_count` unchanged.
3. **AC3:** comment + blank + dummy `ZERO_KEY`; `Vault:` once.
4. **F39:** local AC4 seed; do not import `fixture_rebind`.
5. **AC6:** no `Vault:` on session-only skip.
6. **AC9:** WORKFLOWS leftover ensure-without-rewrite sentence.

---

## Preflight (plan time — 2026-08-24; fold-in refresh)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in on `6fe734c` (`main`, T294 plan). Parent `2325adc` T293 `#209`. CLEAN at fold-in start. `origin/main` = `2325adc` until this commit. |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T259/T258/T282. No T285–T293.** Hole is in **source + PATH**. **Do not `cargo install`.** |
| Leftover shared-only | **5** roots on `7d97a456`: crawlx, degoo, gimp, homebrew-tap, kinledger |
| Leftover `.env` dests | crawlx `a1a61a6f-578a-683a-0000-000000000000`; degoo `39dadbbe-bef9-1245-0000-000000000000`; kinledger `efb5f6dd-b89b-82de-0000-000000000000`; gimp/homebrew-tap **NO_ENV** |
| Print-only rebind crawlx dest | exit **1** `Project 'a1a61a6f-…' not found in vault.` **Did not `--write`.** |
| `context --help` | docstring “writes local .env”; **no** after_help |
| Early-return | `commands/context.rs:135–151` **before** ensure `:182` |
| `preflight --summary` | Pinned **4058** (volatile; plan 4049 / OpenCode 4049). In-context **0/0/0**. Word **468** (plan 175 / OpenCode 452) |
| Last PR comments | #209 T293 — Cursor/Bugbot/reviews/issue **empty**. **N/A. No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; GitHub **v4.6.6**; **no clap 5**); serde_json **1.0.150** (1.0.151); uuid **1.23.1** (1.25.0); chrono **0.4.44**; rusqlite **0.39.0** (0.40.2); dotenvy **0.15.7** = crates.io — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** (**3.915**) — do not touch. `sync.rs` **#2**. `governed_common.rs` **#3**. `forget.rs` **#4**. **`commands/context.rs` #5 (2.924)** — extend here. |
| Ledger | 0 pending / 0 drift at scan; planning TX `dd3a3998`; fold-in TX `d578953e` |
| `ISSUES.md` | **Does not exist** (F22) |
| ledgerful search | index `--incremental` **failed** this pass (writer killed); grep: ensure `:107` (`src/context.rs`); early-return only `commands/context.rs:150` |
| Online | clig.dev default-right + tell-user-on-state-change; 12-factor config / dotenv never rewrite already-set; T259 F9 dest-must-exist; T240 F2; uuid `parse_str` hashed IDs |
| Skill | CAPABILITIES Init + OPERATIONS `:512` + WORKFLOWS leftover `:79` |
| doctor | **4** warn (legacy `.changeguard` / sig-pin / sig-version / timings). OpenCode said 5 — volatile. :8083 **ok**; :8081 unreachable this pass |

---

## Phase 0 — on go (re-verify)

- [x] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [x] Re-read `commands/context.rs` already-initialized arm — still `return Ok(())` **before** ensure
- [x] Confirm `ensure_project_and_session_exists` still `src/context.rs:107` projection-then-append
- [x] Confirm `file_project_id_from_env_text` still trim `strip_prefix` (`:27–34`) — session helper must match (AC1 `_EXTRA`)
- [x] Confirm clap Context still `--new-project` / `--new-session` / `--show` / `--tx-id` (no extra flags); dispatch `:4876`
- [x] Confirm `--show` still returns `:88` before the already-initialized arm (F6)
- [x] Confirm `fixture_rebind` still private in `project_rebind_path.rs` — AC4 local seed (F39); do **not** edit that file
- [x] Confirm T259 `event_count` still `:98` — copy into new test file (AC15)
- [x] Confirm T259 `resolve_project_ref` still dest-missing `not found in vault` (do **not** mint in rebind)
- [x] Confirm leftover shared-only still 5 (or classify drift); print-only dest-missing on crawlx env id **read-only**
- [x] Confirm `#209` still empty Cursor; no mint; Dependabot `#61` still not this track
- [x] Rescan `conductor/deferred.md` — T294 absorbed; T295–T300 / T258 / T259 F9 mint / T240 F2 not stolen
- [x] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, uuid **1.23.1**, dotenvy **0.15.7** — **no bump**
- [x] FEATURE TX `b61c69ee-23ab-4bb1-8d0b-3586cd6d4b3f`
- [x] Did **not** `cargo install`; did **not** write live `.env`; did **not** leftover `--write --yes`; did **not** grow `project.rs`

---

## Absorbed deferred

- [x] leftover dest-missing / context skip upsert → F1–F4 / AC3–AC4
- [x] Placeholder Manual `context` + list contains id + print-only rebind dest exists → AC3 / AC4 / AC10
- [x] T259 F9 runbook `context` first → F8 / F19 honesty (rebind still does not mint)
- [x] T240 F2 / T259 F5 / T276 F9 → affirm F2 / F10 / F11
- [x] last-PR #209 Cursor N/A → F25 no T301

## Declined (written)

- [x] T258 adopt-path steal / T240 F2 reopen
- [x] Dest mint inside `rebind-path`
- [x] Live leftover `--write --yes` without owner confirm
- [x] T282 `--show` steal / T293 Completed steal / T295–T300
- [x] Identity `7d97` vs `fcb8a40f` new TNN
- [x] clap 5 / rusqlite 0.40 / H2 / 750 ms

---

## Phase 1 — Red (TDD)

- [x] `file_session_id_from_env_text__padded_value__trimmed` + `_EXTRA` → `None` (AC1)
- [x] rstest hashed-shape + v4 `FromStr` (AC2)
- [x] `context__already_initialized_foreign_hashed_id__upserts_env_bytes_unchanged` (AC3 rich `.env`)
- [x] `context__already_initialized_foreign_hashed_id__rebind_print_only_dest_exists` (AC4 local seed)
- [x] `context__already_initialized_second_run__event_count_unchanged` (AC15)
- [x] Commit red allowed (combined with green in implement commit)

---

## Phase 2 — Green

- [x] Already-initialized arm: parse F3 → ensure F1 → `Vault:` F32 → return **without** `fs::write` / sync pull
- [x] Invalid UUID exit 1 (AC7)
- [x] Session-only skip hashed mint (AC6)
- [x] `--show` skip ensure (AC8)
- [x] clap Context after_help + docstring dual-truth; T259 after_help F19
- [x] Commit green allowed

---

## Phase 3 — Stay-green + docs

- [x] Smoke `test_cli_context_idempotency` (AC5)
- [x] Session-only skip: no `Vault:` (AC6)
- [x] T282 `context_show_leftover.rs`
- [x] T259 `project_rebind_path__dest_missing__exit_1`
- [x] First-init still writes (AC11 — smoke + Manual)
- [x] Feature-off AC12 (default hermetic bin)
- [x] CAPABILITIES / OPERATIONS `:512` / WORKFLOWS leftover ensure-without-rewrite / CLI-EXIT-CODES / CHANGELOG (AC9)
- [x] `context --help` hermetic AC9 / AC14

---

## Phase 4 — Review + gate + publish

- [x] Phase-1 review → `conductor/tracks/trackT294-context-vault-upsert/review.md`
- [x] Cross-model FEATURE (`codex-review`) — fresh PASS after P2 fixes
- [x] `cargo fmt --check` ; clippy workspace `-D warnings` ; nextest workspace ; `cargo deny check` ; `cargo audit`
- [x] `ledgerful verify --scope full` (pre-P2) + post-P2 workspace nextest/deny/audit exit 0
- [x] Manual AC10 (hermetic). Live leftover `context` / `--write` **skipped** (F27)
- [x] conductor **Completed**; deferred closeout table
- [x] Pin `DECISION:` (upsert follows `.env`; T240 F2; T259 F9 rebind still does not mint)
- [ ] implement-track Phase 6: push `track/T294-*`, PR, watch GHA `CI` green, squash-merge, prune. Never `git push origin main`. Never force-push.

---

## DoD (checkable)

- [x] Hermetic `context` on `.env` dest-not-in-vault: exit **0**, `.env` bytes unchanged (comment/blank/dummy KEY survive), `project list` JSON contains dest id, stdout `already initialized` + `Vault:` **once**
- [x] Second `context` on that dest: `event_count` unchanged (AC15)
- [x] Print-only `rebind-path --to <that id> --format human` dest exists (exit **0**, not “not found in vault”) — local seed, not `fixture_rebind`
- [x] T259 dest-missing AC8 still exit **1** for a UUID context never ensured
- [x] No live leftover `--write --yes` unless owner confirmed
- [x] No `project.rs` production edit
- [x] F0 was respected (no product commits as planning)

---

## Isolation recap

Do **not** `cargo install`. Do **not** rewrite `.env`. Do **not** leftover `--write`. Do **not** grow `project.rs`. Do **not** mint dest in `rebind-path`. Do **not** steal T258 / T282 / T295–T300.
