# T270 — Retention plan must inventory live `memory_legacy` rows

- **Track ID:** T270-RetentionLiveClassification
- **Status:** **Completed**
- **Category:** FEATURE / UX / HONESTY
- **Owner:** Grok
- **Source:** Audit 2026-08-16 — `retention plan` **6/5**; 0 candidates across ~35k memories (live 2026-08-20: **38,208** pinned)
- **Depends on:** T166 ✅ class matrix; T167/T168 classify_legacy (do **not** call); T248 ✅ TTY human
- **Blocks / feeds:** Operators can believe `Nothing to dispose.` because the matrix shows the vault’s pins as **held**. Does **not** unblock leftover-identity mismatch quiet, T273 F7 `bridge_search_args`, or nightly CE.
- **Absorbs:** deferred.md “`retention plan` 0 candidates on 35,300 memories”; placeholder F1–F4; T166 §5.1.5 stream-A `memory_legacy` scan that was never coded; T248 empty-check lift (`Nothing to dispose.` = no **dispose** work)
- **Not absorbed (DoD):** `retention apply` mutation; CE wipe; `classify_legacy` / `migrate governed`; auto-forget pins; T166 horizon retune; T248 format tokens / apply JSON default; doctor 16th; clap 5 / pin bumps; contracts new keys; leftover `7d97a456` vs `fcb8a40f` mismatch (T240/T257/T258)
- **Research date:** 2026-08-20 (plan dogfood HEAD `fdd4924` T272 `#187`; fold-in against `70d61cd` — docs-only; product `src/` identical)
- **AI fold-in:** 2026-08-20 `agy-review.md` + `opencode-review.md`. **B 0 / M 0.** **Agree:** Agy m1 forgotten-only sample SQL (AC1/AC4); Agy m2 `classes` sort after merge (F30 / AC17); Agy O1 notes const (F31). **Already covered:** Agy O2 SQL `LIMIT 5` (F5 / AC16); OpenCode O4 `ISSUES.md` (F24); OpenCode SOOT `:755/:771` is `collect_candidates` — merge stays **after** `build_report` (F6). **Fold snapshot:** OpenCode m HEAD `70d61cd`; OpenCode m nightly `:511–535`; OpenCode O pin-count / hotspot / search line drift. **Decline:** OpenCode deferred-table misread that F8 lifts to `totals.candidates == 0` — F8 is **dispose-work** (`ce_wipe + projection_delete`). Disposition **§13**.
- **Ledger:** planning DOCS TX `3ebebd1f-58e1-4663-b559-75f900edfc95`. Fold-in DOCS TX `56696e5a-9104-46c6-9313-447d2bacb7d1`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** run live `retention apply --confirm`. Do **not** call `classify_legacy` / `migrate governed`. Do **not** `cargo install`, rewrite `.env`, pin-as-implement to the live vault, or mutate schtasks. Do **not** grow hotspot `project.rs` / `sync.rs` / `preflight.rs`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **The live vault is visible.** `retention plan` on a vault with `memory_projection` rows must show non-zero `memory_legacy` counts (pinned → **held**, other statuses → **skip**), not a blank `skip 0` next to `Nothing to dispose.`
2. **“Nothing to dispose.” stays true.** That sentence means **no CE wipe and no projection delete**, not “we did not look.” Pins are inventory, not work.
3. **Plan is still a query.** Overlay is COUNT + ≤5 truncated sample ids. No events on `plan`. `memory_legacy / none_auto` never auto-forgets. Apply still JSON + `--confirm`.
4. **North star.** Capture independence: SQL counts only. Append-only log unchanged by plan. Operators (and agents) can read the class matrix as a retention **schedule plus inventory**, not a silent empty report on a 38k-pin vault.

This unblocks daily honesty for class-based retention: T248 made the matrix readable; T166 never scanned memories; the audit scored 6/5 because “nothing to dispose” on tens of thousands of pins is a product lie.

---

## 2. Live baseline (re-scan 2026-08-20)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | **Plan dogfood:** `fdd4924` T272 `#187` (product `src/` unchanged since). **This fold-in:** `70d61cd` (docs-only planning commit; **ahead 1** of `origin/main`). Product tree identical. Tree **CLEAN** at fold-in. |
| PATH `ai-brains` | **0.1.1**. `retention plan --format human` is the T248 matrix. **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic / PATH (PATH already has T248). |
| PATH `retention plan --format human` | `Nothing to dispose.` `memory_legacy none_auto skip 0`. Totals all **0**. Four honesty shorts. **No** inventory of pins. **Live hole confirmed.** |
| PATH `retention plan --format json` | `api_version=1`, `classes=[]` (len **0**), `totals.candidates=0`, **4** warnings. |
| `memory list --summary --global` | **Pinned 38,208** at plan (OpenCode review: **38,210** — volatile; Forgotten **29**). Leftover `7d97a456` still **18,035** pinned. Path `C:\dev\ledgerful` is `fcb8a40f` (**4,753**). This cwd path owner `3581317d` (**3,255**). |
| `project whoami` | `mismatch: false`. Effective/path/detect `3581317d`. Shell leftover `7d97a456` overridden by local `.env`. |
| Last GitHub PR | [#187](https://github.com/Ryan-AI-Studios/AI-Brains/pull/187) T272 (2026-08-20). Issue comments **0**, reviews **[]**, inline **[]**. **last-PR Cursor: N/A (empty).** Open PRs: Dependabot remotes only (not this HEAD). **No T274 from #187.** |
| Identity / doctor ambient | Scope `3581317d`; discovery grants 0 of 3 (T241); ledgerful doctor leftover `.changeguard` / sig-pin / timings. 0 pending / 0 drift at plan scan. Do not “fix” here. |
| Preflight summary | project=`C:\dev\ai-brains` `3581317d`; pinned in-context **3255**. |

### 2.2 Why 0 candidates is a lie

| Layer | Truth |
|-------|--------|
| T166 §5.1.5 | Stream A: “Memory aged policy (v1 none) → `memory_legacy`; **skip if pinned**.” **Never coded.** `collect_candidates` scans turns, query traces, closed reviews, disposable decisions, envelopes — **not** `memory_projection` as a class. |
| T166 R11 | Dry-run **may** list pinned as `skip` / `held`. Envelope pin-hold is implemented (`list_pinned_memory_ids` + stream B). Standalone pins with **no** envelope never appear. |
| T166 §11 non-goal | “Classifying entire legacy vault without operator review (T167 import is separate).” That forbids **governed import** / `classify_legacy` as retention DoD — **not** a COUNT overlay. |
| `retain` in `collect_candidates` | Drops `mechanism == skip` unless class is `unclassified`. A naïve per-row `memory_legacy` skip scan would be thrown away. Overlay must merge **after** that retain (or use held/skip totals without going through that filter as noise). |
| T248 pretty | Zero-row `memory_legacy` mechanism is **`skip`**. `Nothing to dispose.` when `classes.is_empty() \|\| totals.candidates == 0`. Work table prints **every** non-zero class. If we add 38k held into `candidates` without lifting those two rules, the human path either keeps the lie **or** labels pins as **Work**. |
| T167 `classify_legacy` | Walks **event envelopes** into a governed `ImportPlan` (Evidence / skip / …). Not retention `content_class`. Not a plan-time remediator. Live `migrate governed` is mutating + T170 stop-before. |
| Nightly | Already calls `plan_retention` and `eprintln!` totals (T166 F-004). Overlay flows through automatically. Do **not** restyle the line. |

Placeholder F2 (“honesty sentence **or** overlay”) is **resolved at this plan:** **overlay (COUNT + samples)** plus a named honesty warning. A sentence without counts still looks like T248 chrome on a 38k vault.

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Plan CLI | `commands/retention.rs` `run_plan` | `plan_retention` + `emit_report`. Read-only. |
| Pretty | `format_retention_pretty` `:403` | Empty check `:417`. Work table `:422–438` all non-zero classes. Matrix fills zeros via `zero_row_mechanism` (`memory_legacy` → `skip`). |
| Honesty shorts | `honesty_short_label` `:360` | Known T166/T248 constants; unknown echoed. |
| Engine | `class_based_retention.rs` `plan_retention` `:234` / `collect_candidates` `:269` (OpenCode search also `:239/:240/:990`; prepare call `:771`) / `build_report` `:604` / `prepare_retention_apply` `:755` | `collect_candidates` also used by apply prepare + in-process `apply_retention`. **SOOT:** merge inventory **after** `build_report` in **one** assemble helper used by plan **and** prepare. Do **not** push inventory rows into `collect_candidates` (F5). |
| Stream A SQL | `store/projections/retention.rs` | `list_old_turns`, traces, reviews, decisions. `list_pinned_memory_ids` is R11 envelope hold only. **No** memory COUNT helper. |
| Fixture insert | CP test `insert_memory` | Columns `memory_id, content, privacy, status, level, created_at, updated_at`. Statuses used: `pinned`, `active`. |
| Contracts | `ai-brains-contracts/src/retention.rs` | `CLASS_MEMORY_LEGACY`, horizon `"none_auto"`, `api_version` **1**, sparse `classes`. **No** new required keys. |
| Empty JSON hermetic | `tests/retention_plan_human.rs` `retention_plan__format_json__frozen_keys_empty_classes` | Empty vault `classes` may be `[]`. **Keep.** |
| Empty pretty unit | `format_retention_pretty__empty__nothing_to_dispose_matrix_skip_no_next` | Zero-row `memory_legacy` is **skip not held**. **Keep** (empty vault has no overlay). |
| Existing R11 | `retention_plan__pinned_memory__held` | Envelope hold. Overlay is **additive** (`would_held >= 1` stays). |
| Empty CP | `retention_plan__empty_vault__zero_counts` | Only exact `candidates == 0`. Stays if overlay is zero on empty projection. |
| clap Plan | `main.rs` `RetentionCommands::Plan` `:2283` | `--format` default `auto`; `after_help` TTY + json examples. Additive honesty. |
| Nightly log | `nightly.rs` `:511–535` (plan cited `:509–531`; OpenCode m2 — import `:511`, call `:514`, totals eprintln `:524–531`, fail `:533–535`) | `plan_retention` totals. Untouched format. |
| Hotspots | `project.rs` **#1** (plan 4.008; OpenCode review 3.999 — still #1); `sync.rs` #2; `preflight.rs` #7 | **Do not touch.** `class_based_retention.rs` **1204** lines; CLI `retention.rs` **902**; store `retention.rs` **450** — not top-10. Helpers stay in those files (no new crate). |
| Apply | `prepare_retention_apply` match (`run_apply` `:146`) | `held`/`skip` already no-op. Inventory must **not** invent `turn` / `content_key_id`. |

### 2.4 Dependency / standards research (2026-08-20)

**Snapshot — re-verify at execute.**

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). **No clap 5.** | **No bump.** Additive `after_help` only. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** JSON keys frozen. |
| `chrono` | lock **0.4.44** | crates.io **0.4.45** (Dependabot #62 open) | **No bump.** |
| `rusqlite` | lock **0.39.0** | crates.io **0.40.2** (Dependabot #61; T213 L4 `table_exists`) | **No bump.** Plain `COUNT(*)` / `LIMIT 5`. |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | — | **Zero.** No `comfy-table`. |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Deletion must be recorded against a **schedule** | [ISO 27001:2022 A.8.10](https://knowledge.adoptech.co.uk/a.8.10-information-deletion) (implementation guidance: record deletion; retain only as long as necessary) | Plan is the pre-delete review record. Inventory makes “retain pins / none_auto” **visible**. Not a compliance product. |
| Indefinite store without a documented trigger fails audit | [A.8.10 checklist — retention period alignment](https://hightable.io/iso-27001-annex-a-8-10-audit-checklist/) | `none_auto` **is** the trigger (never auto). Showing `held 38208` documents that choice. Zero-count hides it. |
| Classification drives disposal | ISO 27001 A.8.2 practice (classification → how long / how dispose) | Overlay **classifies for the report** (`memory_legacy` + held/skip). It does not promote pins to governed Evidence (T167). |
| Dry-run prints policy even when nothing is due | restic `forget --dry-run` (T248 research); GDPR Art. 5(1)(e) category limits (T166) | Keep `Nothing to dispose.` + full matrix. Add counts. |
| clap `after_help` | [docs.rs/clap/4.6.6 `Command::after_help`](https://docs.rs/clap/4.6.6/clap/struct.Command.html) | Keep derive `after_help = "…"`. No clap 5. |

**N/A:** SQLCipher page crypto, schtasks, T180 preflight 2-key DTO, Windows service, llama.cpp `/health`.

**Could not verify:** live `turn_projection` row count (no CLI). `raw_turn` COUNT **0** on this vault is enough — T270 does not retune the 90-day turn horizon.

**ledgerful / ai-brains:** `preflight --summary`; `whoami` mismatch false; `memory list --summary --global` 38208; `retention plan` human+json hole; `ledgerful doctor` (5 warn, work root this repo); ledger 0 pending / 0 drift; `index --incremental`; `search --json -- "collect_candidates"` hits `class_based_retention.rs:234/:269/:771/:990`; `scan --impact` CLEAN at `fdd4924`; `hotspots` project.rs #1 (do not grow); `recall` T248 review memories (matrix shipped; overlay not). Semantic `ask` not required (search hit).

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `3ebebd1f`. Fold-in is DOCS TX `56696e5a`. Implement starts a **FEATURE** TX. |
| **F1 — Overlay, not migrate** | Read-only **inventory overlay**: `COUNT` of `memory_projection` by status + ≤**5** truncated sample ids. **Do not** call `classify_legacy`, `migrate governed`, or walk the event log. T167/T168 stay importers. |
| **F2 — Held vs skip** | `status = 'pinned'` → `held` (R11). Any other status (`forgotten`, `active`, …) → `skip` (`none_auto`). **Never** `soft_forget` / `projection_delete` / `ce_wipe` from this overlay. |
| **F3 — none_auto stands** | No auto-forget of pins. Apply loop already no-ops `held`/`skip`. Overlay rows have **no** `turn` / `content_key_id`. |
| **F4 — Apply still gated** | `retention apply` default JSON + `--confirm`. Do **not** live-apply as this track’s dogfood. Prepare **does** merge the same overlay so apply JSON matches plan (held counts in `RetentionApplied` if someone applies). |
| **F5 — Do not materialize N candidates** | Store helper returns `{ pinned: u64, other: u64, sample_ids: Vec<String> }` via `COUNT` + SQL `ORDER BY memory_id LIMIT 5` (Agy O2 — **not** fetch-all then slice in Rust). Sample query: if `pinned > 0` then `status = 'pinned'`; else `status != 'pinned'` (Agy m1). **Forbidden** to push one `Candidate` per memory into `collect_candidates` (38k no-op apply loop / retain filter). |
| **F6 — Assemble SOOT** | `plan_retention`, `prepare_retention_apply`, and in-process `apply_retention` all: `collect_candidates` → `build_report` → `merge_memory_legacy_inventory`. One helper. Merge is **after** `build_report`, not inside `collect_candidates` (OpenCode `:771` is the prepare *call* to collect, not the overlay). Stream B never emits `memory_legacy` today; if it ever does, **merge into the existing bucket** (at most one `memory_legacy` class). |
| **F7 — JSON keys frozen** | `api_version` **1**. No new required keys on `RetentionPlanReport`. Overlay is a normal `classes[]` bucket + totals + an extra `warnings[]` string when inventory &gt; 0. Empty vault stays `classes: []` / candidates 0 (T248 AC8 / CP empty test). |
| **F8 — Pretty empty-check lift** | `Nothing to dispose.` iff `would_ce_wipe + would_projection_delete == 0` (not `candidates == 0`). Inventory-only vaults still print it. T248 empty-vault unit stays green (no overlay). |
| **F9 — Work table is dispose-only** | Human `Work` lists classes whose mechanism is `ce_wipe` or `projection_delete` only. `held`/`skip` inventory is **Class matrix** (and totals), never Work. No `next:` when dispose-work is 0 (T248 F11 already). |
| **F10 — Honesty constant** | New contracts string `RETENTION_HONESTY_MEMORY_LEGACY_INVENTORY` = `memory_legacy inventory is none_auto; pins held; apply does not auto-forget`. Appended only when inventory total &gt; 0. Human short: `memory_legacy inventory ≠ auto-forget`. Unknown warnings still echo. |
| **F11 — Samples / plaintext** | Sample cells: existing `truncate_id` / `truncate_sample_ids` (max 5). **Never** memory `content` / bodies in JSON or pretty (existing R11 body test stays). |
| **F12 — Plan writes nothing** | Hermetic: event-log COUNT (or equivalent) unchanged across `retention plan`. No `RetentionApplied` on plan. |
| **F13 — Capture independence** | SQL + pretty. No models, graph, embeddings, or ledgerful on this path. |
| **F14 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.1**. rusqlite stays **0.39.0**. |
| **F15 — Contracts DTO** | No new fields. No `api_version` bump. Additive warning const + optional class bucket only. |
| **F16 — PATH** | Do not `cargo install` unless the user asks. |
| **F17 — Stop-before live apply** | Even after go: no `retention apply --confirm` on the live vault. No CE. No horizon retune. No nightly CE opt-in. |
| **F18 — Decline T167 remediator** | `next:` is **not** `migrate governed`. Inventory **is** the remediator. |
| **F19 — Decline identity mismatch track** | Agent observation Scope `7d97a456` vs path `fcb8a40f` is T240/T257/T258 (`adopt-path`) + leftover T259. This cwd `whoami` `mismatch: false`. **No T274** this pass (not last-PR Cursor; owner did not `/plan-track` the T242 analog). |
| **F20 — Decline doctor / HTTP / desktop / nightly restyle** | T248 F16/F12. Nightly `eprintln!` totals pick up overlay automatically — leave the format. |
| **F21 — Decline T273 F7 / leftover recall drop / T240 F2** | Peers / standing. |
| **F22 — Tests** | Naming `function_or_feature__condition__expected_result`. rstest `#[case]` for status→mechanism. No `unwrap`/`expect`/`panic` in production. `TempEnv` if tests touch `AI_BRAINS_RETENTION_*`. |
| **F23 — Cross-model** | Honesty UX + apply no-op. After Phase-1 review clean, run read-only `codex-review`. |
| **F24 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F25 — last-PR Cursor** | #187 empty → N/A. No mint. |
| **F26 — after_help additive** | Keep T248 examples. Add: `memory_legacy` is inventory (`none_auto`); pins held; plan does not forget. |
| **F27 — Docs** | CAPABILITIES T248 row additive sentence; OPERATIONS `memory_legacy` inventory vs skip-zero; PROTOCOL-COMPAT §5: live vaults may emit a `memory_legacy` bucket (keys unchanged; human path still not a wire contract); root CHANGELOG T270 row. |
| **F28 — Existing tests** | Empty-vault CP + T248 pretty/JSON hermetics stay green. Envelope R11 `would_held >= 1` stays (overlay additive). Do not weaken body-plaintext asserts. |
| **F29 — CLI file growth** | Pretty lift + honesty map in `retention.rs`. Engine merge in `class_based_retention.rs`. SQL in `projections/retention.rs`. **Do not** grow `project.rs`. |
| **F30 — `classes` sort after merge** | After upsert, sort `report.classes` by `class` (byte/`str` order, same as `build_report`'s `BTreeMap`). Do **not** leave `memory_legacy` appended after `raw_turn`. Human matrix still iterates `CANONICAL_CLASSES` (order unchanged). JSON `classes[]` must be deterministic (AGENTS.md sort emitted collections). |
| **F31 — Notes const** | `pub(crate) const NOTE_MEMORY_LEGACY_INVENTORY: &str = "inventory overlay; none_auto; pinned held (R11); other skip";` in `class_based_retention.rs` (Agy O1). Merge notes use this string. Tests assert it. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Store unit: fixture 3 pinned + 2 `active` + 1 `forgotten` → inventory `pinned=3`, `other=3`, `sample_ids.len()==3` (pinned, sorted, truncated). **Second case (Agy m1):** pinned=0 and other&gt;0 → `sample_ids.len()==min(5, other)` from `status != 'pinned' ORDER BY memory_id ASC LIMIT 5`. Empty table → zeros / empty samples. **No** full-id dump. Helper SQL contains `LIMIT 5` (not Rust `.take(5)` on an unbounded SELECT). |
| **AC2** | CP: empty vault still `totals.candidates == 0` and `classes` empty (existing test). |
| **AC3** | CP rstest `#[case]`: only-pinned → `memory_legacy` mechanism `held`, `would_held == N`, `would_ce_wipe == 0`, `would_projection_delete == 0`, warning const present, JSON has **no** pin body. |
| **AC4** | CP: only-`forgotten` (or only-`active`) → mechanism `skip`, `would_skip >= N`, `would_held == 0`, still no dispose mechanisms. `sample_ids` non-empty when N&gt;0 (Agy m1 fallback). Notes contain `NOTE_MEMORY_LEGACY_INVENTORY` (F31). |
| **AC5** | CP: mixed pinned+other → one `memory_legacy` bucket, `totals.candidates == pinned+other`, `would_held`/`would_skip` split, dominant mechanism `held` if pinned &gt; 0. |
| **AC6** | Unit: `format_retention_pretty` on inventory-only report (`would_held=3`, ce=0, pd=0) contains `Nothing to dispose.`, matrix `memory_legacy` count **3** and `held`, honesty short `memory_legacy inventory ≠ auto-forget`, **no** `Work` header, **no** `next: ai-brains retention apply`. Exact Totals line includes `held=3`. |
| **AC7** | Existing T248 empty pretty unit still: `Nothing to dispose.`, zero-row `memory_legacy` **skip** (not held), Totals all zeros, no next. |
| **AC8** | Existing T248 JSON hermetic empty vault: frozen keys, `classes` `[]` or candidates 0. |
| **AC9** | Hermetic: init vault, `pin` one line, `retention plan --format human` contains `held` and a **non-zero** `memory_legacy` count and `Nothing to dispose.` Exit **0**. `--format json` has `classes` with `memory_legacy`, `totals.would_held >= 1`, `api_version == "1"`. |
| **AC10** | Hermetic: same vault, COUNT events (or `memory list --summary` pinned) **unchanged** after `retention plan` (F12). |
| **AC11** | Hermetic: `retention apply` without `--confirm` still `INVALID_PAYLOAD` (T248 AC11). Overlay must not swallow the refuse. |
| **AC12** | Clap: `retention plan --help` after_help contains `none_auto` or `inventory` (F26). Invalid `--format xml` still exit **2**. |
| **AC13** | Manual (source or PATH): live `retention plan --format human` shows `memory_legacy` COUNT matching **pinned ≥ 1** (volatile ~38208) **and** `Nothing to dispose.` **and** no `next: apply`. JSON `totals.would_held >= 1`. Do **not** apply. |
| **AC14** | Docs: CAPABILITIES + OPERATIONS + PROTOCOL-COMPAT §5 + CHANGELOG T270 |
| **AC15** | Existing `cargo nextest run -p ai-brains-control-plane class_based_retention` + CLI retention units + `retention_plan_human` stay green (F28) |
| **AC16** | No `Candidate` per memory: store helper source contains SQL `LIMIT 5` and `COUNT` (Agy O2 — not unbounded SELECT + Rust slice). Review/grep DoD. |
| **AC17** | CP (regression guard, not Phase-1 required red): old `raw_turn` + ≥1 pinned memory → `classes` sorted by `class`; `memory_legacy` index &lt; `raw_turn` index. F30. |

---

## 5. Design notes

### 5.1 Human layout (inventory-only, after)

```text
Retention plan (dry-run)  generated YYYY-MM-DD HH:MM UTC

Nothing to dispose.

Class matrix
CLASS              HORIZON                              MECHANISM          COUNT
raw_turn           90d                                  projection_delete      0
…
memory_legacy      none_auto                            held               38208
…
unclassified       skip_apply                           skip                   0

Totals  candidates=38237 ce_wipe=0 projection_delete=0 skip=29 held=38208

Honesty
  projection delete ≠ CE
  not NIST Purge/Destroy
  stream A and B independent until subject join
  ticket / soft forget ≠ CE
  memory_legacy inventory ≠ auto-forget
```

(`skip`/`held` split is live-volatile; forgotten 29 + any `active`.)

When **also** old turns exist, `Nothing to dispose.` is omitted, `Work` lists **only** `raw_turn` (etc.), matrix still shows `memory_legacy held`.

### 5.2 Merge algorithm

```text
inv = COUNT pinned + COUNT status<>pinned
     + SQL LIMIT 5 ids: pinned if pinned>0 else status != 'pinned'
       ORDER BY memory_id ASC
if inv.total == 0: return report
totals.candidates += inv.total
totals.would_held += inv.pinned
totals.would_skip += inv.other
mechanism = held if pinned > 0 else skip
upsert classes memory_legacy { candidate_count, mechanism, sample_ids, notes: NOTE_MEMORY_LEGACY_INVENTORY }
warnings.push(RETENTION_HONESTY_MEMORY_LEGACY_INVENTORY) if missing
sort classes by class (F30)
```

Notes SOOT: `NOTE_MEMORY_LEGACY_INVENTORY` (F31).

### 5.3 Why not per-row candidates

`list_pinned_memory_ids` already loads all pinned ids for envelope R11 — pre-existing, not this DoD. Inventory **must not** clone that into 38k `Candidate` structs (apply prepare would iterate them; `retain` would drop skips). COUNT + LIMIT 5 is the overlay.

### 5.4 Nightly

`[Nightly] Retention class dry-run: candidates=38237 … held=38208 (no apply)` becomes true (`nightly.rs` `:511–535`). Leave the format (F20). Soft residual: operators may misread `candidates` as dispose-work — honesty warning + human `Nothing to dispose.` are the remediator on the CLI; nightly is a one-liner.

---

## 6. Non-goals

- Live `retention apply --confirm` / CE wipe / nightly CE
- `classify_legacy` / `migrate governed` / pin→Evidence (T263 H2 declined)
- Auto-forget / soft_forget of `memory_legacy`
- Retuning raw_turn 90d / other horizons
- JSON zero-count buckets for the other eight classes
- Doctor `retention_plan` check / HTTP / desktop UI
- T240 F2 silent Scope switch / adopt-path / leftover rebind
- T242-style cross-process identity-mismatch quiet (**no T274** this pass)
- clap 5 / rusqlite 0.40 / chrono 0.4.45 / new crates
- Growing `project.rs` / `preflight.rs` / `sync.rs`
- Restyling nightly `eprintln!`

---

## 7. Verification plan (TDD)

**Red first (must fail on product HEAD `fdd4924`):**

1. AC1 store inventory helper (module does not exist / counts 0) — include pinned=0 other&gt;0 sample case (Agy m1).
2. AC3 CP pinned-only → `memory_legacy` held (today no class).
3. AC6 pretty inventory-only → `Nothing to dispose.` **and** held count (today empty-check uses `candidates==0`; Work would list held if we only merged JSON).
4. AC9 hermetic pin + plan (today `skip 0`).
5. AC10 plan does not append (green even now — keep as regression).
6. AC12 after_help needle (today missing).
7. AC17 class-sort is a **green-phase guard** (OpenCode/T272 analog): not required red until merge exists.

**Then green:** store helper, merge helper, pretty lift, honesty map, after_help, docs.

**Targeted (not full workspace as plan gate):**

```powershell
cargo nextest run -p ai-brains-store -p ai-brains-control-plane -p ai-brains-cli -p ai-brains-contracts --lib --bins
cargo nextest run -p ai-brains-cli -E "test(retention_plan)"
cargo clippy -p ai-brains-store -p ai-brains-control-plane -p ai-brains-cli -p ai-brains-contracts --all-targets -- -D warnings
```

On go finalize: full AGENTS.md gate + `ledgerful verify --scope full`.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Pretty shows 38k pins as **Work** | F9 dispose-only Work table |
| `Nothing to dispose.` vanishes because `candidates>0` | F8 lift to dispose-work |
| Apply iterates 38k no-ops / slow plan | F5 COUNT+LIMIT 5 |
| `classify_legacy` / migrate as fake remediator | F1 / F18 |
| JSON scripts assumed live `classes=[]` | Honesty: that **was** the bug; keys frozen; `api_version` 1; PROTOCOL-COMPAT note |
| Envelope R11 tests break on exact totals | Assert `>=`; overlay additive (F28) |
| Live apply on 38k held writes `RetentionApplied` | F17 stop-before; prepare merge is correct if they apply later |
| Hotspot file growth | F29; no `project.rs` |
| Pin bodies leak into samples | AC3 JSON denylist of fixture content |
| Nightly log looks like work | F20 leave format; CLI pretty is SoT |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit 6/5 zero candidates on ~35k memories | **Absorb** F1–F11 / AC1–AC13 |
| Placeholder F1 honesty sentence | **Absorb as overlay + warning** (F1/F10) — sentence-only declined as insufficient |
| Placeholder F2 optional overlay | **Absorb** — chosen (COUNT+samples) |
| Placeholder F3 `none_auto` | **Affirm** F3 |
| Placeholder F4 apply JSON+confirm | **Affirm** F4 |
| T166 §5.1.5 memory_legacy scan | **Absorb as inventory** (not per-row age wipe) |
| T166 §11 / T167 classify entire vault | **Decline as DoD** F18 — importer, not retention plan |
| T248 F3 empty = candidates==0 | **Lift** F8 — dispose-work; empty vault tests stay |
| T248 F16 doctor retention check | **Decline** F20 |
| T248 F17 T166 engine leftovers (cascade TOCTOU, chrono overflow) | **Decline** — not inventory |
| T266 format maze | **Decline** — tokens frozen |
| T263 H2 pin→Approved | **Decline** |
| T273 F7 `bridge_search_args` | **Decline** F21 |
| T272 Safety skip | **Completed** peer — do not reopen |
| leftover `7d97a456` 18k pins | **Decline** T259 operator rebind; overlay will **count** them as held (honest) |
| Identity mismatch `7d97a456` vs `fcb8a40f` (agent observation) | **Decline** F19 — T240/T257/T258; this cwd mismatch false; **no T274** |
| last-PR Cursor #187 | **N/A** empty |
| Dependabot clap/chrono/rusqlite/tokio PRs | **Decline** F14 — not this track |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `changeguard` | **Decline** |
| T240 F2 / T255 bag / clap 5 / DTO | **Decline** F14/F15/F21 |
| R-CI-BRANCH / packaging | **Decline** — not retention |

---

## 10. Implement order (on go)

1. Phase 0 re-verify live hole + deferred rescan + #187 still empty.
2. Red: AC1/AC3/AC6/AC9/AC12 (required red). AC2/AC7/AC8/AC10/AC11 may already be green. AC17 is a merge-sort guard (not Phase-1 red).
3. Green: store helper (F5 SQL LIMIT 5 + m1 fallback) → merge SOOT (F6/F30/F31) → pretty F8/F9 → honesty map → after_help.
4. Docs F27. Targeted nextest + clippy.
5. Review loop + codex-review. No live apply.
6. FEATURE TX commit; implement-track Phase 6 publish.

---

## 11. Soft residuals

| Residual | Notes |
|----------|--------|
| Nightly one-liner `candidates=` includes held | F20 — not restyled |
| `active` / unknown statuses lumped in `other` skip | Intentional v1; no third mechanism |
| Sample ids prefer pinned; forgotten-only uses other | AC1 |
| `list_pinned_memory_ids` still loads all ids for R11 | Pre-existing; do not “fix” by removing R11 |
| JSON still omits zero buckets for the other eight classes | T248 F5 |
| PATH until `cargo install` | F16 |
| Leftover project 18k pins still owned by `7d97a456` | T259 operator |
| Doctor retention check | T248 F16 |
| rusqlite `table_exists` 0.40 | T213 L4 |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-store/src/projections/retention.rs` | `MemoryLegacyInventory` + `memory_legacy_inventory` (`COUNT` + `LIMIT 5`) |
| `crates/ai-brains-control-plane/src/class_based_retention.rs` | `merge_memory_legacy_inventory`; call from plan/prepare/apply |
| `crates/ai-brains-contracts/src/retention.rs` | Honesty const only |
| `crates/ai-brains-cli/src/commands/retention.rs` | F8/F9 pretty; F10 short label |
| `crates/ai-brains-cli/src/main.rs` | Plan `after_help` additive (RetentionCommands only) |
| `crates/ai-brains-control-plane/tests/class_based_retention.rs` | AC2–AC5 |
| `crates/ai-brains-cli/tests/retention_plan_human.rs` (or sibling hermetic) | AC8–AC12 |
| `Docs/CAPABILITIES.md` / `Docs/OPERATIONS.md` / `Docs/PROTOCOL-COMPAT.md` / `CHANGELOG.md` | F27 |
| `conductor/conductor.md` / `conductor/deferred.md` / this spec+plan | Registry |

**Do not touch:** `project.rs`, `preflight.rs`, `sync.rs`, `legacy_import.rs`, `nightly.rs` (except it already calls `plan_retention`), `doctor.rs`, daemon, migrations.

---

## 13. AI fold-in

Inputs: `agy-review.md` + `opencode-review.md` (2026-08-20). **Do not edit those files.** Product tree at fold-in = plan dogfood `fdd4924`. **B 0 / M 0.**

### Pins locked by fold-in

1. **F5 / AC1:** forgotten-only / active-only samples use SQL `status != 'pinned' ORDER BY memory_id ASC LIMIT 5` (Agy m1). Mixed fixtures still sample pinned first.
2. **F5 / AC16:** `LIMIT 5` is in SQL, not a Rust slice of an unbounded SELECT (Agy O2).
3. **F30 / AC17:** after merge, `classes` sorted by `class`. Guard, not Phase-1 red.
4. **F31:** `NOTE_MEMORY_LEGACY_INVENTORY` const in `class_based_retention.rs` (Agy O1).
5. **F6:** merge **after** `build_report`. OpenCode `:771` is `collect_candidates` inside prepare — not a license to invent inventory `Candidate`s there.
6. **F8 stands:** `Nothing to dispose.` = `would_ce_wipe + would_projection_delete == 0`. OpenCode deferred-table “lifts to `totals.candidates == 0`” is **wrong** and **declined**.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** sample fallback when `pinned==0` | **Folded** F5 / AC1 second case / AC4 `sample_ids` non-empty |
| Agy | **m2** `classes` sort after upsert | **Folded** F30 / AC17 |
| Agy | **O1** notes const | **Folded** F31 |
| Agy | **O2** SQL `LIMIT 5` | **Already** F5 / AC16 — tightened “not Rust `.take` on unbounded SELECT” |
| OpenCode | B / M | None filed — live line/symbol table **affirmed** |
| OpenCode | **m** HEAD `70d61cd` vs plan `fdd4924` | **Folded** §2.1 — product tree identical |
| OpenCode | **m** nightly `:509–531` vs `:511–535` | **Folded** §2.3 / §5.4 |
| OpenCode | **O1** pinned 38,210 vs 38,208 | **Folded** §2.1 volatile |
| OpenCode | **O2** hotspot 3.999 vs 4.008 | **Folded** §2.3 snapshot |
| OpenCode | **O3** search `:239/:240/:269/:990` | **Folded** §2.3 — `:771` still prepare collect |
| OpenCode | **O4** `ISSUES.md` missing | **Already** F24 |
| OpenCode | deferred table “F8 → `candidates==0`” | **Decline** — F8 is dispose-work; re-trigger only if owner reopens T248 empty-check |
| OpenCode | “SOOT is collect_candidates `:771`” | **Partial / already F6** — call site yes; overlay merge stays after `build_report` |
| both | last-PR #187 Cursor | **Affirm N/A** — no T274 |

No Blockers/Majors to decline. No new placeholder minted.
