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
 * https → open URL; path-like → reveal; other text → display only; absent → none.
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
