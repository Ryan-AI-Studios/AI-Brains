# T260 — Recall: demote code-symbol stubs

- **Track ID:** T260-RecallDemoteSymbolStubs
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** FEATURE / UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — recall default **5/4**; `--semantic` **6/6**; `--global` **3/3**; real-project semantic **4/3**; opportunity “demote symbol stubs”
- **Depends on:** T70 symbol ingest; T215/T218 semantic floors; T217 FTS rescue; T211 rerank
- **Absorbs:** `Module sqlite_backend` / `Struct Project` / `Function capture_metadata` beating DECISION pins; “what is this project” returning T186 hermetic tests; real-project “capture independence” returning error structs with “no semantic hits above threshold”
- **Not absorbed:** Empty-query latency (T261); Scope default (T258); leftover dump (T259)

---

## 1. Objective

Vault-first `recall` / `search` should answer **decisions, constraints, session memory** by default. T70 code symbols stay available behind an explicit `--symbols` (or a clearly labeled second pane), not as the top five hits for a natural-language question.

## 2. Problem (live 2026-08-16)

| Query | Scope | Top hits |
|-------|--------|----------|
| `what is this project` | default `test-alias` | T170 D21 note; **`Struct Project`**; T186 hermetic tests; **`Function list`**; **`Module project`** |
| `graph backend sqlite` | `--global` | **five identical** `Module sqlite_backend (crates/ai-brains-graph/src/lib.rs:7)` |
| `what is the capture independence rule` | `--semantic --project-id 3581317d` | Embedding: *no hits above threshold; showing lexical* — `Function capture_metadata`, `Struct ValidationError`, `Struct VerificationGateRejection`, `Enum CaptureError` |

Pretty recall of *recent* `DECISION:` pins inside `test-alias` was fine. That is capture freshness, not ranking. On the real project (2,673 memories) the product failed the question it exists to answer.

Root class: T70 nightly symbol rows are short, token-dense (`Module`, `Struct`, `Function`, crate paths) and dominate BM25 / leak through the lexical fallback when cosine is below T218 floors.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | Default recall excludes or **hard-demotes** symbol-stub memories (detect via existing source/kind/tag — do not add a new event kind unless store already has one). |
| **F2** | `--symbols` (name TBD at plan) restores today’s mix or symbols-only. |
| **F3** | `--global` uses the same demotion. Five identical module stubs is a fail. |
| **F4** | Pretty honesty when lexical fallback after semantic miss (T218 F11 stays) must not imply the stubs *are* the answer. |
| **F5** | Capture independence: ranking only; no model required on default FTS path. |
| **F6** | Do not delete ingested symbols. Demote, do not forget. |

## 4. Verification sketch

- Hermetic fixture: DECISION pin + `Module foo` stub; query `what did we decide about foo` → DECISION first; stub absent or below fold without `--symbols`.
- `--symbols` still returns the stub.
- No live vault mutate.
