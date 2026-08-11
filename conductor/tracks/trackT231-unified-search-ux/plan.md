# T231 — Unified search UX defaults — Plan

**Status:** 🚧 **Implementing** — hard A+C code + docs in progress 2026-08-11  
**Category:** UX / IA / FEATURE (light)  
**Depends:** T207 empty pretty · T211 ranking/ledger-first · T228 Scope SOOT · T230 labels

## Goal

Close the dual “which search command?” mental model without merging engines: document a clear decision table, harden `sync query` project resolve (no random UUID — pretty **and** ndjson), gate empty pretty ledger next-step so sync empty does not self-mention, and document format/invalid-env asymmetries. Prefer **A+C**; **decline** new `search` noun and recall `text`→pretty arm as DoD.

## Absorbed deferred

| Item | Disposition |
|------|-------------|
| deferred.md Dual recall vs sync query | **DoD** |
| Series README T231 | **DoD on ship** |
| T228 F32 random-UUID project fallback | **Hard F10** |
| T228 F34 sync always-pretty | **Document intentional F7/F33** |
| Placeholder A | **Hard** (next-step + help; TTY pretty already true) |
| Placeholder C | **Hard** (CAPABILITIES §15 + WORKFLOWS) |
| Placeholder B `search` alias | **Soft residual** |

**Not absorbed:** engine merge; semantic on sync; JSON scope; clap 5; T229; T227 OutputFormat; invalid-env clap converge; recall text→pretty as DoD.

## Research pins (2026-08-11)

| Pin | Evidence |
|-----|----------|
| Live F32 | `--no-project-context` + unset project → `Scope: project=<random uuid>` + 0 memories |
| recall piped | Default **JSON** (non-TTY) |
| sync query piped | Default **pretty** (always) |
| sync `text` | ≡ pretty path (`ndjson` only special-cased) |
| recall `text` | → JSON via `_` arm (help lists only json\|pretty) — **F8** |
| NDJSON second random | `sync.rs:419` `unwrap_or_else(ProjectId::new)` + `Some(…)` to recall |
| BridgeRecord.project_id | required `String` (`bridge.rs:80`) → F21 replacement **`""`** |
| Invalid env | recall clap exit **2**; sync after F10 → None vault-wide exit **0** — **F36** |
| Shared empty hint | `print_pretty_empty_sync` → `build_recall_hint` — **F37** gate required |
| Hermetic F32 | `--no-project-context` + no `.env` clears env (main.rs); explicit `.env()` on Command survives |
| CAPABILITIES §15 | “Code + memory → recall or sync query” — replace with decision table |
| WORKFLOWS | no “Find something” recipe |
| clig.dev | Human-first; suggest next commands |
| Deps | clap **4.6.1** / is-terminal **0.4.17** / chrono **0.4.44** / serde **1.0.228** — **no bump** |
| Zero new crates | — |

## AI fold-in pins (hard)

| ID | Pin |
|----|-----|
| **AI1 M1 / F29** | Pure `resolve_sync_project_id(global, Option<&str>)`; trim; empty/invalid → None |
| **AI1 M2 / F21** | NDJSON: pass `Option` to recall; `BridgeRecord.project_id = map.unwrap_or_default()`; delete `ProjectId::new` |
| **AI1 M3 / F12–F13** | Empty recall pretty ledger next-step |
| **AI1 M4 / AC5–7/13** | Hermetics missing/invalid/valid + global |
| **AI1 L1 / F18** | CAPABILITIES + WORKFLOWS |
| **AI1 L2 / F14** | Help peer cross-ref |
| **AI1 O1 / AC1–4** | Pure units (+ AC4b whitespace) |
| **AI2 M1 / F8** | Document text asymmetry; text arm **soft** |
| **AI2 M2 / F21** | Exact ndjson pin (same as AI1 M2) |
| **AI2 M3 / F36** | Invalid-env asymmetry document only |
| **AI2 L1 / F30** | Hermetic `--no-project-context` + tempdir notes |
| **AI2 L3–L4 / F37 / AC8b** | `include_sync_query_hint: bool` — false for sync empty |
| **AI2 L5 / F29** | Call-site `env::var(...).ok()` — no `"default-project"` |
| **AI2 L6 / F18** | Decision table in CAPABILITIES **§15** |
| **AI2 O8 / F35** | Cross-model soft |

**Soft:** F22 recall text→pretty; search alias; is-terminal→stdlib; non-empty footer.

See `spec.md` §11 full disposition + §12 SOOT snippets.

## Product table (ship copy — CAPABILITIES §15)

| Intent | Command |
|--------|---------|
| Human vault TTY | `recall "…" --format pretty` |
| Agent / pipe | `recall "…"` (JSON) |
| Human vault + ledger | `sync query "…"` |
| Semantic / hybrid | `recall "…" --semantic` |
| Machine vault stream | `recall --format json` or `sync query --format ndjson` |
| Invalid `AI_BRAINS_PROJECT_ID` | recall exit 2; sync query → `project=(none)` vault-wide |
| `text` format | sync ≡ pretty; recall → JSON (undocumented) |

## Frozen decision index

See `spec.md` §4 **F1–F40**. Hard summary:

1. Keep dual commands (F2).  
2. A+C hard; B soft (F3/F4).  
3. Fix sync project resolve → None, never random (F10/F32/F29).  
4. NDJSON same honesty — `""` + pass None (F21/AC14).  
5. Scope honesty via existing SOOT (F11).  
6. Empty recall pretty next-step; **suppress on sync empty** (F12/F37/AC8b).  
7. Document F8 text + F36 invalid-env asymmetries.  
8. Help + CAPABILITIES §15 + WORKFLOWS + new CHANGELOG (F14/F18).  
9. Keep sync always-pretty default; document (F7/F33).  
10. No ranking/contract/dep changes (F15/F19/F34).  

## Task checklist

### 0. Preflight (on go)

- [x] `ai-brains preflight --summary` (session context; ledger TX already open)
- [x] Ledger TX already started: `f80a6b50-d6d4-4d41-a9d6-0feebff0978f` (do not re-start)
- [ ] `ledgerful scan --impact` (optional residual)
- [x] Live F32 reconfirmed fixed via hermetics + dogfood

### 1. Red — project resolve + ndjson

- [x] Unit tests AC1–AC4 + AC4b for `resolve_sync_project_id`
- [x] Hermetic AC5/AC6 (green with F29; written alongside fix)
  - **F30:** tempdir + `--no-project-context` for missing; `.env("AI_BRAINS_PROJECT_ID", "not-a-uuid")` for invalid
- [x] AC14 ndjson honesty hermetic
- [ ] Commit red allowed (combined green implement — no separate red commit)

### 2. Green — project resolve + ndjson

- [x] Implement F29 helper (spec §12)
- [x] Call-site: `resolve_sync_project_id(global, env::var("AI_BRAINS_PROJECT_ID").ok().as_deref())`
- [x] F21: remove `unwrap_or_else(ProjectId::new)`; pass `Option` to recall; `unwrap_or_default()` for BridgeRecord
- [x] AC5–AC7, AC13, AC14 green
- [x] Targeted nextest + clippy on `ai-brains-cli`

### 3. Red/Green — discovery chrome

- [x] F37: plumb `include_sync_query_hint` through `build_recall_hint` / core
- [x] Recall empty pretty: true → AC8
- [x] `print_pretty_empty_sync`: false → AC8b
- [x] Help strings F14 / AC10
- [x] Do not regress `resolve_format` AC11

### 4. Docs

- [x] CAPABILITIES **§15**: replace “Code + memory \| recall or sync query” with full decision table (incl. F8/F36 rows)
- [x] CAPABILITIES sync section: F32 fix note + F33 always-pretty intentional
- [x] WORKFLOWS “Find something” short recipe
- [x] New CHANGELOG T231 row only
- [ ] Optional skill one-liner if table already lists both (skipped — soft)

### 5. Verify / close

- [x] Live dogfood F31 (targeted; full gate deferred to review/close)
- [ ] Full gate: `cargo fmt --check` ; clippy workspace `-D warnings` ; nextest workspace ; deny ; audit ; `ledgerful verify --scope full`
- [ ] Primary review → fix loop
- [ ] Soft cross-model only if scope expands (e.g. text arm)
- [ ] `conductor.md` → Completed; deferred.md strike dual-search; series README update
- [ ] `ledgerful ledger commit` + `ai-brains pin "DECISION: T231 …"`

## Manual test script (on go)

```powershell
# F32 fixed
Remove-Item Env:AI_BRAINS_PROJECT_ID -ErrorAction SilentlyContinue
ai-brains --no-project-context sync query "probe" --limit 1 --quiet --no-bridge --format pretty
# expect: Scope: project=(none)  — NOT a random uuid

# F21 ndjson honesty (no phantom project id)
ai-brains --no-project-context sync query "probe" --limit 1 --quiet --no-bridge --format ndjson
# expect: project_id field empty string in JSON lines when no project

# Agent path unchanged
ai-brains recall "probe" --limit 1 --quiet --no-bridge 2>$null | Select-Object -First 1
# expect: JSON when non-TTY

# F12 + F37
ai-brains recall "zzzz-no-hit" --format pretty --limit 1 --quiet --no-bridge
# expect: empty hint includes sync query line
ai-brains --no-project-context sync query "zzzz-no-hit" --format pretty --limit 1 --quiet --no-bridge
# expect: empty vault block does NOT suggest "sync query" self-mention

ai-brains recall --help
ai-brains sync query --help
```

## Risks / stop-before

- Stop if product owner wants **B** (`search` noun) or recall **text→pretty** as mandatory DoD mid-track — re-open plan.
- Stop if vault-wide-on-None conflicts with policy that missing project must exit 2 (today recall allows None; match that for **missing**; **invalid** on recall stays exit 2 via clap — F36).
- Do not “fix” F34 by inventing JSON for sync without a contracts track.
- Do not add clap `env=` to sync “for consistency” without explicit go (F36).

## Done when

- AC1–AC14 green (as applicable)  
- Live F32 gone (pretty + ndjson)  
- AC8b no self-mention  
- Docs decision table + F8/F36 honesty shipped  
- Gate green; review clean or residual soft only  
- deferred dual-search closed  

## Next after ship

- Soft: search alias; recall text arm; is-terminal→stdlib  
- Ops **T229** or series residual cleanup  
- Optional invalid-env converge track if operators still confused  
