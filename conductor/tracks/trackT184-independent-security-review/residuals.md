# T184 Residual Risk Register

**Status:** Post-review (2026-08-01 independent pass + remediations).  
**Normative process:** `spec.md` §6 / §9; charter §7; AGENTS.md Review & Severity.  
**Cross-check:** `evidence/RESIDUALS-CLAIMS-CROSSCHECK.md` (AC6).

| ID | Residual | Sev | Accept? | Owner / follow-up | Notes |
|----|----------|-----|---------|-------------------|-------|
| R-12 | Path TOCTOU without openat/cap-std | Med | Yes (doc) | Future path-hardening | #12; T154/T182 documented |
| R-34.2 | DataKey rotation / wrap-nonce budget | Med | Yes (dir only) | Future hygiene | ADR-0016/0018 direction; not implemented |
| R-F8 | Page-level SQLCipher not live | Med | Yes (honesty) | T183 elevated rewords | COMPATIBILITY F8 SOT; F-11 zero-key footgun under F8 |
| R-K06 | Wrong-key fail-closed needs page encrypt | Med | Yes (pre-exist) | Future SQLCipher feature | T181 residual |
| R-CE-PRE | Pre-erase backups remain recoverable | Med | Yes | T181 E-01 honesty | Physical fact |
| R-WAL-CKPT | WAL checkpoint ≠ NIST Purge | Info | Yes | Already honest in store | T181 E-16 class |
| R-ACK | Sync ACK ≠ wipe proof | Med | Yes | ADR-0018 / OPERATIONS | Attestation only |
| R-META | Sync metadata residual | Med | Yes | ADR-0018 L14 | Non-claim metadata-private |
| R-HTTP-SYS | LocalSystem HTTP token vs desktop | Low–Med | Yes | T161 residual | Documented OPERATIONS |
| R-MULTI | Multi-user interactive pipe residual | Med | Yes (doc) | Single-owner product model | **Elevated by F-1:** after fix, residual is **Interactive** SIDs (not World); still no pipe bearer |
| R-PIPE-IU | Pipe SDDL SY+BA+IU (not per-user SID) | Low–Med | Yes | Future per-user ACE optional | T184 F-1 remediation residual |
| R-UDS-TMP | UDS path under /tmp predictable | Low | Yes | Prefer HTTP multi-user Unix | T184 F-2: mode 0o600 after bind |
| R-API-VER | api_version unenforced runtime | Low–Med | Yes (honesty) | T180 F36 / T185 | Honesty T183/T185 |
| R-BRIDGE | Bridge capture policy doc-vs-code | Low | Yes | T180 F34 residual | Disposition open process |
| R-DTO-GOLDEN | DTO goldens / API_VERSION SOOT gaps | Low | Yes | T180 F35 residual | |
| R-DOC-CLI | No doctor / recovery export CLI | Low | Yes | Ops residual | T181/T183 honesty |
| R-TB | TrustedBuiltin shares process | Med | Yes (design) | ADR-0019 L1 | Not a bug |
| R-CLOUDOK | CloudOk unused; no trust-label gate | Low | Yes | Future flag | T182 residual |
| R-EXTISM | Wasmtime/Extism patch-lag class | Info | OOS v1 | ADR-0019 forbids host | Document deferral only |
| R-OUTBOUND | OutboundIndex empty in prod | Low | Yes | T156 honesty | Rule3 Unknown |
| R-PQ | Post-quantum not claimed | Info | Yes | ADR-0018 L16 | Non-claim |
| R-STATUS-STALE | status.md historical demote residual | Low | Yes | T183 demote | Soft; re-confirm |
| R-CHANGELOG-PATH | CHANGELOG is **repo root** `CHANGELOG.md` | Info | Yes | Packet uses root | Corrected from inverted seed (F-10) |
| R-DISCLOSURE-TL | SECURITY.md numeric disclosure timeline | Low | **Closed** | SECURITY.md 90d | T184 F-9 fixed |
| R-CI-PERM | CI workflow `permissions:` least-privilege | Med | **Closed** | ci.yml `contents: read` | T184 F-5 fixed |
| R-CI-PIN | Actions pinned to major tag not SHA | Med | Open | **T186** | Scorecard Pinned-Dependencies; F-6 deferred |
| R-CI-DEPBOT | Dependabot/Renovate config | Med | **Closed** | `.github/dependabot.yml` | T184 F-7 fixed |
| R-CI-SAST | No dedicated SAST (clippy ≠ SAST) | Med | Yes (honesty) | Optional later | Scorecard SAST class |
| R-CI-BRANCH | Branch protection not enabled | Low | Open | **Repo admin** | Verified 404 at execute; F-8 |
| R-SLSA | No SLSA provenance claim today | Info | Yes | **T185** scope | v1.2 current; not T184 cert lock |
| R-ZERO-KEY | Daemon default all-zero vault key env | Low | Yes under F8 | Future SQLCipher refuse-missing | F-11 |
| R-DESKTOP-OPEN | Desktop opener path `**` residual | Low | Yes | Desktop README honesty | F-12 |
| R-AUDIT-UNMAINT | audit unmaintained transitive warnings | Info | Yes | T185/T186 | F-13; gate still green |

**Post-review notes:** F-1 High remediated (World → SY+BA+IU). No open Critical/High product findings. Cite residual IDs in T185 claims language.
