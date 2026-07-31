# T178 Plan — Placeholder

Status: **Proposed / Unblocked (design)**. ADR-0018 **Accepted** (T175 Complete). Expand with T177 harness; add security cases as properties land.

## Handoff freezes (from T175 / ADR-0018 Accepted)

- Authoritative matrix: threat-model §7 → T178 test ids
- Must: opacity, tamper, metadata-swap, replay, unknown device pre-verify, revoke, forged ACK, gap, pad buckets
- Non-claims honesty tests / docs
- Local-only default + capture independence

## When expanding

1. Ciphertext opacity + tamper + metadata-swap first.
2. Replay idempotency + enrolled-set pre-verify.
3. Revocation + erasure ACK (signed / forged).
4. Gap + padding fixtures.
5. Write residual metadata section.

## License gate

- [ ] No AGPL security tooling required
- [ ] cargo deny + audit green

## Checklist (empty until ready)

- [x] ADR-0018 Accepted (T175 Complete 2026-07-30)
- [ ] T176–T177 harness ready
- [ ] Claim→test matrix frozen from threat-model §7
- [ ] RED security tests named
- [ ] Residual risk doc outline frozen
