# Track Completion Audit — T194-RecoveryKitSchema

## Verdict: FAIL

Crypto implementation requirements are substantially met, but required closure, provenance, Ledgerful verification, and cross-model SECURITY review evidence remain incomplete. No P3 deferral is proposed.

## Scope Reviewed

- `track/T194-recovery-kit-schema` against `origin/main` plus all unstaged changes.
- Full [`spec.md`](C:/dev/AI-Brains/conductor/tracks/trackT194-recovery-kit-schema/spec.md) and [`plan.md`](C:/dev/AI-Brains/conductor/tracks/trackT194-recovery-kit-schema/plan.md).
- Crypto implementation, tests, CLI consumers, docs, claims, and conductor state.
- No staged or untracked files were present.

## Requirement and DoD Matrix

| Requirement | Result | Evidence |
|---|---|---|
| F1–F4 schema placement/shape | Met | `PassphraseWrappedKey.kdf`, exact nested fields |
| F5–F6 algorithm/version validation | Met | Validation before KDF; `argon2id`, v19 only |
| F7 generation parameters | Met | 19456/2/1, v19 |
| F8 no `Argon2::default()` | Met | [`passphrase.rs:19-38`](C:/dev/AI-Brains/crates/ai-brains-crypto/src/passphrase.rs:19) |
| F9/F9b legacy dual-read | Met | `KdfParams::legacy()` |
| F10 schema version | Met | Remains 1 |
| F11 serialization | Met | Generation sets `kdf: Some(...)`; no skip attribute |
| F12/F21 deserialization | Met | `serde(default)`, `deny_unknown_fields`, partial-object tests |
| F13–F15 effective-param wiring/API | Met | Stored-or-legacy resolution and threaded params |
| F16 CLI surfaces | Met | Export, rotate, and doctor inherit library behavior |
| F17 dependencies | Met | No dependency diff; Argon2 remains 0.5.3 |
| F18 capture independence | Met | Crypto-only implementation |
| F19 secrets | Met | Existing no-leakage tests and unchanged stdout discipline |
| F20 K-07 inversion | Met | Presence/value assertions |
| F22 documentation honesty | Met in content | F37 references updated/struck |
| F23 contracts/events | Met | No DTO/event changes |
| F24 SECURITY review | Partial | Internal review exists; required cross-model review is not evidenced |
| F25 determinism/KAT | Met | Non-default stored-parameter KAT |
| F26 wrong-parameter failure | Met | Tampered `m_cost` fails closed |
| F27 DPAPI arm | Met | Unchanged |
| F28 forced re-export | Met | Correctly out of scope |
| F29 non-default stored params | Met | 12288/3/1 succeeds; legacy fails |
| F30 output length | Met | Fixed 32-byte output |
| F31 construction site | Met | Production construction updated |

### Acceptance criteria

- AC1–AC8: Met by implementation, tests, docs, and reported gates.
- AC9: Partial — full Ledgerful verification and cross-model SECURITY review are not evidenced.
- AC10: Met by shared `RecoveryKit::from_json`/unlock wiring plus new and legacy library tests.

### Plan DoD

- Phases A–C1–C3: checked and implemented.
- C4: unchecked.
- D1: unchecked.
- D2: unchecked.
- D3: unchecked; only internal review artifact exists.
- D4: unchecked; no ledger commit or decision pin is evidenced.

## Findings

### [P1] Required track closure and provenance are incomplete

Confidence: High

Requirement: AC9; plan C4, D1–D4; repository provenance rules.

Location: [`plan.md:12`](C:/dev/AI-Brains/conductor/tracks/trackT194-recovery-kit-schema/plan.md:12), [`plan.md:67-74`](C:/dev/AI-Brains/conductor/tracks/trackT194-recovery-kit-schema/plan.md:67), [`conductor.md:140`](C:/dev/AI-Brains/conductor/conductor.md:140)

Problem: The track remains `In Progress`; decision pinning, `ledgerful verify`, manual verification, cross-model SECURITY review, and ledger commit remain unchecked or unevidenced.

Evidence:

- Internal review is only `review.internal-r1.md`.
- `ledgerful doctor` and `ledgerful ledger status --compact` failed with `unable to open database file`.
- `cargo nextest run -p ai-brains-crypto` could not run because the read-only environment denied access to `target\debug\.cargo-lock`.
- User-supplied full-gate results are recorded as reported, not independently observed here.

Failure scenario: The track can be marked complete without its required SECURITY review, provenance record, final verification, or manual evidence.

Correction: Complete D1–D4, including Ledgerful verification/commit, decision pin, cross-model SECURITY review, manual export/legacy checks, canonical review closeout, and `conductor.md` status update.

Verification: All plan closure boxes checked with corresponding command/output evidence and no pending ledger transaction.

Deferrable: No

### [P2] Normative track metadata and closure claims are inconsistent

Confidence: High

Requirement: Governance/documentation agreement; plan C4/D4.

Location: [`spec.md:5,14`](C:/dev/AI-Brains/conductor/tracks/trackT194-recovery-kit-schema/spec.md:5), [`plan.md:3`](C:/dev/AI-Brains/conductor/tracks/trackT194-recovery-kit-schema/plan.md:3), [`conductor.md:140`](C:/dev/AI-Brains/conductor/conductor.md:140), [`deferred.md:18`](C:/dev/AI-Brains/conductor/deferred.md:18)

Problem: The spec still says “planning only — not implementing” and “plan-only (no TX until implement)”; the plan says In Progress with a transaction started; conductor status is In Progress; meanwhile deferred/claims documentation says F37 is closed by T194.

Correction: After actual closure, update the normative track status and ledger metadata consistently. Until then, avoid presenting the residual as fully closed.

Verification: Spec, plan, conductor, deferred, and review status all agree on the same completion state.

Deferrable: No

## Completeness Sweep

- No T194 production placeholders, stubs, fake paths, or no-op implementations found.
- No production `Argon2::default()`.
- No `skip_serializing_if` on `kdf`.
- No production `unwrap()`/`expect()` in the affected recovery path.
- Only the expected production `PassphraseWrappedKey` construction remains.
- No dependency or lockfile changes.

## Wiring and Regression Review

The production path is correctly wired:

`RecoveryKit::generate` → explicit KDF params → Argon2id derivation → `kdf` serialization → stored-param unlock, with legacy fallback for absent `kdf`.

Export, rotation, and doctor all route through `RecoveryKit`; DPAPI remains separate.

## Verification Evidence

Observed:

- `cargo fmt --check`: passed.
- `cargo metadata --locked --no-deps`: passed.
- `git diff --check`: passed.
- Source, tests, docs, and caller audit: passed.

Reported but not observed in this read-only environment:

- Workspace clippy, nextest 1841, cargo deny, cargo audit.
- Internal SECURITY review.

Blocked:

- Targeted nextest due filesystem read-only access.
- Ledgerful commands due unavailable database access.

## Deferred Candidates

None. The outstanding items are required completion work, not difficult non-blocking P3s.

## Completion Decision

The crypto/schema implementation is ready, but T194 is not independently clearable as complete until the P1/P2 closure and governance findings are resolved.