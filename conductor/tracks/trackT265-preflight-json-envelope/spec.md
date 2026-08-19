# T265 — Preflight JSON structured envelope

- **Track ID:** T265-PreflightJsonEnvelope
- **Status:** **Planned** (Pending in registry; plan-only until go)
- **Category:** UX / CONTRACTS / FEATURE
- **Owner:** —
- **Source:** Audit 2026-08-16 — `preflight --format json` **7/6**; `{text, word_count}` blob
- **Depends on:** T180 compact `{text, word_count}` freeze ✅; T219 newline-preserving `text` ✅; T220 summary JSON (separate path) ✅; T250 compact-ignored-on-JSON ✅; T264 `[8hex]` inside `text` ✅; T266 Family A default (TTY human / pipe JSON) ✅
- **Blocks / feeds:** Agents can `jq '.sections[] | select(.id=="safety")'` without scraping markdown. Scan-roots stays **T268**. Nightly/Router split stays **T269**. Retention classify stays **T270**. Ledger pane stays **T271**. Safety-skip leftover stays **T272**.
- **Absorbs:** Audit T265 row (full non-summary `--format json` is a paste blob); T220/T264 “do not grow `PreflightContextResponse`” *as the standing freeze until this track*; T180-C `obj.len()==2` *as the silent-growth guard this track is allowed to lift*; T214 residual “`PreflightContextResponse` extra keys”; compact JSON “uses `note_machine_stdout` not pretty” (T257) — keep compact
- **Not absorbed:** `--summary --format json` (T220 closed); `--global` isolation (T264 closed); format maze (T266 closed); T272 `safety_ids`; clap 5 / new crates; typed `constraints[]` / governed packet on this surface; `json-v2` opt-in flag
- **Research date:** 2026-08-19 (plan dogfood HEAD `2a00ce3` T267 `#181`)
- **AI fold-in:** none yet (plan-track only)
- **Ledger:** planning DOCS TX `5fa57d64-9fac-4a8e-932a-d0f23c29f347`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** write live `.env`. Do **not** enable `AI_BRAINS_GOVERNED_BRIEFING`. Do **not** edit `ai-brains-retrieval/src/preflight.rs` (T272). Do **not** reopen T240 F2 / T255 declines. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** flip nightly pipes / graph-update JSON.

---

## 1. Objective

Non-summary `preflight --format json` stays a **paste blob** (`text`) for LLM/dogfood, and becomes a **structured envelope** so agents do not re-parse markdown to pick Safety vs Session.

1. **Keep required keys.** `text` and `word_count` stay required, compact `to_string`, same semantics (budget-window content words; no Scope/PrettyCaps chrome).
2. **Add `sections[]`.** Always present. E1 empty is `[]`, never `null`, never omitted. Each section is `{id, title, items}` derived from the same `text`.
3. **This is the T180 track.** `t180_c_preflight_json_keys` currently asserts `obj.len()==2` with the comment “must not grow silent keys **without a track**.” T265 is that track. Required keys stay; additive `sections` is documented, not silent.
4. **Do not invent a second assembly.** Split headers already in `context.text`. Do not retune retrieval SQL, caps, or T272 `safety_ids`.

That advances the north star: capture stays grant-independent; the append-only log stays SoT; agents piping `preflight --format json` get machine sections without a new clap token.

No models. No new crates. No clap 5. No `json-v2`. No typed authority arrays (that is `briefing --format json`).

---

## 2. Live baseline (2026-08-19)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `2a00ce3` T267 `#181`. Tree CLEAN. `main` = `origin/main` (0 ahead). |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **0.1.1** (mtime 2026-08-18 20:08 — **PATH-behind** T267). `--format json -m 80` → keys `['text','word_count']` len **2**. **Do not `cargo install`.** |
| Source `cargo run -p ai-brains-cli -- preflight --format json -m 200` | keys `['text','word_count']` len **2**. `word_count=200`. `text` starts `--- Repository Bearings & Safety ---\n…`. Has `--- Session:`. No Memory Index at `-m 200` (budget). **22** newlines. Compact (one JSON document). **Live hole.** |
| `preflight --summary` | Scope path owner `3581317d` (`C:\dev\ai-brains`). Pinned **3089** (**volatile** — do not lock). Grants **0 of 3** (T241). Five harnesses wiring=ok. |
| `preflight --summary --format json` | T220 pretty object (not this track). |
| Last GitHub PR | [#181](https://github.com/Ryan-AI-Studios/AI-Brains/pull/181) T267. Issue comments **[]**. Review comments **[]**. Reviews **[]**. Open PRs on `main`: none. **last-PR Cursor: N/A.** |
| #179 Cursor leftover | T272 still true: `preflight.rs:329` `safety_ids.insert` before `:336` cap; Index skip `:467`. **Do not remint / steal.** |
| Ledgerful | `doctor` ready (legacy `.changeguard` / sig-pin / timings / :8081 unreachable; :8083 ok). 0 pending 0 drift. Hotspot **#1** `project.rs` (4.064) — **do not edit.** **#7** `preflight.rs` (2.287, **2148** lines) — JSON builder goes in a **sibling**. Retrieval `preflight.rs` **1041** lines — **do not edit** (T272). |
| ai-brains recall | T220 “never grow `PreflightContextResponse`”; T264 F12 “T265 `sections[]`”; T180 2-key freeze. No prior “additive sections on default `--format json`; lift len==2 with a track” pin. |

### 2.2 Why the blob is still a product hole

| Residual | Why it is still a hole / why decline |
|----------|--------------------------------------|
| `{text, word_count}` only (7/6) | `text` has newlines + `--- Section ---` headers (T219). Agents still scrape markdown to pick Safety vs Session. **DoD = additive `sections[]`.** |
| T180 `obj.len()==2` | Guard against *silent* growth. Test comment: “without a track.” **This track lifts the length freeze**, not the required keys. |
| `--format json-v2` | Agents already pass `--format json`. A new token leaves the audit hole on the default flag. Dogfood `parse_legacy_preflight` already ignores extra keys. Serde default ignores unknown. **Decline json-v2.** |
| Typed `constraints[]` / `decisions[]` | That is governed `briefing --format json` / T170 D21. Marker scrape of `text` stays for dogfood. **Decline typed authority on this surface.** |
| Grow retrieval assembly | T272 owns `safety_ids`. T264 caps live there. JSON can split existing `text`. **Do not edit retrieval `preflight.rs`.** |
| Summary JSON | T220 pretty envelope. Separate path. **Leave.** |
| Compact → pretty | PROTOCOL-COMPAT: compact↔pretty without a flag is breaking. Nested strings may contain `\n` (escaped). Document stays **one line**. **Keep `to_string`.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Contracts DTO | `ai-brains-contracts/src/preflight.rs` `PreflightContextResponse { text, word_count }` | **Only** emit site is CLI JSON. Daemon/HTTP do **not** serialize this type (ledgerful search + grep). Dogfood comment names it. |
| CLI emit | `commands/preflight.rs:279–286` | `to_string`; `note_machine_stdout`; no Scope/caps. |
| Default format | `:233–245` | TTY + no `--format` → human; pipe → json; `--pretty` / `human` / `pretty` (case-insensitive) → human; everything else → JSON. **Do not add `value_parser`** (T220 F13: case-sensitive parser would regress `--format JSON`). |
| Pretty splitter | `:353–373` `is_legacy_section_header` / `classify_section_header` | Already classifies Safety / Session / Index / Recent / Other from `---` headers. JSON sibling may **copy** the match table (do not refactor pretty this track). |
| Pretty sibling | `commands/preflight_pretty.rs` | T264 tag peel. **Leave** except `mod` already exists. |
| Retrieval assembly | `ai-brains-retrieval/src/preflight.rs` | Headers: `--- Repository Bearings & Safety ---`; `--- Session: {id} ---` (+ `[8hex]` when global); `--- Memory Index (Briefing) ---`; `--- Most Recent Memories ---`; `--- Ledgerful Intelligence ---` / `(Contextual Risk)`; `--- AI-Brains: New Repository Detected ---`. T272 `:329` / `:467`. **Do not edit.** |
| Internal context | `retrieval::PreflightContext { text, word_count, in_context_project_span }` | Span is summary-only. **Do not grow** this track. |
| Summary DTO | CLI-local `PreflightSummaryJson` | T220. Comment “Never grows `PreflightContextResponse`” — rewrite that comment; summary path still does not grow into the full DTO. |
| T180 lock | `tests/protocol_compat_cli.rs` `t180_c_preflight_json_keys__cli_format_json__compact_stable_keys` | compact + `text` + `word_count` + **`len==2`**. |
| Other `len==2` | T219 AC7; T250 AC12; T264 AC9 | Must update to required keys + `sections` array (AC9 still asserts `[8hex]` in `text`). |
| Dogfood | `dogfood.rs` `parse_legacy_preflight` | Requires `text`; `word_count` optional; **ignores extra keys**. 2-key fixtures must still Deserialize (`#[serde(default)]` on `sections`). |
| Help | `main.rs` Preflight `--format` docstring | Still says “keeps compact `{text, word_count}`”. after_help examples include `--format json`. |
| PROTOCOL-COMPAT §5 | “Keys: `text`, `word_count` only (T180 freeze)” | This track’s docs DoD. |
| CAPABILITIES | “never grow those keys” | This track’s docs DoD. |

### 2.4 Dependency / standards research (2026-08-19)

| Pin / source | Workspace / live | Action |
|--------------|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** / docs.rs: `PossibleValuesParser` is **case-sensitive**; `ignore_case` exists but T220 F13 forbids adding a parser without it | **No bump.** No `value_parser` on preflight `--format`. Snapshot — re-verify at execute. |
| `serde` / `serde_json` | workspace serde **1.0** / lock serde_json **1.0.150** / crates.io **1.0.151** | **No bump.** Default deserialize **ignores unknown fields** (serde.rs; T180 F26: must **not** add `deny_unknown_fields`). `#[serde(default)]` on additive `sections` for N−1 2-key files. |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.1** | **No bump.** |
| clap 5 | not released (max 4.6.x) | Forbidden. |
| New crates | — | **Zero.** |
| [CLIG — Output](https://clig.dev/#output) | `--json` for structure; jq; humans first | Default `--format json` (already the pipe default) must be structured, not a second flag. |
| [CLIG — Future-proofing](https://clig.dev/#future-proofing) | Keep changes additive; JSON is the stable interface | Additive `sections`; keep `text`/`word_count`. Compact style unchanged. |
| [CLIG — Consistency](https://clig.dev/#consistency-across-programs) | Same flag names | Stay `--format json`. No `--json-v2`. |
| T180 PROTOCOL-COMPAT | compact freeze; additive extra-field helper; `len==2` is the silent-growth lock | Required keys stay; length lock lifts; compact stays. |
| SQLCipher / schtasks | N/A — presentation + DTO | N/A (written). |
| Kelly Brazil / CLI JSON tips | Predictable keys; schema; extra fields ignored by consumers | Frozen `id` closed set as **strings** (not a rust enum on the wire) so future ids still deserialize. |

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Additive on default `--format json`** | Grow `PreflightContextResponse`. Required: `text` (string), `word_count` (usize). Additive **always-present** `sections` (array). Compact `serde_json::to_string`. No `json-v2`. No pretty-print of this object. |
| **F2 — `sections` shape** | Each element: `{ "id": string, "title": string, "items": string[] }`. Field order: `id`, `title`, `items`. `id` is a **string** (not a rust enum on the wire). Closed set for **this** schema: `safety` \| `session` \| `index` \| `recent` \| `ledgerful` \| `empty_repo` \| `governed` \| `other`. Unknown future ids must still deserialize (no `deny_unknown_fields`). |
| **F3 — E1** | `sections` is never `null` and never omitted. Empty → `[]`. Empty body under a header → `items: []`. N−1 2-key JSON deserializes with `sections: []` via `#[serde(default)]`. |
| **F4 — Split from `text`** | Pure `split_preflight_sections(text: &str) -> Vec<PreflightSection>` in new `commands/preflight_json.rs`. Input is `context.text` **after** retrieval assembly (post T219 F2b). Do **not** re-query SQL. Do **not** apply PrettyCaps. Do **not** strip role prefixes (JSON `text` may keep them — T219 honesty). |
| **F5 — Header table** | Full-line `---` … `---` (same rule as `is_legacy_section_header`). Map: Bearings/Safety → `safety`; `--- Session:` / `--- Session ` → `session` (one section **per** session header); Memory Index → `index`; Most Recent Memories → `recent`; Ledgerful Intelligence → `ledgerful`; New Repository Detected → `empty_repo`; else → `other`. `title` is the trimmed header line (including `[8hex]` on global Session headers). |
| **F6 — Items** | Body after the header, **blank-line-separated blocks** (same idea as pretty `split_item_blocks`). Trim each block. Drop empty blocks. Index numbered lists without blank lines become **one** item — acceptable v1 (section `id` is the agent win). Trailing F2b `…` / `... [Index Truncated]` stay in the current section’s items. |
| **F7 — No headers** | If `text` has no `---` header: `sections = []` **unless** it contains `# Project Briefing (governed)` → one section `{id:"governed", title:"Project Briefing (governed)", items:[full text]}`. Do **not** scrape `#` / `##` as general headers (T219 F14). Governed authority stays `briefing --format json`. |
| **F8 — `text` / `word_count` freeze** | Semantics unchanged: retrieval budget-window; no Scope chrome; `--compact` ignored; T264 `[8hex]` remains **inside** `text` (and in `title`/`items` when those lines were tagged). `word_count` is still content words of `text`, not of the JSON payload. |
| **F9 — T220 summary** | `--summary --format json` stays pretty `PreflightSummaryJson`. No `sections` on that path. Rewrite the “Never grows `PreflightContextResponse`” comment to “Summary DTO stays CLI-local; full JSON may add `sections` (T265).” |
| **F10 — Decline json-v2 / typed arrays** | No `--format json-v2`. No `constraints[]` / `decisions[]` / `hotspots[]` on this DTO. Dogfood still marker-scans `text`. |
| **F11 — Decline retrieval / T272** | Do **not** edit `crates/ai-brains-retrieval/src/preflight.rs`. `safety_ids` skip stays T272. Do not grow `retrieval::PreflightContext`. |
| **F12 — Hotspot** | New `crates/ai-brains-cli/src/commands/preflight_json.rs` (`pub mod` in `mod.rs`). CLI `preflight.rs` JSON arm becomes a short call. Do **not** grow pretty/summary/grants. Do **not** edit `project.rs` / `governed_common.rs`. Do **not** unify pretty’s walker with the JSON splitter this track (copy the header table; pretty stays). |
| **F13 — Contracts** | Update `ai-brains-contracts` DTO + crate docs. PROTOCOL-COMPAT §5 + T180-C row: required `text`/`word_count`; additive `sections`; compact unchanged. CHANGELOG T265. CAPABILITIES: stop “never grow those keys.” CLI-EXIT-CODES unchanged (exit 0). No daemon/HTTP handler change (type unused there). |
| **F14 — Format parser** | Keep string + case-insensitive human/pretty. Unknown / `json` / `JSON` → JSON path. Do **not** add clap `value_parser` (T220 F13 / T249 AC16 class). |
| **F15 — Stdout purity** | Success: stdout (trim one trailing newline) is **exactly one** compact JSON document. `note_machine_stdout` stays. Env/identity warns stay stderr (T257). |
| **F16 — clap / crates** | No clap 5. No lock bumps. No new crates. Workspace **0.1.1**. |
| **F17 — Tests** | Naming `function_or_feature__condition__expected_result`. Pure split units + N−1 deserialize unit + hermetic JSON. Update T180-C / T219 AC7 / T250 AC12 / T264 AC9 `len==2` to required keys + `sections` is array. T220 summary hermetics stay green. No `unwrap`/`expect`/`panic` in production. |
| **F18 — Cross-model** | FEATURE + contracts DTO. After Phase-1 review clean, run read-only `codex-review`. |
| **F19 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F20 — PATH-behind** | Do **not** `cargo install` unless the user asks. Tests/manual AC use `cargo run` / hermetic bin. |
| **F21 — Capture independence** | Split + serialize only. No events. No models. No graph rebuild. No governed flag. |
| **F22 — Stop-before** | Even after go: do not write `.env`, do not enable `AI_BRAINS_GOVERNED_BRIEFING`, do not live `rebind-path --write`, do not mutate Nightly/Router, do not print `AI_BRAINS_KEY`. |
| **F23 — Decline T240 F2 / T255 bag** | No silent Scope/`.env`. No doctor 16th. No product `nightly-run.cmd`. No live `schtasks`. |
| **F24 — Decline peers** | T268 scan-roots; T269 Router split; T270 classify; T271 ledger pane; T272 `safety_ids`. |
| **F25 — No version key** | Do **not** add `schema_version` / `api_version` on this DTO (detector is presence of `sections`). T220 summary keeps its own `api_version`. |
| **F26 — Help** | Refresh Preflight `--format` docstring + `after_help` one line: JSON is compact `{text, word_count, sections}` ; `--summary --format json` stays T220. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Hermetic: `preflight --format json` exit 0; stdout trim is **one** JSON object; **no** pretty newlines at document level (`trim` contains no raw `\n` **or** T180-C’s existing “compact” assert still holds). Required keys `text`, `word_count`, `sections`. `sections` is an array (never null). |
| **AC2** | Named: `t180_c_preflight_json_keys__cli_format_json__compact_stable_keys` still asserts compact + `text` + `word_count`, and **now** asserts `sections` is array. **Does not** assert `obj.len()==2`. |
| **AC3** | Pure unit: fixture text with Bearings + one Session + Memory Index + Recent → four (or more) sections with ids `safety`, `session`, `index`, `recent` in encounter order. `title` matches the header line. `items` non-empty for bodies. Named: `split_preflight_sections__legacy_headers__ids_in_order`. |
| **AC4** | Pure unit: two `--- Session: … ---` headers → **two** `id=="session"` sections. Named: `split_preflight_sections__two_sessions__two_section_rows`. |
| **AC5** | Pure unit: no `---` header and no governed marker → `sections` empty. Named: `split_preflight_sections__no_headers__empty`. |
| **AC6** | Pure unit: text contains `# Project Briefing (governed)` and no `---` headers → exactly one section `id=="governed"`. Named: `split_preflight_sections__governed_marker__one_section`. |
| **AC7** | Unit: `serde_json::from_str::<PreflightContextResponse>(r#"{"text":"DECISION: one","word_count":2}"#)` succeeds; `sections` is empty. Named: `preflight_context_response__n_minus_1_two_key__sections_default_empty`. |
| **AC8** | T219 `preflight_pretty__json_format__two_keys_and_newlines_in_text`: still asserts `text` has `\n` and no `Scope:` chrome; **stop** asserting `len==2`; assert `sections` is array. T250 AC12 same (uncapped `text` still contains seed). |
| **AC9** | T264 `preflight_global_isolation__compact_json__two_keys_and_hex_tags`: still asserts `[8hex]` in `text`; **stop** asserting `len==2`; `sections` present. |
| **AC10** | Hermetic `--summary --format json`: still pretty T220 object; **no** `sections` key; still `api_version=="1"`. Named existing suite stays green. |
| **AC11** | Hermetic empty new vault (no pins): JSON parses; `text`/`word_count` present; `sections` is `[]` **or** one `empty_repo` if the new-repo header is in `text` (do not require both). |
| **AC12** | Docs: CAPABILITIES full JSON row names required keys + `sections` ids + E1 `[]` + compact; PROTOCOL-COMPAT §5 + T180-C row; CHANGELOG T265; clap `--format` docstring (F26). No pin bumps. No new crate. `deny_unknown_fields` absent on this DTO. Retrieval `preflight.rs` **untouched** (diff). |
| **AC13** | Manual (source bin, this agent non-TTY): `preflight --format json -m 200` parses; `jq`/python shows `sections` with a `safety` id when Bearings is in `text`. Do **not** pin. Do **not** `cargo install`. |
| **AC14** | Existing T220 summary hermetics + T214/T241 grants summary JSON stay green. Dogfood `parse_legacy_preflight` unit with extra `sections` key still counts markers from `text`. |
| **AC15** | Pure unit: Ledgerful Intelligence header → `id=="ledgerful"`. Unknown `--- Foo Bar ---` → `id=="other"`. Named: `split_preflight_sections__ledgerful_and_other`. |
| **AC16** | CLI `preflight.rs` JSON arm does not inline serde struct construction of `sections` (call the sibling). `preflight_json.rs` exists. Pretty `classify_section_header` **may** remain in `preflight.rs` (F12 — no pretty refactor required). |

---

## 5. Design notes

### 5.1 Envelope (compact)

```json
{"text":"--- Repository Bearings & Safety ---\nCONSTRAINT: …","word_count":200,"sections":[{"id":"safety","title":"--- Repository Bearings & Safety ---","items":["CONSTRAINT: …"]}]}
```

`jq '.text'` and dogfood keep working. `jq '.sections[] | select(.id=="safety")'` is the new remediator.

### 5.2 Why not json-v2

T180’s length assert is a **process** lock (“without a track”), not a forever 2-key product claim. Consumers that deserialize with serde already ignore extras. `parse_legacy_preflight` already ignores extras. A new clap token would leave `--format json` as the audit hole. CLIG: JSON is how scripts get structure on the flag they already pass.

### 5.3 Why not typed constraints

Governed packet already has `decisions` / `conclusions` / `constraints`. Full preflight is the **legacy string-scrape** path (T170 D21 / dogfood). Re-implementing marker taxonomy here forks authority. Section `id` is the minimum structure the 7/6 score asked for.

### 5.4 Why split `text` instead of retrieval structs

Retrieval `preflight.rs` is T272’s file (`safety_ids` pre-cap). T264 caps live there. JSON structure must not drift from the blob agents still paste: one `text`, then a pure split. If split is wrong, unit fixtures are the `---` headers already in tests.

### 5.5 Hotspot

CLI `preflight.rs` is **2148** lines, hotspot **#7**. New types + split + emit helper in `preflight_json.rs`. `mod.rs` adds `pub mod preflight_json`. JSON arm in `run` stays a few lines.

---

## 6. Non-goals

- `--format json-v2` / `--structured`
- Typed `constraints[]` / `decisions[]` / `hotspots[]`
- Pretty-print of full preflight JSON
- `--summary` envelope keys
- Retrieval SQL / T264 caps / T272 `safety_ids`
- Pretty walker unification
- clap `value_parser` on `--format`
- clap 5 / lock bumps / new crates
- T240 F2 / T255 doctor 16th / product `.cmd` / live tasks
- T268 / T269 / T270 / T271
- `schema_version` / `api_version` on this DTO
- Enabling `AI_BRAINS_GOVERNED_BRIEFING`
- `cargo install` / printing `AI_BRAINS_KEY`

---

## 7. Verification plan

TDD: failing pure split + N−1 deserialize + T180-C length (Phase 1), then contracts + emit (Phase 2), then update existing `len==2` tests (Phase 3), then docs (Phase 4).

| Phase | Proof |
|-------|-------|
| Red | AC3–AC7 / AC15 fail (no splitter / 2-key only). T180-C still `len==2`. |
| Green DTO + split | F1–F7; AC3–AC7 / AC15. |
| Green emit | AC1 / AC2 / AC8 / AC9 / AC11 / AC16. |
| Freeze | AC10 T220; AC14 dogfood; retrieval file untouched. |
| Docs | AC12 / F26. |
| Manual | AC13 source bin, classify-only. |
| Gate | fmt / clippy `-D warnings` / targeted nextest (`preflight` + `protocol_compat`) / deny / audit on go. Full workspace gate at finalize. |
| Review | `review.md` then `codex-review` (F18). |

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Scripts assert `keys \| length == 2` | T180-C + T219/T250/T264 tests **are** those scripts. Update them. CHANGELOG + PROTOCOL-COMPAT. Dogfood ignores extras. |
| `deny_unknown_fields` accidentally added | F13 / AC12 grep. T180 F26. |
| Split drifts from pretty | F12: copy header table; do not share walker. Units lock headers. |
| Governed markdown mis-classified as `other` | F7 / AC6. Empty `---` → `[]` unless governed marker. |
| Retrieval/T272 steal | F11 / AC12 diff. Stop-before if a review asks to fix `safety_ids` here. |
| Growing hotspot `preflight.rs` | F12 sibling. AC16. |
| PATH-behind | F20. Manual AC uses `cargo run`. |
| Compact document gains pretty newlines | Keep `to_string`. AC1 / T180-C. |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit `preflight --format json` `{text, word_count}` blob (7/6) | **Absorb** F1–F8 / AC1–AC4 / AC13 |
| T214 residual “`PreflightContextResponse` extra keys” | **Absorb** — this is that growth, scoped to `sections` |
| T220 F3 / CAPABILITIES “never grow those keys” | **Absorb** as the freeze this track lifts; summary path still CLI-local (F9 / AC10) |
| T264 F12 “T265 `sections[]`” | **Absorb** — this track |
| T266 F12 “do not grow T180” | **Absorb** as prior freeze; T266 stays Completed |
| T257 “compact JSON uses `note_machine_stdout` not pretty” | **Affirm** F15 — keep compact + `note_machine_stdout` |
| T220 F11/F22 harnesses[] / scope_line on summary JSON | **Decline** — summary path |
| T219 F13 project-scoped selection | **Decline** — not JSON envelope |
| T272 #179 `safety_ids` | **Decline** F11 / F24 — still true at `:329` + `:467` |
| last-PR Cursor #181 | **N/A** — comments/reviews empty |
| T268 / T269 / T270 / T271 | **Decline** F24 |
| T240 F2 / T255 bag | **Decline** F23 |
| R-CI-BRANCH / MSI / packaging | **Not related** — packaging |
| `anyhow` RUSTSEC-2026-0190 allowlist | **Not related** |
| Connector cursor / CE / DataKey rotation | **Not related** |
| Daily 0 of 3 grants | **Not related** — T241 |
| Compact JSON “Family A auto” | **Affirm** T266 F1 — default TTY/pipe unchanged |

---

## 10. Implement order (on go)

1. Phase 0 re-verify pins + deferred rescan + confirm source JSON is still 2-key and T272 still at `:329`/`:467`.
2. Red: AC3–AC7 / AC15 units; T180-C still documents today’s `len==2` until green.
3. Green: contracts DTO + `preflight_json.rs` split + emit; AC1–AC2 / AC16.
4. Update T219 AC7 / T250 AC12 / T264 AC9; keep T220 green (AC10).
5. Docs: CAPABILITIES + PROTOCOL-COMPAT + CHANGELOG + clap help (F26).
6. Targeted clippy/nextest; Phase-1 review; codex-review; full gate; publish.

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| Pretty walker ≠ JSON splitter (duplicated header table) | F12. Unify later if they drift. |
| Index without blank lines = one item | F6 v1. Line-split for `index` only is optional later. |
| `schema_version` on this DTO | Declined F25. |
| PATH `cargo install` | F20. |
| T272 `safety_ids` over-exclude | Peer placeholder. |
| Summary `harnesses[]` / `scope_line` | T220 F22. |
| Pin count / leftover roots / 0 of 3 grants | Volatile / T259 / T241. |

---

## 12. Touch map

| Path | Why |
|------|-----|
| `crates/ai-brains-contracts/src/preflight.rs` | Additive `sections` + nested type + `serde(default)` |
| `crates/ai-brains-cli/src/commands/preflight_json.rs` | **New.** Split + build response |
| `crates/ai-brains-cli/src/commands/mod.rs` | `pub mod preflight_json` |
| `crates/ai-brains-cli/src/commands/preflight.rs` | JSON arm call; comment on summary DTO; **no** pretty refactor |
| `crates/ai-brains-cli/src/main.rs` | `--format` docstring + after_help one-liner (F26) |
| `crates/ai-brains-cli/src/commands/dogfood.rs` | Only if a unit needs an extra-key fixture (AC14); parser already ignores extras |
| `crates/ai-brains-cli/tests/protocol_compat_cli.rs` | T180-C AC2 |
| `crates/ai-brains-cli/tests/preflight_pretty_readability.rs` | AC8 |
| `crates/ai-brains-cli/tests/preflight_global_isolation.rs` | AC9 |
| `crates/ai-brains-cli/tests/preflight_json_envelope.rs` | New hermetics AC1 / AC11 if not folded into existing |
| `Docs/CAPABILITIES.md` | Full JSON row |
| `Docs/PROTOCOL-COMPAT.md` | §5 + T180-C |
| `CHANGELOG.md` | T265 row |
| `crates/ai-brains-retrieval/src/preflight.rs` | **Do not touch** |

---

## 13. AI fold-in

None this pass. Review-track writes `agy-review.md` / `opencode-review.md` only. Fold-in later.
