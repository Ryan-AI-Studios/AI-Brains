/**
 * Shared QueryClient — M15/M23: default retry false so offline/denied paint promptly.
 */
import { QueryClient } from "@tanstack/react-query";
import { mapInvokeError } from "./errors";

function shouldRetry(_failureCount: number, error: unknown): boolean {
  // Only retry explicit transient kinds if a caller re-enables retry later.
  // Default for this client is retry: false; this guard is defense-in-depth.
  const mapped = mapInvokeError(error);
  return mapped.kind === "transient";
}

export function createAppQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        refetchOnWindowFocus: false,
        staleTime: 10_000,
      },
      mutations: {
        retry: false,
      },
    },
  });
}

/** Exported for tests / advanced callers that want kind-gated retry. */
export { shouldRetry };
