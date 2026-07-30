import { useEffect, useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { requestErasure, wipeContentEnvelope } from "../lib/api";
import { asArray } from "../lib/types";
import { useActiveScope } from "../lib/scopeContext";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { StatePanel, statusFromUiError } from "../components/StatePanel";
import { mapInvokeError, type UiError } from "../lib/errors";
import type {
  ContentEnvelopeWipedResponse,
  ErasureAcceptedResponse,
} from "../lib/types";

export function ErasureScreen() {
  const { scope: activeScope, setScope: setActiveScope } = useActiveScope();

  // Ticket path
  const [idsText, setIdsText] = useState("");
  const [reason, setReason] = useState("");
  const [ticketScope, setTicketScope] = useState("");
  const [ticketResult, setTicketResult] =
    useState<ErasureAcceptedResponse | null>(null);
  const [ticketError, setTicketError] = useState<UiError | null>(null);

  // Wipe path
  const [contentKeyId, setContentKeyId] = useState("");
  const [wipeScope, setWipeScope] = useState("");
  const [wipeReason, setWipeReason] = useState("");
  const [dryRun, setDryRun] = useState(true);
  const [confirmWipe, setConfirmWipe] = useState(false);
  const [wipeResult, setWipeResult] =
    useState<ContentEnvelopeWipedResponse | null>(null);
  const [wipeError, setWipeError] = useState<UiError | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);

  // Seed both scope fields from shared active scope once available.
  useEffect(() => {
    if (!activeScope) {
      return;
    }
    if (!ticketScope.trim()) {
      setTicketScope(activeScope);
    }
    if (!wipeScope.trim()) {
      setWipeScope(activeScope);
    }
  }, [activeScope, ticketScope, wipeScope]);

  const ticketMutation = useMutation({
    mutationFn: () => {
      const ids = idsText
        .split(/[\n,]/)
        .map((s) => s.trim())
        .filter(Boolean);
      const scope = ticketScope.trim();
      if (!scope) {
        return Promise.reject({
          kind: "error",
          message: "Resolve scope first — request_erasure requires scope",
        });
      }
      setActiveScope(scope);
      return requestErasure({
        ids,
        reason: reason || undefined,
        scope,
      });
    },
    onSuccess: (resp) => {
      setTicketResult(resp);
      setTicketError(null);
    },
    onError: (err) => {
      setTicketError(mapInvokeError(err));
      setTicketResult(null);
    },
  });

  const wipeMutation = useMutation({
    mutationFn: () => {
      const scope = wipeScope.trim();
      if (!scope) {
        return Promise.reject({
          kind: "error",
          message: "Resolve scope first — wipe requires scope",
        });
      }
      setActiveScope(scope);
      return wipeContentEnvelope({
        content_key_id: contentKeyId.trim(),
        scope,
        reason: wipeReason || undefined,
        dry_run: dryRun,
        confirm: confirmWipe,
      });
    },
    onSuccess: (resp) => {
      setWipeResult(resp);
      setWipeError(null);
      setDialogOpen(false);
    },
    onError: (err) => {
      setWipeError(mapInvokeError(err));
      setDialogOpen(false);
    },
  });

  return (
    <div className="screen">
      <header className="screen-header">
        <h1>Erasure</h1>
        <p className="muted">
          <strong>Ticket ≠ wipe (M13).</strong> Requesting erasure accepts a
          ticket only. Content-envelope wipe is a separate command with dry-run
          defaults and honesty warnings from the API. Scope is required for both
          paths.
        </p>
      </header>

      <section className="card">
        <h2>Request erasure ticket</h2>
        <p className="muted small">
          Does <em>not</em> perform cryptographic wipe. Expect API warnings
          stating wipe was not performed.
        </p>
        <form
          className="form-grid"
          onSubmit={(e) => {
            e.preventDefault();
            if (!ticketScope.trim()) {
              setTicketError({
                kind: "error",
                message:
                  "Resolve scope first — request_erasure requires scope",
              });
              return;
            }
            ticketMutation.mutate();
          }}
        >
          <label className="grow">
            Target ids (comma or newline)
            <textarea
              value={idsText}
              onChange={(e) => setIdsText(e.target.value)}
              rows={3}
              required
            />
          </label>
          <label>
            Reason
            <input
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="Optional reason (no secrets)"
            />
          </label>
          <label>
            Scope (required)
            <input
              value={ticketScope}
              onChange={(e) => setTicketScope(e.target.value)}
              placeholder="Repository:{uuid}"
              required
            />
          </label>
          <button
            type="submit"
            className="btn"
            disabled={ticketMutation.isPending}
          >
            {ticketMutation.isPending ? "Submitting…" : "Request ticket"}
          </button>
        </form>
        {ticketError && (
          <StatePanel
            status={statusFromUiError(ticketError)}
            error={ticketError}
          />
        )}
        {ticketResult && (
          <div className="result-box">
            <p>
              status: <strong>{ticketResult.status}</strong> · request_id:{" "}
              <code>{ticketResult.request_id}</code>
            </p>
            <h3>Warnings (ticket honesty)</h3>
            {asArray<string>(ticketResult.warnings).length === 0 ? (
              <p className="muted">[] — still treat as ticket-only, not wipe.</p>
            ) : (
              <ul className="warn-list">
                {asArray<string>(ticketResult.warnings).map((w) => (
                  <li key={w}>{w}</li>
                ))}
              </ul>
            )}
          </div>
        )}
      </section>

      <section className="card">
        <h2>Content-envelope wipe</h2>
        <p className="muted small">
          Defaults are dry-run safe. Execute only when dry_run is false{" "}
          <em>and</em> confirm is true. Honesty warnings from the API are shown
          in full.
        </p>
        <form
          className="form-grid"
          onSubmit={(e) => {
            e.preventDefault();
            setDialogOpen(true);
          }}
        >
          <label>
            content_key_id
            <input
              value={contentKeyId}
              onChange={(e) => setContentKeyId(e.target.value)}
              required
              placeholder="UUID"
            />
          </label>
          <label>
            scope (required)
            <input
              value={wipeScope}
              onChange={(e) => setWipeScope(e.target.value)}
              required
              placeholder="Scope identity key"
            />
          </label>
          <label>
            reason
            <input
              value={wipeReason}
              onChange={(e) => setWipeReason(e.target.value)}
            />
          </label>
          <label className="checkbox">
            <input
              type="checkbox"
              checked={dryRun}
              onChange={(e) => setDryRun(e.target.checked)}
            />
            dry_run (default true)
          </label>
          <label className="checkbox">
            <input
              type="checkbox"
              checked={confirmWipe}
              onChange={(e) => setConfirmWipe(e.target.checked)}
            />
            confirm (required with dry_run false to execute)
          </label>
          <button type="submit" className="btn btn-danger">
            {dryRun ? "Preview wipe (dry-run)" : "Wipe (requires confirm)"}
          </button>
        </form>
        {wipeError && (
          <StatePanel
            status={statusFromUiError(wipeError)}
            error={wipeError}
          />
        )}
        {wipeResult && (
          <div className="result-box">
            <p>
              status: <strong>{wipeResult.status}</strong> · wrap_destroyed:{" "}
              {String(wipeResult.wrap_destroyed)} · blobs:{" "}
              {wipeResult.blobs_considered}
            </p>
            {wipeResult.tombstone_id && (
              <p className="muted small">
                tombstone: {wipeResult.tombstone_id}
              </p>
            )}
            <h3>Wipe honesty warnings</h3>
            <ul className="warn-list">
              {asArray<string>(wipeResult.warnings).map((w) => (
                <li key={w}>{w}</li>
              ))}
            </ul>
            <pre className="json">
              {JSON.stringify(
                {
                  purged: wipeResult.purged,
                  verify: wipeResult.verify,
                  validation: wipeResult.validation,
                },
                null,
                2,
              )}
            </pre>
          </div>
        )}
      </section>

      <section className="card card-muted">
        <h2>Retention plan</h2>
        <p className="muted">
          Honest unavailable — class-based retention plan UI is not wired on
          this track. Do not invent retention status in the client.
        </p>
      </section>

      <ConfirmDialog
        open={dialogOpen}
        title={dryRun ? "Confirm dry-run wipe" : "Confirm content-envelope wipe"}
        danger={!dryRun && confirmWipe}
        busy={wipeMutation.isPending}
        confirmLabel={dryRun ? "Run dry-run" : "Execute wipe"}
        body={
          <div>
            <p>
              content_key_id: <code>{contentKeyId}</code>
            </p>
            <p>
              scope: <code>{wipeScope}</code>
            </p>
            <p>
              dry_run={String(dryRun)} · confirm={String(confirmWipe)}
            </p>
            {!dryRun && !confirmWipe && (
              <p className="error">
                Execute path requires dry_run=false and confirm=true. This
                request will not destroy wrap material.
              </p>
            )}
            {!dryRun && confirmWipe && (
              <p className="error">
                This will attempt cryptographic erasure for envelope-backed
                content only. Not NIST Purge; backups remain decryptable if
                restored.
              </p>
            )}
          </div>
        }
        onCancel={() => setDialogOpen(false)}
        onConfirm={() => wipeMutation.mutate()}
      />
    </div>
  );
}
