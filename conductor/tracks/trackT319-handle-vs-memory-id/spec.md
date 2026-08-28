# T319 — Governed handle vs vault memory ID namespace

- **Track ID:** T319-HandleVsMemoryId
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `evidence show` / `source show` on a vault `memory_id` → `Handle not found` / `NOT_FOUND`. Two UUID namespaces look identical. Series README `README-T312-T324-CLI-DOGFOOD.md`.
- **Depends on:** T160/T203 governed show; T263 H1 Unknown preview `Handle not found.` (exit **0**); T290 granted-empty lists; T314 expand `--format`; `UNKNOWN_HANDLE_PREVIEW` / `EXIT_NOT_FOUND`; `QueryStore::memory_exists` (T77 forget)
- **Blocks / feeds:** Operators who paste `recall` / `graph neighbors` ids into governed show. Daily vault search stays `recall`.
- **Absorbs:** Audit UUID namespace confusion; evidence Unknown empty preview (T263 overlay never wired on `evidence show`)
- **Not absorbed (DoD):** H2 auto-resolve memory → evidence; fabricating evidence/source rows; DTO required-key growth; T290 list empty copy; T316–T318 / T320–T325
- **Research date:** 2026-08-28 (plan-write product HEAD `fa353c7` T317 `#234`). Snapshot — **re-verify at execute**.
- **Ledger:** planning DOCS TX `844bdbed-7295-4635-a04f-968d224e41ec`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install`. Do **not** grow hotspot `governed_common.rs` (#3) — new sibling `governed_namespace.rs`. Do **not** edit `expand_handle` in control-plane. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** live `policy bootstrap` / `migrate governed` / production `pin`.

---

## 1. Objective

1. **Wrong-namespace is named.** If the UUID exists as a vault `memory_id` (`memory_projection`) but not as a governed handle / source, human + JSON say so and point at `ai-brains recall "what did we decide"`. Do **not** show the memory body. Do **not** coerce it into evidence.
2. **Unknown-unknown stays E1.** Truly missing ids keep T263 `Handle not found.` (expand / evidence) or `NOT_FOUND: source {id}` (source) with **no** namespace sentence.
3. **Exit codes freeze.** `query expand` / `evidence show` Unknown stay exit **0**. `source show` miss stays exit **4** `NOT_FOUND`. Do **not** invent a new product code (CLI-EXIT-CODES 0–7 freeze).
4. **North star.** Capture independence: CLI overlay + existing `QueryStore::memory_exists`. No new events. No hidden CoT. No H2 promotion.

This unblocks daily CLI: operators who paste a recall id into `evidence show` must not conclude the vault is empty of that memory, and must not treat governed show as `memory show`.

---

## 2. Live baseline (re-scan 2026-08-28)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `fa353c7` `feat(cli): T317 graph neighbors RECALLS cap + hierarchy leaf next (#234)`. Tree **CLEAN**. Branch `track/T319-handle-vs-memory-id` off `origin/main` (ahead **0** at plan-write). |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. **T263 expand overlay is on PATH.** **T314 `--format` on expand is not** (`--format` → clap `--log-format`). **T312 / T315 / T313 / T317 are not.** T319 hole **is**. **Do not `cargo install`.** Tests/manual AC use hermetic bin / `cargo run` for T314 human expand. |
| `preflight --summary` (PATH) | Pinned **4554**. In-context **0/0/0**. `Total Word Count: 740` (PATH-behind T315 `Budget window words:`). **Not this DoD.** |
| Live memory `431f6505-50d7-5176-8cda-f8ba2534fe14` | First hit of PATH `recall "graph backend" --no-bridge --limit 1`. Audit dump. |
| `evidence show 431f6505-… --format json` | `kind: "Unknown"`, **`preview: ""`**, exit **0**. Human: `handle: … (Unknown)` / `preview:` empty / `truncated: false`. **No T263 overlay on evidence.** |
| `query expand 431f6505-…` (PATH default JSON) | `kind: "Unknown"`, `preview: "Handle not found."`, `applied_scope: "Repository:3581317d-…"`, exit **0**. **Same payload as a random UUID.** |
| `query expand cccccccc-cccc-cccc-cccc-cccccccccccc` | Identical Unknown + `Handle not found.` exit **0**. |
| `source show 431f6505-… --format json` | `{ "code": "NOT_FOUND", "message": "source 431f6505-…" }` exit **4**. Human stderr `NOT_FOUND: source …`. **No namespace hint.** |
| `evidence list` / `source list` | Authorized empty `items: []` + T290 `next_step` `Ungoverned vault search: ai-brains recall "what did we decide" (Pinned: 4554)`. **Stay-green; do not steal.** |
| Last GitHub PR | [#234](https://github.com/Ryan-AI-Studios/AI-Brains/pull/234) T317. `mergedAt` **2026-08-28T23:23:15Z**. Issue comments **[]**. Review comments **[]**. Reviews **[]**. Commit comments **[]**. PR body Cursor Bugbot is **overview / Low Risk** (no defect). **last-PR Cursor: N/A empty.** `#230` Bugbot already **T325**. Open PRs: **none**. **No T326.** |
| Ledger | 0 pending / 0 drift at scan. Hotspot **#1** `project.rs` (3.715) — **do not touch.** `sync.rs` **#2** (3.519) — **do not touch.** `governed_common.rs` **#3** (3.389, **1029** lines) — **do not grow**; sibling module. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why two UUID namespaces still look identical

| Layer | Truth |
|-------|--------|
| Governed handles ≠ vault memories | `expand_handle` (`query.rs:538–669`) looks up `evidence_projection`, then conclusion/decision UUID. Miss → `kind: "Unknown"`, **empty** preview. Never reads `memory_projection`. |
| T263 overlay is expand-only | `apply_unknown_expand_preview` (`governed_query.rs:68–86`) fills empty Unknown `preview` with `UNKNOWN_HANDLE_PREVIEW` (`"Handle not found."`). `evidence show` emits the DTO raw — live preview is **blank**. |
| Same shape as a random UUID | PATH expand on the live audit dump **equals** expand on `cccccccc-…`. Operators cannot tell “wrong product” from “id does not exist.” |
| `source show` is a different miss | `SourceId::from_str` then `get_source` (`source.rs:142–164`). UUID parses; miss → `fail_api` `NOT_FOUND` exit **4**. Message `source {id}` names the **command**, not the **other namespace**. |
| No `memory show` | `MemoryCommands` is list-only (`main.rs:3283`). Next-step cannot be `memory show <id>`. Recall of a UUID splits on `-` (T217 `extract_fts_tokens`) — **do not** interpolate the UUID into `recall`. |
| H2 would “fix” this the wrong way | T167 EvidenceId *may* equal `memory_id` **after** `migrate governed`. Live pins are not Approved evidence. Auto-resolve is H2. **Decline.** |
| Exit 4 is already source-only | Placeholder guessed “still exit 4 unless CLI-EXIT-CODES update.” **Live evidence/expand Unknown is exit 0** (T263 / CLI-EXIT-CODES `:106`). Do **not** promote Unknown to 4. |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| Expand CP | `control-plane/src/query.rs` `expand_handle` **`:538–669`** | Evidence SQL → conclusion → decision → Unknown empty. **Do not edit.** |
| T263 preview const | `governed_common.rs:140` | `UNKNOWN_HANDLE_PREVIEW = "Handle not found."` **Keep** for unknown-unknown. |
| T263 overlay | `governed_query.rs` `apply_unknown_expand_preview` **`:68–86`** | JSON Unknown + empty preview. Units `:444`. |
| Expand CLI | `run_expand` **`:174–243`** | `to_value` + `applied_scope` + T263 overlay; human kind then preview; Denied exit 3; Unknown exit 0. T314 `--format`. |
| Evidence show | `evidence.rs` `run_show_local` **`:112–164`** | `expand_handle` then `emit_json(&preview)` / human `handle/preview/truncated`. **No overlay today.** Daemon `:167–201` same emit. |
| Source show | `source.rs` `run_show_local` **`:110–169`** | Invalid id → `INVALID_PAYLOAD` **6**. Miss / wrong scope → `NOT_FOUND` **4**. Found `emit_source`. |
| EXISTS helper | `QueryStore::memory_exists` `query_store.rs:735–742` | `SELECT COUNT(*) FROM memory_projection WHERE memory_id = ?`. T77 forget + graph missing-node. **Reuse; no new projection.** |
| Probe pattern | `graph.rs` `vault_memory_present` **`:71–81`** | `Ok(true/false)` / `Err` → false + warn. **Do not import** — `graph` is `#[cfg(feature = "graph")]`. Duplicate 8-line helper in the new sibling. |
| Pin stdout ≠ memory_id | `graph_human_cli.rs:170–171` | Pin prints turn_id. Hermetics must read `memory list --format json` for `items[].memory_id`. |
| T263 AC10 | `governed_vault_pin_honesty.rs:220–255` | Unknown human expand: exit 0; two nonempty lines `Unknown` then `Handle not found.` **Stay-green** for non-memory UUID `00000000-…`. |
| clap Evidence Show | `main.rs:2636–2661` | default `--format json`; `--max-chars` 512; soft-resolve `--scope`. after_help examples only. |
| clap Source Show | `main.rs:2691–2711` | default json. |
| clap Expand | `main.rs:2505–2522` | default json; Trace tokens; after_help “two lines for Unknown/Denied”. |
| HandlePreviewDto | `contracts/briefings.rs:490–499` | `api_version`, `handle_id`, `kind`, `preview`, `truncated`, optional `source_version_id`. **No `next_step`.** Daemon `EvidencePreview` = this DTO. |
| T290 empty lists | `apply_authorized_empty_list_next` / `LIST_RECALL_QUERY` | **Do not reuse** `Ungoverned vault search:` — that is granted-empty lists, not wrong-id. Needle `"what did we decide"` **is** reused. |
| Review | `review.rs` list/resolve only | **No show.** Do not mint. |
| PROTOCOL-COMPAT | `:112` expand row | DTO keys unchanged; CLI overlay `applied_scope`. **No evidence/source show rows today** — **add**. |
| CAPABILITIES | Show **`:346`** | Soft-resolve only. |
| OPERATIONS | `:281–282` | `evidence show` / `source show` examples. Expand Unknown exit 0 (`:260`). |
| CLI-EXIT-CODES | **4** = `NOT_FOUND`; expand Unknown **0** (`:106`) | Footnote only. |
| Hotspots | `project.rs` #1 / `sync.rs` #2 / `governed_common.rs` #3 **1029** lines | **Do not grow #3.** New `governed_namespace.rs`. `evidence.rs` **305** / `source.rs` **303** / `governed_query.rs` **463**. |

### 2.4 Dependency / standards research (2026-08-28) — snapshot; re-verify at execute

| Pin / source | Workspace / live | Action |
|--------------|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** checksum `1ddb117e…` / crates.io **4.6.6** (2026-08-06) / GitHub **v4.6.6** / **no clap 5** | **No bump.** No new flags. Additive after_help only. |
| `serde_json` | lock **1.0.150** | **No bump.** CLI `Value` overlay. |
| `rusqlite` | workspace exact **0.40.2** | **No bump.** Existing `memory_exists` SQL. |
| `uuid` | workspace **1.13** / lock **1.23.1** | **No bump.** Untouched. |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.3** | **No bump.** |
| New crates | — | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Catch errors and rewrite for humans; suggest the next command | [clig.dev Errors](https://clig.dev/#errors) + [Ease of discovery](https://clig.dev/#ease-of-discovery) (fetched 2026-08-28) | “Handle not found.” is true and useless. Name the other namespace + a copy-paste `recall`. |
| Changing human output is usually OK; JSON keys stay stable | [clig.dev Output](https://clig.dev/#output) / [Future-proofing](https://clig.dev/#future-proofing) | Pretty/human preview string may change when `memory_exists`. Required DTO keys freeze. Additive optional `next_step` on CLI JSON only. |
| Do not lie by omission | [clig.dev Saying (just) enough](https://clig.dev/#saying-just-enough) | Unknown-unknown keeps T263. Wrong-namespace **replaces** “Handle not found.” (that sentence is a lie when the row exists). |
| Name the resource type | kubectl `Error from server (NotFound): pods "x" not found`; Confluent `ListResourceSuggestions` / `ServiceAccountNotFoundSuggestions` | `source {id}` already names the **this** type. Add “this UUID is a vault memory_id”. Do not invent a second resource-type flag. |
| Additive extras OK | T180 P-CLI (`PROTOCOL-COMPAT.md:154`): prefer additive optional fields | CLI overlay `next_step` like T290 lists / T241 `skip_serializing_if`. Daemon `HandlePreviewDto` **unaugmented**. |
| SQLCipher / schtasks | N/A — no new SQL beyond existing EXISTS, no tasks | N/A (written). |

**Could not verify:** a live **evidence** handle on this vault (`evidence list` `items: []`). Hermetic seed is SoT for found-handle stay-green. Daemon InspectEvidence overlay is code-planned (do not require `ai-brainsd` as a plan gate). PATH expand `--format human` is T314-behind — Manual uses `cargo run`.

**ledgerful / ai-brains:** `preflight --summary` Pinned **4554** / 0/0/0 / word **740**. `ledgerful doctor` 4 warn; 0 pending / 0 drift; `index --incremental` 0 files; `search "expand_handle"` → `query.rs:538` / `evidence.rs:127` / `governed_query.rs:190`; `search "memory_exists"` → `query_store.rs:735` / `forget.rs:184` / `graph.rs:71`; `scan --impact` CLEAN at `fa353c7`; hotspots `project.rs` #1. Semantic recall of “handle vs memory id” still dump-first (PATH T312 not installed) — not SoT.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `844bdbed`. Implement starts a **FEATURE** TX. |
| **F1 — Probe after miss** | After `expand_handle` returns `kind == "Unknown"` **or** source `get_source` misses, CLI calls `ctx.conn.memory_exists(&id)`. `Ok(true)` → wrong-namespace overlay. `Ok(false)` / `Err` → unknown-unknown (T263 / current NOT_FOUND). Never `?` the EXISTS `Result` on these read paths (copy `vault_memory_present` into the sibling; do **not** import `graph.rs`). |
| **F2 — Evidence Unknown overlay** | `evidence show` (local **and** daemon emit) runs the same overlay as expand. Unknown + empty preview + **no** memory → T263 `Handle not found.` Unknown + memory → F6 strings. Found Evidence/Conclusion/Decision kinds **unchanged**. |
| **F3 — Expand stays exit 0** | `query expand` Unknown (with or without memory) stays exit **0**. Denied stays **3**. Do **not** map wrong-namespace to `NOT_FOUND` / 4. |
| **F4 — Source stays exit 4** | `source show` miss stays `NOT_FOUND` exit **4**. Wrong-namespace adds `details.hint` (JSON) / extra stderr line via existing `emit_error` hint print (T221 F5). Message stays `source {id}`. Invalid `SourceId` stays **6**. |
| **F5 — Kind stays `Unknown`** | Do **not** invent `kind: "Memory"`. That would look like showing a memory as a handle (H2-adjacent). |
| **F6 — Exact strings** | Preview SOOT: `This UUID is a vault memory_id, not a governed handle.` Human next line: `next: ai-brains recall "what did we decide"` (`LIST_RECALL_QUERY`). JSON `next_step`: `ai-brains recall "what did we decide"` (no `next:` prefix; no `(Pinned: N)`; not T290 `Ungoverned vault search:`). Source `details.hint`: `{preview} {next line}` (one string). |
| **F7 — JSON overlay, not DTO** | Do **not** add `next_step` to `HandlePreviewDto`. Overlay on `serde_json::Value` after `to_value` (T263/T290 pattern). Omit the key when unknown-unknown (`skip` by not inserting). Daemon IPC unaugmented. Evidence JSON may grow optional `next_step` the same way. `applied_scope` stays expand-only. |
| **F8 — No new clap flag** | No `--as-memory` / `--resolve-memory` / `--label`. Soft-resolve `--scope` unchanged. |
| **F9 — Do not coerce** | Do not call `expand_handle` on a synthesized evidence id. Do not `pin` / `migrate governed` / `decision propose` from this path. H2 stays declined. |
| **F10 — `expand_handle` freeze** | Control-plane SQL and Unknown empty preview **unchanged**. Overlay is CLI-only. |
| **F11 — T263 unknown-unknown freeze** | Non-memory UUID: expand human remains **exactly two** nonempty lines `Unknown` then `Handle not found.` (AC10 stay-green). JSON preview remains that const. No `next_step` key. |
| **F12 — T290 lists freeze** | `evidence list` / `source list` / progressive empty copy **untouched**. |
| **F13 — T314 format freeze** | Expand token set + default json + human kind-then-preview **stay**. Human Denied second line `Access denied.` **stay**. Unknown+memory human becomes **three** nonempty lines (kind, F6 preview, F6 next). Update after_help. |
| **F14 — Module** | New `crates/ai-brains-cli/src/commands/governed_namespace.rs`: consts, overlay, source-hint helper, EXISTS-result mapper, their units. `mod.rs` `pub mod governed_namespace`. Callers: `governed_query.rs`, `evidence.rs`, `source.rs`. **Do not** add more than a re-export/use to `governed_common.rs`. |
| **F15 — Capture independence** | EXISTS + overlay + docs. No events. No models. No retrieval rank. |
| **F16 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.3**. |
| **F17 — Contracts** | No required DTO keys. PROTOCOL-COMPAT **adds** evidence show + source show rows + expand `next_step` optional overlay. CHANGELOG + CAPABILITIES + OPERATIONS + CLI-EXIT-CODES footnote + after_help. |
| **F18 — Isolation hotspots** | Do not edit `project.rs` / `sync.rs` / `governed_common.rs` body (beyond a one-line `use` if unavoidable — prefer not). Do not edit retrieval / graph / projector. |
| **F19 — last-PR** | `#234` Cursor **N/A empty**. `#230` F8 recency stays **T325**. **No T326.** |
| **F20 — Standing declines** | clap 5; T240 F2; T263 H2; density floors; T307 Blocked; silent `.env`; `cargo install`. |
| **F21 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `deferred.md`. |
| **F22 — PATH-behind** | Live PATH is pre-T314/T317. Do **not** `cargo install`. Hermetic / `cargo run` prove DoD. PATH-behind is not a fail. |
| **F23 — Tests** | Naming `function_or_feature__condition__expected_result`. Units in the sibling. Hermetics in `governed_vault_pin_honesty.rs` (already has `pin_via_hermetic_cmd` + discovery grants) **or** a new `governed_namespace_cli.rs` if that file would grow >80 net. Seed via pin + `memory list --format json` (not pin stdout). No `unwrap`/`expect`/`panic` in production. `rstest` if ≥2 similar cases. |
| **F24 — Cross-model** | FEATURE (operator JSON overlay). After Phase-1 review clean, run read-only `codex-review`. |
| **F25 — Stop-before** | Even after go: no live bootstrap / migrate / production pin / `.env` rewrite / schtasks / `git push origin main`. |
| **F26 — Dual-truth after_help** | Expand: Unknown-unknown two lines; Unknown+memory three lines + JSON optional `next_step`. Evidence/source show: one sentence that a vault `memory_id` is named, not shown. |
| **F27 — Probe scope** | Memory projection only. Do **not** also probe `session_projection` / graph nodes / pin GLOB. Session-id pasted into evidence show stays unknown-unknown unless it is also a `memory_id`. |
| **F28 — PowerShell** | `;` not `&&`. |
| **F29 — Identity stdout** | Overlay runs after identity-warn JSON emit helpers (`emit_json` / `print_json_stdout`). Do not print secrets. |
| **F30 — Daemon** | CLI overlay after `DaemonResponse::EvidencePreview` / source NOT_FOUND envelope. Do not change `ai-brainsd` InspectEvidence / InspectSource handlers. |
| **F31 — Do not interpolate UUID into recall** | T217 token split. Next-step is the T290 needle, not `recall "{id}"`. Do not point at `graph neighbors` (feature-gated; T317 not this DoD). |
| **F32 — Stay in CLI** | Production net: new sibling + small call sites. If `governed_query.rs` production net ≥80, stop and split further — do not dump helpers into `governed_common.rs`. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `apply_unknown_handle_overlay` on Unknown + empty preview + `memory_exists=true` sets `preview` to F6 const and inserts JSON `next_step` = `ai-brains recall "what did we decide"` |
| **AC2** | Unit: same helper with `memory_exists=false` sets `preview` to `Handle not found.` and **does not** insert `next_step` |
| **AC3** | Unit: helper leaves `kind: "Denied"` / found `Evidence:*` objects unchanged (no preview rewrite, no `next_step`) |
| **AC4** | Unit: source hint string is `{F6 preview} {F6 next line}` |
| **AC5** | Hermetic: pin + discovery grants; `query expand <memory_id> --format json` exit **0**; `kind == "Unknown"`; `preview` F6 const; `next_step` F6 JSON; `applied_scope` still present; **must fail on HEAD** (preview is `Handle not found.`, no `next_step`) |
| **AC6** | Hermetic: `query expand <memory_id> --format human` exit **0**; three nonempty lines: `Unknown`, F6 preview, F6 next line. **Must fail on HEAD** (two lines, T263 preview) |
| **AC7** | Hermetic: `evidence show <memory_id> --scope Repository:{pid} --format json` exit **0**; `kind == "Unknown"`; `preview` F6; optional `next_step` F6 JSON; **no** `applied_scope` key. Human contains F6 preview **and** next line |
| **AC8** | Hermetic: `source show <memory_id> --scope Repository:{pid} --format json` exit **4**; `code == "NOT_FOUND"`; `message` still `source {id}`; `details.hint` contains F6 preview and `recall "what did we decide"`. Human stderr contains `NOT_FOUND` **and** the hint |
| **AC9** | Stay-green T263 AC10: `query expand 00000000-… --format human` still exactly two nonempty lines `Unknown` / `Handle not found.`; JSON no `next_step` |
| **AC10** | Docs: CAPABILITIES Show row; PROTOCOL-COMPAT expand row + **new** evidence show + source show rows; OPERATIONS examples; CLI-EXIT-CODES footnote (Unknown+memory still 0; source still 4); CHANGELOG; Expand/Evidence/Source `after_help` (F26) |
| **AC11** | Manual: `cargo run -p ai-brains-cli -- evidence show 431f6505-50d7-5176-8cda-f8ba2534fe14 --format human` names F6 + next (PATH-behind not a fail). Same id on `query expand --format json`. `source show` that id still exit 4 + hint. Pass-with-observed-data if the id is forgotten later — hermetic AC5–AC8 are SoT |
| **AC12** | Manual: `query expand cccccccc-cccc-cccc-cccc-cccccccccccc --format json` still T263 only (no F6 preview, no `next_step`) |
| **AC13** | Diff: `crates/ai-brains-control-plane/src/query.rs` empty; `HandlePreviewDto` struct empty; `project.rs` / `sync.rs` empty; `governed_common.rs` no new helpers (use-only if needed) |
| **AC14** | Stay-green T290 list empty `next_step` (`Ungoverned vault search:`); T221 expand Denied exit 3; T314 `JSON` InvalidValue exit 2 |
| **AC15** | Stay-green found-handle: hermetic evidence/expand of a real evidence id (if the suite already seeds one) **or** CP unit `expand_handle` already covers found — do not require migrate-governed in CLI hermetic. If no evidence seed exists, do **not** invent H2 seed; AC3 unit is the found-kind guard |
| **AC16** | `cargo fmt --check` + `clippy -p ai-brains-cli --all-targets -- -D warnings` + targeted nextest on new tests + `ledgerful verify --scope fast`. Full workspace gate before Complete, not as a plan gate |
| **AC17** | Pin hermetic uses `memory list --format json` for the id (not pin stdout). Document in the test comment (graph_human_cli `:170` analog) |

---

## 5. Design notes

### 5.1 Overlay order (expand + evidence)

1. `expand_handle` (unchanged).
2. `serde_json::to_value` (evidence today emits the struct; switch to `Value` so overlay can add `next_step`).
3. Expand-only: insert `applied_scope` (already shipped).
4. `let present = namespace_memory_present(ctx.conn.memory_exists(&handle_id));`
5. `apply_unknown_handle_overlay(&mut value, present);` — no-op unless `kind == "Unknown"`.
6. Human: kind line; preview line (post-overlay); if `next_step` present, print `next: …` using F6 human line (do not print the JSON form). Denied fill stays T314.
7. JSON: `emit_json(&value)`.

### 5.2 Suggested helpers (plan, not implemented)

```rust
pub(crate) const WRONG_NAMESPACE_PREVIEW: &str =
    "This UUID is a vault memory_id, not a governed handle.";

pub(crate) fn wrong_namespace_next_line() -> String {
    format!("next: ai-brains recall \"{}\"", crate::commands::governed_common::LIST_RECALL_QUERY)
}

pub(crate) fn wrong_namespace_json_next() -> String {
    format!("ai-brains recall \"{}\"", crate::commands::governed_common::LIST_RECALL_QUERY)
}

pub(crate) fn wrong_namespace_source_hint() -> String {
    format!("{WRONG_NAMESPACE_PREVIEW} {}", wrong_namespace_next_line())
}
```

`apply_unknown_handle_overlay` must **replace** empty/`Handle not found.` preview when `memory_exists`; it must **not** stack T263 + F6 (that would keep the lie).

### 5.3 Why not `graph neighbors <id>`

Useful after T317, but graph-off binaries cannot run it, and this hole is the governed/vault split, not graph cardinality. `recall "what did we decide"` is the standing ungoverned daily path (T263/T290/T315).

### 5.4 Why source stays 4

Scripts already branch on exit 4 / `code: NOT_FOUND`. Changing source-miss to exit 0 would look like expand Unknown and break those scripts. Hint is additive (T180).

---

## 6. Non-goals

- H2 pin → Evidence / Approved / `migrate governed`.
- `memory show <id>` new subcommand (T316 is preview, not show-by-id).
- Interpolating the UUID into `recall` / `graph neighbors`.
- New exit code; promoting expand/evidence Unknown to 4.
- `kind: "Memory"`; required `next_step` on `HandlePreviewDto`.
- Probing sessions / graph nodes / pins GLOB.
- Review/decision/conclusion show (review has no show).
- T290 list copy; T316–T318 / T320–T325 steal.
- clap 5 / new crates / silent `.env` / `cargo install` / live pin.
- Editing `expand_handle` SQL or `governed_common.rs` hotspot body.

---

## 7. Verification plan (TDD)

Red first (must fail while expand preview is `Handle not found.` for a live memory_id and evidence preview is empty):

1. `apply_unknown_handle_overlay__memory_exists__preview_and_next_step` (AC1)
2. `apply_unknown_handle_overlay__unknown_unknown__handle_not_found_no_next` (AC2)
3. `apply_unknown_handle_overlay__non_unknown__unchanged` (AC3)
4. `wrong_namespace_source_hint__contains_preview_and_next` (AC4)
5. Hermetic AC5 `query_expand__memory_id__json_names_namespace`
6. Hermetic AC6 `query_expand__memory_id__human_three_lines`
7. Hermetic AC7 `evidence_show__memory_id__names_namespace`
8. Hermetic AC8 `source_show__memory_id__not_found_hint_exit_4`

Green: F1 probe + F6 overlay + F2 evidence + F4 source hint; wire three callers.

Stay-green: AC9 T263 two-line expand; AC14 T290/T221/T314; AC15 found-kind unit.

Manual AC11–AC12 on go. Docs AC10. No full workspace nextest as a plan gate.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| `Handle not found.` stacked with F6 | F6 replace, not append; AC1/AC5 |
| T263 AC10 two-line lock breaks | F11 / AC9 only fire overlay when EXISTS |
| `next_step` on DTO breaks daemon N−1 | F7 CLI `Value` overlay; AC13 DTO empty |
| EXISTS `Err` panics / `?` exits 1 | F1 mapper; never `?` |
| Import `vault_memory_present` from graph | F14 / F1 — graph feature-gated; duplicate helper |
| Pin stdout used as memory_id | AC17 / `memory list` JSON |
| H2 “just resolve it” during implement | F9 / AC13 `expand_handle` empty diff |
| Source exit flipped to 0 | F4 / AC8 |
| UUID interpolated into recall | F31; needle is `LIST_RECALL_QUERY` |
| `governed_common.rs` #3 grows | F14 sibling |
| PATH-behind false AC fail | F22 / AC11 `cargo run` |
| `#234` leftover dropped | F19 N/A empty; T325 already minted |
| Evidence daemon skip overlay | F30 both local and daemon emit |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-28.

| Item | Disposition |
|------|-------------|
| Audit `evidence show` / `source show` vault UUID | **Absorb** F1–F8 / AC5–AC8 / AC11 |
| Audit `query expand` same hole | **Absorb** F3 / F6 / AC5–AC6 (placeholder §2) |
| Evidence Unknown empty preview (T263 never wired) | **Absorb** F2 |
| T263 H1 `Handle not found.` / exit 0 | **Affirm** F3 / F11 / AC9 |
| T290 empty lists `Ungoverned vault search:` | **Not stolen** F12 / AC14 |
| T263 H2 pin→Approved / migrate | **Decline** F9 / F20 |
| T167 EvidenceId prefers `memory_id` on import | **Not stolen** — import-only; not live show |
| T316 memory-list preview | **Not stolen** |
| T317 RECALLS / T318 backup / T320 status / T321 safety | **Not stolen** |
| T322–T324 T311 residuals | **Not stolen** |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T307 Blocked / T308 floors | **Not stolen** / **Decline** |
| T240 F2 / clap 5 | **Decline** F20 |
| last-PR Cursor `#234` | **N/A empty** F19 — **no T326** |
| last-PR `#230` F8 recency | **T325** already Pending |
| conductor/archive / cargo-audit allowlist | **Not related** |
| PATH T315 `Total Word Count` / T312 dump-first | **Not this DoD** |
| T147 turn `memory_id` golden omission | **Not related** (fixture determinism) |
| T314 leftover `--format` PATH | **Not stolen** (Completed; PATH-behind F22) |

---

## 10. Implement order (on go)

1. Phase 0 re-read `expand_handle` `:538–669`, `run_expand` `:174–243`, `run_show_local` evidence `:112` / source `:110`, `memory_exists` `:735`, T263 AC10 `:220–255`, clap Show/Expand, PROTOCOL-COMPAT `:112`; rescan deferred; FEATURE TX.
2. Red AC1–AC8 (must fail on HEAD overlay). Confirm AC9 still passes (stay-green).
3. Green F14 sibling + F1 mapper + F6 overlay; wire expand / evidence local+daemon / source local+daemon.
4. Stay-green AC9 / AC14 / AC15.
5. Docs F17 / AC10 / F26 after_help.
6. Manual AC11–AC12 → AC13 → review → full gate → Complete.

---

## 11. Soft residuals (expected)

| Residual | Note |
|----------|------|
| PATH until `cargo install` | F22 |
| Live `431f6505` may be forgotten later | Hermetic SoT; Manual observed-data |
| Vault still has 0 governed evidence | Honesty, not populate (H1) |
| No `memory show` | T316/later; next-step is recall |
| Daemon unaugmented DTO | F7 / F30 |
| T312 PATH dump-first | Other track |
| T325 F8 PreferRecency | Placeholder |

---

## 12. Touch map (expected)

| Site | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/governed_namespace.rs` | **New.** F6 consts, overlay, hint, EXISTS mapper, units AC1–AC4 |
| `crates/ai-brains-cli/src/commands/mod.rs` | `pub mod governed_namespace` |
| `crates/ai-brains-cli/src/commands/governed_query.rs` | Wire overlay after T263; human third line; keep `apply_unknown_expand_preview` as the false-branch or fold into overlay |
| `crates/ai-brains-cli/src/commands/evidence.rs` | Local + daemon emit via `Value` + overlay |
| `crates/ai-brains-cli/src/commands/source.rs` | NOT_FOUND + `with_details` hint when EXISTS |
| `crates/ai-brains-cli/tests/governed_vault_pin_honesty.rs` **or** `governed_namespace_cli.rs` | AC5–AC9 / AC17 hermetics |
| `crates/ai-brains-cli/src/main.rs` | Expand / Evidence / Source `after_help` F26 only |
| `Docs/CAPABILITIES.md` | Show row `:346` |
| `Docs/PROTOCOL-COMPAT.md` | Expand `:112` + **add** evidence show + source show |
| `Docs/OPERATIONS.md` | `:260` / `:281–282` |
| `Docs/CLI-EXIT-CODES.md` | Footnote Unknown+memory still 0; source 4 + hint |
| `CHANGELOG.md` | T319 Unreleased |
| `conductor/conductor.md` | T319 Planned (status **Pending**) |
| `conductor/deferred.md` | This absorption table |
| `conductor/tracks/README-T312-T324-CLI-DOGFOOD.md` | T319 Planned |

**Do not touch:** `project.rs`, `sync.rs`, `governed_common.rs` hotspot body, `query.rs` `expand_handle`, `HandlePreviewDto`, retrieval, graph, projector, daemon services, Ledgerful sources, `.env`, schtasks.

---

## 13. AI fold-in disposition

Planning pass — no `*-review.md` yet. Fold-in after `/review-track T319`.
