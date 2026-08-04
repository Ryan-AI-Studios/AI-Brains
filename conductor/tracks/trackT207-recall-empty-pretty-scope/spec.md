# T207 — Recall empty pretty + scope honesty

- **Track ID:** T207-RecallEmptyPrettyScope
- **Phase:** Post-T206 skill·CLI audit follow-ups (P1)
- **Status:** 📋 **Proposed / Expanded + AI fold-in** (plan-only until go)
- **Depends on:** T101 pretty default; T111/T133 hints; T112 project-default scope; T198 empty-success; T202 embedding.status + hint precedence; **T206** detect honesty (PR #89 closed)
- **Blocks / feeds:** Operator cold-start for FTS recall; **T208** Cozo INFO quiet (out of scope here); soft T211/T215 ranking
- **Category:** FEATURE / DOCS
- **Source:** Non-destructive skill/CLI audit 2026-08-04 — recall FTS empty **3/3**; pretty blank; scope friction
- **Deferred absorbed:** Audit empty-pretty blank; scope opacity on empty; T133 TTY-only hint guard under explicit/resolved `pretty`
- **Not absorbed:** Cozo INFO (**T208**); backup SQLCipher WARN (**T209**); POLICY_DENIED bootstrap (**T210**); ranking/semantic relevance (**T211/T215**); project list labels (**T212**); auto `--global`; FTS algorithm; daemon semantic; clap 5 multi-heading; **non-empty pretty Scope line** (AC10 deferred residual — M3)
- **Research date:** 2026-08-04
- **AI fold-in:** 2026-08-04 — AI1 affirms F2–F3/F4/F6/tests. AI2 **M1–M5** accepted; **L1** elevated; **L2** soft; **L3–L5** affirm out/soft. Disposition **§14**.
- **Ledger:** plan-only until go

## 1. Objective

1. **Pretty empty-state honesty:** when recall returns zero hits and format is **pretty**, operators always see the same next-action guidance that JSON already carries in `hint` — not a bare `Session:` line.  
2. **Scope transparency:** empty (and soft: all) pretty output names the **active project scope** (`--global` vs project id/alias) so wrong `.env` / inherited `AI_BRAINS_PROJECT_ID` does not look like “the vault is empty.”  
3. **Keep contracts stable:** empty recall remains **exit 0** (T198 empty-success); JSON envelope shape unchanged (`results` + `hint` + optional `embedding`); no retrieval ranking changes.

## 2. Live baseline (re-scan 2026-08-04)

### 2.1 Audit reproduction — confirmed

| Fact | Value |
|------|--------|
| Default project from local `.env` | `441837f6-…` (**test-alias**, 552 memories) |
| Main vault project | `7d97a456-…` (8390 memories) |
| `recall "zzzz…" --format pretty` (non-TTY shell) | Exit **0**; stdout ≈ `Session: <uuid>` only — **no hint** |
| Same query `--format json` | Exit **0**; `hint` present with `--semantic` / `--global` recipe |
| Same query with main `--project-id` | Non-empty hits (scope matters) |
| Cozo INFO line | Still pollutes stderr/log on graph-on — **T208**, not T207 |

### 2.2 Code map — confirmed

| Issue | Location | Gap |
|-------|----------|-----|
| Empty pretty hint gated on TTY | `recall.rs` ~281–293 `&& std::io::stdout().is_terminal()` | **F3** — agent shells, pipes, `--format pretty` → blank |
| T133 intentional TTY guard | T133 AC1 | Correct for **default non-TTY → json**; wrong once format is **pretty** |
| Pretty header only Session | ~244–247 | No project / global scope line — **F4** |
| Generated session always | ~146–153, ~244–247 | Empty pretty looks “busy” with random Session UUID — soft **F5** |
| Hint core text | `build_recall_hint_core` | Strong next-action; soft: name active project in non-global empty |
| Unit tests for hint core | present | **No hermetic** that empty **pretty non-TTY** prints hint |
| Scope default (project) | T112; `RecallOptions.project_id` | Keep; do **not** auto-widen |

### 2.3 Touch map

| File | Role |
|------|------|
| `crates/ai-brains-cli/src/commands/recall.rs` | Drop TTY gate; empty Scope header; F5 omit generated Session on empty; F6 hint clause; extract `print_pretty_empty_state` (**F31**/L1) |
| Store query trait + QueryStore impl | **`get_project_by_id`** (**F32**/M1) — single SELECT name/alias by id (not full `list_projects`) |
| `crates/ai-brains-cli/tests/recall_empty_pretty_scope.rs` | Hermetic AC1–AC4/AC9 via `hermetic_cmd` / `hermetic_vault` (T202 pattern) |
| Unit in `recall.rs` | `build_recall_hint_core` with `project_label: Option<&str>` (**F33**/M5); scope-header formatter |
| `Docs/CAPABILITIES.md` | F23: always-on pretty empty hint; Scope on empty; not TTY-only |
| Soft: OPERATIONS / skill | one-line if agent-facing |
| `CHANGELOG.md` | minor UX (empty pretty only; **not** non-empty format rewrite) |
| Contracts | **No** DTO change required |

### 2.4 Deps

| Crate | Pin / note |
|-------|------------|
| `is-terminal` | workspace `0.4` (crates.io latest **0.4.17**, 2025-10-23) — keep for `resolve_format` only after F3 |
| `clap` | workspace **4.5** — no new flags for DoD |
| `serde_json` | workspace 1.0 — JSON path unchanged |
| **Zero new crates** | F15 |

## 3. Research summary

| Finding | Application |
|---------|-------------|
| Live non-TTY `--format pretty` empty = Session-only | **F3** always print empty pretty hint when format is pretty |
| clig.dev: human-first output; suggest next commands; empty success may be exit 0 | Keep exit 0; print actionable empty state on human format |
| clig.dev: machine vs human by TTY for **format choice** | T101 already: default json non-TTY; once user/resolved format is pretty, treat as human stream |
| T133: suppress pretty hint on pipe to avoid polluting `\| jq` | Default non-TTY is **json** (T101) — pipe pollution case was over-broad for explicit/resolved pretty |
| T202 F6: status vs hint precedence | Preserve; empty pretty still prints Embedding line when semantic ≠ ok |
| T206 wrong-env project | Empty + scope header makes hijack visible without auto-changing scope |
| Capture independence | FTS empty path never requires models |

## 4. Frozen decisions (F1–F30)

| ID | Decision |
|----|----------|
| **F1 — Scope of track** | Pretty empty-state + scope **display** honesty only. No FTS/ranking/semantic algorithm change. No auto `--global`. No daemon changes. |
| **F2 — Empty success** | Zero results → exit **0** (T198). Hint is informational, not an error. |
| **F3 — Pretty empty hint always** | When `format_str == "pretty"` and `results.is_empty()`, **always** print the built hint to **stdout** (after results loop / status line). **Remove** the `is_terminal()` guard on that print. Rationale: format already selected human output; default non-TTY remains json (T101). |
| **F4 — Scope header (empty pretty only)** | When format is pretty **and** results empty, print a **Scope** line **before** any Session line: |
| | • `--global` → `Scope: global` |
| | • else with `project_id` → `Scope: project=<alias-or-name> (<full-uuid>)` when alias/name known; else `Scope: project=<full-uuid>` |
| | • else no project → `Scope: project=(none)` |
| | **DoD = empty pretty only.** Non-empty pretty Scope (**AC10**) is **deferred residual** (M3) — avoid silent format break for all pretty consumers; do not ship under “CHANGELOG minor.” |
| | **Resolution SOOT (M1/F32):** implement `get_project_by_id(project_id) -> Option<(name, alias)>` on store trait + QueryStore (single SELECT + alias join). **Do not** load full `list_projects()` for one id. Fallback: UUID only if lookup fails. |
| **F5 — Generated session on empty (required — M4)** | When results empty **and** session was **generated** solely for graph provenance (not user `--session` / `--session-prefix` / `--session-last` / resolved env session), **omit** the pretty `Session:` line. JSON still includes `effective_session_id`. Safe: empty path emits **no** `MemoryPinned` (hits loop empty). Rationale: removes non-deterministic UUID from empty pretty (F21 / AGENTS determinism). User-supplied or vault-resolved sessions still print on empty. |
| **F6 — Hint text (no name duplicate — M2)** | Keep T111/T202 next-action core. When **not** global, add a short scope clause that does **not** re-print alias/id (**F4 Scope line carries the name**). Canonical: `… Scoped to this project. Try --semantic …, or --global …` (or equivalent without `project=test-alias` again). Global line already says “across all projects.” Do not restate embedding status cause when status ≠ ok (T202 F6). Pure core takes optional `project_scoped: bool` (or label only if needed for tests — **F33**). |
| **F7 — JSON path** | Unchanged: empty → set `response.hint`; print JSON. No new required fields. Soft: optional future `scope` field **declined** as DoD (avoid contract churn). |
| **F8 — Format defaults** | T101 frozen: TTY default pretty / non-TTY json; explicit `--format` wins. Do not flip defaults. Soft: unknown format → exit 2 (optional residual). |
| **F9 — No auto scope widen** | Never promote empty project search to global automatically. Operator must pass `--global`. |
| **F10 — Capture independence** | FTS empty + pretty hint must not open embedding backend. |
| **F11 — Quiet** | `--quiet` does not suppress empty-result **hint** or empty **Scope** line (already true for hint path — AI2). Hermetic regression: quiet + empty pretty still shows Scope + No results. Quiet still suppresses bridge warnings (T81). |
| **F12 — Cozo / bridge INFO** | **Out of scope** → **T208**. Do not “fix blank” by silencing Cozo here. |
| **F13 — Ranking / FTS** | Untouched. No T105 fallback changes. |
| **F14 — Contracts** | No required DTO change. If implementer adds optional pretty-only helpers, keep pure in CLI. |
| **F15 — Zero new crates** | — |
| **F16 — Hermetic locks** | AC1–AC5 minimum; see §5. Prefer `hermetic_bin` + temp vault; non-TTY is default under test capture. |
| **F17 — High findings** | Re-shipping TTY-only empty pretty; auto-global; exit ≠ 0 on empty; ranking change; hiding Cozo as “fix”; deleting Session from JSON; shipping non-empty Scope under silent “minor”; full `list_projects` for one id. |
| **F18 — Exit codes** | Empty: **0**. Usage/session resolve fails: existing (1/2). No new exit class. |
| **F19 — Review** | FEATURE; primary required. Cross-model soft (low risk). |
| **F20 — Series** | After T206; before T208/T209. |
| **F21 — Determinism** | Stable hint templates; **no random Session line on empty** (F5); sort nothing new; no timestamps in empty text. |
| **F22 — Tests** | Unit: `build_recall_hint_core` project-scoped bool/clause (F33); scope header formatter. Hermetic: empty pretty non-TTY includes `No results` + `Scope:`; F5 no random Session when generated. Regression: JSON `hint` + `effective_session_id`; T101; T202 `build_recall_hint__*` (esp. unreachable next-action-only). |
| **F23 — Docs** | CAPABILITIES: empty pretty always prints next-action (**not TTY-only**); Scope on empty; project default + `--global`. CHANGELOG minor for empty UX. Soft skill one-liner. |
| **F24 — Privacy** | Do not dump other projects’ names beyond active scope id/alias. |
| **F25 — MemoryPinned / graph** | Empty path does not emit pins (existing). F5 omits display only; do not invent pin on empty. |
| **F26 — Soft decline** | Auto-global · change default scope · new JSON `scope` field DoD · man pages · clap 5 · Cozo quiet · T212 labels · **non-empty Scope line (AC10)** · unknown format exit 2 as DoD · color |
| **F27 — after_help soft** | Soft: one empty-result example in recall after_help if free. |
| **F28 — Ledger** | On go: `ledgerful ledger start T207-recall-empty-pretty-scope --category FEATURE`. |
| **F29 — AI fold-in** | §14 disposition applied 2026-08-04. |
| **F30 — Implement order** | M1 `get_project_by_id` → F3 + F31 extract → F4 empty Scope → F5 omit generated Session → F6/F33 hint → hermetic → docs → gate. |
| **F31 — Extract empty pretty printer (L1 elevated)** | Required named helper (e.g. `format_pretty_empty_state` / `print_pretty_empty_state`) composing Scope + hint (+ optional Session when user-supplied). Unit-testable without full CLI spawn. |
| **F32 — get_project_by_id (M1)** | Store trait + QueryStore: `get_project_by_id` → `Option<(name, alias)>`. Soft (L2): single query may also return memory count for small-vault clause to avoid two round-trips — not DoD. |
| **F33 — Hint core signature (M5)** | `build_recall_hint_core` gains `project_scoped: bool` (preferred pure) **or** optional label; impure `build_recall_hint` resolves id→name for **F4 only**, passes scoped flag into core for **F6** (no name string required in core if F4 owns the name). Unit B4 asserts scoped clause without vault. |

## 5. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Empty FTS + `--format pretty` (non-TTY capture) prints hint containing `No results` and suggests `--global` or `--semantic` as appropriate; no TTY dependency | Hermetic |
| **AC2** | Same empty pretty prints **Scope** line (`global` or `project=…`) with alias when vault has one | Hermetic |
| **AC3** | Empty JSON still has non-null `hint`; exit 0; `effective_session_id` still present | Hermetic / unit |
| **AC4** | Non-empty pretty still prints results (no false empty hint); **no** required Scope line on non-empty (AC10 deferred) | Hermetic or unit |
| **AC5** | T101 `resolve_format` defaults unchanged | Existing unit |
| **AC6** | Semantic empty + status ≠ ok: Embedding status line + hint without restating status / embedding-model clause (T202 F6 regression) | Unit (existing + F6 additive) |
| **AC7** | CAPABILITIES: empty pretty always next-action (**not TTY-only**); empty Scope line; project default + `--global` | Doc |
| **AC8** | CHANGELOG documents empty pretty + Scope (minor UX; not non-empty format rewrite) | Doc |
| **AC9** | Generated session **omitted** on empty pretty (F5 required); user session still shown | Hermetic |
| **AC10** | **Deferred residual** — Scope on non-empty pretty (M3); not DoD | Residual |
| **AC11** | `get_project_by_id` unit or store test: known id → name/alias | Unit |
| **AC12** | `--quiet` empty pretty still shows Scope + No results (F11) | Hermetic |

## 6. Non-goals

- Auto `--global` or changing T112 default project scope  
- Cozo / bridge INFO quiet (**T208**)  
- Backup list SQLCipher noise (**T209**)  
- Ranking / semantic relevance (**T211/T215**)  
- Project list human labels (**T212**)  
- New contracts fields as DoD  
- Daemon / HTTP  
- clap 5 multi-heading  
- Scope line on **non-empty** pretty (AC10 residual)  
- Unknown `--format` hard-fail as DoD (soft residual)  

## 7. Risk & verification

| Risk | Mitigation |
|------|------------|
| Pipe pollution with pretty empty | Default non-TTY is json; explicit pretty is human |
| Scripts parsing empty Session-only pretty | Scope+hint additive on empty; CHANGELOG |
| Non-empty pretty break | **AC10 deferred** (M3) |
| F5 / graph provenance | No pins on empty; JSON keeps `effective_session_id` |
| Alias lookup cost | **F32** single-id query; not full list |
| Cozo still noisy | Explicit residual → T208 |
| F6 duplicates F4 name | **M2** — hint “this project” only |

**Implement order:** F32 query → F3 + F31 extract → F4 → F5 → F6/F33 → hermetic AC1–4/9/11/12 → docs → gate.

## 8. Residual after ship

- T208 Cozo INFO  
- Soft unknown `--format` exit 2 (L3)  
- Soft JSON `scope` field  
- **AC10** non-empty pretty Scope (M3)  
- Soft L2 combined count+name query if not shipped  
- T212 alias labels in project list  
- Ranking quality  

## 9. Series

T197–T204 closed → T205–T206 closed → **T207** → T208–T216.

## 10. Normative text sketches

### 10.1 Pretty empty (project-scoped FTS) — name only on Scope (M2)

```
Scope: project=test-alias (441837f6-5c55-d075-0000-000000000000)
No results for 'zzzz'. Scoped to this project. Try --semantic for embedding-based search, or --global to search across all projects.
```

Do **not** repeat `test-alias` in the hint line. No `Session:` when session was generated (F5).

### 10.2 Pretty empty (global)

```
Scope: global
No results for 'zzzz' across all projects. The vault may be empty or the query may not match any memories.
```

### 10.3 CAPABILITIES (normative addition)

Under recall **Hints** / **Scope**:

- Empty **pretty** always prints next-action text on stdout (**not TTY-only**).  
- Empty pretty prints **Scope:** (`global` or active `project=…`).  
- Default scope remains project (`AI_BRAINS_PROJECT_ID` / flags); use `--global` to widen.

## 11. Verification plan

```powershell
# After implement (not at plan-only):
cargo nextest run -p ai-brains-cli -E 'test(recall_empty) or test(build_recall_hint) or test(resolve_format) or test(get_project_by_id)'
cargo clippy -p ai-brains-cli --all-targets -- -D warnings
# Full gate before PR
.\scripts\dev-check.ps1
```

Manual:

```powershell
ai-brains recall "zzzznonexistentquery999" --format pretty --no-bridge --quiet
# Expect: Scope line + No results; no random Session if no session env (F5)
ai-brains recall "zzzznonexistentquery999" --format json --no-bridge --quiet
# Expect: hint field + effective_session_id, exit 0
```

## 12. AI fold-in disposition (2026-08-04)

See plan § AI fold-in for full table. Summary: AI1 affirm F3/F4/F6/F2/tests. AI2 M1–M5 + L1 accepted; M3 AC10 deferred residual; L2 soft; L3–L5 out/affirm.

## 13. Stop-before

- Auto `--global`  
- Non-empty Scope as silent minor  
- Silencing Cozo as T207 “fix”  
- Full `list_projects` for one-id Scope  
- Ranking / FTS algorithm  

## 14. Fold-in cross-ref

| Review ID | Spec decision |
|-----------|---------------|
| AI1 #1–5 | F3, F4, F6, F2, F16/F22 |
| AI2 M1 | F32 + F4 |
| AI2 M2 | F6 + §10.1 |
| AI2 M3 | AC10 deferred residual |
| AI2 M4 | F5 required + AC9 |
| AI2 M5 | F33 |
| AI2 L1 | F31 required |
| AI2 L2 | soft F32 note |
| AI2 L3–L5 | F8 soft / F26 decline |
