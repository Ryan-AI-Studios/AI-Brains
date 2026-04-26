---
name: onboarding
description: Trigger this skill when starting a new session on the CrawlX repo, when an agent needs orientation, or when asked "where do I start?", "what's the project state?", "how does work get done here?", or "onboard me". Loads once per session to establish context.
---

# CrawlX Onboarding

You are working on **CrawlX** — a self-hosted, local-first crawl-operations platform that approximates paid Firecrawl capabilities with policy-controlled automation, multi-engine waterfall scraping, and agentic research.

## Core Pillars

1.  **Security First**: Triple-layer SSRF protection (URL validator, DNS guard, network isolation) is a hard prerequisite for any scraping.
2.  **Durable Orchestration**: Every scrape is a durable job with content-addressed artifact persistence and failure classification.
3.  **Waterfall Engine**: Intelligent fallback ladder (Static -> JS -> Playwright -> Branded Browser -> Cloud Escalation).

## Architecture: Monorepo Structure

- **`apps/api`**: Fastify API providing a Firecrawl v2.8-compatible surface.
- **`apps/browser-worker`**: Playwright 1.59 worker with video receipts and ARIA snapshots.
- **`apps/cli`**: Commander.js CLI for operator control.
- **`apps/web`**: React 19 + Vite 8 dashboard for job monitoring and policy management.
- **`packages/*`**: Modular domain logic (core, db, security, waterfall-engine, etc.).

## Current State

- **Plan**: `Implementation-Plan.md` v2 (Track-based execution).
- **Infrastructure**: ChangeGuard initialized, Docker infrastructure defined, Monorepo scaffolded.
- **Track 0**: Security baseline and infrastructure setup is the immediate priority.

## Engineering Principles (Non-Negotiable)

- **neverthrow**: `Result<T, E>` for all fallible operations. No exceptions for control flow.
- **Zod 4**: Strict schema validation as the single source of truth.
- **Content-Addressing**: All artifacts stored by SHA-256 hash for deduplication and change tracking.
- **Design the Seam**: Implement schemas and stubs for deferred features (Agent, Change Tracking) immediately.
- **TDD**: Two-commit minimum (Red → Green).

## CI Gate (Must Pass Before Every Commit)

```bash
pnpm audit --audit-level high ; biome check --write=false . ; pnpm -r run typecheck ; pnpm -r run test
```

## Workflows

1. **Track Lead**: Follow the `Implementation-Plan.md` track by track.
2. **Ledger**: Record all architectural decisions using `changeguard ledger`.
3. **Verify**: Use `changeguard verify` to ensure structural and behavioral integrity.
4. **Scaffold**: Use `pnpm` workspace protocol for internal package dependencies.

## Key Reference Documents

| Document | Purpose |
|----------|---------|
| `Implementation-Plan.md` | Master execution plan and architecture |
| `.agents/rules/core-mandates.md` | Non-negotiable mandates (Security, ADTs) |
| `docs/threat-model.md` | Security boundaries and SSRF protection |
| `skill/SKILL.md` | Instructions for using CrawlX as an agent |

## Quick Start

1. **Read `Implementation-Plan.md`** to understand the 10 tracks.
2. **Run `changeguard doctor`** to verify your environment.
3. **Install Hooks**: `cp .agents/hooks/* .git/hooks/` to enable workflow enforcement.
4. **Start Track 0**: Security baseline implementation in `packages/security`.
