# T184 AC6 — Residuals ↔ CLAIMS-CROSSCHECK

**Date:** 2026-08-01  
**Inputs:** `../residuals.md`, `Docs/SECURITY-LIMITS.md`, `SECURITY.md`,  
`conductor/tracks/trackT183-release-documentation/evidence/CLAIMS-CROSSCHECK.md`

| Residual ID | Honesty / non-claim surface | Match? | Action |
|-------------|----------------------------|--------|--------|
| R-12 | SECURITY-LIMITS connectors / ADR-0019 TOCTOU | Yes | none |
| R-34.2 | ADR-0016 direction; not shipped | Yes | none |
| R-F8 | SECURITY-LIMITS §1 + COMPATIBILITY F8 | Yes | none |
| R-K06 | RECOVERY-DRILLS / F8 | Yes | none |
| R-CE-PRE | SECURITY-LIMITS CE section | Yes | none |
| R-WAL-CKPT | store honesty | Yes | none |
| R-ACK / R-META / R-PQ | SECURITY-LIMITS sync; ADR-0018 | Yes | none |
| R-HTTP-SYS | SECURITY-LIMITS §7 + OPERATIONS | Yes | none |
| R-MULTI / R-PIPE-IU | SECURITY-LIMITS §7 named-pipe row (T184) | Yes | added T184 |
| R-UDS-TMP | SECURITY-LIMITS §7 UDS row (T184) | Yes | added T184 |
| R-API-VER / R-BRIDGE / R-DTO-GOLDEN | SECURITY-LIMITS §8 protocol honesty | Yes | none |
| R-DOC-CLI | SECURITY-LIMITS §6 doctor/export absent | Yes | none |
| R-TB / R-CLOUDOK / R-EXTISM | SECURITY-LIMITS §5 TrustedBuiltin | Yes | none |
| R-OUTBOUND | prior honesty | Yes | leave |
| R-CHANGELOG-PATH | Packet root CHANGELOG.md | Yes | corrected F-10 |
| R-DISCLOSURE-TL | SECURITY.md 90-day section | **Closed** | F-9 fixed |
| R-CI-* closed/open | Process; not product marketing claims | Yes | T185/T186 |
| R-SLSA | T185 axis; not T184 cert | Yes | none |
| R-ZERO-KEY | under F8 | Yes | residual note |
| R-DESKTOP-OPEN | desktop residual | Yes | honesty |

**Forbidden marketing (CLAIMS-CROSSCHECK / SECURITY-LIMITS §9):** No T184 closeout text claims certification, perfect deletion, metadata-private sync, plugin sandbox, or live SQLCipher page encryption.

**Deltas filed as findings:** F-3/F-4 (docs) fixed; F-9 fixed; F-10 residual path corrected.

**Verdict:** AC6 **complete** — every residual has matching honesty or explicit process owner.
