# T272 — Preflight `--global` Safety skip must not hide capped-out Index rows

- **Track ID:** T272-PreflightGlobalSafetyIndexSkip
- **Status:** **Planned** (Pending in registry; plan-only until go)
- **Category:** BUGFIX / UX
- **Owner:** —
- **Source:** Cursor Bugbot on PR [#179](https://github.com/Ryan-AI-Studios/AI-Brains/pull/179) (T264) — Medium “Safety IDs over-exclude Index” (`c5c3a0d4-408f-4ff8-8d39-b3961707fe1a`)
- **Depends on:** T264 ✅ Completed (label+cap+span; PR #179)
- **Blocks / feeds:** `--global` rollup is complete: a CONSTRAINT pin capped out of Safety **8** can still appear in Index/Recent. Does **not** unblock T270 retention classify.
- **Absorbs:** #179 Bugbot Medium; placeholder F1–F4; latent post-`dedup_hotspots` over-exclude (same SOOT: skip **emitted** ids)
- **Not absorbed (DoD):** T264 leftover-project drop from `recall --global` (F11 stands); T264 caps / LIMIT 40 / LIKE / labels / span formula; T265 `sections[]` / T180 2-key; T219 project-scoped selection; T264 Index fetch-80 leftover-heavy; session `HOTSPOT:` content skip; clap 5 / pin bumps
- **Research date:** 2026-08-20 (source HEAD `9008074` T269 `#186`)
- **Ledger:** planning DOCS TX (this pass). Implement starts a **BUGFIX** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** write live `.env`. Do **not** drop leftover from `--global` recall. Do **not** enable `AI_BRAINS_GOVERNED_BRIEFING`. Do **not** reopen T240 F2 / T255 declines. Do **not** grow CLI hotspot `preflight.rs` / `project.rs`. Do **not** retune `GLOBAL_*` caps. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

Under `--global`, Index/Recent skip must match **emitted** Safety ids (post HOTSPOT-suppress + `dedup_hotspots_keyed` + `take_round_robin`), not the pre-cap fetch window (`LIMIT 40`).

1. **Capped-out Safety pins reappear in Index.** A CONSTRAINT that lost the Safety round-robin slot is still a pinned bearing. Hiding it from Index/Recent makes it vanish from the `--global` rollup.
2. **Project-scoped skip stays “what Safety shows.”** No round-robin there. Skip set = post-dedup shown ids (fetch LIMIT 10 ≈ shown, except hotspot path-dedup).
3. **Keep T264 contracts.** Caps, fetch windows, LIKE, `[8hex]` tags, span formula, T180 `{text, word_count}` — unchanged.

That advances the north star: capture stays grant-independent; the append-only log stays SoT; agents starting with `preflight --global` must not lose a CONSTRAINT that T264 already fetched and then capped.

No models. No new crates. No clap 5. No leftover-project drop from `recall --global`.

---

## 2. Live baseline (re-scan 2026-08-20)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `9008074` — T269 `#186` squash. `main` == `origin/main`. Tree **CLEAN**. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **0.1.1**. T264 labels are in PATH (Safety `[C:\dev\ledgerful]` / Sessions `[C:\dev\stl]` / `[C:\dev\ai-brains]`). **Do not `cargo install`.** Skip-set lives in retrieval — PATH and source share the hole. |
| `preflight --summary` (daily) | Scope path owner `3581317d`. Pinned **3224**. Grants **0 of 3** (T241; not this track). |
| `preflight --global --pretty --compact -m 400` | T264 tags present. Compact + 400 words: Safety + Sessions; Index often absent (budget). Not the skip hole. |
| `preflight --global --pretty -m 800` | Safety is one large ledgerful pin (`[C:\dev\ledgerful] ## Objective` …). Sessions follow. **No Memory Index header** in this window — word budget ate Index. Hole is still the HashSet, not the missing header. |
| Code hole | `preflight.rs:329` `safety_ids.insert(memory_id)` on **every** fetched Safety row (`LIMIT 40`). `:337–342` then `take_round_robin` keeps **8**. `:467` Index `if safety_ids.contains(&memory_id) { continue; }`. **Same lines as Bugbot #179.** `ledgerful search --json -- "safety_ids"` hits `:286` / `:329` / `:467`. |
| `safety_for_skip` | Built **after** round-robin (`:346–350`) from emitted Safety bodies. Session CONSTRAINT skip is already post-cap. Index skip is not. |
| Last GitHub PR | [#186](https://github.com/Ryan-AI-Studios/AI-Brains/pull/186) T269 (2026-08-20). Issue comments **0**, review comments **0**, reviews **[]**. **last-PR Cursor: N/A (empty).** Open PRs: Dependabot remotes only (not this HEAD). **No T274.** Source leftover remains **#179** (this track). |
| Ledgerful | `doctor` ready (legacy `.changeguard` / sig-pin / sig-version / timings / :8081 unreachable; :8083 ok). 0 pending 0 drift. Hotspot **#1** `project.rs` (4.017) — **do not edit.** CLI `preflight.rs` **#7** (2.324, **2148** lines) — **do not grow.** Retrieval `preflight.rs` **1041** total / **962** non-blank — this track’s file; keep the diff small. |
| ai-brains recall | Lexical T272/T264 `safety_ids` thin (review-session noise). Code + #179 body are SoT. |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Pre-cap `safety_ids` (Bugbot #179) | T264 raised Safety SELECT to **LIMIT 40** so later projects can fill 8 slots. Skip still uses all 40 ids. Up to **32** CONSTRAINT pins fetched for Safety never print there **and** never print in Index/Recent. **DoD.** |
| Post-dedup over-exclude | `dedup_hotspots_keyed` can drop a HOTSPOT row **after** `:329` insert. That id is not shown in Safety but Index still skips it. Same SOOT as the cap. **Absorb into F1.** |
| Project-scoped LIMIT 10 | No round-robin. Skip-what-was-fetched ≈ skip-what-was-shown except hotspot dedup. Rebuild after dedup keeps F2 honest without a second algorithm. |
| Drop leftover from `recall --global` | T264 F11. `--global` means all projects. **Decline drop.** |
| T264 Index fetch 80 leftover-heavy | Soft R1b-P3-1. Index SQL recency of all pins, not this HashSet. T272 may *surface* capped CONSTRAINT pins in Index; it does not retune `GLOBAL_INDEX_FETCH`. **Decline steal.** |
| Session `content.contains("HOTSPOT:")` | Independent of `safety_ids`. Always hides HOTSPOT turns from Sessions. Not the Bugbot. **Decline as DoD** (soft residual). |
| T265 `sections[]` | Splitters key on `---` headers. Extra Index lines stay `id=index`. **Do not edit** CLI `preflight.rs` / `preflight_json.rs`. |
| T270 retention classify | Peer placeholder. **Decline.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Skip insert | `ai-brains-retrieval/src/preflight.rs:329` | `safety_ids.insert(memory_id)` **before** push to `safety_raw`. `memory_id` is then **dropped** — tuples are `(content, ts, project)` only. |
| Fetch | `:289–304` | Global `LIMIT 40` (`GLOBAL_SAFETY_FETCH`); project `LIMIT 10`. LIKE `CONSTRAINT:` / `INVARIANT:` / `HOTSPOT:` only. **Do not widen.** |
| HOTSPOT suppress | `:325–327` | `continue` **before** insert when ledgerful intelligence already present. Those ids can appear in Index today. **Keep.** |
| Dedup | `dedup_hotspots_keyed` `:799` | Input `(content, ts, T)` → `(content, T)`. Extra T is currently `Option<String>` project. Must carry `memory_id` in T. |
| Round-robin | `:336–342` | `take_round_robin(safety_entries, \|(_, pid)\| project_key(…), GLOBAL_SAFETY_PER_PROJECT=2, GLOBAL_SAFETY_MAX=8)`. |
| Index skip | `:467` | `if safety_ids.contains(&memory_id) { continue; }` — then privacy / low-signal / `GLOBAL_INDEX_FETCH=80`. |
| Index/Recent RR | `:505–524` | Separate round-robin on `collected`. Do **not** retune. |
| Session CONSTRAINT skip | `safety_for_skip` post-cap | Already emitted bodies. **Do not switch it to pre-cap.** |
| Caps / tags | `preflight_global.rs` | `take_round_robin` / `GLOBAL_*` / `prefix_first_line`. **Do not change constants.** New skip helper does **not** belong here unless a pure function is extracted (optional; default is rebuild in `preflight.rs`). |
| Hermetics | `crates/ai-brains-cli/tests/preflight_global_isolation.rs` | AC10 `three_a_one_b` pins A-one/A-two/A-three + B-only. Recency: A-three newest → Safety keeps A-three, B-only, A-two; **A-one is the capped needle.** Reuse that fixture. `safety_section()` exists; add `index_section()`. |
| T180 DTO | `PreflightContextResponse` | `{text, word_count}` (+ T265 `sections[]` additive). **Do not grow keys.** |
| `display_label` | `project.rs:422` | Hotspot #1 — **do not edit.** Tags already applied. |

### 2.4 Dependency / standards research (2026-08-20)

**Snapshot — re-verify at execute.**

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (docs.rs 4.6.6). **No clap 5.** | **No bump.** No new flags. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** T180 keys frozen. |
| `rusqlite` | workspace **0.39.0** / lock **0.39.0** | crates.io **0.40.2** (Dependabot #61 open) | **No bump.** No new SQL. `params![]` stands. |
| `tokio` | workspace **1.52** / lock **1.52.3** | crates.io **1.53.1** | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | `HashSet` is std | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Human output may change; JSON is the stable machine path | [clig.dev](https://clig.dev/) — “Human-readable output is paramount”; “Changing output for humans is usually OK”; `--json` for scripts | Index/Safety **human** (and the `text` blob) may gain a capped pin. T180 keys / T265 `sections[]` ids stay. |
| Saying just enough | clig.dev “Saying (just) enough” | Hiding a fetched-then-capped CONSTRAINT from **both** Safety and Index is saying too little. Skip is de-dupe of **shown** rows, not a second cap. |
| Facet filter = selected bucket | [Azure AI Search filters](https://learn.microsoft.com/en-us/azure/search/search-filters) — facet navigation filters the **selected** category | Analog: Index exclusion is “already in the Safety widget,” which is the **emitted** set. Unselected (capped) buckets stay searchable in the other widget. |
| clap 4 current | crates.io clap **4.6.6** | No flags. **No clap 5.** |

**N/A:** SQLCipher page encrypt, schtasks, Windows service, contracts DTO, llama.cpp `/health`.

**Could not verify from live `--global -m 800`:** a specific vanished id (Safety ate the word budget; Index header absent). Hermetic AC10 fixture is the proof, not live classification of 32k pins.

**ledgerful / ai-brains:** `preflight --summary`; `ledgerful doctor` (5 warn, work root this repo); ledger 0 pending / 0 drift; `index --incremental`; `search --json -- "safety_ids"` → `preflight.rs:286/329/467`; `scan --impact` CLEAN at `9008074`; `hotspots` project.rs #1 / CLI preflight.rs #7 (do not grow).

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a **BUGFIX** TX. |
| **F1 — Skip emitted ids only** | Rebuild `safety_ids` from the rows that remain in `safety_entries` after HOTSPOT-suppress + `dedup_hotspots_keyed` + (global) `take_round_robin`. Do **not** insert during the fetch loop. Index/Recent `contains` stays. |
| **F2 — Project-scoped** | No round-robin. Skip set = post-dedup shown ids. Fetch LIMIT 10 unchanged. Shown CONSTRAINT/INVARIANT still skipped from Index. |
| **F3 — Carry `memory_id`** | Today the id is inserted then dropped. Extra `T` on `dedup_hotspots_keyed` becomes `(Option<String>, String)` = `(project_id, memory_id)`. Round-robin key stays `project_key(project_id)`. Existing `#[cfg(test)] dedup_hotspots` (`T = ()`) stays green. |
| **F4 — T264 freeze** | Do **not** change `GLOBAL_SAFETY_*` / `GLOBAL_INDEX_*` / `GLOBAL_RECENT_*` / `GLOBAL_SESSION_*`. Do **not** change LIKE. Do **not** change tags / peel / span formula. Do **not** drop leftover from `recall --global`. |
| **F5 — `safety_for_skip`** | Stay post-cap (emitted bodies). Do not point session CONSTRAINT skip at the pre-cap HashSet. |
| **F6 — HOTSPOT suppress** | `has_cg_intelligence && content.contains("HOTSPOT:")` `continue` **before** `safety_raw.push`. Those ids never enter the skip set (today and after). |
| **F7 — T265 / T180** | Compact `--format json` stays 2 required keys. Additive `sections[]` splitters are header-based — extra Index lines stay `index`. **Do not edit** CLI `preflight.rs` / `preflight_json.rs`. |
| **F8 — No CLI flags** | No `--include-capped` / `--no-safety-skip`. Default skip is the remediator. |
| **F9 — Module / hotspots** | Diff lives in retrieval `preflight.rs` (carry extra + rebuild HashSet). **Do not** grow `project.rs` / CLI `preflight.rs` / `sync.rs` / `daemon.rs`. **Do not** retune `preflight_global.rs` constants. Optional: a 5-line `emitted_ids` helper next to the rebuild; not a new file. |
| **F10 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.1**. rusqlite stays **0.39**. |
| **F11 — Contracts** | No DTO. PROTOCOL-COMPAT untouched (no new keys). |
| **F12 — Capture independence** | SQL + in-memory HashSet only. No models, embeddings, graph, or new events. |
| **F13 — Tests** | Naming `function_or_feature__condition__expected_result`. Hermetic in `preflight_global_isolation.rs` (extend; `index_section` helper). Retrieval unit for post-dedup skip. Existing T264 AC5/AC10 + T265 2-key stay green. No `unwrap`/`expect`/`panic` in production. |
| **F14 — PATH** | Do not `cargo install` unless the user asks. Tests/manual use `cargo run` / hermetic. |
| **F15 — Stop-before** | Even after go: no live `.env`, no leftover rebind, no `policy bootstrap`, no T240 F2 silent Scope, no `nightly` mutate. |
| **F16 — Decline T270 / T273 F7** | Peers. `bridge_search_args` stays T273 soft residual. |
| **F17 — Decline T264 Index fetch leftover-heavy** | Soft R1b-P3-1. Not this HashSet. |
| **F18 — Decline session HOTSPOT content skip** | `content.contains("HOTSPOT:")` in the session loop is independent of `safety_ids`. Soft residual. |
| **F19 — Decline T219 LIKE / T250 PrettyCaps / word-budget retune** | Project-scoped selection stands. Default `-m` 1500 stands. Extra Index lines may push later sections out of a tight `-m`; accept. |
| **F20 — last-PR Cursor** | #186 empty → **N/A**. #179 is **this** track (absorb F1). **No T274.** |
| **F21 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F22 — Cross-model** | BUGFIX on T264 `--global` body selection (easy regression). After Phase-1 review clean, run read-only `codex-review`. |
| **F23 — Docs** | CAPABILITIES T264 row: one additive clause (Index/Recent skip = post-cap Safety ids). Root CHANGELOG T272 row. PROTOCOL-COMPAT unchanged. |
| **F24 — Span honesty** | `in_context_project_span` still counts **emitted** Safety/Index/Recent/Session items. Capped-out pins that land in Index may raise `N`. Do **not** freeze `N`. T264 AC7 `N >= 2` stands. |
| **F25 — Helper purity** | Rebuild is a `HashSet` from remaining extras. No I/O. Compare ids with `HashSet::contains` (existing). |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Retrieval unit: two HOTSPOT rows share a path; extras carry distinct `memory_id`s. After `dedup_hotspots_keyed`, skip set contains **only** the kept id (not the dropped duplicate). |
| **AC2** | Hermetic (reuse AC10 pin order: B-only, A-one, A-two, A-three): `--global --pretty --no-hook-prompt`. Safety section does **not** contain `A-one`. Index section **does** contain `A-one`. `A-two` / `A-three` / `B-only` stay in Safety (T264 AC10 still true). |
| **AC3** | Hermetic project-scoped (no `--global`): two CONSTRAINT pins in the scoped project both appear in Safety and **neither** unique needle appears in Index. |
| **AC4** | Existing `preflight_global_isolation__three_a_one_b__b_appears_a_capped` (T264 AC10) stays green (`a_count <= 2`, B in Safety). |
| **AC5** | Existing T264 AC5 labels + two-line continuation stay green. |
| **AC6** | Existing T265 compact `--format json` 2-key (`t180_c_preflight_json_keys` or isolation `compact_json__two_keys`) stays green. |
| **AC7** | `--global --summary` still contains `In context spans` (T264 AC7 class). Do not assert a frozen `N`. |
| **AC8** | Docs: CAPABILITIES T264 bullet additive skip-emitted clause; root CHANGELOG T272 row. |
| **AC9** | No contracts DTO; no pin bumps; CLI `preflight.rs` / `project.rs` / `preflight_global.rs` constants untouched; `embeddings.rs` / `doctor.rs` 15-check untouched. |
| **AC10** | Manual (source bin, classify-only): `cargo run -p ai-brains-cli -- preflight --global --pretty --no-hook-prompt` (default `-m`) exits **0**. Safety still tagged. If Memory Index is in the window, it may list CONSTRAINT-class pins that are absent from the Safety section (pass-with-observed-data if Index is budgeted out, same class as T264 AC14). Do **not** pin. Do **not** `cargo install`. |
| **AC11** | Session CONSTRAINT skip still uses `safety_for_skip` (existing session tests / isolation AC5 stay green; no pre-cap leak of A-one into session skip). |

---

## 5. Design notes

### 5.1 Why rebuild, not “insert after round-robin only”

Round-robin is global-only. Dedup runs on **both** paths. The skip set has one owner. Rebuild after the whole Safety pipeline is one SOOT:

```text
fetch LIMIT 40|10
  → drop HOTSPOT-if-cg (never in raw)
  → dedup_hotspots_keyed
  → if global: take_round_robin(2, 8)
  → safety_ids = { memory_id of remaining rows }
```

Insert-during-fetch cannot see later drops. `memory_id` must survive in extra `T`.

### 5.2 Extra tuple (not a new struct unless clippy demands)

```rust
// safety_raw: (content, ts, (project_id, memory_id))
safety_raw.push((strip_ansi(&content), updated_at, (item_project, memory_id)));
let mut safety_entries = dedup_hotspots_keyed(safety_raw);
if global {
    safety_entries = take_round_robin(
        safety_entries,
        |(_, (pid, _))| project_key(pid.as_deref()),
        GLOBAL_SAFETY_PER_PROJECT,
        GLOBAL_SAFETY_MAX,
    );
}
let safety_ids: HashSet<String> = safety_entries
    .iter()
    .map(|(_, (_, id))| id.clone())
    .collect();
```

Call-site loops that currently destructure `(entry, pid)` become `(entry, (pid, _))`. Span still uses `pid`.

### 5.3 AC2 recency (lock it)

`pin_memory` order in AC10: B, A-one, A-two, A-three. `ORDER BY updated_at DESC` → A-three, A-two, A-one, B-only. First-seen projects: **A** then **B**. Round 0: A-three, B-only. Round 1: A-two. **A-one is the capped needle.** Test asserts that exact string, not a count.

### 5.4 Capture independence

No `MemoryPinned`, no governed packet, no models. HashSet membership only.

---

## 6. Non-goals

- Dropping any project from `recall` / `search` / `sync query` `--global`
- Raising/lowering `GLOBAL_*` caps or LIMIT 40 / Index fetch 80
- Adding `DECISION:` to Safety LIKE
- T265 json-v2 / typed arrays / CLI splitter edits
- T219 project-scoped marker selection / T250 `PrettyCaps`
- Session `HOTSPOT:` content skip
- Ledgerful under `--global` (T214 F9)
- `AI_BRAINS_GOVERNED_BRIEFING` / governed multi-project packet
- Growing `PreflightContextResponse`
- clap 5 / lock bumps / new crates / rusqlite 0.40
- Editing `project.rs`
- Live leftover `rebind-path` / `.env` / `policy bootstrap` / `cargo install`
- T240 F2 silent Scope switch; T255 doctor-16th / product `.cmd`
- T270 retention classify; T273 `bridge_search_args`
- Mint T274 from #186 (empty)

---

## 7. Verification plan (TDD)

**Red first (names):**

1. `dedup_hotspots_keyed__duplicate_path__skip_set_omits_dropped_id` (AC1)
2. `preflight_global_isolation__capped_out_safety__appears_in_index` (AC2)
3. `preflight_global_isolation__project_scoped__shown_safety_not_in_index` (AC3)

Prove AC2 fails on current tree: A-one missing from Index (skip of all 4 fetch ids). Then green: carry extra + rebuild HashSet.

Stay green: T264 AC5/AC10, T265 2-key, isolation compact tags, `dedup_hotspots` unit (T=`()`).

```powershell
# Red → green
cargo nextest run -p ai-brains-retrieval --lib
cargo nextest run -p ai-brains-cli -E "test(preflight_global_isolation)"
cargo clippy -p ai-brains-retrieval --all-targets -- -D warnings

# Manual (do not pin / cargo install)
cargo run -q -p ai-brains-cli -- preflight --global --pretty --no-hook-prompt

# Full gate (before finalize)
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| T264 AC10 fails (B missing / 3×A) | F4 freeze caps; AC4 re-run |
| Duplicate CONSTRAINT in Safety **and** Index | AC2 asserts A-two/A-three **in** Safety and A-one **not** in Safety; skip still contains emitted ids |
| Project-scoped Index suddenly lists Safety pins | AC3 |
| `dedup_hotspots` unit type error | Keep `T=()` test helper; only production extra grows |
| CLI hotspot / `project.rs` edits | F9 forbid |
| T180 2-key break | AC6 |
| Word-budget hides Index on live `-m 800` | AC10 pass-with-observed-data; hermetic AC2 is the proof |
| Span `N` changes | F24 — allowed |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-20. `ISSUES.md` does not exist.

| Item | Disposition |
|------|-------------|
| last-PR Cursor #179 Safety IDs over-exclude Index | **Absorb** F1–F3 / AC1–AC4 — still true at `preflight.rs:329` + `:467` |
| Placeholder F1 post-cap skip | **Absorb** F1 |
| Placeholder F2 project-scoped skip-what-shows | **Absorb** F2 |
| Placeholder F3 no leftover recall drop / no T264 retune | **Absorb** F4 |
| Placeholder F4 hermetic capped-out in Index | **Absorb** AC2 |
| Post-dedup over-exclude (latent; same insert-before-drop) | **Absorb** F1 / AC1 |
| T264 Index fetch window 80 leftover-heavy (R1b-P3-1) | **Decline** F17 |
| T264 span vs word-budget trim (R1b-P3-2) | **Decline** — pretty trim residual; F24 span may rise |
| T264 pretty still in CLI hotspot | **Decline** — F9 do not grow CLI `preflight.rs` |
| T265 `safety_ids` peer placeholder | **Absorb this track** (T265 declined F11; we own the file now) |
| T265 pretty-walker duplication / json-v2 | **Decline** F7 / F19 |
| T269 PATH `cargo install` / JSON probe budget | **Decline** F14 — not skip-set |
| T270 retention 0 candidates | **Decline** F16 — peer placeholder |
| T273 F7 `bridge_search_args` | **Decline** F16 |
| last-PR Cursor #186 | **N/A** — comments/reviews empty. **No T274.** |
| T240 F2 / T255 bag (doctor 16th / persist probe / `.cmd` / `--no-vault`) | **Decline** F15 |
| T214 ledgerful-on-global | **Decline** — T264 F14 |
| T219 F13 project-scoped selection | **Affirm** F19 |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `changeguard`, R-CI-BRANCH, rusqlite 0.40 | **Decline** — not skip-set |
| Session `HOTSPOT:` content skip | **Decline** F18 (soft) |

---

## 10. Implement order (on go)

1. Phase 0: re-verify `:329`/`:467` + deferred rescan + BUGFIX TX.
2. **Red:** AC1 unit (dropped hotspot id still in a pre-rebuild mental HashSet — write the unit against a helper or against `dedup` output); AC2 hermetic (fails: A-one absent from Index); AC3 project-scoped negative.
3. **Green:** carry `(project_id, memory_id)` through `safety_raw` / dedup / round-robin; rebuild `safety_ids`; remove fetch-loop insert.
4. Confirm AC4–AC7/AC11 stay green.
5. Docs AC8; manual AC10.
6. Phase-1 review → Codex (F22) → gate → publish (implement-track Phase 6).

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| Session `HOTSPOT:` content skip | F18 — may still hide a capped-out hotspot from Sessions |
| Index fetch 80 leftover-heavy | T264 R1b-P3-1 — F17 |
| Live `-m` windows without an Index header | Word budget; hermetic is DoD |
| PATH `cargo install` | F14 — operator |
| T270 / T273 F7 | Peers |
| rusqlite `table_exists` 0.40 | T213 L4 — not this track |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-retrieval/src/preflight.rs` | Carry `memory_id` in extra; rebuild `safety_ids` after pipeline; AC1 unit |
| `crates/ai-brains-cli/tests/preflight_global_isolation.rs` | `index_section` helper; AC2 + AC3 hermetics |
| `Docs/CAPABILITIES.md` | Additive T264 skip-emitted clause |
| `CHANGELOG.md` | T272 row |
| `conductor/conductor.md` | T272 Planned (registry stays **Pending** until go) |
| `conductor/deferred.md` | Absorb/decline notes |
| **Do not touch** | `project.rs`, CLI `preflight.rs` / `preflight_json.rs`, `preflight_global.rs` constants, `sessions.rs`, contracts DTO, clap pins, live `.env` |
