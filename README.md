# AI-Brains

AI-Brains is an event-sourced, privacy-first memory system for AI agents, optimized for Windows 11 and PowerShell.

## Core Mandate
Capture must be fast, durable, encrypted, and independent of every advanced memory feature. The system ensures that your project history is never lost, even if intelligence services are offline.

## Key Features
- **Canonical Event Log**: SQLCipher-backed append-only history.
- **CQRS Architecture**: Commands append events; queries read read-optimized projections.
- **Privacy First**: Four levels of privacy protection (`CloudOk` to `Sealed`).
- **Nightly Intelligence**: Background workers for summarization, conflict detection, and recipe promotion.
- **Windows Native**: First-class support for PowerShell, DPAPI, and Task Scheduler.

## Quick Start

### Build
```powershell
cargo build --release
```

### Initialize Vault
```powershell
./target/release/ai-brains --vault-path ./vault.db init
```

### Ingest a Turn
```powershell
echo '{"session_id":"...", "project_id":"...", "harness_id":"...", "turn_id":"...", "role":"user", "content":"hello", "privacy":"CloudOk"}' | ./target/release/ai-brains --vault-path ./vault.db ingest
```

### Recall
```powershell
./target/release/ai-brains --vault-path ./vault.db recall "hello"
```

## Documentation
- [Architecture](./Docs/ARCHITECTURE.md)
- [Operations Guide](./Docs/OPERATIONS.md)
- [Project Status](./Docs/status.md)
- [Implementation Plan](./Docs/Implementation-Plan.md)
- [Architectural Deviations](./Docs/Deviations.md)

## Development
This project uses a track-based implementation method managed via ChangeGuard.
```powershell
./scripts/dev-check.ps1
```
