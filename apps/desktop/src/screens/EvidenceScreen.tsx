import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { inspectEvidence } from "../lib/api";
import { useActiveScope } from "../lib/scopeContext";
import { queryKeys } from "../lib/queryKeys";
import { useInvokeQuery } from "../hooks/useInvokeQuery";
import { StatePanel } from "../components/StatePanel";

export function EvidenceScreen() {
  const params = useParams();
  const routeId = params.id ?? "";
  const { scope: activeScope, setScope: setActiveScope } = useActiveScope();
  const [idInput, setIdInput] = useState(routeId);
  const [activeId, setActiveId] = useState(routeId);
  const [scope, setScope] = useState("");

  // Keep input/active id in sync when navigating #/evidence/:id → another id.
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
    queryKey: queryKeys.evidence(activeId, effectiveScope || null),
    queryFn: () =>
      inspectEvidence({
        id: activeId,
        scope: effectiveScope,
      }),
    enabled: !!activeId && !!effectiveScope,
  });

  const needsScope = !!activeId && !effectiveScope;

  return (
    <div className="screen">
      <header className="screen-header">
        <h1>Evidence inspect</h1>
        <p className="muted">
          Bounded preview from the daemon (no full raw dump by default). Scope
          is required by the daemon.
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
          Evidence id
          <input
            value={idInput}
            onChange={(e) => setIdInput(e.target.value)}
            placeholder="evidence id"
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
          unavailableMessage="Resolve scope first (Scope screen or chrome indicator), or enter a scope key above. The daemon rejects inspect_evidence without scope."
        />
      ) : (
        <StatePanel
          status={!activeId ? "idle" : q.status}
          error={q.uiError}
          emptyMessage="Enter an evidence id to inspect."
          onRetry={q.refetch}
        >
          {q.data && (
            <div className="card">
              <h2>
                {q.data.kind} · <code>{q.data.handle_id}</code>
              </h2>
              {q.data.source_version_id && (
                <p className="muted small">
                  source_version: {q.data.source_version_id}
                </p>
              )}
              <pre className="json">{q.data.preview}</pre>
              {q.data.truncated && (
                <p className="muted small">Preview truncated by server.</p>
              )}
            </div>
          )}
        </StatePanel>
      )}
    </div>
  );
}
