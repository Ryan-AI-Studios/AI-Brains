# T176 Plan — Sync Crate + Replication Schema (P11.1)

Status: **Completed** (2026-07-31) — Codex R3 PASS; squash-merged PR #47 (`687239b`).
Normative: ADR-0018 Accepted + `spec.md` locks **R1–R30**.

## Handoff freezes (do not re-litigate)

- Migration **`0027_replication_state`** (not 0026)
- Crate **`ai-brains-sync`**; CLI **`device`** + **`replicate`** (keep Ledgerful `sync` + `safety sync`)
- Wrap PK **`(content_key_id, recipient_device_id)`** with **upsert** replace; X25519 + HKDF `Some(&[])` + AES-256-GCM
- Control cleartext `N=0`; data body `nonce‖ct‖tag`; single erasure code **`0x0012`**
- Deps: ed25519-dalek **3.x**, x25519-dalek **3.x** (**no `getrandom` feature**), curve25519-dalek **5.x** transitive, hkdf **0.13**
- **R25** panic-free keygen; **R6** DataKey + DPAPI dual-wrap (Windows); **R23** revoke wrap DELETE
- No sockets; no HPKE/OpenMLS; no DataKey rotation impl (#34.2 stays open)
- ACK projection schema + types (#34.1 implement)

## Preflight (before first edit)

- [ ] `ledgerful doctor`
- [ ] `ledgerful ledger status --compact`
- [ ] `ledgerful scan --impact` (scope: store migrations, crypto, cli main)
- [ ] Confirm migrations end at 0026; next free **0027**
- [ ] `cargo tree` baseline (no dalek yet)

## Phase A — Dependencies + crate scaffold

- [x] **A1** Add workspace deps: `ed25519-dalek` 3 (`serde`+`zeroize`+`rand_core`), `x25519-dalek` 3 (`serde`+`zeroize`+`static_secrets` only — **no getrandom**), `hkdf` 0.13
- [x] **A2** Create `crates/ai-brains-sync` + `Cargo.toml` + `lib.rs` (error types, modules stubs)
- [x] **A3** Register workspace member in root `Cargo.toml`
- [x] **A4** `cargo deny check` + `cargo audit` + inventory `curve25519-dalek` 5.x (`cargo tree -i curve25519-dalek`)
- [x] **A5** Module docs: R25 keygen, R11 salt clarity (not crypto fork), §5.1.1 private blob AAD

## Phase B — Crypto primitives (TDD)

- [x] **B1** RED: enrollment package + fingerprint KAT + R24 hyphen format
- [x] **B2** GREEN: `enrollment.rs` + dual-key generate (`device_keys.rs`, R25 fallible fill, zeroize)
- [x] **B3** RED: `signed_bytes` canonical concat fixture
- [x] **B4** GREEN: encode/sign/verify envelope outer; meta-swap fails
- [x] **B5** RED: HKDF OKM KAT for fixed IKM+info (`Some(&[])` call site)
- [x] **B6** GREEN: `wrap.rs` per-recipient wrap/unwrap; wrong recipient fails
- [x] **B7** Data body nonce‖ct‖tag pack/unpack + padding buckets
- [x] **B8** Control payloads: enroll/revoke/`ContentErasureTombstone`/`ErasureAck`/`GapSkipAudit`; `wrap_count=0`
- [x] **B9** `apply_order` pre-decrypt tie-break only (R30) — not domain topo
- [x] **B10** Device private blob seal/open under DataKey (§5.1.1 AAD kind `0x03`)

## Phase C — Migration + store APIs (TDD)

- [x] **C1** RED: migration empty-state / upgrade tests expecting 0027 tables
- [x] **C2** GREEN: `0027_replication_state.sql` + register in `migrations.rs` (no content_hash column)
- [x] **C3** Store module — identity (`local`/`active`/`revoked`), tombstone, private key wrap (`protection`), peer wrap upsert, envelope index, cursors, gap buffer metadata, erasure ACK, gap-skip audit index
- [x] **C4** CHECK constraints / length checks; `enrolled_by_device_id NOT NULL`
- [x] **C5** Rebuild policy comments in `replay.rs` (retain side stores)
- [x] **C6** R23: `delete_peer_wraps_for_recipient` on revoke path
- [x] **C7** Optional low-risk: CE destroy → `delete_peer_wraps_for_key`

## Phase D — Device key storage

- [x] **D1** Inner seal: 64-byte seeds under DataKey (§5.1.1)
- [x] **D2** Windows: outer DPAPI via `ai_brains_crypto::dpapi`; `protection = datakey_dpapi`
- [x] **D3** Non-Windows: DataKey-only path; no raw secrets; Debug redaction

## Phase E — CLI

- [x] **E1** Add `Commands::Device` / `Commands::Replicate`; **do not** touch Sync or Safety Sync semantics
- [x] **E2** `device bootstrap` (R26/R27) | `fingerprint` (R24) | `list` | `package-export` | `enroll` | `revoke` (R23)
- [x] **E3** Document §9.1 ceremony in `--help` / module docs
- [x] **E4** `replicate status | cursors`; `push`/`pull` → structured T177 deferred error
- [x] **E5** Honesty: optional; not PQ; not remote wipe; not metadata-private
- [x] **E6** Capture independence check

## Phase F — Deferred hygiene

- [x] **F1** #34.1 implementing (schema+types+unit encode/verify); full CE/relay proof → T177/T178
- [x] **F2** Leave #34.2 DataKey rotation open
- [x] **F3** #50 note: T176 AI fold-in complete

## Phase G — Verification & close

- [x] **G1** Targeted nextest + clippy on touched packages
- [x] **G2** Full gate: fmt, clippy, nextest, deny, audit
- [ ] **G3** Manual evidence: bootstrap → fingerprint (hyphen) → list → second bootstrap fails → replicate status
- [ ] **G4** `ledgerful verify` + ledger commit when implementing
- [ ] **G5** `review.md`; cross-model for SECURITY
- [ ] **G6** Handoff notes for T177
- [ ] **G7** conductor → Complete only after review convergence

## RED tests named (minimum)

```
enrollment_package__dual_keys__fingerprint_stable
fingerprint_format__hyphen_groups__16_groups
signed_bytes__fixture__exact_hex
verify_envelope__metadata_swap__err
wrap_dek__hkdf_okm__kat
wrap_dek__roundtrip_recipient__ok
wrap_dek__wrong_static_key__err
data_body_pack__nonce_ct_tag__roundtrip
control_device_enrolled__wrap_count_zero__ok
padding__len_to_bucket__256_4k_64k
apply_order__tie_break__device_seq_event
device_keys__try_fill__no_unwrap_err
device_private_blob__aad__kind_0x03
migration_0027__fresh_vault__tables_exist
device_tombstone__re_enroll_same_id__rejected
peer_wrap_pk__upsert__replaces
revoke__deletes_recipient_wraps
bootstrap__second_call__err
first_device__enrolled_by__self
envelope_index__duplicate_event_id__idempotent
cli_device_bootstrap__temp_vault__lists_local
cli_device_fingerprint__hyphen_form
cli_replicate_push__no_relay__structured_err
```

## Out of scope checklist (reject if PR sneaks them in)

- [ ] Real TCP/HTTP/WebSocket relay
- [ ] Fake multi-client relay harness (T177)
- [ ] Full T178 threat-model matrix
- [ ] `hpke` / `openmls` dependency
- [ ] x25519 `getrandom` feature / panic `random()` keygen
- [ ] Epoch group KEK as primary wrap
- [ ] DataKey rotation rewrap job
- [ ] Repurpose `ai-brains sync` CLI
- [ ] `unwrap`/`expect`/panic in production
- [ ] `content_hash_sha256` integrity column

## Implementation order reminder

```
deps → crate types/crypto (RED→GREEN) → migration/store → key storage → CLI → gate → review
```

No implement until user go-ahead.
