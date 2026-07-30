import { Link, useLocation, useParams } from "react-router";
import { asArray } from "../lib/types";
import type { EvidenceHandle } from "../lib/types";

/** Optional route state when opening a claim from Home/Query packets. */
interface ClaimLocationState {
  evidence_handles?: EvidenceHandle[];
  statement?: string;
}

/**
 * Read-only claim detail from route params (id/kind).
 * Claim id is never treated as an evidence id. Evidence is linked only when
 * handles are provided via route state (or later packet load).
 */
export function ClaimDetailScreen() {
  const { kind = "", id = "" } = useParams();
  const location = useLocation();
  const state = (location.state ?? null) as ClaimLocationState | null;
  const evidenceHandles = asArray<EvidenceHandle>(state?.evidence_handles);

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
        {state?.statement && <p>{state.statement}</p>}
        <p className="muted small">
          Claim id is not an evidence id. Inspect evidence only via handles from
          the source packet.
        </p>

        <h2>Evidence handles</h2>
        {evidenceHandles.length === 0 ? (
          <p className="muted small">
            No evidence handles on this navigation. Open the claim from Home or
            Query (packet-backed links pass handles), or inspect evidence by id
            from those screens.
          </p>
        ) : (
          <ul className="claim-list">
            {evidenceHandles.map((h) => (
              <li key={h.evidence_id}>
                <Link
                  to={`/evidence/${encodeURIComponent(h.evidence_id)}`}
                  className="inline-link"
                >
                  {h.cite_label || h.evidence_id}
                </Link>
              </li>
            ))}
          </ul>
        )}

        <div className="btn-row">
          <Link className="btn btn-ghost" to="/query">
            Back to query
          </Link>
          <Link className="btn btn-ghost" to="/">
            Back to home
          </Link>
        </div>
      </div>
    </div>
  );
}
