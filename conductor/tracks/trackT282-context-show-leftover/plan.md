# T282 Plan — `context --show` leftover shell vs `.env`

**Status:** **Completed** 2026-08-22
**Spec:** [spec.md](./spec.md) F0–F36 / AC1–AC14 + §13 AI fold-in
**Category:** UX / HONESTY
**Ledger TX (planning):** `fe4e6895-6619-490d-8bbb-72a0fab55bb7` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `11b44c43-0a67-47a5-906b-f25e1f9035e8` (DOCS)
**Ledger TX (implement):** FEATURE `c93b313e-4270-446f-bce5-eb80f8fec7f0`

---

## AI fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 0**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F19:** no new skill section; `.claude/skills/ai-brains/SKILL.md` existing `--show` (`:50`/`:57`/`:88`) gets one leftover sentence; `.agents` skill no-op.
2. **AC4:** leftover exact string **once**; hermetic **must** `isolate_empty_home`.
3. **F33 / AC2:** `file_project_id_from_env_text` strip+trim + whitespace-padded case.
4. **F34 / AC3:** exact `KEY=` / `VAULT_KEY=`; `KEYRING=` / `VAULT_KEY_PATH=` passthrough.
5. **F35:** capture-site comment only.
6. **F36:** keep `VAULT_KEY` redact (daemon/elevation live — OpenCode “not in crates/” is false).
7. **Affirm:** #197 N/A; no T285.

---

## Preflight (plan time — 2026-08-22)

| Check | Result |
|-------|--------|
| HEAD / tree | **Plan dogfood:** `65108cd` T281 `#197`. CLEAN. `origin/main` = HEAD (`0 0`) after fetch |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-22 14:49, 25 443 840 bytes. Likely post-T281. Leftover line **absent**. **Do not `cargo install`.** |
| `preflight --summary` | Pinned **3619** (volatile); in-context 0/0/0; grants **0 of 3**; Scope `3581317d` |
| PATH `context --show` | Dump `PROJECT_ID=3581317d-…` + models + Repository. **No** `7d97a456`. **No** `x'` |
| PATH `project whoami` | JSON `shell_project_id=7d97a456-…`; `mismatch: false`; remediations `[]` |
| Last PR comments | #197 T281 — **empty** (N/A). #188 closed by T284. No T285 |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); rusqlite **0.39.0** (0.40.2); uuid lock **1.23.1** (1.25.0); tokio **1.52.3** (1.53.1) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** — do not grow. `sync.rs` #2 / `forget.rs` #3 / `governed_common.rs` #4 / **`context.rs` #5** (intended) |
| Ledger | 0 pending / 0 drift at scan; planning TX `fe4e6895` |
| `ISSUES.md` | **Does not exist** (F23) |
| ledgerful search | `shell_project_id_captured` `project.rs:160/:704` |
| Online | clig.dev human-first + just-enough; 12-factor config litmus (no credentials in dump); clap 4.6.6 `hide_env_values` is help-only (T256) |
| Skill | `.agents` no `context` match (no-op). `.claude` **already** `--show` at `:50`/`:57`/`:88` — F19 one-liner on go (OpenCode m-1) |

---

## Phase 0 — on go (re-verify)

- [x] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [x] Re-read `context.rs` `:19–35` show dump — early return before write
- [x] Re-read clap Context `main.rs` `:1284–1298` — **no new flags**
- [x] Re-read shell capture `main.rs` `:3256–3263` + `project.rs` `:156–163` — **call existing**, do not grow `project.rs`
- [x] Re-read whoami leftover match `project.rs` `:703–709` — **do not change**
- [x] Re-read T242 `env_warn.rs` SOOT — **do not restyle**
- [x] Re-read T256 `hide_env_values` `main.rs` `:997` — **do not reopen help**
- [x] Confirm no existing `tests/` coverage for `context --show`
- [x] Rescan `conductor/deferred.md` — T282 rows absorbed; T283 still placeholder
- [x] Confirm #197 comments/reviews still empty (N/A); no mint; Dependabot `#61` still not this track
- [x] Re-dogfood `context --show` + `project whoami` **read-only**. **Did not** write `.env`
- [x] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [x] FEATURE TX
- [x] Did **not** `cargo install`; did **not** grow `project.rs` / `sync.rs` / `forget.rs` / `env_warn.rs`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit `--show` misses leftover shell | **DoD** F1 / AC1–AC5 / AC10 |
| Placeholder no `AI_BRAINS_KEY` on stdout | **DoD** F3 / AC3 / AC6 |
| T276 shell leftover pointer | This track |

## Declined (written)

| Item | Why |
|------|-----|
| T240 F2 write | F2 / AC7 |
| T206 L3 path-mismatch on `--show` | F10 |
| T242 restyle | F6 |
| T256 `--help` | F7 |
| `--format json` / vault-free `--show` | F4 / F11 |
| T283 / leftover rebind / clap 5 / rusqlite 0.40 | F12/F17 |
| last-PR #197 Cursor | N/A empty |
| Dependabot rusqlite `#61` | F12 — no T285 |
| SESSION leftover / no-`.env` `(.env overrides)` | F26 / F27 |

---

## Phase 1 — Red (TDD)

- [x] `format_shell_leftover_line__known_uuid__frozen_80` — AC1 (prefix 27, suffix 17, line 80, no `Warning:`)
- [x] `leftover_shell_vs_file__differ__some` + rstest None cases — AC2 (include F33 padded file value)
- [x] `map_show_env_line__key__redacted` + passthrough / skip — AC3 (F34 `KEYRING=` / `VAULT_KEY_PATH=` passthrough)
- [x] Commit red allowed

---

## Phase 2 — Green

- [x] F1 consts + `format_shell_leftover_line` + `leftover_shell_vs_file` + F33 `file_project_id_from_env_text` in `context.rs`
- [x] F3/F34 `map_show_env_line` in the dump loop; F36 VAULT_KEY rustdoc; F35 capture comment
- [x] After `Repository:`, `if let Some(line) = leftover_shell_vs_file(captured.as_deref(), file_id)` println **once**
- [x] Hermetic `tests/context_show_leftover.rs` AC4–AC8 (`hermetic_bin` + **must** `isolate_empty_home` + leftover `count() == 1`; dummy KEY ≠ zero vault KEY)
- [x] AC9 / AC13 / AC14 stay green
- [x] Commit green

---

## Phase 3 — Docs

- [x] CAPABILITIES Show-only row: leftover next-line + KEY redact
- [x] OPERATIONS `--show` bullet: same
- [x] PROTOCOL-COMPAT: no new required keys
- [x] CLI-EXIT-CODES: show still exit 0
- [x] Root CHANGELOG T282
- [x] `.claude/skills/ai-brains/SKILL.md`: one leftover sentence on existing `--show` (F19). **Skip** `.agents/skills/ai-brains/SKILL.md` (no `context` match)

---

## Phase 4 — Gate + publish (on go)

- [x] Classify-only AC10 (`cargo run -p ai-brains-cli -- context --show`). **No** `.env` write
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [x] Review log `review.md`; FEATURE cross-model (F22)
- [x] `scripts/dev-check.ps1` (not repo-root `dev-check.ps1`)
- [x] implement-track Phase 6: push `track/T282-*` → PR → watch GHA `CI` green → squash-merge → prune. Never `git push origin main`

## DoD

- [x] `--show` names leftover shell vs `.env` when they differ (F1)
- [x] KEY / VAULT_KEY file lines redacted (F3)
- [x] No T240 F2 write (AC7)
- [x] Conductor **Completed** only after merge + hygiene
