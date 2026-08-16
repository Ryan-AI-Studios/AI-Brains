# T256 Plan — Help redacts secrets

**Status:** **Pending** (planned; F0 until go)
**Spec:** [spec.md](./spec.md) F0–F19 / AC1–AC11
**Category:** SECURITY / UX
**Ledger TX (planning):** `0cb0fe21-2325-4b22-917a-4b649d6dadef` (DOCS)
**Ledger TX (implement):** start **SECURITY** on go

---

## Preflight (plan time — 2026-08-16)

| Check | Result |
|-------|--------|
| HEAD / tree | `d6749b6` on `main`; 0 ahead of `origin/main`. Dirty: conductor series docs from T256–T271 registration (not product crates). |
| T256 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe`. Root `--help` / `-h`: `has_xquote_assign=True`. `recall --help`: no `AI_BRAINS_KEY`. |
| `--key` site | `main.rs:432` `env = "AI_BRAINS_KEY"` — **no** `hide_env_values` |
| clap | workspace 4.5 / lock **4.6.1** / builder **4.6.0**. `Arg::hide_env_values` present (`:2667`, `feature = "env"`). crates.io latest **4.6.6**. **No clap 5.** Snapshot — re-verify at execute. |
| rustc / workspace | 1.95.0 / **0.1.1** |
| `hide_env_values` in tree | ledgerful search: **no matches** |
| Last PR Cursor | #169 empty comments/reviews; HEAD `main`; Dependabot #68–#72 only. **N/A.** |
| `deferred.md` | Full scan. Overlap: audit help leak (absorb); T181 helper (absorb); T257+ peers (point); init print / daemon key (decline). |
| ai-brains | `preflight --summary` ok (test-alias mismatch → T258). Recall: T197 / Session-0 quoting / no-commit-keys. No prior “hide help env” pin. |
| ledgerful | doctor ready (hygiene warns). 0 pending at start. `ask` local model thinking-only — noted. |
| `ISSUES.md` | **Does not exist** |
| Hotspots | Do not touch `project.rs`. Change is one attribute on `main.rs` + new test file. |

---

## Phase 0 — on go (re-verify)

- [ ] Re-read `Cli.key` in `main.rs` (line may have drifted). Confirm still `env = "AI_BRAINS_KEY"` and no `hide_env_values`.
- [ ] Re-check lock clap version + `clap_builder-*/src/builder/arg.rs` `hide_env_values`.
- [ ] `cargo search clap` / crates.io: still no clap 5 (or this track is not that bump).
- [ ] Rescan **entire** `conductor/deferred.md` for new open help/secret rows.
- [ ] Last merged PR + open HEAD PR Cursor comments. Mint placeholder if a leftover fits nowhere.
- [ ] `ledgerful ledger start T256-help-redact-secrets --category SECURITY`
- [ ] Do **not** `cargo install`, rewrite `.env`, or paste a live key.

---

## Phase 1 — Red

- [ ] Add `crates/ai-brains-cli/tests/cli_help_secret_redaction.rs` with dummy ≠ `ZERO_SQLCIPHER_KEY`.
- [ ] `root_long_help__dummy_key_env__does_not_echo_payload`
- [ ] `root_short_help__dummy_key_env__does_not_echo_payload`
- [ ] `root_long_help__key_unset__still_names_env` (`hermetic_bin_no_key`)
- [ ] `doctor_help__dummy_key_env__does_not_echo_payload`
- [ ] `recall_help__dummy_key_env__does_not_echo_payload`
- [ ] `unknown_flag__dummy_key_env__does_not_echo_payload`
- [ ] Proof helper: `assert_no_secret_leakage` + no `AI_BRAINS_KEY=x'`
- [ ] `cargo nextest run -p ai-brains-cli --test cli_help_secret_redaction` **fails** (red)

---

## Phase 2 — Green

- [ ] `hide_env_values = true` on `Cli.key` only
- [ ] Same nextest filter **passes**
- [ ] `cargo nextest run -p ai-brains-cli --test cli_help_ia` green (T204)
- [ ] `key_resolve` units green (no file change)
- [ ] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`

---

## Phase 3 — Docs

- [ ] `Docs/CAPABILITIES.md` `--key` row: help never echoes the live value
- [ ] Root `CHANGELOG.md` T256 row (name stays; value hidden)
- [ ] INSTALL placeholders **unchanged**
- [ ] No PROTOCOL-COMPAT / contracts

---

## Phase 4 — Review + gate

- [ ] Phase-1 review → `conductor/tracks/trackT256-help-redact-secrets/review.md`
- [ ] Medium+ not silently dropped
- [ ] SECURITY `codex-review` → `review.codex.md` until clean
- [ ] Manual AC11: classify-only (`has_xquote_assign` false, name still present). **Do not paste values.**
- [ ] Full gate: `cargo fmt --check` ; clippy workspace `-D warnings` ; `cargo nextest run --workspace` ; `cargo deny check` ; `cargo audit` ; `ledgerful verify --scope full`
- [ ] Note skip if deny/audit not installed locally (same class as T255)

---

## Phase 5 — Close

- [ ] conductor T256 → **Completed** with evidence
- [ ] `deferred.md`: strike the help-leak row; keep PATH-behind as soft
- [ ] SECURITY TX commit
- [ ] Optional pin: `DECISION: T256 clap hide_env_values on --key; help documents AI_BRAINS_KEY and never echoes the value`
- [ ] Do not push to `main` without owner

---

## DoD (checkable)

- [ ] AC1–AC11 evidenced
- [ ] No live key in review / PR / chat
- [ ] `key_resolve.rs` / `init.rs` / `help_ia.rs` untouched
- [ ] No clap 5 / new crate / pin bump
- [ ] T204 help IA still green
- [ ] No open critical/high; mediums fixed or deferred.md with cap
- [ ] F0 was respected (no product commit as “planning”)

---

## Absorbed deferred (plan-time)

| Item | Action |
|------|--------|
| Audit `--help` key leak | DoD AC1–AC2 |
| T181 leak helper | F8 |
| Doctor “no secrets on stdout” | Decline as DoD; AC4 still locks doctor help |
| T257–T271 peers | Point, do not steal |
| last-PR Cursor | N/A — none |
| Init print / daemon VAULT_KEY | Decline F6/F7 |
