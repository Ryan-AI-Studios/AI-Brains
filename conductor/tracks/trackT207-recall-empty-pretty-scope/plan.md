# T207 Plan — Recall empty pretty + scope honesty

Status: **In Progress** (implementing). Spec: [spec.md](./spec.md).

## Absorbed

| Residual | Disposition |
|----------|-------------|
| Audit recall FTS empty 3/3 pretty blank | **F3** always print empty pretty hint |
| Scope friction / wrong project looks empty | **F4** empty Scope line + **F6** “this project” clause (no name dupe) |
| T133 TTY-only guard vs agent shells | **F3** reverse for pretty format only |
| T206 wrong `.env` project | Display honesty only (no auto-global) |
| T202 F6 status vs hint | Regression lock AC6 |
| Non-deterministic empty Session UUID | **F5 required** (M4) omit generated Session on empty pretty |
| Cozo INFO pollution | **Out → T208** (F12) |
| Ranking / semantic quality | **Out → T211/T215** |
| Non-empty pretty Scope | **AC10 deferred residual** (M3) |

## Research (2026-08-04)

| Source | Takeaway |
|--------|----------|
| Live repro | `--format pretty` empty → Session-only; json has `hint` |
| `recall.rs` ~281–293 | `is_terminal()` gates pretty empty hint |
| clig.dev | Human-first; suggest next actions; exit 0 on intentional empty OK |
| is-terminal crates.io | **0.4.17** latest; keep workspace `0.4` |
| clap | **4.5** — no new flags for DoD |
| T101/T111/T133/T202 | Defaults + hint core + status precedence preserved |
| Store trait | No `get_project_by_id` today — **F32** add (M1) |

## AI fold-in (2026-08-04)

| ID | Source | Action |
|----|--------|--------|
| **AI1 #1** | Remove TTY guard | **Affirm** → F3 |
| **AI1 #2** | Scope header | **Affirm** → F4 (empty-only after M3) |
| **AI1 #3** | Project-aware hints | **Affirm + M2** → F6 without name duplicate |
| **AI1 #4** | Exit 0 | **Affirm** → F2 |
| **AI1 #5** | Hermetic tests | **Affirm** → F16/F22 |
| **M1** | `get_project_by_id` vs list_projects | **Accept (b)** → **F32**; plan B0 |
| **M2** | F4/F6 name redundancy | **Accept** → F6 “Scoped to this project”; §10.1 |
| **M3** | AC10 non-empty Scope break | **Accept defer** → AC10 residual; empty-only DoD |
| **M4** | Elevate F5 | **Accept** → F5 required; AC9; F21 |
| **M5** | `build_recall_hint_core` signature | **Accept** → **F33** `project_scoped: bool` |
| **L1** | Extract empty pretty printer | **Elevate required** → **F31** |
| **L2** | Combine count + name query | **Soft** preferred with F32 |
| **L3** | Unknown `--format` exit 2 | **Soft residual** (F8); not DoD |
| **L4** | `--export` on recall | **Decline** — not needed |
| **L5** | NO_COLOR / color | **Out** — no color today |

## Phases

### A0 — Expand + fold-in (done)

- [x] Live repro + code map  
- [x] Spec F1–F33 + AC1–AC12  
- [x] AI fold-in disposition  
- [x] Conductor/deferred roll  
- [x] On **go**: ledger TX already started (orchestrator) — implementer did not start another  
- [x] On go: implement on `feat/T207-recall-empty-pretty-scope`  

### A1 — Red tests

- [x] **B0** Unit/store: `get_project_by_id` known id → name/alias (AC11 / F32)  
- [x] **B1** Hermetic: empty pretty non-TTY → stdout contains `No results` (AC1)  
- [x] **B2** Same → contains `Scope:` / `project=` or `global` (AC2)  
- [x] **B3** Empty json → `hint` + `effective_session_id`, exit 0 (AC3)  
- [x] **B4** Unit: `build_recall_hint_core(..., project_scoped=true)` has “this project” / scoped clause; no alias string required (F6/F33); T202 unreachable tests still green (AC6)  
- [x] **B5** Hermetic: empty pretty **no** random Session when session generated (AC9 / F5)  
- [x] **B6** Hermetic: `--quiet` empty pretty still Scope + No results (AC12)  
- [x] **B7** Non-empty pretty still shows hits; no empty hint (AC4)  

### B — Green

- [x] **C0** **F32** trait + QueryStore `get_project_by_id` (single SELECT; soft L2 count join)  
- [x] **C1** **F3** remove TTY guard on pretty empty hint  
- [x] **C2** **F31** extract `format_pretty_empty_state` / print helper (Scope + hint [+ Session if user])  
- [x] **C3** **F4** empty Scope line via F32 (alias/name + uuid)  
- [x] **C4** **F5** omit generated Session on empty pretty  
- [x] **C5** **F6/F33** hint core `project_scoped` clause; no name dupe  
- [x] **C6** F11 quiet regression (already default; lock AC12)  

### C — Docs + closeout

- [x] **D1** CAPABILITIES F23 / AC7 (not TTY-only; empty Scope)  
- [x] **D2** CHANGELOG minor empty UX only (not non-empty rewrite) AC8  
- [x] Soft skill / OPERATIONS one-line  
- [ ] **D3** Review log clean (no open high/medium)  
- [ ] **D4** Full gate + PR; conductor Completed; deferred strike; residual AC10 noted  

## Test plan

| Lock | Assert |
|------|--------|
| AC1–AC3 | empty pretty always + scope; json hint |
| AC4 | non-empty no false hint; no required Scope |
| AC5 | resolve_format T101 |
| AC6 | T202 `build_recall_hint__*` green |
| AC9 | no generated Session on empty pretty |
| AC11–AC12 | get_project_by_id + quiet |
| Soft L2 | combined query if free |

Prefer suite: `recall_empty_pretty_scope` (`hermetic_cmd` / `hermetic_vault` / `init_vault` from T202 `recall_briefing_clarity`).

## Manual (on go / after ship)

- [ ] `ai-brains recall "zzzznonexistentquery999" --format pretty --no-bridge --quiet` → Scope + No results; no random Session if no session env  
- [ ] Same with `--global` → `Scope: global`  
- [ ] Same `--format json` → hint + effective_session_id  
- [ ] Real TTY: default pretty empty still shows hint  
- [ ] Cozo INFO still present if graph-on (T208 owns quiet)  

## Stop-before

- Auto `--global` on empty  
- Changing T112 default scope  
- Silencing Cozo as part of T207  
- Non-empty Scope as silent “minor”  
- Full `list_projects` for one-id Scope  
- Ranking / FTS changes  

## Done when

AC1–AC9 + AC11–AC12 green; AC10 deferred residual recorded; review clear; full gate green; PR merged.

## Implement notes (for implementer)

1. **F31** extract empty pretty composer (required).  
2. **F32** `get_project_by_id` — not `list_projects` filter. Soft L2: fold memory count into same query for small-vault clause.  
3. **F5:** track whether session came from user/resolve vs generated; only omit when generated **and** empty.  
4. Print order empty pretty: **Scope** → (Session if user) → Embedding status if semantic ≠ ok → empty hint.  
5. Do **not** print empty hint when results non-empty.  
6. Keep all existing `build_recall_hint__*` and `resolve_format__*` units green.  
