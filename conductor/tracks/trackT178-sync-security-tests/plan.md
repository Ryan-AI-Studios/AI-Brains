# T178 Plan — Sync Security Tests + Acceptance Gates

Status: **In Progress** (2026-07-31) — harness + Must suite implemented; pending review/PR.
Depends: T176 **Completed**, T177 **Completed**, ADR-0018 **Accepted**.  
Category: **SECURITY**.

## Preflight (before first code edit)

- [x] `ledgerful doctor` (orchestrator pre-started TX)
- [x] `ledgerful ledger status --compact` (TX open)
- [x] `ledgerful scan --impact` (touched sync/store/OPERATIONS)
- [x] Read threat-model §7 + ADR-0018 + spec F1–F28
- [x] Confirm T177 harness live; extracted to common::twin_vaults
- [x] Optional evidence: capture Cargo.toml gate (F21)
- [x] **No** new production deps; no proptest

## Dependency research pin (2026-07-31)

| Crate | Pin | Action |
|-------|-----|--------|
| ed25519-dalek | 3.0.0 | keep |
| x25519-dalek | 3.0.0 | keep |
| hkdf | 0.13.0 | keep (stable 2026-03-30; OKM pin extendable) |
| aes-gcm | 0.10.3 | **hold** (0.11 exists — out of scope) |
| proptest | 1.11.0 optional dev | only if justified; **cases ≤100** |

## Phase A — Harness extract + scaffolding + RED shells

- [x] **A0** **F23:** Extract `crates/ai-brains-store/tests/common/` (`TestVault`, `TwinVaults`, `assert_converged`, helpers) from `replication_converge.rs`; both converge + security import `mod common`
- [x] **A1** Freeze Must-id checklist (this plan master list)
- [x] **A2** Create `replication_security.rs` importing common
- [x] **A3** Implement `capture_security_snapshot` + **`assert_rejected_no_side_effect` (F19)** in common
- [x] **A4** Optional `triple_enrolled()` for 3-device residual paths (**F23**)
- [x] **A5** KAT dir `crates/ai-brains-sync/tests/kats/` + `include_str!` loaders (**F20**)
- [x] **A6** RED tests for Codex R2 ids (**F28** tags on each):
  - `T178-WRAP-kat-info-aad-bytes`
  - `T178-L5-content-nonce-in-blob`
  - `T178-L5-control-cleartext-parse`
  - `T178-L7-ack-cleartext-signed`
  - `T178-L7-forged-ack-reject` (both F24 layers)
  - `T178-L4-deviceid-permanently-retired`
  - `T178-R-ack-attestation-not-wipe` / `T178-NC-ack-not-wipe-proof`
- [x] **A7** RED shells for **F22** replay A/B/C
- [x] **A8** RED shell **F21** L12 capture isolation
- [x] **A9** Index: `T178-*` id → `test_fn` (incl. smuggled-membership alias)

## Phase B — Crypto KATs + fail-closed unit (`ai-brains-sync`)

- [x] **B0** **Seed export prerequisite (AI2 #1 / F20):** make `wrap_with_eph` `pub(crate)` **or** add `#[cfg(test)]` / `#[doc(hidden)]` `wrap_content_dek_for_recipient_with_seed(eph, nonce, …)` — **not** a public consumer API break
- [x] **B1** `T178-L5-sig-canonical-bytes` — thin `t178_l5_sig_canonical_bytes__kat` over existing fixture (**F7**)
- [x] **B2** `T178-L5-content-nonce-in-blob`
- [x] **B3** `T178-L5-control-cleartext-parse`
- [x] **B4** `T178-L5-meta-swap-fails` (+ F19 on apply path)
- [x] **B5** `T178-L5-wrap-list-tamper` (+ F19)
- [x] **B6** `T178-L5-tamper-ct` (+ F19)
- [x] **B7** `T178-WRAP-kat-info-aad-bytes` — **static** info/aad/okm only
- [x] **B8** `T178-WRAP-kat-seeded-ciphertext` — **Must** after B0; pinned wrap_ct golden
- [x] **B9** `T178-WRAP-per-recipient-roundtrip` + `T178-WRAP-wrong-recipient-fails`
- [x] **B10** `T178-WRAP-no-shared-datakey-over-relay` — structural field/encode assert
- [x] **B11** Should: `T178-WRAP-nonce-uniqueness` (N wraps, distinct nonces)
- [x] **B12** `T178-L14-pad-buckets`
- [x] **B13** `cargo nextest run -p ai-brains-sync` green

## Phase C — Engine / TwinVaults adversarial integration

- [x] **C1** `T178-L8-unknown-device-preverify` — `NotEnrolled` (not `SignatureInvalid`) + F19; document code-path no-verify (**F8**)
- [x] **C2** `T178-L8-aead-fail-closed` (+ F19)
- [x] **C3** `T178-L8-smuggled-membership-reject` — elevate T177 smuggled DeviceRevoked-in-DataEvent
- [x] **C4** `T178-L4-post-revoke-reject` + `T178-L4-revoke-no-future-wrap`
- [x] **C5** `T178-L4-revoke-signer-must-be-enrolled` + `T178-L4-deviceid-permanently-retired`
- [x] **C6** `T178-L3-enroll-fingerprint` + binds-x25519 + enroll-signer-must-be-enrolled
- [x] **C7** **F22 three-vector replay** (exact / modified-seq / post-revoke)
- [x] **C8** `T178-L7-ack-signed` + cleartext-signed + **ack-states** (incl. **status normalization pin** AI2 H)
- [x] **C9** `T178-L7-forged-ack-reject` **F24**:
  - bad random sig → SignatureInvalid; eraser pending
  - wrong enrolled signer spoofed peer_device_id → binding reject; eraser pending
- [x] **C10** `T178-L9-relay-no-decrypt` (no plaintext substring / no clear DEK-DataKey)
- [x] **C11** `T178-L9-relay-no-forge` via **F25** test-local body byte flip + F19 (parse-without-verify allowed)
- [x] **C12** `T178-L1-relay-opaque` + `T178-L2-device-pub-only-relay`
- [x] **C13** `T178-L13-gap-buffer` + gap-no-corrupt-apply
- [x] **C14** `T178-L6-no-lww-conflict`
- [x] **C15** `T178-L11-partial-ce-ux` + `T178-R-offline-ce-pending-ack`
- [x] **C16** `T178-R-revoke-past-still-open` — document 2-vault vs `triple_enrolled` approach
- [x] **C17** Optional proptest bit-flip: `ProptestConfig::with_cases(100)` max; optional feature-gate

## Phase D — Optionality, capture independence, honesty docs

- [x] **D1** `T178-L1-local-only-default`
- [x] **D2** `T178-L12-capture-without-sync` — **F21** programmatic gate primary
- [x] **D3** **F26:** Write **`Docs/OPERATIONS.md`** section **“Multi-device sync residuals”**
  - metadata leakage (sizes, counts, timing, device graph)
  - ACK attestation residual (not wipe proof)
  - offline CE lag
  - classical / not PQ; not NIST Purge multi-device
  - pad ≠ metadata-private
  - #34.2 still open; per-seal wraps improve multi-device nonce budget but do not close DataKey rotation
- [x] **D4** **F27** `doc_claims_honesty` scanner (`include_str!`): required phrases present; forbidden claims absent
- [x] **D5** Doc/id asserts: R-metadata, R-ack-attestation, NC-*
- [x] **D6** Explicit defer rows: L10, L15, PIN if absent, #34.2, HPKE
- [x] **D7** **F28** grep checklist: all 7 Codex R2 ids tagged

## Phase E — Gates, review, closeout

- [x] **E1** Targeted nextest sync + store (incl. replication_security)
- [x] **E2** clippy `-D warnings` on touched crates
- [ ] **E3** Full gate: fmt, clippy workspace, nextest workspace, deny, audit
- [ ] **E4** Manual evidence: Must-id matrix; forged-ACK both layers; L12; OPERATIONS residual
- [ ] **E5** Review log; **cross-model** (SECURITY) until clean
- [ ] **E6** conductor T178 → Completed; Phase 11 rollup
- [ ] **E7** deferred: #54 close notes; reaffirm #34.2 open
- [ ] **E8** Optional pin: `ai-brains pin "DECISION: T178 security suite …"`

## Master Must-id checklist (track bar)

### Crypto / L5 / WRAP
- [x] `T178-L5-sig-canonical-bytes` (thin wrapper)
- [x] `T178-L5-meta-swap-fails` (+ F19)
- [x] `T178-L5-wrap-list-tamper` (+ F19)
- [x] `T178-L5-content-nonce-in-blob`
- [x] `T178-L5-control-cleartext-parse`
- [x] `T178-L5-tamper-ct` (+ F19)
- [x] `T178-L5-replay-idempotent` (umbrella via F22)
- [x] `T178-L5-replay-exact-duplicate`
- [x] `T178-L5-replay-modified-seq` (+ F19)
- [x] `T178-WRAP-per-recipient-roundtrip`
- [x] `T178-WRAP-wrong-recipient-fails`
- [x] `T178-WRAP-kat-info-aad-bytes` (static)
- [x] `T178-WRAP-kat-seeded-ciphertext` (after B0)
- [x] `T178-WRAP-no-shared-datakey-over-relay` (structural)
- [x] `T178-WRAP-nonce-uniqueness` (Should)

### Membership / L3–L4 / L8
- [x] `T178-L3-enroll-fingerprint`
- [x] `T178-L3-enroll-binds-x25519`
- [x] `T178-L3-enroll-signer-must-be-enrolled`
- [x] `T178-L4-revoke-no-future-wrap`
- [x] `T178-L4-post-revoke-reject`
- [x] `T178-L4-revoke-signer-must-be-enrolled`
- [x] `T178-L4-deviceid-permanently-retired`
- [x] `T178-L8-unknown-device-preverify` (+ F19; F8)
- [x] `T178-L8-aead-fail-closed` (+ F19)
- [x] `T178-L8-replay-revoked-device` (F22-C)
- [x] `T178-L8-smuggled-membership-reject`

### ACK / L7 / residual
- [x] `T178-L7-ack-signed`
- [x] `T178-L7-ack-cleartext-signed`
- [x] `T178-L7-forged-ack-reject` (F24 both layers + F19)
- [x] `T178-L7-ack-states` (incl. status normalization pin)
- [x] `T178-R-ack-attestation-not-wipe`
- [x] `T178-R-offline-ce-pending-ack`
- [x] `T178-NC-ack-not-wipe-proof`

### Relay / L1–L2 / L9 / L13–L14
- [x] `T178-L1-local-only-default`
- [x] `T178-L1-relay-opaque`
- [x] `T178-L2-device-pub-only-relay`
- [x] `T178-L9-relay-no-decrypt`
- [x] `T178-L9-relay-no-forge` (F25)
- [x] `T178-L13-gap-buffer`
- [x] `T178-L13-gap-no-corrupt-apply`
- [x] `T178-L14-pad-buckets`
- [x] `T178-L14-pad-not-metadata-private`

### Convergence / optionality / honesty
- [x] `T178-L6-no-lww-conflict`
- [x] `T178-L11-partial-ce-ux`
- [x] `T178-L12-capture-without-sync` (F21)
- [x] `T178-R-metadata-doc` (F26/F27)
- [x] `T178-R-revoke-past-still-open`
- [x] `T178-NC-metadata`
- [x] `T178-NC-partial-erase`
- [x] `T178-NC-no-purge-claim`
- [x] `T178-NC-no-pq-claim`

### Explicit defers
- [x] L10 CLI naming
- [x] L15 multi-user
- [x] L16 / PQ implementation
- [x] #34.2 DataKey rotation
- [x] HPKE / MLS
- [x] `T178-L3-reject-unbound-pin` if no PIN API
- [x] CAVP/FIPS certification
- [x] Pre-erase physical backups

## Implementation notes

1. **A0 before A2** — extract common; avoid duplicating TwinVaults.
2. **B0 before B8** — seed export required for full wrap_ct KAT (ADR §17.4).
3. **F19** on every negative apply; security-relevant snapshot only.
4. **F24** two forged-ACK cases explicit.
5. **F25** mutate blob test-locally; do not block on AdversarialRelay mutate API.
6. **F26** residual section lives only in `Docs/OPERATIONS.md`.
7. **F27** honesty scanner is CI, not manual-only.
8. **F28** greppable `// T178-…` or test names for 7 Codex R2 ids.
9. **proptest:** max 100 cases; optional feature-gate.
10. **PowerShell:** `;` not `&&`.
11. **Ledger:** category SECURITY after go-ahead.

## Stop-before

- Must id needs production network or large schema expansion
- aes-gcm 0.11 upgrade appears necessary
- #34.2 requested in-scope
- Ambiguous threat-model vs live API conflict

## Definition of Done (plan)

All Must-id boxes green or deferred with residual; F19–F28 honored; deny/audit green; SECURITY cross-model clean; conductor + deferred updated; Phase 11 fake-relay security acceptance claimable.
