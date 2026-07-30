import { mapInvokeError, type UiError } from "../lib/errors";
import { StatusBadge } from "./StatusBadge";

export type PanelStatus =
  | "idle"
  | "loading"
  | "empty"
  | "error"
  | "denied"
  | "offline"
  | "unavailable"
  | "ok";

interface StatePanelProps {
  status: PanelStatus;
  error?: UiError | null;
  emptyMessage?: string;
  unavailableMessage?: string;
  loadingMessage?: string;
  children?: React.ReactNode;
  onRetry?: () => void;
}

export function StatePanel({
  status,
  error,
  emptyMessage = "Nothing to show yet.",
  unavailableMessage = "This surface is not available in the current daemon build.",
  loadingMessage = "Loading…",
  children,
  onRetry,
}: StatePanelProps) {
  if (status === "loading") {
    return (
      <div className="state-panel state-loading" role="status">
        <p className="muted">{loadingMessage}</p>
      </div>
    );
  }

  if (status === "offline") {
    return (
      <div className="state-panel state-offline" role="alert">
        <h3 className="state-heading">
          <StatusBadge kind="offline" label="Offline" />
          <span>Daemon offline</span>
        </h3>
        <p>
          {error?.message ??
            "Cannot reach the AI-Brains daemon on loopback. Start the daemon and ensure the user-session token exists."}
        </p>
        {onRetry && (
          <button type="button" className="btn" onClick={onRetry}>
            Retry
          </button>
        )}
      </div>
    );
  }

  if (status === "denied") {
    return (
      <div className="state-panel state-denied" role="alert">
        <h3 className="state-heading">
          <StatusBadge kind="denied" label="Denied" />
          <span>Access denied</span>
        </h3>
        <p>
          {error?.message ??
            "Session token missing or not authorized. Place a user-session token at %USERPROFILE%\\.ai-brains\\http.token."}
        </p>
        {onRetry && (
          <button type="button" className="btn" onClick={onRetry}>
            Retry
          </button>
        )}
      </div>
    );
  }

  if (status === "unavailable") {
    return (
      <div className="state-panel state-unavailable" role="status">
        <h3 className="state-heading">
          <StatusBadge kind="unavailable" label="Unavailable" />
          <span>Unavailable</span>
        </h3>
        <p className="muted">{unavailableMessage}</p>
      </div>
    );
  }

  if (status === "empty") {
    return (
      <div className="state-panel state-empty" role="status">
        <p className="muted">{emptyMessage}</p>
      </div>
    );
  }

  if (status === "error") {
    return (
      <div className="state-panel state-error" role="alert">
        <h3 className="state-heading">
          <StatusBadge kind="error" label="Error" />
          <span>Error</span>
        </h3>
        <p>{error?.message ?? "Request failed."}</p>
        {error?.kind === "transient" && (
          <p className="muted small">Transient failure — you may retry.</p>
        )}
        {onRetry && (
          <button type="button" className="btn" onClick={onRetry}>
            Retry
          </button>
        )}
      </div>
    );
  }

  if (status === "idle") {
    return (
      <div className="state-panel state-idle" role="status">
        <p className="muted">{emptyMessage}</p>
      </div>
    );
  }

  return <>{children}</>;
}

/** Map a structured UI error to a panel status (kind-aware titles). */
export function statusFromUiError(error: UiError): PanelStatus {
  switch (error.kind) {
    case "offline":
      return "offline";
    case "denied":
      return "denied";
    case "unavailable":
      return "unavailable";
    default:
      return "error";
  }
}

/** Map a react-query error + loading flags into a panel status. */
export function statusFromQuery(opts: {
  isLoading: boolean;
  isError: boolean;
  error: unknown;
  isEmpty?: boolean;
  isIdle?: boolean;
}): { status: PanelStatus; uiError: UiError | null } {
  if (opts.isIdle) {
    return { status: "idle", uiError: null };
  }
  if (opts.isLoading) {
    return { status: "loading", uiError: null };
  }
  if (opts.isError) {
    const uiError = mapInvokeError(opts.error);
    return { status: statusFromUiError(uiError), uiError };
  }
  if (opts.isEmpty) {
    return { status: "empty", uiError: null };
  }
  return { status: "ok", uiError: null };
}
