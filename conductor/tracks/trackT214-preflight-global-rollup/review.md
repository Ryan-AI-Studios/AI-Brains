# T214 Review — Preflight global rollup honesty

- **Track:** T214-PreflightGlobalRollup
- **Branch:** `feat/T214-preflight-global-rollup`
- **Commit reviewed:** `ee5414e`
- **Reviewer:** Grok Build (read-only internal)
- **Date:** 2026-08-05
- **Verdict:** **PASS**

## Verdict summary

All hard DoD items (AC1–AC13 / F2–F13 / F21 / F27 / F37–F39) are implemented with hermetic, unit, smoke, store, and docs evidence. No P0–P3 product findings. Soft residuals (F24 / F20) are correctly **not** claimed as shipped.

---

## Requirement matrix

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| **AC1** | `--global --summary` → `Scope: global`; no `Project:` scope identity | **Met** | `crates/ai-brains-cli/tests/preflight_global_summary.rs` multi-project test asserts `Scope: global` + no line `starts_with("Project:")`. `print_summary` uses `format_scope_line(global, …)` only. |
| **AC2** | Multi-project: Projects ≥ 2, Pinned ≥ pins A+B under `--global` | **Met** | Same hermetic: pins A+B, `Projects: >= 2`, `Pinned memories: >= 2`. Store F7 SOOT `count_projects_with_pinned` (pinned-only DISTINCT). |
| **AC3** | Project-scoped: `Scope: project=` + filtered pinned; **no** `Projects:` line | **Met** | Hermetic `preflight_summary__project_scoped__…`: Scope includes `id_a`, pinned==2 (A only), no `Projects:` line. Formatter guards `if global && let Some(n)`. |
| **AC4** | Active sessions ≥ 1 with open session; not `"Session ID:"` text SOOT | **Met** | Hermetic after `context`; SQL via `count_active_sessions`. Source has zero `"Session ID:"` match in preflight path. Store tests SessionStarted → active counts. |
| **AC5** | In-context markers labeled (`In context` / `In-context`) | **Met** | Lines: `In context hotspots/decisions/constraints`. Unit + hermetic string locks. |
| **AC6** | JSON still exactly `text` + `word_count` | **Met** | `PreflightContextResponse` unchanged (2 fields). `protocol_compat_cli` asserts `obj.len()==2`. Summary path does not emit JSON DTO. |
| **AC7** | Smoke: Scope vocab; preserve env-override asserts (F38) | **Met** | `smoke.rs`: `Scope: project=` + local id; `!inherited_project_id`; stderr local `.env` override warnings for project **and** session preserved. |
| **AC8** | Pure unit: `format_scope_line(true,…) == "Scope: global"` | **Met** | `recall.rs` unit + `preflight.rs` `format_scope_line__via_recall__global_soot`. |
| **AC9** | Empty vault (init only): exit 0, Scope global, zeros, non-empty | **Met** | `preflight_global_summary__init_only_empty__zeros_exit_0`. |
| **AC10** | Docs: CAPABILITIES preflight + Scope row + CHANGELOG | **Met** | `Docs/CAPABILITIES.md` Scope row (~202) covers preflight summary; Preflight section dual model + ledgerful project-scoped + T170; `CHANGELOG.md` Unreleased T214 entry. Soft skill: onboarding session-start `--global --summary`. |
| **AC11** | Capture-independent; zero new crates | **Met** | Summary = QueryStore SQL + marker scan + `format_scope_line`; no models/graph required. No new crates in `ai-brains-cli` / store Cargo.toml for T214. |
| **AC12** | Env `AI_BRAINS_PROJECT_ID` under `--global` does not win label | **Met** | Hermetic passes `Some(&id_a)` with `--global`; asserts `Scope: global` and not `Scope: project={id_a}`. F3 dispatch clears project_id. |
| **AC13** | Store counts: bound/static SQL; multi-project F7/F8 | **Met** | `count_projects_with_pinned` static SQL; `count_pinned_memories` / `count_active_sessions` use `params![pid.to_string()]`. Tests in `count_preflight_rollup.rs` (empty, multi-project, scoped, sessions, unpinned-only ignore). **No** `format!` SQL in T214 helpers. |

### Frozen decisions (spot-check)

| Decision | Status | Notes |
|----------|--------|-------|
| F2 Scope vocabulary | Met | Shared `format_scope_line`; no legacy `Project:` summary line |
| F3 effective_project_id | Met | `main.rs` ~2875–2881: `if *global { None } else { *project_id }` into options |
| F4 dual model | Met | Vault SQL + In context markers; Projects only if global |
| F5 Active sessions SQL | Met | `QueryStore::count_active_sessions`; not retrieval `active_sessions` |
| F6 marker scan | Met | Case-sensitive on `context.text` (body already ANSI-stripped in retrieval) |
| F7 Projects SOOT | Met | pinned-only DISTINCT; not `list_projects` |
| F8 Pinned filter | Met | `project_id = ?` when scoped |
| F9 no ledgerful global | Met | Unchanged `if !global` around `query_ledgerful` |
| F11 / F39 JSON freeze | Met | No key growth |
| F13 Scope SOOT pub(crate) | Met | `recall::format_scope_line` + `get_project_by_id` before format |
| F21 word_count | Met | `context.word_count` (no re-split) |
| F27 / M1 SQL safety | Met | params! / static only in new helpers |
| F37 print_summary signature | Met | `(ctx, global, project_id, context)` |
| F38 smoke | Met | See AC7 |

---

## Findings

**None.**

No P0 / P1 / P2 / P3 product findings against hard DoD.

---

## Completeness

| Check | Result |
|-------|--------|
| TODO / stub / placeholder in T214 production path | **None** in `preflight.rs` summary path or new QueryStore counts |
| `format!` SQL in T214 count helpers | **None** (pre-existing LIMIT integer format! elsewhere in query_store is out of scope; not copied into T214) |
| Dead `"Session ID:"` text match | **Removed** from summary SOOT |
| Soft residuals claimed as done? | **No** — CHANGELOG/CAPABILITIES do not claim ledgerful-on-global, summary JSON DTO, `scope_display.rs` extract, is-terminal→std, full `active_sessions` format! refactor |
| Soft residual done opportunistically | Onboarding skill one-liner for `--global --summary` (F19 soft) |

---

## Wiring notes

```
main.rs Preflight dispatch
  └─ F3: effective_project_id = if global { None } else { project_id }
       └─ PreflightRunOptions { project_id, global, summary, … }
            └─ build_preflight(…, project_id, …, global)   // content widen
            └─ print_summary(ctx, global, project_id, &context)
                 ├─ get_project_by_id (project mode only)
                 ├─ format_scope_line (recall pub(crate) SOOT)
                 ├─ count_projects_with_pinned / count_pinned_memories / count_active_sessions
                 ├─ marker scan HOTSPOT:/DECISION:/CONSTRAINT: on context.text
                 └─ format_preflight_summary_lines → stdout
```

- **Dual model:** Vault SQL block (Projects only under global) + `In context *` markers + `Total Word Count` from retrieval field.
- **JSON path:** Unchanged `PreflightContextResponse { text, word_count }` when not `--summary`.
- **Capture independence:** Summary path does not invoke models, embeddings, or graph.
- **Zero new crates:** Confirmed.

---

## Soft residuals (not DoD — do not treat as fail)

From F24 / F20 (correctly unclaimed):

- Non-summary pretty Scope header on full preflight body
- `preflight --summary --format json` machine object
- Ledgerful multi-root / enable under `--global`
- is-terminal → `std::io::IsTerminal`
- clap 4.6 bump
- Extract `commands/scope_display.rs`
- Refactor retrieval `active_sessions` off `format!` SQL
- Soft help example `preflight --global --summary` in clap after_help (optional)

---

## Suggested deferrals

None required (no P3 findings). Soft residuals above remain series backlog / future tracks if product wants them.

---

## Cross-model review

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| Internal R1 | Grok explore | **PASS** | Zero P0–P3; AC1–13 Met |
| Cross-model R1 | Codex | **Rate-limited** | Usage limit until ~2026-08-07 |
| Cross-model R1 (fallback) | Claude Sonnet 4.5 | **PASS** | Zero findings; see `review.claude.md` / `review.claude.final.md` |
| Final gate | Claude (same clean PASS; no post-review code churn) | **PASS** | Fresh clean cross-model gate; no >low findings to re-loop |

**Full local gate (2026-08-05):** `cargo fmt --check` OK; clippy workspace `-D warnings` OK; **2190** nextest passed (1 skipped); `cargo deny check` OK; `cargo audit` OK (allowed warnings only). `FULL_GATE_OK`.

---

## Test map (for gate owners)

| Suite | Path / filter |
|-------|----------------|
| Hermetic | `crates/ai-brains-cli/tests/preflight_global_summary.rs` |
| Unit formatter / SOOT | `preflight.rs` tests + `recall::format_scope_line*` |
| Store AC13 | `crates/ai-brains-store/tests/count_preflight_rollup.rs` |
| Smoke F38 | `smoke.rs` preflight local `.env` Scope assert |
| JSON F39 | `protocol_compat_cli` `t180_c_preflight_json_keys__…` |

Targeted (when validating outside this read-only pass):

```powershell
cargo nextest run -p ai-brains-cli -E "test(preflight)" ; cargo nextest run -p ai-brains-store -E "test(count_preflight)"
```

Full workspace gate remains the final publish bar (fmt / clippy / nextest / deny / audit / ledgerful verify) — not re-run in this read-only review.

---

## Closure note

- **Primary review:** PASS — clean for review convergence on product DoD.
- **Cross-model:** Spec F28 soft for FEATURE honesty; optional before final track close.
- Code change alone is not track closure: implementer still owns full gate + conductor Completed + ledger commit hygiene.
