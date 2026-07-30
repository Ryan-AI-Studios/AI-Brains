import { Link, useParams } from "react-router";

/**
 * Read-only claim detail from route params (id/kind).
 * Evidence handles are listed as links — no xyflow graph required.
 * Full claim body is not re-fetched here; deep inspect uses evidence/source screens.
 */
export function ClaimDetailScreen() {
  const { kind = "", id = "" } = useParams();

  if (!id) {
    return (
      <div className="screen">
        <header className="screen-header">
          <h1>Claim detail</h1>
          <p className="muted">
            Open a claim from Home or Query, or pass kind/id in the route.
          </p>
        </header>
        <div className="card">
          <p className="muted">
            Route shape: <code>#/claim/:kind/:id</code>
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="screen">
      <header className="screen-header">
        <h1>Claim detail</h1>
        <p className="muted">
          Read-only handle view (adapter). No client-side authority graph.
        </p>
      </header>

      <div className="card">
        <p>
          <span className="badge badge-muted">{kind || "unknown"}</span>
        </p>
        <p>
          id: <code>{id}</code>
        </p>
        <p className="muted small">
          Evidence bodies are inspected by id. Use Query/Home packets for
          statements and handle lists when available.
        </p>
        <div className="btn-row">
          <Link className="btn" to={`/evidence/${encodeURIComponent(id)}`}>
            Inspect as evidence id
          </Link>
          <Link className="btn btn-ghost" to="/query">
            Back to query
          </Link>
        </div>
      </div>
    </div>
  );
}
