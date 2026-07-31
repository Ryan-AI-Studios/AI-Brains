# T175 Independent Completion Audit (Codex R1)

**Path:** `conductor/tracks/trackT175-sync-threat-model-adr/review.codex.md`  
**Date:** 2026-07-30  
**Auditor role:** independent completion / security design audit (Codex)

> Note: original auditor note reported a read-only write failure; this file is the canonical saved FAIL audit body for the track.

## Verdict

**FAIL**

The design is substantially complete and R2 fixes are present, but two issues must be resolved before ADR-0018 can be Accepted.

## P0

None.

## P1

### P1-001 — Outer envelope signature is not byte-level or exhaustive

ADR-0018 §5 specifies that the signature covers “at least”:

`schema_version, envelope_id, device_id, local_seq, content_type_code, ciphertext`

but does not define:

- canonical byte encoding or field ordering;
- whether `event_id` is signed metadata or authenticated inside ciphertext;
- binding of `content_key_id`, recipient identity, wrap selector, nonce, ephemeral public key, or other routing fields;
- an outer-envelope signature KAT.

This conflicts with the L5 claim that all routing metadata is authenticated and metadata swaps are forbidden. A T176 implementation could satisfy the literal minimum while leaving security-relevant metadata mutable.

Required fix: freeze the complete canonical signature input, including encoding, UUID representation, and inner-versus-outer authenticated fields; add corresponding T178 KAT and tamper cases.

Evidence: [ADR-0018 §5](C:/dev/AI-Brains-wt-t175/Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md:118), [threat-model L5 matrix](C:/dev/AI-Brains-wt-t175/conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md:215).

## P2

### P2-001 — Broken section references weaken traceability

Several references do not resolve:

- ADR-0018 line 36 references HPKE at `§5.2`; the section is §18.
- `spec.md` line 125 references HPKE at `§3.5.1`; HPKE is §3.5.2.
- `spec.md` lines 425 and 432 reference nonexistent subsections `§3.6.8` and `§3.6.6`.
- `spec.md` line 433 references nonexistent `§5.7`; the matrix is §7.

These are easy documentation fixes and should not be deferred to `deferred.md`.

## P3

None. No P3 deferral is proposed.

## Requirement and DoD audit

- Threat model: present with assets, actors, DFD, STRIDE analysis, residuals, non-claims, and authoritative L1–L16 traceability matrix.
- ADR-0018: L1–L16, dual-key enrollment, enrolled-signer rules, revocation, per-recipient X25519/HKDF/AES-GCM wrapping, HPKE deferral, CLI collision handling, migration `0027+`, CE honesty, and PQ non-claims are present.
- Wrap encoding: salt, HKDF info ordering, length prefixes, AAD, nonce requirements, and T178 KAT ID are specified.
- Deferred #34: correctly split; ACK design is absorbed, DataKey rotation remains an implementation residual, and the historical item remains historical.
- T176–T178 handoffs: correctly blocked on ADR acceptance and contain the relevant freezes.
- Production sync code: none found; no `ai-brains-sync` crate and no Cargo changes.
- Capture independence: explicitly preserved.
- ADR status, conductor status, and T176 blocking state are correctly still pending.
- Ledger provenance: not closed; evidence records an open transaction and no commit yet.

## Verification

- `git diff --check`: passed.
- No sync crate or Cargo changes: confirmed.
- `ai-brains preflight --summary`: passed.
- `ledgerful doctor`, scan, and verify were environment-blocked by the read-only Ledgerful database/report path.
- Direct workspace `cargo fmt --check` exposed broad pre-existing newline/formatting drift unrelated to T175; no T175 code changes were implicated.

After fixing P1-001 and P2-001, rerun the independent review, then Accept ADR-0018, mark T175 Complete, unblock T176, and commit the documented ledger provenance.