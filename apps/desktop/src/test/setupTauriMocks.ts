/**
 * Thin wrapper around @tauri-apps/api/mocks for Vitest unit tests.
 */
import { clearMocks, mockIPC, type MockIPCOptions } from "@tauri-apps/api/mocks";
import type { InvokeArgs } from "@tauri-apps/api/core";
import { vi } from "vitest";

export type InvokeHandler = (
  cmd: string,
  payload?: InvokeArgs,
) => unknown;

/**
 * Install mockIPC with the given handler.
 * Call {@link cleanupTauriMocks} in afterEach (or use the returned cleanup).
 */
export function setupTauriMocks(
  handler: InvokeHandler,
  options?: MockIPCOptions,
): () => void {
  mockIPC(handler, options);
  return cleanupTauriMocks;
}

/** clearMocks + restoreAllMocks — safe to call even if no mocks were installed. */
export function cleanupTauriMocks(): void {
  clearMocks();
  vi.restoreAllMocks();
}

export { clearMocks, mockIPC };
