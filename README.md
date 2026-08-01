# AI-Brains

AI-Brains is an event-sourced, privacy-first memory system for AI agents, optimized for Windows 11 and PowerShell.

## Core Mandate
Capture must be fast, durable, encrypted, and independent of every advanced memory feature. The system ensures that your project history is never lost, even if intelligence services are offline.

## Key Features
- **Canonical Event Log**: Append-only history in **bundled SQLite**, with application-level **Content Envelope AES-256-GCM** for sensitive payloads and OS filesystem permissions. **SQLCipher page-level encryption is feature-gated / not live** on the default build — see [Docs/COMPATIBILITY.md](./Docs/COMPATIBILITY.md) (F8).
- **CQRS Architecture**: Commands append events; queries read read-optimized projections.
- **Privacy First**: Four levels of privacy protection (`CloudOk` to `Sealed`).
- **Nightly Intelligence**: Background workers for summarization, conflict detection, and cross-agent synthesis (Phase 15).
- **Windows Native**: First-class support for PowerShell, DPAPI, and Task Scheduler.

## Quick Start

### 1. Initialize a Vault
```powershell
ai-brains init
```

### 2. Set Up a Project
Run this in any new repository to wire up project-specific isolation:
```powershell
ai-brains context
```

### 3. Record a Turn
```powershell
powershell .agents/skills/ai-brains/scripts/ingest.ps1 -Content "Finalizing Phase 15..."
```

### 4. Start a Session (Preflight)
```powershell
ai-brains preflight
```
*Returns an indexed briefing followed by recent technical context.*

## 📂 Configuration Hierarchy
AI-Brains uses a hierarchical loading strategy for cross-repository flexibility:
1.  **Local `.env`**: Scopes IDs to the current repo (Created via `context`).
2.  **Global `~/.ai-brains/.env`**: Stores the shared `VAULT_PATH` and Model URLs.
3.  **Env Vars**: Override any of the above.

## Documentation

**Start here:** [Docs/README.md](./Docs/README.md) — documentation index (install, Diátaxis map, seven release topics, non-claims).

| Doc | Role |
|-----|------|
| [Install & first vault](./Docs/INSTALL.md) | Windows-first how-to |
| [Security limits](./Docs/SECURITY-LIMITS.md) · [SECURITY.md](./SECURITY.md) | Honest non-claims hub |
| [Capabilities](./Docs/CAPABILITIES.md) | Feature inventory |
| [Architecture](./Docs/ARCHITECTURE.md) | System explanation |
| [Operations](./Docs/OPERATIONS.md) | Day-to-day ops reference |
| [Compatibility](./Docs/COMPATIBILITY.md) | OS tiers + F8 vault encryption SOT |
| [Protocol compat](./Docs/PROTOCOL-COMPAT.md) | Wire / N−1 honesty |
| [Recovery drills](./Docs/RECOVERY-DRILLS.md) | Backup / kit / CE drills |
| [Workflows](./Docs/WORKFLOWS.md) | Recipes |
| [Changelog](./CHANGELOG.md) | Keep a Changelog |
| [Domain Language](./CONTEXT.md) | Glossary |
| [Control-plane vision](./Docs/MEMORY-CONTROL-PLANE-VISION.md) | Product vision |
| [Implementation Plan](./Docs/Implementation-Plan.md) | Historical master plan (CLI §8 may drift) |
| [status.md](./Docs/status.md) | **Historical** freeze only — live status = `conductor/conductor.md` |
| [Deviations](./Docs/Deviations.md) | Architectural departures |
| [Research comparison](./Docs/RESEARCH/memory-systems-comparison-2026-07.md) | Historical research |

## License

AI-Brains is licensed under the **PolyForm Noncommercial License 1.0.0** (`LICENSE`), with additional permissions for qualified small entities in `COMMERCIAL-EXCEPTION.md`. Contact: legal@ledgerful.dev.

## Development
This project uses a track-based implementation method managed via Ledgerful.
```powershell
./scripts/dev-check.ps1
```
