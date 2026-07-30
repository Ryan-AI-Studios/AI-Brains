import { afterEach, describe, expect, it } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ScopeIndicator } from "./ScopeIndicator";
import { ActiveScopeProvider } from "../lib/scopeContext";
import {
  cleanupTauriMocks,
  setupTauriMocks,
} from "../test/setupTauriMocks";

function renderIndicator() {
  const client = new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0 },
    },
  });
  return render(
    <QueryClientProvider client={client}>
      <ActiveScopeProvider>
        <ScopeIndicator />
      </ActiveScopeProvider>
    </QueryClientProvider>,
  );
}

describe("ScopeIndicator", () => {
  afterEach(() => {
    cleanupTauriMocks();
  });

  it("ScopeIndicator__token_missing__shows_token_missing_badge", async () => {
    setupTauriMocks((cmd) => {
      if (cmd === "get_daemon_connection_info") {
        return {
          loopback_base_url: "http://127.0.0.1:7432",
          token_file_present: false,
        };
      }
      throw { kind: "error", message: `unexpected cmd ${cmd}` };
    });

    renderIndicator();

    await waitFor(() => {
      expect(screen.getByText("token missing")).toBeInTheDocument();
    });
    expect(screen.getByText("no session token")).toBeInTheDocument();
    expect(screen.getByTestId("scope-indicator")).toBeInTheDocument();
  });

  it("ScopeIndicator__token_present_and_scope_ok__shows_authoritative_badge", async () => {
    setupTauriMocks((cmd) => {
      if (cmd === "get_daemon_connection_info") {
        return {
          loopback_base_url: "http://127.0.0.1:7432",
          token_file_present: true,
        };
      }
      if (cmd === "resolve_scope") {
        return {
          api_version: "1",
          scope: "Repository:fixture-uuid",
          confidence: "High",
          authoritative: true,
          evidence: [],
          warnings: [],
          alternatives: [],
        };
      }
      throw { kind: "error", message: `unexpected cmd ${cmd}` };
    });

    renderIndicator();

    await waitFor(() => {
      expect(screen.getByText("token present")).toBeInTheDocument();
      expect(screen.getByText("Repository:fixture-uuid")).toBeInTheDocument();
    });
  });
});
