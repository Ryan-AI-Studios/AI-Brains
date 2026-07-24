# T146 — Governed Memory Control Plane Architecture

- **Track ID:** T146-GovernedMemoryControlPlaneArchitecture
- **Execution repo:** `C:\dev\ai-brains`
- **Status:** Complete — deliverables complete; full repository gate green on native Windows (2026-07-24); see `review.md`
- **Category:** ARCHITECTURE / DOCS
- **Implementation:** Documentation and research only; no runtime or schema changes

## 1. Objective

Record the shared domain model, hard-to-reverse architectural decisions, research comparison, evaluation rubric, and evolutionary path from AI-Brains to a local-first governed memory control-plane product.

## 2. Context

AI-Brains already provides an event-sourced Rust memory system with CQRS projections, harness capture, local retrieval, graph augmentation, a daemon, and Ledgerful integration. The successor is not a clean rewrite. It adds explicit authority and lifecycle semantics, source-driven freshness, distinct Project and Personal briefings, governed progressive retrieval, stronger privacy/deletion, a Tauri application, a public protocol, and optional encrypted event replication.

The design was developed through a one-question-at-a-time `grill-with-docs` session on 2026-07-23. The user confirmed the product scope and desired outcomes; the detailed architecture records the resulting recommendations and their rationale.

## 3. In scope

1. Add a root `CONTEXT.md` glossary containing domain language only.
2. Add a consolidated product-vision document.
3. Add concise ADRs for the genuinely hard-to-reverse decisions.
4. Source-audit and compare the Cerebras external context pattern, Karpathy-style LLM Wiki/Obsidian, Nate Jones's second-brain system, Mem0, and the existing AI-Brains/Honcho/Obsidian stack as of July 2026.
5. Define a trust-first scoring rubric and clearly state score direction and uncertainty.
6. Link the new documents from repository navigation.
7. Produce a harness-ready implementation plan with exact phases, file targets, tests, migrations, rollback gates, and stop conditions.

## 4. Out of scope

- Runtime, schema, CLI, daemon, Tauri, MCP, connector, migration, or sync implementation.
- Product naming, branding, pricing, or licensing changes.
- Live production-vault migration or writes.
- Modification of the unrelated pre-existing `.agents/skills/codex-review/SKILL.md` worktree change.
- Merge, push, or release.

## 5. Accepted architecture

- Evolutionary AI-Brains successor with versioned vault migration.
- Standalone local-first control plane with optional Ledgerful and Hermes integrations.
- Rust service plus Tauri, CLI, versioned IPC/HTTP/NDJSON, and MCP adapter.
- Evidence → Conclusion → Decision authority model.
- Risk-tiered conclusion lifecycle and protected approval categories.
- Append-only event ledger plus materialized projections.
- Source-change invalidation and fallback revalidation.
- Progressive governed retrieval with continuation handles and traces.
- Distinct Project and Personal briefing schemas.
- Repository → Workspace → explicit personal/global grants.
- Structured action evidence; no hidden chain-of-thought; raw logs opt-in.
- Claim-preserving temporal and authority-aware conflict handling.
- Class-based retention, cryptographic erasure, and encrypted event replication.
- One human owner with multiple attributable agents and team-ready principals.
- Windows 11/WSL first with a portable Rust core.
- Dogfood-first but distributable.

## 6. Definition of Done

- [x] `CONTEXT.md` exists and contains no implementation details.
- [x] Product vision records every accepted decision and the north-star scenario.
- [x] ADR-0010 through ADR-0015 are present and internally consistent.
- [x] Comparative research cites primary sources, distinguishes verification from inference, and states the July 2026 cutoff.
- [x] Scores define whether a higher number is better and avoid false precision.
- [x] Repository documentation links resolve.
- [x] `.hermes/plans/2026-07-23_204630-memory-control-plane-successor.md` covers phases P0–P12 and passes structural validation.
- [x] Markdown/link checks pass.
- [x] `ledgerful verify` reports no drift or unaccounted architecture changes.
- [x] Git diff contains only intended T146 files plus the pre-existing unrelated modification, which remains unstaged and untouched.
- [x] Outcome recorded in `review.md`; registry status updated accurately; Ledgerful transaction committed only after verification.

## 7. Verification

```powershell
ledgerful verify

git diff --check

# Resolve relative markdown links with the repository's documentation checker
# or a deterministic local script if no checker exists.

git status --short
```
