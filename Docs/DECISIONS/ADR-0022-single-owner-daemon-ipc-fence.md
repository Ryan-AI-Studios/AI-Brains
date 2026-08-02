# ADR-0022: Single-Owner Daemon IPC Fence

## Status

**Accepted** — 2026-08-02.

Normative for **T195** (Daemon / Multi-User Residuals). Complements
[ADR-0018](ADR-0018-encrypted-event-replication-protocol.md) **L15** (multi-user
**vault** sharing out of scope) with an explicit **IPC** product fence: the
daemon local transport model is **single-owner desktop / single-vault**, not a
multi-user or multi-tenant product.

Freezes: **F1–F35**, acceptance **AC1–AC11** — see
[`conductor/tracks/trackT195-daemon-multiuser-residuals/spec.md`](../../conductor/tracks/trackT195-daemon-multiuser-residuals/spec.md).

## Context

Post-T161 (loopback HTTP + bearer) and T184 (pipe SDDL SY+BA+IU; UDS 0o600), four
honesty residuals remained:

| Residual | Risk class |
|----------|------------|
| **R-PIPE-IU** | Any Interactive logon can open the named pipe; no per-user pipe bearer |
| **R-UDS-TMP** | Predictable `/tmp` UDS path / bind-race residual |
| **R-HTTP-SYS** | LocalSystem service HTTP token under SYSTEM profile ≠ desktop-readable |
| **R-MULTI** | Umbrella: no multi-user pipe auth product |

Without a fence, marketing could overclaim “multi-user safe IPC.” Without
mitigations, operators had no opt-in tighten path and no honest refuse for
service HTTP.

## Decision

### 1. Product fence (F1 / F12)

Primary model remains **single-owner desktop / single-vault**.

T195 does **not** ship:

- Multi-user federation / multi-tenant vault
- OAuth / IdP
- Per-user pipe SID or per-user pipe bearer as default
- Shared multi-session HTTP token under ProgramData

**Multi-user product requires a future ADR** (same posture as ADR-0018 L15 for
vault sharing).

### 2. Residual dispositions (F2 / §5)

| Residual | Disposition | Mechanism |
|----------|-------------|-----------|
| **R-PIPE-IU** | **(b) opt-in harden + residual** | Default SDDL keeps **IU**; `AI_BRAINS_PIPE_ACL=service-only` → SY+BA only |
| **R-MULTI** | **(c) permanent fence** | This ADR + claims honesty; no multi-user pipe auth claim |
| **R-UDS-TMP** | **(a)/(b) mitigate + residual** | Shared XDG resolver + env override + pre-bind/shutdown ownership hygiene; `/tmp` fallback residual |
| **R-HTTP-SYS** | **(a)/(b) mitigate + residual** | Service host refuses HTTP unless `AI_BRAINS_HTTP_SERVICE` truthy; residual when opted in |

### 3. Technical locks (summary)

| Area | Lock |
|------|------|
| Pipe name | `\\.\pipe\ledgerful-bridge` **unchanged** (ledgerful interop) |
| Default pipe SDDL | `D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)` |
| Optional pipe ACL | `AI_BRAINS_PIPE_ACL=interactive` (default) \| `service-only` |
| UDS resolver SOOT | `ai_brains_daemon_api::resolve_daemon_socket_path` (daemon + CLI) |
| UDS order | `AI_BRAINS_DAEMON_SOCKET` (absolute) → valid `XDG_RUNTIME_DIR` → `/tmp` + warn |
| UDS mode | Post-bind **0o600** |
| UDS pre-bind/shutdown | Unlink only socket + euid-owned (no libc/nix direct dep) |
| Service HTTP | Gate in `windows_service` only; interactive `maybe_start_http` unchanged |
| Wire `principal_id` | **Policy label**, not IPC authentication |

### 4. Forbidden claims

Do **not** claim:

- “Multi-user safe”
- “Per-user pipe isolation”
- “Service HTTP ready for desktop clients”
- “UDS TOCTOU-closed under `/tmp`”

### 5. Relationship to other ADRs

| ADR | Relationship |
|-----|----------------|
| ADR-0018 L15 | Vault multi-user out of scope; this ADR fences **daemon IPC** the same way |
| ADR-0019 | TrustedBuiltin / no third-party sandbox claim — orthogonal |
| Future multi-user ADR | Per-user SID, pipe bearer, shared Session0/1 token — only with new design |

## Consequences

- Operators who need tighter pipe ACL on multi-user Windows hosts can set
  `AI_BRAINS_PIPE_ACL=service-only` and accept interactive CLI **NotRunning**
  against a SYSTEM service pipe (document in OPERATIONS).
- Unix hosts with valid `$XDG_RUNTIME_DIR` bind/connect under the runtime dir;
  prior `/tmp`-hardcoded external clients need `AI_BRAINS_DAEMON_SOCKET` on both
  sides (CHANGELOG migration note).
- Service installs do not expose HTTP by default even if `AI_BRAINS_HTTP=1`;
  opt-in `AI_BRAINS_HTTP_SERVICE=1` keeps the LocalSystem token residual honest.

## References

- Spec: `conductor/tracks/trackT195-daemon-multiuser-residuals/spec.md`
- Claims: [RELEASE-CLAIMS.md](../RELEASE-CLAIMS.md) residual rows
- Limits: [SECURITY-LIMITS.md](../SECURITY-LIMITS.md) §7
- Ops: [OPERATIONS.md](../OPERATIONS.md) env table + service notes
