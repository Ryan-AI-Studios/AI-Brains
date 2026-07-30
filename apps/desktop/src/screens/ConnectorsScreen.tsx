import { StatePanel } from "../components/StatePanel";

/**
 * Connectors are honest-unavailable on T172 (M4).
 * No fake connected state; no TS connector policy.
 */
export function ConnectorsScreen() {
  return (
    <div className="screen">
      <header className="screen-header">
        <h1>Connectors</h1>
        <p className="muted">
          External connector management is not part of the T161 minimum HTTP
          surface for this track.
        </p>
      </header>
      <StatePanel
        status="unavailable"
        unavailableMessage="Connectors are unavailable in this desktop build. No connector grants, OAuth, or remote source UI is wired. Capture and local sources continue via the daemon/CLI path."
      />
    </div>
  );
}
