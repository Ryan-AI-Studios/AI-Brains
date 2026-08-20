# T273 Plan — sync query dash-leading Ledgerful flags

**Status:** **Completed** (local gate green 2026-08-20; publish Phase 6)
**Spec:** [spec.md](./spec.md) F0–F23 / AC1–AC14 + §13 AI fold-in
**Category:** BUGFIX
**Ledger TX (planning):** `1d4391ae-3769-4cfa-9d04-8be1c7f138bd` (DOCS)
**Ledger TX (fold-in):** `0d001d8e-0608-4ba0-8ac2-fb9d836c71b4` (DOCS)
**Ledger TX (implement):** `20892666-f2ed-4710-b05f-371b02200567` (BUGFIX)

---

## AI fold-in (2026-08-19) — `agy-review.md` + `opencode-review.md`

Agy: **Planned**, no B/M. OpenCode: **Planned after fixes**, **B-1** (AC10 flag order). Disposition in spec **§13**.

### Pins locked by fold-in

1. **F21 / AC10:** `sync query --no-bridge -- --limit` (flags before `--`). Never `-- --limit --no-bridge`.
2. **F23 / AC14:** `--quiet -- --limit` required; pane must print.
3. **AC4:** needle `"--"`.
4. **F22 / AC8:** `ErrorKind::InvalidValue` + `--limit <LIMIT>` (clap 4 empty option value; T247 analog was wrong).
5. **F6:** `after_help` contrasts needle vs vault `--limit 10`.

---

## Preflight (plan time — 2026-08-19)

| Check | Result |
|-------|--------|
| HEAD / tree | `f3f6cbd` CLEAN; `main` == `origin/main` (T268 #184 merged) |
| T273 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | **0.1.1**. **Do not `cargo install`.** |
| Live hole | `cargo run -- sync query -- --limit` → `Ledger search failed: error: a value is required for '--limit <LIMIT>'` |
| Ledgerful control | `ledger search --json --limit` exit 2; `ledger search --json -- --limit` exit 0 **≥1** rows (volatile T211/T217) |
| Layer 1 | `sync query --days` → clap tip `use '-- --days'`; quoted `"--limit"` still hits vault `--limit` |
| SoT | `sync_query_ledger.rs:157` `cmd.args(["ledger","search","--json", query])` |
| Hotspots | `project.rs` #1 / `sync.rs` #2 — do not grow. Helper stays in sibling. |
| clap / serde_json | lock clap **4.6.1** / crates.io **4.6.6**; serde_json lock **1.0.150** / crates.io **1.0.151**. rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute |
| Last PR Cursor | #184 Bugbot Medium Linux Path units — **already `#[cfg(windows)]`**. Decline. #183 dash-query still this track. No open PR on `main` |
| `deferred.md` | Full scan. Overlap: #183 dash-query **absorb**; T90 argv **affirm decline**; T269–T272 / #184 / recall bridge / historical CE-wipe **decline** |
| ai-brains | `preflight --summary` Scope `3581317d`; pins **3146**; grants 0 of 3 (T241) |
| ledgerful | doctor ready (hygiene warns). 0 pending 0 drift at scan. Index incremental 1 file. TX `1d4391ae` |
| Research | clap 4.6.6 `--` / `allow_hyphen_values` (known flags win); POSIX `--`; live Ledgerful `--` accepted |
| `ISSUES.md` | **Does not exist** |
| Live `.env` / bootstrap / nightly mutate / pin | **Not written** / **not run** / **not scheduled** / **not pinned** this pass |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Dash QUERY parsed as Ledgerful flags | #183 Bugbot / T271 closeout / stub | **DoD** F1–F4 / AC1–AC5 / AC9 |
| Rescue never starts after clap fail | #183 body | **DoD** F4 (T271 F6 stands; argv is the remediator) |
| Stub F1–F4 | placeholder | **Absorb** |
| T90 on ledger argv | T90 / T271 F5 | **Affirm** F3 |
| Query `after_help` missing | live `main.rs` Query | **DoD** F6 / AC12 |
| T271 miss classes / `--no-bridge` / quiet | T271 | **Affirm** F4 / AC6 / AC10 |
| AC10 flags after `--` | OpenCode B-1 | **Folded** F21 / AC10 — `--no-bridge -- --limit` |
| `"--"` helper needle | Agy m1 | **Folded** AC4 |
| AC8 ErrorKind | OpenCode O-1 | **Folded** F22 |
| Quiet honesty optional | OpenCode O-2 | **Folded** AC14 required |

## Declined (written)

| Item | Why |
|------|-----|
| last-PR #184 Linux Path units | Already `#[cfg(windows)]` + Unix unit; T268 P1 fixed. No T274 |
| recall `bridge_search_args` | F7 soft — `ledgerful search` code, not this pane |
| Our Query `allow_hyphen_values` | Known `--limit` still wins (clap 4.6.6) |
| `sync query --limit` as needle | Vault cap; AC8 |
| T269 / T270 / T272 | Peers |
| T211 F25 / T217 MATCH OR | Other SOOT |
| Ledgerful source / token-OR | Other repo; T271 F23 |
| clap 5 / pin bumps / DTO | F10 / F13 |
| T240 F2 / T255 bag | Standing |
| Historical deferred.md (CE wipe, connector cursor, `anyhow` allowlist, archive changeguard, MSI) | No overlap |

---

## Phase 0 — on go (re-verify)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact`
- [x] Re-read `run_ledger_search` + live `ledgerful ledger search --help`
- [x] Confirm `ledgerful ledger search --json -- --limit` still exit 0
- [x] Confirm source `sync query -- --limit` still prints `--limit <LIMIT>` required (pre-green)
- [x] Re-check lock clap **4.6.1** / serde_json **1.0.150**. rustc **1.95.0**. No clap 5 (crates.io clap 4.6.6)
- [x] Rescan **entire** `conductor/deferred.md`
- [x] Last merged PR Cursor comments (plus open PR on HEAD) — #184 Linux units declined F8; #183 is this track
- [x] BUGFIX TX start `20892666-f2ed-4710-b05f-371b02200567`

---

## Phase 1 — Red (commit allowed)

- [x] Unit `ledger_search_argv__json_dash_limit__end_of_options_before_query` (AC1)
- [x] Unit `ledger_search_argv__human_dash_limit__no_json_flag` (AC2)
- [x] Unit `ledger_search_argv__plain_phrase__still_emits_double_dash` (AC3)
- [x] AC4 needles (`--days`, `--breaking`, `--json`, `-l`, `-d`, `-b`, `"--"`) as separate tests (no rstest dep on cli crate; F13)
- [x] Clap `sync_query__posix_end_of_options__limit_is_query` (AC7)
- [x] Clap `sync_query__bare_limit_flag__still_requires_value` asserts `ErrorKind::InvalidValue` + `--limit <LIMIT>` (AC8 / F22)
- [x] Prove they fail on current tree before green (E0432 unresolved `ledger_search_argv`)

---

## Phase 2 — Green

- [x] `pub(crate) fn ledger_search_argv(query: &str, json: bool) -> Vec<String>`
- [x] `run_ledger_search` uses **only** that helper (JSON + human)
- [x] Empty-query never-ran still happens **before** spawn (T271 F18)
- [x] Query `after_help` contrasts `sync query -- --limit` (needle) vs `sync query "text" --limit 10` (vault cap) (F6 / AC12)
- [x] No T90; no `allow_hyphen_values` on Query.query; no `sync.rs` / `project.rs` / `recall.rs` edits

---

## Phase 3 — Stay green + docs

- [x] T271 units AC1–AC19 (forwarder / rescue / miss / classifier / json_non_empty)
- [x] `sync_query__no_bridge__skips_ledgerful_section` (AC6)
- [x] T211 ranking + T231 resolve hermetics (AC13)
- [x] CAPABILITIES T271 pane bullet + CHANGELOG T273 row (AC12)
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [x] `cargo nextest run -p ai-brains-cli --bins` + named clap/argv tests (crate has no `--lib`)

---

## Phase 4 — Manual (on go)

- [x] AC9: `cargo run -p ai-brains-cli -- sync query -- --limit` → ledger hits **or** ran-empty `'--limit'`; **not** `--limit <LIMIT>` required
- [x] AC10: `cargo run -p ai-brains-cli -- sync query --no-bridge -- --limit` → Recall, **no** ledger pane (flags **before** `--`)
- [x] AC11: `ledgerful ledger search --json -- --limit` ≥1
- [x] AC14 (required): `cargo run -p ai-brains-cli -- sync query --quiet -- --limit` **prints** the ledger pane

---

## Phase 5 — Review + publish

- [x] `conductor/<track>/review.md` (post-execute)
- [x] Medium+ not silently dropped
- [x] Residuals appended to `deferred.md`
- [x] Local gate: `dev-check.ps1` 3206 passed / 1 skipped; `ledgerful verify --scope full` passed
- [ ] conductor **Completed** after implement-track Phase 6 (push → PR → GHA green → squash-merge → prune)
- [ ] Never `git push origin main` / force-push

---

## Definition of done

- [x] AC1–AC14 green or manual-recorded
- [x] T271 suite still green
- [x] F0 was respected (no product commits as “planning”)
- [ ] Ledger BUGFIX TX committed; 0 pending / 0 drift
- [ ] Spec header + registry Completed **after** merge hygiene
