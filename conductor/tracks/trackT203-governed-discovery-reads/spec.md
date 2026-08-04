# T203 — Governed Discovery Read Paths

- **Track ID:** T203-GovernedDiscoveryReads
- **Phase:** Post-T202 CLI UX series (P2)
- **Status:** ✅ **Completed** (2026-08-04) — PR #86 squash-merged `2748d12`
- **Depends on:** T152/T160 governed surfaces; **T201** exit contract (F27 boundary); **T202** progressive fail_usage + soft-resolve handoff
- **Blocks / feeds:** Operator discovery without memorizing IDs; **T204** help IA may link list examples; residual progressive soft-fill optional
- **Category:** FEATURE / CONTRACTS (additive list DTOs + daemon variants)
- **Source:** CLI audit 2026-08-02 P2 — scores **&lt;7**: `evidence show` / `source show` (E5, no list); `review list` ceremony (E5/C6); T201 F36 leave (evidence/source missing-scope still exit-6 class); T202 F12 soft-resolve deferred here
- **Deferred absorbed:** Governed source/evidence discovery lists (deferred.md); soft scope resolve residual from T202; T201 F27 soft-default boundary; soft T198 O4 project-list JSON **not** core DoD
- **Not absorbed:** Approval workflows; multi-tenant IdP; grant admin UX; ranking/retrieval model changes; full progressive multi-format UI; force single error envelope; MSI; T204 help grouping; clap pin bump
- **Research date:** 2026-08-04 (expand + live re-scan + online)
- **AI fold-in:** 2026-08-04 — AI1 affirms ports/soft-resolve/E1/policy; AI2 **M1–M6** accepted; **L1–L8** notes. Disposition §14.
- **Ledger:** plan-only until implement (`ledgerful ledger start` on go)

## 1. Objective

1. **Read-only discovery** so operators and agents are not stuck with show-by-id only:
   - `source list` (scoped, bounded)
   - `evidence list` (scoped, bounded; optional FTS `--query`)
2. **`review list` soft-default** when `--scope` omitted: fill from authoritative `scope resolve` **or** fail with **USAGE exit 2** + suggested key — **never** reintroduce exit-6 missing-scope (T201 **F27**).
3. **Shared scope resolution helper** for list (and soft show) paths so ceremony is one place.
4. Empty governed lists use **E1** empty arrays (`items: []` / existing list fields) + deny hints where applicable.

## 2. Live baseline (re-scan 2026-08-04)

### 2.1 Commands — show-only discovery gap

| Command | Live state | Residual |
|---------|------------|----------|
| `source show <id>` | Local/daemon; **`--scope: Option`**; missing → `INVALID_PAYLOAD` → exit **6** | No list; F36 leave from T201 |
| `evidence show <id>` | Same Option + exit-6 missing class | No list/search |
| `review list` | **`--scope: String` required (T201 F4)**; missing → clap exit **2**; deny + `details.hint` (T201 F6) | No soft-fill from cwd/project |
| `scope resolve` | Local/daemon; returns `ScopeResolvedResponse` with `authoritative`, `confidence`, `alternatives` | Exists — **reuse**, do not reimplement |
| `policy show` | Scope required; empty grants OK | Out of scope except deny hint pattern |

### 2.2 Ports / store — no list-by-scope for source/evidence

| Surface | Live | Gap |
|---------|------|-----|
| `GovernedQueryStore` | `get_source`, `find_source`, `list_open_review_items`, `list_conclusions_by_scope_state`, `list_decisions` | **No** `list_sources_for_scope` / `list_evidence_for_scope` |
| Projections | `source_projection` indexed on `scope`; `evidence_projection` + **`evidence_fts`** (FTS5 on summary) | SQL ready for list/search |
| Contracts | `SourceDto`, **`SourceListResponse`** (`sources: Vec`) exists but **unused** on wire | No `ListSourcesRequest`; no evidence list DTO |
| Daemon | `InspectSource`, `InspectEvidence`, `ListReviewItems` | No ListSources / ListEvidence |
| HTTP/desktop | Inspect routes only for source/evidence | Additive later; CLI+daemon SOOT for T203 |

### 2.3 Routing map

| File | Role |
|------|------|
| `ai-brains-cli/src/main.rs` | `SourceCommands` / `EvidenceCommands` / `ReviewCommands` clap |
| `ai-brains-cli/src/commands/source.rs` | show only |
| `ai-brains-cli/src/commands/evidence.rs` | show only |
| `ai-brains-cli/src/commands/review.rs` | list + resolve; scope required |
| `ai-brains-cli/src/commands/scope.rs` | resolve |
| `ai-brains-cli/src/commands/governed_common.rs` | `fail_usage`, `policy_denied_hint_details`, exit map, path policy |
| `ai-brains-control-plane/src/ports.rs` + `adapters.rs` | query trait + SQLite impl |
| `ai-brains-control-plane/src/scope_resolver.rs` | `resolve_scope`, `is_authoritative` |
| `ai-brains-contracts/src/sources.rs` | SourceDto / SourceListResponse |
| `ai-brains-contracts/src/review.rs` | ReviewQueueResponse E1 `items: []` |
| `ai-brains-daemon-api` + `ai-brainsd` services/dispatch | protocol variants |
| Docs CAPABILITIES / OPERATIONS / CHANGELOG / CLI-EXIT-CODES | honesty |

## 3. Research summary (2026-08-04)

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — ease of discovery: suggest next commands; errors rewrite with next action; conversation as norm | Soft-resolve + fail_usage with copy-paste `source list --scope …`; after_help examples |
| clig — human-first TTY; machine JSON; empty success OK when intentional | Empty list exit **0** with `items: []` (E1); deny still exit **3** |
| clig — noun/verb consistency; list/search discovery family | `source list`, `evidence list` (+ optional `--query`); soft `evidence search` alias if free |
| clap **4.6.5** on crates.io; workspace pin **4.5** + `env` | **No pin bump** (F16) — 4.5 sufficient for Option + flags |
| T201 **F27** | Soft-default **must not** reintroduce exit-6 missing-scope. Allowed: fill-before-run from resolve; or reopen Option with **USAGE exit 2** only |
| T202 `fail_usage` + progressive exit 2 | Reuse `fail_usage` for non-authoritative soft-resolve failures |
| `ScopeResolvedResponse.authoritative` + `is_authoritative()` | **Gate** for silent fill: only when authoritative + non-empty scope key |
| `evidence_fts` already in migration 0020 | Prefer FTS for `--query`; plain list when query omitted |
| `SourceListResponse` unused on wire | E1 rename to `items` safe **if** `#[serde(alias = "sources")]` (M1) |
| `sanitize_fts_query` in `ai-brains-retrieval` | Reuse — **do not** add retrieval→control-plane dep (M4); lift or call from CLI/adapters carefully |
| Capture independence | List/search are projection reads — **no models** |
| AI2 hermetic env | `--no-project-context` does not clear shell `AI_BRAINS_PROJECT_ID`; hermetic_bin must control env for AC4/AC5 (M2) |

## 4. Frozen decisions (F1–F42)

| ID | Decision |
|----|----------|
| **F1 — Read-only only** | New work is **list/search read paths** only. No propose/resolve/observe/register source in this track. |
| **F2 — Commands (L3)** | **`source list`**, **`evidence list`** with optional `--query`. Prefer separate **`evidence search`** subcommand that **requires** `--query` and delegates to the same handler (not a clap `alias` — arg requirements differ). Soft: omit `search` if schedule slips; list alone is DoD. |
| **F3 — Scope requirement class** | Lists need a scope key **before** policy + query. Sources of scope: (1) explicit `--scope`; (2) soft-resolve fill when authoritative. Missing + non-authoritative → **`fail_usage` exit 2**, never exit 6. |
| **F4 — Review list soft-default (F27)** | Reopen CLI `review list --scope` to **`Option<String>`** (clap not required). Resolution order: explicit `--scope` wins → else resolve cwd/project → if `authoritative` fill → else `fail_usage` with suggested key / alternatives / copy-paste. **Daemon/HTTP wire** already has `ListReviewItemsRequest.scope: Option` (L6 — no wire change); CLI always sends `Some(key)` after fill (or fails before send). **Do not** restore exit-6 missing-scope path on CLI. |
| **F5 — Soft-resolve gate** | Silent fill **only** when `is_authoritative(&resolved)` (or wire `authoritative == true`) **and** scope key non-empty. Low / Ambiguous / Personal-not-auto → **no silent fill**. |
| **F6 — Shared helper (M3)** | One helper in CLI (`governed_common::resolve_scope_key_for_cli` or sibling). Inputs: explicit `Option<&str>` scope + ability to build `ScopeResolveInput` (cwd, project_id, force_personal, …). **Reuse** `resolve_scope` / `is_authoritative` — do not reimplement. **Return type freeze:** `Result<String, Box<dyn std::error::Error>>` (or a small enum with Cp vs usage); **call `fail_usage` at the call site** when usage class — do **not** invent a non-existent `FailUsage` type. **`--no-project-context`:** soft-resolve **still runs** (no special short-circuit); without project id / env it is typically non-authoritative → natural `fail_usage` exit 2. Note: flag clears `.env` load only — shell `AI_BRAINS_PROJECT_ID` may still make resolve authoritative (M2). Used by **list** paths; show via F7. |
| **F7 — Show missing-scope (M6 preferred include)** | Apply F6 helper to **`source show` / `evidence show`**. Insertion points: replace the `match options.scope.as_deref() { None => fail_api INVALID_PAYLOAD }` blocks in `source.rs` (~59) and `evidence.rs` (~55) **before** `parse_scope_key` / `expand_handle`. Soft-fill or exit **2** (not 6). Preferred include — closes T201 F36 leave. |
| **F8 — Response shapes + serde alias (M1)** | **Evidence list:** new `EvidenceListResponse { api_version, items: [], more_available, warnings: [] }` + `EvidenceListItemDto`. **Source list:** rename `SourceListResponse.sources` → **`items`** for E1 parity with review. **Required:** `#[serde(alias = "sources")]` on `items` so old JSON still deserializes. AC9 must assert alias roundtrip (deserialize with `sources` and with `items`). Empty = empty array, never null. |
| **F9 — Bounds + more_available (M5)** | Default **`--limit` = 50**; hard clamp **max 200**. Deterministic order: `recorded_at`/`last_observed_at` DESC, id ASC. **Pagination idiom:** query **`LIMIT (limit + 1)`**; if `rows.len() > limit`, set **`more_available = true`** and truncate to `limit`. Do **not** rely on separate COUNT unless already free. AC15 asserts more_available when seed exceeds limit. |
| **F10 — Evidence query + sanitizer (M4)** | `evidence list --query` optional. Non-empty = FTS: `evidence_fts MATCH ?` → join projection → `source_projection.scope = ?`. **Sanitizer:** reuse `sanitize_fts_query` (today in `ai-brains-retrieval`). **Preferred:** **lift** to `ai-brains-core` (or tiny shared module both crates already use) so control-plane adapters do **not** depend on retrieval (capture-independence / dep graph). **Decline:** adding `ai-brains-retrieval` as control-plane dependency. Alt if lift blocked: sanitize at CLI boundary and pass pre-sanitized query into port (document in plan B2). |
| **F11 — Policy + deny hint (L5)** | List source/evidence requires **`ReadEvidence`**. Review list keeps **`ReadConclusions`**. Deny → exit **3** + **`details.hint`** via `policy_denied_hint_details()` on **all new list paths**. Soft: backfill same hint on **source/evidence show** deny (T201 residual non-universal hint) when F7 touched. |
| **F12 — Anti-enumeration** | Scope isolation mandatory: only rows whose owning scope key matches resolved scope. Wrong-scope IDs never leak across list (same class as show NOT_FOUND / deny). |
| **F13 — Daemon + contracts additive** | Add `ListSourcesRequest` / response + `ListEvidenceRequest` / response; daemon `DaemonRequest`/`Response` variants; services + dispatch + protocol wire tests. HTTP route soft if free (`/v1/sources/list`, `/v1/evidence/list`). Desktop **not** required DoD. |
| **F14 — Capture independence** | No embeddings, no models, no graph required for list/search. No control-plane→retrieval dep (F10). |
| **F15 — Zero new crates / no pin bumps** | Stay on workspace clap 4.5 / is-terminal 0.4. No clap 4.6. Sanitizer lift is refactor, not a new crate. |
| **F16 — Progressive soft-fill (soft)** | Optional progressive project-id fill **not** required DoD. Prefer leave progressive as-is. |
| **F17 — Project list JSON (soft decline as DoD)** | T198 O4 remains soft; **not** T203 acceptance. Prefer **T204** / later. |
| **F18 — Hermetic locks (≥8)** | (1) source list empty exit 0; (2) source list happy; (3) evidence list empty; (4) evidence FTS query hit; (5) review soft-resolve **authoritative success** AC4 with controlled hermetic env; (6) review non-authoritative **exit 2** + fail_usage template (not clap text) AC5; (7) POLICY_DENIED + hint; (8) **more_available** when over limit AC15. Soft: F7 show exit 2; Active-only AC16; serde alias AC9. |
| **F19 — High findings (pre-ship)** | Exit-6 reintroduction; unbounded list; cross-scope leak; silent fill Ambiguous; list without policy; retrieval dep on control-plane; hermetic env flaky AC4/AC5; production unwrap. |
| **F20 — Series** | After T202. Coordinate `main.rs` with T204. |
| **F21 — Determinism** | Stable sort; stable fail_usage templates; sort emitted collections. |
| **F22 — Review** | FEATURE + CONTRACTS; primary required; cross-model soft-required when protocol + soft-default both land. |
| **F23 — Docs** | CAPABILITIES; OPERATIONS; CHANGELOG **minor**; CLI-EXIT-CODES soft-resolve → **2** (runtime fail_usage, not only clap). |
| **F24 — Status filter default Active (L7)** | **Default** SQL includes **`AND status = 'Active'`** (columns present on both projections). Soft optional `--status` override if free. Non-Active rows **excluded by default** — AC16. Review keeps existing Open filter. |
| **F25 — No ranking / retrieval redesign** | No hybrid embed search. |
| **F26 — Privacy (L8)** | List paths: **policy capability** (`ReadEvidence` / `ReadConclusions`) + **status filter** (F24). **Row-level privacy column filtering is out of scope** for T203 (no privacy-aware list helper today; expand_handle uses PolicyContext, not projection privacy filter). Soft: exclude non-Active/erased via status. No vault keys in output. |
| **F27 — T201 boundary compliance** | CLI missing-scope class for review/list/show (after F7) is **2**, not **6**. Daemon None arms may still INVALID_PAYLOAD — F35 honesty. |
| **F28 — Human output** | Json default. Human: one line per item + empty `(none)`. Soft L1: empty list stderr next-command hint not DoD. |
| **F29 — after_help (L2)** | Examples: list with `--scope`; second line omit `--scope` when project context authoritative. Coordinate T204 regroup. |
| **F30 — Port trait growth** | `list_sources_for_scope` + `list_evidence_for_scope` on `GovernedQueryStore`; mocks + adapters. |
| **F31 — Summary truncate** | Evidence list summary default **160** chars; not full body dump. |
| **F32 — Write path unchanged** | Observe/register/CE/resolve untouched except helper import. |
| **F33 — fail_usage template** | Must include example `--scope Repository:<uuid>` (or resolved suggestion), `ai-brains scope resolve`, non-authoritative not filled. Soft: suggest `source list` / `evidence list` after resolve. |
| **F34 — No --from-context required flag** | Auto soft-resolve when omitted. |
| **F35 — Grep inventory update** | On ship: review list → Option + exit 2; show if F7. |
| **F36 — Contract version** | `api_version: "1"`; additive fields. |
| **F37 — exit_contract path flip (M2)** | Existing `review_list__missing_scope__exit_2` must assert **runtime fail_usage** (stderr template / F33), **not** clap “required argument” text. Same exit **2**. **AC4** new: hermetic env with authoritative project (`AI_BRAINS_PROJECT_ID` or seeded project) + no `--scope` → exit **0**. **AC5**: hermetic with project id **unset** (and `--no-project-context` as needed) → exit **2**. Implementer **must** document how `hermetic_bin()` controls `AI_BRAINS_PROJECT_ID`. Highest regression surface. |
| **F38 — Policy show unchanged** | `policy show` / `erasure request` remain clap-required scope (T201). Only review list (+ list/show discovery) soft-resolve. |
| **F39 — AI1 affirm** | Ports F30, soft-resolve F4/F5, E1 F8, policy F11/F12 — all above. |
| **F40 — FTS lift ownership** | Sanitizer lift (F10) is in-track if small; if blocked, CLI pre-sanitize alt + plan note. |
| **F41 — Soft empty next-command (L1)** | Optional stderr/human hint on empty list → not DoD. |
| **F42 — Show deny hint soft (L5)** | When F7 touches show, add `policy_denied_hint_details()` to bare show denies if free. |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| Governed source/evidence discovery lists (deferred.md) | **Absorb** F1–F3, F8–F13 |
| Soft scope resolve (T202 F12) | **Absorb** F4–F6 |
| T201 F27 boundary | **Absorb** F4, F27 |
| evidence/source missing-scope exit 6 (T201 F36 leave) | **Prefer absorb** F7 |
| POLICY_DENIED list hints | **Absorb** F11 |
| Progressive auto project fill | Soft F16 |
| project list JSON (T198 O4) | Soft decline F17 → T204/later |
| Help IA / OutputFormat matrix | **T204** |
| Grant admin / approve UX | **Decline** |
| conclusion/decision list CLI | Soft out — ports exist; not audit P2 core |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | `source list` with scope (explicit or soft-filled) returns E1 empty or items; exit **0** when allowed | Hermetic |
| **AC2** | `evidence list` scoped empty + happy path; exit **0** | Hermetic |
| **AC3** | `evidence list --query` returns FTS-bounded hits for seeded summary | Hermetic |
| **AC4** | `review list` without `--scope` + **authoritative** hermetic env (F37) → exit **0**, items array present (may be empty) | Hermetic |
| **AC5** | `review list` without `--scope` + non-authoritative hermetic env → **exit 2** via **fail_usage**; stderr matches template class (not clap “required argument”) | Hermetic |
| **AC6** | POLICY_DENIED on list → exit **3** + non-empty `details.hint` (json) | Hermetic |
| **AC7** | Limit default/clamp; no unbounded full-table dump in default path | Unit / hermetic |
| **AC8** | Cross-scope: items only match requested scope key | Unit CP or hermetic |
| **AC9** | Protocol roundtrip new list types; **SourceListResponse** deserializes both `items` and alias `sources` (M1) | nextest |
| **AC10** | Full gate green | Process |
| **AC11** | CHANGELOG minor + CAPABILITIES/OPERATIONS + CLI-EXIT-CODES soft-resolve note | Diff |
| **AC12** | No production `unwrap`/`expect`; no control-plane→retrieval dep; capture independence held | Review |
| **AC13** | Soft AC: show missing-scope soft-fill or exit 2 (F7) | Hermetic if F7 |
| **AC14** | Cross-model or primary review clean on protocol + exit ceremony | Process |
| **AC15** | Seed count &gt; limit → `more_available: true` and items.len() == limit (F9/M5) | Unit / hermetic |
| **AC16** | Non-Active status rows excluded by default list (F24/L7) | Unit |

## 7. Non-goals

- Source register/observe CLI redesign  
- Evidence content envelope dumps / CE wipe  
- Review resolve / grant issuance UX  
- Multi-tenant IdP  
- Embedding-backed evidence search  
- Forcing clap 4.6  
- Desktop UI list screens  
- Project list JSON as hard DoD  
- Progressive multi-format pretty UI  
- Reintroducing exit-6 missing-scope on CLI  
- Unbounded export dumps  
- Row-level privacy projection filtering (F26)  
- control-plane depending on ai-brains-retrieval  

## 8. Handoffs

| To | What |
|----|------|
| deferred.md | Strike discovery lists + soft-resolve residual on ship |
| T201 | F27 honored; F36 inventory note update |
| T202 | Soft-resolve ownership closed |
| T204 | Help examples for list; project list JSON soft; OutputFormat matrix |
| CLI-EXIT-CODES | Soft-resolve → exit 2 |
| CAPTURE | Unchanged independence |
| HTTP/desktop | Optional consumers of new list DTOs |

## 9. Implementation sketch

### 9.1 Query ports (control-plane)

```rust
// GovernedQueryStore — additive
fn list_sources_for_scope(&self, scope_key: &str, limit: usize) -> Result<Vec<SourceRow>>;
fn list_evidence_for_scope(
    &self,
    scope_key: &str,
    query: Option<&str>,
    limit: usize,
) -> Result<Vec<EvidenceListRow>>; // new row type: id, summary, status, source_id, recorded_at
```

SQL sketch:

- Sources: `SELECT … FROM source_projection WHERE scope = ? AND status = 'Active' ORDER BY … LIMIT ?` (request limit+1)
- Evidence list: join `evidence_projection e` → `source_projection s` WHERE `s.scope = ?` AND `e.status = 'Active'`
- Evidence query: sanitized MATCH on `evidence_fts` join projection + scope filter
- more_available: F9 LIMIT+1 idiom

### 9.2 Contracts

```rust
// SourceListResponse { items: Vec<SourceDto> }  // serde(alias = "sources") on items
pub struct ListSourcesRequest { api_version, principal_id?, scope?, limit? }
// EvidenceListResponse { items, more_available, warnings }
pub struct ListEvidenceRequest { api_version, principal_id?, scope?, query?, limit? }
```

### 9.3 CLI

- Clap: `SourceCommands::List`, `EvidenceCommands::List`, soft `EvidenceCommands::Search` (required query)
- `ReviewCommands::List.scope: Option<String>`
- Helper returns `Result<String, _>`; call site `fail_usage` on usage class
- Local path: ports + policy + list + emit
- Daemon path: new request variants; CLI sends filled scope

### 9.4 Daemon

- `services::list_sources` / `list_evidence` mirror show policy gates
- dispatch arms + wire tests
- Defensive None scope → INVALID_PAYLOAD retained for raw IPC (F27 honesty)

## 10. Testing strategy

- **Unit:** port list SQL; FTS hit/miss; LIMIT+1 more_available; Active filter; serde alias
- **Hermetic CLI:** AC4/AC5 with explicit `hermetic_bin` env control for `AI_BRAINS_PROJECT_ID`; exit_contract review test path flip (F37)
- **Protocol:** daemon-api roundtrip new variants
- **Regression:** policy show still clap-required; review no longer clap-required text
- **Manual:** scope resolve; lists; review without scope; deny path

## 11. Risk & stop-before

| Risk | Mitigation |
|------|------------|
| Reopen review scope → exit 6 | F4/F27; AC5 exit 2 + fail_usage text |
| Hermetic env flaky AC4/AC5 | F37 document hermetic_bin PROJECT_ID |
| FTS sanitizer pulls retrieval into CP | F10 lift or CLI pre-sanitize |
| Serde break on sources rename | F8 alias + AC9 |
| Cross-scope leak | F12; AC8 |
| Unbounded FTS | F9 LIMIT+1 |
| Concurrent T204 main.rs | Coordinate |
| Show F7 slip | Prefer include; soft cut |

**Stop-before:** reintroduce exit-6 missing-scope; remove daemon None arms; unbounded list; models on discovery path; control-plane→retrieval dep; scope exceeds list/soft-resolve.

## 12. Docs & closeout

- CAPABILITIES: source/evidence list + review soft-default  
- OPERATIONS: discovery workflow example  
- CLI-EXIT-CODES: soft-resolve / fail_usage  
- CHANGELOG minor  
- deferred.md strike  
- conductor.md → Completed on ship  
- `ai-brains pin` decisions for soft-resolve gate + bounds  

## 13. Verification (on go)

```powershell
ledgerful doctor
ledgerful ledger status --compact
ledgerful ledger start T203-governed-discovery-reads --category FEATURE --message "Governed source/evidence list + review soft-resolve"
# TDD red→green
cargo nextest run -p ai-brains-cli -p ai-brains-control-plane -p ai-brains-contracts -p ai-brains-daemon-api -p ai-brainsd
cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings
# full gate before finalize
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

## 14. AI fold-in disposition (2026-08-04)

| Item | Source | Disposition |
|------|--------|-------------|
| Ports list_* + FTS join | AI1 | **Affirm** F10/F30 — already planned |
| Soft-resolve order + exit 2 never 6 | AI1 | **Affirm** F3–F5, F27 |
| E1 items + limit 50/200 | AI1 | **Affirm** F8–F9 |
| ReadEvidence + scope isolation | AI1 | **Affirm** F11–F12 |
| **M1** serde `alias = "sources"` required | AI2 | **Accept** → F8, AC9 |
| **M2** exit_contract path flip + AC4 env + hermetic PROJECT_ID | AI2 | **Accept** → F37, AC4/AC5 |
| **M3** helper return type; `--no-project-context` still runs resolve | AI2 | **Accept** → F6 |
| **M4** FTS sanitizer lift / no CP→retrieval | AI2 | **Accept** → F10, F14, AC12 |
| **M5** LIMIT+1 more_available | AI2 | **Accept** → F9, AC15 |
| **M6** show insertion points before parse/expand | AI2 | **Accept** → F7 |
| **L1** empty-list next-command hint | AI2 | Soft F41 — not DoD |
| **L2** after_help second example | AI2 | **Accept** F29; T204 coord |
| **L3** evidence search as required-query subcommand | AI2 | **Accept** F2 |
| **L4** items vs sources | AI2 | **Accept** items + M1 alias |
| **L5** deny hint on list (+ soft show) | AI2 | **Accept** F11, F42 |
| **L6** ListReviewItemsRequest already Option | AI2 | **Affirm** F4 |
| **L7** default Active filter + AC | AI2 | **Accept** F24, AC16 |
| **L8** row privacy OOS; policy-only | AI2 | **Accept** F26 |
| Draft AC3 exit 6 | prior | **Rejected** (already) |
| clap 4.6 bump | research | **Decline** F15 |

_Await external AI review file fold-in after expand; freeze set F1–F36 ready for critique._

| Item | Disposition |
|------|-------------|
| Draft AC3 “exit 6 with suggested scope” | **Rejected** — superseded by F4/F27 (exit **2** only) |
| Draft F2 “require scope or --use-resolved-scope” | **Superseded** by auto soft-resolve F4/F34 |
| SourceListResponse unused | **Absorb** F8 reshape if needed |
| T202 progressive soft-fill | Soft F16 |
| Project list JSON | Soft F17 |
