/** Stable react-query keys for T172 screens. */

export const queryKeys = {
  ping: ["ping"] as const,
  connectionInfo: ["daemon-connection-info"] as const,
  health: ["health"] as const,
  projectBriefing: (scope?: string | null, cwd?: string | null) =>
    ["briefing", "project", scope ?? null, cwd ?? null] as const,
  personalBriefing: (scope?: string | null) =>
    ["briefing", "personal", scope ?? null] as const,
  reviewItems: (status?: string | null, scope?: string | null) =>
    ["review", "items", status ?? null, scope ?? null] as const,
  scopeResolve: (cwd?: string | null, forcePersonal?: boolean) =>
    ["scope", "resolve", cwd ?? null, forcePersonal ?? false] as const,
  knowledgeQuery: (query: string, scope?: string | null) =>
    ["knowledge", "query", query, scope ?? null] as const,
  evidence: (id: string) => ["evidence", id] as const,
  source: (id: string) => ["source", id] as const,
};
