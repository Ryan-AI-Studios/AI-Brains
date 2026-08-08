# T234 — Message-only capture contract (shared SOOT)

- **Track ID:** T234-MessageOnlyCaptureContract
- **Phase:** Harness seamless ingest series foundation (T234–T239)
- **Status:** 📋 **Proposed / Expanded** (plan-only until **go**)
- **Depends on:** Existing `ai-brains-adapters` antigravity `extract_turns`; `ai-brains-capture` user/assistant ingest; AGENTS **Capture Privacy** mandate; series [README](../README-T234-T239-HARNESS-INGEST.md)
- **Blocks / feeds:** **T236–T239** must call this filter for all harness → vault content; T235 capability honesty (“message-only”); soft T224 role-prefix is **display** only (orthogonal)
- **Category:** ARCHITECTURE / FEATURE / DOCS
- **Source:** Research 2026-08-08 multi-harness ingest; user confirmation “messages in/out only, not tool calls/thinking”; live dogfood 2026-08-08
- **Deferred absorbed:** series C1–C3 invariants; antigravity partial message-only (extract_turns); agy-hook weak role filter; Grok/OpenCode adapter stubs without filter; README T234 foundation row
- **Not absorbed:** Hook install (T235); AGY wire+binding (T236); Grok hooks/batch (T237); OpenCode plugin/export (T238); nightly multi-harness (T239); display role-prefix strip (T224); project binding; delta watermark policy beyond reusing existing extract
- **Research date:** 2026-08-08 (expand + live re-scan Grok/AGY logs + online CoT/tool-block norms)
- **AI fold-in:** 2026-08-08 — AI1 affirms architecture F1–F6/F12–F18; elevates Grok multipart (1), AGY tool-step content leak (2), interleaved parts (3), **UTF-8 slice safety (4)**, thinking DTO freeze (5). AI2 file content is **stale T216** — not applied. Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start` on go)

## 1. Objective

1. **One shared, pure, tested** “ingestable turn” contract for every harness path (hooks, `agy-hook`, batch importers, future adapters).  
2. **Keep:** user prompt text + **final** assistant response text only.  
3. **Drop:** tool calls, tool results, backend tools, hidden thinking/reasoning (incl. encrypted/redacted CoT), system chrome, synthetic harness metadata wrappers (extract inner user text when present).  
4. **Migrate SOOT call sites:** antigravity `extract_turns` and agy-hook list filter **must** call the shared module (no duplicate keep/drop matrices).  
5. **Fixture matrix** for AGY step JSONL, Grok `chat_history.jsonl`, OpenCode-export-like messages — exact keep/drop counts.  
6. **Capture independence:** filter is pure string/JSON logic — no models, embeddings, graph, network.  
7. **Docs honesty:** CAPABILITIES / OPERATIONS one-liner “message-only capture”; CHANGELOG; series README points at SOOT.

## 2. Live baseline (re-scan 2026-08-08)

### 2.1 Audit / product signal

| Fact | Live |
|------|------|
| AGENTS Capture Privacy | “ONLY the final assistant response and user prompt”; no CoT / tool logs |
| `antigravity::extract_turns` | Keeps `USER_EXPLICIT`/`USER_INPUT` + non-empty `MODEL`/`PLANNER_RESPONSE` content; skips tool-only planner; **drops** other types via `_` |
| Live AGY `transcript.jsonl` | Heavy `PLANNER_RESPONSE` with **`thinking` + `tool_calls`** and empty content; `VIEW_FILE` / tool steps with **content** (must stay dropped); `SYSTEM`/`CHECKPOINT` chrome |
| `AntigravityStep` serde | **Does not bind** `thinking` field today — thinking ignored by accident of schema, not SOOT policy |
| `agy::parse_agy_transcript` | Pass-through `{role,content}` — **no** tool/thinking strip |
| `agy-hook` | Filters only `role == user \| assistant` — **does not** strip tool-shaped content if labeled assistant |
| Grok `chat_history.jsonl` (large session) | Types: **`user`**, **`assistant`**, **`reasoning`**, **`tool_result`**, **`backend_tool_call`**, **`system`**; user `content` often **JSON array** of parts; assistant often **string** |
| OpenCode adapter | Capability stub only — no export parser yet |
| `IngestRequest.thinking` | Optional field exists; **event builders never write thinking** into payloads (content only) — still must not accept thinking as `content` |
| Capture roles | `user` \| `assistant` only (`UnsupportedRole` else); empty rejected |

### 2.2 Root cause (frozen)

```text
// Partial SOOT exists only inside antigravity::extract_turns (AGY-specific match arms).
// agy-hook: role allowlist only → polluted if transcript has tool text as "assistant".
// Grok/OpenCode: no production filter path yet.
// Series T236–T239 will re-implement drop rules unless T234 freezes one SOOT.
```

### 2.3 Code / touch map

| Site | Role |
|------|------|
| **`ai-brains-adapters/src/message_only.rs` (new)** | Pure SOOT: types, keep/drop, harness normalizers, chrome extractors |
| `ai-brains-adapters/src/lib.rs` | `pub mod message_only` + re-exports |
| `antigravity.rs` | `extract_turns` → call shared SOOT; keep XML strip helpers or move pure strip into message_only |
| `agy.rs` | After parse, map through message_only (or thin wrapper) |
| Soft: `agy_hook.rs` | Use filtered turns from adapters (if parse API changes) |
| Soft: `ai-brains-capture` | Document that `thinking` is never stored; optional refuse non-None thinking with warn (not DoD if free conflict) |
| Fixtures | `crates/ai-brains-adapters/tests/fixtures/message_only/{agy,grok,opencode}_*.jsonl` (synthetic, no secrets) |
| Unit tests | pure matrix in `message_only` + antigravity extract regression |
| Docs | CAPABILITIES capture privacy; OPERATIONS; CHANGELOG; series README link |

### 2.4 Deps / pins (researched 2026-08-08)

| Item | Workspace / crates.io | Decision |
|------|----------------------|----------|
| serde / serde_json | 1.0 / **1.0.151** latest | **No bump** required |
| uuid | 1.13 | unchanged |
| clap | 4.5 (CLI only) | **No bump** — T234 is library-first |
| Zero new crates | Required — pure Rust + existing serde_json |
| Capture independence | Pure filter only |

### 2.5 Online / product research

| Finding | Application |
|---------|-------------|
| OpenAI/Responses-style outputs interleave **text + tool_call** / **reasoning** items | Keep text parts of assistant; **drop** tool_call and reasoning items entirely |
| Anthropic extended thinking: `thinking` / `redacted_thinking` / `tool_use` blocks | Never store thinking or tool_use; keep `text` blocks only |
| Community guidance: drop reasoning items from durable history for privacy/token hygiene | Aligns with AGENTS Capture Privacy |
| Product series C1–C3 | Hard invariants for all T234–T239 |

## 3. Frozen decisions (F1–F42)

| ID | Decision |
|----|----------|
| **F1 — Module home** | New pure module **`ai_brains_adapters::message_only`**. Not a new crate. Capture stays independent of harness parsers; adapters normalize harness → SOOT → `IngestableTurn`. |
| **F2 — Public types** | `IngestableTurn { role: IngestRole, content: String, source_ts: Option<String> }` with `IngestRole::User \| Assistant`. Optional `DropReason` enum for tests/debug (not required on happy path). |
| **F3 — Role allowlist** | After normalization, only **user** and **assistant**. Drop: system, tool, function, reasoning-as-role, unknown. |
| **F4 — Content non-empty** | After chrome strip + whitespace trim, empty → drop. |
| **F5 — No tools** | Drop any turn classified as tool_call / tool_result / backend_tool / tool-only planner (empty content + tool_calls present) / VIEW_FILE-like tool step types. Never serialize tool JSON into `content`. |
| **F6 — No thinking** | Drop `reasoning`, `thinking`, redacted/encrypted CoT fields. If a step has both thinking and visible content, **keep visible content only** (do not concatenate thinking). |
| **F7 — AGY/Antigravity SOOT (AI1 §2)** | Match **strictly** on `(source, step_type)` — **never** “has content ⇒ keep”: |
| | • `("USER_EXPLICIT", "USER_INPUT")` → User after XML strip (F10) |
| | • `("MODEL", "PLANNER_RESPONSE")` → Assistant **only** if non-empty **visible** text content (after trim); tool-only (empty content + tool_calls) → drop |
| | • **Drop regardless of content:** `TOOL_OUTPUT`, `VIEW_FILE`, `RUN_COMMAND`, any other tool step types, `SYSTEM`, `CHECKPOINT`, thinking-only planner rows |
| | Thinking field never enters `IngestableTurn.content` (F6/F38). |
| **F8 — Grok SOOT** | Keep `type=user` with extracted text (F10/F37); keep `type=assistant` with non-empty string/text parts; **drop** `reasoning`, `tool_result`, `backend_tool_call`, `system`. Synthetic user chrome: if `synthetic_reason` present and content is harness system-reminder only (no user_query / no meaningful user text after strip) → **drop**; if synthetic wrapper still contains real user text → keep extracted text. |
| **F9 — OpenCode SOOT** | Map export message roles to user/assistant; drop tool/part tool-invocation payloads; keep text parts only. Fixture-driven (export schema may evolve in T238). |
| **F10 — User text extract (chrome) (AI1 §1)** | Pure helpers: (a) Antigravity XML strip; (b) Grok/OpenAI-like: `extract_text_from_json_content`: |
| | • `Value::String(s)` → `extract_user_text(s)` |
| | • `Value::Array(parts)` → for each object, keep `part["text"]` when `part["type"] == "text"` **or** when a string `"text"` key is present without a tool-ish type; join with `\n`; **drop** image/tool parts |
| | Prefer body inside `<user_query>…</user_query>` when present; pure system-reminder wrappers with no user text → drop turn. Prefer **extract-inner**, not drop-all. |
| **F11 — Assistant multi-part (AI1 §3)** | Array parts: keep `type == "text"` / bare string parts only; **drop** `tool_use`, `tool_call`, objects with tool-shaped `name`/`arguments` without text, thinking parts; join kept text with `\n`. If empty after strip → drop whole turn. |
| **F12 — extract_turns migration** | `antigravity::extract_turns` **must** call message_only (no parallel keep/drop matrix). Preserve public `AntigravityTurn` type or map via `From` — avoid breaking import_antigravity without tests green. |
| **F13 — agy path** | `parse_agy_transcript` / hook path: after parse, run message_only filter (role+content rules). If live files are step-shaped JSONL, T236 owns full step parser; T234 provides filter + optional `filter_agy_simple_lines` for `{role,content}` JSONL. |
| **F14 — API shape** | Core entry points: `fn filter_turn(role, content) -> Option<IngestableTurn>`; harness-specific: `fn filter_antigravity_steps(&[Step]) -> Vec<IngestableTurn>`; `fn filter_grok_history_lines` / `fn filter_opencode_messages` (fixture-ready even if T237/T238 wire later). |
| **F15 — Property** | Unit property: kept content must not contain top-level JSON keys typical of tools when classified as tool-only fixtures (`"tool_calls"`, `"tool_result"` as sole payload). Not a heuristic ban on the substring in legitimate user text discussing tools. |
| **F16 — No vault writes in T234** | Library + unit/hermetic adapter tests only. No new CLI command required. |
| **F17 — Capture thinking field (AI1 §5)** | **Preserve** `IngestRequest.thinking: Option<String>` on the contracts DTO (no field removal — serialization compat). **Normative:** adapter normalizers / message_only **never** populate `thinking`; event builders **never** serialize thinking into the event log (already true for `AssistantFinalRecorded`). Soft residual: capture warn+clear if Some (F24). |
| **F18 — Zero new crates** | — |
| **F19 — Determinism** | Same input steps → same ordered turns; stable tests; sort not required if input order preserved. |
| **F20 — Series order** | Implement T234 before T236–T239 production wiring. |
| **F21 — Fixtures** | Synthetic only under `tests/fixtures/message_only/`; no live user secrets; redact if derived from dogfood. |
| **F22 — Capability notes** | CAPABILITIES: message-only SOOT shipped; full seamless harness still Partial until T236+. |
| **F23 — No models** | Filter never calls LLM to “summarize” tool output into memory. |
| **F24 — Soft residuals** | Capture refuse thinking Some; Grok synthetic classifier edge cases; full OpenCode live export schema; move XML helpers to message_only if churn high; T224 display strip. |
| **F25 — Not in track** | Hook install; project binding; nightly orchestration; reading Grok `updates.jsonl`; OpenCode raw SQLite; auto-pin. |
| **F26 — Error handling** | Malformed lines skip (existing antigravity style); no panic; no unwrap in production. |
| **F27 — Interleaved assistant** | Text + tools in one record: keep text, drop tools (F6/F11). |
| **F28 — System role** | Always drop (not user). |
| **F29 — Docs** | CAPABILITIES Capture Privacy subsection; OPERATIONS ingest honesty; CHANGELOG; README-T234-T239 link SOOT path. |
| **F30 — Tests naming** | `function_or_feature__condition__expected_result` |
| **F31 — Regression** | Existing antigravity unit tests for extract_turns remain green (behavior preserved or strictly stricter on drops). |
| **F32 — Stricter OK** | Dropping more tool/chrome than today is **allowed** if user/assistant text is preserved. Silently keeping new tool types is **forbidden**. |
| **F33 — Harness id** | Message_only does not assign harness IDs (importers own that). |
| **F34 — Privacy inheritance** | Out of scope (no new derived memories). |
| **F35 — Public re-exports** | `pub use message_only::{IngestableTurn, IngestRole, filter_turn, ...}` from adapters lib as needed by CLI later. |
| **F36 — Performance** | O(n) over steps; no full-file regex crates; simple string scans. |
| **F37 — Grok content array** | First-class support: `content: string \| array` deserialization helper returning extracted user/assistant text. |
| **F38 — AGY thinking field** | When deserializing steps for filter path, **ignore** `thinking` for storage even if present on Value (may extend AntigravityStep with `#[serde(default)] thinking: Option<Value>` **skipped** for keep path — or parse via Value). Prefer not expanding keep surface. |
| **F39 — Dual path series** | Hooks and batch both call same filter (C5). |
| **F40 — Consent** | Out (T235). |
| **F41 — Fail-open parse** | Bad JSONL lines skip; do not fail whole file in filter unit API (callers decide). |
| **F42 — Plan freeze date** | 2026-08-08. |
| **F43 — UTF-8 char-safe slicing (AI1 §4)** | All XML strip / tag extract / truncate in message_only (and any moved strip helpers) must be **char-boundary safe**: use `str::find` on ASCII delimiters only, or `char_indices` / `chars().take`; **never** arbitrary byte indices that can panic mid-scalar. Multibyte user text (emoji, non-ASCII) must not panic. Note: existing antigravity strip uses `find` + ASCII tags (boundary-safe for those delimiters) — **re-verify** when moved/copied; add regression unit with emoji inside USER_REQUEST. |
| **F44 — AI1 architecture affirm** | Pure module, role allowlist, drop tools+thinking, zero new deps, capture independence — affirmed; no design delta. |
| **F45 — AI2 disposition** | AI-review.md AI2 body is **stale T216** (forget-list); **out of scope** for T234 fold-in. |
| **F46 — Adapters never set thinking** | Any code path that builds `IngestRequest` from message_only always sets `thinking: None` (agy-hook already does; keep as SOOT comment/test). |
| **F47 — Fold-in freeze date** | 2026-08-08 AI review. |

## 4. Acceptance criteria

| ID | Criterion | Proof |
|----|-----------|--------|
| **AC1** | `message_only` public API compiles; used by `extract_turns` | Code + unit |
| **AC2** | AGY fixture with tools/thinking/`VIEW_FILE`/`RUN_COMMAND` **with non-empty content** → only user+assistant text; tool steps never kept | Unit fixture |
| **AC3** | Grok fixture with reasoning/tool_result/backend_tool_call/system → only user+assistant | Unit fixture |
| **AC4** | OpenCode-export-like fixture → only user+assistant | Unit fixture |
| **AC5** | User chrome: USER_REQUEST kept; ADDITIONAL_METADATA dropped | Unit |
| **AC6** | Grok user array parts + `<user_query>` extracts inner query; image/tool parts dropped | Unit |
| **AC7** | Assistant text+tool interleaved / multi-part keeps text only; empty after tool strip → drop | Unit |
| **AC8** | Empty content after strip → drop | Unit |
| **AC9** | Existing antigravity extract_turns tests green | nextest |
| **AC10** | No production unwrap/expect in new code | Review |
| **AC11** | CAPABILITIES + CHANGELOG + OPERATIONS touch; thinking DTO honesty (F17) | Doc |
| **AC12** | Full CI gate green | CI |
| **AC13** | Capture independence: adapters message_only has no models/graph deps | cargo tree / review |
| **AC14** | agy simple role path uses message_only filter (not bare role allowlist alone) | Unit / code |
| **AC15** | Multibyte (emoji) inside USER_REQUEST / user_query strip → no panic; correct inner text (F43) | Unit |
| **AC16** | `(MODEL, VIEW_FILE)` and `(MODEL, RUN_COMMAND)` with content → **0** kept turns (F7) | Unit |

## 5. Non-goals

- Hook install UX / preflight offer (T235)  
- Per-harness live wiring & project binding (T236–T238)  
- Nightly multi-harness orchestration (T239)  
- Display-time `ASSISTANT:` strip (T224)  
- Removing `IngestRequest.thinking` field from contracts  
- Storing tool digests as “evidence” memories  
- LLM-based redaction  

## 6. Risk & verification

| Risk | Mitigation |
|------|------------|
| Over-strip real user text in tags | F10 extract-inner; AC5/AC6 |
| Under-strip VIEW_FILE/RUN_COMMAND content | F7 type match **regardless of content**; AC2/AC16 |
| Grok content array parse bugs | F10/F37 part rules; AC6 |
| Interleaved text+tool empty | F11 drop whole turn if no text; AC7 |
| UTF-8 panic on strip | F43 + AC15 multibyte regression |
| Duplicate matrices drift | F12 single SOOT |
| Contract thinking field confusion | F17/F46 docs + never populate |
| Fixture secrets | F21 synthetic only |

**Implement order on go:** pure types + filter_turn + UTF-8-safe strip red → AGY/Grok/OpenCode fixtures (incl. AC15–AC16) → migrate extract_turns → agy path → docs → gate.

**Manual dogfood (optional):** run unit fixtures only; live vault import remains T236+.

## 7. Residual after ship

| Residual | Disposition |
|----------|-------------|
| Capture refuse `thinking: Some` | Soft F24 |
| Full Grok synthetic taxonomy | Soft / T237 |
| Live OpenCode export schema | T238 |
| Hook install + detect | T235 |
| AGY history binding | T236 |
| T224 display strip | Separate track |

## 8. Series context

**T234 first** in harness series. Suggested order: T234 → T235 → T236 → T237 → T238 → T239 (parallel T236–T238 after T234+T235 if non-intersecting). Orthogonal to CLI quality T217–T232 and T233 multi-root.

## 9. Implementation notes

### 9.1 Core API sketch

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestRole { User, Assistant }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestableTurn {
    pub role: IngestRole,
    pub content: String,
    pub source_ts: Option<String>,
}

pub fn filter_turn(role: &str, content: &str) -> Option<IngestableTurn> { /* F3–F4 */ }

pub fn extract_user_text(raw: &str) -> String { /* F10 antigravity XML + user_query */ }

pub fn extract_text_from_json_content(value: &serde_json::Value) -> Option<String> { /* F11/F37 */ }
```

### 9.2 Grok line sketch

```rust
// type=user → extract content (string|array) → extract_user_text → User
// type=assistant → extract text → Assistant
// type=reasoning|tool_result|backend_tool_call|system → drop
```

### 9.3 AGY step sketch

```rust
// reuse extract_turns logic via message_only classification of (source, type, content, tool_calls)
```

## 10. Definition of Done

- [ ] F1–F47 respected (soft F24 deferred with note)
- [ ] AC1–AC16 met
- [ ] Review log clean for critical/high
- [ ] Full gate green
- [ ] conductor **Completed**; deferred strike; pin closeout
- [ ] No production unwrap/expect

## 11. Manual / live evidence notes (planning)

| Source | Observation |
|--------|-------------|
| AGY transcript | thinking+tool_calls without content; VIEW_FILE content must drop |
| Grok chat_history | reasoning/tool_result/backend_tool_call dominate some sessions; user content often array |

## 14. AI fold-in disposition (2026-08-08)

| ID | Source | Disposition | Landing |
|----|--------|-------------|---------|
| **AI1 §1** | Grok multipart + user_query | **Accept / elevate** | F10, F37, AC6 — `extract_text_from_json_content` string\|array; text parts only |
| **AI1 §2** | AGY tool steps with content leak | **Accept / elevate** | F7, AC2, AC16 — match `(source, type)`; drop VIEW_FILE/RUN_COMMAND/**regardless of content** |
| **AI1 §3** | Interleaved text + tools | **Accept / elevate** | F11, F27, AC7 — keep text parts; drop tool_use/tool_call; empty → drop |
| **AI1 §4** | UTF-8 multi-byte slice panic | **Accept (new hard)** | **F43**, AC15 — char-boundary safe strip; emoji regression |
| **AI1 §5** | Keep `IngestRequest.thinking` field | **Accept / affirm** | F17, F46, AC11 — no DTO removal; never populate from adapters |
| **AI1 arch** | Pure SOOT module design | **Affirm** | F44 |
| **AI1 AC table** | AC1–AC13 + fixtures | **Affirm** (already + AC14–AC16) | §4 |
| **AI2** | Stale T216 forget-list report | **Out of scope** | F45 — not folded into T234 |

**Not folded:** changing event schema to store thinking; LLM-based redaction; hook install; treating AI2 T216 M1–M7 as T234 work.

| **AI2** | Stale T216 forget-list report | **Out of scope** | F45 — not folded into T234 |

**Not folded:** changing event schema to store thinking; LLM-based redaction; hook install; treating AI2 T216 M1–M7 as T234 work.
