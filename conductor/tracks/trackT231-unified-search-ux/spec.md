# T231 — Unified search UX defaults

- **Track ID:** T231-UnifiedSearchUx
- **Phase:** Post-audit CLI quality series (T217–T232) — P2 polish after T230
- **Status:** 🚧 **Implementing**
- **Depends on:** T101 TTY pretty ✅; T207 empty pretty Scope/hint ✅; T211 ranking + ledger-first ✅; T217 FTS rescue ✅; T224 role strip ✅; T228 always-on Scope + `resolve_active_scope_line` ✅; T230 labels ✅
- **Blocks / feeds:** Operators know which command to run; closes dual mental model residual; absorbs T228 F32 hard; series CLI-quality close for search IA
- **Category:** UX / IA / FEATURE (light) / DOCS
- **Source:** Audit 2026-08-05 dual search mental model; series README T231; deferred.md Placeholder; T228 soft F32/F34; CAPABILITIES “Code + memory → recall or sync query” ambiguity
- **Deferred absorbed:** deferred.md “Dual recall vs sync query mental model” → **DoD**; series README T231; T228 **F32** sync random-UUID project fallback → **hard fix**; CAPABILITIES/WORKFLOWS search recipe gap → **docs DoD**; T228 **F34** sync always-pretty → **document intentional** (not flip default)
- **Not absorbed:** New top-level `search` noun as DoD (prefer A+C; soft residual O2); merge `recall` + `sync query` into one retrieval path; `sync query --semantic`; JSON `scope` contract field; T227 F34 surface-wide `OutputFormat`; auto-`--global`; clap 5 / ValueEnum; MSI; T229 ops; bridge multi-round rescue; control-plane FTS; locale stopwords; `is-terminal` → stdlib migrate
- **Research date:** 2026-08-11 (live dogfood + code truth + clig.dev + crates.io pins)
- **AI fold-in:** 2026-08-11 — AI1 **M1–M4 hard**, **L1–L2 hard**, **O1 hard**. AI2 **M1–M3 hard**; **L1/L3–L6 hard**; **L2/L7 no-op**; **L8 soft residual**; **O2–O6 hard**; **O1/O7 soft residual** (recall `text`→pretty arm); **O8** cross-model soft. Disposition **§15**.
- **Ledger:** plan-only — open TX on **go**

## 1. Objective

1. **One recommended human search path** and one agent path, documented and discoverable — without inventing a third retrieval engine.
2. **Keep two commands** (deliberate dual, not accidental dual):
   - **`recall`** — vault-first memory search (semantic / graph / JSON contracts; agent default).
   - **`sync query`** — vault + Ledgerful ledger pane (human unified search; lexical vault only).
3. **Docs + chrome (A+C):** CAPABILITIES “Start here: which search?”, WORKFLOWS recipe, clap help, empty pretty next-step pointing humans at `sync query` when they need ledger.
4. **Harden sync project resolve (F32):** missing or invalid `AI_BRAINS_PROJECT_ID` → `project_id = None` → Scope `project=(none)` — **never** `ProjectId::new()` random UUID. No silent auto-`--global`.
5. **Capture independence:** Display / resolve / docs only. No event appends, no model/embed requirement, no ranking algorithm change, no contracts DTO growth.
6. **JSON frozen:** `recall` JSON shape and exit codes unchanged. `sync query` still has no machine JSON object (pretty / text / ndjson only).

## 2. Live baseline (2026-08-11)

### 2.1 Operator dogfood (this machine)

| Command | Observed |
|---------|----------|
| `recall "…" --format pretty` | Scope + hits or empty hint; TTY default pretty already works |
| `recall "…"` piped (no `--format`) | **JSON** (non-TTY default) — agent-friendly |
| `sync query "…"` no format (TTY **or** piped) | Always **pretty** chrome (`--- AI-Brains Recall ---` + Scope + hits) |
| `sync query --format text` | Same **pretty** path (only `ndjson` is special-cased in `sync.rs`) |
| `recall --format text` | Falls to `_` arm → **JSON** (undocumented; help lists only `json`\|`pretty`) — **F8 asymmetry** |
| `sync query --format ndjson` | NDJSON bridge Insight; on None project currently **`ProjectId::new()`** random (F21 gap) |
| `sync query` + `--no-project-context` + unset `AI_BRAINS_PROJECT_ID` | **Scope: project=\<random uuid\>**; empty “Scoped to this project” + 0 memories — **F32 live** |
| Invalid `AI_BRAINS_PROJECT_ID` | **recall** → clap parse **exit 2**; **sync query** (today) → random UUID; after F10 → vault-wide `None` exit 0 — **F36 asymmetry** |
| Help | No top-level `search`; Daily lists `recall`; `sync` under Harness; CAPABILITIES §15 says “recall or sync query” without decision rule |

### 2.2 Root cause / product gaps (frozen)

```text
// recall — already good human/agent split
resolve_format(explicit, is_tty) → pretty on TTY, json otherwise
project_id: Option<ProjectId> via clap env — invalid fails parse; missing = None

// sync query — human-first, but project resolve is wrong
let fmt = format.unwrap_or_else(|| "pretty".to_string());  // always pretty (even non-TTY)
let project_id_str = env::var("AI_BRAINS_PROJECT_ID")
    .unwrap_or_else(|_| "default-project".to_string());
Some(ProjectId::from_str(&project_id_str)
    .unwrap_or_else(|_| ProjectId::new()))  // ← RANDOM on bad/missing
```

| Gap | Detail |
|-----|--------|
| Dual mental model | CAPABILITIES / help do not say *when* to use which command |
| F32 random project | Bad/missing env → silent random UUID → empty wrong scope (not `project=(none)`) |
| F34 always-pretty | Intentional human default; machines use `recall` JSON or explicit `ndjson` — **document**, do not force TTY gate that breaks human pipes casually |
| `text` format asymmetry (AI2 M1) | **sync** `text` ≡ pretty (documented); **recall** `text` → JSON silent fallthrough (help omits `text`) |
| NDJSON second random (AI1/AI2 M2) | `sync.rs:419` `unwrap_or_else(ProjectId::new)` + `Some(project_id)` to recall even after outer F10 |
| Invalid-env asymmetry (AI2 M3) | clap hard-fail on recall vs manual-read on sync — document; do **not** converge this track |
| Empty pretty next-step | Mentions `--semantic` / `--global`, not ledger `sync query`; `build_recall_hint` shared by **recall + print_pretty_empty_sync** → must **gate** self-mention (AI2 L3/L4) |
| No `search` noun | Placeholder B — soft residual only |

### 2.3 Touch map

| Site | Role |
|------|------|
| `ai-brains-cli/src/commands/sync.rs` | **F10/F29** pure `resolve_sync_project_id`; call-site `env::var(...).ok()` (no `"default-project"`); **F21** ndjson: pass `Option` to recall + `BridgeRecord.project_id = map.unwrap_or_default()`; units AC1–AC4 |
| `ai-brains-cli/src/commands/recall.rs` | **F12/F37** empty next-step with **`include_sync_query_hint: bool`** (true on recall pretty empty; **false** in `print_pretty_empty_sync`) |
| `ai-brains-cli/src/main.rs` | clap help cross-refs F14 / AC10 |
| `ai-brains-cli/tests/sync_query_*.rs` | Hermetics AC5–AC7 (+ AC13 global); use `--no-project-context` + tempdir (clears env when no `.env`) or explicit invalid env |
| `Docs/CAPABILITIES.md` | **§15** decision table replaces ambiguous “Code + memory” row; F8/F36 honesty; F32 |
| `Docs/WORKFLOWS.md` | “Find something” recipe |
| `Docs/CHANGELOG.md` | **New** T231 row only |
| Contracts / bridge | **None** — `BridgeRecord.project_id: String` required → empty string only |

### 2.4 Deps / research pins

| Pin | Evidence | Action |
|-----|----------|--------|
| `clap` workspace `4.5` → lock **4.6.1** (crates.io **4.6.6**) | Cargo.lock / `cargo search` | **No bump** — help text only unless soft alias lands |
| `is-terminal` lock **0.4.17** (crates.io **0.4.17**) | Cargo.lock | **No bump** |
| `chrono` lock **0.4.44** | Cargo.lock | **No bump** |
| `serde` lock **1.0.228** (crates.io **1.0.229**) | Cargo.lock | **No bump** |
| **Zero new crates** | F15 | — |
| [clig.dev](https://clig.dev/) Output | Humans first via TTY; JSON when `--json` / machine; suggest next commands | A+C + empty next-step |
| [clig.dev](https://clig.dev/) Ease of discovery | Help, examples, “what to run next” | CAPABILITIES + WORKFLOWS + empty hint |
| clap `visible_alias` | docs.rs clap 4 | Soft O2 only for top-level `search` → not DoD |
| OpenStatus 2026 “CLI for humans and agents” | TTY vs structured paths | Keep recall JSON; sync human-first |
| T228 F29 | `resolve_active_scope_line` already shared | Reuse for None project |

## 3. Product decision (locked for plan)

| Option | Placeholder | Disposition |
|--------|-------------|-------------|
| **A** | `recall` TTY pretty + footer “also: sync query for ledger” | **Accept hard** — TTY pretty **already true**; add empty pretty next-step + help/docs |
| **B** | New `ai-brains search` alias | **Soft residual O2** — not DoD; prefer A+C first |
| **C** | CAPABILITIES/WORKFLOWS “Start here” recipe | **Accept hard** |

**Not chosen:** Merge engines; make `sync query` call `recall --semantic`; flip sync default to JSON.

### Decision table (ship in CAPABILITIES **§15** — replace ambiguous “Code + memory” row)

| Intent | Command |
|--------|---------|
| Human, vault only, TTY | `recall "…" --format pretty` (or bare `recall` on TTY) |
| Agent / pipe / scripts | `recall "…"` (JSON) or `--format json` |
| Human, vault **+ ledger** / plan vs shipped | `sync query "…" --format pretty` |
| Embeddings / hybrid | `recall "…" --semantic` (not sync query) |
| Machine stream of vault hits | `sync query "…" --format ndjson` **or** `recall --format json` |
| Invalid `AI_BRAINS_PROJECT_ID` | **`recall`** → clap **exit 2**; **`sync query`** → vault-wide `Scope: project=(none)` exit **0** (F36 — pre-existing clap vs manual-read; not converged) |
| `text` format | **`sync query --format text`** ≡ pretty; **`recall --format text`** → JSON (undocumented fallthrough — F8) |

## 4. Frozen decisions (F1–F40)

| ID | Decision |
|----|----------|
| **F1 — Scope** | UX/IA + sync project resolve. No FTS/ranking/semantic changes. No event writes. |
| **F2 — Dual keep** | Keep `recall` and `sync query` as two commands with distinct jobs (vault vs vault+ledger). |
| **F3 — Human default** | Recommended human unified path remains **`sync query`** (pretty + ledger pane). Recommended agent path remains **`recall`** (JSON non-TTY). |
| **F4 — No new noun DoD** | No top-level `search` required to close track (soft residual). |
| **F5 — No engine merge** | Do not reimplement ledger pane inside `recall`; do not add `--semantic` to `sync query` this track. |
| **F6 — TTY pretty recall** | Already correct — **do not regress** `resolve_format` (pretty TTY / json non-TTY). |
| **F7 — Sync format default** | Keep default **pretty** always (F34 intentional). Document that agents use `recall` for JSON; `ndjson` stays explicit on sync. |
| **F8 — text honesty (AI2 M1)** | **Asymmetry is real:** (1) `sync query --format text` ≡ pretty path (help lists text). (2) `recall --format text` → **JSON** via `_` arm (help lists only `json`\|`pretty`). **DoD = document** in CAPABILITIES decision table + CHANGELOG. **Not DoD:** add `"text"` arm on recall → pretty (soft residual O-text). No silent invent of a third renderer. |
| **F9 — Capture independence** | Display/docs/resolve only. |
| **F10 — Project resolve (F32 hard)** | Non-global: trim env; empty/missing/invalid UUID → **`None`**; valid → `Some`. **Never** `"default-project"`, **never** `ProjectId::new()`. Global → `None`. |
| **F11 — Scope honesty** | With `project_id=None` and not global: `resolve_active_scope_line` → `Scope: project=(none)`. Retrieval passes `project_id: None` (vault-wide — same as recall without project; **document**, not auto-`--global`). |
| **F12 — Empty pretty next-step** | Empty **recall** pretty only: append one ledger next-step line (F13). **Do not** spam non-empty hit lists. |
| **F13 — Hint content** | Copy pin: `For vault + Ledgerful ledger in one view: ai-brains sync query "<query>" --format pretty` (literal query or generic `"…"`). No emoji. |
| **F14 — Help strings** | `recall --help` and `sync query --help` each state the other command’s job in one line. |
| **F15 — Zero dep bumps** | No clap/is-terminal/serde bumps. |
| **F16 — Zero new crates** | — |
| **F17 — Exit codes** | Unchanged for happy/empty paths. Invalid env on **recall** stays clap exit **2** (not this track). |
| **F18 — Docs placement** | CAPABILITIES **§15** decision table (replace line ~508 “Code + memory \| recall or sync query”); WORKFLOWS “Find something”; **new** CHANGELOG only. |
| **F19 — Contracts** | No DTO growth; no JSON `scope`; `BridgeRecord.project_id` stays required `String`. |
| **F20 — Parallel-friendly** | Touches sync resolve + recall empty hint + docs; low conflict with T229 if ops-only. |
| **F21 — ndjson exact pin (AI1/AI2 M2)** | When outer `project_id` is `None`: (1) **pass `None`** to `recall` / `RecallOptions` (vault-wide, match pretty path); (2) `BridgeRecord.project_id = project_id.map(\|p\| p.to_string()).unwrap_or_default()` → **`""`**; (3) **delete** `project_id.unwrap_or_else(ProjectId::new)`. Never invent UUID. |
| **F22 — Soft residual text arm** | Optional `recall` match arm `"text"` → pretty (AI2 O1) — **not** DoD. |
| **F23 — Soft residual non-empty footer** | Optional trailing line on non-empty recall pretty — residual if noisy. |
| **F24 — Soft residual search alias** | Top-level `search` → sync query — residual. |
| **F25 — Out of scope** | T229, T233, clap 5, CE wipe, daemon HTTP search, semantic on sync, auto-global, clap-env converge recall↔sync. |
| **F26 — Test naming** | `function_or_feature__condition__expected_result`; hermetic temp vault. |
| **F27 — unwrap ban** | No new `unwrap`/`expect` in production. |
| **F28 — PowerShell** | `;` separators only in gate scripts. |
| **F29 — SOOT project resolve (AI1 M1)** | `pub(crate) fn resolve_sync_project_id(global: bool, env_val: Option<&str>) -> Option<ProjectId>`: if global → None; else trim; empty → None; `ProjectId::from_str(raw).ok()`. **Call-site:** `resolve_sync_project_id(global, std::env::var("AI_BRAINS_PROJECT_ID").ok().as_deref())` — **no** `"default-project"` literal. |
| **F30 — Isolation / hermetics (AI2 L1)** | AC5/AC6: `tempdir` + `--no-project-context` clears env when no project `.env` (reproduces F32). Explicit `.env("AI_BRAINS_PROJECT_ID", …)` on Command **survives** the clear — use for AC6 invalid / AC7 valid. |
| **F31 — Live dogfood** | Missing-project Scope → `project=(none)` not random; CAPABILITIES §15 table present. |
| **F32 — Absorb T228 F32** | **Hard DoD**. |
| **F33 — Absorb T228 F34** | **Document intentional**; do not flip sync to non-TTY JSON. |
| **F34 — Ranking frozen** | No change to `rerank_hits`, ledger-first, limit 5. |
| **F35 — Review** | Primary required; cross-model **soft** (UX light — AI2 O8). |
| **F36 — Invalid-env asymmetry (AI2 M3)** | **Document hard** in CAPABILITIES decision table + §6 Risks: recall invalid env → clap exit 2; sync invalid → `None` vault-wide exit 0. **Do not** add clap `env=` to sync or remove clap env from recall this track. |
| **F37 — No self-mention (AI2 L3/L4)** | `build_recall_hint` / core gains a flag (e.g. `include_sync_query_hint: bool`). Recall empty pretty: **true**. `print_pretty_empty_sync`: **false** (sync empty must not suggest “run sync query”). |
| **F38 — Whitespace env** | Env value that is only whitespace after trim → **None** (same as missing). |
| **F39 — AC global hermetic** | Hermetic `--global` → `Scope: global` (AI1 M4). |
| **F40 — F12 placement** | Ledger next-step is **additive** to existing empty lexical hint (`--semantic` / `--global`); prefer append after core hint, not replace. |

## 5. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Pure unit: `resolve_sync_project_id(false, None) == None` |
| **AC2** | Pure unit: invalid string → None; call twice → same None (no random) |
| **AC3** | Pure unit: valid UUID string → Some(that id) |
| **AC4** | Pure unit: `global=true` → None regardless of env string |
| **AC4b** | Pure unit: whitespace-only env → None (F38) |
| **AC5** | Hermetic: missing project env + `--no-project-context` → Scope `project=(none)`; stdout has **no** random UUID that is not a known fixture id |
| **AC6** | Hermetic: invalid `AI_BRAINS_PROJECT_ID` → Scope `project=(none)` |
| **AC7** | Hermetic: valid project with pins → still scoped hits (regression) |
| **AC8** | Recall empty pretty includes F13 ledger next-step substring |
| **AC8b** | Sync empty pretty does **not** include F13 self-mention (F37) |
| **AC9** | CAPABILITIES §15 table + WORKFLOWS recipe + CHANGELOG T231; F8/F36 rows present |
| **AC10** | Help: recall + sync query peer cross-ref |
| **AC11** | `resolve_format` non-TTY still JSON (regression) |
| **AC12** | Full gate green; no contract crate change |
| **AC13** | Hermetic: `--global` → `Scope: global` (F39) |
| **AC14** | Unit or hermetic: ndjson path with None project does not call `ProjectId::new` — BridgeRecord project field empty string and/or vault-wide (F21) |

## 6. Risks

| Risk | Mitigation |
|------|------------|
| `project_id=None` changes retrieval from “random empty project” to vault-wide | **Correct** honesty; document CAPABILITIES; same as recall without project |
| Empty next-step noise / circular hint | Empty-state only; **F37** suppress on sync empty |
| Scripts depended on random-project empty results | Extremely unlikely; was a bug |
| Invalid-env asymmetry confuses operators | **F36** document in decision table + risks; do not half-converge |
| Scope creep into `search` noun / text arm | Soft residual only |
| F34 flip breaks human pipes | Do not flip; document |
| NDJSON still invents UUID after F10 | **F21** exact pin + AC14 |

## 7. Non-goals

- Single command that does semantic + ledger + graph in one binary path
- Auto-widen to `--global` on empty
- JSON `scope` on recall response
- Converging invalid-env handling (clap env on sync / manual parse on recall)
- clap 5 / ValueEnum migration
- MSI / packaging
- Nightly/router (T229)
- Changing BM25/RRF/rerank
- `is-terminal` → `std::io::IsTerminal` migrate

## 8. Verification plan

1. Preflight: doctor + ledger status + `ledger start T231-unified-search-ux --category FEATURE` + `scan --impact`
2. Red: AC1–AC4/AC4b pure units + AC5/AC6 hermetic failing on random UUID
3. Green: F10/F29 helper + F21 ndjson + wire call-site
4. Red/Green: AC8 + AC8b empty pretty (gated F37)
5. Help + docs AC9/AC10 (CAPABILITIES §15)
6. AC13 global + AC14 ndjson
7. Live dogfood F31
8. Full gate + primary review (cross-model soft)
9. Ledger commit + pin + deferred close + series README

## 9. Absorbed deferred index

| Source | Disposition |
|--------|-------------|
| deferred.md Dual recall vs sync query | **DoD** |
| Series README T231 | **Planning → Complete on ship** |
| T228 F32 random UUID | **Hard F10/F32** |
| T228 F34 always-pretty | **Document F7/F33** (not flip) |
| T217 soft “T231 unified UX” | **Absorb IA only** |
| T224 soft T231 mention | **Absorb docs** |
| Placeholder options A/B/C | **A+C hard; B soft** |
| AI1 M1–M4, L1–L2, O1 | **Hard** (reaffirm plan) |
| AI2 M1–M3, L1/L3–L6, O2–O6 | **Hard** fold-in |

**Not absorbed:** T227 OutputFormat surface-wide; T229; T233; new search noun as DoD; recall text→pretty arm as DoD; invalid-env clap converge.

## 10. Residual after T231 (expected soft)

| Item | Owner |
|------|-------|
| Top-level `search` alias | Soft residual |
| `recall --format text` → pretty arm (AI2 O1) | Soft F22 |
| Distinct rich `text` renderer | Soft |
| Non-empty pretty ledger footer | Soft F23 |
| JSON `scope` field | Future contract |
| Invalid-env clap/manual converge | Residual F36 (documented) |
| `sync query --semantic` | Future if demanded |
| `is-terminal` → stdlib | Residual (AI2 L8) |
| T227 F34 OutputFormat | Separate residual |
| T229 nightly/router ops | Ops track |

## 11. AI fold-in disposition (2026-08-11)

### AI1

| ID | Disposition | Pin |
|----|-------------|-----|
| **M1** pure `resolve_sync_project_id` | **Accept hard → F29/F10** | Spec body; trim + empty → None |
| **M2** NDJSON no `ProjectId::new` | **Accept hard → F21** | Exact `""` + pass None to recall |
| **M3** empty pretty next-step | **Accept hard → F12/F13/F40** | + **F37** gate (from AI2) |
| **M4** hermetics AC5–AC7 + global | **Accept hard → AC5–AC7/AC13** | F30 hermetic notes |
| **L1** CAPABILITIES/WORKFLOWS | **Accept hard → F18** | §15 placement (AI2 O6) |
| **L2** clap help cross-ref | **Accept hard → F14/AC10** | — |
| **O1** pure units AC1–AC4 | **Accept hard → AC1–AC4/AC4b** | — |

### AI2

| ID | Disposition | Pin |
|----|-------------|-----|
| **M1** text asymmetry recall vs sync | **Accept hard → F8** | Document DoD; text arm soft residual |
| **M2** F21 exact ndjson pin | **Accept hard → F21/AC14** | BridgeRecord required String → `""` |
| **M3** invalid-env asymmetry | **Accept hard → F36** | Decision table + risks; no converge |
| **L1** hermetic `--no-project-context` | **Accept hard → F30** | Plan Phase 1 note |
| **L2** should_warn includes sync | **No-op** | tempdir safe |
| **L3/L4** self-mention circular | **Accept hard → F37/AC8b** | `include_sync_query_hint` |
| **L5** call-site `.ok()` | **Accept hard → F29** | No `"default-project"` |
| **L6** CAPABILITIES §15 | **Accept hard → F18** | Replace ~508 row |
| **L7** CHANGELOG | **No-op** | Already F18 |
| **L8** is-terminal migrate | **Soft residual** | §10 |
| **O1** text arm recall | **Soft residual F22** | Not DoD |
| **O2** F21 one-liner | **Hard** (= M2) | — |
| **O3** invalid-env row | **Hard** (= M3/F36) | — |
| **O4** gate is_sync | **Hard** (= F37) | — |
| **O5** call-site pin | **Hard** (= F29) | — |
| **O6** §15 table | **Hard** (= F18) | — |
| **O7** hermetic text format | **Soft** | Only if O1 lands |
| **O8** cross-model soft | **Accept → F35** | No mandatory CX |

### Declined / out of scope

| Item | Why |
|------|-----|
| Converge invalid-env via clap on sync | Scope; F36 document only |
| Force `recall --format text` → pretty as DoD | Format matrix expansion; document F8 instead |
| Top-level `search` as DoD | Prefer A+C |
| Flip sync always-pretty to TTY-gated | No JSON format; F33 document |

## 12. Implementer SOOT snippets (pins)

```rust
// F29
pub(crate) fn resolve_sync_project_id(global: bool, env_val: Option<&str>) -> Option<ProjectId> {
    if global {
        return None;
    }
    let raw = env_val?.trim();
    if raw.is_empty() {
        return None;
    }
    ProjectId::from_str(raw).ok()
}

// Call-site (no "default-project")
let project_id = resolve_sync_project_id(
    global,
    std::env::var("AI_BRAINS_PROJECT_ID").ok().as_deref(),
);

// F21 ndjson — remove unwrap_or_else(ProjectId::new)
// recall options: project_id: project_id (Option)
// BridgeRecord.project_id: project_id.map(|p| p.to_string()).unwrap_or_default()
```
