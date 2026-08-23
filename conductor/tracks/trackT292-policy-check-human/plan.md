# T292 Plan — policy check human allowed/denied line

**Status:** **Pending** (Planned). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F29 / AC1–AC12 + §13 AI fold-in
**Category:** UX / FEATURE
**Ledger TX (planning):** `84c4b2ec-3930-4d49-bcee-6b0bb3abdce3` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `2eafd304-3287-44d2-9e87-51ce0ed42523` (DOCS)
**Ledger TX (implement):** FEATURE on **go**

---

## AI fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 0**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **AC2/F27 (OpenCode m1):** `hermetic_bin` omit `--principal-id`; not `policy_bootstrap.rs` helpers.
2. **F28 (OpenCode m2):** `CheckOptions` only clap dispatch `:4333`.
3. **F8/F29 (OpenCode O3):** after_help catalog byte-stable with `CAPABILITY_CATALOG`.
4. **F12/AC9 (OpenCode O1):** OPERATIONS exact script `--format json` sentence.
5. **AC6 (Agy O2):** clap `--format Pretty` InvalidValue as well as `JSON`.
6. **AC3 (Agy m2):** stderr no `POLICY_DENIED:`.
7. **F3/F26 AC-id slips:** InvalidValue = AC6; show/bootstrap = AC8.

---

## Preflight (plan time — 2026-08-23; fold-in refresh)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `1331786` (`main`, T292 plan). Parent `ea5c947` T291 `#207`. CLEAN at fold-in start. |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T274. No T285–T291.** Hole is in **source + PATH**. **Do not `cargo install`.** |
| `policy check --capability ReadEvidence` | pretty JSON `{allowed:true,…}`; exit **0** |
| `--format human` allow | `allowed: true (ReadEvidence on Repository:3581317d-…)` exit **0** (already exists) |
| `--format human` ProposeConclusion | exit **3**; stderr POLICY_DENIED + HINT; **stdout empty** |
| `--format auto` | PATH **and** `cargo run` → JSON (parse maps auto → Json) |
| `--help` | `[default: json]` |
| `preflight --summary` | Pinned **4041** (volatile); in-context **0/0/0**; word **664** |
| Last PR comments | #207 T291 — Cursor/Bugbot/reviews **empty**. **N/A. No T301.** |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; GitHub **v4.6.6**; **no clap 5**); serde_json **1.0.150** (crates.io 1.0.151); chrono **0.4.44**; rusqlite **0.39.0** (0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** (3.932) — do not touch. `sync.rs` **#2** (3.619). `governed_common.rs` **#3** (3.604) — import consts only. `policy_cmd.rs` not top-10 — human + resolver here. CLI `preflight.rs` **#8** — do not grow. |
| Ledger | 0 pending / 0 drift at scan; planning TX `84c4b2ec`; fold-in TX `2eafd304` |
| `ISSUES.md` | **Does not exist** (F17) |
| ledgerful search | `run_check` → `policy_cmd.rs:141` / `main.rs:4331` |
| Online | clig.dev human-first + next-command + TTY heuristic; clap 4.6.6 PossibleValuesParser case-sensitive; T180 default lift documented; T266 Family A analog (`scope.rs`) |
| Skill | CAPABILITIES policy show/check row (F12) |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [ ] Re-read `policy_cmd.rs` `run_check` `:141` — deny still `fail_api`; allow still `emit_human` F2 line; format still `OutputFormat::parse`
- [ ] Confirm clap Check still `default_value = "json"` and **no** `value_parser` (`:2340`)
- [ ] Confirm `CheckOptions {` only at struct + `main.rs:4333` (F28)
- [ ] Confirm `hermetic_bin` still strips `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` (F27)
- [ ] Confirm Check after_help catalog still matches `CAPABILITY_CATALOG` (F8/F29)
- [ ] Confirm AC6 clap tests cover `JSON` **and** `Pretty` (not `OutputFormat::parse`)
- [ ] Confirm `OutputFormat::parse` still case-insensitive unknown→Json — AC6 **must** use clap `value_parser` (F3)
- [ ] Confirm `resolve_human_json_format` still `auto` TTY human / pipe json — **do not change the map**
- [ ] Confirm `POLICY_BOOTSTRAP_SOOT_SHORT` exact T241 F14 string — F7 line 2
- [ ] Confirm JSON deny hermetic still forces `--format json` (AC5 stay-green)
- [ ] Confirm `CheckResult` still four keys — **do not add**
- [ ] Confirm Show/Bootstrap clap still default json (F26)
- [ ] Re-scan hotspots — `governed_common.rs` still import-only
- [ ] Rescan `conductor/deferred.md` — T292 absorbed + #207 N/A; T293–T300 / T291 not stolen
- [ ] Confirm #207 still empty Cursor; no mint; Dependabot `#61` still not this track
- [ ] Re-dogfood `policy check --capability ReadEvidence` **read-only**. **Did not** pin production decisions; **did not** write `.env`; **did not** extra `policy bootstrap`
- [ ] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [ ] FEATURE TX (new)
- [ ] Did **not** `cargo install`; did **not** grow `project.rs` / `preflight.rs` / `briefing.rs` / `personal.rs` / `OutputFormat::parse`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit `policy check` JSON-only U=7 | **DoD** F1–F7 / AC1–AC4 / AC10 |
| Placeholder Manual two commands | **DoD** AC10 |
| Placeholder auto TTY / pipe JSON | **DoD** F1 / AC6 / AC7 |
| T266 Family D for check | **Lift** F1; show/bootstrap stay D (AC8) |
| last-PR #207 Cursor | **N/A empty** — no T301 |

---

## Phase 1 — red (required first)

- [ ] `format_policy_check_allow_line__read_evidence__exact_string` (AC1)
- [ ] `format_policy_check_deny_line__propose__exact_string` (AC1) + SHORT `assert_eq!`
- [ ] `policy_check__deny__format_human__denied_plus_short_exit_3` (AC3) — fail while stdout empty; assert stderr has no `POLICY_DENIED:`
- [ ] Commit red allowed

## Phase 2 — green format + deny human

- [ ] clap Check `--format` default `auto` + T266 `value_parser`; `CheckOptions.format: String`; dispatch
- [ ] `run_check`: `resolve_human_json_format` + `stdout().is_terminal()` then Human/Json (F3) — **not** `OutputFormat::parse` on the raw token
- [ ] Deny Human arm: two stdout lines (F7); `GovernedCliError` exit 3; skip `fail_api`
- [ ] Allow path uses `format_policy_check_allow_line` (F2)
- [ ] AC2 allow-human hermetic (`hermetic_bin` System bootstrap omit `--principal-id` on **temp** vault; not `policy_bootstrap.rs` helpers)
- [ ] AC4 json keys; AC6 clap `JSON` **and** `Pretty` InvalidValue + default auto; AC7 pipe omit-format still JSON
- [ ] AC12 pretty/md/text ≡ human allow line

## Phase 3 — stay-green + peers

- [ ] AC5 JSON deny / missing-cap / unknown-cap / soft-resolve / bootstrap suite
- [ ] AC8 show/bootstrap `--help` still default json
- [ ] AC11 CheckResult no new fields

## Phase 4 — docs + gate

- [ ] CAPABILITIES Family A row for `policy check`; OPERATIONS human example + exact script `--format json` sentence; CLI-EXIT-CODES; PROTOCOL-COMPAT §5; Check after_help Examples only (catalog freeze F8); CHANGELOG
- [ ] AC9 hermetic `policy check --help` names auto/TTY; catalog lines still present
- [ ] `cargo fmt --check` ; `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Targeted nextest then workspace gate (`dev-check` / nextest + deny + audit)
- [ ] Manual AC10 `cargo run -p ai-brains-cli -- policy check --capability ReadEvidence --format human` and `--format json`
- [ ] `ledgerful verify --scope full`

## Phase 5 — review + publish

- [ ] `conductor/tracks/trackT292-policy-check-human/review.md` phase-1
- [ ] Codex/cross-model when FEATURE
- [ ] Mark conductor **Completed**; append closeout residuals to `deferred.md`
- [ ] Push `track/T292-*` ; PR ; `gh run watch --exit-status` ; `gh pr merge --squash --delete-branch`
- [ ] Hygiene: `git fetch --all --prune`; point local `main` at `origin/main`; delete merged local `track/T292-*` only. Never `git push origin main`. Never force-push.

---

## DoD (checkable)

- [ ] AC1–AC12 green (hermetic + clap + docs + Manual AC10)
- [ ] JSON deny still one `POLICY_DENIED` document (AC5)
- [ ] Show/bootstrap defaults unchanged (AC8)
- [ ] No `OutputFormat::parse` body change
- [ ] No clap 5 / rusqlite 0.40 / `.env` write / extra live bootstrap / `cargo install`
- [ ] Medium+ review findings not silently dropped
- [ ] FEATURE TX committed; conductor Completed only after publish hygiene
