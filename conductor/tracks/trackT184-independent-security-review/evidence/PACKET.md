# T184 Reviewer Packet Index

**Date:** 2026-08-01  
**Charter:** `../charter.md` (Frozen)  
**Scope freeze:** Sync = **Y**, Desktop = **Y** (T2 matrix honesty)  
**Secrets policy:** No live vault paths. Temp vaults only if needed.  

## 1. Charter & governance

| # | Artifact | Path |
|---|----------|------|
| 1 | Charter (frozen) | `conductor/tracks/trackT184-independent-security-review/charter.md` |
| 2 | Spec | `conductor/tracks/trackT184-independent-security-review/spec.md` |
| 3 | Plan | `conductor/tracks/trackT184-independent-security-review/plan.md` |
| 4 | Residual seed | `conductor/tracks/trackT184-independent-security-review/residuals.md` |

## 2. ADRs & threat models

| # | Artifact | Path |
|---|----------|------|
| 5 | ADR-0011 / 0012 (if present) | `Docs/DECISIONS/` |
| 6 | ADR-0016 CE cryptography | `Docs/DECISIONS/ADR-0016-content-envelope-cryptography.md` |
| 7 | ADR-0018 sync protocol | `Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md` |
| 8 | ADR-0019 connector sandbox | `Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md` |
| 9 | T175 threat model | `conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md` |
| 10 | T182 threat model (+ T184 hooks §9) | `conductor/tracks/trackT182-connector-sandbox-decision/threat-model.md` |

## 3. Honesty & release pack (T183)

| # | Artifact | Path |
|---|----------|------|
| 11 | Security limits hub | `Docs/SECURITY-LIMITS.md` |
| 12 | Root security policy | `SECURITY.md` |
| 13 | CLAIMS-CROSSCHECK | `conductor/tracks/trackT183-release-documentation/evidence/CLAIMS-CROSSCHECK.md` |
| 14 | Install honesty | `Docs/INSTALL.md` |
| 15 | Changelog (**repo root**) | `CHANGELOG.md` |
| 16 | Compatibility F8 | `Docs/COMPATIBILITY.md` |
| 17 | Protocol compat | `Docs/PROTOCOL-COMPAT.md` |
| 18 | Recovery drills | `Docs/RECOVERY-DRILLS.md` |
| 19 | Operations (erasure / multi-device) | `Docs/OPERATIONS.md` |

## 4. Automated baseline

| # | Artifact | Path / command |
|---|----------|----------------|
| 20 | Deny log | `evidence/DENY-AUDIT.md` + raw outputs |
| 21 | Audit log | same |
| 22 | How to re-run | See DENY-AUDIT.md |

## 5. Hotspot priority (ledgerful hotspots 2026-08-01)

| Rank | Path |
|------|------|
| 1 | `crates/ai-brains-cli/src/commands/sync.rs` |
| 2 | `crates/ai-brains-cli/src/commands/context.rs` |
| 3 | `crates/ai-brains-cli/src/commands/forget.rs` |
| 4 | `crates/ai-brains-cli/src/commands/project.rs` |
| 5 | `crates/ai-brains-cli/src/commands/daemon.rs` |
| 6 | `crates/ai-brains-control-plane/src/review.rs` |
| 7 | `crates/ai-brainsd/src/main.rs` |
| 8–10 | pin, governed_common, store migrations |

Also review: crypto CE paths, connector reparse refuse, models `allow_cloud`, `.github/workflows/ci.yml`.

## 6. Test pointers (elevate, do not re-prove entire suite)

```powershell
cargo deny check
cargo audit
cargo nextest run -p ai-brains-sync -- security
cargo nextest run -p ai-brains-store -- content_envelope
cargo nextest run -p ai-brains-sources -- connector_contract
cargo nextest run -p ai-brains-security
```

## 7. Pre-hand-off secret scan attestation

**Command (PowerShell):**

```powershell
$paths = @(
  'conductor/tracks/trackT184-independent-security-review/charter.md',
  'conductor/tracks/trackT184-independent-security-review/residuals.md',
  'conductor/tracks/trackT184-independent-security-review/evidence/PACKET.md',
  'Docs/SECURITY-LIMITS.md',
  'SECURITY.md',
  'CHANGELOG.md'
)
$patterns = 'BEGIN (RSA |OPENSSH |EC )?PRIVATE KEY','sk-[A-Za-z0-9]{20,}','ghp_[A-Za-z0-9]{36}','xox[baprs]-','AKIA[0-9A-Z]{16}','password\s*=\s*[''"][^''"]{8,}'
foreach ($p in $paths) {
  if (Test-Path $p) {
    Select-String -Path $p -Pattern $patterns -AllMatches -ErrorAction SilentlyContinue
  }
}
```

**Result (2026-08-01):** No matches on packet files listed above. **No secrets found.**

## 8. Forbidden in packet

- Live production vault paths  
- Real recovery kits / DPAPI material  
- Weaponized open Critical PoCs (sanitize until fixed)
