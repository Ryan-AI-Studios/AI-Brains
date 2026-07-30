/**
 * Map Tauri invoke errors → UI state kinds.
 * Offline / denied must paint promptly (M15/M23).
 */

export type UiErrorKind =
  | "offline"
  | "denied"
  | "transient"
  | "error"
  | "unavailable";

export interface UiError {
  kind: UiErrorKind;
  message: string;
  status?: number;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/**
 * Parse a thrown invoke error into a structured UI error.
 * Tauri serializes `InvokeApiError` as the error payload when the Rust side
 * returns `Err(InvokeApiError)`.
 */
export function mapInvokeError(err: unknown): UiError {
  if (err instanceof Error) {
    return parsePayload(err.message) ?? {
      kind: "error",
      message: err.message || "Unknown error",
    };
  }

  if (typeof err === "string") {
    return parsePayload(err) ?? { kind: "error", message: err };
  }

  if (isRecord(err)) {
    const fromObj = fromErrorObject(err);
    if (fromObj) return fromObj;
  }

  return { kind: "error", message: String(err) };
}

function parsePayload(raw: string): UiError | null {
  const trimmed = raw.trim();
  if (!trimmed.startsWith("{")) {
    return null;
  }
  try {
    const parsed: unknown = JSON.parse(trimmed);
    if (isRecord(parsed)) {
      return fromErrorObject(parsed);
    }
  } catch {
    return null;
  }
  return null;
}

function fromErrorObject(obj: Record<string, unknown>): UiError | null {
  const kindRaw = obj.kind;
  const messageRaw = obj.message;
  if (typeof kindRaw !== "string" || typeof messageRaw !== "string") {
    return null;
  }
  const kind = normalizeKind(kindRaw);
  const status =
    typeof obj.status === "number"
      ? obj.status
      : typeof obj.status === "string"
        ? Number(obj.status)
        : undefined;
  return {
    kind,
    message: messageRaw,
    status: Number.isFinite(status) ? status : undefined,
  };
}

function normalizeKind(kind: string): UiErrorKind {
  switch (kind) {
    case "offline":
    case "denied":
    case "transient":
    case "error":
    case "unavailable":
      return kind;
    default:
      return "error";
  }
}

export function isUiErrorKind(kind: string): kind is UiErrorKind {
  return (
    kind === "offline" ||
    kind === "denied" ||
    kind === "transient" ||
    kind === "error" ||
    kind === "unavailable"
  );
}
