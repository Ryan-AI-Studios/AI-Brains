# T231 Internal Review

**Reviewer:** strict read-only primary  
**Date:** 2026-08-11  
**Scope:** AC1–AC14, F1–F40, completeness sweep (query-path only)  
**Method:** Spec/plan vs code + unit/hermetic tests + docs; no edits, no gate re-run

## Verdict: CLEAN

All hard DoD items for T231 are implemented with matching evidence. No critical/high/medium open defects. Soft residuals are expected track leftovers (F22/F24/etc.), not ship blockers. AC12 full workspace gate remains a **close-out process step** (plan unchecked), not a code finding.

---

## AC Matrix

| AC | Status | Evidence |
|----|--------|----------|
| **AC1** | **met** | Unit `resolve_sync_project_id__missing_env__returns_none` — `crates/ai-brains-cli/src/commands/sync.rs` (~732–734): `resolve_sync_project_id(false, None) == None`. Helper body (~394–406) matches F29 SOOT. |
| **AC2** | **met** | Unit `resolve_sync_project_id__invalid_string__returns_none_stable` (~739–750): invalid twice → both `None` and equal; also refuses literal `"default-project"`. No `ProjectId::new()` path. |
| **AC3** | **met** | Unit `resolve_sync_project_id__valid_uuid__returns_some` (~755–760): fixture UUID → `Some(that id)`. |
| **AC4** | **met** | Unit `resolve_sync_project_id__global_true__returns_none_regardless_of_env` (~765–772): global ignores missing/valid/invalid env. |
| **AC4b** | **met** | Unit `resolve_sync_project_id__whitespace_only__returns_none` (~777–781): `""`, `"   "`, `"\t\n"` → `None` (F38). |
| **AC5** | **met** | Hermetic `sync_query__missing_project_env__scope_project_none_no_random_uuid` — `crates/ai-brains-cli/tests/sync_query_ux.rs` (~71–114): tempdir + `--no-project-context` (F30); asserts `Scope: project=(none)` + no UUID tokens on Scope line. **Would fail pre-T231** (random UUID Scope). |
| **AC6** | **met** | Hermetic `sync_query__invalid_project_env__scope_project_none` (~121–159): `.env("AI_BRAINS_PROJECT_ID", "not-a-uuid")` + exit 0 + `project=(none)` + no echo of invalid string. **Would fail pre-T231** (random UUID). |
| **AC7** | **met** | Hermetic `sync_query__valid_project_with_ingest__returns_scoped_hit` (~166–235): PROJECT_A hit present; PROJECT_B isolation (no cross-project leak). |
| **AC8** | **met** | Unit `build_recall_hint__include_sync_query_hint_true__appends_ledger_next_step` (`recall.rs` ~947–970) + hermetic `recall_empty__pretty__includes_sync_query_next_step` (`sync_query_ux.rs` ~355–392). F13 lead-in + `sync query` present; core `--semantic`/`--global` retained (F40). Call site empty pretty: `true` (`recall.rs` ~310–318). |
| **AC8b** | **met** | Unit `build_recall_hint__include_sync_query_hint_false__no_sync_query_self_mention` (`recall.rs` ~975–987) + hermetic `sync_query_empty__pretty__no_sync_query_self_mention` (`sync_query_ux.rs` ~399–438). `print_pretty_empty_sync` passes `false` (`recall.rs` ~493–494). |
| **AC9** | **met** | `Docs/CAPABILITIES.md` §15 “Start here: which search?” (~517–535) decision table with **F8** `text` row + **F36** invalid-env row; sync section F32/F33 (~343–345). `Docs/WORKFLOWS.md` §5 “Find something” (~159–190). Root `CHANGELOG.md` Unreleased T231 row (~20). Ambiguous “Code + memory \| recall or sync query” row **gone**. |
| **AC10** | **met** | `main.rs` Recall doc (~139): peer points at `sync query` for vault+ledger. Sync Query docs (~1800–1807): human vault+ledger / always-pretty; agents → `recall`. Source-level; no dedicated hermetic lock (acceptable for help chrome). |
| **AC11** | **met** | `resolve_format` unchanged (`recall.rs` ~30–40); units `resolve_format__no_explicit_not_tty__returns_json` / TTY pretty (~758–779). No regression of TTY/json split. |
| **AC12** | **process pending** | Code/tests/docs in place; contracts crate **untouched** by T231 (grep: no T231 surface in `ai-brains-contracts`). Full `fmt/clippy/nextest/deny/audit` + `ledgerful verify` still listed unchecked in plan §5 — **close-out obligation**, not an implementation defect. |
| **AC13** | **met** | Hermetic `sync_query__global_flag__scope_global` (`sync_query_ux.rs` ~242–276) + existing isolation `sync_query_pretty_global_flag_returns_cross_project_results` (`sync_query_isolation.rs`). |
| **AC14** | **met** | F21 wire: ndjson passes `Option` into `RecallOptions` and `BridgeRecord.project_id = project_id.map(...).unwrap_or_default()` (`sync.rs` ~433–468); **no** `unwrap_or_else(ProjectId::new)`. Hermetic `sync_query__ndjson_no_project__project_id_field_empty` (`sync_query_ux.rs` ~283–348) asserts `"project_id": ""` on emitted records. |

---

## Findings

_None open._

---

## Soft residuals only (P3)

Expected per spec §10 / F22–F24; **not** DoD:

| Item | Notes |
|------|--------|
| Top-level `search` alias (F4/F24) | Soft residual O2 — not required |
| `recall --format text` → pretty arm (F8/F22) | Documented asymmetry; arm not DoD |
| Non-empty pretty ledger footer (F23) | Residual if noisy |
| Invalid-env clap/manual converge (F36) | Documented only — correct |
| `is-terminal` → stdlib (L8) | Soft residual |
| Help AC10 hermetic lock | Optional polish; source evidence sufficient |
| AC12 full gate + ledger commit / deferred strike | Plan close-out steps |

Out of query-path (not T231 defects): `sync` **pull** (~151) and **push** (~245–248) still use `ProjectId::new()` on missing/invalid interchange or env — **outside** `run_query` / F10 scope.

---

## Completeness

### Project resolve (F10/F29/F32)

| Check | Result |
|-------|--------|
| Pure helper SOOT matches §12 | **Yes** — global short-circuit; trim; empty → None; `from_str.ok()` |
| Call-site no `"default-project"` | **Yes** — `env::var(...).ok().as_deref()` only (`sync.rs` ~420–424) |
| Call-site no `ProjectId::new()` on query path | **Yes** |
| Remaining `default-project` in query path | **None** (only comments/tests) |
| Remaining `ProjectId::new` in `run_query` | **None** |

### Hint gate (F12/F13/F37/F40)

| Check | Result |
|-------|--------|
| Recall empty pretty includes F13 | **Yes** (`include_sync_query_hint: true`) |
| Sync empty does **not** self-mention | **Yes** (`print_pretty_empty_sync` → `false`) |
| JSON empty path no sync next-step | **Yes** (`false` at `recall.rs` ~362–370) |
| Non-empty hit lists not spammed | **Yes** (hint only on empty pretty) |
| Additive to core lexical hint | **Yes** (append after `--semantic`/`--global` block) |
| Circular hint risk | **Closed** by F37 + AC8b |

### End-to-end wiring

```text
env AI_BRAINS_PROJECT_ID
  → resolve_sync_project_id(global, env.ok().as_deref())  // Option<ProjectId>
  → RecallOptions.project_id (pretty + ndjson)
  → resolve_active_scope_line / print_pretty_empty_sync
       → Scope: project=(none) | project=<uuid> | Scope: global
  → ndjson BridgeRecord.project_id = map.unwrap_or_default() → ""
```

Scope honesty reuses T228 `resolve_active_scope_line` (F11). Vault-wide on `None` matches recall-without-project (documented CAPABILITIES F32).

### Regressions

| Surface | Status |
|---------|--------|
| `resolve_format` TTY pretty / non-TTY JSON | Unchanged + unit-locked |
| Sync always-pretty default (F7/F33) | Intentional: `format.unwrap_or_else(|| "pretty")`; docs F33 honesty present |
| `sync query --format text` ≡ pretty | Still only `ndjson` special-cased |
| Isolation tests (`sync_query_isolation.rs`) | Compatible with honest Option scope (valid PROJECT_B still scopes empty) |
| Contracts / DTO growth | **None** — `BridgeRecord.project_id: String` required → `""` only (F19/F21) |
| Ranking / semantic / engine merge | Untouched (F1/F5/F34) |

### Rust safety (F27)

- No new production `unwrap()` / `expect()` on T231 paths.
- Only test-module `expect` in `sync.rs` unit (~759) and existing recall test helpers.
- `Option::unwrap_or_default()` for BridgeRecord is safe and specified (F21).

### Test quality (would old random-UUID fail?)

| Test | Pre-T231 failure mode |
|------|------------------------|
| AC5 missing env | Scope was `project=<random uuid>` → fail `project=(none)` + UUID token assert |
| AC6 invalid env | Same random fallback → fail `project=(none)` |
| AC2 double-call | `ProjectId::new()` twice → unequal `Some` (if tested that way); pure None is stable |
| AC14 ndjson | `unwrap_or_else(ProjectId::new)` + `Some(pid)` → non-empty `project_id` field → fail `""` |
| AC8b | Without F37 flag, shared hint would self-mention → fail |

Hermetics correctly use F30: `hermetic_bin` ambient strip + `--no-project-context` + tempdir; explicit `.env(...)` survives for invalid/valid cases.

### Docs honesty (F8/F33/F36)

| Item | Present |
|------|---------|
| F8 text asymmetry in CAPABILITIES decision table | **Yes** |
| F36 invalid-env recall exit 2 vs sync exit 0 | **Yes** (table + WORKFLOWS) |
| F33 always-pretty intentional | **Yes** (CAPABILITIES §15 + sync feature table + clap help) |
| CHANGELOG T231 row | **Yes** (repo root `CHANGELOG.md`; plan’s `Docs/CHANGELOG.md` path is historical naming — content lives at root SOOT) |

---

## Closure notes for implementer

1. Run full CI gate (AC12) and record outcome before ledger commit.  
2. Soft residual list above may go to `conductor/ISSUES.md` / deferred if desired — none require fix-loop.  
3. Cross-model review remains soft (F35) — not required for CLEAN primary.

**Primary review status:** `CLEAN` — ready for gate + ship close-out.

---

## Codex R1 (2026-08-11)

**Verdict:** FAIL (P2-01)

| ID | Severity | Disposition |
|----|----------|-------------|
| P2-01 | P2 | **Validated → fixed**: AC14 hermetic now requires ≥1 ndjson record with `project_id=""` and seeded content; zero-record success path removed. |
| P3-01 | P3 | **Process**: governance lag (Implementing vs Planning) — close at ship with Completed + deferred strike. |


---

## Codex R2 (2026-08-11) — after P2-01 fix

**Verdict: PASS WITH DEFERRED P3**

- P2-01 **verified_fixed** (AC14 requires ≥1 ndjson record + empty project_id + seeded content).
- P3-01 process governance lag only — deferred to closeout PR (Completed + deferred strike).
- No P0/P1/P2 open.

Full gate (orchestrator-observed): fmt OK; clippy workspace -D warnings OK; nextest 2572 passed; deny OK; audit OK; ledgerful verify --scope full OK.


---

## Final Codex (closeout) 2026-08-11

**Verdict: PASS** — no findings >P3. Process P3 closed. Soft residuals only.

