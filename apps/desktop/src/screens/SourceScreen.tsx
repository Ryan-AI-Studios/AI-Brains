import { useState } from "react";
import { useParams } from "react-router";
import { inspectSource } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";
import { useInvokeQuery } from "../hooks/useInvokeQuery";
import { StatePanel } from "../components/StatePanel";

export function SourceScreen() {
  const params = useParams();
  const routeId = params.id ?? "";
  const [idInput, setIdInput] = useState(routeId);
  const [activeId, setActiveId] = useState(routeId);

  const q = useInvokeQuery({
    queryKey: queryKeys.source(activeId),
    queryFn: () => inspectSource({ id: activeId }),
    enabled: !!activeId,
  });

  return (
    <div className="screen">
      <header className="screen-header">
        <h1>Source inspect</h1>
        <p className="muted">Inspect a registered source by id via the daemon.</p>
      </header>

      <form
        className="form-row"
        onSubmit={(e) => {
          e.preventDefault();
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
        <button type="submit" className="btn">
          Inspect
        </button>
      </form>

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
    </div>
  );
}
