import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { StatePanel } from "./StatePanel";

describe("StatePanel", () => {
  it("StatePanel__offline__shows_StatusBadge_text_and_icon", () => {
    render(
      <StatePanel
        status="offline"
        error={{ kind: "offline", message: "daemon unreachable" }}
      />,
    );
    expect(screen.getByText("Offline")).toBeInTheDocument();
    expect(screen.getByText("Daemon offline")).toBeInTheDocument();
    expect(screen.getByText("daemon unreachable")).toBeInTheDocument();
    const badge = screen.getByText("Offline").closest("[data-status]");
    expect(badge).toHaveAttribute("data-status", "offline");
    expect(badge?.querySelector("svg")).toBeTruthy();
  });

  it("StatePanel__denied__shows_StatusBadge_text_and_icon", () => {
    render(
      <StatePanel
        status="denied"
        error={{ kind: "denied", message: "token missing", status: 401 }}
      />,
    );
    expect(screen.getByText("Denied")).toBeInTheDocument();
    expect(screen.getByText("Access denied")).toBeInTheDocument();
    expect(screen.getByText("token missing")).toBeInTheDocument();
    const badge = screen.getByText("Denied").closest("[data-status]");
    expect(badge).toHaveAttribute("data-status", "denied");
    expect(badge?.querySelector("svg")).toBeTruthy();
  });

  it("StatePanel__error__shows_StatusBadge_and_message", () => {
    render(
      <StatePanel
        status="error"
        error={{ kind: "error", message: "HTTP 400: bad" }}
      />,
    );
    expect(screen.getByText("HTTP 400: bad")).toBeInTheDocument();
    const badge = document.querySelector('[data-status="error"]');
    expect(badge).toBeTruthy();
    expect(badge).toHaveTextContent("Error");
    expect(badge?.querySelector("svg")).toBeTruthy();
  });

  it("StatePanel__empty__shows_empty_message", () => {
    render(
      <StatePanel status="empty" emptyMessage="No review items for this filter." />,
    );
    expect(
      screen.getByText("No review items for this filter."),
    ).toBeInTheDocument();
  });

  it("StatePanel__loading__shows_loading_message", () => {
    render(<StatePanel status="loading" loadingMessage="Loading…" />);
    expect(screen.getByText("Loading…")).toBeInTheDocument();
    expect(screen.getByRole("status")).toBeInTheDocument();
  });

  it("StatePanel__unavailable__shows_badge_and_message", () => {
    render(
      <StatePanel
        status="unavailable"
        unavailableMessage="Connectors are unavailable in this desktop build."
      />,
    );
    expect(
      screen.getByText("Connectors are unavailable in this desktop build."),
    ).toBeInTheDocument();
    const badge = document.querySelector('[data-status="unavailable"]');
    expect(badge).toBeTruthy();
    expect(badge).toHaveTextContent("Unavailable");
    expect(badge?.querySelector("svg")).toBeTruthy();
  });

  it("StatePanel__offline_retry__invokes_onRetry", async () => {
    const user = userEvent.setup();
    const onRetry = vi.fn();
    render(
      <StatePanel
        status="offline"
        error={{ kind: "offline", message: "down" }}
        onRetry={onRetry}
      />,
    );
    await user.click(screen.getByRole("button", { name: "Retry" }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("StatePanel__ok__renders_children", () => {
    render(
      <StatePanel status="ok">
        <p>packet body</p>
      </StatePanel>,
    );
    expect(screen.getByText("packet body")).toBeInTheDocument();
  });
});
