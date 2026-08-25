# T299 — Empty `forget --list-forgotten` must point at live pins

- **Track ID:** T299-ForgetListUseful
- **Status:** **Planned** (Pending until **go**; not Placeholder)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `forget --list-forgotten` **6/8** `No forgotten memories.`; T274–T284 declined honest empty (E=8) — **reopened** U&lt;8. Placeholder minted with T285–T300 (`76c4db9`). T287 F7 parked empty next here. T298 F24 pointed here.
- **Depends on:** T216 ✅ bounded list + empty `No forgotten memories.` + `--summary` dual COUNT; T198 ✅ empty success exit 0; T214 ✅ `count_pinned_memories` (not used — summary `count_memories` is SoT); T287 ✅ forgotten recency freeze / empty next parked
- **Blocks / feeds:** Operators who run empty `forget --list-forgotten` learn the vault still has pins and that inventory is `memory list`. Graph sparse **T300**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “forget-list empty U=6”; T216 F14 empty const **kept**; T216 F36 “skip next on empty” **partial lift** (stdout `next:` on forgotten-empty only; stderr restore still skipped); T287 F7 “T299 owns empty next-step”
- **Not absorbed (DoD):** Auto-forget / CE wipe; `--summary` on `forget`; JSON new keys / `next_step`; forgotten human mix (T287 F7); `--offset`; tag histogram; clap 5 / rusqlite 0.40; T300; T240 F2
- **Research date:** 2026-08-25 (plan dogfood HEAD `5323034` T298 `#214`. Product `src/` = T298. PATH **0.1.2** 2026-08-22 19:41 **has T216 empty one-liner**, not this remediator. Live vault **Forgotten: 0** / **Pinned: 4152** — do not auto-forget.)
- **Ledger:** planning DOCS TX `4516432b-edbf-49b4-a11a-2e682db985c0`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** rewrite `.env` (T240 F2). Do **not** `forget` live pins (`--match` / `--memory-id -f`) on the operator vault. Do **not** `forget --restore` live. Do **not** grow hotspot `project.rs` / `sync.rs` / `governed_common.rs` / `context.rs` / `forget.rs`. Grow `memory.rs` only (shared inventory emit). Do **not** print or commit `AI_BRAINS_KEY`. Do **not** live `retention apply --confirm`, `graph rebuild`, leftover `rebind-path --write --yes`, or `safety sync` without `--dry-run`.

---

## 1. Objective

1. **Empty forgotten list is useful.** Today it is T216 honest empty: Scope + `status=forgotten` + `No forgotten memories.` Exit **0**. Agents learn nothing: not that the vault has thousands of pins, not that inventory is `memory list`. Print **`Pinned: N`** (same COUNT as `--summary`) and a **copy-paste** `next: ai-brains memory list` without requiring a forget mutation.
2. **Shared backend stays shared.** `forget --list-forgotten` ≡ `memory list --status forgotten` via `run_inventory`. Both surfaces get the remediator. `forget.rs` stays a thin wrapper (hotspot **#5** — do **not** grow).
3. **Keep T216 / T287 / T198 contracts.** Empty const `No forgotten memories.` byte-identical. Non-empty forgotten recency-only (no mix). Default limit **50**. JSON list keys frozen. `--summary` already has `Pinned:` / `Forgotten:` — do not steal. Capture independence: SQL COUNT + string emit.

This unblocks daily ops honesty for the Windows-first vault: a project with **zero** forgotten rows is a **complete** list when it names live pins and the inventory command.

---

## 2. Live baseline (re-scan 2026-08-25)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `5323034` T298 squash `#214`. Tree **CLEAN**. `origin/main` = HEAD (`left-right` `0 0`). Branch `main`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. **Has T216 empty `No forgotten memories.`** **Does not have T298 this-machine** (PATH-behind). **Does not have T299 remediator.** **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4149** at session start (volatile; later **4152**). In-context **0/0/0**. Word **1222**. Capture independence holds. |
| PATH + source `forget --list-forgotten --limit 5` | `Scope: project=C:\dev\ai-brains (3581317d-…)`. `status=forgotten  limit=5`. `No forgotten memories.` Exit **0**. **No** `Pinned:`. **No** `next:`. `cargo run -p ai-brains-cli` **identical** (hole is in **source**, not only PATH). |
| PATH `forget --list-forgotten --limit 5 --format json` | Keys **exactly** `api_version`, `scope`, `project_id`, `status`, `items`, `returned`, `more_available`, `limit`, `total`. `items: []`. `total: 0`. `status: "forgotten"`. **No** `next_step` / `pinned`. |
| PATH `memory list --summary` | `Pinned: 4152`. `Forgotten: 0`. Same Scope. **This is the inventory number T299 must print.** Volatile — hermetic equality is SoT; Manual matches the `--summary` run in the same session. |
| Last GitHub PR | [#214](https://github.com/Ryan-AI-Studios/AI-Brains/pull/214) T298 (merged 2026-08-25T11:02:36Z). `gh pr view --comments`, `/reviews`, `/comments`, `issues/214/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, `#58` tower-http, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / hotspots | Hotspot **#1** `project.rs` (**3.879**) — **do not touch.** `sync.rs` #2. `governed_common.rs` #3. `context.rs` #4. **`forget.rs` #5 (3.040)** — **do not grow.** `memory.rs` **not** top-10. |
| Ledger | **0 pending / 0 drift** at scan (before this DOCS TX). |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Empty `forget --list-forgotten` U=6 | T216 made the list honest and bounded. Agents still cannot tell “no forgotten rows” from “empty vault.” Populate **without** auto-forget. **DoD.** |
| T216 F36 skip next on empty | stderr restore next is for **nonempty** tables. Empty forgotten needs a **stdout** remediator (T251/T297/T298 `next:` last). Partial lift, not a dump of the restore sentence. |
| JSON `next_step` | T216 F10 / T287 F10 freeze the CLI-local shape. Scripts already have `memory list --summary --format json` for `pinned`. Default format is **human** (Family B). **Decline JSON growth.** |
| `--summary` on `forget` | T216 F28. Summary already exists on `memory list`. **Decline.** |
| Auto-forget / live `--match -f` | Would mutate the operator vault. Hermetic pin+empty is enough. **Decline as DoD.** |
| Forgotten human mix | T287 F7 recency-only. Empty next is this track; mix is not. **Affirm freeze.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Shared inventory | `memory.rs` `run_inventory` **`:176–286`** | Forget list **and** `memory list` both land here. JSON vs human split **`:264–285`**. |
| Empty human | `emit_list_human` **`:470–475`** | `rows.is_empty()` → `No forgotten memories.` then **`return Ok(())`** — **before** F36 stderr. **Insert T299 lines here** (Forgotten arm only). |
| F36 stderr next | **`:523–526`** | Nonempty human only: `Use ai-brains forget --memory-id … or … --restore …`. **Stay skipped on empty.** |
| Empty const | **`:473`** | `No forgotten memories.` T216 F14 / T198. **Do not change the string.** |
| JSON DTO | `MemoryListJson` **`:114–124`** | Nine fields. No `next_step`. `emit_list_json` **`:411–445`**. **Do not add fields.** |
| JSON schema test | `tests/memory_list_inventory.rs` `memory_list__format_json__schema_keys` **`:369`** | Asserts required keys **present**, not exact key-set. Still **do not** add keys (F10 freeze). |
| Empty human test | `memory_list__empty_filter__non_blank_exit_0` **`:247–266`** | `contains("No forgotten memories.")` — **stays green** if we add lines. |
| Share-backend test | `forget_list_forgotten__matches_memory_list_status_forgotten` **`:274`** | Nonempty. Stay green. |
| Forget wrapper | `forget.rs` **`:47–62`** | `run_inventory` with `status: "forgotten"`, `summary: false`. **Do not grow this file.** |
| Forget clap | `main.rs` Forget **`:1594–1633`** | `--list-forgotten` + `--global` / `--limit` / `--format` human\|json / `--tag`. No `--summary`. |
| Forget after_help | `main.rs` **`:1597`** | Already names `memory list`. Additive one sentence. |
| Memory list after_help | `main.rs` **`:2985`** | Dual-truth pinned mix. Additive empty-forgotten sentence. |
| Summary COUNT | `run_summary` **`:308–319`** | `count_memories` status Pinned **and** Forgotten, same `project_id` / `tag`. **Reuse this COUNT for `Pinned: N`.** |
| `count_pinned_memories` | `query_store.rs` **`:699–715`** | `mp.project_id = ?` only (no session join). T288 OpenCode m1: can differ from summary. **Do not use.** |
| `count_memories` | `query_store.rs` **`:295–331`** | `memory_list_from_where` session-join (`sp.project_id OR mp.project_id`) + two-stage `--tag`. **SoT.** |
| Limit | `clamp_list_limit` | Default **50**, max **200**. Placeholder “`--limit 5`” is DoD **flag**, not a default change. |
| Hotspot | `forget.rs` #5 | Thin wrapper stays. Helpers in `memory.rs` (**722** lines — keep helper in same file). |

### 2.4 Dependency / standards research (2026-08-25) — snapshot, re-verify at execute

| Pin | Workspace / lock | crates.io / docs (today) | Action |
|-----|------------------|--------------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | **4.6.6** (GitHub latest 2026-08-06). **clap 5 not released.** | **No bump.** No new flags. `--format` stays `human\|json`. |
| `rusqlite` | workspace **0.39.0** | **0.40.2** (Dependabot `#61` open) | **No bump.** COUNT reuses existing method. |
| `serde_json` | workspace **1.0** | current 1.0.x | **No bump.** JSON keys frozen. |
| `rstest` | cli dev-dep **0.25** | already in crate | Reuse for remediator `#[case]`. |
| rustc / edition | **1.95.0** (prior track) / **2024** | — | Unchanged. |
| workspace version | **0.1.2** | — | **No bump.** |
| New crates | — | — | **Zero.** No `serial_test`. |

### 2.5 Online best-practice / implementation research

| Topic | Finding | Use in T299 |
|-------|---------|-------------|
| **[CLIG — Saying (just) enough](https://clig.dev/)** (current) | Too little = user wonders what is going on; too much = dump | One `Pinned: N` line + one `next:` line after the existing empty const. Do **not** dump the summary table or pin previews. |
| **[CLIG — Ease of discovery](https://clig.dev/)** | Suggest what to run next | Copy-paste `next: ai-brains memory list` (add `--global` iff the list was global). |
| **[CLIG — Human-first / future-proof](https://clig.dev/)** | Human output may evolve; scripts pin JSON | Human-only additive. JSON nine keys frozen. Scripts already have `--summary --format json`. |
| **[CLIG — stdout vs stderr](https://clig.dev/)** | stdout is the command result | Empty remediator on **stdout** (agents capture stdout). T216 F36 restore hint stays **stderr** and **nonempty-only**. |
| T216 F10 / T287 F10 | CLI-local JSON shape freeze; PROTOCOL-COMPAT N/A | **No** `next_step` / `pinned` on list JSON. |
| T290 analog | Granted-empty `Pinned: N` via COUNT fail-open | Same fail-open: COUNT `Err` → still print `next:`, omit `Pinned:`. **Different COUNT** (`count_memories` not `count_pinned_memories`) so Manual matches `--summary`. |
| T251/T297/T298 analog | `next:` is the **last** non-empty stdout line | F5. |
| N/A | SQLCipher / schtasks / contracts DTO / HTTP probes / clap 5 — this track does not touch them. | — |

**Could not verify:** exact live `count_memories(Pinned, 3581317d)` without vault SQL (do not print `AI_BRAINS_KEY`). Preflight Pinned **4149** vs summary **4152** is volatile (session pins during this planning pass). Hermetic equality to `--summary` is SoT. Manual AC matches the `--summary` invocation in the same session, not a frozen integer.

**ledgerful / ai-brains:** `preflight --summary` pinned **4149**; dogfood empty forgotten three-line human + JSON nine keys; `memory list --summary` **Pinned: 4152** / **Forgotten: 0**. Lexical/semantic recall of “forget list-forgotten empty” returned T287 plan-audit chrome (PATH-behind ranking) — **not** a contradicting pin; live src `emit_list_human` `:470–475` is SoT. `sync query` same chrome. `ledgerful ledger status --compact` 0 pending / 0 drift at scan; `scan --impact` CLEAN at `5323034`; `hotspots` `forget.rs` #5 — do not grow; `search emit_list_human` = `memory.rs:448` callers `run_inventory` only.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Forgotten-empty body (hard)** | When human format **and** `status=forgotten` **and** `rows.is_empty()`: keep existing Scope + `status=forgotten  limit=N` + exact `No forgotten memories.` Then print `Pinned: {n}` when COUNT is `Ok` (including **0**), then **always** print `next:` as the **last** non-empty stdout line. Exit **0**. |
| **F2 — COUNT is summary SoT (hard)** | `n` = `count_memories(&MemoryListFilter { status: Pinned, project_id, tag, limit: 0 })` — **same call as `run_summary`**. **Do not** call `count_pinned_memories` (no session join; can disagree with `--summary`). Same `--global` / `--tag` as the forgotten list. Fail-open: `Err` → omit the `Pinned:` line, still print `next:`. |
| **F3 — `next:` copy-paste (hard)** | Not global: exact `next: ai-brains memory list`. Global: exact `next: ai-brains memory list --global`. Do **not** echo `--limit` / `--tag` / `--format` / `--status forgotten` / `--project-id`. Default `memory list` is pinned inventory — that is the remediator. |
| **F4 — Empty const freeze (hard)** | `No forgotten memories.` byte-identical (T216 F14 / T198). Do **not** rewrite to “No forgotten memories (N pins).” `Pinned:` is a **following** line. |
| **F5 — `next:` last (hard)** | Last non-empty stdout line is the F3 string (T251 analog). No trailing dump. |
| **F6 — Shared backend (hard)** | Implement **only** in `emit_list_human` / a `pub(crate)` remediator in `memory.rs`. `forget --list-forgotten` and `memory list --status forgotten` both get the lines. **Do not** special-case `forget.rs`. **Do not** grow `forget.rs`. |
| **F7 — Forgotten recency freeze** | T287 F7 stands. Nonempty forgotten does **not** mix, does **not** grow `Pinned:` / `next: ai-brains memory list`. F36 stderr restore next **unchanged**. |
| **F8 — Pinned-empty freeze** | `No pinned memories.` path **unchanged**. Do **not** print forgotten `next:` or a second `Pinned:` on pinned-empty. |
| **F9 — Summary freeze** | `--summary` still COUNT-only (T216 F11/F46). Do **not** add `next:` there. Do **not** add `--summary` to `forget` (T216 F28). |
| **F10 — JSON freeze (hard)** | `--format json` keys stay today’s nine: `api_version`, `scope`, `project_id`, `status`, `items`, `returned`, `more_available`, `limit`, `total`. **No** `next_step`. **No** `pinned`. Empty forgotten JSON stays `items: []` / `total: 0`. Placeholder “additive `next_step` if keys allow” is **rewritten human-only** (scripts use `--summary --format json`). |
| **F11 — Limit freeze** | Default **50**, max **200**. Placeholder Manual uses `--limit 5` — not a default change. |
| **F12 — Scope / exit 2 freeze** | T216 F3. Missing project without `--global` → `fail_usage` exit **2**. Empty forgotten with scope is exit **0**. |
| **F13 — No live forget (hard)** | Do **not** `forget --match` / `--memory-id -f` / `--restore` on the operator vault. Hermetic tempfile pin (and hermetic forget for the nonempty-omit AC) is the proof. Live Manual is **already** Forgotten: 0. |
| **F14 — Pins / crates** | No workspace/lock bumps. clap 4.5 / lock 4.6.1 stay. rusqlite 0.39.0 stays. Zero new crates. |
| **F15 — Capture independence** | String emit + existing `count_memories`. No events. No models. No graph. No contracts crate. No new `QueryStore` method. |
| **F16 — Isolation** | No T240 F2 `.env` rewrite. No daemon start/stop/install. No auto-forget. No T300 steal. No doctor 16th. No `forget.rs` growth. No `project.rs` / `sync.rs` / `governed_common.rs` / `context.rs`. |
| **F17 — PATH** | Do not `cargo install`. Source/hermetic SoT. PATH 0.1.2 until owner asks. |
| **F18 — last-PR Cursor** | **#214** comments/reviews/issue **empty**. **No T301.** Dependabot `#61` rusqlite / `#58–#62` / `#68–#72` **not stolen**. |
| **F19 — Docs** | CAPABILITIES Empty row (`:274`) additive: forgotten-empty prints `Pinned: N` + last-line `next:`. OPERATIONS `:745` additive. WORKFLOWS `:195–198` additive empty case. CHANGELOG T299 Unreleased. Forget after_help `:1597` + memory list after_help `:2985` one sentence each. CLI-EXIT-CODES: empty forgotten still exit **0** (add a sentence if missing). PROTOCOL-COMPAT: **N/A** (CLI-local human; JSON keys unchanged — document “keys unchanged” only if a memory-list row exists at execute; today there is **no** PROTOCOL-COMPAT memory-list row). Phase 0 re-locates these anchors. |
| **F20 — Placeholder JSON rewrite** | Placeholder said “JSON: additive `next_step` if keys allow; else human-only.” Live JSON has **no** `next_step` field and T287 F10 froze the shape. **Human-only.** |
| **F21 — High findings** | Auto-forgetting the live vault; changing `No forgotten memories.`; adding JSON keys; mixing nonempty forgotten; putting T299 lines in `forget.rs`; `--summary` on forget; claiming CE wipe; clap 5; growing hotspot `project.rs`. |
| **F22 — Help** | Additive. Forget after_help already names `memory list`. Add: empty list-forgotten prints `Pinned: N` + `next: ai-brains memory list`. Combined help still lists `--list-forgotten`. |
| **F23 — Exit** | Empty/nonempty forgotten list → **0**. Missing scope → **2**. Invalid `--status` / empty `--tag` stay **2**. Store errors keep today’s fail path. |
| **F24 — Decline peers** | T300 graph sparse; leftover `--write`; T240 F2; T263 H2; T255 750 raise; tag histogram / `--offset` (T216 F24); clap 5 / rusqlite 0.40. |
| **F25 — Soft residuals** | PATH until install; JSON `next_step`; `--summary` on forget; tag histogram; `--offset`; `count_pinned_memories` vs session-join residual (not this COUNT). |
| **F26 — Helper (hard)** | Required `pub(crate) fn forgotten_empty_remediator(pinned: Option<u64>, global: bool) -> Vec<String>` in `memory.rs`. `Some(n)` → `Pinned: {n}` then `next:`. `None` → only `next:`. Global appends ` --global` on the next line only. **One line each.** No `\n` inside a vec item. Units rstest (AC10). `emit_list_human` must pass `tag` into the empty Forgotten COUNT (same as summary F46). |
| **F27 — T216 F36 partial lift** | Empty forgotten **stdout** grows T299 lines then `return Ok(())` (still **before** F36). F36 stderr restore sentence still **nonempty-only**. Do **not** print the restore sentence on empty. |
| **F28 — Existing tests stay green** | T216 empty `contains("No forgotten memories.")`; share-backend nonempty; JSON schema keys present; summary Pinned/Forgotten; tag exact-token; exit 2; T287 mix; F36 nonempty stderr. Additive `Pinned:` / `next:` do not break `contains`. |
| **F29 — 0-pin honesty** | Empty forgotten **and** 0 pins: `Pinned: 0` + `next:` (COUNT Ok). Never fabricate a DECISION preview. |
| **F30 — T266 Family B** | Default stays human (including pipes). Remediator applies to default + `--format human` only. |
| **F31 — `--tag` empty forgotten** | `Pinned: N` is tag-filtered (F2). Hermetic: pin with `TAGS: architecture`, empty forgotten `--tag architecture` matches `--summary --tag architecture` Pinned. |
| **F32 — Do not COUNT on JSON / nonempty / pinned-empty** | Extra SQL only in the forgotten-empty human arm. |
| **F33 — `last_nonempty_line`** | Hermetic AC uses a trim/last-nonempty helper (same idea as T251). Do not invent a shared crate helper unless one already exists in the test file — file-local is enough. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Hermetic vault, ≥1 pinned, **0** forgotten: `forget --list-forgotten --limit 5` exit **0**. Stdout contains exact `No forgotten memories.` Contains `Pinned: {n}` where `{n}` **equals** `memory list --summary` `Pinned:` in the same vault/scope (parse both). **Last non-empty line** is `next: ai-brains memory list`. Does **not** contain F36 restore sentence on stdout. Stderr does **not** contain `forget --restore` / `forget --memory-id`. |
| **AC2** | Same vault: `memory list --status forgotten --limit 5` stdout last-line / `Pinned:` / empty const **match** AC1 (shared backend). |
| **AC3** | Hermetic 0 pins + 0 forgotten: contains `No forgotten memories.` Contains `Pinned: 0`. Last line `next: ai-brains memory list`. Exit **0**. |
| **AC4** | Hermetic nonempty forgotten (pin then `forget --match` in **temp** vault): stdout contains the forgotten preview. Does **not** contain `next: ai-brains memory list`. Does **not** contain a T299 `Pinned:` line after the table (summary is a different command). F36 stderr restore **still** present. Exit **0**. |
| **AC5** | Hermetic empty forgotten `--format json`: parse object; required keys present (`api_version`, `scope`, `status`, `items`, `total`, `returned`, `more_available`, `limit`); **no** `next_step` / `pinned` / `next`; `items` empty array; `total == 0`; `status == "forgotten"`. |
| **AC6** | Hermetic `--global` empty forgotten (two projects, pins only on one): last line exact `next: ai-brains memory list --global`. Contains `Pinned:` matching `memory list --summary --global` Pinned. Does **not** use the non-global next string as last line. |
| **AC7** | `memory list` default (pinned) empty: `No pinned memories.` Does **not** contain `next: ai-brains memory list`. Exit **0**. (Stay-green AC7 empty pinned + T299 omit.) |
| **AC8** | `--summary` still prints `Pinned:` + `Forgotten:` and does **not** gain T299 `next: ai-brains memory list`. Existing summary test stays green. |
| **AC9** | Missing project + not `--global`: `forget --list-forgotten` still exit **2** via `fail_usage` (T216). Isolated empty-home / no `AI_BRAINS_PROJECT_ID`. |
| **AC10** | Unit rstest `forgotten_empty_remediator`: (1) `Some(3), false` → `["Pinned: 3", "next: ai-brains memory list"]`; (2) `None, false` → `["next: ai-brains memory list"]` and **not** `Pinned:`; (3) `Some(0), false` → `Pinned: 0` + non-global next; (4) `Some(2), true` → `Pinned: 2` + `next: ai-brains memory list --global`. Each vec item `!contains('\n')`. |
| **AC11** | Hermetic `--tag` (F31): pin `TAGS: architecture\nbody`; forgotten-empty `--tag architecture` `Pinned:` equals `--summary --tag architecture` Pinned; unknown tag still empty success + `Pinned: 0` (or omit if COUNT 0) + next. |
| **AC12** | Docs: CAPABILITIES Empty row additive; OPERATIONS additive; WORKFLOWS empty case; CHANGELOG T299 Unreleased; after_help Forget + memory list additive. Phase 0 re-locates anchors. |
| **AC13** | `forget --help` still lists `--list-forgotten` and `memory list`. `memory list --help` still lists `--status forgotten`. `cli_help_ia` Daily `memory` stays green. |
| **AC14** | Manual on **live** vault (do **not** forget/restore): `forget --list-forgotten --limit 5` + `memory list --summary`. Pass: forgotten-empty; stdout contains `No forgotten` **and** `Pinned:` matching that `--summary` Pinned (same scope) **and** last line `next: ai-brains memory list`; exit **0**. Record PATH vs `cargo run` if they differ (PATH-behind is F17). |
| **AC15** | No `ai-brains-contracts` type. No pin bumps. No new crate. `forget.rs` production **unchanged** (grep: T299 consts/helper not referenced from `forget.rs`). No new store method. |
| **AC16** | Stay-green: T216 share-backend nonempty; JSON schema keys present on **pinned** json; T287 mix; empty const substring; F36 nonempty stderr. |

---

## 5. Design notes

### 5.1 Human shape (empty forgotten, project scope)

```
Scope: project=C:\dev\ai-brains (3581317d-601e-44f7-ab84-fde90aa12d3c)
status=forgotten  limit=5
No forgotten memories.
Pinned: 4152
next: ai-brains memory list
```

`4152` is live-volatile; hermetic uses the summary COUNT from the same vault.

### 5.2 Helper sketch (`memory.rs`, `pub(crate)`)

```rust
pub(crate) fn forgotten_empty_remediator(pinned: Option<u64>, global: bool) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(n) = pinned {
        lines.push(format!("Pinned: {n}"));
    }
    if global {
        lines.push("next: ai-brains memory list --global".to_string());
    } else {
        lines.push("next: ai-brains memory list".to_string());
    }
    lines
}
```

In `emit_list_human` Forgotten empty arm **after** `No forgotten memories.`:

```rust
let pinned = ctx
    .conn
    .count_memories(&MemoryListFilter {
        status: MemoryListStatus::Pinned,
        project_id: if global { None } else { project_id.copied() },
        tag: tag.map(str::to_string),
        limit: 0,
    })
    .ok();
for line in forgotten_empty_remediator(pinned, global) {
    println!("{line}");
}
return Ok(());
```

Pass `tag: Option<&str>` into `emit_list_human` (already `#[allow(clippy::too_many_arguments)]`). JSON / nonempty / pinned-empty do not call this.

### 5.3 Why not JSON `next_step`

Placeholder allowed additive `next_step`. T216 F10 listed the key set; T287 F10 froze it. Default format is human. `--summary --format json` already exposes `pinned`. Adding a tenth key would be the first memory-list wire change since T216 — not needed for U=6.

### 5.4 Why not `count_pinned_memories`

T214 COUNT is `mp.project_id = ?` only. Inventory `count_memories` joins `session_projection` (`T216 F16`). Manual DoD says matching `--summary` Pinned. T288 already documented a 1-row drift between the two. Use summary’s method.

### 5.5 Why not grow `forget.rs`

Hotspot #5. Shared `run_inventory` is the SoT (T216 F1/F28). A forget-only branch would dual-truth `memory list --status forgotten`.

### 5.6 Why `next:` is `memory list`, not `--summary`

Placeholder: `next: ai-brains memory list`. Summary already printed `Pinned: N` on the empty list. Next is **skim pins** (T216 primary inventory), not recount.

---

## 6. Non-goals

- Auto-forget / retention apply / CE wipe / NIST Purge
- `--summary` on `forget`
- JSON new keys / `next_step` / `pinned` on list JSON
- Forgotten human mix / `--authority`
- `--offset` / tag histogram / relative-time helper extract
- Growing `forget.rs` / `query_store.rs` / contracts
- clap 5 / rusqlite 0.40 / workspace 0.1.3
- T300 live graph rebuild / floor retune
- leftover `--write` / T240 F2 / T263 H2
- `cargo install`

---

## 7. Verification plan (TDD)

**Red first (must fail on current tree):**

1. `forget_list_forgotten__empty_with_pin__pinned_count_and_next` (AC1)
2. `forgotten_empty_remediator__cases` (AC10 rstest — helper missing)
3. `forget_list_forgotten__empty_json__keys_frozen_no_next_step` (AC5 — keys-present may already pass; must assert **absence** of `next_step` **and** human AC1 still red)
4. `forget_list_forgotten__global_empty__next_includes_global` (AC6)

**Then green:** helper + empty Forgotten arm COUNT + pass `tag` + docs.

**Stay-green:** AC2 share / AC4 nonempty omit / AC7 pinned-empty / AC8 summary / AC9 exit 2 / AC16 T216+T287.

**Manual:** AC14 classify-only. Pass-with-observed-data on live Forgotten: 0. **Do not forget live pins.**

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Dual-truth forget vs `memory list --status forgotten` | F6 shared emit only; AC2. |
| `Pinned:` disagrees with `--summary` | F2 same `count_memories` filter; AC1 parse-both. |
| `count_pinned_memories` “already imported” shortcut | F2 forbid; AC15 no new store method. |
| JSON scripts grow a key | F10 / AC5 absence. |
| Live `--match -f` “to have a nonempty list” | F13 / AC14 empty vault is Manual SoT; nonempty is hermetic AC4. |
| F36 restore sentence on empty | F27 return before F36; AC1 stderr omit. |
| `forget.rs` hotspot growth | F6 / AC15 grep. |
| PATH 0.1.2 hides T299 | F17; hermetic + `cargo run` SoT. |
| Hotspot `project.rs` | Do not touch. |
| `--tag` COUNT drift | F31 / AC11 same two-stage as summary. |

---

## 9. Deferred absorb / decline

**Entire `conductor/deferred.md` scanned** (T142 archive through T298 closeout + T285–T300 mint). Overlapping open rows:

| Item | Disposition |
|------|-------------|
| Audit / mint “forget-list empty U=6” | **Absorb** F1–F6 / AC1–AC6 / AC14 |
| Placeholder Manual `forget --list-forgotten` + `memory list --summary` | **Absorb** AC14 / F13 |
| Placeholder keep `No forgotten memories.` + `Pinned: N` + `next: ai-brains memory list` | **Absorb** F1 / F3 / F4 |
| Placeholder JSON additive `next_step` if keys allow | **Rewrite** F20 / F10 — human-only |
| T216 F14 empty const / exit 0 | **Affirm** F4 / F23 |
| T216 F36 skip next on empty | **Partial lift** F27 — stdout `next:` on forgotten-empty; stderr restore still nonempty-only |
| T216 F10 / T287 F10 JSON keys | **Affirm freeze** F10 / AC5 |
| T216 F28 no `--summary` on forget | **Affirm** F9 |
| T216 F6 / F11 limit 50 / summary COUNT | **Affirm** F11 / F2 |
| T216 closeout tag histogram / `--offset` / auto-forget / CE wipe | **Decline** F24 / F13 |
| T287 F7 forgotten recency; “T299 owns empty next” | **Affirm** F7; **absorb** empty next |
| T287/T290/T291/T292/T293/T294/T298 “Decline → T299” | **Absorb** (this track) |
| T298 closeout “T299 forget-list / T300 not stolen” | **Absorb** this track; **T300** still not stolen |
| T274–T284 declined forget empty as E=8 | **Reopened** as this track (same class as T298 device empty) |
| T300 graph sparse live rebuild | **Decline** F24 |
| leftover `--write` / T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F14 / F24 |
| last-PR Cursor **#214** | **N/A empty** — **no T301** F18 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Not this track** — T258 / leftover data |
| Closed T198/T216/T287/T298 DoDs | **Stay closed** |

---

## 10. Implement order (on go)

1. Phase 0 re-verify (plan.md) + FEATURE TX.
2. Red AC1 / AC6 / AC10 (and AC5 absence lock).
3. Green helper + empty Forgotten arm + pass `tag`.
4. Stay-green AC2–AC4 / AC7–AC9 / AC16.
5. Docs AC12.
6. Manual AC14 (read-only; **no** live forget).
7. `scripts/dev-check.ps1`; Phase-1 review; `codex-review`.
8. conductor Completed + deferred closeout + pin.
9. Phase 6 publish (`track/T299-*` → PR → watch GHA `CI` green → squash-merge). Never `git push origin main`.

---

## 11. Soft residuals

| Residual | Notes |
|----------|--------|
| PATH until `cargo install` | F17 — source/hermetic SoT |
| Live Forgotten: 0 | Honest; AC14 empty is the Manual SoT |
| JSON `next_step` on list | F10 decline |
| `--summary` on `forget` | T216 F28 |
| Tag histogram / `--offset` | T216 F24 |
| `count_pinned_memories` vs session-join | T214 vs T216; this track uses inventory COUNT |
| T300 graph sparse | Next placeholder |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/memory.rs` | F26 remediator; empty Forgotten arm COUNT + print; pass `tag` into `emit_list_human`; units AC10 |
| `crates/ai-brains-cli/src/main.rs` | Forget + memory list `after_help` one sentence (F22). No new clap fields. |
| `crates/ai-brains-cli/tests/memory_list_inventory.rs` | AC1–AC9 / AC11 / AC16 hermetics (extend this file) |
| `Docs/CAPABILITIES.md` | Empty row `:274` additive |
| `Docs/OPERATIONS.md` | `:745` additive |
| `Docs/WORKFLOWS.md` | `:195–198` additive empty case |
| `CHANGELOG.md` | T299 Unreleased |
| `Docs/CLI-EXIT-CODES.md` | Empty forgotten still 0 (sentence if missing) |
| `conductor/conductor.md` / `deferred.md` / this spec+plan / README-T285-T300 | Planning now; Completed on go |

**Do not touch:** `forget.rs` production; `query_store.rs`; `doctor.rs`; `project.rs`; `ai-brains-contracts`; `Cargo.lock`; live vault forget/restore; PROTOCOL-COMPAT JSON key **set**.

---

## 13. AI fold-in

(empty until `/fold-in 299`)
