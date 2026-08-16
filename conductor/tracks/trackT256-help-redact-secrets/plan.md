# T256 Plan — Help redacts secrets

**Status:** **Completed** (2026-08-16; SECURITY TX `6d57d26e-63d6-4fc6-a4c3-e6b4d949da3c`)
**Spec:** [spec.md](./spec.md) F0–F19 / AC1–AC12 + §13 fold-in
**Category:** SECURITY / UX
**Ledger TX (planning):** `0cb0fe21-2325-4b22-917a-4b649d6dadef` (DOCS)
**Ledger TX (fold-in):** `84d66be2-7402-4bd2-bc1a-7ca7aa04f2fe` (DOCS)
**Ledger TX (implement):** start **SECURITY** on go

---

## AI fold-in (2026-08-16) — `agy-review.md` + `opencode-review.md`

No Blockers/Majors. OpenCode four minors + one opportunity folded. Agy HEAD note folded; Agy AC6/dummy already covered. Disposition in spec **§13**.

### Pins locked by fold-in

1. **AC12:** `ai-brains help` (default help subcommand) — live leak, same render as `--help`.
2. **AC1/F2:** exact `[env: AI_BRAINS_KEY]`, not a loose name match.
3. **Phase 1:** red = AC1/AC2/AC12 only. AC3–AC6 are guards (green today).
4. **F8:** concatenate stdout+stderr before `assert_no_secret_leakage`.
5. **§2.4:** clap local source is SoT; rustic is illustrative.

---

## Preflight (plan time — 2026-08-16)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan-time dogfood `d6749b6`. Reviews + fold-in `4bea645`. Product `--key` unchanged. |
| T256 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe`. Root `--help` / `-h` / `help`: `has_xquote_assign=True` (help len 6181). `recall --help`: no key. `--not-a-real-flag`: no name. |
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

- [x] Re-read `Cli.key` in `main.rs` (line may have drifted). Confirm still `env = "AI_BRAINS_KEY"` and no `hide_env_values`.
- [x] Re-check lock clap version + `clap_builder-*/src/builder/arg.rs` `hide_env_values`.
- [x] `cargo search clap` / crates.io: still no clap 5 (or this track is not that bump).
- [x] Rescan **entire** `conductor/deferred.md` for new open help/secret rows.
- [x] Last merged PR + open HEAD PR Cursor comments. Mint placeholder if a leftover fits nowhere.
- [x] Confirm product `Cli.key` still has no `hide_env_values` (unchanged since `d6749b6` / `4bea645`).
- [x] `ledgerful ledger start T256-help-redact-secrets --category SECURITY`
- [x] Do **not** `cargo install` (product remediator), rewrite `.env`, or paste a live key. CI tools (`cargo-deny`/`cargo-audit`) installed from `Docs/ci-tooling.md` so full gate is not skipped.

---

## Phase 1 — Red

- [x] Add `crates/ai-brains-cli/tests/cli_help_secret_redaction.rs` with dummy ≠ `ZERO_SQLCIPHER_KEY`.
- [x] `root_long_help__dummy_key_env__does_not_echo_payload` (**must red**)
- [x] `root_short_help__dummy_key_env__does_not_echo_payload` (**must red**)
- [x] `root_help_subcommand__dummy_key_env__does_not_echo_payload` (**must red** — `help`)
- [x] `root_long_help__key_unset__still_names_env` (`hermetic_bin_no_key`) — **guard, green ok**
- [x] `doctor_help__dummy_key_env__does_not_echo_payload` — **guard, green ok**
- [x] `recall_help__dummy_key_env__does_not_echo_payload` — **guard, green ok**
- [x] `unknown_flag__dummy_key_env__does_not_echo_payload` — **guard, green ok**
- [x] Proof helper: concat stdout+stderr → `assert_no_secret_leakage` + no `AI_BRAINS_KEY=x'` + exact `[env: AI_BRAINS_KEY]` on AC1/AC2/AC12
- [x] `cargo nextest run -p ai-brains-cli --test cli_help_secret_redaction` **fails** because AC1/AC2/AC12 fail (do not chase red on guards). Evidence: 3 leak reds + AC3 exact-slot red (unset clap emits `[env: AI_BRAINS_KEY=]`); AC4–AC6 green.

---

## Phase 2 — Green

- [x] `hide_env_values = true` on `Cli.key` only
- [x] Same nextest filter **passes** (7/7)
- [x] `cargo nextest run -p ai-brains-cli --test cli_help_ia` green (T204) (7/7)
- [x] `key_resolve` units green (no file change) (`--bin ai-brains` 9/9)
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`

---

## Phase 3 — Docs

- [x] `Docs/CAPABILITIES.md` `--key` row: help never echoes the live value
- [x] Root `CHANGELOG.md` T256 row (name stays; value hidden)
- [x] INSTALL placeholders **unchanged**
- [x] No PROTOCOL-COMPAT / contracts

---

## Phase 4 — Review + gate

- [x] Phase-1 review → `conductor/tracks/trackT256-help-redact-secrets/review.md`
- [x] Medium+ not silently dropped (two easy P3s fixed + verified)
- [x] SECURITY `codex-review` → `review.codex.md` CX1 product PASS (process P1); CX2 after closeout
- [x] Manual AC11: classify `--help` / `-h` / `help` only (`has_xquote_assign` false, exact `[env: AI_BRAINS_KEY]`). **Do not paste values.**
- [x] Full gate: `cargo fmt --check` ; clippy workspace `-D warnings` ; `cargo nextest run --workspace` **2984 passed / 1 skipped** ; `cargo deny check` ; `cargo audit` ; `ledgerful verify --scope full` **Verification passed**
- [x] Installed `cargo-deny` 0.20.2 + `cargo-audit` 0.22.2 from `Docs/ci-tooling.md` (do not skip).

---

## Phase 5 — Close

- [x] conductor T256 → **Completed** with evidence
- [x] `deferred.md`: strike the help-leak row; keep PATH-behind as soft
- [x] SECURITY TX commit
- [x] Optional pin: `DECISION: T256 clap hide_env_values on --key; help documents AI_BRAINS_KEY and never echoes the value`
- [ ] PR + squash-merge after GHA CI green (owner asked this implement)

---

## DoD (checkable)

- [x] AC1–AC12 evidenced
- [x] No live key in review / PR / chat
- [x] `key_resolve.rs` / `init.rs` / `help_ia.rs` untouched
- [x] No clap 5 / new crate / pin bump
- [x] T204 help IA still green
- [x] No open critical/high; mediums fixed or deferred.md with cap
- [x] F0 was respected (no product commit as “planning”)

---

## Absorbed deferred (plan-time)

| Item | Action |
|------|--------|
| Audit `--help` key leak | DoD AC1–AC2 / AC12 (`help`) |
| T181 leak helper | F8 |
| Doctor “no secrets on stdout” | Decline as DoD; AC4 still locks doctor help |
| T257–T271 peers | Point, do not steal |
| last-PR Cursor | N/A — none |
| Init print / daemon VAULT_KEY | Decline F6/F7 |
