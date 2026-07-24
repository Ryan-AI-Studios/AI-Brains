# ADR-0010: Evolve AI-Brains Into the Successor Product

## Status

Accepted — 2026-07-23

AI-Brains already provides the proven Rust event store, CQRS projections, capture path, retrieval, graph, local-model integration, Windows/WSL path handling, daemon, harness adapters, and Ledgerful bridge needed by the proposed memory control plane. The successor will therefore evolve and extract AI-Brains crates behind versioned contracts, add a Tauri product surface and governed memory semantics, and migrate existing vaults with replay, shadow, rollback, and privacy verification. A clean rewrite was rejected because it would repeat solved platform and integration work while creating avoidable migration and correctness risk; retaining AI-Brains forever as an external backend was rejected because the successor must own its domain model and lifecycle.
