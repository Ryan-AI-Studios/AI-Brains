import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { inspectSource } from "../lib/api";
import { useActiveScope } from "../lib/scopeContext";
import { queryKeys } from "../lib/queryKeys";
import { useInvokeQuery } from "../hooks/useInvokeQuery";
import { StatePanel } from "../components/StatePanel";
import {
  classifyLocator,
  openUrl,
  revealPath,
} from "../lib/openExternal";
import { mapInvokeError, type UiError } from "../lib/errors";
import type { SourceDto } from "../lib/types";

function asSourceDto(data: unknown): SourceDto | null {
  if (!data || typeof data !== "object") {
    return null;
  }
  const rec = data as Record<string, unknown>;
  if (typeof rec.id !== "string" || typeof rec.kind !== "string") {
    return null;
  }
  return {
    id: rec.id,
    kind: rec.kind,
    display_name:
      typeof rec.display_name === "string" ? rec.display_name : rec.id,
    locator: typeof rec.locator === "string" ? rec.locator : null,
    last_observed_at:
      typeof rec.last_observed_at === "string" ? rec.last_observed_at : null,
  };
}

export function SourceScreen() {
  const params = useParams();
  const routeId = params.id ?? "";
  const { scope: activeScope, setScope: setActiveScope } = useActiveScope();
  const [idInput, setIdInput] = useState(routeId);
  const [activeId, setActiveId] = useState(routeId);
  const [scope, setScope] = useState("");
  const [openError, setOpenError] = useState<UiError | null>(null);
  const [openBusy, setOpenBusy] = useState(false);

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
  const source = asSourceDto(q.data);
  const locator = classifyLocator(source?.locator);

  return (
    <div className="screen">
      <header className="screen-header">
        <h1>Source inspect</h1>
        <p className="muted">
          Inspect a registered source by id via the daemon. Scope is required.
          Open/reveal uses dual-layer Rust open only when the API provides a
          locator (never fabricated).
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
              {source && (
                <div className="locator-row">
                  <div>
                    <strong>{source.display_name}</strong>
                    <div className="muted small">
                      {source.kind} · <code>{source.id}</code>
                    </div>
                  </div>
                  {locator.kind === "https" && (
                    <button
                      type="button"
                      className="btn btn-sm"
                      disabled={openBusy}
                      onClick={() => {
                        setOpenBusy(true);
                        setOpenError(null);
                        void openUrl(locator.value)
                          .catch((err) => setOpenError(mapInvokeError(err)))
                          .finally(() => setOpenBusy(false));
                      }}
                    >
                      Open URL
                    </button>
                  )}
                  {locator.kind === "path" && (
                    <button
                      type="button"
                      className="btn btn-sm"
                      disabled={openBusy}
                      onClick={() => {
                        setOpenBusy(true);
                        setOpenError(null);
                        void revealPath(locator.value)
                          .catch((err) => setOpenError(mapInvokeError(err)))
                          .finally(() => setOpenBusy(false));
                      }}
                    >
                      Reveal path
                    </button>
                  )}
                  {locator.kind === "text" && (
                    <p className="muted small">
                      Locator (display only): <code>{locator.value}</code>
                    </p>
                  )}
                  {locator.kind === "none" && (
                    <p className="muted small">No locator available</p>
                  )}
                </div>
              )}
              {openError && (
                <p className="error" role="alert">
                  {openError.message}
                </p>
              )}
              <pre className="json">{JSON.stringify(q.data, null, 2)}</pre>
            </div>
          )}
        </StatePanel>
      )}
    </div>
  );
}
