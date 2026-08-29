# T324 — PowerShell empty TERM on `in-force`

- **Track ID:** T324-PowershellEmptyTerm
- **Status:** **Planned** (Pending until **go**)
- **Category:** BUGFIX / UX / WINDOWS
- **Owner:** Grok
- **Source:** T311 residual **R7** / review **I3** — Windows PowerShell `""` drops the empty argv slot so clap reports missing `<TERM>`, not `fail_usage`. Series README `README-T312-T324-CLI-DOGFOOD.md`.
- **Depends on:** T311 ✅ empty-term usage exit 2 (hermetic `.arg("")`); T323 ✅ conclusion `in-force` copies the same required positional
- **Blocks / feeds:** Windows PowerShell 5.1 operators who type `decision in-force ""` (and, after PATH catch-up, `conclusion in-force ""`). Does **not** steal T325 / T326 / T307. Does **not** retune TERM matching.
- **Absorbs:** T311 R7. T323 same clap hole (live `term: String` required positional on both `DecisionCommands::InForce` and `ConclusionCommands::InForce`).
- **Not absorbed (DoD):** clap **5**; docs-only `--%` / `'""'` as empty (live they are **not** empty); every other required positional on the CLI; T322 `--as-of` empty; T314 progressive `--dry-run` (Completed); T325 / T326; daemon DTO; H2
- **Research date:** 2026-08-29 (plan-write product HEAD `5b50d56` T323 `#245`). Snapshot — **re-verify at execute**.
- **Ledger:** planning DOCS TX `3b998d33-ac46-4a8c-9074-aebcc5931e46`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **BUGFIX** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install`. Do **not** bump clap. Do **not** grow `governed_common.rs` (#3) / `help_ia.rs` / `project.rs` (#1) / `sync.rs` (#2). Do **not** edit CP `in_force.rs` / `conclusion_in_force.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** propose/approve/correct on the live vault.

---

## 1. Objective

1. **Empty term is reachable from Windows PowerShell 5.1.** `ai-brains decision in-force ""` (5.1 drops the slot) and `ai-brains decision in-force --term=` both exit **2** with `term must be non-empty`, not clap `the following required arguments were not provided: <TERM>`. Same clap shape on `conclusion in-force`.
2. **Named `--term` is the documented empty invocation.** T314-shaped optional-value flag: `num_args = 0..=1`, `default_missing_value = ""`, `ArgAction::Set`, **no `require_equals`**. Bare `--term` (5.1 leftover after dropping `""`) is empty. `--term workspace_id` still binds a value.
3. **Hermetic `.arg("")` and whitespace stay green.** Message freeze: `term must be non-empty`.
4. **North star.** Capture independence: clap/usage only. No events. No models. No graph.

This unblocks: T311 already fail_usage's empty/whitespace **when the argv slot exists**. Windows PowerShell 5.1 (and pwsh `$PSNativeCommandArgumentPassing = Legacy`) omits that slot. pwsh 7.6 default **Windows** mode on this machine already preserves `""` — the product is still Windows-first 5.1.

---

## 2. Live baseline (re-scan 2026-08-29)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `5b50d56` `feat(conclusion): T323 in-force walker (Active\|Confirmed, no as-of) (#245)`. Branch `track/T324-powershell-empty-term`. `origin/main` = `5b50d56` (ahead **0** at plan-write). Tree dirty: uncommitted T323 Completed conductor + residuals — **absorbed into this DOCS commit**. |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. **T311 on PATH.** T312–T323 **not**. T323 `conclusion in-force` is **source-only** (PATH `unrecognized subcommand` exit 1). **Do not `cargo install`.** |
| `preflight --summary` (PATH) | Pinned **4645**. In-context **0/0/0**. `Total Word Count: 753` (PATH-behind T315). **Not this DoD.** |
| Shell | Agent **pwsh 7.6.0-rc.1**, `$PSNativeCommandArgumentPassing = Windows` (platform default). Host also has **Windows PowerShell 5.1.26100.9168** (`powershell.exe`). |
| pwsh 7.6 Windows `decision in-force ""` | **`term must be non-empty` exit 2.** Empty argv **preserved**. Same for `''`. |
| pwsh 7.6 Legacy `decision in-force ""` | clap **missing `<TERM>`** exit 2. Drop confirmed. |
| pwsh 7.6 Standard `decision in-force ""` | `term must be non-empty` (preserved). |
| **powershell.exe 5.1** `decision in-force ""` | clap **missing `<TERM>`** exit 2. **This is the hole.** Same for `''`. No-args is the same clap text. Whitespace `"   "` still fail_usage (non-empty token). |
| `'""'` / `` `"`" `` / `--% ""` (pwsh) | JSON `"term": "\"\""` (two quote chars), `ruling: null`, **exit 0**. **Not empty.** Docs-only workarounds **fail IA**. |
| PATH `--term` / `--term=` | `unexpected argument '--term'` exit 2. Flag **does not exist**. |
| PATH `conclusion in-force ""` | unrecognized subcommand (T323 PATH-behind). Source clap is the same required `term: String`. |
| Last GitHub PR | [#245](https://github.com/Ryan-AI-Studios/AI-Brains/pull/245) T323. `mergedAt` **2026-08-29T19:52:25Z**. Issue/review/inline comments **[]**. Open PRs: **none**. `#237` Bugbot already **T326**. `#230` already **T325**. **No T327 from Cursor.** |
| Ledger | 0 pending / 0 drift before this DOCS TX. Doctor 5 warn (impact-stale at scan start; legacy `.changeguard`; sig-pin; sig-version; timings-0) — hygiene, not this DoD. Impact **LOW** (conductor-only dirty). |
| Hotspots | CLI `project.rs` **#1** (3.623). `sync.rs` **#2**. `governed_common.rs` **#3**. `forget.rs` **#5**. **Do not grow those.** `main.rs` clap / `decision.rs` / `conclusion.rs` not top 10. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this residual still matters

| Residual | Why it is still a product hole / why decline extras |
|----------|-----------------------------------------------------|
| 5.1 `in-force ""` → clap missing `<TERM>` | T311 AC5 / CLI empty-term is **fail_usage** when argv exists. 5.1 operators (Windows-first) never reach it. Omit and dropped-empty are **indistinguishable** after the drop — both must become fail_usage (F3). **DoD.** |
| Named `--term` | Non-empty token that 5.1 cannot drop. `--term=` is one token. `--term` leftover after dropping `""` needs `default_missing_value = ""`. Placeholder preferred a flag over a parser hack. **DoD.** |
| Docs-only `--%` / `'""'` | Live on this pwsh they pass **two quote characters** and return unknown-term JSON exit **0**. 5.1 `--%` is fragile with remaining tokens. **Decline as DoD** (F30). after_help must **not** recommend them. |
| pwsh 7.6 Windows already preserves `""` | Honesty: the agent shell is not the hole. 5.1 + Legacy still are. Product stays Windows-first. **DoD is 5.1 + `--term=`, not “pwsh already works.”** |
| Every other required positional (`recall`, `query expand`, …) | Series non-goal. **Decline bag.** |
| T322 `--as-of ""` | Named flag without `default_missing_value`; 5.1 drop → clap “value required for `--as-of`”. Different hole. **Decline** (F6). |
| clap 5 | Standing. **Decline.** |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| Decision clap | `main.rs` `DecisionCommands::InForce` `:2949–2971` | `term: String` + `value_name = "TERM"`. `--scope` optional. `--format` 7-token default json. `--as-of` T322 Rfc3339. **No `--term`.** |
| Conclusion clap | `ConclusionCommands::InForce` `:2889–2904` | Same required `term: String`. **No `--as-of`.** **No `--term`.** |
| Dispatch | Decision `:5043–5058`; Conclusion `:4996–5009` | `term: term.clone()` into `InForceOptions { term: String, ... }`. |
| `run_in_force` | `decision.rs:243–245`; `conclusion.rs:206–208` | `if options.term.trim().is_empty() { return fail_usage("term must be non-empty"); }` — **freeze message**. |
| `fail_usage` | `governed_common.rs:342–346` | stderr + `EXIT_USAGE` **2**. **Do not grow this file.** |
| CP empty | `in_force.rs:88`; `conclusion_in_force.rs:63` | Same string. **Do not edit CP.** CLI fail_usage runs first. |
| Hermetic empty | `decision_in_force.rs:129–157` loops `""` and `"   "`. Conclusion rstest `:132–161`. | Stay-green (F8). |
| Help AC8 | both files `:81` `stdout.contains("<TERM>")` | **Update** to name `--term` + `TERM` (clap will likely render `[TERM]` once optional). Do **not** freeze angle brackets. |
| after_help | Decision parent `:2909` + InForce `:2947`; Conclusion parent `:2854` + InForce `:2887` | Additive `--term=` / `--term workspace_id` examples. |
| `help_ia.rs` | Governed already names `conclusion, decision` `:13` | **Freeze.** Do not edit. |
| CAPABILITIES | Family C rows `:117–118` | Additive `--term` sentence. Default json freeze. |
| OPERATIONS | `:290–293` / `:1037–1038` | Additive `--term=` empty example. |
| CLI-EXIT-CODES | `:13` usage **2** already includes clap missing **and** `fail_usage` | Additive: in-force omit / empty / `--term=` is `fail_usage`, not clap missing `<TERM>`. |

**Pick:** optional positional + named `--term` (T314 optional-value analog). Missing both → merge to `""` → existing fail_usage. Do **not** keep clap-required positional (that leaves 5.1 `""` as missing-arg).

### 2.4 Dependency / standards research (2026-08-29) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **`4.5`** / lock **4.6.1** (checksum `1ddb117e…`) | crates.io **4.6.6** (2026-08-06). GitHub clap-rs latest **v4.6.6**. **No clap 5.** | **No bump.** Optional-value is in 4.5.30+ (`#5912`); lock **4.6.1** already has it (T314). |
| `time` | `"0.3"` / lock **0.3.47** | crates.io 0.3.55 | **No bump.** |
| `serde_json` | lock **1.0.150** | — | **No bump.** JSON keys freeze. |
| `rusqlite` | exact **0.40.2** | — | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| workspace | **0.1.3** | — | **No bump** |
| New crates | — | — | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| 5.1 drops empty native argv | [PowerShell#6280](https://github.com/PowerShell/PowerShell/issues/6280) (closed, Breaking-Change, Resolution-Fixed in 7.3). StackOverflow: empty `""`/`''` quietly omitted on WinPS and pwsh ≤7.2. | Hole is **real on 5.1**. Live reproduced `powershell.exe` 5.1.26100.9168. |
| 7.3+ empty preserved | [about_Parsing (7.6)](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_parsing?view=powershell-7.6) — `$PSNativeCommandArgumentPassing`: `Legacy` / `Standard` / `Windows`. Default **Windows** on Windows, **Standard** elsewhere. Empty strings preserved in Standard; Windows ≈ Standard except cmd/`.bat`/`.cmd` Legacy. **This host:** Windows default **does** preserve `""`; **Legacy does not.** | DoD is 5.1 + Legacy analog (omit-TERM hermetic), not “pwsh 7 already works.” |
| `--%` stop-parsing | Same Learn article. Remaining line is literal until newline/pipe. Live `--% ""` on pwsh passed **two quote chars** (exit 0 unknown term). | **Decline as empty SOOT.** Fragile with `;` / `2>&1`. |
| `'""'` workaround | Common 5.1 advice. Live pwsh: term is `"\"\""`, exit 0. | **Decline as empty SOOT.** after_help must not recommend it. |
| Optional-value flag | clap 4.6.6 `Arg::default_missing_value`: value when flag present but no value. docs still *mention* `require_equals(true)` for `--color`. T314 / clap#5909: `num_args(0..=1)` + `ArgAction::Set` + `default_missing_value` **without** `require_equals` so space form works. Busybox example on docs.rs uses `default_missing_value` without equals. | **No `require_equals`** — 5.1 `--term ""` drop must leave a bare `--term` that takes the missing default. `--term=` also works without equals. |
| `Arg::long` is not positional | clap 4.6.6: default Arg is positional; `short`/`long` turn it into an option. `index` “shouldn’t be used with long.” | **Two fields** (optional positional `term` + `term_flag` `long = "term"`). Merge in dispatch. |
| clig.dev | Flags are full-length; humans first; do not lie in help. | `--term` is the empty path; do not document a quoting spell that is not empty. |
| Exit 2 | `Docs/CLI-EXIT-CODES.md` — clap missing **and** `fail_usage` are both **2**. T241/T203 already prefer fail_usage over clap “required arguments” for operator-empty. | Omit-TERM → fail_usage matches that precedent. |

**N/A:** SQLCipher, schtasks, Windows service, llama.cpp `/health`, FTS5, T307 reqwest/tower-http, Index SQL, clap 5 (not this bump).

---

## 3. Frozen decisions

- **F0** plan-only until **go**. No product commits as planning. No `cargo install`.
- **F1** Named `--term` on **both** `decision in-force` and `conclusion in-force` (copy-not-share clap). T314 analog: `num_args = 0..=1`, `default_missing_value = ""`, `action = clap::ArgAction::Set`. **No `require_equals`.**
- **F2** Positional `TERM` becomes `Option<String>` (not clap-required). Dispatch merge `term_flag.clone().or_else(|| term.clone()).unwrap_or_default()` into existing `InForceOptions.term: String`. **Do not** change `run_in_force` signature beyond receiving the already-merged String.
- **F3** Omit both → fail_usage `term must be non-empty` exit **2**. This **is** the 5.1-drop analog. Not clap `required arguments were not provided`.
- **F4** T311/T323 JSON keys freeze (`term`/`scope`/`ruling`/`chain`; decision additive optional `as_of`). No `next_step` on in-force. No conclusion `--as-of`.
- **F5** `--format` 7-token `value_parser` freeze; default **json**.
- **F6** Decision `--as-of` freeze (T322). Do **not** add `default_missing_value` / optional-value to `--as-of`.
- **F7** after_help SOOT (both commands + parents, additive): `ai-brains decision in-force --term=` (empty) and `ai-brains decision in-force --term workspace_id` (or positional `workspace_id`). Same for conclusion. **Do not** document `'""'` or `--% ""` as empty.
- **F8** Hermetic `.arg("")` and `"   "` stay fail_usage exit 2 (T311/T323 AC5).
- **F9** `--term` (no value) and `--term=` fail_usage exit 2, same message.
- **F10** `--term workspace_id` and positional `workspace_id` still parse. Stay-green `format_nope` with positional.
- **F11** Both positional **and** `--term` → clap conflict exit **2** (`conflicts_with`).
- **F12** Do **not** edit CP `in_force.rs` / `conclusion_in_force.rs` / store projectors / `conclusions.rs` production.
- **F13** Decline daemon `ListInForce` / contracts DTO.
- **F14** Decline T263 H2.
- **F15** Capture independence: clap/usage/docs only.
- **F16** No `unwrap`/`expect`/`panic` in production. Merge uses `or_else` + `unwrap_or_default` (empty String, not panic).
- **F17** Test names `function_or_feature__condition__expected_result`.
- **F18** Implement TX is **BUGFIX** (not FEATURE). Planning is DOCS.
- **F19** `conductor/ISSUES.md` does **not** exist.
- **F20** PowerShell `;` not `&&`.
- **F21** after_help required (F7).
- **F22** 80-net vs go HEAD: clap fields + dispatch merge + after_help + hermetic tests. Do **not** grow `main.rs` test blocks (put new tests in existing `decision_in_force.rs` / `conclusion_in_force.rs`).
- **F23** Isolation: no live vault lifecycle writes; no `.env` rewrite.
- **F24** Stay-green T311/T322/T323 walker, JSON, `--as-of`, deny exit 3, empty-term **message**. AC8 help **updates** (name `--term` + `TERM`; do **not** freeze `<TERM>` angle brackets — optional positional likely renders `[TERM]`).
- **F25** last-PR `#245` empty / `#237` → **T326** / `#230` → **T325** / **no T327**.
- **F26** Decline peers: all other positionals; clap 5; T325; T326; T307; T308 floors; T240 F2.
- **F27** PATH-behind T312–T323. T311 `decision in-force` **is** on PATH (5.1 hole is live). T323 `conclusion in-force` is source/`cargo run`/hermetic SoT until owner install.
- **F28** No live `decision propose` / `conclusion propose` as proof.
- **F29** Copy-not-share clap (two derive sites). **Do not** add a shared helper in `governed_common.rs`.
- **F30** Decline docs-only `--%` / `'""'` as DoD (live not empty).
- **F31** Decline extra CLI: no `--local`/`--daemon` on in-force; no `--term` on `propose`; no `conclusion list`/`show`; no `--as-memory`.
- **F32** `main.rs` additive clap + dispatch only. Do not grow `governed_common.rs` / `help_ia.rs` / `project.rs` / `sync.rs` / `forget.rs` production.
- **F33** Docs honesty: CAPABILITIES Family C, OPERATIONS, CHANGELOG Unreleased, CLI-EXIT-CODES additive fail_usage-omit.
- **F34** `--term` binds the next non-flag token (T314 F34 analog). Empty examples use `--term=` (one token). Named value examples use `--term workspace_id`. Do not put a second positional after bare `--term`.
- **F35** No clap 5. No `require_equals`.
- **F36** fail_usage / CP string exact `term must be non-empty`.
- **F37** `conflicts_with = "term"` on the `--term` field (`term_flag`).

---

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | `decision_in_force__omitted_term__fail_usage_exit_2` — hermetic no TERM argv → exit **2**, stderr contains `term must be non-empty`, **not** `required arguments were not provided`. |
| **AC2** | `decision_in_force__term_flag_no_value__fail_usage_exit_2` — `--term` with no following value → same fail_usage. |
| **AC3** | `decision_in_force__term_flag_equals_empty__fail_usage_exit_2` — `--term=` → same fail_usage. |
| **AC4** | Existing `decision_in_force__empty_term__exit_2` stay-green (`""` and `"   "`). |
| **AC5** | `decision_in_force__term_flag_workspace_id__format_nope__clap_exit_2` — `--term workspace_id --format nope` is clap InvalidValue exit 2 (named value binds). |
| **AC6** | Existing `decision_in_force__format_nope__clap_exit_2` stay-green (positional). |
| **AC7** | `decision_in_force__positional_and_term_flag__clap_conflict_exit_2` — both present → clap conflict exit 2. |
| **AC8** | Help lists `--term` and `TERM` (angle or square). after_help contains `--term=`. Does **not** recommend `'""'` or `--%`. |
| **AC9** | Conclusion mirrors AC1–AC8 (`conclusion_in_force__…`). |
| **AC10** | Existing deny exit **3** stay-green (decision + conclusion). |
| **AC11** | Existing JSON `ruling` key stay-green. |
| **AC12** | Existing T322 `--as-of` help example / parse stay-green (do not break `parse_as_of_rfc3339`). |
| **AC13** | `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`; nextest `decision_in_force` + `conclusion_in_force`. |
| **AC14** | Docs: CAPABILITIES Family C names `--term`; OPERATIONS has `--term=` empty example; CLI-EXIT-CODES notes in-force omit is fail_usage; CHANGELOG Unreleased. |
| **AC15** | **Manual (powershell.exe 5.1):** `ai-brains decision in-force ""` → `term must be non-empty` exit 2, **not** missing `<TERM>`. Source `cargo run` if PATH still pre-T324. |
| **AC16** | **Manual (powershell.exe 5.1):** `ai-brains decision in-force --term=` → same fail_usage. |
| **AC17** | **Manual (pwsh Windows):** `decision in-force ""` stay fail_usage (already true today). |
| **AC18** | **Manual source:** `cargo run -p ai-brains-cli -- conclusion in-force --term=` → fail_usage (PATH-behind T323/T324). **No** live propose. |

---

## 5. Design notes

### 5.1 Clap shape (copy-not-share; both InForce structs)

```rust
/// Term to resolve (e.g. workspace_id)
#[arg(value_name = "TERM")]
term: Option<String>,
/// Named term (PowerShell 5.1: `""` is dropped; use `--term=` for empty)
#[arg(
    long = "term",
    value_name = "TERM",
    num_args = 0..=1,
    default_missing_value = "",
    action = clap::ArgAction::Set,
    conflicts_with = "term"
)]
term_flag: Option<String>,
```

Dispatch (both matches):

```rust
term: term_flag.clone().or_else(|| term.clone()).unwrap_or_default(),
```

`InForceOptions.term: String` unchanged. `run_in_force` trim-empty gate unchanged.

### 5.2 Why omit → fail_usage (not keep clap-required)

After 5.1 drops `""`, argv is identical to `in-force` with no term. Keeping a clap-required positional **cannot** turn dropped-empty into fail_usage. T203/T241 already prefer fail_usage over clap “required arguments” for operator-empty. Help will show `[TERM]` — AC8 updates.

### 5.3 Why `--term=` is the documented empty (not `'""'`)

Live pwsh `'""'` / `--% ""` / backtick-empty sent two quote characters and returned **unknown-term JSON exit 0**. That is a silent wrong-empty. `--term=` is one non-empty token whose value is empty. Bare `--term` is the 5.1 leftover of `--term ""`.

### 5.4 Conclusion is in scope

Placeholder text said decision-only because T323 was not shipped. Live source now has the same required positional. Copy-not-share clap. Do not edit the walker.

---

## 6. Non-goals

- clap 5 / pin bump / `require_equals`
- Docs-only `--%` / `'""'` as the empty path
- Fixing `recall ""` / `query expand ""` / every required positional
- Optional-value `--as-of` / date-only `--as-of`
- Daemon DTO / H2 / `conclusion list` / `memory show`
- Growing `governed_common.rs` / `help_ia.rs`
- Editing CP resolvers or store projectors
- `cargo install` / live propose
- T325 F8 recency / T326 pin-count / T307 tower-http

---

## 7. Verification plan (TDD)

**Red first (must fail on missing `--term` / still-required positional):**

- `decision_in_force__omitted_term__fail_usage_exit_2` — today: clap missing `<TERM>` (assert fail_usage message → **fail**)
- `decision_in_force__term_flag_no_value__fail_usage_exit_2` — today: unexpected `--term` → **fail**
- `decision_in_force__term_flag_equals_empty__fail_usage_exit_2` — same
- `conclusion_in_force__omitted_term__fail_usage_exit_2` / `__term_flag_no_value__` / `__term_flag_equals_empty__`
- Stay-green T311/T323 empty `.arg("")` still **pass** on the red commit

**Green:** F1–F11 clap + dispatch merge; AC8 help; AC9 conclusion copy.

**Docs:** AC14.

**Manual:** AC15–AC18. 5.1 is Complete-blocking for the hole; pwsh Windows is stay-green.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Help `[TERM]` breaks AC8 `<TERM>` | **Update** AC8 (F24). Not a product regression. |
| `--term` steals next positional | F34; conflicts_with; examples use `--term=` / `--term workspace_id`. |
| `default_missing_value = ""` + clap assert wants `require_equals` | T314 already ships `num_args 0..=1` + missing default **without** equals (lock 4.6.1). Re-verify at execute. If clap panics, **Stop-Before** — do not silently add `require_equals` (that breaks 5.1 leftover `--term`). |
| PATH 5.1 manual uses pre-T324 binary | AC15 allows `cargo run`; do not `cargo install`. |
| Conclusion PATH-behind | Hermetic + `cargo run` SoT (F27). |
| `or_else`/`unwrap_or_default` mistaken for panic unwrap | F16 — default is `String::new()`. |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| T311 R7 / I3 PowerShell empty TERM | **Absorb** F1–F11 / AC1–AC9 / AC15–AC16 |
| T323 same required positional | **Absorb** F1 / AC9 (copy-not-share) |
| Placeholder docs-only vs `--term` | **Pick `--term`** F1 / F30 — live `'""'`/`--%` are not empty |
| pwsh 7.6 Windows already preserves `""` | **Honesty** §2.1 — DoD remains 5.1 |
| T322 `--as-of` empty | **Decline** F6 |
| T311 R1 daemon `ListInForce` | **Decline** F13 |
| T314 `--dry-run` unify | **Not stolen** (Completed) |
| T323 `--as-of` / confirm-correct CLI / PATH install | **Not stolen** (T323 residuals) |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T326 `PinnedCountFailed` fake `pinned=0` (`#237`) | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 / T240 F2 | **Not stolen** / **Decline** |
| last-PR Cursor `#245` | **N/A empty** (no defect) |
| last-PR `#237` / `#230` | **T326** / **T325** already Pending — **no T327** |
| T323 uncommitted conductor Completed note | **Plan-write DOCS commit** |
| `ISSUES.md` | **Does not exist** |
| All other required positionals | **Decline** F26 |

---

## 10. Implement order (on go)

1. Phase 0: re-dogfood 5.1 drop; re-read clap InForce + dispatch; clap lock 4.6.1; T325/T326 still Pending.
2. Red: AC1–AC3 + conclusion mirrors (must fail).
3. Green: clap F1–F2 + dispatch merge + after_help F7; AC4–AC12 stay-green/update.
4. Docs AC14.
5. Targeted clippy/nextest; manual AC15–AC18; implement-track publish (never `git push origin main`).

---

## 11. Soft residuals

- Other required positionals still drop empty argv on 5.1 (`recall ""`, …).
- `--as-of ""` on 5.1 still clap “value required”.
- PATH until owner `cargo install` (T312–T324).
- Long-term: pwsh 7.3+ Windows mode already preserves empty; 5.1 remains the Windows-first hole until Microsoft’s default desktop shell is 7+.
- Help `[TERM]` vs old docs `<TERM>` — update only the in-force rows (F33).

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/main.rs` | `DecisionCommands::InForce` + `ConclusionCommands::InForce` clap; dispatch merge; after_help. **No** new test modules here (F22). |
| `crates/ai-brains-cli/src/commands/decision.rs` | **No** `run_in_force` logic change (merged String). Optional doc comment. |
| `crates/ai-brains-cli/src/commands/conclusion.rs` | Same. |
| `crates/ai-brains-cli/tests/decision_in_force.rs` | AC1–AC8 / AC7 |
| `crates/ai-brains-cli/tests/conclusion_in_force.rs` | AC9 |
| `Docs/CAPABILITIES.md` | Family C `--term` |
| `Docs/OPERATIONS.md` | `--term=` example |
| `Docs/CLI-EXIT-CODES.md` | in-force omit = fail_usage |
| `CHANGELOG.md` | Unreleased |
| `conductor/conductor.md` | Planned → (on go) Completed |
| `conductor/deferred.md` | this plan section |

**Do not touch:** `governed_common.rs`, `help_ia.rs`, CP `in_force.rs`, `conclusion_in_force.rs`, store projectors, `project.rs`, `sync.rs`, `forget.rs` production, retrieval, daemon, contracts.

---

## 13. last-PR Cursor / fold-in hooks

| Item | Disposition |
|------|-------------|
| `#245` comments/reviews | **N/A empty** |
| `#237` Bugbot pin-count | **T326** Pending |
| `#230` Bugbot F8 recency | **T325** Pending |
| New leftover fitting no placeholder | **None — no T327** |
