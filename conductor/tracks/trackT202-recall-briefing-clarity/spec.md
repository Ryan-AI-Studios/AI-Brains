# T202 — Recall + Briefing Clarity

- **Track ID:** T202-RecallBriefingClarity
- **Phase:** Post-T201 CLI UX series (P2)
- **Status:** ✅ **Completed** (PR #85 `89ea3ec`, 2026-08-04)
- **Depends on:** T101 recall TTY pretty; T111/T133 hints; T152 briefing + progressive; T198 empty hygiene; **T201** exit contract (prefer exit **2** for missing required project class)
- **Blocks / feeds:** Operator cold-start for recall/briefing/query; **T203** owns scope soft-default / discovery lists (do not implement here); T204 help may link examples + broader TTY format defaults (L2)
- **Category:** FEATURE / CONTRACTS (light additive) / DOCS
- **Source:** CLI audit 2026-08-02 P2 — scores **&lt;7**: `query progressive` (E5/C6), `briefing personal/project` (E6/C6); semantic empty ambiguous when embed backend down
- **Deferred absorbed:** Recall/briefing/query progressive clarity (deferred.md); semantic silent-fail residual; `AI_BRAINS_EMBEDDING_MODEL` ignored on semantic path
- **Not absorbed:** Ranking algorithm changes; new retrieval models; T203 `source list` / `evidence list` / review soft-default; force single error envelope; preflight redesign; full governed TTY format matrix (T204)
- **Research date:** 2026-08-04 (expand + live re-scan + online)
- **AI fold-in:** 2026-08-04 — AI1 affirms F2–F11; AI2 **M1–M7** accepted; **L1/L3–L6/L8** notes; **L2/L5** → T204; **L7/L9** out/affirm. Disposition §14.
- **Ledger:** plan-only until implement (`ledgerful ledger start` on go)

## 1. Objective

1. **Semantic honesty:** when `--semantic` is used, operators and agents can distinguish **no matches** vs **embedding backend unreachable** vs **no stored embeddings** — without breaking capture independence or FTS-only paths.  
2. **Briefing humanization:** denied / empty Project and Personal packets always carry a scannable **one-line why** in markdown (and structured `warnings[].kind` on all deny paths); optional TTY default to markdown.  
3. **Query progressive ceremony:** missing project id fails with a **copy-paste example** and a **USAGE-class exit (2)**; keep clap `env = AI_BRAINS_PROJECT_ID` (already wired). Soft scope-resolve deferred to **T203**.  
4. **Recall TTY pretty:** treat as **already shipped (T101)** — re-verify + document residual only; no re-implementation unless regression.

## 2. Live baseline (re-scan 2026-08-04)

### 2.1 `recall` — mostly strong; semantic gap

| Area | Live state | Residual |
|------|------------|----------|
| TTY default format | `resolve_format(None, tty)` → **pretty** / non-TTY → **json** (`recall.rs`) | **Done T101** — unit tests present |
| Explicit `--format` | Pass-through of string; pretty / json paths | No validation of unknown format strings (soft) |
| Empty FTS | `hint` field (JSON) / stdout hint on TTY pretty (T111/T133) | Keep |
| Empty semantic | Same family of hints *mentions* embedding model | **No structured status**; backend fail looks like empty |
| Semantic failure path | `semantic_search(...).unwrap_or_else` → `eprintln!` + **empty hits**, continue lexical | Agents on JSON never see structured failure; stderr noise |
| Model env | Docs: `AI_BRAINS_EMBEDDING_URL` + `AI_BRAINS_EMBEDDING_MODEL` | **`semantic.rs` hardcodes** `nomic-embed-text-v1.5`; only URL env-read |
| Endpoint default | `http://127.0.0.1:8083` (llama.cpp OpenAI-compat style) | Match nightly/docs |
| Contract | `RecallResponse { results, effective_session_id, hint }` | **No `embedding` field** |
| Exit empty success | exit **0** with empty results + hint | Keep (T198 empty-success class) |

### 2.2 `briefing project|personal`

| Area | Live state | Residual |
|------|------------|----------|
| Default format | **Always json** (`emit_output` unwrap_or `"json"`) | Not TTY-aware (unlike recall/preflight) |
| Markdown | `render_project_markdown` / `render_personal_markdown` — denied blockquote exists | Dense full packet still default on TTY |
| Deny + grant path | Pushes `BriefingWarningDto { kind: "denied", ... }` | Good |
| `empty_denied` helper | Sets `denied=true`, `denial_reason=Some`, **`warnings: []`** | Callers that return bare helper without push **lack kind** (e.g. Project refuses Personal scope) |
| Scope resolve | Project uses `ScopeResolveInput` (cwd / explicit project) | Personal uses user id / principal |
| Examples | after_help all `--format json` | Agent-first; human TTY underserved |

### 2.3 `query progressive` / expand

| Area | Live state | Residual |
|------|------------|----------|
| `--project-id` | clap `Option` + **`env = AI_BRAINS_PROJECT_ID`** | Env already works when set |
| Missing project | `ok_or("project id required (--project-id or AI_BRAINS_PROJECT_ID)")` | Terse; **no copy-paste example**; string error → generic CLI fail (**exit 1** class via `handle_cli_result`) — not USAGE **2** |
| Output | JSON pretty only | Out of scope to add progressive human pretty (optional soft) |
| Soft scope resolve | Not present | **T203** (or later) — do not implement in T202 |
| `query trace` | No `--project-id`; missing → `null` exit 0 | **Excluded** from F10/F11 (L3; T198 empty-success class) |

### 2.4 Routing map

| File | Role |
|------|------|
| `ai-brains-cli/src/commands/recall.rs` | Format resolve; build `RecallResponse`; pretty/json emit; hints |
| `ai-brains-retrieval/src/semantic.rs` | Embed fetch; model hardcode; endpoint env |
| `ai-brains-retrieval/src/recall.rs` | Semantic soft-fail swallow |
| `ai-brains-contracts/src/recall.rs` | Additive embedding status DTO |
| `ai-brains-cli/src/commands/briefing.rs` | Format default; emit |
| `ai-brains-control-plane/src/briefings/{project,personal,renderer}.rs` | Packet deny + markdown |
| `ai-brains-contracts/src/briefings.rs` | `empty_denied` / `BriefingWarningDto` |
| `ai-brains-cli/src/commands/governed_query.rs` | Progressive / expand project gate |
| Docs CAPABILITIES / OPERATIONS / CHANGELOG | Honesty for embedding status + briefing TTY |
| `ai-brainsd` query_memories | Hardcodes `semantic: false` (`lib.rs:273`) | **No daemon semantic surface** — F25 soft-skip (M4) |

## 3. Research summary (2026-08-04)

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — human-first on TTY; JSON for machines; errors rewrite with next action; suggest commands | F1 residual; F10 progressive example; F7–F9 briefing |
| clig: empty success OK when intentional; fail loud on real errors | Keep empty recall exit 0; surface embed *status*, not hard-fail whole recall |
| Embed health practice (Ollama `/api/tags` liveness) | Product path is **llama.cpp-style** at `AI_BRAINS_EMBEDDING_URL` (default `:8083`), not Ollama `:11434`. Prefer **status from actual embed attempt** over inventing Ollama-only probes |
| Capture independence (AGENTS) | Semantic optional; never require models for FTS recall or capture |
| clap 4.5 + `env` feature; is-terminal 0.4 | **No dep bump** (F14); latest 4.6.x / 0.4.x compatible — stay on workspace pins |
| T201 0–7 SOOT + `GovernedCliError` | Missing project → **exit 2** via `fail_usage` (F11/M3) — not clap-required |
| T101 complete | Do not re-plan pretty default as new work |
| dogfood-shadow.ps1 | briefing always `--format json` (lines ~539, 629) — F9 TTY markdown **not BREAKING** (M1) |
| Nightly embed defaults | `nightly.rs` same URL default `http://127.0.0.1:8083` + model env (L1 aligned) |
| CLI tracing | `main.rs` initializes `tracing_subscriber` — F27 `tracing::warn!` is viable (L8) |

## 4. Frozen decisions (F1–F36)

| ID | Decision |
|----|----------|
| **F1 — Recall TTY pretty residual** | **Already shipped (T101).** DoD: unit tests for `resolve_format` still pass; CAPABILITIES/help already accurate or one-line honesty if stale. **No new pretty formatter** unless regression found. Soft: reject unknown `--format` with exit 2 (optional). |
| **F2 — Semantic status field (core)** | When `--semantic`, `RecallResponse` includes additive **`embedding`** object (name freeze: `embedding`). Shape: `{ "status": "<enum>", "endpoint"?: string, "detail"?: string }`. **Statuses (closed set):** `ok` \| `unreachable` \| `error` \| `no_stored_embeddings` \| `skipped`. Omit field or set `skipped` when **not** `--semantic`. No secrets; endpoint is host URL only (env value / default). |
| **F3 — Soft-fail semantics preserved** | Semantic backend failure **must not** abort whole recall. Keep blending FTS/bridge results. Structured field is SOOT for agents; log via `tracing::warn!` (F27) — not status-only-via-eprintln. |
| **F4 — Status derivation (M6/M7)** | Status from **real embed attempt** (no second network probe). Mapping: **connection refused / DNS / timeout / connect errors → `unreachable`**; **HTTP non-2xx / body parse fail / provider error / thread panic → `error`**. After embed **ok**: if `fetch_pinned_embeddings` returns **empty** OR all rows dropped by `bytes_to_f32_vec` (L6) → **`no_stored_embeddings`**; if ≥1 scored hit possible path ran with rows → **`ok`** (even when similarity yields zero hits after truncate — empty results with `ok` means “backend fine, no match”). Soft detail string may note `all_rows_undecodable` vs `zero_rows`. |
| **F5 — Honor `AI_BRAINS_EMBEDDING_MODEL`** | `semantic.rs` **must** read `AI_BRAINS_EMBEDDING_MODEL` with default `nomic-embed-text-v1.5` (parity with nightly). URL continues via `AI_BRAINS_EMBEDDING_URL` default `http://127.0.0.1:8083` (L1 aligned with nightly). |
| **F6 — Hint / pretty precedence (M5)** | **Pretty TTY:** print **one** embedding status line when `--semantic` and `status != ok` (and soft when empty + semantic). **Hint must not repeat the status cause** — focus next action only (`--global`, refine query, import memories). **JSON:** keep short `hint` optional; status field is primary for agents; soft-shorten semantic hint that only said “check embedding model” when status present (L4 — plan notes). |
| **F7 — Briefing warnings (M2)** | Every `denied=true` path **must** have ≥1 `warnings[]` with `kind: "denied"`. **Preferred (freeze):** seed warning inside **both** `empty_denied` helpers (`Project` + `Personal` in contracts); **dedup** call sites that currently push after helper (project grant-deny ~211, partial denies ~284/~364, personal ~120) so no double `denied` warning. **Known missing site:** `project.rs` Personal-scope refuse **~181–188** returns bare `empty_denied` today — fixed by helper seed. AC6 asserts Personal-scope refuse specifically. |
| **F8 — Briefing markdown one-liner** | Keep `> **Denied:** …` (already present); `denial_reason` non-empty on all deny. Soft empty-section copy only if free. |
| **F9 — Briefing TTY default markdown (M1)** | **Freeze preferred path:** TTY + no `--format` → **markdown**; non-TTY → **json**; explicit `--format` wins. **Not BREAKING** for dogfood — `scripts/dogfood-shadow.ps1` always passes `--format json` on briefing. CHANGELOG: **minor** behavior change (not BREAKING). Soft-alt (keep json default) **declined** unless implement finds other bare-default consumers. |
| **F10 — Progressive project message** | Missing project (flag and env unset): message **must** include copy-paste example + `AI_BRAINS_PROJECT_ID`. Same for **`query expand`**. **`query trace` excluded** (L3). Soft T204: after_help examples lack `--project-id` (L5). |
| **F11 — Progressive exit 2 via `fail_usage` (M3)** | **Required:** add `governed_common::fail_usage(msg) -> GovernedResult` that writes usage text to **stderr** and returns `GovernedCliError { exit_code: EXIT_USAGE (2), emitted: true }` — reuses existing `handle_cli_result` downcast (no clap-required `--project-id`). Progressive + expand call it when `project_id` is None after clap env bind. Soft alt: `ApiError` code `"USAGE"` arm in `exit_code_for_api_error` only if Json envelope desired; prefer direct exit_code on `GovernedCliError`. **AC10 must hermetically assert exit 2** (not doc-only). Soft exit-1 fallback **declined** unless blocked. |
| **F12 — No soft scope resolve in T202** | Scope soft-default is **T203**. F10/F11 only. |
| **F13 — No ranking / retrieval model changes** | FTS, cosine, blend order, bridge caps, graph boost untouched. |
| **F14 — Zero new crates / no pin bumps** | No clap 4.6, no new HTTP client. Workspace clap 4.5 / is-terminal 0.4 stay. |
| **F15 — Capture independence** | FTS recall, pin, capture, briefing must not require live embedding server. |
| **F16 — Contracts additive** | `embedding` optional / skip_serializing_if; no breaking RecallResponse consumers. |
| **F17 — Exit empty success** | Empty recall + status field → exit **0**. Embed down is not hard CLI failure for recall. |
| **F18 — Hermetic locks (≥6)** | (1) T101 format residual; (2) semantic mock **connection-refused → `status=unreachable`** specifically (M6), exit 0; (3) semantic ok path `status=ok`; (4) briefing Personal-scope refuse **and** grant-deny → `kind=denied` (M2); (5) progressive missing project: example **+ exit 2** (AC10); (6) soft/required: `no_stored_embeddings` when embed ok + empty rows (M7). Prefer unit map for error-class → status (M6). |
| **F19 — High findings (pre-ship)** | Silent semantic fail without status; model hardcode; bare empty_denied without kind; progressive exit 1 no example; double status+hint print; ambiguous unreachable vs error; ranking changes. |
| **F20 — Series / handoff** | After T201. Parallel OK with T203. Progressive copy not T203. |
| **F21 — Determinism** | Stable status strings; stable usage message template. |
| **F22 — Review** | FEATURE; **primary review required**. Cross-model **soft-required** when F2+F9+F11 all land (contract + default + exit) — prefer one cross-model pass; not blocked solely by F9 (no longer BREAKING). |
| **F23 — Docs** | CAPABILITIES semantic status + model env; OPERATIONS; CHANGELOG **minor** F9 + note F11 exit 2 / additive embedding field. No F9 BREAKING header. |
| **F24 — NDJSON / other formats** | Out of scope. |
| **F25 — Daemon soft-skip (M4)** | Daemon `query_memories` hardcodes `semantic: false` at `ai-brainsd/src/lib.rs:273` — **no parity surface**. CLI local recall is SOOT for `embedding` field. Do not spend effort on daemon IPC semantic status in T202. |
| **F26 — Privacy** | No keys/vault contents/path secrets in `detail`. |
| **F27 — Logging (L8)** | Prefer `tracing::warn!` for semantic fail (subscriber init already in `main.rs`). **Do not** also eprintln the same cause when pretty already printed status (F6). If retrieval runs without subscriber in unit tests, warn is OK no-op. |
| **F28 — Progressive human format soft** | JSON-only progressive remains OK. |
| **F29 — AI1 affirm** | Structured embedding DTO; soft-fail; model env; empty_denied seed; progressive example+exit2 — all above. |
| **F30 — fail_usage message template** | Freeze implement-time exact string including progressive example line + “Or set AI_BRAINS_PROJECT_ID.” Expand may say `query expand`. |
| **F31 — query trace excluded** | F10/F11 do not apply to `query trace` (L3). |
| **F32 — T204 feed (L2/L5)** | Broader governed `OutputFormat::parse` TTY-unaware defaults + progressive after_help `--project-id` examples → **T204** / ISSUES on ship if not already. Not T202 DoD. |
| **F33 — JSON hint soft (L4)** | When `embedding` present, semantic empty JSON hint may drop redundant “check embedding model” clause; optional at implement. |
| **F34 — no_stored_embeddings AC** | AC asserts `no_stored_embeddings` vs `ok` empty-match distinction (M7); unit preferred. |
| **F35 — Unit status map** | Unit test(s): classify sample error strings/kinds → `unreachable` vs `error` (M6). |
| **F36 — F9 dogfood evidence** | Recorded: dogfood-shadow.ps1 briefing uses `--format json` explicitly — F9 safe. |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| Recall/briefing/query progressive clarity (deferred.md) | **Absorb** |
| Semantic silent fail | **Absorb** F2–F4, F6, F27 |
| `AI_BRAINS_EMBEDDING_MODEL` ignored on semantic | **Absorb** F5 |
| TTY recall pretty | **Closed T101** — F1 residual verify |
| Briefing deny scannability | **Absorb** F7–F9 |
| Progressive ceremony | **Absorb** F10–F11; soft-resolve **T203** |
| Ranking / new models | **Decline** F13 |
| source/evidence list | **T203** |
| Help IA | **T204** |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | T101 format defaults still hold (TTY pretty / non-TTY json) | Unit |
| **AC2** | `--semantic` + connection-refused class → `embedding.status=unreachable` (not vague or-error); exit **0**; FTS/bridge may still appear | Hermetic / unit map + one hermetic |
| **AC2b** | Unit: error-class samples map to `unreachable` vs `error` (F35) | Unit |
| **AC3** | `--semantic` + healthy path → `status=ok` | Hermetic / unit |
| **AC3b** | Embed ok + zero stored/decodable embeddings → `no_stored_embeddings` (F34) | Unit |
| **AC4** | Without `--semantic`, field omitted or `skipped` | Unit / hermetic |
| **AC5** | Semantic path uses `AI_BRAINS_EMBEDDING_MODEL` when set | Unit or env hermetic |
| **AC6** | **Personal-scope refuse** (`project.rs` ~181–188 class) **and** grant-deny: `warnings` contains `kind=denied`; no double-denied if helper seeds | Unit CP |
| **AC7** | Markdown denied: one-line `**Denied:**` with non-empty reason | Unit renderer |
| **AC8** | TTY default format for briefing is **markdown** when no `--format` (F9); non-TTY json | Unit / hermetic |
| **AC9** | progressive / expand missing project: message includes example + env name | Hermetic |
| **AC10** | Missing project exit **2** via `fail_usage` / `GovernedCliError` (F11) — hermetic assert, not doc-only | Hermetic |
| **AC11** | No ranking code change beyond status plumbing | Review diff |
| **AC12** | Full gate green | Process |
| **AC13** | CHANGELOG: minor F9 + additive embedding + exit-2 progressive; **no** F9 BREAKING | Diff |
| **AC14** | CAPABILITIES/OPERATIONS embedding honesty | Diff |
| **AC15** | Pretty path does not duplicate status cause in hint (F6/M5) | Unit or snapshot |

## 7. Non-goals

- New embedding models or providers  
- Changing cosine / FTS / bridge ranking  
- T203 discovery lists or review soft-default  
- Progressive multi-format human UI  
- Requiring embedding server for non-semantic recall  
- Force single error envelope  
- clap / is-terminal version bumps  
- Ollama-specific API surface  
- Daemon semantic parity (F25)  
- Broader governed TTY format matrix (T204 / L2)  

## 8. Handoffs

| To | What |
|----|------|
| deferred.md | Strike T202 clarity row on ship |
| T201 | EXIT_USAGE + GovernedCliError reused by `fail_usage` |
| T203 | Soft scope resolve; discovery lists; F12 boundary |
| T204 | Help examples (L5); broader OutputFormat TTY defaults (L2) |
| CAPTURE | Unchanged independence |
| ISSUES | Optional append L2 on ship if not tracked |

## 9. Implementation sketch

### 9.1 Contracts

```rust
// ai-brains-contracts recall.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingStatusDto {
    /// ok | unreachable | error | no_stored_embeddings | skipped
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

// RecallResponse: add
#[serde(skip_serializing_if = "Option::is_none")]
pub embedding: Option<EmbeddingStatusDto>,
```

### 9.2 Retrieval: return status from semantic path

Change internal API so semantic path returns `(Vec<RecallHit>, EmbeddingStatusDto)` instead of swallowing errors with only eprintln. Classify errors per F4 (M6). After embed ok: empty fetch or all-undecodable rows → `no_stored_embeddings` (M7/L6).

### 9.3 Model env

```rust
let model = std::env::var("AI_BRAINS_EMBEDDING_MODEL")
    .unwrap_or_else(|_| "nomic-embed-text-v1.5".to_string());
```

### 9.4 Briefing `empty_denied` (M2 preferred)

```rust
// both Project + Personal empty_denied:
let reason = reason.into();
warnings: vec![BriefingWarningDto {
    kind: "denied".into(),
    message: reason.clone(), // or short fixed template + reason
    subject_id: None,
    subject_kind: None,
}],
// then remove duplicate .warnings.push(kind=denied) at grant-deny call sites
// Personal-scope refuse ~181-188 inherits seed automatically
```

### 9.5 Progressive usage fail (M3)

```rust
// governed_common.rs
pub fn fail_usage(msg: impl Into<String>) -> GovernedResult {
    let message = msg.into();
    eprintln!("{message}"); // or "USAGE: …"
    Err(Box::new(GovernedCliError::emitted(EXIT_USAGE, message)))
}

// governed_query.rs — project_id None after clap env:
return fail_usage(
  "project id required. Example:\n  ai-brains query progressive \"why was graph backend replaced?\" --project-id <uuid>\nOr set AI_BRAINS_PROJECT_ID."
);
// handle_cli_result already downcasts GovernedCliError → exit_code
```

### 9.6 Briefing TTY default (F9 freeze)

```rust
fn resolve_briefing_format(explicit: Option<&str>, is_tty: bool) -> &str {
  match explicit {
    Some(f) => f,
    None if is_tty => "markdown",
    None => "json",
  }
}
```

## 10. Verification plan

| Layer | What |
|-------|------|
| Unit | format residual; status map F35; empty_denied; model env; AC3b; AC15 |
| Hermetic | AC2, AC3, AC4–AC5, AC8–AC10 |
| CP tests | AC6–AC7 (Personal-scope refuse + grant-deny) |
| Docs | AC13–AC14 |
| Process | AC11–AC12; F22 review |
| Regression | T101 pretty; T111 hints; T152 deny; dogfood still passes `--format json` |

## 11. Stop-before

- Hard-failing non-semantic recall when embed server down  
- Ranking / blend order changes  
- Implementing T203 scope soft-default under T202  
- New crates or AGPL deps  
- Breaking RecallResponse without additive serde  
- Daemon semantic parity work (F25)  
- Scope creep into full governed TTY matrix (T204)  

## 12. Suggested implement order

1. Contracts `EmbeddingStatusDto` + RecallResponse + serialize tests.  
2. Retrieval: model env (F5) + status return + F4 map (F35 unit) + no_stored_embeddings (F34).  
3. CLI wire + pretty/hint precedence (F6/AC15).  
4. Briefing `empty_denied` seed + dedup pushes; F9 format.  
5. `fail_usage` + progressive/expand F10–F11 (AC9–AC10).  
6. Hermetic F18; docs + CHANGELOG minor; review/gate; deferred strike.

## 13. Risk notes

| Risk | Mitigation |
|------|------------|
| Double network probe | F4: reuse embed attempt |
| F9 default surprise | M1: dogfood safe; CHANGELOG minor |
| Exit 2 path missing | F11 `fail_usage` + GovernedCliError (proven T160/T201) |
| Double denied warnings | F7 seed + dedup pushes |
| Status+hint double print | F6/AC15 precedence |
| ambiguous unreachable vs error | F4/M6 closed map + F35 unit |
| Agent scripts parse only results | Additive field; skip_serializing when None |

## 14. AI fold-in disposition (2026-08-04)

| ID | Source | Disposition |
|----|--------|-------------|
| AI1 §1–4 | embedding DTO, model env, briefing deny, progressive exit2 | **Accept** (already F2–F11) |
| **M1** | F9 not BREAKING; dogfood `--format json` | **Accept** → F9 freeze, F23, F36, AC8/AC13 |
| **M2** | empty_denied call sites; Personal-scope refuse ~181–188; helper seed + dedup | **Accept** → F7, AC6, §9.4 |
| **M3** | `fail_usage` + EXIT_USAGE; AC10 hermetic | **Accept** → F11, F30, §9.5 |
| **M4** | daemon `semantic: false` at lib.rs:273 | **Accept** → F25 |
| **M5** | pretty status vs hint precedence | **Accept** → F6, AC15 |
| **M6** | unreachable vs error map; assert specific status | **Accept** → F4, F18(2), F35, AC2/AC2b |
| **M7** | no_stored_embeddings derivation | **Accept** → F4, F34, AC3b |
| **L1** | URL default vs nightly | **Accept note** — both `:8083` (F5) |
| **L2** | broader OutputFormat TTY | **Defer T204** → F32 |
| **L3** | query trace excluded | **Accept** → F31 |
| **L4** | JSON hint shorten | **Soft** → F33 |
| **L5** | after_help --project-id | **Defer T204** → F32 |
| **L6** | all rows undecodable | **Absorb** into F4 no_stored_embeddings |
| **L7** | INVALID_TRANSITION cleanup | **Out of scope** |
| **L8** | tracing subscriber present | **Accept** → F27 viable |
| **L9** | cli_principal shared | **Affirm** only |

**Not folded:** inventing Ollama-only health probes; ranking changes; soft-alt exit 1 for F11; F9 soft-alt keep-json-default.
)
