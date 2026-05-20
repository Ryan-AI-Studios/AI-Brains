# Specification: Track T41 - Risk-Weighted Preflight & MADR Ingestion

## Objective
Wire structured MADR (Markdown Architectural Decision Records) into the nightly heartbeat and enable contextual, risk-weighted preflight by passing AI scope to ChangeGuard.

## Architecture & Scope
1. **Nightly MADR Ingestion (`ai-brains-cli`)**: Add an IPC call to ChangeGuard during nightly heartbeat to fetch structured MADR fields (title, context, decision, consequences). Format into MADR markdown locally and ingest as `Decision` nodes in the memory store — not flat JSON strings.
2. **Contextual Preflight (`ai-brains-cli`)**: Modify `preflight` to pass the AI's intended scope (target files/directories) to ChangeGuard via IPC, receiving a contextual risk-weighted impact packet with targeted cross-repo impacts, temporal coupling scores, and failure risk probabilities.

## Technical Constraints & Mandates
- **CQRS Integrity**: MADR ingestion must append events; preflight reads projections.
- **Event Sourcing**: MADR data ingested as immutable `Decision` events with full provenance.
- **Privacy Inheritance**: Decisions inherit privacy from their source BridgeRecord context.
- **Fail-Open**: If ChangeGuard IPC is unavailable during preflight, fall back to generic top-N brittle files.
- **Path Integrity**: All scope paths passed to ChangeGuard must be normalized via `ai-brains-path`.
