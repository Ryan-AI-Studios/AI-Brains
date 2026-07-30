import { useEffect, useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { listReviewItems, resolveReviewItem } from "../lib/api";
import { asArray } from "../lib/types";
import { useActiveScope } from "../lib/scopeContext";
import { queryKeys } from "../lib/queryKeys";
import { useInvokeQuery } from "../hooks/useInvokeQuery";
import { StatePanel, statusFromUiError } from "../components/StatePanel";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { mapInvokeError, type UiError } from "../lib/errors";
import type { ReviewItem } from "../lib/types";

const RESOLUTIONS = ["approved", "dismissed", "deferred"] as const;

export function ReviewScreen() {
  const { scope: activeScope, setScope: setActiveScope } = useActiveScope();
  const [statusFilter, setStatusFilter] = useState("Open");
  const [scope, setScope] = useState("");
  const [pending, setPending] = useState<{
    id: string;
    resolution: string;
  } | null>(null);
  const [resolveError, setResolveError] = useState<UiError | null>(null);
  const [lastWarnings, setLastWarnings] = useState<string[]>([]);
  const qc = useQueryClient();

  useEffect(() => {
    if (activeScope && !scope.trim()) {
      setScope(activeScope);
    }
  }, [activeScope, scope]);

  const effectiveScope = scope.trim();

  const list = useInvokeQuery({
    queryKey: queryKeys.reviewItems(statusFilter || null, effectiveScope || null),
    queryFn: () =>
      listReviewItems({
        status: statusFilter || undefined,
        scope: effectiveScope,
      }),
    enabled: !!effectiveScope,
    isEmpty: (data) => asArray(data?.items).length === 0,
  });

  const mutation = useMutation({
    mutationFn: (args: { id: string; resolution: string }) =>
      resolveReviewItem({
        id: args.id,
        resolution: args.resolution,
        scope: effectiveScope,
      }),
    onSuccess: (resp) => {
      setLastWarnings(asArray<string>(resp.warnings));
      setPending(null);
      setResolveError(null);
      void qc.invalidateQueries({ queryKey: ["review"] });
    },
    onError: (err) => {
      setResolveError(mapInvokeError(err));
    },
  });

  const items = asArray<ReviewItem>(list.data?.items);

  return (
    <div className="screen">
      <header className="screen-header">
        <h1>Review queue</h1>
        <p className="muted">
          List open items and resolve with confirmation. Warnings from the API
          are shown as-is (no local policy). Scope is required by the daemon.
        </p>
      </header>

      <div className="toolbar">
        <label>
          Status filter
          <input
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            placeholder="Open"
          />
        </label>
        <label>
          Scope (required)
          <input
            value={scope}
            onChange={(e) => {
              const next = e.target.value;
              setScope(next);
              if (next.trim()) {
                setActiveScope(next.trim());
              }
            }}
            placeholder="Repository:{uuid}"
          />
        </label>
        <button
          type="button"
          className="btn"
          onClick={() => {
            if (scope.trim()) {
              setActiveScope(scope.trim());
            }
            list.refetch();
          }}
          disabled={!effectiveScope}
        >
          Refresh
        </button>
      </div>

      {!effectiveScope ? (
        <StatePanel
          status="unavailable"
          unavailableMessage="Resolve scope first (Scope screen or chrome indicator), or enter a scope key above. The daemon rejects list_review_items / resolve_review_item without scope."
        />
      ) : (
        <>
          {lastWarnings.length > 0 && (
            <div className="card card-warn">
              <h2>Last resolve warnings</h2>
              <ul className="warn-list">
                {lastWarnings.map((w) => (
                  <li key={w}>{w}</li>
                ))}
              </ul>
            </div>
          )}

          {resolveError && (
            <StatePanel
              status={statusFromUiError(resolveError)}
              error={resolveError}
            />
          )}

          <StatePanel
            status={list.status}
            error={list.uiError}
            emptyMessage="No review items for this filter."
            onRetry={list.refetch}
          >
            <ul className="item-list">
              {items.map((item) => (
                <li key={item.id} className="card item-card">
                  <div>
                    <strong>{item.subject}</strong>
                    <div className="muted small">
                      {item.id} · {item.status}
                      {item.opened_at ? ` · ${item.opened_at}` : ""}
                    </div>
                  </div>
                  <div className="btn-row">
                    {RESOLUTIONS.map((r) => (
                      <button
                        key={r}
                        type="button"
                        className="btn btn-sm"
                        onClick={() =>
                          setPending({ id: item.id, resolution: r })
                        }
                      >
                        {r}
                      </button>
                    ))}
                  </div>
                </li>
              ))}
            </ul>
          </StatePanel>
        </>
      )}

      <ConfirmDialog
        open={!!pending}
        title="Resolve review item"
        danger={pending?.resolution === "dismissed"}
        busy={mutation.isPending}
        confirmLabel={`Resolve as ${pending?.resolution ?? ""}`}
        body={
          <p>
            Resolve item <code>{pending?.id}</code> as{" "}
            <strong>{pending?.resolution}</strong> under scope{" "}
            <code>{effectiveScope || "(missing)"}</code>? The host will attach a
            command_id for idempotency.
          </p>
        }
        onCancel={() => setPending(null)}
        onConfirm={() => {
          if (pending && effectiveScope) {
            mutation.mutate(pending);
          }
        }}
      />
    </div>
  );
}
