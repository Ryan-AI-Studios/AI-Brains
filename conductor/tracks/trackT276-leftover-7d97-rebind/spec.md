# T276 — Leftover `7d97a456` must not starve `--global` (label, do not drop)

- **Track ID:** T276-Leftover7d97Rebind
- **Status:** **Completed** (2026-08-22)
- **Category:** FEATURE / UX / RETRIEVAL
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `recall --global` **7/3**; `project list-paths` **8/7**; leftover ~18k pins; many `C:\dev\*` path aliases. Placeholder minted with T274–T284 (`deabae7`).
- **Depends on:** T259 ✅ `list-paths --shared-only` / `rebind-path` (memories stay F5); T264 ✅ preflight label+cap (**F11 decline leftover recall drop**); T258 ✅ adopt-path (cwd whoami `mismatch: false`); T274 ✅ authority two-pass; T240 F2 freeze
- **Blocks / feeds:** `--global` unique owner pins can enter `candidate_depth`. Leftover hits are labeled, not hidden. Path-alias split stays T259. List row order **T283**. `context --show` shell leftover **T282**. Safety **T279**. Grants **T275**.
- **Absorbs:** Placeholder problem text + Manual DoD (hermetic unique-owner pin vs leftover); deferred.md “leftover `7d97a456` ~18k / `--global` junk”; T264 soft “Recall leftover-first under `--global`” (**filter flag declined as DoD**); T259 leftover-memory-reclassify **declined** (F5); T270 closeout “18k pins still owned by `7d97a456`”; identity-mismatch observation `7d97a456` vs `fcb8a40f` as **leftover data**, not a new identity model
- **Not absorbed (DoD):** Silent exclude leftover from `--global` (T264 F11); memory rewrite / `MemoryMoved` (T259 F5); live leftover `rebind-path --write --yes` without owner confirm; T240 F2 `.env`; T258 adopt-path (cwd already `mismatch: false`); T282 shell leftover on `context --show`; T283 `project list` cwd-first; T274 chrome penalty retune; T275 grants; T279 Safety; T280 hint; T284 #188; clap 5 / rusqlite 0.40 / DTO keys; `--exclude-project` flag
- **Research date:** 2026-08-21 (plan dogfood HEAD `a5562cc` T275 `#190`; product `src/` = T275). Fold-in against `61fd3cb` (plan docs; crates identical to `a5562cc`).
- **AI fold-in:** 2026-08-21 `agy-review.md` + `opencode-review.md`. **B 0 / M 0.** **Agree:** Agy m1 tag-before-score (F4 / AC4); Agy m2 HashSet dedupe (F38 / AC1); Agy O2 preferred-full skip (F39). **Already:** Agy O1 `format_pretty_hit_line` `project_tag` (F18). **Clarify:** OpenCode m1 two `lexical_search` is F1 — COALESCE SELECT stays **F15 for tags**, not the fill route. **Agree:** OpenCode m2 both arms `prefer_authority: true` + bridge stays `project_id` None (F40); OpenCode m3 AC3 is pre-rerank (F41). **Decline:** OpenCode O1 empty-hint “Try --global” (live `build_recall_hint_core` global arm does not say that); OpenCode `display_label` at CP `briefings/project.rs:383` (live is CLI `project.rs:383`); leftover UUID `7d97a51a` typo (`7d97a456`). Disposition **§13**.
- **Ledger:** planning DOCS TX `d5b9a9cc-fa83-4ce9-a74f-aaf77eb591fe`. Fold-in DOCS TX `30332efc-0716-4f22-ab89-5879cde7aa2e`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`, rewrite `.env`, rebind live leftover paths, bootstrap live grants, pin-as-implement to the live vault, or live `retention apply --confirm`. Do **not** grow hotspot `project.rs` / CLI `preflight.rs` / `sync.rs` (except one `RecallOptions` field) / `governed_common.rs` / `ranking.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** raise `candidate_depth` (T261). Do **not** hardcode leftover UUID `7d97a456-…` in retrieval SQL.

---

## 1. Objective

1. **`--global` is a rollup, not leftover’s private index.** A unique pin on the **current effective project** (cwd path-owner / `.env` `AI_BRAINS_PROJECT_ID` before T112 clear) must enter the lexical candidate set and can win top-N against leftover volume. `--global` still means all projects (T264 F11).
2. **Leftover is labeled, not dropped.** Pretty `--global` hits show a T264-class `[8hex]` (upgraded via `display_label` when unique). Agents can see that a hit is leftover instead of treating unlabeled dumps as “this repo.”
3. **T259 tools stay the remediator.** `rebind-path` / `unregister-path` remain print-only-by-default path-alias surgery. This track does **not** move 18k historical pins. Nightly Phase 2 still walks leftover roots until the operator confirms a write **out of band**.
4. **North star.** Capture independence: ranking/retrieval + pretty chrome only. Compensating path events already exist (T259). No hidden CoT. No silent Scope write (T240 F2).

This unblocks the daily product: T259 made leftover **inventory + rebind** possible; T264 labeled **preflight** `--global`; T274 got pins into scoped recall. The 2026-08-21 audit still scores `--global` **7/3** because leftover ~18k monopolizes unscoped `MATCH LIMIT depth` and hits have **no project tag**.

---

## 2. Live baseline (re-scan 2026-08-21)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `a5562cc` T275 squash `#190`. `main` = `origin/main`. Tree **CLEAN**. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-21 05:55**, 25 368 576 bytes, **0.1.1**. **T270** on PATH (before T274 11:52Z / T275 22:44Z). **Do not `cargo install`.** |
| Source debug | `target\debug\ai-brains.exe` mtime **2026-08-21 18:34**. Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **3352**. In-context **0/0/0**. Grants **0 of 3** (T275 hermetic; live not bootstrapped). Capture independence holds. |
| `project whoami --format json` | `mismatch: false`. env/path/detect/effective = `3581317d-…`. **`shell_project_id` leftover `7d97a456-…`**. Remediations `[]`. Placeholder “whoami mismatch:false” is **already true** for cwd (T258). Shell leftover is **T282**. |
| `memory list --summary --global` | Pinned **38833**. Leftover `7d97a456` **18038** (~46%). Next: `fcb8a40f` ledgerful **4875**; `3cededdd` stl **3381**; this repo `3581317d` **3352**. |
| `project list-paths --shared-only --format json` | **11** leftover roots, all `exists: true`, alias empty: `C:\dev\crawlx`, `dedupe`, `degoo`, `family`, `gimp`, `homebrew-tap`, `kinledger`, `ledgerful-action`, `ledgerful-frontend`, `ledgerful-web`, `wondermaker`. **Same 11 as T259.** Rebind tools shipped; **nothing moved.** |
| `recall "T270 memory_legacy" --global --limit 5 --no-bridge` | Unique needle: three `DECISION:` T270 pins then T270 reviews. **Unlabeled** — no `[3581317d]` / leftover tag. PATH-behind T274 chrome. |
| `recall "what did we decide" --global --limit 5 --no-bridge` | Hit #1 T263 `DECISION:` then unlabeled JSON/`## Objective` dumps. No project tag. Leftover vs owner **indistinguishable**. |
| Last GitHub PR | [#190](https://github.com/Ryan-AI-Studios/AI-Brains/pull/190) T275 (2026-08-21). `gh pr view --comments`, `/reviews`, `/comments`, `issues/190/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, actions). **No leftover to mint.** Prior #188 Bugbot Mediums remain **T284** (2 inline comments). |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081). **0 pending / 0 drift.** Hotspot **#1** `project.rs` (**3.981**, **1332** lines). `sync.rs` **#2**. CLI `preflight.rs` **#7** (**2027**). |
| `ISSUES.md` | **Does not exist.** |
| Identity observation `7d97` vs `fcb8a40f` | `fcb8a40f` is a **real** sibling project (`C:\dev\ledgerful`, 4875 pins). Daily Scope leftover vs that path owner is **T258 in that repo**. This track does not mint T285. |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Leftover 18 038 / 11 roots | T259 shipped inventory + one-tx rebind. Operator has not `--write --yes`. Nightly Phase 2 still walks leftover. **DoD is not live split** — it is `--global` usable **despite** leftover remaining. |
| `--global` unlabeled | T264 tagged **preflight** Safety/Index/Recent. `recall` pretty `format_pretty_hit_line` has score + session only. Agents cannot tell leftover from this repo. **DoD: pretty tags.** |
| Candidate starve | T112 `--global` clears `project_id` so MATCH is vault-wide. `candidate_depth(5)=15`. Leftover volume can fill LIMIT before a unique owner pin enters (T260/T274 lesson). Unique needles may already win; generic “what did we decide” is unlabeled dumps. **DoD: prefer-fill**, not exclude. |
| Silent exclude leftover | T264 F11 + T259 F5: `--global` means all projects; 18k historical pins have no other unscoped path. Elastic/Mongo “always filter `tenant_id`” is **wrong** here — leftover is a dump tenant, not a security tenant. **Decline drop.** |
| `--exclude-project` flag | T264 residual “new track only if owner wants a **filter flag**.” clig.dev: prefer flags, but also don’t pollute. Prefer-fill + label is the default-right thing without a new verb. **Decline as DoD** (soft). |
| Memory reclassify by path | T259 F5 / event-sourcing compensating facts stay on the old stream. No `MemoryMoved`. **Decline.** |
| whoami mismatch:false | Already true on cwd (T258). **Not DoD.** Shell leftover **T282**. |
| Mojibake `â€™` | Append-only: do **not** transcode historical content. Label leftover. **Decline rewrite.** |
| List sort leftover-first | **T283.** |
| T240 F2 | Standing. |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| `--global` clear | `main.rs` **`:4320–4326`** | T112: `(None, None)` for project+session. **Keep.** Thread **pre-clear** `project_id` as `preferred_project_id` only. |
| `RecallOptions` | `retrieval/src/recall.rs` **`:14–33`** (`Default`) | **No** `global` / `preferred_project_id` today. Add `preferred_project_id: Option<ProjectId>` (Default `None`). |
| `RecallHit` | `recall.rs` **`:36–54`** | **No** `project_id`. Constructors `fts` `:82`. Bridge leaves `None`. |
| Lexical SELECT | `lexical.rs` `match_sql_and_params` **`:259–265`** | `mp.memory_id, content, privacy, session_id, rank, updated_at` — **no** project. `RetrievalMemory` **`:11–18`** same. |
| T274 two-pass | `lexical.rs` `prefer_authority` **`:167–194`**; `session_chrome.rs` **238** lines | Stays **inside** each MATCH arm. Prefer-fill is a **second** `lexical_search` with `project_id = preferred`, then merge. |
| `rerank_hits` | `ranking.rs` **`:260`** (**939** lines) | F40 single post-blend. **Do not retune.** Prefer-fill feeds the set. |
| Pretty hit | CLI `recall.rs` `format_pretty_hit_line` **`:407–447`** (**1438** lines) | Score / session / badge. **No** project tag. `print_pretty_hits` `:473` also used by `sync.rs` `:544`. |
| T264 tags | `preflight_pretty.rs` **200** lines; `preflight_global.rs` | Peel/upgrade + `display_label`. **Reuse** from recall pretty. Do **not** grow CLI `preflight.rs`. |
| `display_label` | `project.rs` **`:383`** `pub(crate)` | Call only. **Do not grow** hotspot #1 (**1332**). |
| T259 rebind | `project_rebind.rs` (**104** lines); CP `grants.rs` `rebind_path_alias` **`:287`** | Print-only default; `--write --yes`; `memories_moved: false`. **Call only.** |
| Contracts | `RecallResult` — `memory_id, content, source, score, session_id, staleness, score_kind, cosine` | **No** `project_id` key. T180 N−1. |
| `candidate_depth` | `hybrid.rs` | `limit*3` clamp **15..50**. **Do not raise.** |
| Sync vault | `sync.rs` `recall_full` **`:487–502`** (hotspot **#2**) | Pass `preferred_project_id` only. Pretty via `print_pretty_hits`. |
| Search | alias of `recall` (T243) | Same `RecallRunOptions`. |

### 2.4 Dependency / standards research (2026-08-21)

**Snapshot — re-verify at execute.**

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** (builder **4.6.0**) | crates.io **4.6.6** (2026-08-06). GitHub latest tag **v4.6.6**. **No clap 5.** | **No bump.** No new flags. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** (2026-07-20) | **No bump.** JSON keys frozen. |
| `chrono` | workspace **0.4** / lock **0.4.44** | crates.io **0.4.45** (Dependabot #62) | **No bump.** |
| `rusqlite` | lock **0.39.0** | crates.io **0.40.2** (2026-08-08; Dependabot #61) | **No bump.** Extra SELECT column only. |
| `uuid` | workspace **1.13** / lock **1.23.1** | — | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | — | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| `--global` must stay all-projects | T264 F11 (live spec); [clig.dev](https://clig.dev/) “Make the default the right thing” + “Actions crossing the boundary should be explicit” | Silent exclude of leftover hides the only unscoped path to 18k pins. Prefer-fill + label changes **ranking/chrome**, not the meaning of `--global`. |
| Explicit filter flag | clig.dev Prefer flags; T264 §11 “new track only if owner wants a **filter flag**” | A `--exclude-project` would be honest, but pollutes clap and is unused if prefer-fill works. **Decline as DoD.** |
| Multi-tenant search isolation | [Elastic agent-memory DLS](https://www.elastic.co/search-labs/blog/agent-memory-elasticsearch) (2026-06); [MongoDB Search tenant filter](https://www.mongodb.com/docs/search/deployment/multi-tenant-architecture/) | Those are **security** tenants (never leak). Leftover is a **dump identity**, not a confidentiality boundary. Filter-always is the T264 decline. |
| Prefer current tenant without hiding others | [Elastic ecommerce boost](https://www.elastic.co/search-labs/blog/ecommerce-search-optimization-query-governed) (2026-05): boost is policy, not a hard filter. [Qdrant 1.19 per-tenant IDF](https://github.com/qdrant/qdrant/releases) (2026-08) is a scoring fix, not drop. | Prefer-fill = “cwd pins enter the window.” Do **not** retune global BM25 IDF (SQLite FTS5 has no per-tenant IDF). |
| Compensating events; historical facts stay | [Azure Event Sourcing](https://learn.microsoft.com/en-us/azure/architecture/patterns/event-sourcing); Fowler | T259 Removed+Added already. Do **not** rewrite `MemoryPinned`. |
| Mojibake | UTF-8 vs Windows-1252 (`â€™` = U+2019 misread) | Event log is immutable. Do not transcode. Label the tenant. |
| clap 4 current | [docs.rs/clap/4.6.6](https://docs.rs/clap/4.6.6/clap/) | No new `Arg`. `after_help` one-liner optional. |

**N/A:** SQLCipher page crypto, schtasks, T180 2-key DTO growth, Windows service, llama.cpp `/health`, grant bootstrap (T275).

**Could not verify:** live leftover `--write --yes` (stop-before). Hermetic leftover+owner is the proof. PATH `--global` ranking is T270; source has T274 two-pass — Phase 0 re-dogfood `cargo run`.

**ledgerful / ai-brains:** `preflight --summary` 3352 pins @ `3581317d`; leftover 18038 / 11 roots; `whoami` mismatch false + shell leftover; `search "rebind_path_alias"` → `grants.rs:287` / `project_rebind.rs:57`; `search "prefer_authority"` → `lexical.rs:167` / `session_chrome.rs:116`. Recall lexical of leftover still surfaces T259 review-track Objective (PATH-behind T274).

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a **FEATURE** TX. |
| **F1 — Prefer-fill (required)** | When `preferred_project_id` is `Some` (CLI `--global` with a pre-clear effective project): two public `lexical_search` calls (each T274 `prefer_authority: true`, limit = `candidate_depth`) — scoped to preferred, then unscoped global. Merge in `prefer_project.rs` (F38/F39) **then** existing `rerank_hits`. Do **not** exclude leftover. Do **not** add a third MATCH inside `match_query`. |
| **F2 — Preferred id** | Preferred = CLI `project_id` **before** T112 `--global` clear (env / AppContext effective). Retrieval **must not** hardcode `7d97a456-f2f4-43ea-1f13-211af684ad37`. Works for leftover **and** `fcb8a40f` when that repo is cwd. |
| **F3 — Skip when none** | `preferred_project_id == None` (true vault-wide, no `.env`) → current unscoped path only. No fill. Pretty tags still apply if `RecallHit.project_id` is present. |
| **F4 — Pretty tags `--global` only** | Pretty `--global` (recall / search / sync vault pretty) prefixes each hit with T264 grammar: `[` + 8 hex + `]` or upgraded `display_label` (truncate 32, `]` → `·`). **Order (Agy m1):** leading tag, **one space**, then the existing `[score=…]` / `[rank=#n]` bracket. Example: `[3581317d] [score=-21.237 \| session=…] <uuid>: …`. Project-scoped pretty has **no** `[8hex]` tags (T264 F1 analog). |
| **F5 — JSON E1 freeze** | `RecallResult` **no** `project_id` key. Machine agents use scoped recall or parse pretty. N−1 ignore extras still holds. |
| **F6 — T264 F11** | Do **not** `AND project_id != leftover` (or any project) on `--global`. AC3 leftover matching dump still appears in the **pre-rerank** candidate merge when preferred did not fill `depth` (F39/F41). |
| **F7 — T259 F5** | No memory move / copy / forget / CE. `rebind-path` unchanged. |
| **F8 — T240 F2** | No `.env` write. No silent Scope switch. |
| **F9 — Live leftover Stop-Before** | Plan + implement **must not** `rebind-path --write --yes` / `unregister-path` against live leftover roots unless the owner confirms in the go prompt. Hermetic is sufficient DoD. |
| **F10 — whoami** | Cwd `mismatch: false` is **already** T258. Not a T276 AC. Shell leftover → **T282**. |
| **F11 — T283 / T282 / T279 / T280 / T284 / T275** | Do not steal. |
| **F12 — T274 isolation** | `SESSION_CHROME_PENALTY`, authority GLOB, `classify_pin_kind` **untouched**. T274 two-pass stays inside each MATCH. |
| **F13 — Depth** | Do **not** raise `candidate_depth`. |
| **F14 — Ranking.rs** | **Do not edit.** Prefer-fill is the lever (T260/T274: pin must **enter** the set). No leftover penalty constant. |
| **F15 — Internal project_id (tags, not fill)** | `RetrievalMemory.project_id: Option<String>` + `RecallHit.project_id: Option<String>` from `COALESCE(mp.project_id, sp.project_id)` (mig **0015**). **OpenCode m1 clarify:** COALESCE is **only** so F4 can tag leftover vs owner. Prefer-fill is F1 two `lexical_search` calls — do **not** treat COALESCE as a SQL prefer-fill. Constructors default `None`; lexical maps Some. Bridge/graph inherit or None. |
| **F16 — Merge module** | New `crates/ai-brains-retrieval/src/prefer_project.rs` (`merge_preferred_then_global`). Do not grow `ranking.rs`. Keep `lexical.rs` two-pass **inside** each call (F12). |
| **F17 — CLI wiring** | `RecallRunOptions.preferred_project_id`. `main.rs` when `*global`: keep pre-clear clap `project_id` as preferred; still pass `project_id: None` into scoped filter + bridge. `search` same. Live order: `.env` force-set `:3268` → `Cli::parse()` `:3356` (`env = AI_BRAINS_PROJECT_ID` `:1017`) → T112 clear `:4322`. |
| **F18 — Pretty helper** | Extend `format_pretty_hit_line` with `project_tag: Option<&str>` (existing units pass `None`). Shared by `print_pretty_hits` (recall + sync). Optional thin `recall_global.rs` for upgrade/lookup only. Reuse `preflight_pretty` peel/upgrade + CLI `project.rs` `display_label` **`:383`** (not CP briefings). **Do not** grow `project.rs`. |
| **F19 — Sync** | Hotspot #2: pass `preferred_project_id` on `recall_full` only. Pretty tags via existing `print_pretty_hits`. No new sync pane. |
| **F20 — No `--exclude-project`** | Not DoD. Soft residual. |
| **F21 — Pins / crates** | No clap 5, no rusqlite 0.40, no chrono 0.4.45, no new crates, workspace **0.1.1**. |
| **F22 — PATH** | Do not `cargo install` unless the user asks. |
| **F23 — Tests** | Naming `function_or_feature__condition__expected_result`. Hermetic `tempfile::tempdir`. No production `unwrap`/`expect`/`panic`. |
| **F24 — Docs** | CAPABILITIES: `--global` prefer-fill + labels; leftover runbook still T259 rebind (never `set-alias 7d97 AI-Brains`). CHANGELOG minor. OPERATIONS one sentence. |
| **F25 — last-PR Cursor** | #190 comments/reviews **empty** → N/A. #188 two Mediums stay **T284**. **No T285.** Open HEAD PR: none (Dependabot remotes). |
| **F26 — Decline peers** | T277 backup; T278 graph; T279 Safety; T280 hint; T281 nightly; T282 context leftover; T283 list cwd-first; T284 Work/samples; T240 F2; T255 750 ms; T263 H2; T266 JSON freeze; T275 live bootstrap. |
| **F27 — Cross-model** | FEATURE (retrieval + identity chrome). After Phase-1 clean, run read-only `codex-review`. |
| **F28 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F29 — PowerShell** | `;` not `&&`. |
| **F30 — Capture independence** | SQL + pretty only. No new events. No models required. |
| **F31 — T261 contentless** | 0-contentful still empty; no prefer-fill. |
| **F32 — Semantic** | When `--semantic --global` and preferred is Some: same merge **after** local FTS prefer-fill (semantic pool already project-aware via `project_id` None). In-memory `prefer_authority_hits` stays. Do **not** add a third HTTP round. Soft: semantic-only prefer-fill if FTS empty — not required if AC2 lexical is green. |
| **F33 — Tag collision** | T264: 8-hex collision keeps raw `[8hex]`. Reuse `unique_project_id_for_tag`. |
| **F34 — Substring fallback** | T105 small-scope LIKE **unchanged**. Prefer-fill is FTS. |
| **F35 — Existing tests stay green** | T274 `recall_pin_rank`; T259 `project_rebind_path` / `project_path_aliases`; T264 `preflight_global_isolation`; T243 search; T228 Scope; T260 stubs. |
| **F36 — No leftover UUID in SQL/help errors** | Runbook may name `7d97a456` as the live dump. New T276 `--help` / stderr **must not** recommend `set-alias 7d97 … AI-Brains` (T259 F1 / T267). |
| **F37 — Event sourcing** | No `MemoryMoved`. Path rebind remains T259 compensating pair. |
| **F38 — HashSet dedupe (Agy m2)** | `merge_preferred_then_global` tracks seen `memory_id` with `HashSet<String>` (not `Vec::contains`). A hit in both preferred and global appears **once** (preferred wins). AC1 asserts overlap → len unique. |
| **F39 — Preferred-full skip (Agy O2)** | After taking preferred, if `preferred.len() >= depth`, truncate preferred to `depth` and **return without scanning global** (T274 pass-1-full analog). Not a leftover SQL exclude (F6). AC3 applies only when remainder &gt; 0. |
| **F40 — Both arms + bridge (OpenCode m2)** | Both `lexical_search` calls set `LexicalSearchOptions { prefer_authority: true, … }` (same as today’s recall path `:288–292`). `query_ledgerful_bridge` keeps `options.project_id` (**None** when `--global`); never pass `preferred_project_id` as a bridge/SQL scope. |
| **F41 — AC3 is pre-rerank (OpenCode m3)** | “Leftover still appears” is the **merge output before** `rerank_hits`. Post-rerank top-5 may be all-owner. That is intended (label, do not drop). Comment next to the merge so a leftover-free top-5 is not filed as a drop regression. |

---

## 4. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Pure: `merge_preferred_then_global(preferred[3], global[15 chrome + 1 overlapping preferred id], depth=15)` emits preferred ids first; overlapping id **once** (`HashSet`, F38); len ≤ 15. Second case: `preferred.len() >= depth` → output is truncated preferred only, global not appended (F39) | Unit in `prefer_project.rs` |
| **AC2** | Hermetic two-project vault: leftover-like B has **15** chrome rows MATCHing needle; owner A has **one** leading `DECISION: <unique-needle>`. `recall_full` with `project_id: None`, `preferred_project_id: Some(A)`, `limit: 5` → hit **#1** is the A pin | Retrieval hermetic (required red before merge exists) |
| **AC3** | Same fixture (preferred has **1** pin, so remainder &gt; 0): at least one B chrome row appears in the **merge output before** `rerank_hits` (F41). Not a SQL drop. Post-rerank top-5 **may** be all-owner | Same hermetic |
| **AC4** | CLI hermetic `--global --format pretty --no-bridge --limit 5`: owner pin line has leading `[`+8 hex or upgraded label, **then one space**, **then** `[score=` or `[rank=#` (Agy m1 / F4). Project-scoped (no `--global`) stdout has **neither** `[`+8 hex `]` on hits | CLI hermetic |
| **AC5** | `--format json --global`: parsed object has `results[]` **without** `project_id` key; `serde` still works | CLI hermetic |
| **AC6** | `preferred_project_id: None` + unscoped: T274 chrome-monopoly AC still holds **or** current unscoped behavior (no prefer-fill crash) | Retrieval unit / T274 stays green |
| **AC7** | T274 `recall_full__chrome_monopoly__authority_pin_is_hit_one` stays green (scoped or default_opts `preferred=None`) | Regression |
| **AC8** | T259 `project_rebind_path` + T264 `preflight_global_isolation` stay green | Regression |
| **AC9** | Project-scoped `recall` (no `--global`) does **not** call prefer-fill; no tags | CLI / unit |
| **AC10** | Docs + CHANGELOG; CAPABILITIES prefer-fill + label; leftover runbook still `rebind-path` | Grep |
| **AC11** | No production `unwrap`/`expect`/`panic`; no clap/rusqlite bump; no DTO keys | Review / lock diff |
| **AC12** | Live operator vault: this planning pass ran **print-only** `list-paths --shared-only` (11 leftover). Implement **must not** `--write --yes` unless owner confirms | Manual |
| **AC13** | `POLICY_DENIED_HINT` / T275 grant-wall / T274 GLOB **byte-equal** (not stolen) | Existing units |
| **AC14** | Capture independence: recall without grants still works | Review / existing smoke |
| **AC15** | Sync pretty `--global` gets the same tags **if** `print_pretty_hits` is shared (pass preferred). No new JSON keys | Hermetic or shared-unit |
| **AC16** | New T276 help/errors do not contain `set-alias` + leftover UUID + `AI-Brains` together | Grep |

Test names (TDD). **Must fail red before merge exists:** AC2 (owner pin absent or not #1).

- `merge_preferred_then_global__preferred_first_no_dupes`
- `merge_preferred_then_global__overlap_id__once`
- `merge_preferred_then_global__preferred_fills_depth__skips_global`
- `merge_preferred_then_global__preferred_none__identity`
- `recall_full__global_prefer__owner_pin_beats_leftover_chrome`
- `recall_full__global_prefer__leftover_still_in_candidates`
- `recall_full__preferred_none__no_fill_panic`
- `recall__global_pretty__tags_project`
- `recall__scoped_pretty__no_global_tag`
- `recall__global_json__no_project_id_key`

---

## 5. Design notes

### 5.1 Prefer-fill vs T274 two-pass

T274 two-pass: authority GLOB then fill, **same** `project_id` filter.

T276: **two `lexical_search` calls** (each may two-pass):

```text
if let Some(pref) = preferred {
    scoped = lexical_search(..., project_id: Some(pref), prefer_authority: true, limit: depth)
    global = lexical_search(..., project_id: None,       prefer_authority: true, limit: depth)
    local_hits = merge_preferred_then_global(scoped, global, depth)
} else {
    local_hits = lexical_search(..., project_id: None, ...)  // today
}
rerank_hits(&mut local_hits)  // unchanged
```

Do **not** nest a third SQL pass inside `match_query`. Do **not** `AND mp.project_id != leftover`.

`merge_preferred_then_global`: `HashSet<String>` seen ids (F38). If `preferred.len() >= depth`, truncate and return (F39) — leftover absence then is the depth cap, not a filter.

COALESCE on the lexical SELECT is **orthogonal** (F15): it fills `RecallHit.project_id` so F4 can tag leftover vs owner. Dropping it would make leftover pretty-tags `[unknown]`.

### 5.2 Why not demote leftover in `rerank_hits`

T274: chrome −16 only helps if the pin **entered**. Additive leftover penalty would fight BM25 of 18k and special-case one UUID (F2). Prefer-fill is the T260 lesson.

### 5.3 Pretty tag

Reuse T264 first-line tag grammar so preflight and recall look like one product. Hit line:

```text
[3581317d] [score=-21.237 | session=26aab1e0] <uuid>: DECISION: …
```

or upgraded `[C:\dev\ai-brains]` after `display_label` + sanitize. Leftover with empty alias → `[(no alias)]` or raw `[7d97a456]` if upgrade collides — **T264 F24/F33**. Frozen (Agy m1): leading tag **before** the score bracket, **one space**, no extra blank. AC4 asserts that order.

### 5.4 Operator leftover runbook (docs only — not executed)

Unchanged T259 §5.3: `list-paths --project 7d97…` → `context` in that repo → `rebind-path --format human` → `--write --yes`. T276 adds: `--global` recall no longer requires that split to **see this repo’s unique pins**, and leftover hits are tagged.

Do **not** `set-alias 7d97a456 AI-Brains`. Do **not** rebind `C:\dev\ai-brains` off `3581317d`.

### 5.5 `fcb8a40f`

Not leftover. Prefer-fill uses **cwd effective project**. An agent in `C:\dev\ledgerful` with `.env` = path owner gets ledgerful pins into `--global` even if leftover 18k exists. Daily Scope leftover vs path `fcb8a40f` is **T258 adopt-path** in that tree. Shell leftover display is **T282**.

---

## 6. Non-goals

- Silent exclude leftover (or any project) from `--global`.
- `--exclude-project` / `--from-path-owner` clap flags.
- Moving, classifying, or CE-wiping leftover memories.
- Live leftover `--write --yes` without owner confirm.
- Writing `.env` / T240 F2 / T258 adopt-path.
- `project list` sort (T283); `context --show` shell (T282).
- Retuning T274 chrome / T218 floors / `candidate_depth`.
- Editing `ranking.rs` / `project.rs` / CLI `preflight.rs` / `POLICY_DENIED_HINT`.
- New DTO keys; clap 5; rusqlite 0.40; new crates.
- `cargo install`; transcoding mojibake; doctor 16th check.

---

## 7. Verification plan (TDD)

**Phase 1 red (required before green):** AC2 retrieval hermetic (owner pin not #1 today with `preferred` ignored). AC1 unit can go red on missing helper.

Then green: SELECT COALESCE project_id (tags) → merge helper (F38/F39) → `recall_full` second `lexical_search` (F40) → CLI preferred wiring → pretty tags (F4 order).

**Stay green:** AC6–AC9, AC13, T274, T259, T264, T243, T228.

Targeted: `cargo nextest run -p ai-brains-retrieval --test recall_pin_rank --test recall_global_prefer` ; `-p ai-brains-cli --test recall_global_prefer --test project_rebind_path --test preflight_global_isolation` ; `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings`.

Full workspace gate only at implement closeout — **not** a plan gate.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Operators think T276 moved 18k pins | F7 / docs honesty; list-paths still shows leftover |
| Silent exclude sneaks in | F6 / AC3 leftover still in candidates |
| Hardcoded leftover UUID | F2 / AC grep |
| `project.rs` / `sync.rs` growth | F16/F18/F19; one field on sync |
| T274 two-pass broken | F12; AC7 |
| PATH-behind T274 | F22; hermetic/source bin |
| Double MATCH cost | Two depth-15 FTS queries; no depth raise |
| Pretty tag vs T264 collision | Reuse F33 helper |
| Owner wants live rebind | F9 — confirm at go; not this planning pass |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-21 (post-P12 through T275 closeout).

| Row / leftover | Disposition |
|----------------|-------------|
| leftover `7d97a456` ~18k / `--global` junk | **Absorb** F1–F6 / AC1–AC5 / AC12 |
| T264 recall leftover-first / “filter flag not silent exclude” | **Partial:** prefer-fill + label **absorb**. `--exclude-project` **decline F20** |
| T259 leftover memory reclassify by path | **Decline F7** — soft residual |
| Live leftover 11 paths still on `7d97` | **Partial:** document; live write **F9** Stop-Before. Prefer-fill does not require rebind |
| T270 closeout leftover 18k | **Absorb** as this track |
| Identity mismatch quiet / `7d97` vs `fcb8a40f` | **Partial:** leftover volume **this track**. adopt-path **T258** (cwd already false). Shell **T282**. No T285 |
| T267 footer leftover-as-AI-Brains | **Decline** — T267 Completed F3 |
| `project list` leftover-first | **Decline → T283** |
| `context --show` leftover shell | **Decline → T282** |
| briefing/progressive 0 of 3 grants | **Decline → T275 Completed** (live apply still owner-confirm) |
| recall session dumps over pins | **Decline → T274 Completed** (F12) |
| Preflight Safety = Objective | **Decline → T279** |
| deny hint `--scope …` | **Decline → T280** |
| last-PR Cursor #190 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Affirm T284** — 2 inline comments still; no T285 |
| T240 F2 / T255 750 ms / clap 5 / rusqlite 0.40 | **Decline** |
| Closed/strikethrough T259/T264 leftover-split / blender | **Stay closed** — this track is the **recall** residual they pointed at |
| Packaging / MSI / `.changeguard` | **Decline** |

---

## 10. Implement order (on go)

1. Phase 0 re-verify T112 clear `:4322`, clap env `:1017` after force-set `:3268` / parse `:3356`, lexical SELECT `:259`, `format_pretty_hit_line` `:407`, leftover 11 roots, #190 empty, #188 T284.
2. Red: AC2 hermetic + AC1 merge unit.
3. Green: `RetrievalMemory`/`RecallHit.project_id` + `prefer_project.rs` + `recall_full` second search.
4. CLI preferred wiring + pretty tags (AC4/AC5). Sync one field (AC15).
5. Docs F24. No ranking.rs / project.rs / leftover `--write`.
6. Review loop + FEATURE `codex-review` + full gate. implement-track Phase 6.

---

## 11. Soft residuals

| Residual | Why not DoD |
|----------|-------------|
| `--exclude-project` clap flag | F20; prefer-fill is the default |
| Reclassify leftover memories onto dest | T259 F5; later importer if ever |
| Live leftover 11-root rebind | F9 operator out of band |
| Ranking leftover penalty | F14; pin-must-enter |
| Semantic-only prefer-fill e2e | F32; lexical AC2 is the hole |
| Mojibake transcode | Append-only |
| PATH `cargo install` | F22 |
| `display_label` extract out of `project.rs` | Soft — call only |
| JSON `project_id` on `RecallResult` | F5 T180 freeze |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-retrieval/src/prefer_project.rs` | **New** merge (F38 HashSet, F39 skip) + units AC1 |
| `crates/ai-brains-retrieval/src/lexical.rs` | SELECT `COALESCE(mp.project_id, sp.project_id)`; `RetrievalMemory.project_id` |
| `crates/ai-brains-retrieval/src/recall.rs` | `RecallOptions.preferred_project_id`; `RecallHit.project_id`; `recall_full` merge |
| `crates/ai-brains-retrieval/src/lib.rs` | `mod prefer_project` |
| `crates/ai-brains-retrieval/tests/recall_global_prefer.rs` | **New** AC2/AC3/AC6 |
| `crates/ai-brains-cli/src/main.rs` | Thread pre-clear preferred when `--global` |
| `crates/ai-brains-cli/src/commands/recall.rs` | `preferred_project_id`; pretty tag arg / dispatch |
| `crates/ai-brains-cli/src/commands/recall_global.rs` | **Optional new** tag attach (keep `recall.rs` from growing if helper is cleaner) |
| `crates/ai-brains-cli/src/commands/sync.rs` | **One field** `preferred_project_id` on `RecallOptions` |
| `crates/ai-brains-cli/tests/recall_global_prefer.rs` | **New** AC4/AC5/AC9/AC15 |
| `Docs/CAPABILITIES.md` / `CHANGELOG.md` / `OPERATIONS.md` | F24 |
| `conductor/conductor.md` / `deferred.md` / this folder | Registry + absorb notes |

Do **not** touch: `project.rs` (footer/hotspot), `ranking.rs`, CLI `preflight.rs`, `doctor.rs`, `governed_common.rs`, `project_rebind.rs` (except docs), contracts `RecallResult`, migrations, live `.env`, live path aliases.

---

## 13. AI fold-in disposition (2026-08-21)

Sources: `agy-review.md` + `opencode-review.md` (HEAD `61fd3cb`). **B 0 / M 0.** Review files are inputs — **do not edit**.

### Agy

| ID | Verdict | Action |
|----|---------|--------|
| **m1** tag before `[score=]` / one space | **Agree** | **F4 / AC4** — leading tag, one space, then score/rank bracket |
| **m2** HashSet dedupe in merge | **Agree** | **F38 / AC1** + unit `merge_preferred_then_global__overlap_id__once` |
| **O1** `format_pretty_hit_line` `project_tag: Option<&str>` | **Already** | **F18** — shared with `print_pretty_hits` / sync |
| **O2** skip global when preferred fills depth | **Agree** | **F39** + AC1/AC3 remainder condition |

### OpenCode

| ID | Verdict | Action |
|----|---------|--------|
| **m1** two `lexical_search` not SQL prefer-fill; drop COALESCE | **Partial** | Fill route **already F1**. COALESCE **kept F15** for F4 tags (mig 0015 `mp.project_id`). Do not drop the column. |
| **m2** `prefer_authority: true` both arms; bridge `project_id` None | **Agree** | **F40** + Phase 0 |
| **m3** AC3 is pre-rerank; top-5 may be all-owner | **Agree** | **F41 / AC3** |
| **O1** empty-hint still says `Try --global` | **Decline** | Live `build_recall_hint_core` **`:675–683`**: global arm is “across all projects”, not `Try --global`. Re-trigger: if that arm grows a `--global` suggestion. |
| **B2** (summary) pre-clear clap id | **Already F2/F17** | Phase 0: force-set `:3268` → parse `:3356` → clear `:4322` |
| HEAD / counts / line undercount | **Agree snapshot** | Fold HEAD `61fd3cb`; vault totals volatile; `Measure-Object -Line` CRLF undercount — hotspot rank still holds |
| `display_label` at CP `briefings/project.rs:383` | **Decline citation** | Live is CLI `project.rs:383`. CP has no `display_label`. |
| leftover UUID `7d97a51a` | **Decline typo** | Live leftover is `7d97a456-f2f4-43ea-1f13-211af684ad37` |

### Declined / not new design

| Item | Why |
|------|-----|
| Drop lexical COALESCE | F15/F4 — tags need `RecallHit.project_id` on leftover hits |
| Empty-hint rewrite | Live already honest when `global == true` |
| `--exclude-project` / memory move / live rebind / T240 F2 | Unchanged F20 / F7 / F9 / F8 |
| last-PR #190 Cursor | Still N/A empty; **no T285** |
| serde_json 1.0.151 / clap 4.6.6 / rusqlite 0.40.2 | **No bump** (F21) |

### Pins locked by fold-in

1. **F4 / AC4:** tag, one space, then `[score=` / `[rank=#`.
2. **F38 / AC1:** `HashSet<String>` seen ids; overlap once.
3. **F39:** preferred-full skip; AC3 only when remainder &gt; 0.
4. **F15:** COALESCE stays for tags; prefer-fill is two `lexical_search` (F1).
5. **F40:** `prefer_authority: true` on both arms; bridge uses `project_id` None.
6. **F41:** AC3 is pre-`rerank_hits`.
7. **F18:** `project_tag: Option<&str>` on `format_pretty_hit_line` (Agy O1 already).
8. **§2.1 / F17:** HEAD `61fd3cb`; clap parse after `.env` force-set.
