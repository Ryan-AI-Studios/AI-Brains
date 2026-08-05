# T212 Internal Review

- **Reviewer:** internal (read-only code review)
- **Date:** 2026-08-05
- **Track:** T212 Project list human labels
- **Authority:** `spec.md` (F1–F42, AC1–AC12), `plan.md`, AGENTS.md Review & Severity
- **Scope inspected:**
  - `crates/ai-brains-store/src/lib.rs` (`QueryStore` + `ProjectListDetail`)
  - `crates/ai-brains-store/src/query_store.rs` (`list_projects` ORDER BY, `list_projects_detail` SQL)
  - `crates/ai-brains-cli/src/commands/project.rs` (`list`, helpers, footer, units)
  - `crates/ai-brains-cli/src/main.rs` (`ProjectCommands::List` + dispatch)
  - `crates/ai-brains-cli/tests/project_list_labels.rs`
  - `Docs/CAPABILITIES.md`, `CHANGELOG.md`
  - smoke friendly-name + empty_states (regression surface)

## Verdict: CLEAN

Hard DoD (AC1–AC9 + AC11; F4–F9, F11, F13, F16, F36; wiring; docs honesty) is met in code and hermetic/unit coverage. Soft AC10 (path seed) deferred per plan; soft AC12 covered by hermetic. No critical/high/medium findings. Two optional lows only (test assert tightness + smoke message drift). Full CI gate (AC9) not executed in this read-only review session.

---

## Requirement matrix (AC/F → Met/Partial/Unmet + evidence)

### Acceptance criteria

| ID | Status | Evidence |
|----|--------|----------|
| **AC1** alias → human label contains acme | **Met** | Hermetic `project_list__alias_acme__human_label_contains_acme`; header asserts `label`; `display_label` unit returns alias first |
| **AC2** no-alias / baked → exact `(no alias)` | **Met** | Hermetic `project_list__no_alias__label_exactly_no_alias` (strips ` — short`); unit `display_label__baked_no_alias_prefix__literal_no_alias`; context seeds `(no alias) — {short}` |
| **AC3** unaliased footer stderr + id; table stdout | **Met** | Hermetic `project_list__unaliased__stderr_set_alias_footer`; `print_unaliased_footer` uses `eprintln!`; asserts footer not on stdout |
| **AC4** empty vault T198; no footer | **Met** | Hermetic `project_list__empty_vault__t198_no_footer`; early return before footer; empty_states regression still checks T198 line + header |
| **AC5** `--format json` shape + no footer | **Met** | Hermetic `project_list__format_json__shape_and_unaliased_count`; clap `value_parser = ["human","json"]` only (no dual `--json`); envelope fields present |
| **AC6** memory → last_activity non-empty JSON | **Met** | Hermetic pin + JSON assert; SQL `COALESCE(mem.last_activity, p.updated_at)` with `MAX(updated_at)` |
| **AC7** `list_projects` 4-tuple preserved | **Met** | Signature `(String,String,String,usize)` unchanged; detect/resolve/init/set_alias still call `list_projects()`; only `VaultConnection` implements `QueryStore` |
| **AC8** CAPABILITIES + CHANGELOG (+ last_activity semantic) | **Met** | CAPABILITIES § project list: columns, footer stderr, json, last_activity = projection mutation, path honesty; CHANGELOG Unreleased T212 entry |
| **AC9** full CI gate; no production panic | **Partial (review)** | No production `unwrap`/`expect`/`panic!` on T212 list/detail path (units only). Full `fmt/clippy/nextest/deny/audit` not run this session — operator gate still required before ship |
| **AC10** soft path seed | **Soft / deferred** | Path subquery implemented; no hermetic path seed (plan: soft deferred) |
| **AC11** multibyte truncate no panic | **Met** | `truncate_chars` via `.chars().take`; unit `truncate_chars__multibyte_at_width__no_panic` (CJK + em-dash) |
| **AC12** soft active `*` | **Met (soft)** | Hermetic `project_list__active_project_id__star_prefix_on_label`; human path prefixes `*` when env matches |

### Frozen decisions (checklist F items)

| ID | Status | Evidence |
|----|--------|----------|
| **F4** display_label order | **Met** | (1) non-empty alias; (2) `starts_with("(no alias)")` → literal; (3) `Project ` + uuid-ish / full or short id; (4) else name. No regex. Units cover all arms |
| **F5** human columns | **Met** | `label \| project_id \| memories \| last_activity \| path` |
| **F6** path scalar subquery | **Met** | Correlated `(SELECT … ORDER BY normalized_path ASC LIMIT 1)` — not multi-row JOIN |
| **F7** last_activity semantic + display | **Met** | SQL COALESCE MAX(mp.updated_at), p.updated_at; CAPABILITIES honesty; relative helper `<365d` |
| **F8** footer stderr only | **Met** | `eprintln!`; empty early-return; JSON path skips footer |
| **F9** `--format` only; JSON schema | **Met** | `default_value = "human"`, parser human\|json; no List `--json`; pretty envelope with required fields + path null |
| **F11** 4-tuple + detail method | **Met** | `list_projects_detail` + `ProjectListDetail` on store; list UI uses detail only |
| **F13 / F41** ORDER BY both methods | **Met** | Both SQL: `ORDER BY memory_count DESC, p.project_id ASC` |
| **F14** truncation widths | **Met** | Label 30 chars; path 40; project_id never truncated |
| **F15** empty vault | **Met** | Header + `No projects registered. (0 projects)` |
| **F16** active `*` on human label | **Met** | Process env match → `*{label}`; missing/invalid → no star; JSON soft `active` |
| **F36** char-safe truncate | **Met** | No byte slice of names; `chars().take`; AC11 unit |
| **F18** zero new crates / no clap bump | **Met** | clap 4.5 value_parser only |
| **F10 / F2 / F3** display-only, no auto-alias, no prompt | **Met** | List/query path only; footer non-interactive |
| **F26** soft git suggestion | **Met (bonus)** | `footer_alias_suggestion` via `get_git_repo_slug` + sanitize; fallback `my-project` |
| **F24** soft verbose | **Deferred** | Plan: not free — OK soft |
| **F40** smoke friendly name | **Met** | Still asserts `(no alias)` substring (message text slightly stale — L1) |

---

## Audit checklist results

| # | Check | Result |
|---|--------|--------|
| 1 | AC1–AC9 + AC11 DoD; AC10/AC12 soft | **Met** (AC9 gate not re-run here) |
| 2 | F4 display_label order exact | **Met** |
| 3 | F6 path scalar subquery | **Met** |
| 4 | F8 footer stderr; empty/JSON no footer | **Met** |
| 5 | F9 `--format json` only; schema fields | **Met** |
| 6 | F11 4-tuple preserved | **Met** |
| 7 | F13 ORDER BY both methods | **Met** |
| 8 | F16 active `*` on human label | **Met** |
| 9 | F36 char-safe truncate | **Met** |
| 10 | No production unwrap/expect/panic | **Met** on T212 path |
| 11 | Tests prove ACs / fail on old behavior | **Met** (label header, strip baked, stderr footer, JSON schema, truncate unit) |
| 12 | CAPABILITIES last_activity semantic | **Met** |
| 13 | TODO/FIXME/stubs in T212 code | **None found** |
| 14 | List format flag wired to `list()` | **Met** — `ProjectCommands::List { format } => commands::project::list(&ctx, format)` |

---

## Findings

### [T212-R1-L1] low — AC5 `unaliased_count` assert is loose (`>= 1`)

**severity:** low  
**description:** Hermetic JSON test seeds one aliased + one unaliased project but asserts `unaliased_count >= 1` rather than exact `1`. A double-count regression would still pass.  
**files:** `crates/ai-brains-cli/tests/project_list_labels.rs` (`project_list__format_json__shape_and_unaliased_count`)  
**required_fix (optional):** `assert_eq!(unaliased, 1)` for the two-project fixture.  
**status:** open  

### [T212-R1-L2] low-info — smoke friendly-name failure message still describes baked form as list output

**severity:** low  
**description:** `test_project_list_friendly_default_name` correctly asserts `stdout.contains("(no alias)")` (F40), but the assert message still says the list should show `'(no alias) — <short-uuid>'`. Product list now shows literal `(no alias)` with full id in `project_id` column; baked form remains the *stored* name only.  
**files:** `crates/ai-brains-cli/tests/smoke.rs`  
**required_fix (optional):** Update message/comment to match label-first display.  
**status:** open  

---

## Key implementation evidence (snippets)

### F4 `display_label`

```161:172:crates/ai-brains-cli/src/commands/project.rs
pub(crate) fn display_label(name: &str, alias: &str, project_id: &str) -> String {
    if !alias.is_empty() {
        return alias.to_string();
    }
    if name.starts_with("(no alias)") {
        return "(no alias)".to_string();
    }
    if is_non_human_project_name(name, project_id) {
        return "(no alias)".to_string();
    }
    name.to_string()
}
```

### F6 / F13 detail SQL

```358:379:crates/ai-brains-store/src/query_store.rs
            SELECT
                p.project_id,
                p.name,
                COALESCE(a.alias, '') AS alias,
                COALESCE(mem.memory_count, 0) AS memory_count,
                COALESCE(mem.last_activity, p.updated_at) AS last_activity,
                (
                    SELECT normalized_path
                    FROM repository_path_alias_projection r
                    WHERE r.project_id = p.project_id
                    ORDER BY r.normalized_path ASC
                    LIMIT 1
                ) AS path
            ...
            ORDER BY memory_count DESC, p.project_id ASC
```

### F9 clap + wiring

```1405:1409:crates/ai-brains-cli/src/main.rs
    List {
        /// Output format: human (default table) or json
        #[arg(long, default_value = "human", value_parser = ["human", "json"])]
        format: String,
    },
```

Dispatch: `ProjectCommands::List { format } => commands::project::list(&ctx, format)`.

### F36 truncate

```210:221:crates/ai-brains-cli/src/commands/project.rs
pub(crate) fn truncate_chars(s: &str, max_chars: usize) -> String {
    ...
    let truncated: String = s.chars().take(keep).collect();
    format!("{truncated}…")
}
```

---

## Soft residuals (non-blocking; not findings)

| Item | Disposition |
|------|-------------|
| AC10 path hermetic | Soft deferred (plan) — subquery + CAPABILITIES honesty shipped |
| F24 `--verbose` raw name | Soft deferred (plan) |
| AC9 full gate run | Operator/CI before ship |
| Manual live vault | Plan phase 4 residual |

---

## Completeness / placeholders

- No TODO/FIXME/unimplemented/stub in T212 list/detail implementation.
- Soft F26 git slug footer suggestion implemented.
- Smoke `(no alias)` + empty_states T198 regressions remain compatible with label-first header (`project_id` still present).

## Ship readiness (review lens)

| Gate | Status |
|------|--------|
| Hard ACs / core Fs in code + tests | **Pass** |
| Docs honesty (M4 last_activity) | **Pass** |
| Medium+ findings | **None** |
| Full CI / manual live list | **Not verified this session** |

**Verdict remains CLEAN.** Optional tighten L1/L2 if free; not required for medium+ clearance.

---

## Cross-model review (2026-08-05)

- **Codex:** rate-limited (usage limit) — no findings file produced (eview.codex.raw.log).
- **Claude (fallback):** **PASS** — full audit in eview.claude.md. No P0–P3 findings. Soft AC10 path seed + F24 verbose remain plan-deferred (not new deferred.md debt).
- **Final gate:** Fresh clean Claude PASS with no open findings greater than low. Ready for PR/CI.

## Local verification (orchestrator)

| Gate | Result |
|------|--------|
| cargo fmt | OK |
| cargo clippy --workspace --all-targets -- -D warnings | OK |
| cargo nextest run --workspace | **2127 passed** (1 skipped) |
| cargo deny check | OK (existing wildcard warnings) |
| cargo audit | OK |
| Manual live vault | label-first, active *, last_activity relative, stderr set-alias footer, JSON schema OK |
