# T294 Plan — context vault upsert (already-initialized)

**Status:** **Pending** (Planned; F0 until **go**). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F38 / AC1–AC14
**Category:** UX / IDENTITY / FEATURE
**Ledger TX (planning):** `dd3a3998-4754-49e8-9558-524c7b1761c3` (DOCS)
**Ledger TX (implement):** FEATURE on **go** (new)

---

## Preflight (plan time — 2026-08-24)

| Check | Result |
|-------|--------|
| HEAD / tree | `2325adc` T293 `#209` on `main`. CLEAN. `origin/main` = HEAD. |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T259/T258/T282. No T285–T293.** Hole is in **source + PATH**. **Do not `cargo install`.** |
| Leftover shared-only | **5** roots on `7d97a456`: crawlx, degoo, gimp, homebrew-tap, kinledger |
| Leftover `.env` dests | crawlx `a1a61a6f-578a-683a-0000-000000000000`; degoo `39dadbbe-bef9-1245-0000-000000000000`; kinledger `efb5f6dd-b89b-82de-0000-000000000000`; gimp/homebrew-tap **NO_ENV** |
| Print-only rebind crawlx dest | exit **1** `Project 'a1a61a6f-…' not found in vault.` **Did not `--write`.** |
| `context --help` | docstring “writes local .env”; **no** after_help |
| Early-return | `commands/context.rs:135–151` **before** ensure `:182` |
| `preflight --summary` | Pinned **4049** (volatile). In-context **0/0/0**. Word **175** |
| Last PR comments | #209 T293 — Cursor/Bugbot/reviews/issue **empty**. **N/A. No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; GitHub **v4.6.6**; **no clap 5**); serde_json **1.0.150** (1.0.151); uuid **1.23.1** (1.25.0); chrono **0.4.44**; rusqlite **0.39.0** (0.40.2); dotenvy **0.15.7** = crates.io — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** (3.924) — do not touch. `sync.rs` **#2**. `governed_common.rs` **#3**. `forget.rs` **#4**. **`commands/context.rs` #5 (2.932)** — extend here. |
| Ledger | 0 pending / 0 drift at scan; planning TX `dd3a3998` |
| `ISSUES.md` | **Does not exist** (F22) |
| ledgerful search | index `--incremental` **failed** this pass (writer killed); grep: ensure `:107` (`src/context.rs`); early-return only `commands/context.rs:150` |
| Online | clig.dev default-right + tell-user-on-state-change; 12-factor config / dotenv never rewrite already-set; T259 F9 dest-must-exist; T240 F2; uuid `parse_str` hashed IDs |
| Skill | CAPABILITIES Init + OPERATIONS `:512` + WORKFLOWS leftover `:79` |
| doctor | **4** warn (legacy `.changeguard` / sig-pin / sig-version / timings). :8081/:8083 unreachable this pass (volatile) |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [ ] Re-read `commands/context.rs` already-initialized arm — still `return Ok(())` **before** ensure
- [ ] Confirm `ensure_project_and_session_exists` still `src/context.rs:107` projection-then-append
- [ ] Confirm `file_project_id_from_env_text` still trim `strip_prefix`
- [ ] Confirm clap Context still `--new-project` / `--new-session` / `--show` / `--tx-id` (no extra flags)
- [ ] Confirm T259 `resolve_project_ref` still dest-missing `not found in vault` (do **not** mint in rebind)
- [ ] Confirm leftover shared-only still 5 (or classify drift); print-only dest-missing on crawlx env id **read-only**
- [ ] Confirm `#209` still empty Cursor; no mint; Dependabot `#61` still not this track
- [ ] Rescan `conductor/deferred.md` — T294 absorbed; T295–T300 / T258 / T259 F9 mint / T240 F2 not stolen
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, uuid **1.23.1**, dotenvy **0.15.7** — **no bump**
- [ ] FEATURE TX (new)
- [ ] Did **not** `cargo install`; did **not** write live `.env`; did **not** leftover `--write --yes`; did **not** grow `project.rs`

---

## Absorbed deferred

- [ ] leftover dest-missing / context skip upsert → F1–F4 / AC3–AC4
- [ ] Placeholder Manual `context` + list contains id + print-only rebind dest exists → AC3 / AC4 / AC10
- [ ] T259 F9 runbook `context` first → F8 / F19 honesty (rebind still does not mint)
- [ ] T240 F2 / T259 F5 / T276 F9 → affirm F2 / F10 / F11
- [ ] last-PR #209 Cursor N/A → F25 no T301

## Declined (written)

- [ ] T258 adopt-path steal / T240 F2 reopen
- [ ] Dest mint inside `rebind-path`
- [ ] Live leftover `--write --yes` without owner confirm
- [ ] T282 `--show` steal / T293 Completed steal / T295–T300
- [ ] Identity `7d97` vs `fcb8a40f` new TNN
- [ ] clap 5 / rusqlite 0.40 / H2 / 750 ms

---

## Phase 1 — Red (TDD)

- [ ] `file_session_id_from_env_text__padded_value__trimmed` (AC1)
- [ ] rstest hashed-shape + v4 `FromStr` (AC2)
- [ ] `context__already_initialized_foreign_hashed_id__upserts_env_bytes_unchanged` (AC3) — **must fail** while early-return stands
- [ ] `context__already_initialized_foreign_hashed_id__rebind_print_only_dest_exists` (AC4) — **must fail** dest-missing
- [ ] Commit red allowed

---

## Phase 2 — Green

- [ ] Already-initialized arm: parse F3 → ensure F1 → `Vault:` F32 → return **without** `fs::write` / sync pull
- [ ] Invalid UUID exit 1 (AC7)
- [ ] Session-only skip hashed mint (AC6)
- [ ] `--show` skip ensure (AC8)
- [ ] clap Context after_help + docstring dual-truth; T259 after_help F19
- [ ] Commit green allowed

---

## Phase 3 — Stay-green + docs

- [ ] Smoke `test_cli_context_idempotency` (AC5)
- [ ] T282 `context_show_leftover.rs`
- [ ] T259 `project_rebind_path__dest_missing__exit_1`
- [ ] First-init still writes (AC11)
- [ ] Feature-off AC12
- [ ] CAPABILITIES / OPERATIONS `:512` / WORKFLOWS leftover / CLI-EXIT-CODES / CHANGELOG (AC9)
- [ ] `context --help` hermetic AC9 / AC14

---

## Phase 4 — Review + gate + publish

- [ ] Phase-1 review → `conductor/tracks/trackT294-context-vault-upsert/review.md`
- [ ] Cross-model FEATURE (`codex-review`)
- [ ] `cargo fmt --check` ; clippy workspace `-D warnings` ; nextest workspace ; `cargo deny check` ; `cargo audit`
- [ ] `ledgerful verify --scope full`
- [ ] Manual AC10 (hermetic). Live leftover `context` / `--write` **only if owner confirmed**
- [ ] conductor **Completed**; deferred closeout table
- [ ] Pin `DECISION:` (upsert follows `.env`; T240 F2; T259 F9 rebind still does not mint)
- [ ] implement-track Phase 6: push `track/T294-*`, PR, watch GHA `CI` green, squash-merge, prune. Never `git push origin main`. Never force-push.

---

## DoD (checkable)

- [ ] Hermetic `context` on `.env` dest-not-in-vault: exit **0**, `.env` bytes unchanged, `project list` JSON contains dest id, stdout `already initialized` + `Vault: project and session present.`
- [ ] Print-only `rebind-path --to <that id> --format human` dest exists (exit **0**, not “not found in vault”)
- [ ] T259 dest-missing AC8 still exit **1** for a UUID context never ensured
- [ ] No live leftover `--write --yes` unless owner confirmed
- [ ] No `project.rs` production edit
- [ ] F0 was respected (no product commits as planning)

---

## Isolation recap

Do **not** `cargo install`. Do **not** rewrite `.env`. Do **not** leftover `--write`. Do **not** grow `project.rs`. Do **not** mint dest in `rebind-path`. Do **not** steal T258 / T282 / T295–T300.
