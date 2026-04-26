# Core Mandates - CrawlX

1. **Security & SSRF Protection**: Hard prerequisite. Every scrape attempt must pass the three-layer egress firewall (URL validator, DNS guard, container network isolation). Never allow requests to private/internal IPs or cloud metadata endpoints.
2. **Explicit Error Handling**: Use `neverthrow` for all fallible operations. `Result<T, E>` is mandatory for domain logic. No `try/catch` for control flow; no `unwrap()` or `throw` in production code.
3. **Strict Schema Validation**: Zod 4 is the single source of truth. All API inputs, configuration, and engine responses must be strictly validated.
4. **Content-Addressed Artifacts**: All scraped content (HTML, Markdown, screenshots) must be stored using SHA-256 hashes. The database stores hash pointers, never inline content, ensuring deduplication and change tracking.
5. **Composition Over Inheritance**: Prioritize explicit wrappers, adapters, and functional composition. Use the `WaterfallEngine` pattern for multi-engine fallback rather than complex retry logic.
6. **TDD (Two-Commit Minimum)**: Behavioral correctness must be proven via tests before implementation. Commit 1 = failing tests (Red). Commit 2+ = implementation (Green).
7. **CI Gate**: Before every commit, the following must pass:
   `pnpm audit --audit-level high` ; `biome check --write=false .` ; `pnpm -r run typecheck` ; `pnpm -r run test`
8. **Design the Seam Now**: For deferred features (Agent, change tracking, cloud escalation), the database schemas, TypeScript interfaces, and Zod schemas must exist from day one, returning `501 Not Implemented`.
9. **Provenance via ChangeGuard**: All architectural decisions and major changes must be recorded in the `changeguard ledger`.
