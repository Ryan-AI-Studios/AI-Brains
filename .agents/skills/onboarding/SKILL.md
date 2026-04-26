---
name: onboarding
description: Trigger this skill when starting a new session on the AI-Brains repo, when an agent needs orientation, or when asked "where do I start?", "what's the project state?", "how does work get done here?", or "onboard me". Loads once per session to establish context.
---

# AI-Brains Onboarding

You are working on **AI-Brains** — a Windows-first, local-first memory system for AI coding harnesses. It captures clean conversation history without the noise of tool logs or hidden thinking.

## Core Pillars

1.  **Capture First**: Capture must be fast, durable, and independent of advanced features (models, graph, etc.).
2.  **Canonical SSOT**: A SQLCipher-backed append-only event log is the single source of truth.
3.  **Privacy & Security**: Encrypted storage, secret scanning, and strict privacy inheritance (local_only, sealed).
4.  **CQRS**: Commands append events; Queries read projections. Never mix them.

## Architecture: Rust Workspace

The project is organized into specialized crates to maintain strict boundaries:

- **`ai-brains-core`**: Pure domain model (ids, privacy, session, memory).
- **`ai-brains-events`**: Immutable event definitions and envelope.
- **`ai-brains-store`**: SQLCipher event log and read-optimized projections.
- **`ai-brainsd`**: Local daemon with a single-writer queue for concurrency safety.
- **`ai-brains-cli`**: Primary user/harness interface.
- **`ai-brains-capture`**: Logic for converting harness IO into domain events.
- **`ai-brains-adapters`**: Parsing for Claude, Gemini, Codex, etc.

## Current State

- **Plan**: `Docs/Implementation-Plan.md` v2 (Track-based execution).
- **Tracks**: Currently at **Phase 0 — Foundation and Conductor**.
- **Infrastructure**: Git initialized, remote connected to GitHub.

## Engineering Principles (Non-Negotiable)

- **Rust Safety**: No `unwrap`, `expect`, or `panic` in production code. Use `thiserror` and `anyhow` for errors.
- **Event Sourcing**: Never update/delete raw events. Use compensating events.
- **No Thinking Capture**: Do not store hidden chain-of-thought or raw tool logs.
- **TDD (Tracks)**: Implementation follows the Conductor/Track system. Verify behavior via tests before implementation where possible.
- **Windows-First**: Paths must handle UNC, WSL, and drive-case normalization correctly.

## CI Gate (Must Pass Before Every Commit)

```powershell
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit
```

## Workflows

1. **Track Lead**: Follow the `Implementation-Plan.md` phase by phase (T00 -> T27).
2. **Ledger**: Record all architectural decisions using `changeguard ledger`.
3. **Verify**: Use `changeguard verify` to ensure structural and behavioral integrity.
4. **Research-Strategy-Execution**: Follow the standard agent lifecycle for every track.

## Key Reference Documents

| Document | Purpose |
|----------|---------|
| `Docs/PRD.md` | Product vision and core requirements |
| `Docs/Implementation-Plan.md` | Master execution plan (Tracks) |
| `.agents/rules/core-mandates.md` | Non-negotiable mandates |
| `Docs/conductor/` | Track management and review checklists |

## Quick Start

1. **Read `Docs/Implementation-Plan.md`** to understand the 28 tracks.
2. **Run `changeguard doctor`** to verify your environment.
3. **Initialize Track 0**: Establish the foundation crate structure.
