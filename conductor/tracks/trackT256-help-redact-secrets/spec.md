# T256 — Help redacts secrets

- **Track ID:** T256-HelpRedactSecrets
- **Status:** **Planned** (requirements written; conductor row stays **Pending**. F0 = plan-only until go)
- **Category:** SECURITY / UX
- **Owner:** Grok
- **Source:** Non-destructive CLI audit 2026-08-16 — `ai-brains --help` quality **3**
- **Depends on:** T197 key bootstrap (`AI_BRAINS_KEY=x'<64 hex>'`); clap 4 `env` feature; T204 help IA (do not reopen)
- **Blocks / feeds:** Any future help IA; T257 JSON hygiene does **not** absorb this
- **Absorbs:** `--help` / `-h` print live `AI_BRAINS_KEY`; opportunity “never print `x'<hex>'` in help”; T181 `assert_no_secret_leakage` as the proof helper
- **Not absorbed:** daemon `AI_BRAINS_VAULT_KEY` sidecar; recovery kit; doctor kit-path; `init` generate-and-print (intentional once); `AI_BRAINS_VAULT_PATH` / `AI_BRAINS_PROJECT_ID` env display (not secrets)
- **Research date:** 2026-08-16 (source HEAD `d6749b6`; clap lock **4.6.1** / clap_builder **4.6.0**)
- **Ledger:** planning DOCS TX `0cb0fe21-2325-4b22-917a-4b649d6dadef`. Implement starts a **SECURITY** TX on **go**.
- **Isolation:** Do **not** print, commit, or log a live key. Do **not** reopen T197 resolve, T204 group labels, T240 F2, T255 declines. Do **not** bump clap / add crates.

---

## 1. Objective

`--help` / `-h` (and any clap help that inherits global `--key`) must **never** echo the live vault key or any other secret env default. Keep documenting the **variable name**. How the key is *used* (T197) does not change.

This is a capture-independence unblock: agent transcripts, screenshots of “start here” help, CI `--help` dumps, and support paste-bins currently receive the SQLCipher product key. The vault stays local-first only if help does not leak it.

---

## 2. Live baseline (2026-08-16)

### 2.1 Operator dogfood (this machine)

Classified without printing values (PowerShell match only):

| Surface | Observation |
|---------|-------------|
| HEAD | `d6749b6` — T252–T255 closeout (#169). Tree dirty: conductor placeholders / series READMEs from T256–T271 registration (unrelated to product crates). Ahead of `origin/main` by **0**. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` |
| `ai-brains --help` | `AI_BRAINS_KEY` **present**; `AI_BRAINS_KEY=x'` **true**; env-equals **true**. Length 6181. **Leak confirmed.** |
| `ai-brains -h` | Same leak class. Length 5064. |
| `ai-brains recall --help` | `AI_BRAINS_KEY` **absent**. `--key` is **not** `global = true`, so subcommand help does not inherit it today. |
| `AI_BRAINS_VAULT_PATH=` | Also shown on root help. **Not a secret.** Leave. |
| Preflight | Scope `test-alias` `441837f6` vs path owner `3581317d` (T258). Identity warn on every command (T257). Do not “fix” here. |
| Last GitHub PR | [#169](https://github.com/Ryan-AI-Studios/AI-Brains/pull/169) merged 2026-08-16. `gh pr view --comments`, `/reviews`, `/comments` all **empty**. HEAD is `main` (no open PR). Open PRs #68–#72 are Dependabot only. **last-PR Cursor: N/A.** |
| Ledgerful | `doctor` ready (legacy `.changeguard` / sig-pin / timings warnings). 0 pending at plan start. `search hide_env_values` → **no matches** (knob unused). `ask` local model thinking-only — noted, not blocking. |

### 2.2 Why this still matters

| Residual | Why it is a product hole / why decline |
|----------|----------------------------------------|
| Root `--help` / `-h` echo `AI_BRAINS_KEY=x'<64 hex>'` | Audit quality **3**. Lands in agent sessions (this planning skill forbids pasting `--help` for that reason). **DoD.** |
| Subcommand help | Does not show `--key` today (not global). Still lock a dummy-key hermetic on `doctor --help` so a later `global = true` cannot regress silently. **DoD as belt-and-suspenders.** |
| `init` prints generated key | Intentional one-shot. **Decline.** |
| Hide every clap `env` value | Would hide `AI_BRAINS_PROJECT_ID` / vault path (useful). **Decline.** Secrets only. |
| Command-wide `hide_env_values` | **Not** on `Command` in clap_builder **4.6.0** (Arg-only). **N/A.** |
| Custom help hook / regex scrub | Invented chrome. Official knob exists. **Decline.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| `--key` | `crates/ai-brains-cli/src/main.rs` `Cli.key` **:432** | `#[arg(long, env = "AI_BRAINS_KEY", help_heading = "Global options")]`. **No** `hide_env_values`. **No** `global = true`. |
| `--vault-path` | same, `:428` | `env = "AI_BRAINS_VAULT_PATH"`. Not a secret. |
| Other clap `env` | `AI_BRAINS_PROJECT_ID`, `AI_BRAINS_SCOPE`, `LEDGERFUL_TX_ID`, `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` | Not key material. Do not hide. |
| `migrate governed` keys | `main.rs` `:1971–1979` | `--source-key` / `--destination-key` / local `--key` — **no** `env =`. No clap env display. |
| Doctor after_help | `main.rs` `:633` | Claims “no secrets on stdout”. Format example `x'<64 hex>'` is a **shape**, not a live value. Keep. |
| T197 resolve | `key_resolve.rs` | `--key` trim → `AI_BRAINS_KEY` → Missing. **Do not touch.** |
| Init generate | `commands/init.rs` | Prints generated key once. **Do not touch.** |
| Daemon sidecar | `ai-brainsd/src/vault_key.rs` | `AI_BRAINS_VAULT_KEY` then `AI_BRAINS_KEY`. **Not** a clap `env` on the CLI. |
| Help IA | `help_ia.rs` | T204 group appendix. **Do not rewrite.** |
| Hermetic help suite | `tests/cli_help_ia.rs` | `hermetic_bin()` **sets** `ZERO_SQLCIPHER_KEY`. Today that zero key is what `--help` would echo in CI. New ACs go in a **new** test file so T204 locks stay untouched. |
| Leak helper | `ai_brains_crypto::test_support::assert_no_secret_leakage` | T181 — hex / b64 / raw / Debug. **Reuse.** |
| Dummy zero key | `tests/common/mod.rs` `ZERO_SQLCIPHER_KEY` | `x'00…00'` (64 hex zeros). Do **not** assert “no `x'`” globally — doctor after_help contains the format string. |
| Unset helper | `hermetic_bin_no_key()` | Empty home + `--no-project-context` so global dotenv cannot re-inject. Use for the unset AC. |

### 2.4 Dependency / standards research (2026-08-16)

| Pin | Workspace / lock | Action |
|-----|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** (builder **4.6.0**) | **No bump.** `Arg::hide_env_values` is on **4.6.0** (`clap_builder-4.6.0/src/builder/arg.rs:2667`, `cfg(feature = "env")`). Feature `env` already enabled. |
| crates.io clap | latest **4.6.6** (2026-08-06). **No clap 5.** | Forbidden future-bump guard. Snapshot — re-verify at execute. |
| Official docs | clap 4.6.0 `Arg::hide_env_values(true)` example: `$ CONNECT=super_secret connect --help` omits `[default: CONNECT=super_secret]`. Derive: `hide_env_values = true` (same as historical structopt; rustic `RUSTIC_KEY` uses this). | **This is the implementation.** |
| `serde_json` / `tokio` | unchanged | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.1** | **No bump.** |
| New crates | — | **Zero.** |
| Best practice | OWASP / CISA: never echo secrets in help/debug. Reference CLIs (rustic, structopt cookbook) hide env **values** and keep the **name**. Do not invent `(set)`/`(unset)` chrome unless clap stops showing the name. | Fits this repo: one attribute, existing leak helper. |

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a **SECURITY** TX. |
| **F1 — Knob** | On `Cli.key` only: `hide_env_values = true` next to `env = "AI_BRAINS_KEY"`. Keep `long`, `help_heading`. Do **not** set `hide_env = true` (that hides the name). Do **not** set `global = true`. |
| **F2 — Chrome** | After the change, root help still contains `[env: AI_BRAINS_KEY]` (name). It must **not** contain `AI_BRAINS_KEY=` followed by the live / dummy value, `x'<payload>'` of that value, or the raw 64-hex payload. Clap’s native “name without `=value`” **is** the `(set)` equivalent. Do not invent custom `(set)`/`(unset)` text. |
| **F3 — Surfaces** | Redact `-h` and `--help` on the root parser. Hermetic `doctor --help` with a dummy key must also be clean (defense if inheritance changes). `recall --help` stays a regression lock (name may be absent today). |
| **F4 — Resolve freeze** | Do not change `key_resolve.rs`, dotenv merge, T197 missing/zero/format codes, or how `--key` wins over env. |
| **F5 — Non-secrets stay visible** | `AI_BRAINS_VAULT_PATH=`, `AI_BRAINS_PROJECT_ID=`, `LEDGERFUL_TX_ID=`, principal / scope env display stay as clap default. |
| **F6 — Init print** | `init` may still print a **newly generated** key once. That is not help. |
| **F7 — Daemon key** | `AI_BRAINS_VAULT_KEY` is not a CLI clap `env`. Do not add a clap arg just to hide it. |
| **F8 — Tests** | New file `crates/ai-brains-cli/tests/cli_help_secret_redaction.rs`. Dummy key **must not** be the all-zero fixture (doctor after_help / INSTALL mention `x'<64 hex>'`; zero-hex is too collision-prone). Use a distinctive 64-hex payload + `assert_no_secret_leakage` on the **bytes**. Also assert no `AI_BRAINS_KEY=x'` substring. |
| **F9 — Unset** | `hermetic_bin_no_key()` + `--help`: still documents `AI_BRAINS_KEY` (name). Combined stdout/stderr must not contain a dummy payload (there is none). |
| **F10 — T204 freeze** | Do not edit `help_ia.rs` or existing `cli_help_ia.rs` group-label ACs except if a new assertion is added **additively** (prefer the new file). |
| **F11 — Docs** | CAPABILITIES `--key` row: one honesty clause that help never echoes the live value. INSTALL format examples stay placeholders (`x'<64 hex>'`). Root CHANGELOG T256 row. No PROTOCOL-COMPAT / contracts DTO. |
| **F12 — Pins / crates** | No clap 5, no lock bump, no new crates, workspace **0.1.1**. |
| **F13 — Capture independence** | Help/docs only. No events. No vault open on `--help`. |
| **F14 — Error usage** | Hermetic unknown-flag (`ai-brains --not-a-real-flag`) combined output with dummy `AI_BRAINS_KEY` must not contain the payload. If clap usage-on-error still prints env values, that is **in scope** (same attribute or documented clap limitation + follow-up). Do not ship a silent residual leak. |
| **F15 — Stop-before** | Never print a live `AI_BRAINS_KEY` in review logs, plan evidence, or PR text. Dummy keys in tests only. |
| **F16 — Cross-model** | SECURITY. After Phase-1 review clean, run read-only `codex-review`. |
| **F17 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals go to `conductor/deferred.md`. |
| **F18 — PATH-behind** | Live PATH binary may stay leaky until the user `cargo install`. Tests/manual AC use `cargo run` / hermetic bin. Do **not** `cargo install` unless asked. |
| **F19 — Decline extras** | T257 warn/JSON; T258 Scope rebind; clap 5; command-wide hide; custom help hook; making `--key` global; hiding vault path / project-id. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Hermetic: `hermetic_bin()` + `.env("AI_BRAINS_KEY", DUMMY)` + `--help` exits **0**. Combined stdout+stderr does **not** contain `DUMMY`’s 64-hex (via `assert_no_secret_leakage` on the decoded bytes) and does **not** contain `AI_BRAINS_KEY=x'`. **Does** contain `[env: AI_BRAINS_KEY]` or `AI_BRAINS_KEY` as the documented name. `DUMMY` ≠ `ZERO_SQLCIPHER_KEY`. |
| **AC2** | Same as AC1 for `-h`. |
| **AC3** | `hermetic_bin_no_key()` + `--help` exits **0**. Output contains `AI_BRAINS_KEY`. No dummy payload (none set). |
| **AC4** | `hermetic_bin()` + dummy + `doctor --help` combined output has no dummy payload / no `AI_BRAINS_KEY=x'`. Format string `x'<64 hex>'` in doctor after_help **may** remain. |
| **AC5** | `hermetic_bin()` + dummy + `recall --help` combined output has no dummy payload. |
| **AC6** | `hermetic_bin()` + dummy + `--not-a-real-flag` (expect non-zero): combined output has no dummy payload (F14). |
| **AC7** | Existing `cli_help_ia` group-label tests stay green (T204). |
| **AC8** | `key_resolve` unit suite stays green (no resolve change). |
| **AC9** | Docs: CAPABILITIES `--key` honesty + root CHANGELOG T256. INSTALL placeholders unchanged. |
| **AC10** | No contracts DTO; no pin bumps; no new crate; `key_resolve.rs` / `init.rs` / `help_ia.rs` untouched (or only a comment if forced). |
| **AC11** | Manual (source bin, **do not paste values**): classify `--help` the same way as §2.1 — `has_xquote_assign` **false**, `has_AI_BRAINS_KEY` **true**. PATH-behind may still leak (F18). |

Test names (TDD; fail first):

- `root_long_help__dummy_key_env__does_not_echo_payload`
- `root_short_help__dummy_key_env__does_not_echo_payload`
- `root_long_help__key_unset__still_names_env`
- `doctor_help__dummy_key_env__does_not_echo_payload`
- `recall_help__dummy_key_env__does_not_echo_payload`
- `unknown_flag__dummy_key_env__does_not_echo_payload`

---

## 5. Design notes

### 5.1 Product change (entire body)

```rust
    /// Hex-encoded key for the vault (or dummy)
    #[arg(
        long,
        env = "AI_BRAINS_KEY",
        hide_env_values = true,
        help_heading = "Global options"
    )]
    key: Option<String>,
```

One attribute. Clap 4.6.0 `env` feature already on.

### 5.2 Dummy key for tests

Use a fixed distinctive product-form key, e.g.

`x'deadbeefcafebabe0123456789abcdefdeadbeefcafebabe0123456789abcdef'`

Decode the inner 64 hex to 32 bytes for `assert_no_secret_leakage`. Do not use the live vault key. Do not use all-zero.

### 5.3 Why not a post-filter

A regex scrub on help output can hide a later second leak site and fights clap. The upstream knob is the documented fix for this exact `$SECRET=super_secret --help` case.

---

## 6. Non-goals

- Changing key resolve, dotenv order, or zero-key policy
- Making `--key` global
- Hiding non-secret env defaults
- Daemon `AI_BRAINS_VAULT_KEY` clap surface
- Stopping `init` from printing a generated key
- clap 5 / pin bumps / new crates
- T257 / T258 / T259 identity or warning work
- `cargo install` / live `.env` rewrite
- Printing `(set)`/`(unset)` unless clap stops showing the name (then a one-line help tweak is allowed; prefer staying on clap chrome)

---

## 7. Verification plan

1. **Red:** add `cli_help_secret_redaction.rs` ACs. `cargo nextest run -p ai-brains-cli --test cli_help_secret_redaction` must **fail** on current tree (root `--help` echoes dummy).
2. **Green:** F1 attribute. Same test **pass**. AC7/AC8 green.
3. **Docs:** AC9.
4. Targeted: `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` ; nextest that test + `cli_help_ia` + `key_resolve` units.
5. Phase-1 review → `review.md`. SECURITY → `codex-review`.
6. Manual AC11 with classify-only (no value paste).
7. Full gate before finalize. `ISSUES.md` does not exist.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Reviewer / agent pastes live `--help` | F15; classify-only evidence; dummy keys in tests |
| `hide_env_values` hides the name too | AC1 asserts name remains; F1 forbids `hide_env` |
| Doctor after_help `x'<64 hex>'` false-fails a naive `x'` assert | F8 / AC4 — payload + `AI_BRAINS_KEY=x'` only |
| Usage-on-error still prints env | AC6 / F14 in scope |
| PATH binary stays leaky | F18 honesty |
| T204 help-size / group tests flake | New file; do not rewrite appendix |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-16 (post-P12 through T255 closeout + T256–T271 map + T241 lows + historical T142–T196).

| Item | Disposition |
|------|-------------|
| `--help` prints live `AI_BRAINS_KEY` (audit quality 3) | **Absorb** F1–F3 / AC1–AC2 |
| T181 `assert_no_secret_leakage` | **Absorb** F8 as proof helper (do not fork) |
| Doctor after_help “no secrets on stdout” | **Decline as DoD** — already a doctor claim; this track is root clap help. Doctor `--help` still locked by AC4. |
| `init` generate-and-print | **Decline** F6 (intentional) |
| Daemon `AI_BRAINS_VAULT_KEY` sidecar / Session 0 `daemon.env` quoting | **Decline** F7 — not clap help. Already closed as product path 2026-08-16. |
| T257 identity warn / JSON interleave | **Point** — do not steal |
| T258 daily Scope / T259 leftover `7d97a456` | **Point** — identity, not help |
| T260–T271 remainder of this series | **Point** — not help |
| T255 declined bag (doctor 16th, persist probe, `.cmd`, clap 5) | **Decline** — stay declined |
| T240 F2 no silent Scope switch | **Decline** — not this track |
| T204 help IA / group labels | **Decline reopen** F10 |
| T197 key bootstrap | **Depends** — do not change F4 |
| R-CI-BRANCH / MSI / notarization / App Store | **Not related** — packaging / admin |
| `anyhow` RUSTSEC-2026-0190 allowlist | **Not related** |
| `#34.2` DataKey rotation | **Closed** T189 — not related |
| T142 archive `changeguard` strings | **Not related** |
| T210–T232 / T234–T255 soft residuals | **Not related** unless they mention help-key leak (they do not) |
| last-PR Cursor (#169 + open HEAD PR) | **N/A** — #169 comments/reviews/inline all empty; HEAD is `main`; open PRs are Dependabot #68–#72 (no Cursor/Bugbot findings). **No leftover to mint.** |
| Closed/strikethrough deferred rows | Stay closed |

---

## 10. Implement order (on go)

1. Phase 0: re-verify clap lock + `hide_env_values` still on builder; rescan `deferred.md` + last PR Cursor.
2. Red: `cli_help_secret_redaction.rs` (AC1–AC6 names).
3. Green: F1 attribute on `Cli.key`.
4. Docs AC9.
5. Targeted clippy + nextest.
6. Review + SECURITY codex-review.
7. Manual AC11 classify-only.
8. Full gate; conductor **Completed**; deferred closeout line.

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| PATH `ai-brains` still leaky until reinstall | F18 — operator |
| `AI_BRAINS_VAULT_PATH=` on help | Intentional (F5) |
| `init` one-shot print | F6 |
| Daemon env file contents | F7 / T145 ACL — not help |
| Command-level hide (if clap adds it later) | Not needed; Arg-level is enough |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/main.rs` | `hide_env_values = true` on `Cli.key` only |
| `crates/ai-brains-cli/tests/cli_help_secret_redaction.rs` | **New** hermetic ACs |
| `Docs/CAPABILITIES.md` | `--key` honesty clause |
| `CHANGELOG.md` | T256 row |
| `conductor/tracks/trackT256-help-redact-secrets/{spec,plan}.md` | This plan |
| `conductor/conductor.md` | Pending row text |
| `conductor/deferred.md` | Absorb note |

Do **not** touch: `key_resolve.rs`, `init.rs`, `help_ia.rs`, `daemon.rs` wrapper env, `ai-brainsd`, contracts, `project.rs` (hotspot #1).
