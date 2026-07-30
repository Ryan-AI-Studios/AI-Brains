import { useEffect, useState } from "react";
import {
  getDaemonConnectionInfo,
  ping,
  type DaemonConnectionInfo,
  type PingResponse,
} from "./lib/api";

type LoadState<T> =
  | { status: "loading" }
  | { status: "ok"; data: T }
  | { status: "error"; message: string };

export default function App() {
  const [pingState, setPingState] = useState<LoadState<PingResponse>>({
    status: "loading",
  });
  const [connState, setConnState] = useState<LoadState<DaemonConnectionInfo>>({
    status: "loading",
  });

  useEffect(() => {
    let cancelled = false;

    void (async () => {
      try {
        const data = await ping();
        if (!cancelled) {
          setPingState({ status: "ok", data });
        }
      } catch (err) {
        if (!cancelled) {
          setPingState({
            status: "error",
            message: err instanceof Error ? err.message : String(err),
          });
        }
      }

      try {
        const data = await getDaemonConnectionInfo();
        if (!cancelled) {
          setConnState({ status: "ok", data });
        }
      } catch (err) {
        if (!cancelled) {
          setConnState({
            status: "error",
            message: err instanceof Error ? err.message : String(err),
          });
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main className="app">
      <header className="header">
        <h1>AI-Brains Desktop</h1>
        <p className="tagline">
          Adapter-only shell (T171). Invoke-first — no domain authority in the UI.
        </p>
      </header>

      <section className="card" aria-labelledby="ping-heading">
        <h2 id="ping-heading">Host ping</h2>
        {pingState.status === "loading" && <p className="muted">Loading…</p>}
        {pingState.status === "error" && (
          <p className="error" role="alert">
            Ping failed: {pingState.message}
          </p>
        )}
        {pingState.status === "ok" && (
          <pre className="json" data-testid="ping-result">
            {JSON.stringify(pingState.data, null, 2)}
          </pre>
        )}
      </section>

      <section className="card" aria-labelledby="conn-heading">
        <h2 id="conn-heading">Daemon connection info</h2>
        <p className="muted small">
          Presence metadata only — bearer never returned to the webview.
        </p>
        {connState.status === "loading" && <p className="muted">Loading…</p>}
        {connState.status === "error" && (
          <p className="error" role="alert">
            Connection info failed: {connState.message}
          </p>
        )}
        {connState.status === "ok" && (
          <pre className="json" data-testid="conn-result">
            {JSON.stringify(connState.data, null, 2)}
          </pre>
        )}
      </section>
    </main>
  );
}
