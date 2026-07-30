# T177 Plan — Placeholder

Status: **Proposed / Unblocked (design)**. ADR-0018 **Accepted** (T175 Complete). Requires T176 envelope/cursor types before implement.

## Handoff freezes (from T175 / ADR-0018 Accepted)

- Fake-relay-first before network
- Topological apply; never LWW
- Gap buffer / fail-closed skip
- Idempotent event/envelope ids
- Engine ops align with **`replicate`** (not Ledgerful `sync`)

## When expanding

1. RED: two vaults + memory relay diverge then converge.
2. Add reorder + duplicate + gap cases.
3. File relay optional for multi-process.
4. No public bind.

## License gate

- [ ] No new deps OR deny-green only
- [ ] No cloud SDKs

## Checklist (empty until ready)

- [x] ADR-0018 Accepted (T175 Complete 2026-07-30)
- [ ] T176 types ready
- [ ] Convergence oracle frozen
- [ ] Scenario matrix listed
- [ ] RED tests named
- [ ] GREEN scope frozen
