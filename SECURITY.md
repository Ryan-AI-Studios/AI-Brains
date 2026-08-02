# Security policy

Please read the product security limits and honest non-claims before reporting issues or making security claims:

**→ [Docs/SECURITY-LIMITS.md](Docs/SECURITY-LIMITS.md)**

That hub summarizes vault encryption honesty (F8), content-envelope erasure limits, optional multi-device replication, cloud defaults, connector trust (`TrustedBuiltin` only), recovery export (shipped T188), and missing `doctor` CLI.

Normative detail:

- [Docs/COMPATIBILITY.md](Docs/COMPATIBILITY.md) — platform tiers and F8 vault wording  
- [Docs/DECISIONS/ADR-0016-content-envelope-cryptography.md](Docs/DECISIONS/ADR-0016-content-envelope-cryptography.md)  
- [Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md](Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md)  
- [Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md](Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md)  
- [Docs/RECOVERY-DRILLS.md](Docs/RECOVERY-DRILLS.md)  

## Reporting

If you believe you have found a vulnerability in AI-Brains, please open a private report via GitHub Security Advisories for this repository (or contact the maintainers through the channel listed on the repository), and include:

1. Affected version / commit  
2. Reproduction steps  
3. Impact assessment  
4. Whether vault keys, recovery kits, or live production data were involved  

Do not file public issues with exploit details until coordinated disclosure.

## Coordinated disclosure timeline

Target: **90 days** from a confirmed fix landing in a released commit, **or** the next public release that includes the fix — whichever is **later** — before full public write-up of exploit details. Earlier disclosure may be agreed with the reporter. Critical issues may be summarized sooner without a weaponized PoC.

## Scope notes

- AI-Brains is licensed under PolyForm Noncommercial 1.0.0 with a small-entity commercial exception — see `LICENSE` and `COMMERCIAL-EXCEPTION.md`.  
- This project does **not** claim SOC2, ISO, or GDPR certification.  
- Default builds use **SQLCipher page-level encryption** (T187) plus application-level Content Envelope encryption; not FIPS / not NIST Purge (see SECURITY-LIMITS / COMPATIBILITY F8).
