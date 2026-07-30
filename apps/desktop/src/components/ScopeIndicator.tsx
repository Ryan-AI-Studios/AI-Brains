import { useQuery } from "@tanstack/react-query";
import { getDaemonConnectionInfo, resolveScope } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";

/**
 * Chrome indicator: connection + best-effort scope resolve.
 * Never treats non-authoritative scope as a full grant (M5).
 */
export function ScopeIndicator() {
  const conn = useQuery({
    queryKey: queryKeys.connectionInfo,
    queryFn: getDaemonConnectionInfo,
  });

  const scope = useQuery({
    queryKey: queryKeys.scopeResolve(undefined, false),
    queryFn: () => resolveScope({}),
    enabled: !!conn.data?.token_file_present && !!conn.data?.loopback_base_url,
    retry: false,
  });

  const tokenOk = conn.data?.token_file_present === true;
  const base = conn.data?.loopback_base_url ?? "—";

  let scopeLabel = "scope unresolved";
  let scopeClass = "badge badge-muted";
  if (scope.isSuccess) {
    const s = scope.data;
    if (s.authoritative) {
      scopeLabel = s.scope || "authoritative scope";
      scopeClass = "badge badge-ok";
    } else {
      scopeLabel = `${s.confidence || "Low"}: ${s.scope || "(empty)"}`;
      scopeClass = "badge badge-warn";
    }
  } else if (scope.isError) {
    scopeLabel = "scope unavailable";
    scopeClass = "badge badge-muted";
  } else if (!tokenOk) {
    scopeLabel = "no session token";
    scopeClass = "badge badge-warn";
  }

  return (
    <div className="scope-indicator" data-testid="scope-indicator">
      <span className="muted small" title={base}>
        {base}
      </span>
      <span className={tokenOk ? "badge badge-ok" : "badge badge-warn"}>
        {tokenOk ? "token present" : "token missing"}
      </span>
      <span className={scopeClass} title={scope.data?.warnings?.join("; ") ?? ""}>
        {scopeLabel}
      </span>
    </div>
  );
}
