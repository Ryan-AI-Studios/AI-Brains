import { useEffect, useState } from "react";
import { resolveScope } from "../lib/api";
import { asArray, type ScopeEvidence } from "../lib/types";
import { useActiveScope } from "../lib/scopeContext";
import { queryKeys } from "../lib/queryKeys";
import { useInvokeQuery } from "../hooks/useInvokeQuery";
import { StatePanel } from "../components/StatePanel";

export function ScopeScreen() {
  const { applyResolved, setResolveFromCwd } = useActiveScope();
  const [cwd, setCwd] = useState("");
  const [explicitProjectId, setExplicitProjectId] = useState("");
  const [forcePersonal, setForcePersonal] = useState(false);
  const [runKey, setRunKey] = useState(0);

  const q = useInvokeQuery({
    queryKey: [
      ...queryKeys.scopeResolve(cwd || null, forcePersonal),
      explicitProjectId,
      runKey,
    ],
    queryFn: () =>
      resolveScope({
        cwd: cwd || undefined,
        explicit_project_id: explicitProjectId || undefined,
        force_personal: forcePersonal,
      }),
    enabled: runKey > 0,
  });

  const data = q.data;

  useEffect(() => {
    if (!data) {
      return;
    }
    const key = data.scope?.trim() ?? "";
    if (!key) {
      return;
    }
    applyResolved({
      scope: key,
      authoritative: data.authoritative,
      confidence: data.confidence,
    });
    setResolveFromCwd(cwd.trim() || null);
  }, [data, applyResolved, setResolveFromCwd, cwd]);

  return (
    <div className="screen">
      <header className="screen-header">
        <h1>Scope resolve</h1>
        <p className="muted">
          Authoritative flag, confidence, evidence, warnings, and alternatives
          come from the daemon. Grant listing is{" "}
          <strong>honest unavailable</strong> here (no T161 grants list route
          wired for this chrome). Successful resolve updates the shared active
          scope used by Query, Review, Evidence, Source, and Erasure.
        </p>
      </header>

      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          setRunKey((k) => k + 1);
        }}
      >
        <label>
          cwd
          <input
            value={cwd}
            onChange={(e) => setCwd(e.target.value)}
            placeholder="Working directory"
          />
        </label>
        <label>
          explicit_project_id
          <input
            value={explicitProjectId}
            onChange={(e) => setExplicitProjectId(e.target.value)}
            placeholder="Optional UUID"
          />
        </label>
        <label className="checkbox">
          <input
            type="checkbox"
            checked={forcePersonal}
            onChange={(e) => setForcePersonal(e.target.checked)}
          />
          force_personal
        </label>
        <button type="submit" className="btn">
          Resolve
        </button>
      </form>

      <StatePanel
        status={runKey === 0 ? "idle" : q.status}
        error={q.uiError}
        emptyMessage="Submit the form to resolve scope."
        onRetry={q.refetch}
      >
        {data && (
          <div className="stack">
            <div className="card">
              <h2>Result</h2>
              <p>
                <code>{data.scope || "(empty scope)"}</code>
              </p>
              <p>
                confidence: <strong>{data.confidence}</strong>{" "}
                <span
                  className={
                    data.authoritative ? "badge badge-ok" : "badge badge-warn"
                  }
                >
                  {data.authoritative
                    ? "authoritative"
                    : "not authoritative — not a full grant"}
                </span>
              </p>
            </div>

            <div className="card">
              <h2>Evidence signals</h2>
              {asArray<ScopeEvidence>(data.evidence).length === 0 ? (
                <p className="muted">[]</p>
              ) : (
                <ul>
                  {asArray<ScopeEvidence>(data.evidence).map((e, i) => (
                    <li key={`${e.signal}-${i}`}>
                      <strong>{e.signal}</strong>: {e.detail}
                    </li>
                  ))}
                </ul>
              )}
            </div>

            <div className="card">
              <h2>Warnings</h2>
              {asArray<string>(data.warnings).length === 0 ? (
                <p className="muted">[]</p>
              ) : (
                <ul className="warn-list">
                  {asArray<string>(data.warnings).map((w) => (
                    <li key={w}>{w}</li>
                  ))}
                </ul>
              )}
            </div>

            <div className="card">
              <h2>Alternatives</h2>
              {asArray<string>(data.alternatives).length === 0 ? (
                <p className="muted">[]</p>
              ) : (
                <ul>
                  {asArray<string>(data.alternatives).map((a) => (
                    <li key={a}>
                      <code>{a}</code>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </div>
        )}
      </StatePanel>

      <div className="card card-muted">
        <h2>Grants list</h2>
        <p className="muted">
          Honest unavailable — this desktop track does not surface a grants
          inventory endpoint. Do not treat empty UI as “no grants.”
        </p>
      </div>
    </div>
  );
}
