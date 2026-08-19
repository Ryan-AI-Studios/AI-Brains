# T266 — Format policy convergence

- **Track ID:** T266-FormatPolicyConvergence
- **Status:** **Planned** (Pending in registry; plan-only until go)
- **Category:** UX / FEATURE
- **Owner:** —
- **Source:** Audit 2026-08-16 — friction “format policy is a maze”; `project list-paths` **7/5**; `retention plan` default **6/5**
- **Depends on:** T248 retention TTY human ✅; T249 scope TTY human ✅; T254 list-paths/scan-roots `auto` ✅; T246 graph human ✅; T255 nightly pipes-stay-human ✅; `format_resolve::resolve_human_json_format` (closed 2026-08-16)
- **Blocks / feeds:** Operators can predict `--format` on inventory. Preflight envelope stays **T265**. List footer leftover-as-AI-Brains stays **T267**. scan-roots parent/`--root` stays **T268**. Retention classify stays **T270**.
- **Absorbs:** Audit T266 row (format maze; list-paths JSON wall; retention default JSON on this agent’s non-TTY); T227 F34 *pointer only* (do not flip governed `OutputFormat::parse`); T249/T255 shared-resolver closeout; incomplete CAPABILITIES OutputFormat table; missing PROTOCOL-COMPAT list-paths/scan-roots rows; three forked `use_json_output` helpers
- **Not absorbed:** T265 `sections[]` / T180 2-key; T246 F6 `graph update` default JSON (T74); T255 F2 nightly pipes-stay-human; T248 apply default JSON; T227 F34 silent-JSON surface-wide; T240 F2; T255 declines; clap 5 / new crates; T267 footer; T268 scan behavior; T270 classify; T272 safety_ids Index skip
- **Research date:** 2026-08-18 (HEAD `4088106` T264 `#179`)
- **AI fold-in:** —
- **Ledger:** planning DOCS TX `201a3883-3053-4487-ba46-8942565eeae5`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** write live `.env`. Do **not** flip `graph update` / nightly / recall / retention apply / governed list defaults. Do **not** grow T180 `{text, word_count}`. Do **not** reopen T240 F2 / T255 declines. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

Make `--format` **predictable** on operator inventory without breaking frozen script contracts.

1. **Document one taxonomy.** Four families, written in CAPABILITIES: **auto** (TTY human / pipe JSON), **always-human** (pipes stay human), **always-JSON** (opt-in human), **governed silent-JSON** (`OutputFormat::parse`).
2. **Converge Family A tokens.** `list-paths`, `scan-roots`, `whoami`, `adopt-path`, `rebind-path` use the same clap token set and `resolve_human_json_format` as T248/T249. `--format pretty|human|text|markdown|md` is a table even on a pipe. `--format json` is the frozen object.
3. **Keep the auto default.** The 2026-08-16 “JSON wall” on this agent is `auto` + non-TTY — that is the rule, not a bug. Agents force a table with `--format human` (already works). Help must say so.
4. **Do not silently flip frozen surfaces.** Nightly pipes stay human. `graph update` stays pretty JSON. `retention apply` stays JSON. `recall` stays pretty/json. Governed list/show stay `OutputFormat::parse`.

That advances the north star: capture stays grant-independent; the append-only log stays SoT; agents and operators can pick human vs JSON on purpose instead of guessing the next command.

No models. No new crates. No clap 5. No T180 key growth.

---

## 2. Live baseline (2026-08-18)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `4088106` — T264 `#179` squash on `main`. Tree **CLEAN**. In sync with `origin/main`. T266 still Placeholder until this plan. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **0.1.1**. Format flags for list-paths exist (T254). **Do not `cargo install`.** |
| This session | Agent **non-TTY**. Same class as the 2026-08-16 audit. |
| `project list-paths` (source, no `--format`) | **124-line pretty JSON** (`api_version=1`, 53-ish roots). Audit “126-line wall” is live. |
| `project list-paths --format human` | **18-line table** (`path` / `project` / `project_id` / `exists`). Leftover `7d97a456` rows labeled `(no alias)` — footer/alias honesty is **T267**. |
| `project scan-roots` | Pretty JSON. `scan_root=C:\dev\AI-Brains`, this repo already registered. |
| `project scan-roots --format human` | 2-line table. `suggested` still names `register-path` for an already-registered root — scan **behavior** is **T268**. |
| `retention plan` | Pretty JSON. `api_version=1`, `classes: []` / `candidates=0` (T270). Auto + pipe = JSON. **T248 already shipped.** |
| `scope resolve` | Pretty JSON. Auto + pipe. **T249 already shipped.** |
| `project whoami` | Pretty JSON (`effective`/`path` = `3581317d`; `shell_project_id` leftover `7d97a456`). Auto + pipe. Identity remediator is **T258/T267**, not format. |
| `nightly --status --quick` | **Human** `=== Nightly Status ===`. Last result **0**. Last nightly `2026-08-18T07:08:48Z`. Pipes stay human (**T255 F2**). |
| `doctor --summary` | Human (`status=degraded`, 15-check). Always-human. |
| `memory list --limit 1` | Human table. Always-human. |
| `project list` | Human table. Leftover first (18032) — **T267**. |
| PATH `graph update` | Pretty JSON T213 keys (`nodes=21844`, `status=sparse`). Default JSON. Source `cargo run` without `--features graph` → `FEATURE_UNAVAILABLE` (expected). |
| `preflight --summary` | Scope path owner `3581317d`. Pinned **3022**. Grants **0 of 3** (T241; not this track). |
| Last GitHub PR | [#179](https://github.com/Ryan-AI-Studios/AI-Brains/pull/179) T264. Cursor Bugbot **1 Medium**: `safety_ids` filled from Safety fetch LIMIT 40 *before* cap, so Index/Recent skip the 32 capped-out ids. **Still true** at `preflight.rs:329` + `:467`. Does **not** fit T266. **Minted T272.** No open PR on `main`. |
| Ledgerful | `doctor` ready (legacy `.changeguard` / sig-pin / timings / :8081 unreachable; :8083 ok). 0 pending 0 drift. Hotspot **#1** `project.rs` (3.865) — whoami format match only. `#5` `governed_common.rs` — **do not** change `OutputFormat::parse`. |
| ai-brains recall | T248/T249 review notes + T258 `use_json_output` mention. No prior “converge inventory onto `resolve_human_json_format`; do not flip graph update / nightly” pin. |

### 2.2 Why the maze is still a product hole

| Residual | Why it is still a hole / why decline |
|----------|--------------------------------------|
| Operators cannot predict the next command | Same non-TTY session: list-paths/whoami/retention/scope → JSON; nightly/doctor/memory/project list → human; graph update → JSON. CAPABILITIES table lists only a subset. **DoD = complete taxonomy.** |
| list-paths JSON wall (7/5) | `default_value = "auto"` + `use_json_output` already TTY-switches. Agent non-TTY **should** get JSON under the documented rule. `--format human` already prints the table. Quality 5 is **undiscoverable remediator** + **narrow token map** (`pretty` is clap `InvalidValue`). **DoD = tokens + help, not a default flip.** |
| retention default JSON on this agent (6/5) | T248 `auto` + pipe. Human matrix exists via `--format human`. **Do not restyle the matrix.** Document Family A. Classification of 0 candidates is **T270**. |
| Three `use_json_output` forks | `project_paths.rs`, `project_adopt.rs`, `project_rebind.rs`: `to_ascii_lowercase`, tokens `auto\|human\|json` only, unknown → `fail_usage`. Shared resolver is case-sensitive and maps `pretty/text/md`. Maintainers have two policies. **DoD = delete forks.** |
| whoami inline match | `project.rs:698` `_ => !is_terminal()` (unknown-as-auto). Clap already rejects unknown. `--format pretty` is rejected today. **DoD = one call to the shared resolver.** Do not grow the hotspot. |
| `graph update` pretty JSON | T246 **F6** / T74: default JSON; `auto` does **not** TTY-switch. F17 lists TTY-auto as soft. Flipping would break T74 + TTY JSON consumers. **Decline.** |
| T227 F34 silent-JSON | `OutputFormat::parse` `_ => Json` for evidence/source/review/policy/…. Intentional governed contract. Surface-wide `parse_or_fail` is a different track. **Decline.** |
| Nightly pipes | T255 F2 / AC10: default human including pipes. **Decline flip.** |
| T265 envelope | `preflight --format json` stays `{text, word_count}`. **Decline.** |
| Cursor #179 safety_ids | T264 Completed selection bug. **Mint T272.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Shared resolver | `commands/format_resolve.rs` `resolve_human_json_format` | `pretty\|human\|text\|markdown\|md` → `"human"`; `json` → `"json"`; `auto` + TTY → human; `auto` + pipe → json; `_` fail-closed json. Units already cover the map. |
| Callers today | `scope.rs`, `retention.rs`, `nightly_status.rs` | Thin wrappers. Copy this pattern. |
| list-paths / scan-roots | `project_paths.rs` `use_json_output` | Fork. Human table + JSON envelopes already exist. Keys frozen T254 F10. |
| adopt-path / rebind-path | `project_adopt.rs` / `project_rebind.rs` | Same fork. Print-only by default. |
| whoami | `project.rs:692` | Hotspot **#1**. Format match only. |
| clap inventory | `main.rs` ListPaths / ScanRoots / Whoami / AdoptPath / RebindPath | `default_value = "auto"`, `value_parser = ["auto", "human", "json"]`. |
| clap T248/T249 | retention plan / scope resolve / nightly | `["auto", "pretty", "human", "text", "json", "markdown", "md"]`. Case-sensitive (`JSON`/`Pretty` → exit **2**). |
| Graph query | `graph.rs` `resolve_graph_format` | Local. Resolves to `"pretty"` not `"human"`. **Leave.** |
| Graph update | `main.rs` Update `default_value = "json"` | `auto` = json. Human is opt-in. **Leave.** |
| Governed parser | `governed_common.rs` `OutputFormat::parse` | Lowercase + `_ => Json`. **Leave.** |
| Harness status | `harness.rs` `run_status` | Default `"human"`; `json` only if token is `json` (lowercase). Family B. Document only. |
| CAPABILITIES table | `Docs/CAPABILITIES.md` § OutputFormat | Missing list-paths, scan-roots, whoami, adopt/rebind, graph, nightly, harness, memory list, project list. |
| PROTOCOL-COMPAT §5 | `Docs/PROTOCOL-COMPAT.md` | No list-paths / scan-roots rows. Compact↔pretty without a flag is breaking — inventory already gates via `--format`. |
| Tests | `tests/project_path_aliases.rs` | Forces `--format human` / `json`. No `--format pretty`. No clap `JSON`/`Pretty` on list-paths. |
| T74 | `smoke.rs` | Piped `graph update` must parse as JSON. Do not touch. |

### 2.4 Dependency / standards research (2026-08-18)

| Pin / source | Workspace / live | Action |
|--------------|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** / docs.rs 4.6.1 `PossibleValuesParser` is **case-sensitive** (T249 AC16 already relies on this) | **No bump.** Expand `value_parser` lists only. Snapshot — re-verify at execute. |
| `serde_json` | lock **1.0.150** / crates.io **1.0.151** | **No bump.** Existing `to_string_pretty`. |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| `is-terminal` crate | **Removed** 2026-08-16; CLI uses `std::io::IsTerminal` | Keep. Do not re-add the crate. |
| clap 5 | not this track | Forbidden. |
| New crates | — | **Zero.** |
| [CLIG — Output](https://clig.dev/#output) | Humans first; TTY heuristic; `--json` for structure; human output may evolve; scripts pin `--json` | Keep Family A `auto`. Help names `--format human` and `--format json`. |
| [CLIG — Future-proofing](https://clig.dev/#future-proofing) | Changing human output is usually OK | Human tables are **not** wire contracts. JSON **keys** frozen. |
| [CLIG — Consistency](https://clig.dev/#consistency-across-programs) | Same flag names for the same thing | Inventory joins T248/T249 token set. |
| T180 / PROTOCOL-COMPAT | compact `{text, word_count}` freeze; compact↔pretty without a flag is breaking | Do not grow preflight DTO. Inventory already has `--format`. |
| SQLCipher / schtasks | N/A — presentation only | N/A (written). |

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Taxonomy (four families)** | **A auto:** TTY human / pipe JSON (`scope resolve`, `retention plan`, `list-paths`, `scan-roots`, `whoami`, `adopt-path`, `rebind-path`, graph neighbors/hierarchy/session, preflight, briefing, recall). **B always-human:** pipes stay human (`nightly --status`, `doctor`, `daemon status`, `memory list`, `project list`, `device list`/`status`, `harness status`). **C always-JSON:** default JSON, opt-in human (`retention apply`, `graph update`). **D governed:** `OutputFormat::parse` silent-JSON (evidence/source/review/policy/…). CAPABILITIES lists **every** row. |
| **F2 — Nightly exception** | Family B. Default `--format human`; piped `nightly --status` **stays human** (T255 F2/AC10). Do **not** silently switch pipes to JSON. Scripts pass `--format json`. The table must call this out by name (audit stub + T255 contract). |
| **F3 — Inventory token map** | `list-paths`, `scan-roots`, `whoami`, `adopt-path`, `rebind-path` clap `value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]`. Default stays **`auto`**. Unknown / `JSON` / `Pretty` → clap `InvalidValue` exit **2** (T249 AC16). No lowercase coerce. |
| **F4 — Shared resolver** | Those five commands call `format_resolve::resolve_human_json_format` (or a one-line wrapper that compares to `"json"`). **Delete** the three `use_json_output` functions. whoami: replace only the format `match` in `project.rs` — **no** other hotspot edits. |
| **F5 — JSON keys frozen** | `--format json` envelopes unchanged: list-paths `{api_version, paths:[{project_id,label,alias,normalized_path,exists}]}`; scan-roots `{api_version, scan_root, truncated, roots:[…]}`; whoami / adopt / rebind existing keys. Pretty-printed. No new required keys. |
| **F6 — Human is not a wire contract** | Tables / labeled lines may gain columns later. Scripts pin `--format json`. `markdown`/`md` tokens resolve to the **same table** as `human` (shared resolver) — do not invent a markdown renderer. |
| **F7 — Help** | list-paths + scan-roots `after_help` must say: default `auto` = TTY table / pipe JSON; agents that want a table pass `--format human`; scripts pass `--format json`. Examples already exist — extend, do not replace T254/T259 filter examples. |
| **F8 — Decline graph-update TTY-auto** | T246 F6 / T74 stand. Default JSON. `auto` stays JSON. `--format human` remains opt-in labeled lines. Soft F17 stays soft. |
| **F9 — Decline retention apply flip** | T248: apply default JSON; `auto` does not TTY-switch. Dangerous / scripted. |
| **F10 — Decline recall default** | T101/T243: TTY pretty / pipe json. Not inventory. Do not change `resolve_format`. |
| **F11 — Decline T227 F34** | Do **not** change `OutputFormat::parse`. Governed list/show stay silent-JSON-on-unknown. Future `parse_or_fail` is not this track. |
| **F12 — Decline T265 / T180** | Do not grow `PreflightContextResponse`. Full envelope stays T265. |
| **F13 — Decline T267 / T268 / T270** | Footer leftover-as-AI-Brains; scan-roots parent/`--root` / already-registered suggested line; retention 0-candidate classify. |
| **F14 — Decline T240 F2 / T255 bag** | No silent Scope/`.env`. No doctor 16th. No product `nightly-run.cmd`. No live `schtasks` mutate. |
| **F15 — Cursor #179** | T264 `safety_ids` over-exclude is **T272**, not this track. Do not edit `ai-brains-retrieval`. |
| **F16 — clap / crates** | No clap 5. No lock bumps. No new crates. Workspace **0.1.1**. No ValueEnum DoD. |
| **F17 — Contracts** | No `ai-brains-contracts` DTO. PROTOCOL-COMPAT **additive** list-paths + scan-roots rows (TTY/pipe + frozen keys). CHANGELOG T266 row. CLI-EXIT-CODES unchanged (exit 2 already usage). |
| **F18 — Tests** | Naming `function_or_feature__condition__expected_result`. Clap AC for list-paths (and one sibling) `pretty` parses, `xml`/`JSON`/`Pretty` InvalidValue. Hermetic: `--format pretty` list-paths contains the human header `path` + `project_id` and is **not** JSON; `--format json` parses `api_version == "1"`. Existing T254/T259 hermetics stay green. No `unwrap`/`expect`/`panic` in production. |
| **F19 — Cross-model** | FEATURE (operator format contract). After Phase-1 review clean, run read-only `codex-review`. |
| **F20 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals go to `conductor/deferred.md`. |
| **F21 — PATH-behind** | Do **not** `cargo install` unless the user asks. Tests/manual AC use `cargo run` / hermetic bin. |
| **F22 — Capture independence** | Presentation + docs only. No events. No models. No graph rebuild. |
| **F23 — Stop-before** | Even after go: do not write `.env`, do not live `rebind-path --write`, do not `retention apply --confirm`, do not mutate Nightly/Router. |
| **F24 — Graph local resolver** | `resolve_graph_format` stays in `graph.rs`. Do not force it through `resolve_human_json_format` (pretty ≠ human). |
| **F25 — Harness / memory / project list** | Family B. Document in the table. Do **not** add `auto` or change defaults. |
| **F26 — Decline extras** | Color / pager / `comfy-table`; clap ValueEnum unify; `std` IsTerminal already shipped; doctor compact JSON DTO; T204 Start-here rewrite that reorders F31 groups (additive one-liner in CAPABILITIES only). |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Existing `resolve_human_json_format` units stay green (auto TTY/pipe, pretty aliases, json, fail-closed). No behavior change to the helper. |
| **AC2** | Clap: `project list-paths --format xml` → `InvalidValue` exit **2**. `--format JSON` → `InvalidValue`. `--format Pretty` → `InvalidValue`. `--format pretty` **parses** (not InvalidValue). Same four asserts for `project scan-roots`. |
| **AC3** | Hermetic: `list-paths --format pretty` on empty vault contains `No path aliases registered.` and `next: ai-brains project register-path` and does **not** start with `{`. |
| **AC4** | Hermetic: `list-paths --format json` (two aliases) parses one object, `api_version == "1"`, `paths` is an array (T254 keys present). `--format human` on the same vault prints the `path` header and does not parse as JSON. |
| **AC5** | Hermetic: `scan-roots --format pretty` on a temp dir with one `.ledgerful` child contains the `path` / `registered_to` header (or `No .ledgerful roots found.`) and is not JSON. `--format json` parses `api_version`, `scan_root`, `truncated`, `roots`. |
| **AC6** | Unit or compile: `use_json_output` **does not exist** in `project_paths.rs` / `project_adopt.rs` / `project_rebind.rs`. Call sites go through `resolve_human_json_format`. |
| **AC7** | Clap: `project whoami --format pretty` parses. `project adopt-path --format pretty` parses. `project rebind-path C:\x --to y --format pretty` parses (does not run the remediator in the clap unit). |
| **AC8** | Existing T254/T259 list-paths / scan-roots / rebind hermetics stay green. T258 adopt-path hermetics stay green. T240 whoami hermetics stay green. |
| **AC9** | T74 / T246: piped `graph update` (graph-on) still parses as JSON when `--format` omitted. Do not add a failing assert that it is a table. |
| **AC10** | Hermetic or process: default `nightly --status --quick` (no `--format`) still prints `=== Nightly Status ===` when stdout is not a TTY. |
| **AC11** | Docs: CAPABILITIES OutputFormat table lists Families A–D and names nightly as Family B (pipes stay human). PROTOCOL-COMPAT has list-paths + scan-roots rows. Root CHANGELOG has a T266 row. list-paths + scan-roots `after_help` mention `--format human` for agents. |
| **AC12** | No contracts DTO. No pin bumps. No new crates. `OutputFormat::parse` unchanged (unit or grep). `ai-brains-retrieval` untouched. |
| **AC13** | Manual (source bin, this agent non-TTY): `project list-paths --format human` is a table; default `project list-paths` is JSON; `retention plan --format human` is the T248 matrix; `nightly --status --quick` stays human. Do **not** pin. Do **not** `cargo install`. |

---

## 5. Design notes

### 5.1 Token map (inventory)

Same string as T248/T249/T255:

```text
auto | pretty | human | text | json | markdown | md
```

Resolver output is only `"human"` or `"json"`. Inventory printers already branch on that boolean. `markdown`/`md` do **not** grow a markdown table.

### 5.2 whoami hotspot

```rust
// project.rs whoami — replace the match only
let use_json = crate::commands::format_resolve::resolve_human_json_format(
    format,
    std::io::stdout().is_terminal(),
) == "json";
```

Do not move `build_whoami_report` / `display_label` / detect order.

### 5.3 CAPABILITIES table shape (additive rewrite of the existing matrix)

Keep the honesty sentence (“no blanket TTY default flip for governed JSON”). Expand rows so an operator can predict the next command. Family labels in the Notes column. Nightly row must say **pipes stay human**.

### 5.4 Why not flip list-paths default to always-human?

That would break scripts that already pipe `list-paths` and parse `api_version`. T254 shipped `auto` on purpose. CLIG: scripts pin `--format json`; humans on a TTY already get the table. The audit score is discoverability, not a false default.

---

## 6. Non-goals

- clap 5 / lock bumps / new crates / ValueEnum
- T265 preflight `sections[]` / T180 growth
- T246 F6 `graph update` TTY-auto
- T255 nightly pipe→JSON
- T227 F34 `OutputFormat::parse` surface-wide
- T267 list footer / whoami next / harness self-next
- T268 scan-roots parent / `--root` / already-registered suggestion
- T270 retention live classify
- T272 `--global` safety_ids Index skip
- T240 F2 silent Scope; T255 doctor 16th / product `.cmd` / live task mutate
- Color, pager, `comfy-table`
- `cargo install` / live `.env` / live `rebind-path --write` / `retention apply --confirm`
- Printing `AI_BRAINS_KEY`

---

## 7. Verification plan

TDD: failing clap + hermetic names first (Phase 1), then wire resolver (Phase 2), then docs (Phase 3).

| Phase | Proof |
|-------|-------|
| Red | AC2 / AC3 / AC7 clap+hermetic fail because `pretty` is not in `value_parser` |
| Green | F3–F4; AC1–AC8 green; AC6 forks gone |
| Freeze | AC9 graph update; AC10 nightly human pipe |
| Docs | AC11 |
| Manual | AC13 source bin, classify-only |
| Gate | fmt / clippy `-D warnings` / targeted nextest / deny / audit on go. Full workspace gate at finalize. |
| Review | `review.md` then `codex-review` (F19) |

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Scripts that passed `--format pretty` to list-paths and expected clap 2 | **Additive** — they get a table (what pretty means everywhere else). Unlikely; token was invalid. |
| Scripts that passed `JSON` (uppercase) via whoami lowercase coerce | whoami **already** has case-sensitive `value_parser`. No change. |
| Hotspot `project.rs` churn | F4: format match only. |
| Accidental `OutputFormat::parse` edit | F11 / AC12. |
| Accidental graph-update flip | F8 / AC9 / T74. |
| Scope exceeds inventory | F12–F15. Stop-before if a review asks to flip Family B/C/D. |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit format maze; list-paths 7/5; retention 6/5 | **Absorb** F1–F7 / AC2–AC5 / AC11 / AC13 |
| Shared `resolve_*_format` (T249 pin later closed 2026-08-16) | **Absorb** F4 — use the existing helper; do not extract a second one |
| T227 F34 OutputFormat surface-wide | **Decline** F11 — governed contract; not inventory |
| T246 F17 TTY-auto `graph update` | **Decline** F8 — T246 F6 / T74 |
| T255 F2 nightly pipes | **Affirm** F2 / AC10 |
| Compact JSON `note_machine_stdout` (T257 residual) | **Decline** — T265/T266 intentional; this track does not restyle compact JSON |
| T265 `preflight --format json` blob | **Decline** F12 → T265 |
| T267 harness/whoami/list next | **Decline** F13 |
| T268 scan-roots cwd / re-register suggestion | **Decline** F13 |
| T269 nightly vs Router mix | **Decline** — not format |
| T270 retention 0 candidates | **Decline** F13 |
| T271 sync query ledger pane | **Decline** — not format |
| T240 F2 / T255 declines | **Decline** F14 |
| last-PR Cursor #179 safety_ids | **Mint T272** F15 — still true; does not fit T265–T271 |
| R-CI-BRANCH / MSI / packaging | **Not related** — packaging |
| T214 F9 ledgerful-on-global | **Not related** — preflight retrieval |
| `anyhow` RUSTSEC-2026-0190 allowlist | **Not related** |
| Connector cursor / CE / DataKey rotation | **Not related** |
| T264 leftover recall drop | **Decline** — T264 F11 stands; not format |

---

## 10. Implement order (on go)

1. Phase 0 re-verify pins + deferred rescan + confirm `pretty` still clap-rejects on list-paths.
2. Red: AC2/AC3/AC7 tests.
3. Green: expand five `value_parser`s; wire resolver; delete three forks; whoami match only.
4. Docs: CAPABILITIES + PROTOCOL-COMPAT + CHANGELOG + after_help.
5. Confirm AC9/AC10 still green without product edits.
6. Targeted clippy/nextest; Phase-1 review; codex-review; full gate; publish.

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| T246 F17 TTY-auto `graph update` | Stays soft. Do not absorb. |
| T227 F34 `parse_or_fail` | Stays residual. |
| Harness status no `value_parser` | Family B; unknown tokens currently lowercase-compare. Soft only. |
| TTY/`auto` hermetic (T254 F12) | Suite still forces `human`/`json`. Same `IsTerminal` as whoami. |
| PATH `cargo install` | F21. |
| CAPABILITIES Start-here T204 F31 order | Additive table only; no group reorder. |

---

## 12. Touch map

| Path | Why |
|------|-----|
| `crates/ai-brains-cli/src/main.rs` | Five `value_parser` lists + clap AC tests + after_help |
| `crates/ai-brains-cli/src/commands/project_paths.rs` | Delete `use_json_output`; call shared resolver |
| `crates/ai-brains-cli/src/commands/project_adopt.rs` | Same |
| `crates/ai-brains-cli/src/commands/project_rebind.rs` | Same |
| `crates/ai-brains-cli/src/commands/project.rs` | whoami format match **only** |
| `crates/ai-brains-cli/src/commands/format_resolve.rs` | Read-only unless a one-line `== "json"` wrapper is cleaner |
| `crates/ai-brains-cli/tests/project_path_aliases.rs` | AC3–AC5 hermetics |
| `Docs/CAPABILITIES.md` | Family table |
| `Docs/PROTOCOL-COMPAT.md` | list-paths + scan-roots rows |
| `CHANGELOG.md` | T266 row |
| `conductor/conductor.md` | T266 Planned note (status stays Pending until implement) |
| `conductor/deferred.md` | Absorb/decline + T272 pointer |
| **Do not touch** | `governed_common.rs` `OutputFormat::parse`; `graph.rs` resolver; `nightly.rs` default; `recall.rs`; `ai-brains-retrieval`; `ai-brains-contracts`; `project.rs` beyond the whoami match |
