# T175 Plan — Sync Protocol Threat Model + ADR (P11.0)

Status: **Completed** (2026-07-30). Design-only; ADR-0018 **Accepted** after Codex R3 PASS. **Unblocks T176–T178** (implementation still not started).

Category for ledger: `SECURITY` / `ARCHITECTURE`.

## Preconditions

- [x] Re-read ADR-0015 (stub), ADR-0016 (CE), vision §12, master plan Task 11.0–11.3
- [x] Re-read T176–T178 placeholders for handoff fields
- [x] Confirm migrations end at `0026_content_envelopes_erasure.sql` (replication ≠ 0026)
- [x] Inventory **both** CLI collisions: `Commands::Sync` **and** `SafetyCommands::Sync`
- [ ] `ledgerful doctor` healthy before doc commits (orchestrator / commit gate)

## Phase A — Inventory & research freeze

- [x] A1 Confirm live crypto deps (`aes-gcm` 0.10, zeroize, subtle, sha2) — no ed25519/x25519/hkdf/hpke yet
- [x] A2 Candidate inventory **including transitives**: ed25519-dalek 3.x, x25519-dalek 3.x, **curve25519-dalek 5.x**, hkdf 0.13; feature flags (`serde`, `zeroize` mandatory, `rand_core`); openmls deferred; **hpke 0.14 considered-deferred**
- [x] A3 Non-goals: no CRDT default, no SQLCipher file sync, no AGPL Matrix, no MLS v1, **no multi-user vault**, no PQ claim
- [x] A4 Map deferred #34: ACK → design here + implement T176–T178; DataKey rotation → **direction only** (impl residual stays open)
- [x] A5 Freeze wrap construction prose for ADR: **per-recipient X25519+HKDF+AES-GCM** (not epoch KEK primary)

## Phase B — Threat model document

- [x] B1 Assets + actors + trust-boundary DFD (text)
- [x] B2 STRIDE tables (device↔vault, device↔relay, peer-via-relay) incl. enrolled-set DoS + metadata-swap
- [x] B3 Residuals: metadata; offline CE lag; revoked-past-keys; pre-erase backups; **classical ECC / PQ harvest-now**; no FIPS
- [x] B4 Non-claims: metadata-private; perfect multi-device deletion; NIST Purge/remote wipe; certs; **PQ resistance**; multi-user vault
- [x] B5 Draft claim → T178 map (feeds C9; C9 authoritative)
- [x] B6 Write `threat-model.md` under this track dir

## Phase C — ADR-0018 draft

- [x] C1 Create `Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md` (Proposed)
- [x] C2 Freeze **L1–L16** from spec
- [x] C3 Naming: dual collision (`sync` + `safety sync`); multi-device CLI = **`device`** + **`replicate`**
- [x] C4 Migration: T176 **`0027+`** (not 0026)
- [x] C5 DataKey rotation: **direction only**; implementation deferred P11 hygiene
- [x] C6 Selective sync default: whole-vault (or explicit defer of filters)
- [x] C7 Fake-relay-first (T177 before network)
- [x] C8 Cross-links ADR-0015/0016; wrap construction §3.5.1; HPKE deferral
- [x] C9 **Traceability matrix:** each L1–L16 + residual + non-claim → T178 test id or explicit defer (ADR annex or threat-model §7; merges B5)
- [x] C10 Signature scope: canonical signed fields include metadata + ciphertext
- [x] C11 Sequence gap (L13) + padding buckets (L14) normative text
- [x] C12 Single-owner (L15) + PQ non-claim (L16)

## Phase D — Security review gate

- [x] D1 Internal review (findings → ADR/threat-model amendments) — R1 NEEDS FIXES → R2 CLEAN; Codex R1 FAIL→fix; Codex R2 FAIL→fix
- [x] D2 Resolve remaining open questions §12 or Accepted-with-explicit-defer — defaults in ADR-0018 §22 now normative on Accept
- [x] D3 Mark ADR-0018 **Accepted** only after sign-off — **Accepted 2026-07-30** after **Codex R3 PASS**
- [ ] D4 Optional pin: `ai-brains pin "DECISION: ADR-0018 …"` (optional; orchestrator/session)
- [x] D5 **Stop-before T176:** no schema/crate until Accepted — gate cleared; T176 may start implementation on go-ahead

**Note:** Phase D complete for doc Accept path. ADR-0018 **Accepted** 2026-07-30 after internal R2 CLEAN + Codex R1 FAIL→fix + Codex R2 FAIL→fix + **Codex R3 PASS**.

## Phase E — Handoff & closeout

- [x] E1 Pointer notes on T176–T178 when unblocked (placeholders updated with freezes; ADR Accepted / design unblocked)
- [x] E2 Deferred #34 **partial only:**
  - Strike / promote sub-item **(1)** multi-device ACK — design T175, implement T176–T178  
  - Mark sub-item **(2)** DataKey rotation — direction frozen in ADR-0018; **implementation residual remains open**  
  - Leave sub-item **(3)** historical  
  - **Do not strike #34 wholesale**
- [x] E3 Conductor: T175 Completed + evidence (ADR-0018 Accepted, threat-model, Codex R3 PASS, zero crates)
- [ ] E4 Ledger commit for docs when authorized — **orchestrator** (do not commit in closeout agent)
- [x] E5 No `Cargo.toml` / network / sync crate changes in this track

## Proof / verification (docs track)

| Check | How |
|-------|-----|
| Completeness | Spec §11 DoD |
| Consistency | ADR L-locks match threat model + C9 matrix |
| Wrap freeze | Per-recipient construction present; epoch not primary |
| Naming | Both CLI sync surfaces + device/replicate |
| Dep inventory | curve25519-dalek + HPKE evaluation present |
| No code drift | No production sync Rust |
| Capture independence | Explicit non-dependency |

## Risks

| Risk | Mitigation |
|------|------------|
| Scope creep into T176 | Stop-before; ADR-only |
| Metadata over-claim | Non-claims + L14 honesty |
| LWW via “auto merge” | Explicit forbid |
| Migration 0026 collision | Hard 0027+ |
| CLI collision miss (`safety sync`) | L10 dual inventory |
| Epoch KEK complexity | Rejected as v1 primary |
| Hand-composed wrap bugs | Frozen labels + HPKE hygiene path |
| Multi-user trust creep | L15 single-owner fence |

## Out of scope

Network relay, production enrollment UX, OpenMLS, HPKE crate add (this track), changing CE local wipe, desktop, connector cursors, ChangeGuard renames, DataKey rotation **implementation**.

## Success criteria

1. Threat model reviewed.  
2. ADR-0018 **Accepted** with L1–L16.  
3. T176 unblocked.  
4. Residuals (metadata, multi-device CE lag, PQ) documented.  
5. Zero new runtime dependencies in T175.  
