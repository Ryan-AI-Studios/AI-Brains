import { useQuery, type UseQueryOptions } from "@tanstack/react-query";
import { mapInvokeError, type UiError } from "../lib/errors";
import type { PanelStatus } from "../components/StatePanel";

export function useInvokeQuery<T>(
  options: UseQueryOptions<T, unknown> & {
    isEmpty?: (data: T | undefined) => boolean;
  },
): {
  data: T | undefined;
  status: PanelStatus;
  uiError: UiError | null;
  refetch: () => void;
  isFetching: boolean;
} {
  const { isEmpty, ...queryOptions } = options;
  const q = useQuery({
    ...queryOptions,
    retry: false,
  });

  let status: PanelStatus = "ok";
  let uiError: UiError | null = null;

  if (q.isLoading || (q.isFetching && !q.data && !q.isError)) {
    status = "loading";
  } else if (q.isError) {
    uiError = mapInvokeError(q.error);
    if (uiError.kind === "offline") status = "offline";
    else if (uiError.kind === "denied") status = "denied";
    else if (uiError.kind === "unavailable") status = "unavailable";
    else status = "error";
  } else if (isEmpty?.(q.data)) {
    status = "empty";
  } else if (q.isPending && !q.isFetching) {
    status = "idle";
  }

  return {
    data: q.data,
    status,
    uiError,
    refetch: () => {
      void q.refetch();
    },
    isFetching: q.isFetching,
  };
}
