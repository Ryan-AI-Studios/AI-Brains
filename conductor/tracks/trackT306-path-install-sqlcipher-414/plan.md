# T306 Plan — PATH install SQLCipher 4.14

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `c62396f6`. Implement **CHORE** on go.

## Phase 0

- [ ] Confirm HEAD lock rusqlite 0.40.2
- [ ] Confirm PATH `ai-brains --version` and current `doctor` `cipher_page` message (no key)
- [ ] CHORE TX; **do not install until go**

## Tasks

- [ ] `cargo install --path crates/ai-brains-cli --locked --features graph`
- [ ] `ai-brains doctor --summary` — `cipher_page` includes `4.14`; `graph_feature` available
- [ ] Conductor T306 Completed + deferred R3 absorbed

## DoD

- [ ] PATH matches source rusqlite 0.40.2 / SQLCipher 4.14.x (AC1–AC3)
- [ ] No key in logs (AC4); no live rebuild/encrypt
