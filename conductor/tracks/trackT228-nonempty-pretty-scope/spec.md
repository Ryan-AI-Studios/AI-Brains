# T228 — Non-empty recall pretty Scope

- **Track ID:** T228-NonemptyPrettyScope
- **Phase:** Post-audit CLI quality series (T217–T232) — P3 polish after T227
- **Status:** ✅ **Completed** (PR #134 `e51d5e4`, 2026-08-11)
- **Depends on:** T207 empty pretty Scope + `format_scope_line` ✅; T214 preflight summary Scope SOOT ✅; T219 full-body Scope ✅; T211 `print_pretty_hits` shared with sync ✅
- **Blocks / feeds:** Operators who get hits no longer wonder “which project is this?”; aligns recall/sync pretty with memory list + preflight; closes T207 AC10 residual
- **Category:** UX / FEATURE (light) / DOCS
- **Source:** T207 residual **AC10** / M3 — empty pretty always prints `Scope:`; non-empty pretty does not; CAPABILITIES L257 residual callout; series README row “Non-empty pretty no Scope”
- **Deferred absorbed:** deferred.md “Non-empty pretty Scope (T207 soft)” → **DoD**; T207 AC10 residual; series README T228; CAPABILITIES residual sentence; CHANGELOG T207 “non-empty deferred” note **supersede via new T228 entry** (do not rewrite history)
- **Not absorbed:** JSON `scope` field / contract growth; auto-`--global`; ranking/FTS; T230 global labels; T231 unified search product merge; T227 F34 surface-wide `OutputFormat` silent-JSON; clap 5 / ValueEnum; color; pager; **sync TTY-independent default pretty** (pre-existing residual); **sync missing-project → random UUID search** (pre-existing — document, optional soft fix)
- **Research date:** 2026-08-11 (live dogfood + code truth + clig.dev + dep pins)
- **AI fold-in:** 2026-08-11 — AI1 **M1–M4 hard**; **L1–L2 hard**; **O1 elevate hard** (shared helper). AI2 **M1–M3 hard**; **L1 hard**; **L2/L3/L6 soft**; **O2/O3/O5 hard**; **O1 fold into F29**; **O4 soft residual**; **O6 soft cross-model focus**. Disposition **§15**.
- **Ledger:** FEATURE TX `ee28f999-1495-4d82-a45c-665d7ea3c83d` committed; product PR #134 squash-merged `e51d5e4`

## 1. Objective

1. **Always-on Scope chrome for pretty recall:** when format is **pretty**, print the same T207/T214 `Scope:` line for **both empty and non-empty** result sets.
2. **Stable SOOT + single resolver:** one `resolve_active_scope_line` (lookup + `format_scope_line`) for empty/non-empty recall and empty/non-empty sync vault — zero vocabulary drift vs preflight / memory list.
3. **sync query parity:** non-empty pretty vault section also prints Scope under `--- AI-Brains Recall ---` (empty already does via `print_pretty_empty_sync`); ledger-first interleaving keeps Scope **inside** the vault block only.
4. **JSON / contracts frozen:** machine envelope unchanged; no new DTO fields; exit codes unchanged.
5. **Capture independence:** Scope display must not open models, embeddings, or graph.

## 2. Live baseline (re-scan 2026-08-11)

### 2.1 Operator dogfood (this machine)

| Command | Observed |
|---------|----------|
| `recall "DECISION" --limit 2 --format pretty` | **`Session: <uuid>`** then hit lines — **no `Scope:`** |
| `recall "DECISION" --limit 2 --global --format pretty` | Same — Session + hits, **no `Scope: global`** |
| `recall "zzzznomatchesT228xyz" --format pretty` | **`Scope: project=test-alias (441837f6-…)`** + empty hint (T207 OK) |
| Preflight / memory list pretty | Scope already always-on (T214/T219/memory inventory) |

### 2.2 Root cause (frozen)

```text
// crates/ai-brains-cli/src/commands/recall.rs — pretty branch
if hits.is_empty() {
    // T207: get_project_by_id + format_scope_line → Session? → Embedding? → hint
} else {
    // Non-empty: Session + Embedding? + print_pretty_hits
    // Comment: "no required Scope (AC10 deferred)"
}

// crates/ai-brains-cli/src/commands/sync.rs — vault pretty
println!("--- AI-Brains Recall ---");
if hits.is_empty() {
    print_pretty_empty_sync(...); // has Scope
} else {
    print_pretty_hits(&hits);     // no Scope
}

// sync.rs:399-409 — pre-existing: missing/invalid AI_BRAINS_PROJECT_ID
// → "default-project" → ProjectId::from_str fail → ProjectId::new() random UUID
// Empty path already shows Scope: project=<random-uuid> on lookup miss; T228 elevates visibility on non-empty.
```

| Gap | Detail |
|-----|--------|
| Non-empty recall | Scope lookup + print only on empty path |
| Non-empty sync | Same asymmetry under `--- AI-Brains Recall ---` |
| Hermetic | `recall_nonempty__pretty__shows_hits_no_empty_hint` explicitly **does not** assert Scope (AC10 deferred) |
| Docs | CAPABILITIES: “Non-empty **recall** pretty Scope remains deferred (T228)” |
| Sync test sites | `sync_query_isolation.rs:86`, `sync_query_ranking.rs` (4), `smoke.rs:86` run non-empty pretty — **safe substring** today; need explicit Scope lock + verify green |
| Sync project fallback | Random UUID Scope under missing project (pre-existing) |

### 2.3 Touch map

| Site | Role |
|------|------|
| `ai-brains-cli/src/commands/recall.rs` | **F29:** extract `resolve_active_scope_line` (global short-circuit + `get_project_by_id` + `format_scope_line`); empty + non-empty pretty use it; print Scope **first** on non-empty |
| `print_pretty_empty_sync` | Use shared resolver (no dup lookup) |
| `ai-brains-cli/src/commands/sync.rs` | Non-empty vault: after `--- AI-Brains Recall ---`, print resolved Scope then hits; empty path via updated `print_pretty_empty_sync` |
| `tests/recall_empty_pretty_scope.rs` | Elevate AC10; **hard** AC2 global + AC6 quiet named hermetics; AC11 order |
| `tests/sync_query_isolation.rs` | **Hard AC8:** assert `Scope: global` on non-empty global pretty (`sync_query_pretty_global_flag_returns_cross_project_results`); verify empty-path isolation still green |
| `tests/sync_query_ranking.rs` | **Verify still green** (4 tests; substring hit order unaffected by Scope prefix) |
| `tests/smoke.rs` | **Verify still green:** `sync_query__no_bridge__skips_ledgerful_section` (~:86); `recall_pretty__shows_session_prefix` (~:459) substring safe |
| Units | Existing `format_scope_line__*`; unit for `resolve_active_scope_line` global skip + project path if pure enough |
| Docs | CAPABILITIES residual → always-on; **new** CHANGELOG T228 row (**do not edit** T207 historical row L82); soft OPERATIONS |
| Contracts | **None** |

### 2.4 Deps / research pins

| Pin | Evidence | Action |
|-----|----------|--------|
| `clap` workspace `4.5` → lock **4.6.1** | Cargo.toml / Cargo.lock | **No bump** — no new flags |
| `is-terminal` workspace `0.4` → lock **0.4.17** | crates.io 0.4.x | **No bump** — T101 defaults untouched |
| **Zero new crates** | F15 | — |
| [clig.dev](https://clig.dev/) | Human-first; human output may change; keep machine JSON stable; consistent subcommand chrome | Intentional Scope-first; JSON frozen |
| memory list | Always-on Scope empty+non-empty (`memory.rs:411`) | Precedent |
| sync default format | `sync.rs:397` always `"pretty"` (not TTY-aware) vs recall T101 | **Residual** — T228 does not fix; F12 still applies to pretty path |
| Non-empty Session always | `effective_session_id` always `Some` via generate; non-empty prints `Session:` always | AC11: lines[0]=Scope, lines[1]=Session on non-empty |

## 3. Frozen decisions (F1–F36)

| ID | Decision |
|----|----------|
| **F1 — Scope of track** | Pretty **display** Scope chrome on non-empty recall (+ sync vault section). No FTS/ranking/semantic change. No auto-global. No daemon/contracts. |
| **F2 — Always-on pretty Scope** | When `format_str == "pretty"`, **always** print `format_scope_line(...)` whether `hits` empty or not. |
| **F3 — Vocabulary SOOT** | Exact T207 strings: `Scope: global` \| `Scope: project=<label> (<uuid>)` \| `Scope: project=<uuid>` \| `Scope: project=(none)`. Prefer alias over name (existing). |
| **F4 — Print order (non-empty recall)** | **Strict:** (1) `Scope:` (2) `Session:` (always on non-empty per F5) (3) optional Embedding honesty (4) hit lines. **No** Scope after hits. |
| **F5 — Session rules unchanged** | Non-empty **always** prints `Session:` today (`effective_session_id` always `Some`). T207 omit-generated-Session remains **empty-only** — do not extend omit to non-empty. |
| **F6 — Empty path freeze** | Empty pretty order + hint + omit-generated-Session **unchanged** (T207 regression suite must stay green). |
| **F7 — JSON path** | Unchanged envelope. **No** JSON `scope` field. |
| **F8 — Format defaults** | T101 frozen for **recall**. Sync’s TTY-independent pretty default is **pre-existing residual** (F34) — out of DoD. Explicit `--format` wins where present. |
| **F9 — Quiet** | `--quiet` does **not** suppress Scope on empty **or** non-empty pretty. Bridge quiet remains T81. |
| **F10 — No auto scope widen** | Never promote project search to global on empty or non-empty. |
| **F11 — Lookup cost** | Single `get_project_by_id` when `!global && project_id.is_some()`. **Do not** `list_projects()`. |
| **F12 — sync query** | **Hard:** under `--- AI-Brains Recall ---`, non-empty pretty prints Scope before hits. **Ledger-first:** Scope appears only inside the vault block after ledger section (Note → ledger → blank → vault header → **Scope** → hits). Ledger chrome unchanged. |
| **F13 — Hit line format** | `format_pretty_hit_line` / badges / T218 / T224 **untouched**. |
| **F14 — Embedding lines** | After Scope + Session, before hits (relative placement unchanged vs Session). |
| **F15 — Zero new crates** | — |
| **F16 — Exit codes** | Unchanged. |
| **F17 — Capture independence** | Scope chrome must not open embedding backend or graph. |
| **F18 — High findings** | Shipping without Scope on non-empty; Scope only on TTY; JSON shape change; auto-global; `list_projects`; empty regression; ranking change; silent CHANGELOG omit; editing historical T207 CHANGELOG row. |
| **F19 — CHANGELOG honesty** | **New** T228 entry for intentional Scope-first non-empty pretty. **Do NOT modify** historical T207 CHANGELOG line (~L82 “non-empty deferred”). |
| **F20 — Docs** | CAPABILITIES: remove “deferred (T228)”; always-on pretty recall + sync vault Scope. Soft OPERATIONS. |
| **F21 — Tests** | See §5. Elevate AC10; hard AC2/AC6/AC8/AC11. |
| **F22 — Privacy** | Do not dump other projects’ names beyond active scope label. |
| **F23 — Review** | Primary required. Cross-model **soft** with focus (AI2 O6): no test asserts `lines[0] == "Session:"`; sync isolation/ranking/smoke still green; smoke session-prefix substring safe. |
| **F24 — Parallel** | Low conflict with T229–T231 if they avoid recall/sync pretty headers. |
| **F25 — Soft decline** | JSON `scope` · auto-global · color · pager · clap ValueEnum · T231 merge · omit Session on non-empty · blank after Scope · **sync random-UUID fallback fix as DoD** (F35 residual) · sync TTY default alignment as DoD. |
| **F26 — Blank line** | Match empty: **no** blank line between `Scope:` and `Session:` (consecutive `\n`, not `\n\n`). |
| **F27 — Implement order** | Red AC10/AC11 → green shared resolver + non-empty recall → green sync + AC8 → AC2/AC6 → regression sweep → docs → gate. |
| **F28 — Ledger** | On go: `ledgerful ledger start T228-nonempty-pretty-scope --category FEATURE`. |
| **F29 — Shared resolver (AI1 M1 / O1 elevated)** | **Hard:** extract `pub(crate) fn resolve_active_scope_line(conn, global, project_id) -> Result<String, …>`: if `global` → `"Scope: global"` **without** `get_project_by_id`; else lookup name/alias when `Some(pid)` then `format_scope_line`. Use for: empty recall, non-empty recall, `print_pretty_empty_sync`, non-empty sync. |
| **F30 — Global short-circuit (AI1 L1)** | Inside resolver: `global == true` skips DB lookup entirely. |
| **F31 — Sync affected tests (AI2 M1)** | Plan **must** list: `sync_query_isolation.rs` (global non-empty + empty isolation), `sync_query_ranking.rs` (4), `smoke.rs` sync no-bridge + recall session-prefix. AC8 locks `Scope: global` on isolation global test (or dedicated hermetic). |
| **F32 — Sync random-UUID residual (AI2 M2)** | Pre-existing `sync.rs:399-409`: missing/invalid `AI_BRAINS_PROJECT_ID` → `ProjectId::new()` random. T228 makes non-empty Scope show `project=<uuid>` lookup-miss. **DoD = document** in §7 risk + §11 residual + CHANGELOG honesty note if observed. **Soft O4:** fix to `None` → `project=(none)` only if implementer proves search/exit semantics stay safe and **no auto-widen**; otherwise leave for T231. **Not a silent DoD fix.** |
| **F33 — AC11 order pin (AI2 M3)** | Non-empty pretty: first chrome line **starts with** `Scope:`; second chrome line **starts with** `Session:` (always, F5); optional Embedding; then hits. Remove ambiguous “(or …)” fallback wording. |
| **F34 — Sync TTY default residual (AI2 L2)** | `sync.rs:397` defaults pretty regardless of TTY; recall is TTY-aware. Pre-existing; list in §11; **out of T228 DoD**. |
| **F35 — Helper vs inline (AI2 L3)** | F29 extract is **hard** (four call sites). Document helper signature in `review.md` on ship. |
| **F36 — AI fold-in** | §15 disposition applied 2026-08-11. |

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | Non-empty `recall <seed> --format pretty` stdout contains `Scope:` **and** hit content; exit **0**. |
| **AC2** | Non-empty pretty **global** → `Scope: global` before hits. **Hard hermetic:** `recall_nonempty__pretty_global__scope_global`. |
| **AC3** | Non-empty pretty **project** with alias → `Scope: project=<alias> (<uuid>)` (or name if no alias). |
| **AC4** | Non-empty pretty does **not** print empty “No results…” hint. |
| **AC5** | Empty pretty still prints Scope + hint; generated Session still omitted (T207 regression). |
| **AC6** | `--quiet` + non-empty pretty still shows Scope (+ hits). **Hard hermetic:** `recall_nonempty__pretty_quiet__keeps_scope`. |
| **AC7** | JSON non-empty path: no required `Scope` chrome; envelope keys unchanged (no new `scope` field). |
| **AC8** | **sync query** non-empty pretty vault section contains `Scope:` after `--- AI-Brains Recall ---`. **Hard:** assert `Scope: global` on `sync_query_pretty_global_flag_returns_cross_project_results` (or dedicated hermetic). |
| **AC9** | CAPABILITIES residual removed/updated; **new** CHANGELOG T228 entry; **T207 historical row untouched**. |
| **AC10** | Non-empty pretty **requires** Scope — hermetic asserts presence (replaces deferred comment). |
| **AC11** | **Non-empty order:** first non-empty line of chrome is `Scope:…`; next is `Session:…` (always on non-empty); no blank line between them (F26); hits after optional Embedding. Hermetic: `lines` filter non-empty → `[0].starts_with("Scope:")` && `[1].starts_with("Session:")`. |
| **AC12** | No ranking/score_kind/wire change; T218/T224 + **sync_query_ranking** + listed smoke still green. |
| **AC13** | Unit or hermetic: `global == true` → resolver returns `Scope: global` without requiring a project row (F30). |

## 5. Testing strategy

### 5.1 Red → green

1. **Red AC10/AC11:** extend/rename non-empty hermetic — assert Scope presence + order Scope before Session + no `No results`.
2. **Green:** F29 resolver + non-empty recall Scope-first.
3. **Red/green AC2:** `recall_nonempty__pretty_global__scope_global`.
4. **Red/green AC6:** `recall_nonempty__pretty_quiet__keeps_scope`.
5. **Green F12 + AC8:** Scope on non-empty sync; lock `Scope: global` on isolation global test.
6. **Regression sweep:** `recall_empty_pretty_scope` full suite; `sync_query_isolation` empty path; `sync_query_ranking` (4); smoke sync no-bridge + recall session-prefix; quiet Cozo if graph-on.

### 5.2 Naming (AGENTS)

- `recall_nonempty__pretty__prints_scope_before_hits` (AC1/AC4/AC10/AC11)
- `recall_nonempty__pretty_global__scope_global` (**hard AC2**)
- `recall_nonempty__pretty_quiet__keeps_scope` (**hard AC6**)
- Prefer extend `sync_query_pretty_global_flag_returns_cross_project_results` for **AC8** (`Scope: global`)

### 5.3 Hermetic rules

- `tempfile` vault; `hermetic_cmd`; pin unique seed.
- `--no-bridge` where Cozo noise matters.
- Assert specific strings and **line order** (AC11), not only `is_ok` / `contains` loosely.

## 6. Non-goals

- Changing default project scope resolution (T112) except optional soft sync fallback (F32).
- Adding Scope to JSON / NDJSON / daemon API.
- Unifying recall vs sync into one command (T231).
- Filling blank project labels under global (T230).
- Aligning sync TTY default with T101 (F34 residual).
- Unknown `--format` fail-usage surface-wide (T227 F34).
- Preflight blank-after-Scope style on recall.
- Editing historical T207 CHANGELOG text.

## 7. Risk & rollback

| Risk | Mitigation |
|------|------------|
| Scripts scrape first line as `Session:` | Human pretty not a contract; JSON machine SOOT; CHANGELOG Scope-first |
| Extra `get_project_by_id` latency | Single SELECT; global short-circuit (F30) |
| Existing sync/ranking/smoke tests | Enumerate + verify green (F31); AC8 explicit lock |
| Empty regression | T207 suite mandatory |
| **sync missing/invalid PROJECT_ID → random UUID Scope** (AI2 M2) | Pre-existing; T228 elevates visibility on non-empty. **Document residual (F32)**; soft fix to `None` only with proven no auto-widen |
| Ledger-first interleaving | Scope only inside vault block after ledger (F12) |
| Historical CHANGELOG rewrite | New T228 entry only (F19) |

Rollback: revert non-empty Scope print + resolver call sites; re-defer CAPABILITIES (undesirable).

## 8. Docs checklist

| Doc | Change |
|-----|--------|
| `Docs/CAPABILITIES.md` Scope row | Always-on pretty recall Scope; drop T228 deferred; sync vault pretty parity |
| `CHANGELOG.md` | **New** minor UX entry for T228 Scope-first non-empty; **leave T207 row as historical truth** |
| Soft: `Docs/OPERATIONS.md` | Only if samples show Session-only non-empty |
| Soft: skill text | Only if skill claims Session-first non-empty |

## 9. Manual verification (on go)

```powershell
ai-brains recall "DECISION" --limit 2 --format pretty
# expect: Scope: project=… \n Session: … \n hits  (no blank between Scope/Session)

ai-brains recall "DECISION" --limit 2 --global --format pretty
# expect: Scope: global …

ai-brains recall "zzzznomatchesT228xyz" --format pretty
# expect: Scope still + No results (empty unchanged)

ai-brains sync query "DECISION" --no-bridge --format pretty
# expect: --- AI-Brains Recall --- \n Scope: … \n hits
```

## 10. Implement order (on go)

1. Preflight: doctor + ledger status + `ledger start T228-nonempty-pretty-scope --category FEATURE` + `scan --impact`.
2. Red AC10/AC11 hermetic.
3. Green F29 resolver + non-empty recall (F2–F4, F26, F30).
4. Green sync non-empty Scope (F12) + AC8 lock + F31 verify-green list.
5. Hard AC2 + AC6 hermetics.
6. Empty + ranking + isolation + smoke regression.
7. Docs AC9 (new CHANGELOG only).
8. Full gate + primary review (+ soft cross-model O6 focus) + ledger commit + pin.

## 11. Residual after T228 (expected soft)

| Item | Owner |
|------|-------|
| JSON optional `scope` field | Future contract track |
| Omit generated Session on non-empty | Soft product polish |
| T230 global labels | Separate track |
| T231 unified search UX | Separate track (may absorb sync project fallback) |
| T227 F34 OutputFormat surface-wide | Separate residual |
| T229 nightly/router ops | Ops track |
| **sync.rs:399-409 random UUID on bad/missing project** (F32) | Soft O4 / T231 — document if unfixed |
| **sync.rs:397 TTY-independent pretty default** (F34) | Residual / T231 |
| Smoke-level non-empty recall Scope (AI2 L6) | Soft — hermetic AC1/AC10 sufficient |

## 12. Absorbed deferred index

| Source | Disposition |
|--------|-------------|
| deferred.md “Non-empty pretty Scope (T207 soft)” | **DoD this track** |
| T207 AC10 / M3 residual | **Elevate to hard AC10** |
| CAPABILITIES “remains deferred (T228)” | **Close in docs** |
| Series README T228 row | **Planning → Complete on ship** |
| T219 soft residual “T228 recall Scope” | **Absorb** |
| T224 soft residual T228 mention | **Absorb display only** |

**Not absorbed:** T207 soft L2 combined count+name; T230; T231; contract `scope`; sync random-UUID **fix** as mandatory DoD; sync TTY default alignment.

## 15. AI fold-in disposition (2026-08-11)

### AI1

| ID | Disposition | Pin |
|----|-------------|-----|
| **M1** extract `resolve_active_scope_line` | **Accept hard → F29/F35** | Four call sites; eliminates 3rd/4th dup |
| **M2** strict non-empty order | **Accept hard → F4/F33** | Already planned; reaffirm |
| **M3** zero blank Scope/Session | **Accept hard → F26** | Match empty `format_pretty_empty_state` |
| **M4** elevate AC10 hermetic | **Accept hard → AC10/AC1** | Already planned |
| **L1** short-circuit global lookup | **Accept hard → F30** | Inside resolver |
| **L2** CAPABILITIES + CHANGELOG | **Accept hard → F19/F20** | New CHANGELOG only |
| **O1** pub(crate) resolver for T231 | **Elevate hard → F29** | Same helper |

### AI2

| ID | Disposition | Pin |
|----|-------------|-----|
| **M1** enumerate sync/smoke affected tests + AC8 lock | **Accept hard → F31/AC8** | isolation:86, ranking×4, smoke:86; isolation empty path verify |
| **M2** sync random-UUID fallback elevated | **Accept risk doc hard → F32**; **fix soft residual** | Prefer document; O4 only if no auto-widen proven |
| **M3** AC11 rewrite + lines[0]/[1] hermetic | **Accept hard → F33/AC11** | Non-empty always Session |
| **L1** new CHANGELOG, don’t edit T207 | **Accept hard → F19/AC9** | Event-sourcing for docs |
| **L2** sync TTY default residual | **Accept soft → F34/§11** | Out of DoD |
| **L3** extract vs inline decision | **Accept hard extract → F29/F35** | Document in review.md |
| **L4/L5** governed/dogfood | **Out of scope** | No action |
| **L6** smoke non-empty Scope | **Soft** | Hermetic sufficient |
| **O1** shared header helper | **Fold into F29** | Hard |
| **O2** Scope: global on isolation global | **Accept hard → AC8** | Cheaper than new file |
| **O3** named AC2 hermetic | **Accept hard → AC2** | Already named; pin hard |
| **O4** fix sync fallback to None | **Soft residual F32** | Not DoD without proof |
| **O5** named AC6 hermetic | **Accept hard → AC6** | |
| **O6** cross-model order focus | **Soft → F23** | Cheap insurance |
| **ledger-first note** | **Accept hard → F12** | Scope inside vault block only |

**Verdict after fold-in:** Go-ready plan. No Highs. Mediums absorbed. Still plan-only until **go**.
