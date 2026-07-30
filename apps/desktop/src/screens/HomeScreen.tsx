import { useState } from "react";
import { Link } from "react-router";
import { personalBriefing, projectBriefing } from "../lib/api";
import { asArray } from "../lib/types";
import { queryKeys } from "../lib/queryKeys";
import { useInvokeQuery } from "../hooks/useInvokeQuery";
import { StatePanel } from "../components/StatePanel";
import type {
  BriefingClaim,
  BriefingWarning,
  EvidenceHandle,
  PersonalBriefingPacket,
  PersonalPreference,
  PersonalReviewItem,
  ProjectBriefingPacket,
} from "../lib/types";

type BriefingMode = "project" | "personal";

export function HomeScreen() {
  const [mode, setMode] = useState<BriefingMode>("project");
  const [cwd, setCwd] = useState("");
  const [scope, setScope] = useState("");
  const [enabled, setEnabled] = useState(true);

  const project = useInvokeQuery({
    queryKey: queryKeys.projectBriefing(scope || null, cwd || null),
    queryFn: () =>
      projectBriefing({
        scope: scope || undefined,
        cwd: cwd || undefined,
      }),
    enabled: enabled && mode === "project",
  });

  const personal = useInvokeQuery({
    queryKey: queryKeys.personalBriefing(scope || null),
    queryFn: () =>
      personalBriefing({
        scope: scope || undefined,
      }),
    enabled: enabled && mode === "personal",
  });

  const active = mode === "project" ? project : personal;
  const packet =
    mode === "project"
      ? project.data?.packet
      : personal.data?.packet;

  return (
    <div className="screen">
      <header className="screen-header">
        <h1>Briefing</h1>
        <p className="muted">
          Project and Personal packets are separate (M12) — never silently merged.
          Claims and freshness come from the daemon packet only.
        </p>
      </header>

      <div className="toolbar">
        <div className="segmented" role="tablist" aria-label="Briefing kind">
          <button
            type="button"
            role="tab"
            aria-selected={mode === "project"}
            className={mode === "project" ? "seg active" : "seg"}
            onClick={() => setMode("project")}
          >
            Project
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={mode === "personal"}
            className={mode === "personal" ? "seg active" : "seg"}
            onClick={() => setMode("personal")}
          >
            Personal
          </button>
        </div>
        <button
          type="button"
          className="btn"
          onClick={() => {
            setEnabled(true);
            active.refetch();
          }}
        >
          Refresh
        </button>
      </div>

      <form
        className="form-row"
        onSubmit={(e) => {
          e.preventDefault();
          setEnabled(true);
          active.refetch();
        }}
      >
        {mode === "project" && (
          <label>
            cwd (optional)
            <input
              value={cwd}
              onChange={(e) => setCwd(e.target.value)}
              placeholder="C:/path/to/repo"
            />
          </label>
        )}
        <label>
          scope (optional)
          <input
            value={scope}
            onChange={(e) => setScope(e.target.value)}
            placeholder={
              mode === "project"
                ? "Repository:{uuid}"
                : "Personal:{user_id}"
            }
          />
        </label>
        <button type="submit" className="btn">
          Load
        </button>
      </form>

      <StatePanel
        status={active.status}
        error={active.uiError}
        emptyMessage="Briefing packet is empty."
        onRetry={active.refetch}
      >
        {mode === "project" && packet && (
          <ProjectPacketView packet={packet as ProjectBriefingPacket} />
        )}
        {mode === "personal" && packet && (
          <PersonalPacketView packet={packet as PersonalBriefingPacket} />
        )}
      </StatePanel>
    </div>
  );
}

function ProjectPacketView({ packet }: { packet: ProjectBriefingPacket }) {
  const decisions = asArray<BriefingClaim>(packet.decisions);
  const conclusions = asArray<BriefingClaim>(packet.conclusions);
  const warnings = asArray<BriefingWarning>(packet.warnings);

  if (packet.denied) {
    return (
      <div className="card">
        <h2>Denied</h2>
        <p className="error">{packet.denial_reason ?? "Policy denied this briefing."}</p>
      </div>
    );
  }

  return (
    <div className="stack">
      <div className="card">
        <h2>Scope</h2>
        <p>
          <code>{packet.scope?.scope_key ?? "—"}</code>{" "}
          <span
            className={
              packet.scope?.authoritative ? "badge badge-ok" : "badge badge-warn"
            }
          >
            {packet.scope?.authoritative
              ? "authoritative"
              : `non-authoritative · ${packet.scope?.confidence ?? "?"}`}
          </span>
        </p>
        {asArray<string>(packet.scope?.warnings).length > 0 && (
          <ul className="warn-list">
            {asArray<string>(packet.scope.warnings).map((w) => (
              <li key={w}>{w}</li>
            ))}
          </ul>
        )}
      </div>

      <div className="card">
        <h2>Freshness (from packet)</h2>
        <p className="muted small">
          total {packet.freshness?.total_sources ?? 0} · fresh{" "}
          {packet.freshness?.fresh_count ?? 0} · stale{" "}
          {packet.freshness?.stale_count ?? 0} · unavailable{" "}
          {packet.freshness?.unavailable_count ?? 0} · worst{" "}
          {packet.freshness?.worst_state ?? "Unknown"}
        </p>
      </div>

      <ClaimList title="Decisions" claims={decisions} />
      <ClaimList title="Conclusions" claims={conclusions} />

      {warnings.length > 0 && (
        <div className="card">
          <h2>Warnings</h2>
          <ul className="warn-list">
            {warnings.map((w, i) => (
              <li key={`${w.kind}-${i}`}>
                <strong>{w.kind}</strong>: {w.message}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

function PersonalPacketView({ packet }: { packet: PersonalBriefingPacket }) {
  if (packet.denied) {
    return (
      <div className="card">
        <h2>Denied</h2>
        <p className="error">{packet.denial_reason ?? "Policy denied this briefing."}</p>
      </div>
    );
  }

  const prefs = asArray<PersonalPreference>(packet.preferences);
  const warnings = asArray<BriefingWarning>(packet.warnings);
  const reviews = asArray<PersonalReviewItem>(packet.open_review_items);

  return (
    <div className="stack">
      <div className="card">
        <h2>Personal scope</h2>
        <p>
          <code>{packet.scope_key}</code>
        </p>
        <p className="muted small">
          Continuity: {packet.continuity?.summary || "(empty)"}
        </p>
      </div>

      <div className="card">
        <h2>Preferences</h2>
        {prefs.length === 0 ? (
          <p className="muted">No preferences.</p>
        ) : (
          <ul>
            {prefs.map((p) => (
              <li key={p.id}>{p.statement}</li>
            ))}
          </ul>
        )}
      </div>

      <div className="card">
        <h2>Open review items</h2>
        {reviews.length === 0 ? (
          <p className="muted">None.</p>
        ) : (
          <ul>
            {reviews.map((r) => (
              <li key={r.id}>
                {r.subject} · {r.status} · {r.criticality}
              </li>
            ))}
          </ul>
        )}
      </div>

      {warnings.length > 0 && (
        <div className="card">
          <h2>Warnings</h2>
          <ul className="warn-list">
            {warnings.map((w, i) => (
              <li key={`${w.kind}-${i}`}>
                <strong>{w.kind}</strong>: {w.message}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

function ClaimList({
  title,
  claims,
}: {
  title: string;
  claims: BriefingClaim[];
}) {
  return (
    <div className="card">
      <h2>{title}</h2>
      {claims.length === 0 ? (
        <p className="muted">None.</p>
      ) : (
        <ul className="claim-list">
          {claims.map((c) => (
            <li key={c.id}>
              <Link
                to={`/claim/${encodeURIComponent(c.kind)}/${encodeURIComponent(c.id)}`}
                state={{
                  evidence_handles: asArray<EvidenceHandle>(c.evidence_handles),
                  statement: c.statement,
                }}
              >
                {c.title || c.statement.slice(0, 120)}
              </Link>
              <span className="badge badge-muted">{c.state}</span>
              <div className="muted small">
                evidence:{" "}
                {asArray<EvidenceHandle>(c.evidence_handles)
                  .map((h) => h.evidence_id)
                  .join(", ") || "—"}
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
