---
name: tooling
description: Use this skill when using Sourcebot, GitHub CLI, or ChangeGuard for research, CI verification, or codebase exploration. Trigger when searching the codebase, checking CI, or running changeguard commands.
---

# Tooling & Research - CrawlX

Load this skill when using research or operational tools on this codebase.

## Sourcebot (Deep Research)

Sourcebot is the primary tool for codebase navigation.

### Patterns

1. **Symbol Search**: Find interface implementations (e.g., `implements CrawlEngine`).
2. **Context Mapping**: Before modifying a module, use `ask_codebase` to identify dependencies.

## GitHub CLI (`gh`)

The `gh` CLI bridges local development and the remote repository.

### Patterns

1. **CI Status**: `gh run list` to check the status of the `pnpm test` pipeline.
2. **PR Review**: `gh pr diff` to review changes before final verification.

## ChangeGuard (Governance)

ChangeGuard provides local risk assessment and architectural provenance.

### Patterns

1. **Pre-Flight**: `changeguard scan --impact` before starting an edit.
2. **Provenance**: `changeguard ledger start` for tracked architectural decisions.
3. **Verification**: `changeguard verify` to run the project's test suite.

## Frontend Design (CrawlX Dashboard)

The React 19 dashboard uses Vite 8 and Biome for styling.

### Patterns

1. **UX Alignment**: Before implementing a new page, check `apps/web` for existing patterns.
2. **Component Reference**: Use existing components in `apps/web/src/components` to maintain visual consistency.

## Key Reference Documents

- `Implementation-Plan.md` — Technical implementation roadmap and architectural contracts.
- `.agents/rules/core-mandates.md` — Security, TDD, and Engineering mandates.
