# T221 — Governed first-run + deny exit honesty

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Audit — source/evidence/review exit 3 POLICY_DENIED; progressive `denied:true` **exit 0** empty; no grants until bootstrap
- **Scores:** usefulness **4–5** · quality **5–6** (honest walls, dead product)
- **Category:** UX / CONTRACT
- **Depends on:** T210 bootstrap; T203 lists; T202 progressive

## Objective

Make governed discovery **reachable in one step** for local operators and align exit codes so agents don’t treat denied progressive as “no knowledge.”

## Draft decisions

| F1 | Progressive/query when policy-denied → exit **3** (or **6** per CLI-EXIT-CODES) + same bootstrap hint as lists — **not exit 0** |
| F2 | Optional `doctor` soft check `policy_grants` warn when discovery empty |
| F3 | First-run: `policy bootstrap` one-liner in empty list human output (already JSON hint) |
| F4 | Soft: interactive prompt once — **decline** unless product asks (non-interactive default) |
| F5 | Hermetic: progressive denied → non-zero |

## Non-goals

Auto-grant on init without user action; admin/revoke full surface (T210 F24).
