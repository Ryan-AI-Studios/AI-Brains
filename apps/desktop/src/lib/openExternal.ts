/**
 * Dual-layer external open wrappers (U3/U20).
 * Invoke Rust open_url / reveal_path only — never import @tauri-apps/plugin-opener.
 */
import { invoke } from "@tauri-apps/api/core";

export async function openUrl(url: string): Promise<void> {
  await invoke("open_url", { url });
}

export async function revealPath(path: string): Promise<void> {
  await invoke("reveal_path", { path });
}

export type LocatorKind =
  | { kind: "https"; value: string }
  | { kind: "path"; value: string }
  | { kind: "text"; value: string }
  | { kind: "none" };

/**
 * Classify an API-provided locator without fabricating one.
 * https → open URL; other URI schemes (http/file/javascript/…) → display-only text;
 * path-like → reveal; other text → display only; absent → none.
 */
export function classifyLocator(
  locator: string | null | undefined,
): LocatorKind {
  if (locator == null) {
    return { kind: "none" };
  }
  const value = locator.trim();
  if (!value) {
    return { kind: "none" };
  }
  if (/^https:\/\//i.test(value)) {
    return { kind: "https", value };
  }
  // Non-https URI schemes are display-only — never path, never openable here.
  // Covers http:, file:, javascript:, data:, etc.
  // Scheme must be ≥2 chars so Windows drive letters (`C:\…`) fall through to path.
  if (/^[a-z][a-z0-9+.-]+:/i.test(value) && !/^[a-zA-Z]:[\\/]/.test(value)) {
    return { kind: "text", value };
  }
  // Path-like: drive letter, UNC/root slash, or contains path separators.
  if (
    /^[a-zA-Z]:[\\/]/.test(value) ||
    /^[\\/]/.test(value) ||
    value.includes("\\") ||
    value.includes("/")
  ) {
    return { kind: "path", value };
  }
  return { kind: "text", value };
}
