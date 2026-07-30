import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { inspectSource } from "../lib/api";
import { useActiveScope } from "../lib/scopeContext";
import { queryKeys } from "../lib/queryKeys";
import { useInvokeQuery } from "../hooks/useInvokeQuery";
import { StatePanel } from "../components/StatePanel";

export function SourceScreen() {
  const params = useParams();
  const routeId = params.id ?? "";
  const { scope: activeScope, setScope: setActiveScope } = useActiveScope();
  const [idInput, setIdInput] = useState(routeId);
  const [activeId, setActiveId] = useState(routeId);
  const [scope, setScope] = useState("");

  // Keep input/active id in sync when navigating #/source/:id → another id.
  useEffect(() => {
    setIdInput(routeId);
    setActiveId(routeId);
  }, [routeId]);

  useEffect(() => {
    if (activeScope && !scope.trim()) {
      setScope(activeScope);
    }
  }, [activeScope, scope]);

  const effectiveScope = scope.trim();

  const q = useInvokeQuery({
    queryKey: queryKeys.source(activeId, effectiveScope || null),
    queryFn: () =>
      inspectSource({
        id: activeId,
        scope: effectiveScope,
      }),
    enabled: !!activeId && !!effectiveScope,
  });

  const needsScope = !!activeId && !effectiveScope;

  return (
    <div className="screen">
      <header className="screen-header">
        <h1>Source inspect</h1>
        <p className="muted">
          Inspect a registered source by id via the daemon. Scope is required.
        </p>
      </header>

      <form
        className="form-row"
        onSubmit={(e) => {
          e.preventDefault();
          const nextScope = scope.trim();
          if (nextScope) {
            setActiveScope(nextScope);
          }
          setActiveId(idInput.trim());
        }}
      >
        <label className="grow">
          Source id
          <input
            value={idInput}
            onChange={(e) => setIdInput(e.target.value)}
            placeholder="source id"
            required
          />
        </label>
        <label>
          Scope (required)
          <input
            value={scope}
            onChange={(e) => setScope(e.target.value)}
            placeholder="Repository:{uuid}"
            required
          />
        </label>
        <button type="submit" className="btn">
          Inspect
        </button>
      </form>

      {needsScope ? (
        <StatePanel
          status="unavailable"
          unavailableMessage="Resolve scope first (Scope screen or chrome indicator), or enter a scope key above. The daemon rejects inspect_source without scope."
        />
      ) : (
        <StatePanel
          status={!activeId ? "idle" : q.status}
          error={q.uiError}
          emptyMessage="Enter a source id to inspect."
          onRetry={q.refetch}
        >
          {q.data && (
            <div className="card">
              <pre className="json">{JSON.stringify(q.data, null, 2)}</pre>
            </div>
          )}
        </StatePanel>
      )}
    </div>
  );
}
