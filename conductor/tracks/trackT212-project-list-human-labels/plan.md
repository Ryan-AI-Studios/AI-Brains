# T212 Plan — Project list human labels

Status: **Completed** (PR #95 `09e34ba`). Spec: [spec.md](./spec.md).

## Phases

### Phase 0 — Plan

- [x] Live re-scan + research
- [x] Spec F1–F42 + AC1–AC12
- [x] AI fold-in §14 (M1–M5 accept; L2/L3/L5 elevate)
- [x] User **go** → ledger start (d43fe0eb)

### Phase 1 — Red (TDD)

- [x] Unit: `display_label` order (alias / `(no alias) — x` → `(no alias)` / Project uuid / human name)
- [x] Unit: char-safe truncate no panic on multibyte at width (AC11)
- [x] Unit: relative last_activity (&lt;365d vs date)
- [x] Hermetic: AC1–AC6 scaffold
- [x] Note smoke friendly-name + empty list regressions

### Phase 2 — Green

- [x] Store: `list_projects_detail` — MAX(mp.updated_at), path **scalar subquery** ORDER BY path LIMIT 1
- [x] Store: `list_projects` add `, project_id ASC` tie-break (F41)
- [x] Keep 4-tuple signature
- [x] CLI: rewrite `list` human table; footer **stderr**; F16 `*`
- [x] clap: `--format human|json` only
- [x] Soft F26 git suggestion if free
- [ ] Soft F24 verbose if free (deferred — not free)

### Phase 3 — Docs

- [x] CAPABILITIES: columns, stderr footer, json, last_activity semantic, path honesty
- [x] CHANGELOG minor

### Phase 4 — Review + gate

- [x] Internal review CLEAN (lows fixed: exact unaliased_count; smoke message)
- [x] Manual live vault (label-first, `*` active, footer stderr, JSON shape)
- [x] Full local gate: fmt + clippy -D warnings + nextest 2127 pass + deny + audit
- [x] Claude cross-model **PASS** (Codex rate-limited)
- [x] PR #95 + CI Win/Linux/macOS green + squash-merge `09e34ba`

## Absorbed AI fold-in

| Item | Handling |
|------|----------|
| Label-first / JSON / footer | F4–F9 |
| M1 byte-slice panic | F36 + AC11 |
| M2 `(no alias)` strip | F4 |
| M3 footer stderr | F8 |
| M4 activity semantic | F7 docs |
| M5 path subquery | F6 |
| L2 list_projects sort | F41 |
| L3 no dual --json | F9 |
| L5 active * | F16 DoD |

## Structural SQL sketch (M5)

```sql
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
FROM project_projection p
LEFT JOIN project_alias_projection a ON p.project_id = a.project_id
LEFT JOIN (
  SELECT project_id, COUNT(*) AS memory_count, MAX(updated_at) AS last_activity
  FROM memory_projection
  GROUP BY project_id
) mem ON p.project_id = mem.project_id
ORDER BY memory_count DESC, p.project_id ASC;
```

## Touch map

| File | Change |
|------|--------|
| `store/src/lib.rs` | QueryStore detail method |
| `store/src/query_store.rs` | detail SQL + list_projects ORDER BY |
| `cli/.../project.rs` | list UI + helpers + footer stderr |
| `cli/src/main.rs` | `--format` on List |
| `cli/tests/project_list_labels.rs` | hermetic AC |
| smoke / empty_states | regression guards |
| CAPABILITIES, CHANGELOG | honesty |

## Ledger (on go)

```powershell
ledgerful ledger start T212-project-list-human-labels --category FEATURE --message "Project list label-first; last_activity; path subquery; set-alias footer stderr; --format json; char-safe truncate"
```

## DoD checklist

- [x] AC1–AC9 + AC11 met (AC10/AC12 soft — AC12 covered; AC10 path soft deferred)
- [x] list_projects signature preserved + ORDER BY tie-break
- [x] No production panic on multibyte truncate
- [x] CAPABILITIES + CHANGELOG
- [x] Review clean for >low (internal CLEAN + Claude PASS)
- [x] Full CI on PR green; conductor Completed after merge

## Explicit non-work

- Auto-alias / name migration
- Interactive wizard
- Dual `--json` flag
- detect --json
- T213–T216 scopes

## Residual after ship

- Soft path empty until path aliases registered
- Soft F24/F26 if not free
- detect --json still T206 soft
