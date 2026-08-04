# T207 Review Log — Recall empty pretty + scope honesty

## Scope
- Branch: `feat/T207-recall-empty-pretty-scope`
- Ledger TX: `8528293a-f249-45da-a48c-5370529f5722` (FEATURE)
- Primary: empty pretty always-on hint (F3); Scope line empty-only (F4); omit generated Session (F5); `get_project_by_id` (F32); extract printer (F31); project_scoped hint (F6/F33)

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| R1 | Internal subagent (read-only) | **PASS** (P3 only) | All AC1–9/11–12 met; AC10 correctly deferred |
| R1b | Orchestrator polish | P3 fixed | name-only Scope unit; drop no-op TTY test; CAPABILITIES Session wording |
| R2 | Cross-model Claude (Codex rate-limited) | **PASS** | Zero findings; full F1–F33 / AC1–12 matrix met; AC10 correctly residual |

## Findings disposition

| ID | Sev | Description | Status |
|----|-----|-------------|--------|
| R1-P3-1 | P3 | Missing unit for empty-alias → name Scope label | **verified_fixed** — `format_scope_line__project_empty_alias__uses_name` |
| R1-P3-2 | P3 | No-op TTY compile unit | **verified_fixed** — removed; hermetic owns F3 |
| R1-P3-3 | P3 | Soft skill/OPERATIONS one-liner | **deferred** — soft; CAPABILITIES+CHANGELOG meet hard AC7/AC8 |
| R1-P3-4 | P3 | CAPABILITIES “resolved” vs env session | **verified_fixed** — wording: user `--session` / prefix / last only |
| R2-reg-1 | P2 | `sync_query_isolation` naive `!contains(query)` broke when empty pretty always quotes query in hint | **verified_fixed** — assert no *hit* leak; empty hint query echo OK |

## Gate evidence (orchestrator)

```
cargo nextest -p ai-brains-cli (recall_empty|recall_nonempty|format_scope|format_pretty|build_recall_hint): 21 passed
cargo nextest -p ai-brains-store (get_project_by_id): 3 passed
```

## AC matrix (current)

| AC | Status |
|----|--------|
| AC1–AC9, AC11–AC12 | Met |
| AC10 | Deferred residual (non-empty Scope) — not DoD |

## Cross-model
Pending first Codex (or Claude fallback) audit after this log.
