# T323 — Conclusion in-force resolver

- **Track ID:** T323-ConclusionInForce
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE
- **Owner:** Grok
- **Source:** T311 residual **R5** (conclusion in-force was a written non-goal). Series README `README-T312-T324-CLI-DOGFOOD.md`. Placeholder gate: **walker if a successor chain exists in live src; else honesty decline.** Live src **has** the chain (`correct_conclusion` + projector `superseded_by`) — this is a **walker**, not a decline.
- **Depends on:** T311 pattern (CP resolver, not retrieval); T150 `conclusion_projection.superseded_by` + `valid_from`/`valid_until`; T160 `propose`/`confirm`/`correct`/`reject`; `conclusion_valid_at` already parameterized (private today)
- **Blocks / feeds:** Operators can ask “what conclusion is in force for term X?” Dual-model pins stay orientation (H2 declined). Does **not** steal T322 `--as-of` / T324 empty TERM / T325 / T326.
- **Absorbs:** T311 R5. Phase 0 chain proof §2.2.
- **Not absorbed (DoD):** T322 `--as-of` (decision-only; copy-not-share later); T324 PowerShell empty TERM; T311 R1 daemon `ListInForce`; H2 pin→Confirmed; FTS `conclusion list`; CLI `confirm`/`correct`/`activate`; projector edits; `conclusions_valid_at` SQL reuse; clap 5
- **Research date:** 2026-08-29 (plan-write product HEAD `766a6c8` T322 `#244`). Snapshot — **re-verify at execute**.
- **Ledger:** planning DOCS TX `61b188d1-fd07-48e6-9bec-bdce0d197c60`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** grow `governed_common.rs` (#3) / `sync.rs` (#2) / CLI `project.rs` (#1) / `forget.rs` (#5). `briefings/project.rs`: **visibility-only** on `conclusion_valid_at`. Do **not** edit `in_force.rs` (decision). Do **not** edit store `projections/conclusion.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** `cargo install`. Do **not** propose/confirm/correct on the **live** operator vault as proof.

---

## 1. Objective

1. **Resolve the current conclusion for a term.** `ai-brains conclusion in-force <TERM>` walks `conclusion_projection.superseded_by` in-scope and returns the current Active-or-Confirmed, valid-now successor (or honest none).
2. **Stay a governed read.** `ReadConclusions`. No new events. No pin→Confirmed. No daemon DTO. No `--as-of` this track.
3. **Do not lie if the chain is missing.** Phase 0 proved the chain exists (`correct_conclusion` writes `ConclusionSuperseded`; projector stores `superseded_by`). Therefore **ship the walker**, not a decline.
4. **North star.** Capture independence: projection query over the existing correction chain. No models, no graph, no embeddings.

This unblocks: T311 shipped `decision in-force`; `conclusion` is still propose-only. After `correct_conclusion`, the prior Confirmed is `state=Superseded` and briefing/query no longer surface it as current. Operators need “for term X, after correction, what governs?” without treating vault pins as Confirmed.

---

## 2. Live baseline (re-scan 2026-08-29)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Plan-write against `766a6c8` `feat(decision): T322 in-force --as-of hop-stop (#244)`. Branch `track/T323-conclusion-in-force`. `origin/main` = `766a6c8` (ahead **0** at checkout; dirty conductor T322 Completed note absorbed into this DOCS commit). |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. **T311 on PATH.** T312–T322 **not** (T322 `--as-of` is source-only). T323 hole **is** on PATH **and** source (`conclusion` is Propose-only). **Do not `cargo install`.** |
| `preflight --summary` (PATH) | Pinned **4640**. In-context **0/0/0**. `Total Word Count: 777` (PATH-behind T315 `Budget window words:`). **Not this DoD.** |
| PATH `conclusion --help` | Subcommand **`propose` only**. after_help one propose example. |
| PATH `conclusion in-force --help` | clap **unrecognized subcommand** exit **1**. Hole confirmed. |
| Last GitHub PR | [#244](https://github.com/Ryan-AI-Studios/AI-Brains/pull/244) T322. `mergedAt` **2026-08-29T17:08:39Z**. Issue/review/inline comments **[]**. Open PRs: **none**. `#237` Bugbot already **T326**. `#230` already **T325**. **No T327 from Cursor.** |
| Ledger | 0 pending / 0 drift before this DOCS TX. Doctor 5 warn (impact-stale at scan start; legacy `.changeguard`; sig-pin; sig-version; timings-0) — hygiene, not this DoD. |
| Hotspots | CLI `project.rs` **#1** (3.631). `sync.rs` **#2**. `governed_common.rs` **#3**. `forget.rs` **#5**. `briefings/personal.rs` **#9**. `conclusion.rs` / `conclusions.rs` / `in_force.rs` / store conclusion projector **not** top 10. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Chain proof (placeholder gate — **walker**, not decline)

| Layer | Truth |
|-------|--------|
| Row has `superseded_by` | `ConclusionRow.superseded_by: Option<String>` (`ports.rs:95`). Also `supersedes` (bidirectional; decisions have only `superseded_by`). |
| Correction writes the hop | `correct_conclusion` (`conclusions.rs:355–441`) batches `ConclusionProposed` (successor **Candidate**) + `ConclusionSuperseded` (`superseded_by: new_id`). Old must be Active/Confirmed/Stale/Disputed — **not** Candidate/Rejected (`:389–392`). |
| Projector persists it | `ConclusionSuperseded` sets `state='Superseded', superseded_by, updated_at=occurred_at` and links successor `supersedes` (`store/projections/conclusion.rs:91–112`). Does **not** close `valid_until`. |
| Confirm exists | `confirm_conclusion` (`:186–259`) Candidate/Active → Confirmed with human `ApprovalAuthority`. Unsupported (empty evidence) **cannot** confirm. Projector `update_state` only (`:79–80`) — **no approver column** on `ConclusionRow`. |
| Reject analog of revoke | `reject_conclusion` → `Rejected`. No `superseded_by`. |
| Activate | Candidate → Active (agent, non-protected). Briefing authority includes Active. |
| `conclusion_valid_at` | `briefings/project.rs:688–690` **private** today: `valid_from <= at && valid_until.map(\|u\| u > at).unwrap_or(true)`. `valid_from` on the row is **required** (`OffsetDateTime`, not `Option` — unlike decisions). **Fold:** `pub(crate)` only. No copy. No other `project.rs` edits. |
| `conclusions_valid_at` SQL | `adapters.rs:851–873` valid-time; **excludes** `Superseded`/`Rejected`. **Do not reuse** for the walker — hop parents are Superseded. |
| `list_conclusions_by_scope_state` | `adapters.rs:675–708`. `state: None` = all states. `ORDER BY valid_from ASC, conclusion_id ASC`. Scope filter when `Some`. |
| `get_conclusion` | `ports.rs:242` — hop lookup. |
| `current_successor` | `conflicts.rs:228–231` is `row.superseded_by.as_deref()` — **too thin**; do not import as the walker. |
| CLI | `commands/conclusion.rs` **propose-only** (188 physical). `ConclusionCommands` `main.rs:2856–2885` one variant. Dispatch `:4948–4976`. |
| Decision analog | T311/T322 `in_force.rs` **276** physical. **Copy-not-share** — do **not** edit it. New `conclusion_in_force.rs`. |
| ADR-0011 | Conclusions: Candidate → Active → Confirmed → Stale/Disputed/Superseded. Protected needs human confirm. Pins ≠ authority. |

**Pick:** T311-shaped **now-walker** over `superseded_by`. Ruling = tip is `Active` **or** `Confirmed` and `conclusion_valid_at(now)` (matches briefing authority `project.rs:305` `["Active", "Confirmed"]` and CAPABILITIES dual-model). Rejected-root → none + empty chain (T311 Revoked analog). `--as-of` **declined** this track (T322 copy-not-share residual).

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| New resolver | `control-plane/src/conclusion_in_force.rs` (**new**) | `resolve_conclusion_in_force(query, clock, scope_key, term)`. Export from `lib.rs`. **Not** retrieval. **Not** a generic over `in_force.rs`. |
| Decision resolver | `in_force.rs:52–131` | **Do not edit.** Stay-green T311/T322. |
| JSON | new `ConclusionInForceResponse` | `term`, `scope`, `ruling`, `chain`. Ruling: `conclusion_id`, `statement`, `state` (`"in_force"`), `updated_at`. **No** `title` / `approver` / `unsupported` / `as_of`. Do **not** skip `ruling`. |
| Valid-time helper | `briefings/project.rs:688` private | **`pub(crate)` only.** Call with `clock.now()`. |
| CLI handler | `commands/conclusion.rs` | Add `InForceOptions` + `run_in_force`. Copy **read** pattern from `decision.rs` `run_in_force` (`resolve_scope_key_for_cli`, `production_policy`, `ReadConclusions`, `fail_api` + `policy_denied_hint_details`). Propose path **untouched**. |
| clap | `main.rs:2852–2885` `ConclusionCommands` | Add `InForce { TERM, --scope, --format value_parser 7 tokens default json, --principal-id }`. **No** `--as-of`. **No** `--local`/`--daemon`. Dispatch next to Propose. |
| after_help | parent `:2854` + Propose `:2859` | **Additive** `ai-brains conclusion in-force workspace_id` on `InForce` and parent. |
| Hermetics | new CP `tests/conclusion_in_force.rs`; new CLI `tests/conclusion_in_force.rs` | Named below. Seed `ReadConclusions` (CLI analog of `seed_read_decisions`). CP uses real `propose`/`confirm`/`correct`/`reject`/`activate`. |
| Policy | `ReadConclusions` + `fail_api` exit **3** | Analog T311 F10. |
| Empty term | `fail_usage` exit **2** | **Freeze.** T324 is PowerShell `""` dropping the argv slot — **not stolen.** |
| Clock | `SystemClock` / `Clock` trait | `clock.now()` for valid-time (T311 F11). |
| `help_ia.rs:13` | Governed already names `conclusion` | **Freeze.** |
| Line counts | CLI `conclusion.rs` **188**; CP `conclusions.rs` **472**; `in_force.rs` **276**; `project.rs` **969**; store conclusion projector **131**; `main.rs` **5748**. Snapshot — **F22 80-net is phase diff vs go HEAD**. |
| Contracts | none | No DTO. PROTOCOL-COMPAT has **no** in-force row. **Do not add.** |

### 2.4 Dependency / standards research (2026-08-29) — snapshot, re-verify at execute

| Pin | Workspace / lock | crates.io / docs | Action |
|-----|------------------|------------------|--------|
| clap | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06); docs.rs 4.6 `value_parser` list + `Error::exit_code` 2 | **No bump.** Same 7-token parser as T311 F3. clap **5** forbidden. |
| time | workspace **0.3** / lock **0.3.47** | crates.io **0.3.55** (2026-08-01) | **No bump.** Rfc3339 only for `updated_at` emit (already on decision in-force). No `--as-of` parser. |
| serde / serde_json | workspace 1.0 / lock **1.0.150** | — | **No bump.** |
| rusqlite | **0.40.2** / SQLCipher **4.14.0 community** | — | **No bump.** No new SQL. |
| uuid | workspace **1.13** / lock **1.23.1** | — | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged. |
| workspace version | **0.1.3** | — | **No bump.** |
| New crates | — | — | **Zero.** |

**Current-state vs as-of research (primary sources):**

- SQL Server temporal: **current** table is a plain `SELECT` (no `FOR SYSTEM_TIME`). `AS OF` is the history clause ([Microsoft Learn — Query data in a system-versioned temporal table](https://learn.microsoft.com/en-us/sql/relational-databases/tables/temporal/query-data), updated 2026-08-24). **Fit:** T323 is the current-table analog (T311 now). T322 already shipped AS OF for **decisions**. Do not copy `--as-of` here.
- `conclusions_valid_at` documents “Does NOT use recorded_at / occurred_at (bitemporal: domain valid time)” and skips Superseded. Walker needs those rows as hop parents. **Do not reuse.**
- Successor-pointer current revision (CMS / git `HEAD`) matches `superseded_by` walk-to-tip then F9-class filter.

N/A: Windows schtasks / SQLCipher pin change / clap 5.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Layer** | `resolve_conclusion_in_force` lives in `ai-brains-control-plane/src/conclusion_in_force.rs` (export from `lib.rs`). **Not** retrieval. **Not** a parameterization of `in_force.rs`. Do **not** edit `in_force.rs` / `decisions.rs` / store decision projector. Do not grow `governed_common.rs`. `project.rs`: **visibility-only** on `conclusion_valid_at`; no other briefing edits. |
| **F2 — CLI** | `ai-brains conclusion in-force <TERM> [--scope] [--format] [--principal-id]`. Positional term. No `--local`/`--daemon` (local projection only). No `--as-of`. No `--term` (T324). |
| **F3 — Format** | Default **`json`**. `--format` uses clap `value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]` (T311 F3 / T314 tokens). Unknown → clap `InvalidValue` exit **2**. Then `OutputFormat::parse`: human/pretty/text → Human; markdown/md → Markdown; json/auto → Json (`auto` is not a TTY switch). |
| **F4 — JSON keys** | Frozen object. Always emit `term`, `scope`, `ruling` (`null` or object), `chain` (array, possibly empty). **Do not** `skip_serializing_if` on `ruling`. **No** `as_of` / `next_step`. Ruling object: `conclusion_id`, `statement`, `state` (`"in_force"` only), `updated_at` (RFC3339). **No** `title` (row has none). **No** `approver` (unprojected). Chain link: `conclusion_id`, `status` (`superseded_by:<uuid>`). |
| **F5 — Scope** | `resolve_scope_key_for_cli` (T226). Match **only** `list_conclusions_by_scope_state(Some(scope_key), None)`. Never `list_conclusions_by_scope_state(None, …)` for fallback. Never `conclusions_valid_at`. Successor `row.scope` must equal query scope or chain **stops**. |
| **F6 — Term match** | Trim. Empty/whitespace → `fail_usage` exit **2**. Else (statement only — no title): (1) statement `Term: <term>` case-insensitive exact after prefix; else (2) statement substring. Both case-insensitive. |
| **F7 — Root** | Among scoped matches whose `state` is `Active`, `Confirmed`, `Superseded`, or `Rejected` (skip `Candidate` / `Stale` / `Disputed`), pick **earliest** `valid_from` then `conclusion_id` (matches adapter `ORDER BY valid_from ASC, conclusion_id ASC`). Rejected root with no usable successor → `ruling: null`, `chain: []`. |
| **F8 — Walk** | Follow `superseded_by` while non-empty. Cap **32** hops. Repeat id → `ControlPlaneError::InvalidTransition`, CLI `fail_cp`. Broken id / missing successor → stop; if current is Active\|Confirmed + valid-now keep it, else none. **No** T322 hop-stop (`as_of` is None always). |
| **F9 — Current ruling** | After walk, `ruling` is `Some` iff current `state` is `"Active"` **or** `"Confirmed"` **and** `conclusion_valid_at(now)` (`pub(crate)` helper). `state` JSON is always `"in_force"` when present. Predecessors only in `chain`. Candidate tip (uncorrected propose, or `correct_conclusion` successor not yet activated/confirmed) → none. Stale/Disputed tip → none. |
| **F10 — Policy** | `ReadConclusions` + `production_policy`. Deny → `POLICY_DENIED` + `policy_denied_hint_details()` (T280 omit `--scope`). Exit **3**. |
| **F11 — Clock** | `SystemClock.now()` for valid-time. Tests inject `Clock` if needed; prefer wall + stored `valid_from`. |
| **F12 — Empty unknown** | Authorized + no match: exit **0**, `ruling: null`, `chain: []`. JSON keys stay F4 (**no** `next_step` key). Human: `No in-force ruling for term "<term>".` then `format_authorized_empty_next(None, None)` → `Ungoverned vault search: ai-brains recall "what did we decide"` (T290 helper; no H2). |
| **F13 — Decline daemon / DTO / HTTP** | No `DaemonRequest` / contracts / PROTOCOL-COMPAT row. Soft residual. |
| **F14 — Decline H2 / pins** | Do not promote vault pins. In-force reads **conclusion_projection** only. |
| **F15 — Capture independence** | Projection read only. No new events. |
| **F16 — unwrap** | Production: no `unwrap`/`expect`/`panic`. Rfc3339 format `map_err` into `ControlPlaneError::Query`. |
| **F17 — Test naming** | `function_or_feature__condition__expected_result`. No `test_` prefix. `#[allow(clippy::disallowed_methods)]` on the new test files only. |
| **F18 — FEATURE** | Implement-track starts FEATURE TX + cross-model. |
| **F19 — `ISSUES.md`** | Does not exist. Residuals → `deferred.md`. |
| **F20 — PowerShell `;`** | Not `&&`. |
| **F21 — Help** | `ConclusionCommands` `after_help` **and** `InForce` after_help add `ai-brains conclusion in-force workspace_id`. `help_ia` group list unchanged (already `conclusion`). |
| **F22 — 80-net** | Production net vs **go HEAD**. Test blocks in `main.rs` clap may inflate physical. |
| **F23 — Isolation** | Touch `conclusion_in_force.rs` (new) + `lib.rs` export + CLI `conclusion.rs` + `main.rs` ConclusionCommands clap/dispatch/after_help + `project.rs` visibility-only + CP/CLI tests + CHANGELOG + CAPABILITIES/OPERATIONS one-liners. Do **not** grow `governed_common.rs` / `help_ia.rs` / projector / daemon-api / contracts / `in_force.rs` / `conclusions.rs` production / `adapters.rs` `conclusions_valid_at`. |
| **F24 — Stay-green T311/T322** | Decision `in_force.rs` + CP `tests/in_force.rs` + CLI `tests/decision_in_force.rs` **untouched**. |
| **F25 — last-PR** | `#244` empty. `#237` → T326. `#230` → T325. **No T327.** |
| **F26 — Decline peers** | T322 `--as-of` copy / T324 / T325 / T326 / T307 Blocked / T308 floors / H2 / clap 5 / T240 F2. |
| **F27 — PATH-behind** | T312–T322 not on PATH. T323 hole **is** on PATH. Hermetic/`cargo run` SoT. PATH install not Complete-blocking. |
| **F28 — No live vault writes** | Hermetic CP tests + clap hermetics prove DoD. Do **not** `conclusion propose` / confirm / correct / reject on the operator vault. |
| **F29 — Copy-not-share types** | Do **not** reuse `InForceResponse` / `InForceRuling` (those have `decision_id`/`title`/`approver`/`as_of`). New `ConclusionInForceResponse` / `ConclusionInForceRuling` / `ConclusionInForceChainLink`. |
| **F30 — Decline `--as-of`** | T322 F29 already said copy-not-share. Residual §11. Do not add the flag, parser, or JSON key. |
| **F31 — Decline extra CLI** | No `--quiet` / new format tokens / `--confirm` / `--global` / `--as-memory`. No `conclusion show` / `conclusion list` / CLI `confirm`/`correct`/`activate`. |
| **F32 — `main.rs` physical** | Additive clap variant + dispatch + after_help only. Do not relocate `ConclusionCommands`. |
| **F33 — Docs honesty** | CAPABILITIES Family C row for `conclusion in-force`. OPERATIONS one example. CHANGELOG Unreleased Added. |
| **F34 — help_ia freeze** | Governed list already has `conclusion`. |
| **F35 — Evidence for Confirmed** | CP fixtures that `confirm_conclusion` **must** propose with **non-empty** `evidence_ids` (`UnsupportedCannotConfirm` otherwise). `EvidenceId::new()` without an evidence row is enough (same as `conclusion_commands.rs`). |
| **F36 — Unconfirmed successor** | `correct_conclusion` leaves the successor **Candidate**. Walk lands on Candidate → F9 none (honest). AC1 confirms the successor before asserting a ruling. |
| **F37 — Cycle fixture** | Self-cycle via `correct_conclusion(..., new_conclusion_id: Some(old_id))` after Confirmed (propose+supersede same id). If that InvalidTransitions at implement, stop-before and use a documented test-only hop — do **not** skip AC7. |

---

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | `resolve_conclusion_in_force__superseded_root__current_confirmed_in_force`: C1 Confirmed (statement `Term: workspace_id`) then `correct_conclusion` + confirm C2 → ruling C2, `state=in_force`, chain len 1 C1→C2. Must **fail** today (module missing). |
| **AC2** | `resolve_conclusion_in_force__successor_term__empty_chain`: term matches C2 statement only (`Term: successor_id`) → ruling C2, `chain=[]`. |
| **AC3** | `resolve_conclusion_in_force__rejected_root__none`: reject, no successor → `ruling=None`, empty chain. |
| **AC4** | `resolve_conclusion_in_force__unknown_term__none`. |
| **AC5** | `resolve_conclusion_in_force__empty_term__err` (CP) **and** CLI empty/whitespace → exit **2** (`fail_usage`). |
| **AC6** | `resolve_conclusion_in_force__other_scope_row__not_visible`: row in a different `scope` key does not match. |
| **AC7** | `resolve_conclusion_in_force__cycle__error` (self `superseded_by` via F37). |
| **AC8** | CLI clap: `conclusion in-force --help` lists `<TERM>`, `--scope`, and `--format` possible values. `--format nope` → clap exit **2**. after_help names `conclusion in-force`. Hermetic. Must **fail** today (no subcommand). |
| **AC9** | Policy deny hermetic: no `ReadConclusions` → exit **3**, stdout/stderr `POLICY_DENIED`, hint omits `--scope`. |
| **AC10** | JSON always has `ruling` key (null or object). `seed_read_conclusions` + unknown term is enough. **No** `as_of` key. **No** `next_step` key. |
| **AC11** | `resolve_conclusion_in_force__active_only__in_force`: propose + **activate** (not confirm) → ruling Some, `state=in_force`. Proves F9 Active. |
| **AC12** | `resolve_conclusion_in_force__candidate_only__none`: propose only (no activate/confirm) → `ruling=None`. |
| **AC13** | `resolve_conclusion_in_force__uncorrected_successor_candidate__none`: C1 Confirmed then `correct_conclusion` **without** activating/confirming C2 → `ruling=None` (F36). Chain may be non-empty (hop was taken). |
| **AC14** | Targeted: `cargo clippy -p ai-brains-control-plane -p ai-brains-cli --all-targets -- -D warnings`; nextest those packages (plus new tests). Full workspace gate on implement-track publish, not plan. |
| **AC15** | Docs: CAPABILITIES Family C `conclusion in-force` row; OPERATIONS one example; CHANGELOG Unreleased Added. Grep, not a docs-file hermetic. |
| **AC16** | Manual (on go, after green): `cargo run -p ai-brains-cli -- conclusion in-force --help` lists `<TERM>`. `conclusion in-force workspace_id --format json` → `ruling: null` on **this** live vault (pass-with-observed-data). **Do not** propose to the operator vault. |

---

## 5. Design notes

### 5.1 Algorithm (`resolve_conclusion_in_force`)

1. Reject empty trim.
2. `list_conclusions_by_scope_state(Some(scope_key), None)`.
3. Filter F6 on **statement**; skip non-F7 states; sort `valid_from` then id; take first.
4. Walk F8 (`get_conclusion`); scope check F5.
5. Apply F9 with `clock.now()`.
6. Rejected-root with no usable successor → none + empty chain (F7).

### 5.2 CLI

Copy the **read** shape of `decision.rs` `run_in_force` (no `as_of` branch):

- `fail_usage` on empty term.
- `resolve_scope_key_for_cli` / `parse_scope_key`.
- `ReadConclusions` + `fail_api` / `policy_denied_hint_details`.
- `resolve_conclusion_in_force(&ports.query, &SystemClock, &scope_key, &options.term)`.
- Human: `Term:` / `Scope:` / `Ruling: {statement} ({conclusion_id})` or F12 none + next. Use the statement as-is (fixtures are short). Do **not** add a truncate helper this track.

Propose daemon/local path **untouched**.

### 5.3 Tests without sleep (F35 / F36)

Helper `propose_confirmed(ports, principal, scope, statement)`:

1. `propose_conclusion` with `evidence_ids: vec![EvidenceId::new()]`, `valid_from: None`.
2. `confirm_conclusion` (human).

Helper `correct_and_confirm(...)` calls `correct_conclusion` then `confirm_conclusion` on the new id (F35 evidence on the correct call).

AC13 stops after `correct_conclusion`. AC11 uses `activate_conclusion` (agent) instead of confirm.

Do **not** `sleep`. Do **not** live-propose.

---

## 6. Non-goals

- `--as-of` historical hop-stop (T322 residual; copy-not-share)
- PowerShell empty TERM (T324)
- Daemon `ListInForce` / contracts DTO / PROTOCOL-COMPAT row
- `conclusion list` / `conclusion show` / CLI confirm/correct/activate
- Closing `valid_until` on `ConclusionSuperseded`
- Reusing `conclusions_valid_at` SQL / editing `in_force.rs`
- pin→Confirmed (H2)
- clap 5 / pin bumps / growing `governed_common.rs` except calling existing helpers
- Live vault propose/confirm as Complete-blocking AC
- T325 / T326 / T307

---

## 7. Verification plan

**Red first (TDD):** AC1 CP test on today’s tree (module missing) → fail compile. Then green `conclusion_in_force.rs`. Then AC2–AC7 / AC11–AC13. Then CLI clap + deny hermetics AC8–AC10.

**Manual (on go, after green):** AC16. Record JSON. Do not require live Confirmed chain.

Do **not** require full workspace nextest to finish the **plan**.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Inventing a chain | §2.2 proof; live `correct_conclusion` + projector |
| Mixing Active vs Confirmed | F9 both; AC11 Active-only; AC1 Confirmed after correct |
| Unconfirmed successor false-pass | F36 / AC13 |
| JSON key clash with decision in-force | F29 new types; no `decision_id`/`as_of` |
| Signature break T311/T322 | F24 do not edit `in_force.rs` |
| Growing hotspots | F23; `project.rs` visibility-only |
| Cycle fixture InvalidTransition | F37 stop-before; do not drop AC7 |
| H2 creep | F14 / F12 T290 helper |
| T324 steal | F2 / F5 empty term is whitespace `fail_usage` only |
| Evidence-less confirm | F35 |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-29 (T322 implement residuals through T142). Overlapping **open** rows:

| Item | Disposition |
|------|-------------|
| T311 R5 conclusion in-force | **Absorb** F1–F12 / AC1–AC13 / AC16 — **walker** (chain exists) |
| T311 R2 `--as-of` | **Not stolen** — T322 Completed; conclusion as-of **decline** F30 / residual |
| T311 R1 daemon `ListInForce` | **Decline** F13 — no consumer |
| T311 R3 sibling Approved | **Decline** — N/A (F7 earliest-root freeze analog) |
| T311 R4 `approved_at` column | **Decline** — T322 already declined; conclusions have no approver column |
| T311 R7 PowerShell empty TERM | **Not stolen** T324 |
| T322 implement residuals (PATH / propose→approve gap / AC16 same-tick) | **Not stolen** — T322 Completed; absorb dirty conductor Completed note into this DOCS commit |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T326 `PinnedCountFailed` fake `pinned=0` (`#237`) | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 / T240 F2 | **Not stolen** / **Decline** |
| last-PR Cursor `#244` | **N/A empty** — comments/reviews/issue `[]`. **No T327.** |
| last-PR `#237` / `#230` | **T326** / **T325** already Pending |
| `ISSUES.md` | **Does not exist** |
| Placeholder “decline if no chain” | **Superseded** — §2.2 chain proof |

---

## 10. Implement order (on go)

1. Phase 0: re-read `conclusions.rs` `correct_conclusion` / `confirm` / `reject` / `activate`; projector `ConclusionSuperseded`; `conclusion_valid_at`; clap `ConclusionCommands`; T311 `in_force.rs` as **pattern only** (do not edit); lock clap **4.6.1**; FEATURE TX. **Do not install.** **Do not** live-propose.
2. Red AC1 / AC8 (must fail).
3. Green `conclusion_in_force.rs` + `lib.rs` export + `pub(crate) conclusion_valid_at`. AC1–AC7 / AC11–AC13.
4. CLI `InForce` + `value_parser` `--format` + `run_in_force` + F12 helper. AC8–AC10.
5. Stay-green T311/T322 tests (untouched).
6. CHANGELOG + CAPABILITIES/OPERATIONS (AC15).
7. Targeted clippy/nextest AC14. Implement-track full gate before publish.

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| Conclusion `--as-of` hop-stop | F30 — copy-not-share T322; mint later if an audit needs point-in-time conclusions |
| PATH until owner `cargo install` | F27 — hermetic/`cargo run` SoT |
| Live vault `workspace_id` ruling null | Honesty; AC16 pass-with-observed-data |
| Daemon `ListInForce` | F13 / T311 R1 |
| Long-statement human dump | F12 uses statement as-is; no truncate this track |
| Stale/Disputed not in-force | F7/F9 by design; briefing has separate sections |
| CLI still propose-only besides in-force | confirm/correct stay CP-only (F31) |
| `correct_conclusion` successor is Candidate | F36; operators must confirm/activate before a new ruling |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-control-plane/src/conclusion_in_force.rs` | **New** resolver |
| `crates/ai-brains-control-plane/src/lib.rs` | `mod` + `pub use` |
| `crates/ai-brains-control-plane/src/briefings/project.rs` | `conclusion_valid_at` **`pub(crate)` only** |
| `crates/ai-brains-control-plane/tests/conclusion_in_force.rs` | **New** AC1–AC7 / AC11–AC13 |
| `crates/ai-brains-cli/src/commands/conclusion.rs` | `run_in_force` + options |
| `crates/ai-brains-cli/src/main.rs` | `ConclusionCommands::InForce` + dispatch + after_help |
| `crates/ai-brains-cli/tests/conclusion_in_force.rs` | **New** AC8–AC10 / AC5 CLI |
| `CHANGELOG.md` | Unreleased Added |
| `Docs/CAPABILITIES.md` | Family C row |
| `Docs/OPERATIONS.md` | One example |

Do **not** touch: `governed_common.rs`, `in_force.rs`, `conclusions.rs` production, `ai-brains-store/src/projections/conclusion.rs`, `adapters.rs` `conclusions_valid_at`, `help_ia.rs`, daemon-api, contracts, retrieval, graph, INSTALL.md.
