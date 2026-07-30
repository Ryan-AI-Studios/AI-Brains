import { useEffect, type ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import { getDaemonConnectionInfo, resolveScope } from "../lib/api";
import { useActiveScope } from "../lib/scopeContext";
import { queryKeys } from "../lib/queryKeys";
import { StatusBadge } from "./StatusBadge";

/**
 * Chrome indicator: connection + best-effort scope resolve.
 * Never treats non-authoritative scope as a full grant (M5).
 * On success with a non-empty scope key, populates shared ActiveScope context.
 */
export function ScopeIndicator() {
  const { applyResolved } = useActiveScope();

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

  useEffect(() => {
    if (!scope.isSuccess || !scope.data) {
      return;
    }
    const key = scope.data.scope?.trim() ?? "";
    if (!key) {
      return;
    }
    applyResolved({
      scope: key,
      authoritative: scope.data.authoritative,
      confidence: scope.data.confidence,
    });
  }, [scope.isSuccess, scope.data, applyResolved]);

  const tokenOk = conn.data?.token_file_present === true;
  const base = conn.data?.loopback_base_url ?? "—";

  let scopeBadge: ReactNode;
  if (scope.isSuccess) {
    const s = scope.data;
    if (s.authoritative) {
      scopeBadge = (
        <StatusBadge
          kind="ok"
          label={s.scope || "authoritative scope"}
          title={s.warnings?.join("; ") ?? ""}
        />
      );
    } else {
      scopeBadge = (
        <StatusBadge
          kind="warn"
          label={`${s.confidence || "Low"}: ${s.scope || "(empty)"}`}
          title={s.warnings?.join("; ") ?? "Non-authoritative scope"}
        />
      );
    }
  } else if (scope.isError) {
    scopeBadge = (
      <StatusBadge kind="unavailable" label="scope unavailable" />
    );
  } else if (!tokenOk) {
    scopeBadge = <StatusBadge kind="warn" label="no session token" />;
  } else {
    scopeBadge = <StatusBadge kind="unavailable" label="scope unresolved" />;
  }

  return (
    <div className="scope-indicator" data-testid="scope-indicator">
      <span className="muted small" title={base}>
        {base}
      </span>
      <StatusBadge
        kind={tokenOk ? "ok" : "warn"}
        label={tokenOk ? "token present" : "token missing"}
      />
      {scopeBadge}
    </div>
  );
}
