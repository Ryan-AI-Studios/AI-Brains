# T220 Plan — Preflight summary JSON honesty

**Status:** 🔄 **In Progress** (implementation after go; ledger TX open)  
**Category:** BUGFIX / CONTRACT  
**Depends:** T214 ✅, T180 ✅, T216 envelope pattern ✅, T235 harness human sibling ✅  
**Spec:** [spec.md](./spec.md) — includes AI fold-in **§14**

## Goal

1. Honor `--summary --format json` with a **machine object** (no human banner).  
2. Keep dual vault / in-context counts identical to human summary (T214 SOOT).  
3. **Never** grow `PreflightContextResponse` (T180).  
4. Pure stdout JSON; harness/install status **stderr only** on JSON path (**M1**).  
5. Honest machine `scope`: `"global"` \| `"project"` \| `"none"` (**M2**).  
6. Hermetic + protocol_compat + docs; zero new crates; case-insensitive `--format JSON` (**M3**).

## Absorbed deferred / audit / research / AI fold-in

| Source | Item | Handling |
|--------|------|----------|
| deferred.md T214 F11/F24 | summary JSON machine object | Hard DoD F1–F9, AC1–AC15 |
| Series README T220 | flag lie quality 3 | This track |
| Placeholder draft | Expanded F1–F34 + AC1–AC15 | spec.md |
| T180 freeze | full preflight 2 compact keys | F3/F19/AC6 |
| T170 D21 | summary ≠ governed | F11 docs |
| T216 MemorySummaryJson | api_version + pretty envelope | F4/F5 |
| T235 harness | human sibling | F8 omit array v1 |
| clig.dev | honor --json / --format json | F1/F9 |
| clap crates.io 4.6.6 | workspace 4.5 / resolved 4.6.1 | **no bump** |
| Dogfood 2026-08-09 | human fallback on `--summary --format json` | AC1 freeze |
| **AI1 #1** | projects Option skip | **F4/F34/AC4** |
| **AI1 #2–3** | harness purity + case json | **F8 / F2** |
| **AI2 M1** High | install-hooks × JSON | **F8 + AC8b** hard |
| **AI2 M2** | unresolved scope lie | **F29 + AC14** `scope:"none"` |
| **AI2 M3** | value_parser case trap | **F13** no case-sensitive parser; **AC13** |
| **AI2 M4** | word_count semantic | **F30/F20** docs |
| **AI2 M5** | in_context governed zeros | **F31/AC5** legacy fixture |
| **AI2 M6** | format help | **F20** hard help |
| **AI2 M7** | hermetic gaps | **AC13–AC15** |
| **AI2 M8** | is-terminal final | soft F22 |

**Not absorbed:** harness JSON array hard DoD; change unresolved SQL filters; T219 pretty wall; T228; ledgerful global; PreflightContextResponse growth; clap 5; is-terminal migrate DoD; case-sensitive PossibleValuesParser.

## Live dogfood freeze (2026-08-09)

| Command | Observed |
|---------|----------|
| `preflight --summary` | Human dual counts + Scope (T214 OK) |
| `preflight --summary --format json` | **Human banner** — flag lie |
| `preflight --format json` | Compact `{text, word_count}` only |
| `memory list --summary --format json` | Pretty `api_version` envelope SOOT |

## Research freeze (2026-08-09)

| Topic | Note |
|-------|------|
| Root cause | early `if summary { print_summary; return }` ignores format |
| clig.dev | machine JSON when requested; human default |
| clap | pin 4.5; latest 4.6.6 — no bump |
| serde_json | `to_string_pretty` for summary envelope |
| T180 | new path ≠ grow full preflight keys |

## JSON envelope (implement exactly)

```json
{
  "api_version": "1",
  "scope": "global",
  "project_id": null,
  "projects": 2,
  "pinned": 5,
  "active_sessions": 1,
  "in_context_hotspots": 3,
  "in_context_decisions": 4,
  "in_context_constraints": 1,
  "word_count": 100
}
```

| Case | `scope` | `project_id` | `projects` key |
|------|---------|--------------|----------------|
| `--global` | `"global"` | null | present (u64) |
| project resolved | `"project"` | uuid string | **omit** |
| no global, no project | `"none"` | null | **omit** |

`word_count` = full preflight text budget (not summary size).  
`in_context_*` = legacy `HOTSPOT:`/`DECISION:`/`CONSTRAINT:` markers.

## Phases

### Phase 0 — Plan freeze

- [x] Preflight / doctor / ledger status
- [x] Live dogfood summary vs summary+json vs full json
- [x] Code map (`run` early-return; T214 formatter; T180 protocol_compat; install println sites)
- [x] Online clig.dev + clap pin research
- [x] Spec F1–F28 + AC1–AC12 (initial)
- [x] deferred.md + conductor → **Planning**
- [x] series README status note
- [x] `ai-brains pin` plan-start + freeze
- [x] **AI review fold-in** M1–M8 / AI1 → F8 rewrite, F29–F34, AC8b/AC13–15, §14
- [x] `ai-brains pin` AI fold-in
- [x] User **go** before code / ledger TX

### Phase 1 — Ledger + red (after go)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` (TX pre-started by orchestrator)
- [x] `ledgerful ledger start T220-preflight-summary-json --category BUGFIX --message "…"` (TX `f51e8caa-b159-4830-84bb-f79f3be131f6`)
- [ ] `ledgerful scan --impact` (optional at go)
- [x] Red pure: envelope keys + global projects omit + scope none (AC9) → implemented green
- [x] Red hermetic: AC1 → implemented green (JSON path)

### Phase 2 — Pure JSON builder (F4–F6 / F29 / F34)

- [x] CLI-local `PreflightSummaryJson` Serialize (`projects: Option<u64>` skip)
- [x] Pure builder + scope three-valued mapping
- [x] Unit AC2/AC9 green

### Phase 3 — Wire format branch (F1/F2/F8/F9) **M1 before ship**

- [x] `run` / `print_summary`: summary && format=json (case-insensitive) → pretty JSON only
- [x] Skip harness human stdout + install prompt on JSON path
- [x] **`--install-hooks` still runs**; all status lines → **stderr** on JSON path (enumerate println sites)
- [x] Human path regression (AC7)

### Phase 4 — Hermetic + protocol (AC1–AC8b, AC13–15, AC6)

- [x] Hermetic global + project-scoped + **none** (AC14) + **uppercase JSON** (AC13)
- [x] AC8b install-hooks × JSON purity
- [x] Single-document assert (AC15)
- [x] protocol_compat non-summary 2-key compact still green (verify)
- [x] **Do not** add case-sensitive value_parser (F13)

### Phase 5 — Docs + gate (F20 / AC10–AC12)

- [x] CAPABILITIES: keys + word_count + in_context legacy + scope none + T180 full 2 keys
- [x] PROTOCOL-COMPAT inventory row (summary pretty)
- [x] CHANGELOG Unreleased
- [x] preflight `--format` help text (M6)
- [x] Soft skill one-liner (skipped — soft residual F22 / deferred)
- [x] Manual dogfood AC12 (live vault summary JSON pure object; full path compact 2-key)
- [x] review.md written; targeted nextest + clippy + fmt green
- [ ] Full CI gate via PR; `ledgerful verify` / commit after CI green
- [ ] conductor → Completed; deferred strike T220 / T214 F11 (closeout PR)

## Implement notes

- **PowerShell:** `;` not `&&`.  
- **Do not** edit `.ledgerful/` by hand.  
- **Do not** add fields to `PreflightContextResponse`.  
- **Do not** change T214 SQL count helpers (reuse).  
- **Do not** silent-skip `--install-hooks` on JSON.  
- Prefer pure builder + thin I/O; parameterize status sink (`stdout` vs `stderr`) for install.  
- Capture independence mandatory.

## Manual test script (on go)

```powershell
ai-brains preflight --summary
ai-brains preflight --summary --format json
ai-brains preflight --summary --format JSON
ai-brains preflight --global --summary --format json
ai-brains preflight --summary --format json --install-hooks
ai-brains preflight --format json   # still compact 2 keys
```

Expect: summary+json parses; no banner; install-hooks does not pollute stdout; full json still 2 keys.

## Out of scope reminders

T219 pretty wall · T224 role strip · harness array hard · change unresolved SQL · ledgerful global · clap bump · governed summary authority.
