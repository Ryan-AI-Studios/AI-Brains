# T187 Review Log — SQLCipher Page Encryption

## Scope
Live SQLCipher page encryption for default production builds: feature flip, open/backup/zero-key/migrate, strict recovery drills, claims/docs, CI Perl prereq.

## Reviewers / rounds
| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| Internal R1 | general-purpose subagent | **FAIL** | Zero-key hermeticity, RECOVERY-DRILLS stale, deferred #8 not struck |
| Internal R1 fix | orchestrator | fixed | GHA env, fixture TempEnv, docs, deferred §65, path-collision test fix |
| Internal R2 | orchestrator | **PASS WITH DEFERRED P3** | Easy P2s fixed; soft residuals recorded |
| Codex R1 | codex gpt-5.4 | **FAIL** | AC8 doc drift (README/SECURITY/Docs/README/WORKFLOWS) |
| Codex R2 | codex gpt-5.4 | **PASS WITH DEFERRED P3** | AC8 fixed; cipher_version recorded post-review |

## DoD / AC (final)
| AC | Status | Evidence |
|----|--------|----------|
| AC1 SQLCipher features | Met | `Cargo.toml` bundled-sqlcipher-vendored-openssl |
| AC2 header not plain | Met | unit `open__new_vault_header_not_plain_after_write` |
| AC3 wrong-key fail-closed | Met | connection + recovery drills strict |
| AC4 remove if plain | Met | store+cli recovery_drills |
| AC5 LegacyPlaintext + vault encrypt | Met | header.rs + encrypt.rs + vault CLI |
| AC6 backup keyed source | Met | run_backup apply_key_pragmas |
| AC7 zero-key refuse | Met | VaultConnection + TempEnv tests + GHA/dev-check env |
| AC8 docs/claims | Met | Deviations §1, COMPAT F8, RELEASE-CLAIMS, SECURITY-LIMITS, RECOVERY-DRILLS |
| AC9 full gate | Met | nextest 1725 pass; clippy -D warnings; deny ok; audit exit 0 |
| AC10 deferred #8 struck | Met | deferred.md §59 #8 + §65 |
| AC11 cipher_version | Met | unit smoke |
| AC12 validate/is_zero | Met | sqlcipher.rs |
| AC13 scratch hygiene | Met | check_db/check_vault removed |

## Findings disposition
| ID | Severity | Disposition |
|----|----------|-------------|
| P0-1 zero-key hermetic | P0 | Fixed: GHA `AI_BRAINS_ALLOW_ZERO_KEY`, dev-check, fixture TempEnv, hermetic_bin |
| P1-1 RECOVERY-DRILLS stale | P1 | Fixed |
| P1-2 deferred #8 | P1 | Struck |
| P2-1 COMPAT non-claims | P2 | Fixed |
| P2-2 GHA Perl | P2 | Fixed (PATH ensure step) |
| P2-3 deny archive | P2 | deny-check.txt in track dir |
| Test path collision | High (test) | Fixed: separate vault parents for dual-key drill |

## Deferred (P3 only)
- `cipher_integrity_check` on backup verify (spec L1 / out of scope)
- R-ZERO-KEY escape-hatch honesty remains (documented)
- #34.2 DataKey rotation → T189

## Gate results
- `cargo fmt --check` pass
- `cargo clippy --workspace --all-targets -- -D warnings` pass
- `cargo nextest run --workspace --profile ci` → **1725 passed**, 1 skipped
- `cargo deny check` → advisories/bans/licenses/sources ok
- `cargo audit` → exit 0 (allowlisted warnings only, pre-existing)
- `ledgerful verify --scope full` → Verification passed

## Pin
`DECISION: T187 SQLCipher live via bundled-sqlcipher-vendored-openssl; plain→encrypt via sqlcipher_export (vault encrypt); wrong-key fail-closed; zero-key refuse unless AI_BRAINS_ALLOW_ZERO_KEY; cipher_version 4.10.0 community; not FIPS/Purge`
