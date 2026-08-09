# T220 — Preflight summary JSON honesty

- **Track ID:** T220-PreflightSummaryJson
- **Phase:** Post-audit CLI quality series (T217–T232) — P1 honesty after T217
- **Status:** 📋 **Planning** (plan-only until **go**)
- **Depends on:** T214 dual counts + Scope SOOT ✅; T180 `PreflightContextResponse` key freeze ✅; T170 D21 (summary ≠ governed authority); T216 `MemorySummaryJson` envelope pattern ✅; T235 harness sibling (human-only, no arity growth on T214 formatter)
- **Blocks / feeds:** Agent/scripts that pipe `preflight --summary --format json`; residual T219 pretty wall; optional harness array later
- **Category:** BUGFIX / CONTRACT / UX
- **Source:** Non-destructive CLI audit 2026-08-05 — `preflight --summary --format json` **quality 3** (flag lie); T214 soft residual F11/F24
- **Deferred absorbed:** deferred.md T214 F11/F24 summary JSON DTO; series README T220 row; CAPABILITIES “JSON remains non-summary only”
- **Not absorbed:** Grow `PreflightContextResponse` keys (T180); harness JSON array as hard DoD; T219 pretty readability; T228 non-empty pretty Scope; ledgerful-on-global; clap 5; MSI; clap 4.6 workspace bump
- **Research date:** 2026-08-09 (live dogfood + code map + clig.dev / clap pins / memory JSON SOOT)
- **AI fold-in:** 2026-08-09 — AI1 affirms F1–F10 core + three blind spots (already mostly F4/F8/F2). AI2 **M1–M7** accepted (M1 hard; M2 scope enum; M3 case-safe format; M4–M7 docs/AC); **M8** soft residual affirm. Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start` on go)

## 1. Objective

1. **Stop the flag lie:** `ai-brains preflight --summary --format json` must **not** print the human banner `--- AI-Brains Preflight Summary ---`.  
2. **Emit a machine object** with Scope + dual vault/in-context counts (same numbers as human summary).  
3. **Preserve T180:** non-summary `preflight --format json` stays exactly `{ "text", "word_count" }` compact.  
4. **CLI-local DTO** (prefer no `ai-brains-contracts` change) with `api_version: "1"`.  
5. **Capture independence:** no models, embeddings, or graph required.  
6. **T170 D21:** summary JSON remains **orientation only** — never governed claim authority.

## 2. Live baseline (re-scan 2026-08-09)

### 2.1 Dogfood

| Command | Observed |
|---------|----------|
| `preflight --summary` | Human dual-count summary; Scope correct (T214) |
| `preflight --summary --format json` | **Same human text** — `--format` ignored |
| `preflight --summary --pretty` | Same human summary (pretty does not unlock full body when `--summary`) |
| `preflight --format json` (no summary) | Compact `{"text":"…","word_count":N}` — T180 OK |
| `memory list --summary --format json` | Pretty envelope `api_version`, `scope`, `project_id`, counts — **SOOT for shape** |

### 2.2 Root cause (frozen)

```text
// preflight.rs run():
if options.summary {
    print_summary(...);  // always human println lines
    return Ok(());       // never consults options.format / pretty
}
// format path only runs for full preflight
```

### 2.3 Touch map

| Site | Role |
|------|------|
| `ai-brains-cli/src/commands/preflight.rs` | Branch summary on format; pure JSON builder; pure human lines unchanged |
| `ai-brains-cli/src/main.rs` | Soft: tighten `--format` help / `value_parser` if free (`human`/`json`/`pretty`) |
| Hermetic | Extend `tests/preflight_global_summary.rs` or new `preflight_summary_json.rs` |
| Unit | Pure serialize / key locks / global projects omit |
| `protocol_compat_cli.rs` | Must stay green: non-summary still 2 compact keys |
| Docs | CAPABILITIES preflight JSON row; PROTOCOL-COMPAT inventory row; CHANGELOG |

### 2.4 Deps / pins (researched 2026-08-09)

| Item | Workspace / note |
|------|------------------|
| clap | Workspace **`4.5`** (resolved **4.6.1**); crates.io latest **4.6.6** — **no bump** DoD |
| serde / serde_json | **1.0** — reuse; zero new crates |
| is-terminal | Soft residual migrate to `std::io::IsTerminal` — not T220 DoD |
| Contracts | **No** `PreflightContextResponse` field growth |

### 2.5 Online / product research

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — machine-readable JSON when format flag set | Honor `--format json`; stdout pure JSON (no human banner) |
| clig.dev — human default | Default `--summary` remains human |
| Memory / project list envelopes | `api_version: "1"`, pretty `to_string_pretty`, snake_case keys |
| T180 PROTOCOL-COMPAT | Full preflight JSON stays **compact** + 2 keys; **summary JSON is a new path** (pretty OK) |
| T170 D21 | Docs: summary JSON ≠ governed packet |
| T214 F11 | Was soft decline → **this track implements** the machine object |

## 3. Frozen decisions (F1–F34)

| ID | Decision |
|----|----------|
| **F1 — Primary fix** | Implement **machine object** for `--summary --format json`. **Do not** choose exit-2-only as the product outcome (exit 2 only for invalid format if parser rejects). |
| **F2 — Format resolution (summary)** | When `summary`: `format` matches **`json` case-insensitively** (`eq_ignore_ascii_case`) → JSON path; else human path. `--pretty` alone does **not** unlock full preflight body under `--summary`. Unknown/non-json values on the **summary** path → **human** (document divergence from full path: full preflight treats non-human/pretty as JSON today). |
| **F3 — Separate DTO (T180)** | New **CLI-local** `PreflightSummaryJson` (name flexible). **Never** add summary fields to `PreflightContextResponse`. Non-summary path unchanged. |
| **F4 — Envelope keys (frozen)** | Pretty JSON object: |
| | • `api_version`: `"1"` (string) |
| | • `scope`: **`"global"` \| `"project"` \| `"none"`** (F29) — not the human `Scope: …` line |
| | • `project_id`: string uuid or `null` (`null` when global or none) |
| | • `projects`: `Option<u64>` — **only when `scope=="global"`** via `#[serde(skip_serializing_if = "Option::is_none")]`; **never** emit `projects: null` or `0` under project/none |
| | • `pinned`: u64 (vault SQL; same SOOT as human “Pinned memories”) |
| | • `active_sessions`: u64 (SQL SOOT) |
| | • `in_context_hotspots` / `in_context_decisions` / `in_context_constraints`: usize — **legacy-path marker** counts of rendered text (`HOTSPOT:` / `DECISION:` / `CONSTRAINT:`) (F31) |
| | • `word_count`: usize — **full preflight budget-window text** (`context.word_count`), **not** the size of the summary payload (F30; human “Total Word Count:” parity) |
| **F5 — Style** | Summary JSON uses **`serde_json::to_string_pretty`** (memory/project list family). Full non-summary preflight stays **compact** `to_string` (T180). |
| **F6 — Pure builder** | Extract pure `build_preflight_summary_json(...)` (or Serialize struct from the same inputs as `format_preflight_summary_lines`) — unit-testable without vault I/O. |
| **F7 — Human path unchanged** | Default summary human lines + footer + T214 dual model + T235 harness sibling **unchanged** when not JSON. |
| **F8 — Harness under JSON (M1 hard)** | **v1: omit** harness array from summary JSON (soft residual). JSON path rules: |
| | 1. **Never** print harness human block on **stdout** (`format_harness_summary_lines` bypassed for stdout). |
| | 2. **Never** interactive install prompt (`AskOnce` skipped; treat as non-interactive). |
| | 3. **`--install-hooks` MUST still run** install side effects (not silent no-op). **Every** install status line that is `println!` today (already installed, no ready harness, Installed…, DryRun, Declined, etc.) must go to **`eprintln!` / tracing** on the JSON path so stdout remains one parseable JSON document. Enumerate sites in `append_harness_summary_and_maybe_prompt` / `report_preflight_install` (preflight.rs ~286–408, 462–467). Prefer a `stdout_json: bool` / `emit_status: Fn` parameter over duplicated install logic. |
| | 4. Soft residual: optional `harnesses[]` array later. |
| **F9 — Stdout purity** | JSON success: **entire** stdout (after trim of a single trailing newline) is **exactly one** JSON document. No banner, footer, harness lines, or install chatter. Env override warnings remain **stderr** (existing). |
| **F10 — Capture independence** | Counts + markers only; no models/graph. |
| **F11 — Governed honesty** | CAPABILITIES / docs: summary JSON is orientation; governed truth stays `preflight --format json` (full text) / `briefing` (T170). Under governed rendering, `in_context_*` may be **0** (no legacy markers) — document (F31). |
| **F12 — Exit codes** | Success **0** when build succeeds. No new exit class for “unsupported summary json”. Do **not** introduce a case-sensitive clap reject of `--format JSON` (F13). |
| **F13 — Format parser (M3)** | **Do not** add clap `value_parser = ["human","json","pretty"]` **without** `ignore_case` — PossibleValuesParser is case-sensitive and would **regress** `--format JSON` on the full path. **v1:** keep string + `eq_ignore_ascii_case("json")` for summary. Soft residual: `ValueEnum` + `#[arg(ignore_case = true)]` unifying both paths later. |
| **F14 — Global projects key** | Match human: include `projects` only under `--global` (SQL `count_projects_with_pinned`). |
| **F15 — Scope enum vs line** | Machine `scope` is the three-valued enum (F29); do **not** require `scope_line` key in v1. Soft residual: add `scope_line` if agents need alias text. |
| **F16 — Zero new crates** | — |
| **F17 — High findings** | Silent human fallback when `format=json`; polluting JSON with banner/harness/install `println!`; **silent no-op of `--install-hooks` on JSON path**; growing `PreflightContextResponse`; claiming summary JSON as governed; auto-prompt install on JSON path; emitting `scope:"project"` with vault-wide counts when unresolved (F29). |
| **F18 — Hermetic locks** | Multi-project: `--global --summary --format json` → parse OK; `scope=="global"`; `projects>=2`; `pinned>=2`; **no** banner substring; project-scoped: no `projects` key; `api_version=="1"`; plus AC13–AC15. |
| **F19 — Protocol lock** | `t180_c_preflight_json_keys__cli_format_json__compact_stable_keys` remains green (no `--summary`). |
| **F20 — Docs (M4/M5/M6)** | CAPABILITIES: summary JSON keys; `word_count` = full budget text not summary size; `in_context_*` = legacy markers (may be 0 under governed); `scope` three-valued; T180 full path still 2 compact keys. PROTOCOL-COMPAT new row (summary pretty). CHANGELOG. **Hard:** clap help for preflight `--format` lists accepted values and notes `--summary` honors `--format json` (case-insensitive). Soft skill one-liner. |
| **F21 — Determinism** | Stable field order via struct serde; no timestamps in envelope. |
| **F22 — Soft residuals** | Optional `harnesses[]` array; `scope_line` string; is-terminal → `std::io::IsTerminal` (M8: pin already latest 0.4.17 final); clap 4.6 workspace pin; ValueEnum ignore_case unify; T219 pretty wall; shared `scope_display.rs`. |
| **F23 — Not T219** | Pretty body readability out of scope. |
| **F24 — Footer human** | Keep “Use --pretty or --format json for full context.” on human summary (full packet, not summary JSON). |
| **F25 — Review** | BUGFIX honesty + contract surface; primary review required. Cross-model soft (CLI JSON). |
| **F26 — Implement order** | Pure JSON builder red (incl. scope none + projects omit) → wire format branch → **JSON install-hooks stderr path (M1) before ship** → hermetic AC1–AC8/AC8b/AC13–15 → protocol_compat green → docs F20 → gate. |
| **F27 — Ledger TX** | On go: `ledgerful ledger start T220-preflight-summary-json --category BUGFIX` (or FEATURE). |
| **F28 — Plan-only** | No production code until user **go**. |
| **F29 — Scope three-valued (M2 hard)** | Machine `scope` mapping: |
| | • `--global` → `"global"` (`project_id: null`; include `projects`) |
| | • `project_id` present → `"project"` (`project_id` string; omit `projects`) |
| | • else → **`"none"`** (`project_id: null`; omit `projects`) — matches human `Scope: project=(none)` honesty without lying that counts are project-scoped |
| | Counts SOOT unchanged: when `project_id` is `None`, SQL `count_*` filters are unscoped (vault-wide) — same as human today. Docs state: **`scope:"none"` ⇒ unresolved project; pinned/active_sessions are vault-wide.** Do **not** use `"project"` + null id for this case. |
| **F30 — word_count semantic (M4)** | Document only; keep field name `word_count` for human parity. Not “summary object size.” |
| **F31 — in_context_* SOOT (M5)** | Marker scan of rendered text only; AC5 fixtures use **legacy** (non-governed) path so counts are meaningful. Docs: governed markdown may yield zeros. |
| **F32 — Case JSON hermetic (M3/M7)** | Hermetic covers `--format JSON` (uppercase) on summary path. |
| **F33 — Single-document assert (M7)** | Hermetic: `serde_json::from_str(stdout.trim())` succeeds as one object; no dual documents / leading human text. |
| **F34 — projects Option only (AI1 #1)** | `projects: Option<u64>` + skip_serializing_if; unit asserts project-scoped serialize string lacks `"projects"`. |

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | Hermetic: `--summary --format json` exit 0; stdout is one JSON object (F33); **does not** contain `--- AI-Brains Preflight Summary ---`. |
| **AC2** | Required keys present: `api_version`=`"1"`, `scope`, `project_id`, `pinned`, `active_sessions`, `in_context_hotspots`, `in_context_decisions`, `in_context_constraints`, `word_count`. |
| **AC3** | Hermetic multi-project `--global --summary --format json`: `scope`=`"global"`, `projects` ≥ 2, `pinned` ≥ 2, `project_id` is null. |
| **AC4** | Hermetic project-scoped: `scope`=`"project"`, `project_id` matches fixture; **no** `projects` key (F34). |
| **AC5** | In-context counts match marker SOOT on **legacy-path** fixture (or pure unit 1:1); do not use governed-only fixture for AC5. |
| **AC6** | Non-summary `preflight --format json` still exactly 2 compact keys `text` + `word_count` (protocol_compat). |
| **AC7** | Human `--summary` (no format) unchanged dual model + banner (regression). |
| **AC8** | JSON path: no interactive harness install prompt; stdout pure JSON. |
| **AC8b** | Hermetic (or targeted): `--summary --format json --install-hooks` → exit 0 or honest non-zero on refuse; **stdout still pure JSON** (parses; no banner); **stderr** carries install outcome text when any install path runs (not silent no-op of the flag). |
| **AC9** | Pure unit: Serialize for global (`projects` present) vs project (key absent) vs **none** (`scope=="none"`, no `projects`). |
| **AC10** | Docs: CAPABILITIES (keys + word_count + in_context + scope none) + PROTOCOL-COMPAT + CHANGELOG + format help (F20). |
| **AC11** | Full CI gate; zero new crates; capture-independent. |
| **AC12** | Manual dogfood: live vault `--summary --format json` jq-able; full `--format json` still compact 2-key. |
| **AC13** | Hermetic: `--summary --format JSON` (uppercase) takes JSON path (F32). |
| **AC14** | Hermetic: no `--global`, no project env/id → `scope`=`"none"`, `project_id` null, no `projects` key (F29). |
| **AC15** | Hermetic AC1 family: stdout trim parses as **exactly one** JSON value (F33). |

## 5. Non-goals

- Growing `PreflightContextResponse` / daemon preflight DTO  
- Harness JSON array as hard DoD (soft F22)  
- Changing dual-count SQL semantics when unresolved (counts stay vault-wide; only **label** honesty via `scope:"none"`)  
- T219 pretty wall / role strip (T224)  
- Governed multi-project packet / summary-as-authority  
- Ledgerful under `--global`  
- clap 5 / dependency bumps / is-terminal migrate DoD  
- Unifying full-path unknown-format→json with summary human fallback (document only)  

## 6. Verification plan

| Phase | Proof |
|-------|-------|
| Red | Pure builder + hermetic AC1 fail on current binary (human banner) |
| Green | Branch in `run`/`print_summary`; pure + hermetic green |
| M1 | Install-hooks JSON path: stderr status, stdout pure (AC8b) before claiming done |
| Targeted | `cargo nextest run -p ai-brains-cli -E 'test(preflight)'` + protocol_compat; clippy `-p ai-brains-cli` |
| Manual | Live `--summary --format json` / `--format JSON` parse; full JSON still 2 keys |
| Full gate | fmt, clippy workspace, nextest workspace, deny, audit, ledgerful verify |
| Review | `review.md`; soft cross-model |

## 7. Risks

| Risk | Mitigation |
|------|------------|
| Agents scrape human banner | AC1 / F33 lock; pure JSON stdout |
| Silent T180 break | AC6 protocol_compat |
| Harness / install `println!` breaks `jq` | F8 M1 hard; AC8b |
| Silent `--install-hooks` no-op | F8 run installs; AC8b |
| `scope:"project"` + vault-wide counts lie | F29 `scope:"none"` + AC14 |
| `--format JSON` regression | F2/F13 case-insensitive; AC13; no case-sensitive value_parser |
| word_count / in_context misread | F30/F31 docs |
| Claiming governed truth | F11 docs |

## 8. Coordination

- **T214:** dual counts + Scope — reuse SQL + marker scan; close F11 soft residual.  
- **T180:** full preflight JSON freeze.  
- **T216:** `MemorySummaryJson` envelope pattern.  
- **T235:** harness human sibling; install path stdout sites.  
- **T219:** pretty body later.  
- **T170:** summary ≠ governed.  

## 9. Suggested implement snippet (guidance only)

```rust
if options.summary {
    let json_mode = options
        .format
        .as_deref()
        .is_some_and(|f| f.eq_ignore_ascii_case("json"));
    print_summary(..., json_mode, ...)?;
    return Ok(());
}

// scope machine enum
let scope = if global {
    "global"
} else if project_id.is_some() {
    "project"
} else {
    "none"
};
```

Human: existing `format_preflight_summary_lines` + harness.  
JSON: `println!("{}", serde_json::to_string_pretty(&PreflightSummaryJson { ... })?)` only; install status via `eprintln!` when `json_mode`.

## 14. AI fold-in (2026-08-09)

Sources: `C:\dev\AI-review.md` — AI1 (architecture affirm + 3 blind spots) + AI2 (M1–M8 deep findings).

| Item | Source | Disposition |
|------|--------|-------------|
| **Diagnosis** early return ignores format | AI1 + AI2 | **Affirmed** — root cause freeze §2.2 |
| **Schema / F1–F10** machine object, T180, pretty, capture-indep | AI1 | **Affirmed** — already plan |
| **AI1 #1** projects Option + skip_serializing | AI1 | **Absorbed** F4/F34/AC4 — hard |
| **AI1 #2** harness stdout isolation | AI1 | **Elevated with M1** F8 |
| **AI1 #3** case-insensitive json match | AI1 | **Absorbed** F2/AC13 |
| **M1** install-hooks × JSON unspecified / println pollution or silent no-op | AI2 High | **Absorbed** F8 rewrite + **AC8b** hard DoD |
| **M2** scope project + null + vault-wide counts | AI2 Med-High | **Absorbed** F29 `scope:"none"` + **AC14** |
| **M3** value_parser case-sensitive / format path diverge | AI2 Med | **Absorbed** F2/F13 (no case-sensitive parser); document summary unknown→human; soft ValueEnum |
| **M4** word_count = full budget text | AI2 Med | **Absorbed** F4/F30/F20 docs |
| **M5** in_context_* zero under governed | AI2 Low/Med | **Absorbed** F31/AC5 legacy fixture + docs |
| **M6** format help incomplete | AI2 Low | **Absorbed** F20 hard help text |
| **M7** hermetic gaps (JSON case, unresolved, single doc) | AI2 Low | **Absorbed** AC13–AC15 / F32–F33 |
| **M8** is-terminal final 0.4.17 | AI2 Low | **Soft residual** F22 — no T220 DoD migrate |
| Dep pins clap/serde_json | AI2 | **Affirmed** — no bumps |
| T216 envelope SOOT | AI2 | **Affirmed** |
| Auto-implement as coded | tone | Plan-only until **go** |

**Rejected / not absorbed:** changing SQL count filters for unresolved scope (would break human parity without T214 revisit); forcing summary unknown-format → JSON (surprising vs human default); emoji/noise; contracts growth.
