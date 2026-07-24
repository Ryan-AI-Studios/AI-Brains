# ADR-0015: Combine an Append-Only Ledger With Cryptographic Erasure and Encrypted Event Replication

## Status

Accepted — 2026-07-23

The authoritative internal history remains an append-only event ledger with fast materialized projections, but sensitive payloads use separable encryption keys so deletion can make content unrecoverable while retaining only minimal safe tombstones. Optional multi-device sync replicates end-to-end encrypted event envelopes through an untrusted relay and rebuilds projections locally; it does not synchronize mutable SQLite files. This preserves provenance and conflict history without using append-only semantics as an excuse for permanent sensitive-data retention.
