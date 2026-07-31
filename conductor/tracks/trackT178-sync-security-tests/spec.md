# T178 — Sync Security Tests + Acceptance Gates (P11.3)

- **Track ID:** T178-SyncSecurityTests
- **Phase:** P11 Task 11.3
- **Status:** 📋 **Proposed / Expanded** — ready to implement after human go-ahead
- **Depends on:** T176 **Completed**; T177 **Completed** (`AdversarialRelay`, TwinVaults, ReplicateEngine, AIBR wire); ADR-0018 **Accepted**; threat-model §7 authoritative
- **Blocks:** Phase 11 rollup (optional sync acceptance); residual marketing/claims honesty (T185 if present)
- **Category:** SECURITY
- **Normative design:** [ADR-0018](../../../Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md) + [threat-model §7](../trackT175-sync-threat-model-adr/threat-model.md) + [T177 harness](../trackT177-fake-relay-convergence/spec.md)
- **Deferred absorbed:** #53 T178 handoff (full §7 matrix + WRAP KAT + adversarial crypto); #34.1 **security proof** of multi-device CE/ACK (functional path in T177 C10–C11); T176 residual full WRAP golden matrix. **Not** #34.2 DataKey rotation.

## 1. Objective

Prove **privacy and integrity properties** of the encrypted event-replication protocol under the T175 threat model, using the **T176–T177 live harness** (no production network):

1. **Claim → executable test map** for every threat-model §7 id (or explicit `defer:` with justification).
2. **Known-answer tests (KATs)** for outer `signed_bytes`, AIBR wire framing, and **ADR-0018 §17 WRAP** (info/AAD bytes + fixed-key wrap ciphertext).
3. **Adversarial suite** via `AdversarialRelay<R>` + direct envelope mutation: metadata-swap, wrap-list tamper, forged ACK, bit-flip ciphertext, unknown/revoked device pre-verify, wrong-recipient unwrap.
4. **Honesty gates**: residual metadata leakage, ACK = attestation not wipe proof, non-claims (metadata-private, NIST Purge, PQ, perfect multi-device deletion).
5. **Optionality**: sync default off; capture works without `ai-brains-sync`.

After T178:

| Capability | Present |
|------------|---------|
| Stable `T178-*` security suite green in CI | Yes |
| WRAP golden KAT file / pinned hex | Yes |
| Meta-swap / forged ACK / L8 pre-verify adversarial proofs | Yes |
| Residual + non-claim doc asserts | Yes |
| Production network relay | **No** |
| Formal verification / FIPS / PQ | **No** (non-claims) |
| DataKey rotation (#34.2) | **No** — remains open |

## 2. Live baseline (re-scan 2026-07-31)

| Area | Live state |
|------|------------|
| T175 / ADR-0018 | **Accepted** — §7 matrix IDs frozen; L1–L16; WRAP §17 byte freeze |
| T176 | **Completed** — `ai-brains-sync` (sign/wrap/control/signed_bytes/wire); mig `0027`; partial WRAP unit KAT (OKM hex pin); control meta-swap unit smoke |
| T177 | **Completed** — `RelayPort` Memory/File/`AdversarialRelay`; `ReplicateEngine` (L8, multi-gap drain, F21 CE+ACK, outbox `0028`); TwinVaults C1–C15; store→sync prod edge; Codex **R5 PASS** |
| TwinVaults | In `ai-brains-store/tests/replication_converge.rs` (test-local); `new_enrolled_pair`, `assert_converged`, engine helpers |
| AdversarialRelay | `ai-brains-sync::relay::AdversarialRelay` — delay/drop/reorder/duplicate knobs; **exported for T178** |
| Capture | No `ai-brains-sync` dependency — must stay that way |
| Migrations | Through **0028** (outbox); T178 prefers **zero schema** unless a proven security hole forces it |
| Docs | ADR-0018 residuals; OPERATIONS CE honesty; no dedicated “sync residual metadata” section yet |

### 2.1 What T177 already proves (functional) vs T178 (security claim ids)

| T177 scenario / unit | Overlaps | T178 elevation |
|----------------------|----------|----------------|
| C7 unknown device reject | L8 | Named `T178-L8-unknown-device-preverify` + assert **no Ed25519 verify attempt** when practical |
| C8/revoked pre-verify | L4/L8 | `T178-L4-post-revoke-reject` + `T178-L8-*` |
| C10–C11 CE + ACK + timeout | L7 | `T178-L7-*`, `T178-R-offline-ce-pending-ack`, `T178-R-ack-attestation-not-wipe` |
| C4–C5 / gap drain | L13 | `T178-L13-gap-buffer`, `T178-L13-gap-no-corrupt-apply` |
| C15 seq collision | integrity | Keep as adjacent; not new id |
| C9 no LWW | L6 | `T178-L6-no-lww-conflict` (event-level OK) |
| control unit meta-swap | L5 | Promote + expand to data path + wrap-list + wire |
| wrap OKM hex pin | WRAP | Full golden matrix `T178-WRAP-kat-info-aad-bytes` |

**Rule:** Prefer **dedicated security test modules** with function names containing the stable `T178-*` id (or a comment table mapping id → test). Do **not** renumber T177 tests; thin re-assert wrappers are fine if they avoid logic drift.

## 3. Research summary (online + standards, 2026-07-31)

### 3.1 Dependency posture (crates.io / Cargo.lock)

| Crate | Workspace / lock | crates.io max_stable | License | T178 action |
|-------|------------------|----------------------|---------|-------------|
| `ed25519-dalek` | **3.0.0** | **3.0.0** | BSD-3-Clause | ✅ No bump; feature set `serde,zeroize,rand_core` OK |
| `x25519-dalek` | **3.0.0** | **3.0.0** | BSD-3-Clause | ✅ No bump; `static_secrets` + `zeroize` as today |
| `curve25519-dalek` | **5.0.0** (transitive) | 5.x | BSD-3-Clause | ✅ Transitive only |
| `hkdf` | **0.13.0** | **0.13.0** | MIT/Apache-2.0 | ✅ Matches ADR §17; salt `Some(&[])` pattern already used |
| `aes-gcm` | **0.10.3** | **0.11.0** available | MIT/Apache-2.0 | **Hold 0.10.3** — upgrade is a separate hygiene track (semver + aead trait churn); not required for suite |
| `tempfile` | 3.x | 3.27+ | MIT/Apache-2.0 | Already present |
| `proptest` | **not in workspace** | **1.11.0** | MIT/Apache-2.0 | **Optional** `[dev-dependencies]` only if property cases add unique coverage beyond fixed scenarios |
| `hpke` | not used | 0.14.x | MIT/Apache-2.0 | **Still deferred** (ADR §18); no T178 dep |

**Verdict:** Zero new **production** dependencies. Prefer zero new **dev** deps; exhaustive scenario + KAT matrix is the primary bar. If proptest is added: deny.toml allowlist check (MIT/Apache already OK) + `cargo deny check` + `cargo audit` after.

### 3.2 API / crypto notes (docs.rs + dalek)

| Topic | Finding | T178 implication |
|-------|---------|------------------|
| `x25519-dalek` 3.x secrets | Prefer `EphemeralSecret` for one-shot DH; live wrap uses `StaticSecret::from(eph_seed)` (works; zeroizes on drop via ZeroizeOnDrop) | **Do not migrate wrap path in T178** unless a test forces it — optional quality note in residual; KATs pin outputs of **current** API |
| Ed25519 verify | Strict signature verification; fail-closed on bad sig | Meta-swap / bit-flip must return structured error, never apply |
| AES-GCM (NIST SP 800-38D) | 96-bit nonce uniqueness; AAD binds identity | Content body `nonce(12)‖ct‖tag(16)`; wrap AAD = schema‖content_key‖recipient (ADR §17.3) |
| HKDF-SHA256 | salt empty; info length-prefix order frozen | WRAP KATs pin exact info/aad byte hex |
| CAVP vectors | Informal AES-GCM vectors exist; **not** FIPS validation | May use informal vectors for **primitive sanity** only; **do not** claim CAVP/FIPS. Protocol KATs (ADR labels) are mandatory |

### 3.3 Security testing best practices (applied)

| Practice | Application in T178 |
|----------|---------------------|
| **Fail-closed** | Every negative case asserts reject **and** no projection/event side effect |
| **Deterministic keys** | Fixed seeds / fixed UUIDs for KATs; no wall-clock flakiness |
| **Hermetic** | `tempfile` vaults; `MemoryFakeRelay` / `AdversarialRelay`; no real network |
| **Named claim IDs** | Threat-model §7 IDs are stable labels; test names or index table must include them |
| **Attack-first order** | Opacity → tamper → meta-swap → pre-verify → revoke → forged ACK → gap → honesty docs |
| **No AGPL tooling** | No external pen-test frameworks; in-tree only |
| **Property tests optional** | proptest only for reorder/duplicate/bit-position combinations beyond fixed cases |
| **Separate functional vs security** | T177 = converge; T178 = adversary + KAT + non-claim honesty |

### 3.4 Standards alignment

| Source | Application |
|--------|-------------|
| STRIDE (T175) | Suite covers Spoof/Tamper/InfoDisc/DoS gates already designed |
| NIST SP 800-38D | Nonce packing + AAD discipline KATs; honesty: not a validated module |
| NIST SP 800-88r2 | CE multi-device best-effort; **no** Purge/remote wipe claim tests must pass honesty scan |
| RFC 9180 HPKE | Deferred — suite asserts **no** HPKE crate dependency (optional cargo tree assert) |
| Untrusted-relay E2EE | Relay opacity + no forge (`T178-L9-*`) |
| AGENTS.md | No `unwrap`/`expect` in prod; capture independence; deny/audit gate |

## 4. Freezes (F1–F28)

| ID | Freeze |
|----|--------|
| **F1** | Authoritative claim map = threat-model **§7**; do not invent competing IDs |
| **F2** | Every executable id has a green test **or** an explicit `defer:` row with reason + owner residual |
| **F3** | Prefer **zero** new production deps; proptest only optional dev |
| **F4** | Hold `aes-gcm` **0.10.x** — no opportunistic 0.11 upgrade in this track |
| **F5** | Reuse T177 harness: `AdversarialRelay`, TwinVaults pattern, `ReplicateEngine` |
| **F6** | WRAP KATs pin **exact** ADR §17.1–17.3 bytes (`T178-WRAP-kat-info-aad-bytes`) — **static protocol KATs** (info/aad/okm) are mandatory; full ciphertext goldens only via **seeded** path (F20) |
| **F7** | Outer sig KATs pin `build_signed_bytes` + wire `AIBR` framing (`T178-L5-sig-canonical-bytes`) via **thin T178-id wrappers** over existing T176 fixtures (prefer new `t178_*` test fn + comment map, not rename-only) |
| **F8** | L8: unknown/revoked/inactive device → reject **before** expensive verify when engine path allows. **Runtime** asserts: error kind is `NotEnrolled`/`DeviceRevoked` (not `SignatureInvalid`) + F19. **"No verify attempt"** is a **code-path review** claim (structural return before `verify_envelope`), not runtime instrumentation |
| **F9** | L5 fail-closed: meta-swap, wrap-list reorder/tamper, ciphertext bit-flip → no apply |
| **F10** | L7: forged ACK rejected; honest states `pending\|acked\|failed\|unreachable`; ACK ≠ wipe proof. Forged ACK **Must** cover **two layers** (F24) |
| **F11** | L9: relay stores only opaque bytes + public device graph residue. **Parse ≠ forge** (F25): wire decode without verify is allowed residual metadata; forge = mutated body fails peer verify/apply |
| **F12** | L4: post-revoke no future wraps; DeviceId permanently retired (re-enroll same id fails) |
| **F13** | L12: capture crate graph has **no** edge to `ai-brains-sync` — **programmatic CI test** (F21), not manual-only `cargo tree` |
| **F14** | Local-only default: replicate without explicit fake-relay remains error (T177 CLI contract) |
| **F15** | Residual + non-claim honesty documented in **`Docs/OPERATIONS.md`** (canonical home — F26); automated markdown honesty scanner (F27) |
| **F16** | No production network surface; no AGPL security SaaS |
| **F17** | `#34.2` DataKey rotation **out of scope** (defer remains); residual doc may note per-seal wrap keys improve multi-device nonce budget but **do not** close vault DataKey rotation |
| **F18** | Zero `unwrap`/`expect`/`panic` in production paths touched; tests may use allowlisted test helpers |
| **F19** | **Side-effect isolation (AI1 BS1):** every negative apply path **Must** use `assert_rejected_no_side_effect` — reject **and** security-relevant snapshot equality (event_ids, envelope_index, peer cursors, device_identity rows, erasure_ack rows). `Err` alone is insufficient. |
| **F20** | **WRAP KAT split (AI1 BS2 + AI2 #1):** (1) **Static Must** — `build_wrap_info` / `build_wrap_aad` / `derive_wrap_key` fixed IKM → exact hex (`T178-WRAP-kat-info-aad-bytes`). (2) **Seeded Must** — full wrap_ct golden via **test-export** of internal `wrap_with_eph` as `pub(crate)` or `#[cfg(test)]` / `#[doc(hidden)]` helper with injected eph seed + nonce (`T178-WRAP-kat-seeded-ciphertext`). Production `wrap_content_dek_for_recipient` keeps OS CSPRNG. **Never** pin ciphertext from `thread_rng`. Prefer `include_str!` goldens under `crates/ai-brains-sync/tests/kats/`. |
| **F21** | **Capture isolation gate (AI1 BS4):** automated test parses `crates/ai-brains-capture/Cargo.toml` (and/or `cargo metadata`) and fails if `ai-brains-sync` appears. Optional evidence: `cargo tree -p ai-brains-capture` + assert **absence** of `ai-brains-sync` in output (**not** `-i ai-brains-sync`, which exits non-zero with "did not match") |
| **F22** | **Three-vector replay (AI1 BS3):** exact-duplicate + modified-seq + post-revoke replay (§5.1.1) |
| **F23** | **Shared harness extract (AI2 A):** Phase A0 extracts `tests/common/` (`TestVault`, `TwinVaults`, snapshots, F19 helper) before `replication_security.rs`; both converge + security import it. Prefer `triple_enrolled()` helper for 3-device residual cases |
| **F24** | **Forged ACK two layers (AI2 #6):** (1) random/bad Ed25519 sig → `SignatureInvalid`, eraser stays pending; (2) valid sig from **different enrolled** device with spoofed `peer_device_id` → binding reject, eraser stays pending |
| **F25** | **L9 forge without AdversarialRelay mutate knob (AI2 #4–5):** Prefer **test-local** blob mutation: pull/build `RelayBlob`, flip one body byte, `apply_blob` + F19. Optional later: `MemoryFakeRelay` mutate helper. Do **not** require AdversarialRelay body-mutate for Must |
| **F26** | **Residual doc home = `Docs/OPERATIONS.md`** section "Multi-device sync residuals" (AI2 #9). Include metadata, ACK attestation, offline CE lag, PQ/classical, #34.2 note, pad≠metadata-private |
| **F27** | **Doc honesty scanner (AI1 opp 2):** `doc_claims_honesty` (or equivalent) via `include_str!` on OPERATIONS (+ ADR residual phrases as needed). Require presence of key disclaimers; forbid marketing strings ("Zero-knowledge relay", "Post-quantum secure", unqualified "NIST Purge" as product claim for multi-device, etc.) |
| **F28** | **Codex R2 id tags (AI1 opp 3):** each of the 7 non-negotiable ids appears in source as `// T178-…` (or test name containing the id) so the matrix is greppable |

## 5. Claim → test matrix (authoritative expansion)

Legend: **Must** = minimum bar for track Complete; **Should** = implement if cheap; **Defer** = document only.

### 5.1 Design locks L1–L16

| Test id | Disposition | Must | Implementation sketch |
|---------|-------------|------|------------------------|
| `T178-L1-local-only-default` | Executable | Yes | Config/CLI: replicate without `--fake-relay` → structured error; no silent network |
| `T178-L1-relay-opaque` | Executable | Yes | MemoryFakeRelay store inspect: body bytes not equal plaintext event payload; no key material fields |
| `T178-L2-device-pub-only-relay` | Executable | Yes | Relay index / enrollment packages expose pubs only; private key material never written to relay paths |
| `T178-L3-enroll-fingerprint` | Executable | Yes | OOB fingerprint = SHA-256(package); mismatch rejects enroll |
| `T178-L3-reject-unbound-pin` | Should / Defer if no PIN UX | — | If no PIN surface: **defer: no unbound PIN API in v1** |
| `T178-L3-enroll-binds-x25519` | Executable | Yes | Enrollment package X25519 swapped vs fingerprint → reject / no wrap to attacker key |
| `T178-L3-enroll-signer-must-be-enrolled` | Executable | Yes | Elevate T177 bad-signer case with T178 id |
| `T178-L4-revoke-no-future-wrap` | Executable | Yes | After revoke, new data envelopes omit revoked recipient wrap row |
| `T178-L4-post-revoke-reject` | Executable | Yes | Envelopes signed by revoked device rejected at L8 |
| `T178-L4-revoke-signer-must-be-enrolled` | Executable | Yes | DeviceRevoked from unknown signer rejected |
| `T178-L4-deviceid-permanently-retired` | Executable | Yes | Re-enroll same DeviceId after revoke fails |
| `T178-L5-sig-canonical-bytes` | Executable KAT | Yes | Thin `t178_l5_sig_canonical_bytes__kat` wrapper over T176 `signed_bytes__fixture__exact_hex` (+ wire if free) (**F7**) |
| `T178-L5-meta-swap-fails` | Executable | Yes | Same body, different meta → verify fail; **F19** no side effects |
| `T178-L5-wrap-list-tamper` | Executable | Yes | Mutate wrap list under same outer sig → fail; **F19** |
| `T178-L5-content-nonce-in-blob` | Executable KAT | Yes | Data body = 12+ct+16; outer covers full blob; no separate unsigned nonce |
| `T178-L5-control-cleartext-parse` | Executable | Yes | Control N=0; parse after sig; no DEK unwrap |
| `T178-L5-tamper-ct` | Executable | Yes | Bit-flip ciphertext → AEAD or sig fail-closed; **F19** no side effects |
| `T178-L5-replay-idempotent` | Executable umbrella | Yes | Parent id; implement via §5.1.1 three-vector suite |
| `T178-L5-replay-exact-duplicate` | Executable | Yes | Same envelope_id + local_seq twice → idempotent; set stable (**F22**) |
| `T178-L5-replay-modified-seq` | Executable | Yes | Old body re-stamped with new `local_seq` without re-sign → sig fail; F19 (**F22**) |
| `T178-L8-replay-revoked-device` | Executable | Yes | Valid historical envelope from device later revoked → L8 drop; F19 (**F22**) |
| `T178-L6-topo-apply` | Should | — | Parent/causal order if cheap; else note covered by domain apply rules |
| `T178-L6-no-lww-conflict` | Executable | Yes | Elevate C9 event-level both-present |
| `T178-L7-ack-signed` | Executable | Yes | Valid ErasureAck verifies and updates projection |
| `T178-L7-ack-cleartext-signed` | Executable | Yes | ACK is control cleartext (wrap_count=0) |
| `T178-L7-forged-ack-reject` | Executable | Yes | **F24** two sub-cases (bad-sig + wrong-signer binding) + **F19**; eraser not spuriously `acked` |
| `T178-L7-ack-states` | Executable | Yes | `pending\|acked\|failed\|unreachable` transitions (tick N=3); **pin status normalization** for peer-supplied garbage/`wiped` (live: non-acked/failed coerce to `acked` — document + test so behavior cannot silently change) |
| `T178-L8-unknown-device-preverify` | Executable | Yes | Elevate C7; assert `NotEnrolled` (not `SignatureInvalid`) + F19; no-verify is code-path review (**F8**) |
| `T178-L8-aead-fail-closed` | Executable | Yes | Wrong DEK wrap / bad tag → no append |
| `T178-L8-smuggled-membership-reject` | Alias / reaffirm | Yes | Elevate T177 `project_data__smuggled_device_revoked__reject` under T178 id (membership control must not ride DataEvent body) |
| `T178-L9-relay-no-decrypt` | Executable | Yes | Blob body has no plaintext substring; DEK/DataKey material not present as clear fields; opacity inspect (**F11**) |
| `T178-L9-relay-no-forge` | Executable | Yes | Mutated body fails peer verify/apply + F19; **parse-without-verify is allowed** (**F11**, **F25**) |
| L10 CLI naming | **Defer** | — | `defer: CLI naming — T176/T177 surface docs; not crypto suite` |
| `T178-L11-recovery-no-resurrect-dek` | Should | — | RecoveryKit path does not restore destroyed content DEK (P8+store); skip if fixture costly with ISSUES note |
| `T178-L11-partial-ce-ux` | Executable | Yes | Offline peer → pending/unreachable honesty (not silent full wipe) |
| `T178-L12-capture-without-sync` | Executable | Yes | **F21** programmatic Cargo.toml / metadata gate; optional `cargo tree -p ai-brains-capture` + absence grep (not `-i`) |
| L12 deny/audit | **Gate** | Yes | `cargo deny check` + `cargo audit` green (not a unit test id) |
| `T178-L13-gap-buffer` | Executable | Yes | Elevate gap drain fixture |
| `T178-L13-gap-no-corrupt-apply` | Executable | Yes | No apply past gap until fill or signed GapSkipAudit |
| `T178-L13-gap-skip-audit` | Executable | Should | Elevate T177 gap_skip_audit if present |
| `T178-L14-pad-buckets` | Executable | Yes | Padding module bucket membership unit test |
| `T178-L14-pad-not-metadata-private` | Doc assert | Yes | Residual doc states padding ≠ metadata-private |
| L15 multi-user | **Defer** | — | `defer: product fence — no multi-user API` |
| L16 PQ | Doc | Yes | Covered by `T178-NC-no-pq-claim` |

### 5.2 Residuals (R)

| Test id | Must | Sketch |
|---------|------|--------|
| `T178-R-metadata-doc` | Yes | **F26** `Docs/OPERATIONS.md` residual section + **F27** scanner phrases (sizes/counts/timing/device graph) |
| `T178-R-offline-ce-pending-ack` | Yes | Offline peer → pending until sync |
| `T178-R-ack-attestation-not-wipe` | Yes | Doc + UX: acked ≠ proven wipe everywhere; **F28** tag |
| `T178-R-revoke-past-still-open` | Yes | Historical content still openable on revoked device's **local** vault (future exclusion only). Fixture: **2-vault sufficient** if B keeps local DEK/wraps after A revokes B; optional **triple_enrolled** if proving "no wrap on new content to B while C gets wraps" (**F23**) |
| Pre-erase backups | Defer | Physical residual ADR-0016 |
| PQ harvest-now | Defer | Residual doc + NC |
| FIPS / Purge | Defer | Claims gate / NC tests |
| DataKey rotation | Defer | **#34.2** remains open |
| Gap skip operator error residual | Doc | Fail-closed default already tested |

### 5.3 Non-claims (NC)

| Test id | Must | Sketch |
|---------|------|--------|
| `T178-NC-metadata` | Yes | **F27** presence: not metadata-private / metadata leakage residual; absence of "Zero-knowledge relay" |
| `T178-NC-partial-erase` | Yes | Partial ACK UX / doc |
| `T178-NC-ack-not-wipe-proof` | Yes | **F27** "attestation" honesty; **F28** tag |
| `T178-NC-no-purge-claim` | Yes | **F27** no multi-device NIST Purge product claim |
| `T178-NC-no-pq-claim` | Yes | **F27** classical / not post-quantum secure |
| Compliance cert | Defer | T185 / product claims gate |
| Multi-user vault | Defer | L15 |

### 5.4 WRAP construction

| Test id | Must | Sketch |
|---------|------|--------|
| `T178-WRAP-per-recipient-roundtrip` | Yes | Existing unit elevate + multi-recipient list; DEK equality under live CSPRNG |
| `T178-WRAP-wrong-recipient-fails` | Yes | Wrong static secret → WrapOpenFailed |
| `T178-WRAP-kat-info-aad-bytes` | Yes | **Static** (F20): exact info/aad/okm hex for fixed IKM — no RNG; elevates live `wrap_dek__hkdf_okm__kat` |
| `T178-WRAP-kat-seeded-ciphertext` | Yes | **Must** after Phase **B0** seed-export (F20 / AI2 #1 / ADR §17.4); fixed eph seed + nonce → pinned wrap_ct |
| `T178-WRAP-no-shared-datakey-over-relay` | Yes | **Structural:** `WrapRecord` / `encode_wrap_record` emit only recipient‖eph_pub‖nonce‖wrap_ct; no DataKey/vault_key field; wrap_ct length consistent with DEK+tag (AI2 #8) |
| `T178-WRAP-nonce-uniqueness` | Should | Wrap same DEK N≈100–1000× same recipient; all wrap nonces distinct (birthday sanity) |

### 5.1.1 Three-vector replay suite (F22 — AI1 BS3)

| Vector | Test id | Attacker action | Expected |
|--------|---------|-----------------|----------|
| **A** Exact duplicate | `T178-L5-replay-exact-duplicate` | Re-push identical signed envelope (`envelope_id` + `local_seq`) | Idempotent success / no double side effect; event_id set stable |
| **B** Sequence re-stamp | `T178-L5-replay-modified-seq` | Copy old envelope, bump `local_seq` (or other signed meta) **without** re-signing | Signature fail; **F19** no apply / no cursor advance past honest state |
| **C** Post-revoke replay | `T178-L8-replay-revoked-device` | After device revoked, re-present an **old valid** signed envelope from that device | L8 pre-verify drop; **F19**; does not re-open membership |

Parent matrix id `T178-L5-replay-idempotent` remains the umbrella claim from threat-model §7; implementers ship the three cases above (C may also be asserted under L8).

**Must-include residual/control/wrap ids (Codex R2 handoff — non-negotiable; **F28** tag each in source):**  
`T178-R-ack-attestation-not-wipe`, `T178-NC-ack-not-wipe-proof`, `T178-WRAP-kat-info-aad-bytes`, `T178-L5-control-cleartext-parse`, `T178-L5-content-nonce-in-blob`, `T178-L7-ack-cleartext-signed`, `T178-L4-deviceid-permanently-retired`.

## 6. Architecture of the suite

### 6.1 Module layout (preferred)

```
crates/ai-brains-sync/
  src/…                    # production unchanged unless KAT hooks needed
  tests/                   # optional integration if not only unit
  # Prefer #[cfg(test)] modules + dedicated integration tests:

crates/ai-brains-store/tests/
  common/mod.rs            # F23 A0 extract — TwinVaults, snapshots, F19
  common/twin_vaults.rs
  replication_security.rs  # NEW — T178-* scenarios
  replication_converge.rs  # T177 — import common (refactor only)

crates/ai-brains-sync/
  src/wrap.rs              # B0: export wrap_with_eph / with_seed for F20 seeded KAT
  src/* tests              # thin t178_* wrappers + KATs
  tests/kats/*.hex         # include_str! goldens (preferred)

Docs/OPERATIONS.md         # F26 "Multi-device sync residuals" (canonical)
# optional: store or sync tests/doc_claims_honesty.rs for F27
```

### 6.2 Shared harness

| Helper | Location | Notes |
|--------|----------|-------|
| `TwinVaults::new_enrolled_pair` | **`tests/common/`** (**F23** extract first) | Enroll OOB + MemoryFakeRelay |
| `triple_enrolled` | common (**F23**) | Optional 3-vault + shared MemoryFakeRelay for revoke-past / multi-peer CE |
| `assert_converged` | common | event_id sets |
| `AdversarialRelay` | `ai-brains-sync` | delay/drop/reorder/duplicate; **not** required for body mutate (**F25**) |
| `mutate_blob_body_byte` | test-local (**F25**) | Flip one byte in `RelayBlob.body` then `apply_blob` |
| `capture_security_snapshot` | common (**F19**) | Security-relevant state only — not full SQL dump |
| `assert_rejected_no_side_effect` | common (**F19** — **mandatory** on negative apply) | Snapshot → apply → `Err` kind match → snapshot equality |
| `kat_hex!` / `include_str!` | `tests/kats/` | Build-time goldens (AI2 C) |
| `assert_capture_has_no_sync_dep` | capture or store test (**F21**) | Cargo.toml / metadata parse |

**`assert_rejected_no_side_effect` contract (normative sketch):**

```rust
// Pseudocode — adapt to live TestVault / EngineError types
fn assert_rejected_no_side_effect(
    vault: &TestVault,
    blob: &RelayBlob,
    // optional: match err kind without requiring Debug eq of full message
) {
    let before = vault.capture_security_snapshot();
    let res = vault.apply_blob(blob);
    assert!(res.is_err(), "adversarial blob must fail closed");
    let after = vault.capture_security_snapshot();
    assert_eq!(
        before, after,
        "adversarial rejection MUST leave zero security-relevant DB/cursor side-effects"
    );
}
```

**Rationale (AI1 BS1):** returning `Err` after partial mutation (cursor advance, index insert of unverified row, identity row) is a real integrity bug. Snapshot equality closes that gap.

### 6.3 KAT golden file format (F20 — AI1 BS2)

Prefer **committed hex fixtures** (deterministic, reviewable). **Split WRAP suite:**

| Tier | What is pinned | RNG? | Must |
|------|----------------|------|------|
| **Static protocol KATs** | `info`, `aad`, HKDF OKM for fixed IKM + fixed ids/label | **No** | Yes (`T178-WRAP-kat-info-aad-bytes`) |
| **Seeded ciphertext KAT** | full `wrap_nonce` + `wrap_ct` for fixed eph seed + fixed wrap nonce | Test-only seed / inject | Should |
| **Live round-trip** | wrap → unwrap equality | OS CSPRNG (production path) | Yes (`T178-WRAP-per-recipient-roundtrip`) — asserts equality of DEK, **not** ciphertext bytes |

```
# kats/wrap_info_schema1_ids_1_2_3.hex  (static)
# comment lines with #
ac8b...

# kats/wrap_aad_schema1_....hex
# kats/wrap_okm_ikm42_....hex
# kats/wrap_ct_seeded_....hex   (only if seeded API lands)
```

**Forbidden:** pinning ciphertext produced by `thread_rng` / production wrap without a seed inject (flaky by construction).

**Production API:** keep `wrap_content_dek_for_recipient` on OS CSPRNG. Seeded path is `#[cfg(test)]` or `pub(crate)` + test-only module — do **not** expose a production “insecure seed” feature flag.

### 6.4 Adversarial mutation helpers

| Helper | Mutates | Expect |
|--------|---------|--------|
| `swap_meta(outer fields)` | device_id / local_seq / event_id | sig fail |
| `tamper_wrap_list` | reorder or replace wrap row | sig fail |
| `bit_flip_body` | one byte in ciphertext blob | fail-closed |
| `forge_ack_bad_sig` | random 64-byte sig | L7 SignatureInvalid + F19 (**F24-1**) |
| `forge_ack_wrong_signer` | valid sig other enrolled device, spoofed peer_device_id | L7 binding reject + F19 (**F24-2**) |
| `mutate_blob_body_byte` | one body byte flip | L9 no-forge + F19 (**F25**); prefer over AdversarialRelay mutate |

## 7. CLI / optionality

| Check | Expected |
|-------|----------|
| Default install / local vault | Multi-device **off** |
| `ai-brains replicate push` without fake-relay config | Structured error (T177) |
| Capture path | Works with sync unused |
| `--format json` on status | If T177 shipped; else Should |

No new CLI commands required for T178. Optional: `ai-brains replicate status` surfaces security-relevant gap/ack states already from T177.

## 8. Deferred.md absorption

| Item | Disposition in T178 |
|------|---------------------|
| **#53** full threat-model matrix + WRAP KAT + adversarial | **Absorbed** (this track) |
| **#34.1** multi-device CE/ACK | Functional T177; **security proof** T178 L7/R/NC |
| T176 WRAP golden residual | **Absorbed** via `T178-WRAP-kat-info-aad-bytes` |
| **#34.2** DataKey rotation | **Out of scope** — remains open |
| **#51** signer/schema | Already T177; reaffirm under L3/L8 ids |
| T177 residual CLI bootstrap→outbox | Out of scope unless blocks a Must id |
| L10 / L15 / L16 product fences | Explicit defer rows |
| HPKE / MLS | Defer |

## 9. Non-goals

- Production HTTP/WebSocket/libp2p relay hardening  
- Formal verification (ProVerif, Tamarin, etc.)  
- FIPS 140 / CAVP certification program  
- Post-quantum hybrid KEM  
- DataKey rotation implementation (#34.2)  
- Claiming metadata-private or remote wipe  
- AGPL or commercial SaaS security scanners in CI  
- Upgrading aes-gcm 0.10 → 0.11  
- Migrating wrap to `EphemeralSecret` (optional residual note only)  
- Changing capture pipeline  
- Repurposing CLI `sync` / `safety sync`  

## 10. Testing strategy

### 10.1 Unit (`ai-brains-sync`)

| Area | Tests |
|------|-------|
| WRAP static KATs | info/aad/okm hex (**F20**); no RNG |
| WRAP seeded (Should) | full wrap_ct golden with test-only RNG |
| WRAP live | roundtrip DEK equality; wrong recipient |
| signed_bytes / wire | Canonical hex; meta-swap; wrap tamper |
| control | Cleartext parse; forged ACK; ErasureAck states encode |
| padding | Bucket membership |
| device keys | Pub-only serialization if exposed |
| L12 isolation | **F21** capture Cargo.toml / metadata parse test |

### 10.2 Integration (`ai-brains-store` tests)

| Area | Tests |
|------|-------|
| L3–L5–L8–L9 | TwinVaults + AdversarialRelay / mutated blobs; **all negatives via F19** |
| L4 revoke suite | Future wrap exclusion + permanent retirement + **F22 vector C** |
| L5 replay | **F22** vectors A/B (+ C under L8) |
| L7 CE/ACK adversarial | Forged ACK over relay + F19 |
| L13 gap | No corrupt apply |
| L1/L12 | Local-only + programmatic capture independence |

### 10.3 Doc / honesty (**F26** / **F27**)

| Test | Intent |
|------|--------|
| `doc_claims_honesty` / `include_str!("…/OPERATIONS.md")` | Residual section present; required disclaimers; forbidden marketing strings |
| `T178-R-metadata-doc` | sizes/counts/timing/device graph residual language |
| NC-* | not metadata-private; attestation only; classical; no Purge/PQ/ZK-relay claims |

### 10.4 Optional property (`proptest` dev-dep)

Only if fixed matrix leaves gaps (unique value: bit-position enumeration):

- Random bit position flips on sealed blob → always reject + F19  
- If used: **`ProptestConfig::with_cases(100)`** (or equivalent) so default tier stays **&lt;60s** (AI1 opp 1; AGENTS.md)  
- Prefer optional `#[cfg(feature = "proptest")]` if default CI budget is tight (AI2 E)

### 10.5 Gates

```powershell
cargo nextest run -p ai-brains-sync -p ai-brains-store --test replication_security
cargo nextest run -p ai-brains-sync -p ai-brains-store
cargo clippy -p ai-brains-sync -p ai-brains-store --all-targets -- -D warnings
cargo deny check
cargo audit
# F21 primary = unit test. Optional evidence (prefer absence grep, not -i):
cargo tree -p ai-brains-capture
# assert output does not contain ai-brains-sync
```

Full workspace gate before finalize per AGENTS.md.

## 11. Risks & honesty

| Residual | Handling |
|----------|----------|
| Suite ≠ formal proof | Document limitation; claim only tested properties |
| Compromised enrolled peer false ACK | L7 residual + `T178-R-ack-attestation-not-wipe` |
| Metadata leakage remains | `T178-R-metadata-doc` + NC |
| Fake relay trust model | Still L9 adversarial; file path is local trust |
| aes-gcm 0.11 not adopted | Hygiene residual; not a security regression of protocol |
| #34.2 open | Explicit defer |
| TwinVaults not lib-exported | OK to keep under `tests/`; extract helper module if duplication hurts |

## 12. Definition of Done

- [ ] Full §7 matrix table in this spec has every **Must** id green or justified defer with residual owner  
- [ ] WRAP **static** KAT + **seeded** wrap_ct KAT green (**F20**, B0 export done); no flaky ciphertext pins  
- [ ] Meta-swap, wrap-list tamper, forged ACK (**F24** both layers), L8 pre-verify, L9 opacity/forge green — all with **F19**  
- [ ] Three-vector replay suite (**F22**) green  
- [ ] **F21** capture isolation + **F27** doc honesty scanner + **F26** OPERATIONS residual section  
- [ ] **F28** all 7 Codex R2 ids greppable in source/test names  
- [ ] Capture independence held; no production network; deny/audit green  
- [ ] Manual evidence recorded; review log clean (critical/high verified)  
- [ ] conductor + deferred updated; #53 struck/promoted; #34.2 still open  
- [ ] Phase 11 rollup: T175–T178 complete for fake-relay security acceptance  

## 13. AI1 + AI2 review fold-in (2026-07-31)

### AI1

| Source | Disposition |
|--------|-------------|
| **BS1** side-effect isolation | **Agree** → **F19** (security-relevant snapshot; `matches!` err kinds) |
| **BS2** static vs seeded WRAP KATs | **Agree** → **F20** |
| **BS3** three-vector replay | **Agree** → **F22** |
| **BS4** programmatic L12 | **Agree** → **F21** (tree evidence without `-i`) |
| **Opp1** proptest `with_cases(100)` | **Agree** → §10.4 if proptest used |
| **Opp2** markdown honesty scanner | **Agree** → **F27** |
| **Opp3** Codex R2 id tags in source | **Agree** → **F28** |
| **§5 summary 1–4** | Covered by F19/F20/F22/F27 |

### AI2

| Source | Disposition |
|--------|-------------|
| **#1** seed-inject for full wrap_ct | **Agree** → seeded KAT **Must** + Phase **B0** export `wrap_with_eph` |
| **#2** L5-sig thin wrapper | **Agree** → **F7** |
| **#3** L8 no-verify not instrumentable | **Agree** → **F8** (runtime kind + code-path review) |
| **#4–5** L9 parse≠forge; mutate path | **Agree** → **F11** + **F25** (test-local byte flip) |
| **#6** forged ACK two layers | **Agree** → **F24** |
| **#7** revoke-past fixture | **Agree** → **F23** optional `triple_enrolled` |
| **#8** WRAP-no-shared-datakey structural | **Agree** |
| **#9** residual doc path | **Agree** → **F26** `Docs/OPERATIONS.md` |
| **#10** cargo tree `-i` bug | **Agree** → full tree + absence |
| **#11** aes-gcm 0.11 hold | **Agree** — no action (F4) |
| **A** extract `tests/common` | **Agree** → **F23** Phase A0 |
| **C** `include_str!` goldens | **Agree** |
| **E** proptest optional + bound | **Agree** with AI1 Opp1 |
| **F** nonce uniqueness Should | **Agree** → `T178-WRAP-nonce-uniqueness` |
| **G** smuggled-membership id | **Agree** → `T178-L8-smuggled-membership-reject` |
| **H** ACK status normalization pin | **Agree** → under `T178-L7-ack-states` |

**Not folded:** AdversarialRelay body-mutate required for Must; aes-gcm 0.11 upgrade; #34.2 implementation.

## 14. Phase 11 acceptance rollup

- [x] T175 threat model + ADR **Accepted**  
- [x] T176 crate/schema  
- [x] T177 fake relay convergence  
- [ ] T178 security tests (this track)  
- [ ] Sync optional; local-only intact (reaffirm via suite)

## 15. License / commercial

- Tests use in-tree crypto + fake relay only.  
- PolyForm NC product license unchanged; third-party deps stay permissive.  
- No AGPL pen-test frameworks.  
- No uploading customer envelopes to third-party security SaaS in CI.
