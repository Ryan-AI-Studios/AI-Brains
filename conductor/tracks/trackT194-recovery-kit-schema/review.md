# T194 Review Log — Recovery Kit Schema Hygiene

**Track:** T194-RecoveryKitSchema  
**Category:** SECURITY / CRYPTO  
**Orchestrator:** Grok  
**Date:** 2026-08-02  
**Ledger TX:** `e8844831-decb-43c8-8850-e338dea1ba26`

## Scope

Pin Argon2id KDF parameters into RecoveryKit JSON (`passphrase.kdf`); dual-read pre-T194 kits via fixed `KdfParams::legacy()`; never use `Argon2::default()`; F29 non-default params proof; invert T181-K-07; docs honesty for F37.

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| Internal r1 | Subagent read-only | **PASS WITH DEFERRED P3** | No P0–P2; P3 edge tests + process residual |
| Internal fix | Orchestrator | P3-1 edge tests added | t/p caps, zero costs, m&lt;8p, deny_unknown_fields |
| Codex r1 | gpt-5.6-luna high | **FAIL** (process only) | P1 closure incomplete mid-flight; P2 spec metadata “planning only” vs implement |
| Codex r1 fix | Orchestrator | P2 fixed | spec/plan/deferred status aligned; D1–D3 evidenced |
| Codex final r2 | gpt-5.6-luna high | **PASS WITH DEFERRED P3** | No P0–P2; ship-process C4/D4 + optional CLI kdf assert only |

## DoD matrix (engineering)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 embed kdf | **Met** | `recovery_kit__generate__embeds_kdf_params` |
| AC2 non-default stored | **Met** | F29 test m=12288,t=3,p=1; LEGACY fails |
| AC3 legacy omit kdf | **Met** | legacy JSON unlock tests |
| AC4 no Argon2::default | **Met** | production path + structural test |
| AC5 fail closed | **Met** | caps/algorithm/version/partial/unknown field |
| AC6 invert K-07 + CLI | **Met** | inverted tests; export/rotate/doctor inherit |
| AC7 docs F37 | **Met** | RECOVERY-DRILLS, ADR-0020, RELEASE-CLAIMS, CAPABILITIES, deferred |
| AC8 deps | **Met** | argon2 0.5.3 hold; no new deps |
| AC9 gate + review | **Met** (engineering) | full gate green; Codex r2 PASS WITH DEFERRED P3; C4/D4 after merge |
| AC10 doctor kit | **Met** | library dual-read; doctor uses from_json+unlock |

## Findings disposition

### Internal r1 P3-1 (edge tests) — **verified_fixed**
Added unit tests for t_cost/p_cost caps, zero costs, m&lt;8p; integration deny_unknown_fields.

### Internal r1 P3-2 (process) — **open until ship**
C4/D4 Completed + pin + ledger commit after PR merge.

### Internal r1 P3-3 (CLI kdf assert) — **deferred** (optional plan C1)
Library AC1 sufficient; optional CLI wire assert not required.

### Codex r1 P1 (closure incomplete) — **partly valid / process**
Engineering DoD met; track intentionally not Completed until PR+CI+merge. Disposition: complete ship checklist after merge.

### Codex r1 P2 (metadata inconsistency) — **verified_fixed**
spec.md status updated from “planning only”; plan status Implementation complete; deferred F37 wording notes PR pending.

## Gates observed (orchestrator)

```
cargo fmt --check / cargo fmt OK
cargo clippy --workspace --all-targets -- -D warnings OK
cargo nextest run --workspace → 1841 passed, 1 skipped
cargo nextest run -p ai-brains-crypto → 74 passed
cargo deny check OK
cargo audit → allowed warnings only (pre-existing)
```

## Residual (post-ship deferred candidates)

| Item | Severity | Notes |
|------|----------|-------|
| Optional CLI export assert kdf | P3 | Plan C1 optional |
| Future strength bump track | out of scope | F7 holds 19456/2/1 |
| Typed algorithm enum (O1) | declined | String + F5 |
| Lower interactive DoS caps (O2) | declined | F14 1 GiB enough |

## Completion decision

**Engineering clearance: YES** (Codex r2 PASS WITH DEFERRED P3).  
**Shipped:** PR #76 squash-merged `2c06464`; closeout `a9a4168`; C4/D4 done; pin + ledger committed; coordinated deferred updated.
