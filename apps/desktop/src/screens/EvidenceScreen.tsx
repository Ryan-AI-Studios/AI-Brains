import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { inspectEvidence } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";
import { useInvokeQuery } from "../hooks/useInvokeQuery";
import { StatePanel } from "../components/StatePanel";

export function EvidenceScreen() {
  const params = useParams();
  const routeId = params.id ?? "";
  const [idInput, setIdInput] = useState(routeId);
  const [activeId, setActiveId] = useState(routeId);

  // Keep input/active id in sync when navigating #/evidence/:id → another id.
  useEffect(() => {
    setIdInput(routeId);
    setActiveId(routeId);
  }, [routeId]);

  const q = useInvokeQuery({
    queryKey: queryKeys.evidence(activeId),
    queryFn: () => inspectEvidence({ id: activeId }),
    enabled: !!activeId,
  });

  return (
    <div className="screen">
      <header className="screen-header">
        <h1>Evidence inspect</h1>
        <p className="muted">
          Bounded preview from the daemon (no full raw dump by default).
        </p>
      </header>

      <form
        className="form-row"
        onSubmit={(e) => {
          e.preventDefault();
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
        <button type="submit" className="btn">
          Inspect
        </button>
      </form>

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
    </div>
  );
}
