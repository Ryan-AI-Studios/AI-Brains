import { useEffect, useId, useRef, useState, type ReactNode } from "react";

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  body: ReactNode;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
  busy?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  /**
   * When set, Confirm is disabled until the user types this exact phrase.
   * Enter in the phrase field focuses Confirm — it does not auto-submit.
   */
  typedConfirmPhrase?: string;
  /** Optional id for aria-describedby (defaults to internal body id). */
  ariaDescribedBy?: string;
}

/**
 * Native HTML `<dialog showModal()>` confirm (U6/U7).
 * Built-in focus trap, Escape cancel, top-layer, restore-focus.
 */
export function ConfirmDialog({
  open,
  title,
  body,
  confirmLabel = "Confirm",
  cancelLabel = "Cancel",
  danger = false,
  busy = false,
  onConfirm,
  onCancel,
  typedConfirmPhrase,
  ariaDescribedBy,
}: ConfirmDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const confirmBtnRef = useRef<HTMLButtonElement>(null);
  const phraseInputRef = useRef<HTMLInputElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);
  const titleId = useId();
  const bodyId = useId();
  const liveId = useId();
  const [phrase, setPhrase] = useState("");

  const requiresPhrase = Boolean(typedConfirmPhrase);
  const phraseMatched =
    !requiresPhrase || phrase === (typedConfirmPhrase ?? "");
  const confirmDisabled = busy || (requiresPhrase && !phraseMatched);

  useEffect(() => {
    const el = dialogRef.current;
    if (!el) {
      return;
    }

    if (open) {
      previousFocusRef.current =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
      setPhrase("");
      if (!el.open) {
        el.showModal();
      }
      // Prefer typed field focus when present; otherwise confirm is fine.
      queueMicrotask(() => {
        if (requiresPhrase) {
          phraseInputRef.current?.focus();
        } else {
          confirmBtnRef.current?.focus();
        }
      });
    } else if (el.open) {
      el.close();
    }
  }, [open, requiresPhrase]);

  // Restore focus after the dialog fully closes (including Escape / cancel).
  useEffect(() => {
    const el = dialogRef.current;
    if (!el) {
      return;
    }
    const onClose = () => {
      const prev = previousFocusRef.current;
      previousFocusRef.current = null;
      if (prev && typeof prev.focus === "function") {
        prev.focus();
      }
    };
    el.addEventListener("close", onClose);
    return () => el.removeEventListener("close", onClose);
  }, []);

  const liveMessage = requiresPhrase
    ? phraseMatched
      ? "Phrase matched"
      : `Type ${typedConfirmPhrase} to enable confirm`
    : "";

  const describedBy = [ariaDescribedBy ?? bodyId, requiresPhrase ? liveId : null]
    .filter(Boolean)
    .join(" ");

  return (
    <dialog
      ref={dialogRef}
      className="dialog native-dialog"
      aria-labelledby={titleId}
      aria-describedby={describedBy || undefined}
      onCancel={(e) => {
        e.preventDefault();
        if (!busy) {
          onCancel();
        }
      }}
      onClick={(e) => {
        // Backdrop click (dialog itself, not children) cancels.
        if (e.target === dialogRef.current && !busy) {
          onCancel();
        }
      }}
    >
      <div className="dialog-inner" onClick={(e) => e.stopPropagation()}>
        <h2 id={titleId}>{title}</h2>
        <div id={bodyId} className="dialog-body">
          {body}
        </div>

        {requiresPhrase && (
          <div className="dialog-typed">
            <label htmlFor={`${titleId}-phrase`}>
              Type <code>{typedConfirmPhrase}</code> to confirm
              <input
                id={`${titleId}-phrase`}
                ref={phraseInputRef}
                type="text"
                autoComplete="off"
                spellCheck={false}
                value={phrase}
                disabled={busy}
                onChange={(e) => setPhrase(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    // Move focus to Confirm — do not auto-submit wipe (U6).
                    confirmBtnRef.current?.focus();
                  }
                }}
              />
            </label>
            <p id={liveId} className="muted small" aria-live="polite">
              {liveMessage}
            </p>
          </div>
        )}

        <div className="dialog-actions">
          <button
            type="button"
            className="btn btn-ghost"
            onClick={onCancel}
            disabled={busy}
          >
            {cancelLabel}
          </button>
          <button
            ref={confirmBtnRef}
            type="button"
            className={danger ? "btn btn-danger" : "btn"}
            onClick={onConfirm}
            disabled={confirmDisabled}
          >
            {busy ? "Working…" : confirmLabel}
          </button>
        </div>
      </div>
    </dialog>
  );
}
