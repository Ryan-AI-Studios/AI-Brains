# T175 Evidence Notes — Sync Protocol Threat Model + ADR (P11.0)

Date: 2026-07-30  
Branch: `feat/t175-sync-threat-model-adr`  
Ledger tx (open, do not commit here): `a87e8852-e8d4-48dd-b937-2e3a016252b2`  
Scope: **design / docs only** — no production sync code.

## Live baseline confirmations

### Migrations end at 0026

Directory: `crates/ai-brains-store/migrations/`

- Highest CE migration: **`0026_content_envelopes_erasure.sql`**
- Prior: `0025_briefings_query_traces.sql` … through `0001_event_log.sql`
- **`0017_sync_state.sql`** exists (Ledgerful bridge KV) — **not** multi-device replication
- T176 replication migration must be **`0027+`** (not 0026)

### Dual CLI “sync” collisions

File: `crates/ai-brains-cli/src/main.rs`

| Surface | Location | Role |
|---------|----------|------|
| `Commands::Sync` | enum variant ~**L264–267**; match arm ~**L2202** | Ledgerful bridge (`SyncCommands`: pull/push/query) |
| `SyncCommands` | ~**L1163+** | Subcommands under `ai-brains sync` |
| `SafetyCommands::Sync` | enum ~**L1217–1224**; match arm ~**L2198** | Hotspot pin into vault (`ai-brains safety sync`) |

**Freeze (ADR-0018 L10):** keep both; multi-device CLI = **`ai-brains device`** + **`ai-brains replicate`**. Crate name `ai-brains-sync` OK.

### Crypto inventory (no T175 dep add)

Workspace already has (among others): `aes-gcm` 0.10, `zeroize`, `subtle`, `rand`, `sha2`, `argon2`.  
**Not yet present:** ed25519-dalek, x25519-dalek, hkdf, openmls, hpke.  
Named candidates only in ADR-0018 §21 — **zero `Cargo.toml` / `Cargo.lock` changes in this track.**

### Capture independence

Unchanged: capture path has no dependency on sync crate (does not exist). ADR L12 + threat-model non-claims reaffirm.

## Deliverable file list

| Path | Role |
|------|------|
| `conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md` | Full threat model §1–7 + authoritative T178 matrix |
| `Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md` | ADR-0018 **Accepted** 2026-07-30 |
| `conductor/tracks/trackT175-sync-threat-model-adr/plan.md` | Phases A–E doc work complete; E4 ledger → orchestrator |
| `conductor/tracks/trackT175-sync-threat-model-adr/evidence/NOTES.md` | This file |
| `conductor/deferred.md` | #34 partial promote; §50 deliverables note |
| `conductor/conductor.md` | T175 → ✅ Completed; T176–T178 design unblocked |
| `conductor/tracks/trackT176-sync-crate-schema/{spec,plan}.md` | Handoff freezes; **Unblocked** (not implementing) |
| `conductor/tracks/trackT177-fake-relay-convergence/{spec,plan}.md` | Handoff freezes; design unblocked (depends T176) |
| `conductor/tracks/trackT178-sync-security-tests/{spec,plan}.md` | Matrix pointer + test id index; design unblocked (depends T176–T177) |

## Explicit non-actions (this track)

- [x] No `crates/ai-brains-sync` created  
- [x] No network / relay production code  
- [x] No Cargo.toml / Cargo.lock edits  
- [x] ADR marked **Accepted** only after Codex R3 PASS (closeout)  
- [x] No git commit / ledger commit by implementer or closeout agent (orchestrator)  

## Residual risks (documented; not closed by T175)

- Metadata leakage to untrusted relay (timing, sizes, graph) — L14 padding only  
- Offline multi-device CE lag; revoked-device past keys  
- Pre-erase backups / exports  
- Classical ECC only / PQ harvest-now (L16 non-claim)  
- No FIPS-validated module → no NIST Purge marketing  
- DataKey rotation **implementation** still open (#34.2)  

## R1 fix summary (2026-07-30) — Internal Review NEEDS FIXES

All R1 findings addressed in docs; ADR Status remains **Proposed**. Disposition log: `conductor/tracks/trackT175-sync-threat-model-adr/review.md` (all `fixed_pending_verification`).

| ID | Fix |
|----|-----|
| **M1** | Enrollment package = `schema_version ‖ DeviceId ‖ Ed25519_pub ‖ X25519_pub`; OOB fingerprint = SHA-256 of full package; X25519 mismatch → no DEK wrap; matrix `T178-L3-enroll-binds-x25519` |
| **M2** | `DeviceEnrolled` / `DeviceRevoked` signed by already-enrolled device; first device RecoveryKit-local only; unknown self-enroll rejected at L8; matrix enroll/revoke signer tests |
| **M3** | ADR §17 freezes HKDF salt empty; exact length-prefixed `info` order; exact AAD bytes; T178 WRAP KATs |
| **L1** | Control-stream cross-ref → ADR §22 Q1 (not §12.1) |
| **L2** | `spec.md` status → Implementing — pending security review |
| **L3** | Re-enroll after revoke: full OOB; prefer new DeviceId; tombstone old |
| **L4** | Spec §12 notes defaults in ADR-0018 §22 pending Accept |

**Files touched:**

- `Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/spec.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/review.md` (created)
- `conductor/tracks/trackT175-sync-threat-model-adr/evidence/NOTES.md` (this section)

**Explicit non-actions:** no Cargo/Rust; no ADR Accept; no conductor Complete; no git/ledger commit.

## Next gate

1. ~~Re-review (R2) of threat-model + ADR-0018 vs R1 fixes → mark `verified_fixed` in `review.md`~~ **done (R2 CLEAN)**  
2. ~~Codex independent audit (Codex R1) → **FAIL** on P1-001 / P2-001~~ — fixes applied; **closed as fixed** by Codex R2 (prior findings closed)  
3. ~~Codex R2 independent re-audit → **FAIL** on P1-002/P1-003 + P2-002/P2-003/P2-004~~ — fixes applied; post-fix CLEAN  
4. ~~Codex R3 independent re-audit~~ → **PASS** (no new findings)  
5. ~~Mark ADR-0018 **Accepted**~~ — **Accepted 2026-07-30**  
6. ~~Unblock T176~~ — T176 **Proposed / Unblocked** (not implementing in T175)  
7. Ledger + git commit — **orchestrator** (tx `a87e8852-e8d4-48dd-b937-2e3a016252b2`)

## Codex R1 fix summary (2026-07-30) — Independent audit FAIL

Dispositions: `conductor/tracks/trackT175-sync-threat-model-adr/review.md` (Codex R1 section). Full audit text: `review.codex.md`. ADR Status remains **Proposed**.

| ID | Fix |
|----|-----|
| **P1-001** | ADR-0018 §5 freezes complete outer `signed_bytes` (BE, 16B UUIDs): schema_version, envelope_id, device_id, local_seq, content_type_code, event_id, content_key_id, ciphertext_len+bytes, wrap_count, wrap records sorted by recipient; wraps = outer signed metadata; control N=0; T178 `T178-L5-sig-canonical-bytes`, `T178-L5-meta-swap-fails`, `T178-L5-wrap-list-tamper`; threat-model L5/§4.2/§4.4 + spec L5 aligned |
| **P2-001** | ADR HPKE ref `§5.2` → `§18`; spec HPKE `§3.5.1` → `§3.5.2`; fold-in `§3.6.8` → §3.6 item 8; `§3.6.6` → §3.6 item 6 / L11; `§5.7` → threat-model §7 |

**Files touched (Codex R1 pass):**

- `Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/spec.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/review.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/evidence/NOTES.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/review.codex.md` (audit body already present; confirmed)
- `conductor/tracks/trackT178-sync-security-tests/spec.md` (L5 test id index aligned)

**Explicit non-actions:** no Cargo/Rust; no ADR Accept; no conductor Complete; no git/ledger commit.

## Codex R2 fix summary (2026-07-30) — Independent re-audit FAIL

Dispositions: `conductor/tracks/trackT175-sync-threat-model-adr/review.md` (Codex R2 section). Full audit text: `review.codex.r2.md` (Verdict **FAIL**; prior P1-001/P2-001 closed). ADR Status remains **Proposed**.

| ID | Fix |
|----|-----|
| **P1-002** | Public signed **cleartext control** freeze (ADR §5.1.1): control types not DEK-encrypted; `wrap_count=0`; prose `payload` for control body; zero `content_key_id` except erasure/ACK target; why-public rationale; data path AEAD+wraps unchanged; tests `T178-L5-control-cleartext-parse`, `T178-L7-ack-cleartext-signed` |
| **P1-003** | Data body packing freeze (ADR §5.3): `ciphertext` = `nonce(12)‖ct‖tag(16)`; `ciphertext_len` full blob; nonce under outer sig; no separate unsigned nonce; local→wire convert; KAT `T178-L5-content-nonce-in-blob` |
| **P2-002** | §17.3 “fails open” → **fail-closed**; L5/L8/spec aligned for AEAD/sig/metadata swap |
| **P2-003** | After `DeviceRevoked`, DeviceId **permanently retired**; re-enroll always new DeviceId + new keys + full OOB; removed same-id reuse language; `T178-L4-deviceid-permanently-retired` |
| **P2-004** | L7 ACK residual: attestation not wipe proof; false ACK from compromised enrolled peer; UX honesty; residual/non-claim matrix rows |

**Files touched (this pass):**

- `Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/spec.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/review.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/evidence/NOTES.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/review.codex.r2.md` (FAIL verdict already present; confirmed)
- `conductor/tracks/trackT178-sync-security-tests/spec.md` (test id index + ACK freeze)

**Explicit non-actions (this fix pass):** no Cargo/Rust; no ADR Accept yet; no conductor Complete yet; no git/ledger commit.

## Codex R3 + closeout (2026-07-30) — PASS → Accept → Complete

**Source:** `review.codex.r3.md` — Verdict **PASS** (no new P0–P3; prior findings closed). Gate cleared for engineering Accept.

| Action | Outcome |
|--------|---------|
| ADR-0018 Status | **Accepted** — 2026-07-30 |
| Review path | Internal R2 CLEAN + Codex R1 FAIL→fix + Codex R2 FAIL→fix + **Codex R3 PASS** |
| T175 conductor | ✅ **Completed** |
| T176 | **Proposed / Unblocked** (not implementing) |
| T177 / T178 | Design unblocked by T175 Complete; still depend on T176 (+ T177 for T178) |
| Deferred #34 | Partial remains correct: (1) ACK design absorbed / impl T176–T178; (2) DataKey rotation **impl residual open** |
| Cargo / production sync | Still zero (docs-only track) |
| Ledger / git | **Orchestrator** — open tx `a87e8852-e8d4-48dd-b937-2e3a016252b2`; closeout agent does not commit |

**Ready for PR** after orchestrator ledger + git commit.

**Closeout files touched:**

- `Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md`
- `conductor/tracks/trackT175-sync-threat-model-adr/{plan,spec,review,threat-model,evidence/NOTES}.md`
- `conductor/conductor.md`
- `conductor/deferred.md`
- `conductor/tracks/trackT176-sync-crate-schema/{spec,plan}.md`
- `conductor/tracks/trackT177-fake-relay-convergence/{spec,plan}.md`
- `conductor/tracks/trackT178-sync-security-tests/{spec,plan}.md`
