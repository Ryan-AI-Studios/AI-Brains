# T311 — Decision in-force resolver

- **Track ID:** T311-DecisionInForce
- **Status:** **Planned** (Pending until go)
- **Category:** FEATURE
- **Owner:** Grok
- **Source:** Archived local WIP `track-t95-in-force` (tag `archive/track-t95-in-force` @ `7812b61`). Not last-PR Cursor. Not a `deferred.md` row.
- **Depends on:** T150 decision lifecycle + `superseded_by`; T160 `decision propose`; T203 discovery reads; T210/T241 `ReadDecisions`; T221 deny exit **3**; T226 `--scope` soft-fill; T263 H1 (pins ≠ Approved); T280 omit-`--scope` deny hint; T288/T290 granted-empty `next_step`; T310 PATH graph-on + 4.14 (owner elevated install 2026-08-27).
- **Blocks / feeds:** Operators can ask “what is in force for term X?” without treating vault pins as Approved (H2 still declined).
- **Absorbs:** Archived resolver **with corrections** (layer, scope isolation, ruling `state`, empty-term usage, policy). Tag remains the provenance snapshot — **do not cherry-pick blindly**.
- **Not absorbed (DoD):** T307 dual tower-http; T308 density floors; `recovery_kit_event`; clap 5; T263 H2; `ai-brainsd --version`; conclusion in-force; daemon `ListInForce` wire; projection `approved_at` column; FTS/`list_decisions(None)`; `--global`; INSTALL 0.1.2 header.
- **Research date:** 2026-08-27 (HEAD `bc74098`; PATH elevated install). Snapshot — **re-verify pins at execute**.
- **AI fold-in:** none yet (plan-write). Review-track → `<slug>-review.md`.
- **Ledger:** planning DOCS TX `67c2081c-5040-464e-9214-4022556e7f25`. Implement starts a FEATURE TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install` / `daemon stop` / `sc start` as planning. Do **not** print or commit `AI_BRAINS_KEY`. Never `git push origin main`.

---

## 1. Objective

1. **Resolve the ruling that is in force for a term.** `ai-brains decision in-force <TERM>` walks the `decision_projection.superseded_by` chain (T150) and returns the current Approved, valid-now successor (or honest none).
2. **Stay a governed read.** `ReadDecisions` on the resolved scope. Soft-fill `--scope` like T203/T226. Deny is T221 exit **3** + T280 hint. No new events. No pin→Approved (H2).
3. **Do not ship the archive as-is.** Put the resolver in `ai-brains-control-plane`, not retrieval. Scope-bound matching (no vault-wide `list_decisions(None)`). The **current** node is `in_force`; predecessors live in `chain`. Empty term is usage exit **2**.
4. **Capture independence.** Projection query + CLI. No models, no graph, no embeddings.

---

## 2. Live baseline (re-scan 2026-08-27)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `bc74098` T310 `#228` on `main`. Branch `track/T311-decision-in-force`. Tree CLEAN at plan-write. |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,842,112** B; LastWriteTime **2026-08-27 05:52:13**. `ai-brains 0.1.3`. Owner **elevated** `cargo install` (non-elevated Access denied — T306 R1). |
| PATH `ai-brainsd.exe` | **22,377,984** B; LastWriteTime **2026-08-27 05:51:37**. (T310 OR-path was 22,173,184 B / 12:04:58 AM.) |
| `doctor --summary` | `degraded`. `cipher_page` **`cipher_version=4.14.0 community`**. `graph_feature=available`. `graph_density` sparse E/N **0.408**. `recovery_kit_event` warn. JSON **omits** `graph_density.remediation` (T308 live on PATH). |
| `daemon status` | **Running** PID **15200**; vault `C:\dev\ai-brains\vault.db`; LLM/Embedding TCP Open. |
| `ai-brainsd --version` | `Error: Missing` — **T310 F15 stands; do not add**. |
| `decision` CLI | **Propose only.** `DecisionCommands` is a single variant. PATH `decision --help` has no `in-force`. |
| Archive tag | `archive/track-t95-in-force` → `7812b61`. Unique vs `main`: `in_force.rs` in **retrieval**, CLI `run_in_force`, tests that label the successor `Superseded`. |
| Last GitHub PR | [#228](https://github.com/Ryan-AI-Studios/AI-Brains/pull/228) T310. `pulls/228/comments`, `/reviews`, `issues/228/comments` all **`[]`**. **last-PR Cursor: N/A. No T312.** Open PRs: **none**. |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX). |
| `ISSUES.md` | **Does not exist.** |
| Hotspots | `project.rs` #1; `governed_common.rs` **#3**. `decision.rs` / `decisions.rs` **not** top-10. **F1:** new CP module; do not grow `governed_common.rs`. |

### 2.2 Why the archive still matters (and what is wrong)

| Item | Why it is still a product hole / why correct it |
|------|--------------------------------------------------|
| No in-force CLI | Briefing lists **Approved** in a scope (T152/T288). It does not answer “for term X, after supersession, what governs?” **DoD.** |
| Retrieval crate | Archive put `resolve_in_force` in `ai-brains-retrieval`. Retrieval already depends on CP, but in-force is a **governed query**, not FTS/semantic. **CP module.** |
| `list_decisions(None)` fallback | Archive title/statement scan is vault-wide → leftover `7d97a456` can leak into `3581317d`. **Scope-bound only.** |
| Ruling `state: Superseded` | Archive test `in_force_resolves_latest_superseding_decision` asserts D2.`state == Superseded` because it followed D1. The **current** ruling is in force; chain records D1→D2. **Do not copy.** |
| Empty term → exit 0 | T261/T252: contentless is usage. **Exit 2.** |
| No policy | Archive used `StoreGovernedQuery` with no `ReadDecisions`. T203/T210 require the grant. |
| `approved_at` = `updated_at` | Projection has **no** `approved_at` column (`DecisionApproved` writes `approver` + `updated_at` only). JSON key **`updated_at`**. Do **not** add a column. |
| Daemon wire | No `DaemonRequest` for in-force. T159 queries are off-queue; local projection is enough. **Local-only this track.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| CLI enum | `main.rs` `DecisionCommands::Propose` `:2482–2515` | Add `InForce { term, scope, format, principal_id }`. Dispatch beside `:4513`. |
| CLI handler | `commands/decision.rs` | Propose-only today. Add `run_in_force`. Copy **read** pattern from `source.rs` `run_list_local` (`resolve_scope_key_for_cli`, `production_policy`, `ReadDecisions`, `fail_api` + `policy_denied_hint_details`). |
| Lifecycle | `control-plane/src/decisions.rs` | `propose` / `approve` / `supersede` / `revoke`. Reuse in tests (not raw events). |
| Row | `ports.rs` `DecisionRow` | `state`, `title`, `statement`, `scope`, `approver`, `valid_from`/`until`, `recorded_at`, `updated_at`, `superseded_by`. |
| List | `adapters.rs` `list_decisions` `:725` | `AND scope = ?`; `ORDER BY recorded_at ASC, decision_id ASC`. |
| Valid-time | `briefings/project.rs` `decision_valid_at` `:695` **private** | Copy 4 lines into CP `in_force.rs`. Do **not** refactor briefing this track. |
| Successor helper | `conflicts.rs` `current_successor` | Conclusions only. Do **not** overload. |
| Policy | `GrantCapability::ReadDecisions` | Same as briefing/progressive. |
| Help IA | `help_ia.rs` | Governed already names `decision`. Add `in-force` example on `DecisionCommands` `after_help` only. |
| Mock port | `tests/ports_are_implementable.rs` | Trait unchanged — stay-green. |
| Archive | tag `7812b61` | Provenance only. |

### 2.4 Dependency / standards research (2026-08-27) — snapshot, re-verify at execute

| Pin | Workspace / lock | crates.io / docs | Action |
|-----|------------------|------------------|--------|
| clap | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6**; docs.rs 4.5 derive `Subcommand` | **No bump.** Add a variant. clap **5** forbidden. |
| serde / serde_json | workspace 1.0 / lock **1.0.150** | — | **No bump.** CLI JSON from CP structs (`serde` already on CP). |
| time | workspace **0.3** / lock **0.3.47** | crates.io **0.3.55**; `Rfc3339` already used in projections | **No bump.** `format` → `map_err`, never `unwrap`. |
| thiserror | workspace **2.0** / lock **2.0.20** (T302) | — | Reuse `ControlPlaneError`. |
| rusqlite | **0.40.2** / SQLCipher **4.14.0 community** | T305/T306/T310 | **No bump.** No new SQL. |
| tokio | workspace **1.53** / lock **1.53.1** | — | **No bump.** |
| uuid | workspace **1.13** | — | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged. |
| workspace version | **0.1.3** | INSTALL.md header still **0.1.2** | **Decline** docs drift this track. |
| New crates | — | — | **Zero.** |

**In-force query research (primary sources):** SQL Server temporal `AS OF` is `ValidFrom <= t AND ValidTo > t` ([Microsoft Learn](https://learn.microsoft.com/en-us/sql/relational-databases/tables/temporal/overview)). Wikipedia [temporal database](https://en.wikipedia.org/wiki/Temporal_database): valid-time vs transaction-time; current row has open end. CortexDB/XTDB: supersession **closes** the old fact and keeps the chain as a timeline, not as the current value’s status. **Fit:** T150 already stores `valid_from`/`valid_until` + `superseded_by`. Current = follow chain to a node with no `superseded_by`, `state == Approved`, and `decision_valid_at(now)`. Chain is the timeline. Do **not** mark that node superseded.

N/A: no Windows schtasks / SQLCipher pin change.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Layer** | `resolve_in_force` lives in `ai-brains-control-plane/src/in_force.rs` (export from `lib.rs`). **Not** retrieval. Do not cherry-pick archive `in_force.rs`. Do not grow `governed_common.rs` / `project.rs`. |
| **F2 — CLI** | `ai-brains decision in-force <TERM> [--scope] [--format] [--principal-id]`. Positional term. No `--local`/`--daemon` (local projection only). |
| **F3 — Format** | Same tokens as `decision propose` (`OutputFormat::parse`). Default **`json`** (sibling T160). Human/pretty/text/markdown/md → two-block human. Unknown → clap exit **2**. |
| **F4 — JSON keys** | Frozen object. Always emit `term`, `scope`, `ruling` (`null` or object), `chain` (array, possibly empty). **Do not** `skip_serializing_if` on `ruling`. Ruling object: `decision_id`, `title`, `statement`, `state` (`"in_force"` only), `approver` (string, empty if none), `updated_at` (RFC3339). Chain link: `decision_id`, `status` (`superseded_by:<uuid>`). |
| **F5 — Scope** | `resolve_scope_key_for_cli` (T226). Match **only** `list_decisions(Some(scope_key), None)`. Never `list_decisions(None, …)` for fallback. Successor `row.scope` must equal query scope or chain **stops** (anti-enumeration / leftover `7d97a456`). |
| **F6 — Term match** | Trim. Empty/whitespace → `fail_usage` exit **2**. Else: (1) title `Term: <term>` case-insensitive exact after prefix; else (2) title substring; else (3) statement substring. Both case-insensitive. |
| **F7 — Root** | Among scoped matches whose `state` is `Approved`, `Superseded`, or `Revoked` (skip `Proposed`), pick **earliest** `recorded_at` then `decision_id` (matches adapter ORDER BY). Revoked root with no usable successor → `ruling: null`, `chain: []`. |
| **F8 — Walk** | Follow `superseded_by` while non-empty. Cap **32** hops. Repeat id → `ControlPlaneError` (invalid state), CLI `fail_cp`. Broken id / missing successor → stop; if current is Approved+valid-now keep it, else none. |
| **F9 — Current ruling** | After walk, `ruling` is `Some` iff current `state == "Approved"` **and** `decision_valid_at(now)` (copied helper). `state` JSON is always `"in_force"` when present. Predecessors only in `chain`. |
| **F10 — Policy** | `ReadDecisions` + `production_policy`. Deny → `POLICY_DENIED` + `policy_denied_hint_details()` (T280 omit `--scope`). Exit **3**. |
| **F11 — Clock** | `SystemClock.now()` for valid-time. Tests inject `Clock`. |
| **F12 — Empty unknown** | Authorized + no match: exit **0**, `ruling: null`, `chain: []`. Human: `No in-force ruling for term "<term>".` last line `next: ai-brains recall "what did we decide"` (T290 copy-paste; no H2). |
| **F13 — Decline daemon** | No `DaemonRequest` / contracts DTO / HTTP route. Soft residual. |
| **F14 — Decline H2 / pins** | Do not promote vault pins. In-force reads **decision_projection** only. |
| **F15 — Decline `--global` / FTS / conclusion in-force / `--version` / clap 5 / T307 / floors / recovery kit / INSTALL 0.1.2** | Written. |
| **F16 — Archive tests** | Do **not** assert successor `InForceState::Superseded`. CP tests use `decision_commands.rs` `open_ports` + real `propose`/`approve`/`supersede`/`revoke`. `#[allow(clippy::disallowed_methods)]` on that test file only (same as peers). |
| **F17 — unwrap** | Production: no `unwrap`/`expect`/`panic`. RFC3339 `format` `map_err` into `ControlPlaneError::Query`. |
| **F18 — Help** | `DecisionCommands` `after_help` adds `ai-brains decision in-force workspace_id`. `help_ia` group list unchanged (already `decision`). |

---

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | `resolve_in_force__superseded_root__current_approved_in_force`: D1 Approved then superseded by D2 Approved; term matches D1 title `Term: workspace_id` → ruling D2, `state=in_force`, chain len 1 D1→D2. |
| **AC2** | `resolve_in_force__successor_term__empty_chain`: term matches D2 only → ruling D2, `chain=[]`. |
| **AC3** | `resolve_in_force__revoked_root__none`: revoked, no successor → `ruling=None`, empty chain. |
| **AC4** | `resolve_in_force__unknown_term__none`. |
| **AC5** | `resolve_in_force__empty_term__err` (CP) **and** CLI empty/whitespace → exit **2** (`fail_usage`). |
| **AC6** | `resolve_in_force__other_scope_row__not_visible`: row in a different `scope` key does not match. |
| **AC7** | `resolve_in_force__cycle__error` (self `superseded_by` or 2-cycle). |
| **AC8** | CLI clap: `decision in-force --help` lists `<TERM>` and `--scope`. Hermetic. |
| **AC9** | Policy deny hermetic: no `ReadDecisions` → exit **3**, stdout/stderr `POLICY_DENIED`, hint omits `--scope`. |
| **AC10** | JSON always has `ruling` key (null or object). Fixture snapshot or `serde_json` pointer. |
| **AC11** | Targeted: `cargo clippy -p ai-brains-control-plane -p ai-brains-cli --all-targets -- -D warnings`; nextest those packages (plus new tests). Full workspace gate on implement-track publish, not plan. |

---

## 5. Design notes

Algorithm (single function, generic over `GovernedQueryStore` + `Clock`):

1. Reject empty trim.
2. `list_decisions(Some(scope_key), None)`.
3. Filter F6; skip `Proposed`; sort recorded_at/id; take first (F7).
4. If none or revoked-without-successor walk → none.
5. Walk F8; scope check F5.
6. Apply F9 with `clock.now()`.

CLI maps `ControlPlaneError::PolicyDenied` through existing `fail_api` / `fail_cp` so exit codes stay T201/T221.

Human (non-json): `Term:` / `Scope:` / `Ruling:` one line or the F12 none + `next:`.

---

## 6. Non-goals

- `decision list` / `decision show`
- Conclusion in-force
- `--as-of` historical valid-time (current `now` only)
- Daemon / HTTP / contracts DTO
- Changing `DecisionApproved` projection
- Graph edges / nightly / pin
- `cargo install` as DoD
- Minting T312 (last-PR empty)

---

## 7. Verification plan

**Red first (TDD):** AC1 CP test on today’s tree (module missing) → fail compile. Then green `in_force.rs`. Then AC2–AC7. Then CLI clap + deny hermetics AC8–AC10.

**Manual (on go, after green):**

```powershell
ai-brains decision in-force --help
ai-brains decision in-force workspace_id --format json
ai-brains decision in-force "" --format json
```

Record JSON shape (likely `ruling: null` on live vault — few Approved decisions). Deny path: principal without grant **or** skip if live grants 3/3 and note hermetic AC9 as proof.

Do **not** require full workspace nextest to finish the **plan**.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Cherry-pick archive | F1 / F16; rewrite in CP |
| Cross-project leak | F5 scope equality on every hop |
| Infinite chain | F8 cap + cycle set |
| Format `unwrap` | F17 `map_err` |
| Growing hotspots | F1 new file; CLI `decision.rs` + `main.rs` only |
| H2 creep | F14 |
| Daemon prefer breaking reads | F13 local-only |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-27 (header through T142 / T192 closeout). Overlapping **open** rows:

| Item | Disposition |
|------|-------------|
| Archived T95 in-force WIP (tag `7812b61`; not in deferred.md) | **Absorb** this plan |
| T310 R1 self-replace os error 5 / elevated install | **Partial** — recorded as live PATH evidence; not DoD |
| T310 R2 `ai-brainsd --version` Missing | **Decline** — T310 F15 |
| T308 R1 live E/N ~0.41 | **Decline** — floors frozen |
| T308 R3 PATH remediator rebuild | **Done** — elevated PATH JSON omits `remediation` |
| T307 dual tower-http Blocked | **Not stolen** |
| T306 R5 `recovery_kit_event` | **Decline** |
| T306 R6 INSTALL 0.1.2 vs workspace 0.1.3 | **Decline** |
| T263 H2 pin→Approved | **Decline** F14 |
| clap 5 / Cargo `default = []` | **Decline** |
| last-PR Cursor `#228` | **N/A empty** — comments/reviews/issue comments `[]`. **No T312.** |
| T240 F2 silent Scope switch | **Decline** — standing; `--scope` soft-fill only |
| T218 F18 DECISION-line boost | **Decline** — recall ranking, not in-force |
| Dependabot close hygiene | **Decline** — standing |

---

## 10. Implement order (on go)

1. Phase 0: re-read `DecisionCommands`, `list_decisions`, `decision_valid_at`; lock clap **4.6.1** / rusqlite **0.40.2**; PATH mtimes; FEATURE TX. **Do not install.**
2. Red AC1 (missing module).
3. Green `in_force.rs` + `lib.rs` export. AC2–AC7.
4. CLI `InForce` + `run_in_force` + help. AC8–AC10.
5. CHANGELOG + CAPABILITIES/OPERATIONS one-liners.
6. Targeted clippy/nextest AC11. Implement-track full gate before publish.

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| R1 — daemon `ListInForce` | F13 |
| R2 — `--as-of` | Non-goal |
| R3 — sibling Approved same term | Earliest-root F7; no conflict packet |
| R4 — `approved_at` column | Projection honesty |
| R5 — conclusion in-force | Non-goal |
| R6 — PATH until next elevated install | Source SoT after go |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-control-plane/src/in_force.rs` | **New** resolver |
| `crates/ai-brains-control-plane/src/lib.rs` | `mod` + `pub use` |
| `crates/ai-brains-control-plane/tests/in_force.rs` | AC1–AC7 |
| `crates/ai-brains-cli/src/main.rs` | `InForce` variant + dispatch |
| `crates/ai-brains-cli/src/commands/decision.rs` | `run_in_force` |
| `crates/ai-brains-cli/tests/` (existing hermetic help or new `decision_in_force`) | AC8–AC10 |
| `Docs/CHANGELOG.md` | Unreleased Added |
| `Docs/CAPABILITIES.md` / `Docs/OPERATIONS.md` | One-line |

Do **not** touch: retrieval, `governed_common.rs` (call existing helpers only), `project.rs`, `decision_projection`, daemon-api, contracts, graph, INSTALL.md version header.
