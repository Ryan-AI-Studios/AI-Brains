# T292 — `policy check` needs a human allowed/denied line

- **Track ID:** T292-PolicyCheckHuman
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / FEATURE
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `policy check --capability ReadEvidence` **7/8** JSON-only. Placeholder minted with T285–T300 (`76c4db9`). T266 ✅ Family D froze governed silent-JSON (`OutputFormat::parse`) for policy/list/show — **this track lifts `policy check` only** onto Family A `auto` (TTY human / pipe JSON). T241 ✅ catalog + T226 ✅ soft-resolve stay. T227 F34 / T266 F11 `OutputFormat::parse` **unchanged**.
- **Depends on:** T160 ✅ `run_check` + `CheckResult`; T201 ✅ deny `details.hint`; T210 ✅ bootstrap set; T221 ✅ exit 3 + `emit_error`; T226 ✅ soft-resolve; T241 ✅ catalog / SHORT SOOT; T249/T266 ✅ `resolve_human_json_format` + clap `value_parser`; T257 ✅ `emit_json` pretty; T180 ✅ P-CLI checklist
- **Blocks / feeds:** Operators who run `policy check --capability ReadEvidence` on a TTY see `allowed: true (ReadEvidence on …)` instead of a JSON object. Denied caps print `denied:` + bootstrap next on stdout (exit **3**). Neighbors **T293**. Forget-list **T299**. Device **T298**.
- **Absorbs:** Placeholder problem text + Manual DoD two commands; deferred.md “`policy check` JSON-only”; T266 F1 Family D **for check only** (show/bootstrap/lists stay D)
- **Not absorbed (DoD):** `policy show` / `policy bootstrap` default JSON; `OutputFormat::parse` surface-wide `parse_or_fail` (T227 F34 / T266 F11); new capabilities; auto-grant; T263 H2; T293–T300; T240 F2; clap 5 / rusqlite 0.40; CheckResult new keys; daemon PolicyCheck RPC
- **Research date:** 2026-08-23 (plan dogfood HEAD `ea5c947` T291 `#207`; plan commit `1331786`. Product `src/` = Family D default json + human allow already exists + human deny empty stdout. PATH **0.1.2** 2026-08-22 19:41 **without** T285–T291 — hole is in **source and PATH**)
- **AI fold-in:** 2026-08-23 `agy-review.md` (`1331786`) + `opencode-review.md` (`1331786`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** OpenCode m1 AC2 `hermetic_bin` omit-principal (not `policy_bootstrap.rs` helpers); OpenCode m2 only clap constructs `CheckOptions`; OpenCode O1 OPERATIONS script `--format json` sentence; OpenCode O3 T241 F6b catalog after_help freeze; Agy O2 `--format Pretty` clap InvalidValue. **Already:** Agy m1 F25; Agy m2 F7; Agy O1 F12/AC9; OpenCode O2 F1 stdout TTY. **Decline:** none of B/M (none filed). Word/pin/hotspot snapshot only. Disposition **§13**.
- **Ledger:** planning DOCS TX `84c4b2ec-3930-4d49-bcee-6b0bb3abdce3`. Fold-in DOCS TX `2eafd304-3287-44d2-9e87-51ce0ed42523`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** pin production decisions to the live vault as implement. Do **not** rewrite `.env`. Do **not** extra live `policy bootstrap`. Do **not** change `OutputFormat::parse`. Do **not** grow hotspot `project.rs` / `sync.rs` / `forget.rs` / CLI `preflight.rs` / `personal.rs` / `briefing.rs` / `governed_common.rs` (hotspot **#3** at **3.604** fold-in — reuse consts only). Human + format resolve live in `policy_cmd.rs` (not top-10) + clap in `main.rs`. Reuse `format_resolve::resolve_human_json_format` (do **not** change its map). Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **TTY `policy check` is scannable.** Default `--format auto` is TTY human / pipe JSON. Allow is one stdout line containing `allowed:` and the capability (not a JSON object). Scripts pass `--format json` and keep frozen `CheckResult` keys.
2. **Denied human is a remediator, not empty stdout.** `--format human` (and `auto` on TTY) on deny prints `denied: {cap}` plus T241 SHORT bootstrap next. Exit **3** stays. JSON deny stays one `ApiError` document on stdout (T160 R1-01).
3. **Do not blanket-flip Family D.** `policy show` and `policy bootstrap` stay default JSON. `OutputFormat::parse` `_ => Json` stays for every other governed command. This is a **named** Family A lift for `policy check` only.
4. **North star.** Capture independence: CLI presentation on an existing policy probe. No new events. No hidden CoT. No new grants. Deny-by-default until the operator bootstraps (T210 F13).

This unblocks the unused human path: `run_check` already emits `allowed: true ({cap} on {scope})` when `--format human` is passed, but clap default is `json` and `--format auto` is silent-JSON via `OutputFormat::parse`. The 2026-08-22 audit still scores **7/8** because TTY dumps JSON and deny human leaves stdout empty.

---

## 2. Live baseline (re-scan 2026-08-23)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `ea5c947` T291 `#207` on `main`. Tree **CLEAN**. `origin/main` = `ea5c947`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. Has T274. **Does not have T285–T291.** Policy-check hole is in **source + PATH** (T160-era default json). **Do not `cargo install`.** |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4041** (volatile; plan 4020 / OpenCode 4020). In-context **0/0/0**. Word **664** (plan 267 / OpenCode 408). Grants omitted (live **3 of 3** — ReadEvidence allows). Phase 0 re-verify. |
| `policy check --capability ReadEvidence` (PATH, no `--format`) | pretty JSON `{allowed:true, principal_id, capability, scope}` on stdout. Exit **0**. **This is the 7/8 hole.** |
| `policy check --capability ReadEvidence --format human` | stdout `allowed: true (ReadEvidence on Repository:3581317d-…)`. Exit **0**. **Human allow already exists.** Do not restyle except freeze the line. |
| `policy check --capability ProposeConclusion --format human` | Exit **3**. stderr `POLICY_DENIED: ProposeConclusion denied for principal …` + `POLICY_DENIED_HINT`. **stdout empty.** **This is the deny-human hole.** |
| `policy check --capability ReadEvidence --format auto` | PATH **and** source `cargo run` → **same JSON object** (parse maps `auto` → Json). `auto` is **not** a real token today. |
| `policy check --help` | `--format <FORMAT> [default: json]`. Catalog after_help intact (T241 F6b). |
| Last GitHub PR | [#207](https://github.com/Ryan-AI-Studios/AI-Brains/pull/207) T291 (2026-08-23). **Cursor / Bugbot / reviews / issue comments: empty.** **N/A — no T301.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, `#58` tower-http, actions `#68–#72`). |
| Identity / doctor | ledgerful doctor **4** warn (legacy `.changeguard` / sig-pin / sig-version / timings). Optional :8081 unreachable; :8083 ok at fold-in (plan: both unreachable — volatile). **0 pending / 0 drift.** Hotspot **#1** `project.rs` (**3.932**; plan 3.941) — **do not touch.** `sync.rs` **#2** (3.619). `governed_common.rs` **#3** (**3.604**) — **reuse consts only.** `policy_cmd.rs` **not** top-10 — format + human deny live here. CLI `preflight.rs` #8 (2.151). |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why JSON-only still trains “scripts-only”

| Layer | Truth |
|-------|--------|
| Human allow already shipped | `run_check` Human/Markdown arm `:222–225` prints `allowed: true ({cap} on {scope})`. Undiscoverable because default is json. |
| Default is Family D | clap `default_value = "json"` on `Option<String>` (`main.rs:2340`). T266 F1 listed policy under **D governed**. |
| `auto` is silent-JSON | `OutputFormat::parse` (`governed_common.rs:360`) lowercases and maps unknown/`auto` → Json. Passing `--format auto` today is JSON on TTY **and** pipe. |
| Deny human empty stdout | `!allowed` always `fail_api` → `emit_error`. Json → ApiError stdout. Human → stderr CODE + hint, **stdout empty**. Operators cannot scan allow vs deny on one stream. |
| T160 R1-01 | Deny `--format json` is **exactly one** JSON document on stdout. **Keep.** |
| T241 catalog | Omit `--capability` → `fail_usage` exit **2** catalog. Format-independent. **Keep.** |
| T226 soft-resolve | Omit `--scope` when authoritative. **Keep.** |
| T210 bootstrap | Issues Read* only. ProposeConclusion deny after bootstrap is honest. Human next still names SHORT (residual §11). |
| CheckResult is CLI-local | Not a contracts DTO. Keys `allowed`, `principal_id`, `capability`, `scope`. Pretty `emit_json`. |
| PROTOCOL-COMPAT §5 | **No row** for `policy check` today. Add a Family A row (T180 honesty for the default lift). |
| CAPABILITIES table | `policy check` is lumped under “list/show (governed…)” Family D. Named exception row is DoD. |
| T266 F11 / T227 F34 | Do **not** change `OutputFormat::parse`. Check resolves `auto` **before** parse (scope.rs analog). |
| T274–T284 decline | Series called U=7 “JSON-only, not a defect.” T285–T300 reopened U&lt;8. |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| CLI check | `policy_cmd.rs` `run_check` `:141–227` | `OutputFormat::parse(options.format.as_deref())` `:145`. Deny `fail_api` `:189–202`. Allow `CheckResult` + `emit_json` / `emit_human` `:205–226`. **Replace format resolve + deny Human arm.** |
| Options | `CheckOptions { capability, scope, format: Option<String>, principal_id }` `:30–37` | `format` becomes `String` (scope-resolve analog). |
| clap | `main.rs` `PolicyCommands::Check` `:2333–2343` | `default_value = "json"`; **no** `value_parser`. after_help catalog `:2330`. Dispatch `:4326–4338`. Add T266 `value_parser`; default **`auto`**. |
| Policy after_help | `main.rs` Commands::Policy **`:1719`** + enum **`:2311`** + Check **`:2330`** | Examples have no `--format human`. AC10 extends Check (and parent if it names format). |
| Format enum | `OutputFormat::parse` `:360–365` | Case-insensitive; unknown/`auto` → **Json**. **Cannot** implement AC7. Clap `value_parser` rejects `JSON`/`Pretty`. After clap accepts a token, `resolve_human_json_format` then map human→Human, json→Json. |
| Shared resolver | `format_resolve.rs` `resolve_human_json_format` `:8–16` | `pretty\|human\|text\|markdown\|md` → `"human"`; `json` → `"json"`; `auto` + TTY → human; `auto` + pipe → json. **Reuse. Do not change the map.** Precedent `scope.rs:29–38`. |
| Deny hint | `POLICY_DENIED_HINT` `:51` / `policy_denied_hint_details()` | JSON deny **frozen**. Human deny next is **SHORT**, not this 172-char HINT. |
| Short SOOT | `POLICY_BOOTSTRAP_SOOT_SHORT` `:160–161` | T241 F14 freeze. Human deny line 2 **exact**. |
| Catalog | `CAPABILITY_CATALOG` / `capability_required_usage_message` | T241 F6/F6b. **Do not change.** |
| Hermetic deny JSON | `exit_contract.rs` `policy_check__deny__exit_3_details_hint` `:180` | Forces `--format json`. **Stay green.** |
| Hermetic no-grant | `governed_surface.rs` `cli_policy_check__no_grant__exit_code_3` `:354` | Forces `--format json`. **Stay green.** |
| Bootstrap suite | `policy_bootstrap.rs` | Almost every check passes `--format json`. Default auto + non-TTY still JSON — **stay green without edits** if hermetic is a pipe. |
| Soft-resolve | `policy_soft_resolve.rs` | Forces `--format json`. **Stay green.** |
| Missing cap | `policy_check__missing_capability__fail_usage_catalog_exit_2` | Format-independent. **Stay green.** |
| Contracts / daemon | none | No `CheckResult` DTO. No PolicyCheck RPC. |

### 2.4 Dependency / standards research (2026-08-23)

| Pin / source | Workspace / live | Action |
|--------------|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** / GitHub **v4.6.6** (2026-08-06) / docs.rs 4.6.6 `PossibleValuesParser` is **case-sensitive** unless `ignore_case` (not set — T249 AC16 / T266 / T291 already rely on this) | **No bump.** clap **5** not current. Snapshot — re-verify at execute. |
| `serde_json` | lock **1.0.150** / crates.io **1.0.151** | **No bump.** Existing `emit_json` pretty. |
| `chrono` | lock **0.4.44** / crates.io **0.4.45** (`#62`) | **No bump.** |
| `rusqlite` | lock **0.39.0** / crates.io **0.40.2** (`#61`) | **No bump.** |
| `thiserror` | lock **2.0.18** / crates.io **2.0.20** (`#60`) | **No bump.** |
| `tokio` | lock **1.52.3** (`#59`) | **No bump.** |
| rustc / edition / nextest | **1.95.0** / **2024** / **0.9.140** | Unchanged. |
| workspace version | **0.1.2** | **No bump.** |
| New crates | — | **Zero.** `std::io::IsTerminal` already used (T266). Do not re-add `is-terminal`. |
| [clig.dev Output](https://clig.dev/#output) (fetched 2026-08-23) | Humans first; TTY heuristic; stdout for primary output; `--json` for structure; suggest next command; changing human output is usually OK; machine JSON is an interface | Default `auto` TTY human. Scripts pin `--format json`. Human deny next names bootstrap. |
| [clig.dev Future-proofing](https://clig.dev/#future-proofing) | Keep changes additive; human output may evolve | Human lines are **not** a wire contract. JSON keys frozen. |
| T180 P-CLI §8 | Prefer additive keys; type change → documented lift + tests; compact↔pretty switch is breaking unless flagged | Default json→auto is a **documented CLI default lift** (TTY only; pipes stay JSON). CheckResult keys **unchanged**. Pretty `emit_json` unchanged. Add §5 row. |
| T266 F1 / F11 | Four families; do not change `OutputFormat::parse` | **Affirm parse freeze.** Lift **check only** D→A. |
| T227 F34 | silent-JSON-on-unknown residual for non-briefing governed | **Affirm.** Not `parse_or_fail` this track. |
| T241 F14 SHORT | `next: run \`ai-brains policy bootstrap --dry-run\` then \`ai-brains policy bootstrap\`` | Human deny line 2 **exact**. Do not rewrite. |
| SQLCipher / schtasks | N/A — presentation only | N/A (written). |

Training data is not a pin. Re-verify clap/serde_json at execute.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Family A lift (`policy check` only)** | clap `default_value = "auto"` + T266 `value_parser`. TTY human / pipe JSON via `resolve_human_json_format` + **`std::io::stdout().is_terminal()`** (scope.rs analog; T291 `governed_query.rs:245`). Do **not** probe `stdin()` or `stderr()`. `--format json` unchanged object. `--format human`/`pretty`/`text`/`markdown`/`md` force human **regardless of TTY**. `policy show` / `policy bootstrap` stay Family D default json. |
| **F2 — Human allow freeze** | One stdout line **exact shape** `allowed: true ({cap} on {scope})` (already shipped `:223`). Contains `allowed:` and the capability. No leading `{`. No second next-line on allow (grant already exists). Helper `format_policy_check_allow_line` in `policy_cmd.rs`. |
| **F3 — clap `value_parser` (not `OutputFormat::parse`)** | Tokens `["auto", "pretty", "human", "text", "json", "markdown", "md"]`. **Case-sensitive.** `JSON`/`Pretty`/`xml` → clap InvalidValue exit **2**. Do **not** route **AC6** through `OutputFormat::parse` (lowercases; `auto`/`JSON` → Json). After clap accepts a token: resolver → `"human"` or `"json"` → `OutputFormat::Human` or `Json`. Precedent `main.rs:2446` / `:2459` (retention) and Trace `:1997`. |
| **F4 — Exit codes frozen** | Allow **0**. Deny **3**. Missing `--capability` / soft-resolve fail **2**. Unknown capability **6**. Do not use exit 4. |
| **F5 — JSON allow keys frozen** | `CheckResult` `{allowed: bool, principal_id, capability, scope}` pretty `emit_json`. **No** new required keys (`next_step` / `found` / `denied` are not on the allow object). CLI-local — not contracts. |
| **F6 — JSON deny frozen** | `fail_api` ApiError `POLICY_DENIED` + `details.hint` = `POLICY_DENIED_HINT` on **stdout**, exactly one JSON document (T160 R1-01). Exit 3. Existing hermetics stay green. |
| **F7 — Human deny (SOOT)** | Two stdout lines. Line 1 **exact** `denied: {cap}` (`format_policy_check_deny_line`). Line 2 **exact** `POLICY_BOOTSTRAP_SOOT_SHORT` (T241 F14). Exit **3**. Not a JSON object. Do **not** call `fail_api`/`emit_error` on this arm (stderr stays empty so TTY scan is stdout-only). Return `GovernedCliError::emitted(EXIT_POLICY_DENIED, …)`. Markdown ≡ human. |
| **F8 — T241 / T226 / T210 freeze** | `--capability` still optional at clap; omit → catalog `fail_usage` (format-independent). Soft-resolve `--scope` unchanged. Bootstrap still Read* only. No auto-grant (T210 F13). **F6b catalog:** Check `after_help` “Valid capabilities (discovery first):” block stays **byte-stable** with `CAPABILITY_CATALOG` (`governed_common.rs:147–157` / clap comment `:2329`). Additive format examples go in the Examples block **only** — do not restyle or reorder the catalog (OpenCode O3). |
| **F9 — `OutputFormat::parse` frozen** | Do **not** add `Auto`. Do **not** `parse_or_fail`. Show/bootstrap/lists still silent-JSON-on-unknown. |
| **F10 — No contracts / daemon** | Do not mint a CheckResult DTO. Do not add PolicyCheck RPC. PROTOCOL-COMPAT is **P-CLI** stdout only. |
| **F11 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. Hermetic `tempfile::tempdir`. JSON deny/allow hermetics that pass `--format json` stay. New human ACs. Clap AC for `JSON` InvalidValue + default `auto`. |
| **F12 — Docs** | CAPABILITIES OutputFormat table: named `policy check` Family A row (list/show remain D). PROTOCOL-COMPAT §5 new row. OPERATIONS add `--format human` example **and** the exact operator sentence: `scripts that previously parsed TTY default JSON must pass --format json`. Check `after_help` Examples + clap `///` docstring (catalog freeze is F8). CHANGELOG on implement. CLI-EXIT-CODES: deny exit 3 unchanged; mention human stdout lines. |
| **F13 — PATH** | Soft. Source/hermetic SoT. Do not `cargo install` as implement. |
| **F14 — Capture independence** | No models, embeddings, graph, or new events. Probe + emit only. |
| **F15 — Isolation hotspots** | Do not grow `project.rs` / `sync.rs` / `forget.rs` / `preflight.rs` / `personal.rs` / `briefing.rs`. `governed_common.rs` (**#3** at **3.604**) — **import existing consts only** (SHORT, HINT, parse, fail_api, emit_*). Do not change `resolve_human_json_format` behavior. |
| **F16 — Identity stdout** | JSON allow/deny call `emit_json` / `print_json_stdout` (T257). Human path does **not** mark machine JSON. |
| **F17 — ISSUES.md** | Does not exist. Debt is `deferred.md`. |
| **F18 — Decline peers** | T293 neighbors; T294 leftover; T295 backup; T296 nightly Router; T297 daemon vs LLM; T298 device; T299 forget-list; T300 graph sparse. T291 Completed — not stolen. |
| **F19 — Standing declines** | T240 F2; T263 H2; 750 ms; clap 5; rusqlite 0.40; DTO new required keys; auto-grant; new capabilities. |
| **F20 — No T301** | #207 Cursor **N/A empty**. Dependabot remotes are not tracks. #206 sanitizer is T291 F16 Completed. |
| **F21 — `auto` token lift** | Today `--format auto` is silent-JSON. After, `auto` TTY-switches. Document in PROTOCOL-COMPAT. Scripts that ran check on a TTY without `--format` would have parsed JSON — they must pass `--format json` (clig). Hermetic/CI non-TTY omit-format **stays JSON**. |
| **F22 — Cross-model** | FEATURE (operator format contract). After Phase-1 review clean, run read-only `codex-review`. |
| **F23 — Stop-before** | Even after go: do not write `.env`; do not extra live `policy bootstrap`; do not `retention apply --confirm`; do not mutate Nightly/Router; do not `graph rebuild`. |
| **F24 — Propose* honesty** | Human deny next is SHORT for **all** caps (including Propose*/Approve*/Erase/Export). Bootstrap does not issue those. Residual §11 — do not invent grant-admin. |
| **F25 — Helper locality** | `format_policy_check_allow_line` / `format_policy_check_deny_line` are `pub(crate)` in `policy_cmd.rs`. **Not** `pub`. **Not** re-exported from `commands/mod.rs`. Do not move into `governed_common.rs` (Agy m1). |
| **F26 — Show/bootstrap clap** | Do **not** add `value_parser` or flip default on Show/Bootstrap this track. **AC8** locks their `--help` default still `json`. |
| **F27 — AC2 principal (OpenCode m1)** | AC2 uses `common::hermetic_bin()` (denylist includes `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` at `tests/common/mod.rs:55`). Both bootstrap and check **omit** `--principal-id` so both resolve `cli_principal()` System (`a1b2a1b2-…`). Do **not** reuse `policy_bootstrap.rs` `policy_bootstrap` / `policy_check` helpers — they force `--principal-id PRINCIPAL` and `--format json`. Same class as T291 AC5 not `progressive_cmd`. |
| **F28 — `CheckOptions` construction (OpenCode m2)** | Live constructors: struct `:30` + clap dispatch `main.rs:4333` **only**. After `format: String`, no test struct-literal with `None`. Phase 0 grep `CheckOptions {`. |
| **F29 — Catalog after_help (OpenCode O3)** | See F8. Hermetic `policy check --help` still contains every `CAPABILITY_CATALOG` line in discovery-first order. |

---

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | Unit: `format_policy_check_allow_line("ReadEvidence", "Repository:aaaa-bbbb")` **exact** `allowed: true (ReadEvidence on Repository:aaaa-bbbb)`; `!starts_with('{')`; contains `allowed:` and `ReadEvidence`. `format_policy_check_deny_line("ProposeConclusion")` **exact** `denied: ProposeConclusion`. Deny human line 2 unit equals `POLICY_BOOTSTRAP_SOOT_SHORT` (`assert_eq!`). **Required red.** |
| **AC2** | Hermetic via `common::hermetic_bin()` (F27): after System `policy bootstrap` (**omit** `--principal-id`) on a temp vault + `--scope Repository:<uuid>`, `policy check --capability ReadEvidence --format human` (**omit** `--principal-id`) exit **0**; stdout is **not** a JSON object (trim no leading `{`); contains `allowed:` and `ReadEvidence`; one line (no extra blank). Do **not** call `policy_bootstrap.rs` helpers. |
| **AC3** | Hermetic: ungranted vault `policy check --capability ProposeConclusion --scope Repository:<uuid> --format human` exit **3**; stdout is **not** a JSON object; contains `denied:` and `ProposeConclusion`; contains `policy bootstrap`; two lines; stdout is **not** parseable as `ApiError`. **Stderr** does **not** contain `POLICY_DENIED:` (F7 skip `fail_api`; `main.rs:3568` `!emitted` skips `eprintln!`). |
| **AC4** | Hermetic: same allow vault `policy check --capability ReadEvidence --format json` exit **0**; object keys `allowed==true`, `capability=="ReadEvidence"`, `principal_id` + `scope` present; **no** `next_step` / `found` keys. |
| **AC5** | Stay-green: `policy_check__deny__exit_3_details_hint` (json) still exit 3 + `code=POLICY_DENIED` + non-empty `/details/hint`; `cli_policy_check__no_grant__exit_code_3` still one JSON document; missing-capability catalog exit 2; unknown capability exit 6; T226 soft-resolve tests. |
| **AC6** | clap: `policy check --capability ReadEvidence --format JSON` **and** `--format Pretty` InvalidValue exit **2** (stderr clap, not `OutputFormat::parse`; Agy O2). `--format json` parses. `--format pretty` parses. Omitted `--format` clap default is **`auto`**. |
| **AC7** | Hermetic non-TTY omit `--format` on allow still JSON `allowed:true` (auto + pipe). Locks F21: CI/scripts without a TTY do not silently become human. |
| **AC8** | `policy show --help` and `policy bootstrap --help` still `default: json` (or clap default json). Do not flip Family D peers. |
| **AC9** | Docs + help: CAPABILITIES named `policy check` Family A row; PROTOCOL-COMPAT §5 row; OPERATIONS `--format human` example **plus** `scripts that previously parsed TTY default JSON must pass --format json` (OpenCode O1). Check after_help Examples mention default `auto` = TTY human / pipe JSON; **Valid capabilities** catalog block unchanged (F8/F29). Hermetic: `policy check --help` contains `auto` (or TTY) and does **not** claim JSON-only; still lists `ReadEvidence (discovery)` … `ProposeDecision` in catalog order. |
| **AC10** | Manual (on go, `cargo run -p ai-brains-cli --`, no `--daemon`): `policy check --capability ReadEvidence --format human` and `--format json`. Pass = AC2/AC4 shape on this vault (live 3 of 3 → allow). Optional: `--format human` on `ProposeConclusion` = AC3. Unique canary pin **not** required. **Do not** `cargo install`. **Do not** extra live bootstrap. |
| **AC11** | `CheckResult` serde: no new fields. Allow JSON golden keys still parse. |
| **AC12** | Markdown/`md`/`pretty`/`text` on allow ≡ F2 line (resolver). Not a separate markdown renderer. |

---

## 5. Design notes

### 5.1 Why not leave Family D and only document `--format human`

`--format human` already works for **allow**. The audit U=7 is default TTY JSON (clig: make the default the right thing for humans). Agents and operators omit `--format`. Family A `auto` is the T249/T266 inventory rule. Check is a dry-run probe, not a mutation — unlike `retention apply` / `graph update` (Family C).

### 5.2 Why not change `OutputFormat::parse`

T227 F34 and T266 F11 froze silent-JSON-on-unknown for the governed family. Changing parse would flip evidence/source/review/show/bootstrap/erasure defaults or reject `auto` globally. Check resolves tokens **before** parse, like `scope.rs`.

### 5.3 Why human deny skips `fail_api`

`fail_api` → `emit_error` Human prints `POLICY_DENIED: …` + HINT on **stderr** and leaves stdout empty. The product hole is “cannot scan allowed vs denied.” Allow is stdout; deny must be stdout too. JSON deny still uses `fail_api` (T160 one document). Other governed denies unchanged.

### 5.4 Why SHORT not recall / not HINT

T290 recall next is granted-empty **lists** (vault search). Policy deny is a **grant wall** — T241 F14 SHORT is the SOOT. `POLICY_DENIED_HINT` stays on the JSON envelope (`details.hint`) and is longer (T280 omit-scope wording). Human line 2 is the short operator next.

### 5.5 clap vs parse (AC6 / F3)

Same pin as T291 AC7 / T266 AC2: `value_parser` is case-sensitive; `OutputFormat::parse` cannot reject `JSON`. Copy `scope resolve` / `query trace` clap block.

### 5.6 AC2 bootstrap

System principal when `--principal-id` omitted (`cli_principal()` → `a1b2a1b2-…` unless `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` is set). `hermetic_bin` **strips** that env (`tests/common/mod.rs:55`), so omit-principal on both bootstrap and check is the T221 F31 / T291 AC5 pattern. Do **not** reuse `policy_bootstrap.rs` helpers (they pin `PRINCIPAL` + `--format json`). Do **not** extra live vault bootstrap. Live Manual AC10 uses existing 3 of 3.

### 5.7 Human deny `emitted` (Agy m2 / OpenCode looks-solid #2)

`GovernedCliError::emitted` sets `emitted: true` (`governed_common.rs:259`). `main.rs:3568` only `eprintln!`s when `!emitted`. Human deny returns `GovernedCliError::emitted(EXIT_POLICY_DENIED, …)` **after** the two stdout lines — stderr stays empty. JSON deny still `fail_api` → `emit_error` stdout ApiError.

### 5.8 Default lift vs T180

Pipes/hermetic stay JSON (`auto` + non-TTY). TTY omit-format becomes human. Document as P-CLI default lift, not a DTO type change. CheckResult keys unchanged. Compact↔pretty: still pretty `emit_json`.

---

## 6. Non-goals

- Flip `policy show` / `policy bootstrap` / evidence/source/review list defaults
- Change `OutputFormat::parse` or add `parse_or_fail`
- New capabilities, auto-grant, grant revoke UI, daemon IssueGrant
- Pin→Approved (T263 H2)
- Human renderer for `query progressive` (T290 F10 freeze)
- T293 neighbors / T294 leftover upsert / T299 forget-list
- clap 5 / rusqlite 0.40 / lock bumps / new crates
- `cargo install` / live `.env` / extra live `policy bootstrap`
- Color / pager / ValueEnum unify

---

## 7. Verification plan (TDD)

**Red first (required):**

1. `format_policy_check_allow_line__read_evidence__exact_string` (AC1)
2. `format_policy_check_deny_line__propose__exact_string` (AC1)
3. `policy_check__allow__format_human__allowed_line_not_json` (AC2) — fail while default/human-deny unimplemented or allow already passes once helper is used
4. `policy_check__deny__format_human__denied_plus_short_exit_3` (AC3) — fail while stdout empty

Then clap AC6 (`policy_check__format_JSON__clap_invalid_value` in `main.rs` tests, T266/T291 pattern).

**Green:** resolver in `run_check`; deny Human arm; clap default auto + value_parser.

**Stay-green:** AC5 json deny / missing cap / unknown cap / soft-resolve / bootstrap suite.

**Docs:** AC9 after behavior is green.

**Manual:** AC10 `cargo run` only.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| TTY scripts that parsed default JSON | F21 + PROTOCOL-COMPAT + help: pin `--format json`. Pipe/CI stay JSON (AC7). |
| `OutputFormat::parse` accidental edit | F9 / F15; resolver lives in `policy_cmd.rs` |
| JSON deny two documents | F6; deny Human skips `fail_api`; json still one document AC5 |
| Family D peer flip | F26 / AC8 help default json on show/bootstrap |
| Hotspot `governed_common.rs` | F15 import-only |
| Propose* next names bootstrap | F24 residual — do not invent admin |
| PATH-behind | F13 `cargo run` / hermetic SoT |
| last-PR leftover missed | #207 empty (verified). Dependabot not tracks. |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit `policy check` JSON-only U=7 | **Absorb** F1–F7 / AC1–AC4 / AC10 |
| Placeholder Manual `--format human` + `--format json` | **Absorb** AC10 |
| Placeholder `--format auto` TTY human / pipe JSON | **Absorb** F1 / F21 / AC6 / AC7 |
| Placeholder deny `denied: <cap> — next` | **Absorb** F7 two lines (SHORT exact, not one-line em-dash) |
| T266 F1 Family D for policy | **Partial lift** — check only → A; show/bootstrap **affirm D** F26 |
| T266 F11 / T227 F34 `OutputFormat::parse` | **Affirm freeze** F9 |
| T241 F6 catalog / F14 SHORT | **Affirm** F8 / F7 |
| T226 soft-resolve | **Affirm** F8 / AC5 |
| T160 R1-01 one JSON deny document | **Affirm** F6 / AC5 |
| T210 F13 no auto-grant | **Affirm** F8 / F19 |
| T291 query-trace envelope | **Decline** — Completed `#207` |
| T293 neighbors dump sessions | **Decline → T293** |
| T294 leftover dest-missing | **Decline → T294** |
| T298 device/replicate empty | **Decline → T298** |
| T299 forget-list empty | **Decline → T299** |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F19 |
| last-PR Cursor **#207** | **N/A empty** — **no T301** F20 |
| Identity leftover `7d97a456` | **Not this track** — T258 / T294 |
| Open T293–T300 | **Not related** except named declines |
| Closed T274–T291 | **Stay closed** |
| Dependabot `#58–#72` | **Not this track** |
| T142 #4 archive specs / connector cursor / … | **Not related** (no policy-check overlap) |

---

## 10. Implement order (on go)

1. Phase 0 re-verify HEAD / deferred / #207 still empty / live default still JSON / human deny still empty stdout
2. FEATURE TX
3. Red AC1 helpers + AC3 deny-human (stdout empty today)
4. Green: clap `--format` default auto + `value_parser`; `run_check` resolver; deny Human arm; allow helper
5. Red/green AC2 allow-human; AC4 json keys; AC6 clap `JSON`; AC7 pipe default json
6. Stay-green AC5 / AC8 / AC11
7. Docs AC9
8. Clippy + nextest + deny/audit
9. Manual AC10
10. Phase-1 review → codex-review
11. Publish: push `track/T292-*` → PR → watch GHA `CI` green → squash-merge → prune

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| PATH until `cargo install` | F13 |
| Propose*/Approve*/Erase/Export human deny next still SHORT | F24 — bootstrap does not issue those caps |
| `policy show` TTY still JSON by default | F26 / Family D |
| `policy bootstrap` TTY still JSON by default | F26 |
| Human deny stderr empty (no `POLICY_DENIED:` CODE) | F7 by design; JSON deny still stderr-free / stdout ApiError |
| `OutputFormat::parse` silent-JSON for other governed | T227 F34 / T266 F11 |
| T293–T300 | Not stolen |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/policy_cmd.rs` | Resolver + helpers + deny Human arm; `CheckOptions.format: String`; units AC1 |
| `crates/ai-brains-cli/src/main.rs` | Check `--format` default `auto` + T266 `value_parser`; after_help; dispatch; clap ACs (JSON InvalidValue, default auto, pretty parses) |
| `crates/ai-brains-cli/src/commands/format_resolve.rs` | **Reuse only** — do not change map |
| `crates/ai-brains-cli/tests/` (new or existing policy_*.rs) | AC2/AC3/AC4/AC7 hermetics |
| `Docs/CAPABILITIES.md` | `policy check` Family A row |
| `Docs/OPERATIONS.md` | `--format human` example + default auto + exact `scripts that previously parsed TTY default JSON must pass --format json` |
| `Docs/CLI-EXIT-CODES.md` | human deny stdout note; exit 3 unchanged |
| `Docs/PROTOCOL-COMPAT.md` | §5 `policy check` row |
| `CHANGELOG.md` | on implement |
| `conductor/conductor.md` | Planned now; Completed on implement only |
| `conductor/deferred.md` | this planning table (now); closeout later |

**Do not touch:** `governed_common.rs` `OutputFormat::parse` body; `policy show` / `bootstrap` clap defaults; CP policy evaluator; contracts; `briefing.rs`; `personal.rs`; CLI `preflight.rs`; `project.rs`; `.github/workflows/ci.yml`.

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` (HEAD `1331786`) + `opencode-review.md` (HEAD `1331786`). Fold-in HEAD `1331786` on `main`. Live verify: `run_check` `:141` deny `fail_api` `:192` / allow human `:223`; clap Check `default_value = "json"` no `value_parser` `:2340`; `OutputFormat::parse` `:360` unknown/`auto` → Json; `resolve_human_json_format` `:8–16`; TTY gate precedent `governed_query.rs:245` stdout; `GovernedCliError::emitted` `:259` + `main.rs:3568` `!emitted`; `CheckOptions {` only struct `:30` + dispatch `:4333`; `hermetic_bin` strips `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` `:55`; `policy_bootstrap.rs` helpers force `--principal-id PRINCIPAL` + `--format json`; T241 F6b comment `:2329` + `CAPABILITY_CATALOG` `:147`; OPERATIONS `:296–297` json-only example; CAPABILITIES Family D lump `:95`; PROTOCOL-COMPAT §5 no `policy check` row. Hotspot `governed_common.rs` **#3** (3.604). Pins **snapshot — re-verify at execute** (clap lock 4.6.1 / crates.io 4.6.6; rusqlite 0.39.0; no clap 5). Last merged PR still **#207**. **No T301.**

### Pins locked by fold-in

1. **AC2 / F27 (OpenCode m1):** `hermetic_bin` omit `--principal-id` both sides; do **not** reuse `policy_bootstrap.rs` helpers.
2. **F28 (OpenCode m2):** `CheckOptions` only clap dispatch `:4333`.
3. **F8 / F29 / AC9 (OpenCode O3):** Check after_help catalog block byte-stable with `CAPABILITY_CATALOG`.
4. **F12 / AC9 (OpenCode O1):** OPERATIONS exact sentence `scripts that previously parsed TTY default JSON must pass --format json`.
5. **AC6 (Agy O2):** clap `--format Pretty` InvalidValue as well as `JSON`.
6. **AC3 (Agy m2):** stderr has no `POLICY_DENIED:` (skip `fail_api`).
7. **F3 AC-id slip:** InvalidValue is **AC6**, not AC7. **F26** peer lock is **AC8**, not AC12.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** helpers `pub(crate)` in `policy_cmd.rs` | **Already** F25; **tightened** not `pub` / not `mod.rs` re-export |
| Agy | **m2** human deny skip `fail_api` | **Already** F7; **tightened** AC3 stderr empty + §5.7 `emitted` |
| Agy | **O1** CAPABILITIES Family A row | **Already** F12 / AC9 |
| Agy | **O2** clap `JSON`/`Pretty` InvalidValue | **Already** AC6 JSON / F3 Pretty; **folded** AC6 Pretty case |
| OpenCode | B / M | None filed |
| OpenCode | **m1** AC2 principal env | **Folded** F27 / AC2 / §5.6 — denylist already strips; forbid T210 helpers |
| OpenCode | **m2** `CheckOptions.format: String` | **Already** one clap constructor; **folded** F28 |
| OpenCode | **O1** OPERATIONS script `--format json` sentence | **Folded** F12 / AC9 exact wording |
| OpenCode | **O2** `stdout().is_terminal()` not stdin/stderr | **Already** F1; **tightened** |
| OpenCode | **O3** T241 F6b catalog after_help | **Folded** F8 / F29 / AC9 |
| OpenCode | word 267→408 / pinned 4020 | **Snapshot only** (fold-in word **664** / pinned **4041**); Phase 0; not DoD |
| both | last-PR #207 Cursor | **Affirm F20** — no T301 |
| both | deferred T293–T300 / H2 / clap 5 | **Affirm** |

No Blockers. No Majors. No new placeholder minted. Do **not** edit `*-review.md`.
