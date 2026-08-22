# T284 — Retention Work table and apply samples must not hide dispose work

- **Track ID:** T284-RetentionWorkSamples
- **Status:** **Planned** (Pending until **go**)
- **Category:** BUGFIX / HONESTY
- **Owner:** Grok
- **Source:** Last-PR Cursor Bugbot on [#188](https://github.com/Ryan-AI-Studios/AI-Brains/pull/188) (T270) — two Mediums (`e03e500a`, `04bc5b81`). Verified still true on HEAD `abaab31` (T277 `#192`). Placeholder minted with T274–T284.
- **Depends on:** T270 ✅ overlay + F9 Work-dispose-only; T248 ✅ human matrix; T166 ✅ class engine + R11 pin-hold + R12 `RetentionApplied`
- **Blocks / feeds:** Operators (and agents) can believe `next: apply` because Work lists the **dispose identities**. Apply audit samples name what was actually queued for CE/projection, not the inventory overlay.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “#188 Work hides CE when held dominates; apply samples prefer overlay ids”; T270 F9 **lift** (Work is still dispose-only; filter is class-level dispose counts, not dominant `mechanism`); T270 closeout residual “last-PR Cursor #188 Work table / apply samples”
- **Not absorbed (DoD):** Live `retention apply --confirm`; CE wipe on the operator vault; T270 overlay removal; `dominant_mechanism` rewrite; `classify_legacy` / `migrate governed`; doctor 16th; clap 5 / rusqlite 0.40; DTO **required** keys / `api_version` bump; T278–T283 peers; T240 F2; leftover `7d97a456` rebind
- **Research date:** 2026-08-22 (plan dogfood HEAD `abaab31` T277 `#192`; product `src/` = T277). Fold-in against `da6f316` (plan docs; crates identical to `abaab31`).
- **AI fold-in:** 2026-08-22 `agy-review.md` + `opencode-review.md`. **B 0 / M 0.** **Already:** Agy m1 F6 fallback; Agy m2 F7 de-dupe; Agy O2 / OpenCode m1 F38 comment; OpenCode m2 F28 no `Default`. **Agree:** Agy O1 exact 5-key omit (F37/AC5); OpenCode O1 same-file `audit_sample_ids` unit (F41/AC16); Agy m1 named pretty fallback unit (AC17). **Affirm:** #192 N/A; no T285. Disposition **§13**.
- **Ledger:** planning DOCS TX `d2010eda-264a-449b-9f37-f3e7687e9fe1`. Fold-in DOCS TX `9c454170-57a9-405a-b6e6-ace0b177b472`. Implement starts a **BUGFIX** TX on **go**.
- **Isolation:** Do **not** run live `retention apply --confirm`. Do **not** call `classify_legacy` / `migrate governed`. Do **not** `cargo install`, rewrite `.env` (T240 F2), pin-as-implement, or mutate schtasks. Do **not** grow hotspot `project.rs` / `preflight.rs` / `sync.rs` / `doctor.rs` / `ranking.rs`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Work lists dispose work.** When `totals.would_ce_wipe + would_projection_delete > 0`, the human `Work` table has **at least one data row** (class + dispose count + dispose mechanism + dispose sample ids). An empty `Work` header plus `next: apply` is a product lie.
2. **Held can still dominate the class.** R11 pin-holds keep `held` candidates in the same class as aged CE (`secret` / `evidence`). `dominant_mechanism` (majority; ties prefer `held` via `BTreeMap` + `max_by_key` last-wins) stays for the **Class matrix**. Work does **not** use that dominant string as the row filter.
3. **Apply audit samples name disposal.** `RetentionApplied.sample_ids` (cap 5) prefer CE keys / turn identities over T270 overlay pin ids. `memory_legacy` sorting first by class name must not starve dispose samples.
4. **Inventory-only stays honest.** Live vault with 0 CE / 0 projection still prints `Nothing to dispose.`, no Work, no `next:`. Overlay COUNT + held/skip stay (T270).
5. **North star.** Capture independence: SQL counts + pretty + event sample ids only. No models/graph. No hidden CoT. Append-only log: plan writes nothing; apply still emits `RetentionApplied` (R12) with **action** samples.

This unblocks the daily product: T270 made 39k pins visible as inventory. Bugbot #188 showed the overlay + F9 dominant-mechanism filter can hide real CE/projection work and poison the apply audit. T284 is the correction, not a new retention engine.

---

## 2. Live baseline (re-scan 2026-08-22)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | **Plan dogfood:** `abaab31` T277 `#192`. **This fold-in:** `da6f316` (`docs(conductor): plan T284 Work dispose rows and apply samples`). `git diff abaab31 HEAD -- crates/` empty — product `src/` identical to T277. Tree **CLEAN** at plan; fold-in dirties conductor only. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-21 05:55**, 25 368 576 bytes, **0.1.1**. **T270** on PATH (before T274–T277). Retention Work/samples hole is **T270-era — PATH is valid**. **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **3429**. In-context **0/0/0**. Grants **0 of 3** (T275 hermetic; live not bootstrapped). Capture independence holds. |
| `project whoami` | `mismatch: false`. Effective/path/detect `3581317d`. Shell leftover `7d97a456` overridden by local `.env`. **Not this track** (T258 adopt-path; leftover volume T276; `--show` leftover **T282**). |
| `memory list --summary --global` | Pinned **39118** / Forgotten **29**. Leftover `7d97a456` **18039**. Path owner `3581317d` **3429**. |
| PATH `retention plan --format human` | `Nothing to dispose.` Class matrix `memory_legacy none_auto held 39147`. Totals `candidates=39147 ce_wipe=0 projection_delete=0 skip=29 held=39118`. Honesty includes `memory_legacy inventory ≠ auto-forget`. **No** `Work` header. **No** `next:`. **T270 freeze still true on this vault.** Mixed held+CE hole is **hermetic** (no live CE candidates). |
| PATH `retention plan --format json` | One class `memory_legacy` held 39147. Bucket keys **exactly** `class`, `candidate_count`, `mechanism`, `sample_ids`, `notes`. Sample ids are overlay pins (first two `0000a8af-…`, `00015743-…`). |
| Last GitHub PR | [#192](https://github.com/Ryan-AI-Studios/AI-Brains/pull/192) T277 (2026-08-22). `gh pr view --comments`, `/reviews`, `/comments`, `issues/192/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, actions, tokio, …). **No leftover to mint.** |
| Prior #188 Bugbot | Two inline Mediums, still true on today’s tree (line drift: Work filter now `:430–436`; `dominant_mechanism` `:686`; `append_retention_applied` samples `:1263–1270`). **Absorbed here.** |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081). **0 pending / 0 drift.** Hotspot **#1** `project.rs` (**3.962**). CLI `preflight.rs` **#7**. `retention.rs` **981** / CP `class_based_retention.rs` **1292** / store `retention.rs` **497** — **not** top-10. `doctor.rs` **1855** / `main.rs` **4835** — **do not grow** doctor. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why these two Mediums still matter

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Work empty when held dominates CE | `Nothing to dispose.` keys off **totals** (`:420`). Work rows key off **dominant** `c.mechanism` (`:434–436`). R11 (`classify_envelope` `:497–513`) puts held + aged CE in the **same** class. `dominant_mechanism` (`:686–696`) majority; tie → last `BTreeMap` key with max count (`held` after `ce_wipe`). `dispose_work > 0` still prints `Work` header + `next: apply` (`:514–522`) with **zero data rows**. **DoD.** |
| `RetentionApplied` samples prefer overlay | `merge_memory_legacy_inventory` sorts `classes` by name (`:787`) so `memory_legacy` precedes `raw_turn` / `secret`. `append_retention_applied` (`:1263–1270`) concatenates `c.sample_ids` until 5. Overlay SQL already supplies ≤5 pin ids. A real CE/projection apply can record **only memory ids**. ISO 27001 A.8.10 / GDPR deletion logs must sample **what was deleted**, not the retained inventory. **DoD.** |
| Rewrite `dominant_mechanism` to prefer CE | Class matrix would label 10 000 held + 1 CE as `ce_wipe` COUNT=10001. T270 inventory honesty dies. **Decline.** |
| Split two buckets same class name | `class_bucket_map` last-wins; matrix collides. **Decline.** |
| Live apply to prove samples | Stop-before. Hermetic `prepare_retention_apply` is DoD. **Decline live apply.** |
| Remove T270 overlay | Standing. Overlay is why 39k pins are visible. **Decline.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|--------|
| Pretty Work | CLI `retention.rs` `format_retention_pretty` **`:406–445`** | `dispose_work` `:420` = `would_ce_wipe + would_projection_delete`. Empty → `Nothing to dispose.` Non-empty → `Work` header then skip `candidate_count==0` **and** skip unless `mechanism` is `ce_wipe` / `projection_delete`. |
| `next:` | **`:514–522`** | Same totals test as empty-check. CE → `--scope`; projection-only → `--confirm` only. T248 F11 last after Errors. **Keep.** |
| Inventory pretty lock | `format_retention_pretty__held_inventory_only__nothing_to_dispose_no_work_no_next` **`:682`** | Must stay green. |
| Dominant | CP `class_based_retention.rs` `dominant_mechanism` **`:686`** | `BTreeMap<&str, u64>` + `max_by_key` count. Empty → `skip`. **Do not change.** Comment at `:629` (“first non-skip if mixed”) is **stale** vs the majority impl — F38 rustdoc. |
| Report build | `build_report` **`:610–684`** | One bucket per class. Totals **per candidate**. `sample_ids` = first 5 of `items` (held can fill the cap before CE). **No** per-class dispose counters today. |
| Overlay merge | `merge_memory_legacy_inventory` **`:711–788`** | After `build_report` (T270 F6). Upserts `memory_legacy` held/skip. Sort by `class` (T270 F30). Does **not** emit CE. |
| Apply samples | `append_retention_applied` **`:1248–1291`** | `class_counts` = dominant + `candidate_count` (leave). `sample_ids` walk classes in report order. Called from prepare **`:928`**, finalize **`:1019`**, in-process **`:1144`**. |
| R11 | `classify_envelope` **`:497–513`** | Any pinned memory subject → whole key `held`. Envelope id `content_key:{uuid}`. |
| Stream A turns | `collect_candidates` **`:325–337`** | `id` = `turn:{session}:{index}`; `projection_delete`. |
| DTO bucket | contracts `retention.rs` `RetentionClassBucket` **`:125–135`** | Required: `class`, `candidate_count`, `mechanism`, `sample_ids`, `notes`. Live JSON matches. |
| Event | `RetentionAppliedPayload` **`:603–616`** | `sample_ids` optional skip empty. Cap 5 recommended. No bodies. |
| clap | `main.rs` `RetentionCommands::Plan` **`:2298`**; `Apply` **`:2312`** | Plan `--format` default `auto`. Apply default `json` + `--confirm`. **No new flags.** |
| Overlay apply lock | `retention_apply__pinned_inventory__held_in_report_no_delete` **`:1031`** | Prepare does not delete. **Does not** assert event `sample_ids`. |
| Mixed held+CE | **none** | R11 test `retention_plan__pinned_memory__held` **`:439`** is held-only (envelope bound to the pin). |
| Nightly | `nightly.rs` **`:518`** | Totals eprintln. **Do not restyle** (T270 F20). |
| Contracts tests | `retention.rs` **`:281`** roundtrip bucket literal | New optional fields need `#[serde(default)]` + skip-if-zero so roundtrip + live inventory JSON stay green. |
| CLI pretty fixtures | `retention.rs` **`:692`, `:833`, `:890`** | Three `RetentionClassBucket { … }` literals. |
| Hotspots | `project.rs` #1 3.962; `preflight.rs` #7 | **Do not touch.** Helpers stay in CLI `retention.rs` + CP `class_based_retention.rs` + contracts `retention.rs`. |

### 2.4 Dependency / standards research (2026-08-22) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). **No clap 5.** | **No bump.** No new flags. Additive `after_help` only. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** Optional skip-if-zero keys. |
| `chrono` | lock **0.4.44** | crates.io **0.4.45** (Dependabot #62 open) | **No bump.** |
| `rusqlite` | lock **0.39.0** + sqlcipher + backup | crates.io **0.40.2** (Dependabot #61; T213 L4 `table_exists`) | **No bump.** No new SQL. |
| `uuid` | lock **1.23.1** | — | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | — | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Record **deletion** activities, not the retained store | [ISO 27001:2022 A.8.10](https://knowledge.adoptech.co.uk/a.8.10-information-deletion) “Recording Deletion”; [AuditFront A.8.10](https://www.auditfront.com/frameworks/iso-27001/technological/a-8-10/) “what was removed, when, the method”; IT Governance 8.10 “Auditors will expect sample evidence” | `RetentionApplied.sample_ids` are the deletion log sample. Overlay pin ids are **inventory**, not deletion. Work table is the pre-delete review of **due** work. |
| Deletion log must not re-store bodies | [ISMS Lite GDPR deletion log](https://www.ismslite.de/en/blog/gdpr-data-deletion-policy): date, class, count, method; **not** names/content | Keep truncated ids only (T166 R4). Do not add bodies. Prefer dispose **identities** (`content_key:…` / `turn:…`). |
| Schedule + inventory can coexist | ISO 27001 A.8.10 “Retention Period Alignment”; T270 overlay | Class matrix still shows held COUNT. Work is the **due** slice. |
| clap `after_help` | [docs.rs/clap/4.6.6 `Command::after_help`](https://docs.rs/clap/4.6.6/clap/struct.Command.html) | Keep derive `after_help`. No clap 5. |
| Additive JSON extras | T180 PROTOCOL-COMPAT additive; serde `default` + `skip_serializing_if` | Required report keys stay. Class-bucket optional dispose fields omitted when 0 so live inventory JSON **byte-keys** stay the five today. |
| rusqlite 0.40 | crates.io 0.40.2; breaking VTab in 0.40.0 | Forbidden this track (SQLCipher + T213 L4 residual). |

**N/A:** SQLCipher page crypto, schtasks, llama.cpp `/health`, T180 preflight 2-key DTO (this is retention class bucket extras, not `PreflightContextResponse`).

**Could not verify:** a live mixed held+CE class on this vault (0 CE / 0 projection). Hole is proven from src + Bugbot + R11 + `dominant_mechanism`. Hermetic AC is DoD.

**ledgerful / ai-brains:** `preflight --summary`; `whoami` mismatch false; `memory list --summary --global` 39118; `retention plan` human+json inventory freeze; `ledgerful doctor` (4 warn, work root this repo); ledger 0 pending / 0 drift; `index --incremental`; `search --json -- "dominant_mechanism"` hits `:629/:686`; `append_retention_applied` `:928/:1019/:1144/:1248`; `scan --impact` CLEAN at `abaab31`; `hotspots` project.rs #1 (do not grow); `recall` T248/T270 reviews + pin that #188 Mediums map to T284. Semantic `ask` not required (search hit).

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `d2010eda`. Fold-in is DOCS TX `9c454170`. Implement starts a **BUGFIX** TX. |
| **F1 — T270 F9 lift** | Work is still **dispose-only**. Filter is **class-level dispose counts** (`would_ce_wipe + would_projection_delete` on the bucket), not `c.mechanism == ce_wipe \| projection_delete`. Inventory-only (`totals` dispose 0) still `Nothing to dispose.` / no Work / no `next:` (T270 F8 stands). |
| **F2 — One bucket per class** | Do **not** split `secret` into two `classes[]` rows. `class_bucket_map` and CANONICAL_CLASSES matrix stay one row per class. Dominant `mechanism` + `candidate_count` still describe the **majority** (held may win). |
| **F3 — Do not change `dominant_mechanism`** | Majority + tie → `max_by_key` last-wins (`held` beats `ce_wipe` on a 1–1 tie). Matrix honesty for inventory-heavy classes. |
| **F4 — Additive optional bucket fields** | On `RetentionClassBucket`, add: `would_ce_wipe: u64`, `would_projection_delete: u64`, `dispose_sample_ids: Vec<String>`. `#[serde(default, skip_serializing_if = "…zero/empty")]`. **E1:** missing / 0 / absent all mean no dispose in that class. `api_version` stays **1**. Report-level keys unchanged (PROTOCOL-COMPAT T248/T270). Live inventory JSON **omits** the three keys (dogfood today: five keys only). |
| **F5 — Fill in `build_report`** | Per class: count candidate mechanisms into the two u64s; `dispose_sample_ids` = `truncate_sample_ids` of items whose mechanism is `ce_wipe` then `projection_delete` (CE first), cap 5. Existing `sample_ids` stay first-5 of **all** items (matrix/JSON inventory samples unchanged). Overlay merge **does not** add dispose counts or dispose samples. |
| **F6 — Work rows** | For each class with dispose count > 0: print one row per non-zero dispose mechanism (`ce_wipe` and/or `projection_delete`). `COUNT` = that mechanism’s class count (**not** `candidate_count`). `MECHANISM` = that token. `SAMPLES` = `dispose_sample_ids` (shared across two rows if both > 0 — rare; R13 usually one stream per class). If `dispose_sample_ids` empty but dispose count > 0 (old JSON), fall back to `sample_ids`. |
| **F7 — Apply audit samples** | If `totals.would_ce_wipe + would_projection_delete > 0`, `RetentionApplied.sample_ids` fill **only** from classes with class dispose count > 0, using `dispose_sample_ids` (fallback `sample_ids`), cap 5, de-dupe, `truncate_id`. Do **not** pad with overlay pins. If totals dispose == 0 (inventory-only apply), keep today’s class-order fill (pins OK — nothing was disposed). Helper `audit_sample_ids` used by all three `append_retention_applied` callers. |
| **F8 — `class_counts` frozen** | Event `RetentionClassCount` stays `{class, count: candidate_count, mechanism: dominant}`. Do **not** split event class_counts this track (Bugbot did not ask; schema freeze). |
| **F9 — Overlay stands** | T270 COUNT overlay, F5 SQL LIMIT 5, F6 merge-after-build, F8 empty-check, F10 honesty const, F30 sort, F31 notes — **untouched**. No `--exclude-project`. No `MemoryMoved`. |
| **F10 — Apply still gated** | Default JSON + `--confirm`. No live apply as dogfood. Hermetic `prepare_retention_apply` is the sample AC. |
| **F11 — No new clap flags** | Plan/Apply `--format` unchanged. Additive Plan `after_help` sentence: Work lists dispose identities even when the class’s dominant mechanism is `held`. |
| **F12 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.1**. rusqlite stays **0.39.0**. |
| **F13 — Capture independence** | Pretty + SQL already collected + event sample ids. No models, graph, embeddings, or ledgerful on this path. |
| **F14 — Plan writes nothing** | `retention plan` does not append `RetentionApplied`. Event-log COUNT unchanged across plan (existing T270 AC). |
| **F15 — PATH** | Do not `cargo install` unless the user asks. |
| **F16 — Stop-before live apply** | Even after go: no `retention apply --confirm` on the live vault. No CE. No horizon retune. No nightly CE opt-in. |
| **F17 — Decline doctor / HTTP / desktop / nightly restyle** | T248 F16. Nightly `eprintln!` totals unchanged. |
| **F18 — Decline identity / leftover tracks** | Shell leftover `7d97a456` is T258/T276/T282. `project list` leftover-first is **T283**. Graph density **T278**. Safety **T279**. Policy `--scope` **T280**. Nightly 750 ms **T281**. |
| **F19 — T240 F2 / T255 750 ms / clap 5 / rusqlite 0.40** | Standing declines. |
| **F20 — last-PR Cursor** | #192 empty → **N/A**. #188 two Mediums → **this track**. **No T285.** Dependabot rusqlite/chrono PRs are not findings. |
| **F21 — Tests** | Naming `function_or_feature__condition__expected_result`. rstest `#[case]` for tie vs majority mixed secret. No `unwrap`/`expect`/`panic` in production. `TempEnv` if tests touch `AI_BRAINS_RETENTION_*`. |
| **F22 — Cross-model** | Honesty UX + apply audit. After Phase-1 review clean, run read-only `codex-review`. |
| **F23 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F24 — Docs** | CAPABILITIES T248/T270 row: Work lists dispose identities (not dominant mechanism). OPERATIONS Audit + Work sentence. PROTOCOL-COMPAT: report keys unchanged; class bucket **may** emit optional `would_ce_wipe` / `would_projection_delete` / `dispose_sample_ids` when non-zero (absent = 0). CHANGELOG T284. |
| **F25 — Existing tests stay green** | Empty-vault CP + T248 pretty/JSON hermetics; inventory pretty `:682`; overlay apply `:1031`; R11 held `:439`; raw_turn Work `:855`; CE next-scope `:880`. Do not weaken body-plaintext asserts. |
| **F26 — CLI file growth** | Pretty Work lift in `retention.rs`. Engine fill + `audit_sample_ids` in `class_based_retention.rs`. Optional fields in contracts `retention.rs`. **Do not** grow `project.rs` / `preflight.rs` / `doctor.rs`. |
| **F27 — Helpers** | `pub(crate) fn class_dispose_count(b: &RetentionClassBucket) -> u64` in contracts (sum of the two u64s). CLI Work iterates mechanisms. CP `pub(crate) fn audit_sample_ids(report: &RetentionPlanReport) -> Vec<String>` — crate-visible for same-crate units; **not** `pub` (integration `tests/` must keep using events — AC2). |
| **F28 — Struct literals** | All `RetentionClassBucket { … }` sites (~6) set new fields (or `..Default` if Default is added). Prefer explicit `0` / `vec![]` in production constructors; tests may use `Default` **only if** `class`/`mechanism` remain required at type level — **do not** `#[derive(Default)]` a bucket with empty class. |
| **F29 — Mixed CE+PD in one class** | Two Work rows. Samples may repeat `dispose_sample_ids` on both rows. Do not invent a `ce_wipe+projection_delete` token. |
| **F30 — Secrets** | Never print `AI_BRAINS_KEY`. Sample cells truncated ids only. |
| **F31 — Workspace** | `0.1.1`. |
| **F32 — Contracts DTO** | Optional fields only. No new required keys. No `RetentionPlanReport` new fields. No event payload new fields. |
| **F33 — JSON scripts** | Totals already tell global dispose. Optional class fields are for mixed-class honesty + Work. Scripts that ignore extras stay valid (T180). |
| **F34 — Fixture for AC1** | Reuse CP `insert_memory` + `insert_active_key` + `insert_blob` (`content_class=secret`, aged `created_at`). ≥1 pinned envelope + ≥1 unpinned aged secret envelope. rstest cases: **1 held + 1 CE** (tie → held dominant) and **2 held + 1 CE** (majority held). |
| **F35 — Fixture for AC2** | Overlay ≥5 pinned memories (so overlay samples fill cap 5) + ≥1 old `raw_turn` (`insert_turn` 120d). `prepare_retention_apply(..., confirm=true, dry_run=false)`. Parse `Payload::RetentionApplied` `sample_ids`: at least one `turn:` prefix; **not** five overlay UUIDs only. |
| **F36 — Pretty unit AC3** | Construct a `secret` bucket `mechanism=held`, `candidate_count=12`, `would_ce_wipe=2`, `dispose_sample_ids=["content_key:ck-ce"]`, `sample_ids=[pin ids]`. `format_retention_pretty` contains `Work`, a `secret` line with `2` and `ce_wipe` and `content_key:ck-ce`, **not** an empty Work header. Totals `would_ce_wipe=2`. `next:` present. |
| **F37 — Inventory JSON omits extras** | Held-only / zero-dispose class JSON object keys are **exactly** `class`, `candidate_count`, `mechanism`, `sample_ids`, `notes` (Agy O1). Omit `"would_ce_wipe"`, `"would_projection_delete"`, `"dispose_sample_ids"` (skip-if-zero). Live PATH json today is the exhibit. |
| **F38 — Stale comment** | Replace `build_report` “first non-skip if mixed, else majority” with “majority; ties prefer later BTreeMap key (`held` over `ce_wipe`)”. |
| **F39 — No T285** | #192 empty; #188 absorbed; Dependabot not a leftover. |
| **F40 — Gate** | Targeted nextest `-p ai-brains-contracts retention` + `-p ai-brains-control-plane --lib` (F41 units) + `-p ai-brains-control-plane --test class_based_retention` + `-p ai-brains-cli --test retention_plan_human` + clippy those packages. Full workspace gate only at implement closeout — **not** a plan gate. |
| **F41 — `audit_sample_ids` same-file unit (OpenCode O1)** | Required `#[cfg(test)]` in `class_based_retention.rs` (file currently has **no** test module; integration tests stay in `tests/`). Overlay-only report → pin ids OK (AC12). Mixed dispose>0 → dispose ids only, cap 5, de-duped, includes `turn:` / `content_key:`, **no** overlay pin padding. Do **not** require an event-log round trip for this helper. |

---

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | CP: mixed held+CE `secret` (F34 cases). `secret.mechanism == "held"`. `totals.would_ce_wipe >= 1`. `secret.would_ce_wipe >= 1`. `dispose_sample_ids` contains the unpinned `content_key:` id. `sample_ids` **may** still be pins. |
| **AC2** | CP: overlay ≥5 pins + 1 old turn (F35). `prepare_retention_apply` appends `RetentionApplied` whose `sample_ids` include a `turn:` identity and do **not** equal the overlay pin list. |
| **AC3** | CLI unit F36: mixed held-dominant pretty prints Work **data row** for `secret` / `ce_wipe` / dispose count / dispose sample. Contains `next:`. Does **not** print only an empty Work header. |
| **AC4** | Inventory-only pretty (`:682` / live dogfood shape) still `Nothing to dispose.`, no Work, no `next:`. Overlay apply `:1031` still no CE/projection enqueue. |
| **AC5** | Serde F37: zero-dispose class JSON keys are **exactly** `class`, `candidate_count`, `mechanism`, `sample_ids`, `notes`. Roundtrip `retention_plan_report__roundtrip` still equal. `api_version` **1**. |
| **AC6** | `format_retention_pretty__raw_turn_work__…` and CE `--scope` next still green (pure dispose class, dominant == dispose). |
| **AC7** | Plan hermetic: event-log COUNT unchanged (T270 F12). |
| **AC8** | No plaintext bodies in JSON/pretty (existing R4 asserts). |
| **AC9** | clap: no new flags; unknown `--format` still exit **2**. |
| **AC10** | Docs: CAPABILITIES / OPERATIONS / PROTOCOL-COMPAT extras / CHANGELOG T284. |
| **AC11** | `dominant_mechanism` unit or comment F38; tie 1+1 still returns `held` (do not “fix” the tie). |
| **AC12** | Inventory-only `RetentionApplied` (overlay prepare) may still sample pin ids when totals dispose == 0. |
| **AC13** | PATH live (optional on go): `retention plan --format human` still inventory-only on this vault. **Do not** live apply. |
| **AC14** | Clippy `-D warnings` on touched crates. No `unwrap`/`expect`/`panic` in production. |
| **AC15** | `class_counts` in the event still use dominant mechanism + `candidate_count`. |
| **AC16** | F41: `audit_sample_ids__overlay_only__pins_ok` and `audit_sample_ids__mixed_dispose__prefers_dispose_ids_cap5_deduped` in CP `class_based_retention.rs` `#[cfg(test)]`. Mixed case: output contains a `turn:` or `content_key:` id, len ≤ 5, no duplicate ids, no overlay pin-only fill. |
| **AC17** | F6 fallback pretty unit: `would_ce_wipe=1`, `dispose_sample_ids=[]`, `sample_ids=["content_key:ck-legacy"]` → Work SAMPLES contains `content_key:ck-legacy` (not `—`). |

---

## 5. Design notes

### 5.1 Why not change dominant mechanism

Bugbot’s Work hole is a **presentation/filter** bug. The engine already counts CE per-candidate into **totals**. The class bucket compresses to one mechanism for the matrix. Changing the compressor would make `memory_legacy`-style inventory look like due work (or make due work look like the whole class). Split the views: matrix = dominant + total COUNT; Work = dispose slice.

### 5.2 Why optional DTO fields

Pretty and `append_retention_applied` only see `RetentionPlanReport`. Without per-class dispose counts, Work cannot print an honest COUNT for mixed `secret`. Optional skip-if-zero keeps T248 “keys frozen” for the empty/inventory path (live JSON exhibit). Mixed reports gain extras; T180 additive.

### 5.3 Apply sample pass

```text
if totals.ce + totals.pd == 0:
    // inventory-only: existing class-order fill (pins OK)
else:
    for class in report.classes where class_dispose_count > 0:
        push dispose_sample_ids (else sample_ids) until 5 unique truncated ids
    // do not continue into memory_legacy
```

### 5.4 Windows / sharing

N/A. No file create. No `Connection` vs `remove_file`.

---

## 6. Non-goals

- Live `retention apply --confirm` / CE on the operator vault.
- Removing or retuning T270 overlay / `none_auto` / pin hold.
- Changing `dominant_mechanism` or splitting `classes[]` by mechanism.
- New event fields / `api_version` bump / required JSON keys.
- Doctor retention check; HTTP `/v1/retention`; desktop UI.
- Nightly CE; horizon retune; `classify_legacy` / `migrate governed`.
- clap 5; rusqlite 0.40; chrono/serde_json bumps; new crates; `cargo install`.
- T240 F2 `.env`; leftover rebind; T278–T283 peers.
- Rewriting `class_counts` on `RetentionApplied`.
- Shared `resolve_*_format`; growing `project.rs`.

---

## 7. Verification plan (TDD)

**Phase 1 red (required before green):**

1. AC1 `retention_plan__mixed_held_and_ce_secret__held_dominant_dispose_counts` (rstest tie + majority) — today `would_ce_wipe` field does not exist / Work would be empty if pretty used the report.
2. AC3 `format_retention_pretty__held_dominates_ce_same_class__work_shows_dispose_row`.
3. AC2 `retention_apply__overlay_plus_raw_turn__applied_samples_include_turn` — today samples are overlay-first.

Then green: F4 fields + F5 fill + F6 Work + F7 `audit_sample_ids` → AC4/AC5/AC6 stay green. Same green: AC16 helper units + AC17 fallback pretty + AC5 exact 5-key omit.

**Stay green:** T248 empty pretty; T270 inventory pretty + overlay apply; R11 held; R4 no bodies; clap format.

Targeted: `cargo nextest run -p ai-brains-contracts retention` ; `-p ai-brains-control-plane --lib` ; `-p ai-brains-control-plane --test class_based_retention` ; `-p ai-brains-cli --test retention_plan_human` ; `cargo clippy -p ai-brains-contracts -p ai-brains-control-plane -p ai-brains-cli --all-targets -- -D warnings`.

Full workspace gate only at implement closeout — **not** a plan gate.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Optional JSON keys surprise scripts | skip-if-zero; inventory path omits; PROTOCOL-COMPAT extras; `api_version` 1 |
| Work COUNT vs matrix COUNT confuse operators | Docs: Work = due; matrix = class total + dominant |
| Apply sample change breaks a golden | No golden for sample_ids today; AC12 inventory-only pins stay |
| Struct-literal compile churn | F28; ~6 sites |
| Live apply temptation | F16; hermetic only |
| `project.rs` / `doctor.rs` growth | F26 |
| PATH-behind T277 | F15; hole is T270-era |
| Tie-break “fix” | F3/AC11 lock held-on-tie |
| Both CE+PD in one class | F29 two rows; rare |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-22 (post-P12 through T277 closeout).

| Row / leftover | Disposition |
|----------------|-------------|
| #188 Work hides CE when held dominates; apply samples prefer overlay ids | **Absorb** F1–F7 / AC1–AC3 |
| T270 closeout “last-PR Cursor #188 Work table / apply samples” | **Absorb** this track |
| T270 F9 Work dispose-only | **Lift** F1 — dispose **counts**, not dominant mechanism |
| T270 F8 `Nothing to dispose.` = no CE/PD | **Affirm** F1 / AC4 |
| T270 F20 nightly `candidates=` includes held | **Decline** F17 — not restyled |
| T248 F16 doctor retention check | **Decline** F17 |
| T166 CE wipe / live apply | **Decline** F16 |
| T248 JSON keys frozen | **Partial F4** — report keys unchanged; class-bucket **optional** skip-if-zero |
| last-PR Cursor #192 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Absorb** (source of this track) — no T285 |
| leftover `7d97a456` 11 roots / `--global` | **Decline → T276 Completed** (live rebind still F9 there) |
| `context --show` leftover shell | **Decline → T282** |
| `project list` leftover-first | **Decline → T283** |
| graph sparse / neighbors preview | **Decline → T278** |
| preflight Safety vs hotspots | **Decline → T279** |
| deny/`policy show` `--scope` vs doctor omit | **Decline → T280** |
| nightly Completion timeout vs daemon Open | **Decline → T281** |
| T277 live `backup create --no-prune` | **Decline** — T277 Completed hermetic; live skip residual |
| T275 live bootstrap | **Decline** |
| T240 F2 / T255 750 ms / clap 5 / rusqlite 0.40 | **Decline** F19 |
| T259 leftover memory reclassify / `MemoryMoved` | **Decline** |
| T264 leftover recall drop | **Decline** |
| Packaging / MSI / `.changeguard` / `anyhow` allowlist | **Decline** |
| device/replicate/query-trace/forget empty; T266 JSON | **Decline** — README-T274-T284 |

---

## 10. Implement order (on go)

1. Phase 0 re-verify Work `:420–445`, `dominant_mechanism` `:686`, `append_retention_applied` `:1263`, clap Plan `:2298`, T270 inventory pretty `:682`, overlay apply `:1031`, #192 empty, #188 still the two Mediums, clap 4.6.1 / rusqlite 0.39.0.
2. Rescan `deferred.md`.
3. FEATURE/BUGFIX TX.
4. **Red** AC1 + AC3 + AC2 (failing tests first).
5. **Green** F4 contracts fields → F5 `build_report` fill → F6 Work → F7 `audit_sample_ids` → F11 after_help → F24 docs → F38 comment → AC5 5-key omit → AC16 helper units → AC17 fallback pretty.
6. Targeted nextest + clippy (F40).
7. Review log + cross-model (F22).
8. Full gate at closeout. **Do not** live apply. **Do not** `cargo install`.
9. implement-track Phase 6 publish.

---

## 11. Soft residuals

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` until `cargo install` / `Build-AIBrains.ps1` | F15 — operator; tests use source |
| Live vault still 0 CE / 0 projection | Honest; mixed hole is hermetic |
| Event `class_counts` still dominant | F8 |
| Two Work rows sharing samples if CE+PD mix | F29 |
| Nightly `candidates=` includes held | T270 F20 |
| Doctor retention check | T248 F16 |
| Pretty formatter still in CLI `retention.rs` | F26 — not a peel this track |

---

## 12. Touch map

| File | Change |
|------|--------|
| `crates/ai-brains-contracts/src/retention.rs` | Optional F4 fields + `class_dispose_count` + skip helpers + roundtrip/omit unit |
| `crates/ai-brains-control-plane/src/class_based_retention.rs` | F5 fill; F7 `audit_sample_ids`; F38 comment; constructors; F41 `#[cfg(test)]` AC16 |
| `crates/ai-brains-control-plane/tests/class_based_retention.rs` | AC1 rstest + AC2 apply samples |
| `crates/ai-brains-cli/src/commands/retention.rs` | F6 Work rows; AC3 pretty unit; AC17 fallback pretty; F28 literals |
| `crates/ai-brains-cli/src/main.rs` | Plan `after_help` additive sentence only |
| `crates/ai-brains-cli/tests/retention_plan_human.rs` | Stay green; add hermetic mixed Work if CLI e2e is cheap (optional — AC3 unit is required) |
| `Docs/CAPABILITIES.md` | Work = dispose identities |
| `Docs/OPERATIONS.md` | Audit samples prefer dispose ids |
| `Docs/PROTOCOL-COMPAT.md` | Optional class-bucket extras |
| `CHANGELOG.md` | T284 row |
| `conductor/conductor.md` / `deferred.md` / README-T274-T284 | Planned + absorb table |

**Do not touch:** `project.rs`, `preflight.rs`, `sync.rs`, `doctor.rs`, `ranking.rs`, `nightly.rs` format, backup crates, `.env`.

---

## 13. AI fold-in

Inputs: `agy-review.md` + `opencode-review.md` (HEAD `da6f316`; product crates identical to `abaab31`). **B 0 / M 0.** last-PR #192 still empty (re-checked 0/0). No T285. Do **not** edit the review files.

### Per-AI

| Source | Item | Disposition |
|--------|------|-------------|
| Agy m1 | Work fallback to `sample_ids` when `dispose_sample_ids` empty | **Already** F6; **folded test** AC17 |
| Agy m2 | `audit_sample_ids` de-dupe before cap 5 | **Already** F7; locked by AC16 |
| Agy O1 | Contracts serde omit extras; exact 5 baseline keys | **Folded** F37 / AC5 |
| Agy O2 | Stale `dominant_mechanism` comment | **Already** F38 |
| OpenCode m1 | Stale comment `:629` vs impl `:686` | **Already** F38 |
| OpenCode m2 | No `Default` on `RetentionClassBucket`; ~6 literals | **Already** F28 |
| OpenCode O1 | Pure unit for `audit_sample_ids` (no event-log) | **Folded** F41 / AC16 (`pub(crate)` + same-file `#[cfg(test)]`) |
| last-PR #192 Cursor | empty | **Affirm N/A** — no T285 |
| No B/M | — | Nothing to decline of B/M |

### Declined / not new design

| Item | Why |
|------|-----|
| `#[derive(Default)]` on `RetentionClassBucket` | F28 — empty `class` is not a valid default |
| `pub fn audit_sample_ids` for `tests/` | F27/F41 — helper stays crate-visible; AC2 is the event lock |
| Change `dominant_mechanism` / split buckets / live apply | Unchanged F2/F3/F16 |
| serde_json 1.0.151 / clap 4.6.6 / rusqlite 0.40.2 | **No bump** (F12) |

### Pins locked by fold-in

1. **F37 / AC5:** zero-dispose class JSON keys **exactly** `class`, `candidate_count`, `mechanism`, `sample_ids`, `notes`.
2. **F41 / AC16:** `audit_sample_ids` same-file units (overlay pins OK; mixed dispose-only, cap 5, de-duped).
3. **AC17:** pretty fallback when `dispose_sample_ids` empty.
4. **F27:** `audit_sample_ids` is `pub(crate)`, not `pub`.
5. **F6 / F7 / F28 / F38:** already specified; reviewers confirmed live lines.
6. **§2.1:** fold-in HEAD `da6f316`; product crates identical to `abaab31`.
