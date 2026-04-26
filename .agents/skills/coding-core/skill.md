---
name: coding-core
description: Use this skill when writing, modifying, or reviewing TypeScript code in CrawlX. Trigger when editing .ts files, making architectural decisions, implementing features, or discussing error handling patterns, module boundaries, or waterfall scraping.
---

# Coding Core - CrawlX

Load this skill when working on the Fastify API, Playwright workers, or shared packages.

## Retrieval Precedence

1. **Active File / Context**: Current code and task context.
2. **Local Rules**: `.agents/rules/*.md`.
3. **Documentation**: `Implementation-Plan.md`.
4. **External**: `context7` for library docs (Fastify, Drizzle, Playwright), `exa` for web search.

## Engineering Standards

- **TypeScript**: ~5.8. Strict mode, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`.
- **Error Handling**: `neverthrow` for all fallible operations. `Result<T, E>` is mandatory.
- **Validation**: Zod 4 for all inputs and cross-boundary data.
- **ORM**: Drizzle ORM (PostgreSQL). Migrations via `drizzle-kit`.
- **API Framework**: Fastify 5. Use `@fastify/type-provider-zod` for type-safe routes.
- **Composition**: Prefer adapters and wrappers over inheritance.
- **Immutability**: Use `readonly` for all interfaces and arrays by default.

## Monorepo Boundaries

| Path | Responsibility |
|------|----------------|
| `apps/api` | Fastify server, v2.8 compatibility layer, BullMQ job producers. |
| `apps/browser-worker` | Playwright 1.59 instance, artifact capture, session vault. |
| `packages/core` | Shared domain types, value objects (URL, Hash), and errors. |
| `packages/db` | Drizzle schema, migrations, and repository patterns. |
| `packages/security` | Egress firewall, URL validation, DNS guard, SSRF protection. |
| `packages/waterfall-engine` | Multi-engine fallback orchestrator (Static -> JS -> Playwright). |
| `packages/artifact-store` | Content-addressed persistence (SHA-256). |
| `packages/model-adapter` | LLM abstraction (Ollama, OpenAI-compat) with capability routing. |

## Patterns & Performance

- **Job Persistence**: Every scrape is a job in `jobs` table + BullMQ queue.
- **Deduplication**: Artifacts are stored by content hash. DB references the hash.
- **Streaming**: Prefer streaming large responses and using `undici` for HTTP calls.
- **SSRF Guard**: Always validate URLs and resolved IPs against `packages/security` policy before any request.

## Security Mandates

- **Egress Firewall**: Blocks private CIDRs, metadata IPs, and non-HTTP schemes.
- **Secret Redactor**: Scrub sensitive tokens from logs and stored artifacts.
- **Session Vault**: Encrypted browser profiles (AES-256-GCM) with domain-scoping.
