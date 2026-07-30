/**
 * Shared active scope for live screens that call scope-gated daemon APIs.
 *
 * Populated by ScopeIndicator / ScopeScreen after resolve_scope succeeds.
 * Screens may override locally; never invent a fake scope when missing.
 */
import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";

export interface ResolvedScopeMeta {
  scope: string;
  authoritative: boolean;
  confidence: string;
}

export interface ActiveScopeValue {
  /** Last resolved or user-set scope key; null when unknown. */
  scope: string | null;
  setScope: (scope: string | null) => void;
  authoritative: boolean | null;
  confidence: string | null;
  /**
   * Apply a successful resolve_scope payload when `data.scope` is non-empty.
   * Empty scope keys are ignored (honest unresolved).
   */
  applyResolved: (result: ResolvedScopeMeta) => void;
  /** cwd last associated with a resolve (optional chrome metadata). */
  resolveFromCwd: string | null;
  setResolveFromCwd: (cwd: string | null) => void;
}

const ActiveScopeContext = createContext<ActiveScopeValue | null>(null);

export function ActiveScopeProvider({ children }: { children: ReactNode }) {
  const [scope, setScopeState] = useState<string | null>(null);
  const [authoritative, setAuthoritative] = useState<boolean | null>(null);
  const [confidence, setConfidence] = useState<string | null>(null);
  const [resolveFromCwd, setResolveFromCwd] = useState<string | null>(null);

  const setScope = useCallback((next: string | null) => {
    const trimmed = next?.trim() || null;
    setScopeState(trimmed);
    if (!trimmed) {
      setAuthoritative(null);
      setConfidence(null);
    }
  }, []);

  const applyResolved = useCallback((result: ResolvedScopeMeta) => {
    const key = result.scope.trim();
    if (!key) {
      return;
    }
    setScopeState(key);
    setAuthoritative(result.authoritative);
    setConfidence(result.confidence || null);
  }, []);

  const value = useMemo<ActiveScopeValue>(
    () => ({
      scope,
      setScope,
      authoritative,
      confidence,
      applyResolved,
      resolveFromCwd,
      setResolveFromCwd,
    }),
    [
      scope,
      setScope,
      authoritative,
      confidence,
      applyResolved,
      resolveFromCwd,
    ],
  );

  return (
    <ActiveScopeContext.Provider value={value}>
      {children}
    </ActiveScopeContext.Provider>
  );
}

export function useActiveScope(): ActiveScopeValue {
  const ctx = useContext(ActiveScopeContext);
  if (!ctx) {
    throw new Error("useActiveScope must be used within ActiveScopeProvider");
  }
  return ctx;
}

/** Seed a local input from shared scope without overwriting user edits. */
export function seedScopeInput(
  current: string,
  shared: string | null,
): string {
  if (current.trim()) {
    return current;
  }
  return shared?.trim() ?? "";
}
