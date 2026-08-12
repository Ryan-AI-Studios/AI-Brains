# T243 Review Log — Search surface unify

**Track:** T243-SearchSurfaceUnify  
**Category:** FEATURE / UX / CONTRACT  
**Review requirement:** Hard cross-model (F25) — alias + contracts `next_step`

---

## Findings

| ID | Severity | Status | Summary | Notes |
|----|----------|--------|---------|-------|
| R1-S1 | medium | verified_fixed | AC1 hermetic `alias OR vault-first` would pass if `search` routed to sync/progressive | Tightened: alias text + Usage recall + vault-first + `--no-bridge`; parent `--help` locks `[aliases: search]` |
| R1-S2 | medium | verified_fixed | AC2 accepted `results\|\|hits` and did not distinguish progressive | Locks live `RecallResponse.results`; forbids `hits` and progressive fields |
| R1-S3 | medium | verified_fixed | Deny hermetic did not lock `next_step` omit | `v.get("next_step").is_none()` |
| R1-N1 | low | verified_fixed | Soft F22 skill one-liner | `.claude/skills/ai-brains/SKILL.md` |
| R1-N2 | low | deferred | F23 non-empty recall footer; F24 daemon `next_step` | Spec residuals |
| CX1-P1 | high (process) | out_of_scope | Track not yet marked Completed / gate / CX pending at review time | Expected mid-loop; cleared by dogfood + full gate + final CX |
| CX1-P2 | medium | fixed_pending_verification | F3 `Some(other) => other` had no unit | `resolve_format__explicit_unknown__passthrough` |

### Status legend

- `open` — not yet fixed  
- `fixed_pending_verification` — implementer claims fix; needs re-verify  
- `verified_fixed` — reviewer confirmed  
- `deferred` — medium/low only; justification + ISSUES.md  

---

## Review passes

| Pass | Reviewer | Date | Result |
|------|----------|------|--------|
| Internal R1 completeness | explore subagent | 2026-08-12 | NEEDS_FIX (3 medium test tightness) |
| Internal R1 correctness | explore subagent | 2026-08-12 | NEEDS_FIX (same AC1/AC2/deny hermetic) |
| Internal R1 fix | orchestrator | 2026-08-12 | Tests 15/15; skill F22 shipped |
| Internal R2 | explore subagent | 2026-08-12 | CLEAN |
| Cross-model CX1 | Codex gpt-5.6-luna high | 2026-08-12 | FAIL (P1 process/evidence; P2 F3 pass-through unit) |

---

## Notes

- Spec AC2 said `api_version` + `hits`. Live `RecallResponse` is `results` with no `api_version`. Tests lock live shape; do not invent fields.
- clap 4.6 `search --help` shows `Alias: \`search\`` and `Usage: … recall`. Parent `--help` lists `[aliases: search]`.
