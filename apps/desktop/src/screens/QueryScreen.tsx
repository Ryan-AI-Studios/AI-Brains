import { useState } from "react";
import { Link } from "react-router";
import { queryKnowledge } from "../lib/api";
import {
  asArray,
  type EvidenceHandle,
  type ProgressiveQueryHit,
} from "../lib/types";
import { queryKeys } from "../lib/queryKeys";
import { useInvokeQuery } from "../hooks/useInvokeQuery";
import { StatePanel } from "../components/StatePanel";

export function QueryScreen() {
  const [queryText, setQueryText] = useState("");
  const [scope, setScope] = useState("");
  const [submitted, setSubmitted] = useState<{
    query: string;
    scope: string;
  } | null>(null);
  const [expanded, setExpanded] = useState<string | null>(null);

  const q = useInvokeQuery({
    queryKey: queryKeys.knowledgeQuery(
      submitted?.query ?? "",
      submitted?.scope || null,
    ),
    queryFn: () =>
      queryKnowledge({
        query: submitted!.query,
        scope: submitted!.scope || undefined,
      }),
    enabled: !!submitted?.query,
    isEmpty: (data) =>
      !!data && !data.denied && asArray(data.results).length === 0,
  });

  const results = asArray<ProgressiveQueryHit>(q.data?.results);

  return (
    <div className="screen">
      <header className="screen-header">
        <h1>Knowledge query</h1>
        <p className="muted">
          Progressive query via daemon. Results are ranked by the control plane —
          UI does not re-rank or invent authority.
        </p>
      </header>

      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          if (!queryText.trim()) return;
          setSubmitted({ query: queryText.trim(), scope: scope.trim() });
        }}
      >
        <label className="grow">
          Query
          <input
            value={queryText}
            onChange={(e) => setQueryText(e.target.value)}
            placeholder="Search governed knowledge…"
            required
          />
        </label>
        <label>
          Scope (optional)
          <input
            value={scope}
            onChange={(e) => setScope(e.target.value)}
            placeholder="Repository:{uuid}"
          />
        </label>
        <button type="submit" className="btn">
          Search
        </button>
      </form>

      <StatePanel
        status={!submitted ? "idle" : q.status}
        error={q.uiError}
        emptyMessage="No results for this query."
        onRetry={q.refetch}
      >
        {q.data?.denied && (
          <div className="card card-error">
            <h2>Denied</h2>
            <p>{q.data.denial_reason ?? "Policy denied this query."}</p>
          </div>
        )}

        {q.data && !q.data.denied && (
          <div className="stack">
            <p className="muted small">
              scope {q.data.applied_scope} · policy {q.data.applied_policy} ·
              trace {q.data.query_trace_id}
              {q.data.more_available ? " · more available" : ""}
            </p>
            <ul className="item-list">
              {results.map((hit) => {
                const isOpen = expanded === hit.id;
                return (
                  <li key={hit.id} className="card item-card">
                    <div className="grow">
                      <button
                        type="button"
                        className="linkish"
                        onClick={() =>
                          setExpanded(isOpen ? null : hit.id)
                        }
                      >
                        <strong>{hit.kind}</strong>: {hit.statement.slice(0, 140)}
                        {hit.statement.length > 140 ? "…" : ""}
                      </button>
                      <div className="muted small">
                        {hit.state} · freshness {hit.freshness}
                        {hit.conflict_status
                          ? ` · conflict ${hit.conflict_status}`
                          : ""}
                      </div>
                      {isOpen && (
                        <div className="expand">
                          <p>{hit.statement}</p>
                          <p className="muted small">
                            ranking authority={hit.ranking?.authority}{" "}
                            valid_time={hit.ranking?.valid_time}
                            {hit.ranking?.relevance != null
                              ? ` relevance=${hit.ranking.relevance}`
                              : ""}
                          </p>
                          <p className="muted small">
                            evidence:{" "}
                            {asArray<EvidenceHandle>(hit.evidence_handles).map(
                              (h) => (
                              <Link
                                key={h.evidence_id}
                                to={`/evidence/${encodeURIComponent(h.evidence_id)}`}
                                className="inline-link"
                              >
                                {h.cite_label || h.evidence_id}
                              </Link>
                              ),
                            )}
                            {asArray<EvidenceHandle>(hit.evidence_handles)
                              .length === 0 && "—"}
                          </p>
                          <Link
                            to={`/claim/${encodeURIComponent(hit.kind)}/${encodeURIComponent(hit.id)}`}
                            state={{
                              evidence_handles: asArray<EvidenceHandle>(
                                hit.evidence_handles,
                              ),
                              statement: hit.statement,
                            }}
                          >
                            Open claim detail
                          </Link>
                        </div>
                      )}
                    </div>
                  </li>
                );
              })}
            </ul>
          </div>
        )}
      </StatePanel>
    </div>
  );
}
