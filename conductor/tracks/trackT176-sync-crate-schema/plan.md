# T176 Plan — Placeholder

Status: **Proposed / Unblocked**. ADR-0018 **Accepted** 2026-07-30 (T175 Complete). Not implementing yet. Migration id: **0027+** (not 0026).

## Handoff freezes (from T175 / ADR-0018 Accepted)

- CLI: **`device`** + **`replicate`** (keep Ledgerful `sync` + `safety sync`)
- Wrap table: `(content_key_id, recipient_device_id)`
- Deps: ed25519-dalek 3.x, x25519-dalek 3.x, curve25519-dalek 5.x transitive, hkdf 0.13; features zeroize+serde+rand_core
- Control envelopes on same stream; whole-vault default; gap fail-closed
- See ADR-0018 + threat-model §7 for full locks

## When expanding (design gate cleared)

1. Scaffold crate with types only.
2. Migration `0027+` + projection stubs.
3. Wire crypto ports (sign + per-recipient wrap).
4. No sockets.

## License gate

- [ ] Deny dry-run for any new signature/KEX crate
- [ ] Optional feature keeps default build light
- [ ] Inventory curve25519-dalek 5.x transitive

## Checklist (empty until ready)

- [x] ADR-0018 Accepted (2026-07-30; T175 Complete / Codex R3 PASS)
- [ ] Schema frozen from ADR
- [ ] Migration id free (0027+)
- [ ] CLI naming disambiguation noted (`device`/`replicate`)
- [ ] RED tests named
