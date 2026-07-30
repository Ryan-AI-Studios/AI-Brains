import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ConfirmDialog } from "./ConfirmDialog";

describe("ConfirmDialog", () => {
  it("ConfirmDialog__open_true__showModal_does_not_throw", () => {
    expect(() =>
      render(
        <ConfirmDialog
          open
          title="Confirm dry-run wipe"
          body={<p>body</p>}
          onConfirm={() => {}}
          onCancel={() => {}}
        />,
      ),
    ).not.toThrow();
    expect(screen.getByText("Confirm dry-run wipe")).toBeInTheDocument();
    const dialog = document.querySelector("dialog");
    expect(dialog).toBeTruthy();
    expect(dialog?.hasAttribute("open") || (dialog as HTMLDialogElement).open).toBe(
      true,
    );
  });

  it("ConfirmDialog__typed_WIPE__confirm_disabled_until_exact_phrase", async () => {
    const user = userEvent.setup();
    const onConfirm = vi.fn();
    render(
      <ConfirmDialog
        open
        title="Confirm content-envelope wipe"
        body={<p>wipe body</p>}
        typedConfirmPhrase="WIPE"
        confirmLabel="Execute wipe"
        onConfirm={onConfirm}
        onCancel={() => {}}
      />,
    );

    const confirm = screen.getByRole("button", { name: "Execute wipe" });
    expect(confirm).toBeDisabled();

    const input = screen.getByLabelText(/Type WIPE to confirm/i);
    await user.type(input, "WIP");
    expect(confirm).toBeDisabled();
    expect(screen.getByText(/Type WIPE to enable confirm/i)).toBeInTheDocument();

    await user.type(input, "E");
    expect(confirm).not.toBeDisabled();
    expect(screen.getByText("Phrase matched")).toBeInTheDocument();

    await user.click(confirm);
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("ConfirmDialog__typed_WIPE_Enter__focuses_confirm_does_not_auto_submit", async () => {
    const user = userEvent.setup();
    const onConfirm = vi.fn();
    render(
      <ConfirmDialog
        open
        title="Confirm content-envelope wipe"
        body={<p>wipe body</p>}
        typedConfirmPhrase="WIPE"
        confirmLabel="Execute wipe"
        onConfirm={onConfirm}
        onCancel={() => {}}
      />,
    );

    const input = screen.getByLabelText(/Type WIPE to confirm/i);
    await user.type(input, "WIPE");
    await user.keyboard("{Enter}");

    expect(onConfirm).not.toHaveBeenCalled();
    const confirm = screen.getByRole("button", { name: "Execute wipe" });
    expect(confirm).toHaveFocus();
  });

  it("ConfirmDialog__aria_live__polite_match_mismatch_messages", async () => {
    const user = userEvent.setup();
    render(
      <ConfirmDialog
        open
        title="Wipe"
        body={<p>body</p>}
        typedConfirmPhrase="WIPE"
        onConfirm={() => {}}
        onCancel={() => {}}
      />,
    );

    const live = document.querySelector('[aria-live="polite"]');
    expect(live).toBeTruthy();
    expect(live).toHaveTextContent(/Type WIPE to enable confirm/i);

    await user.type(screen.getByLabelText(/Type WIPE to confirm/i), "WIPE");
    expect(live).toHaveTextContent("Phrase matched");
  });

  it("ConfirmDialog__cancel__invokes_onCancel", async () => {
    const user = userEvent.setup();
    const onCancel = vi.fn();
    render(
      <ConfirmDialog
        open
        title="Resolve review item"
        body={<p>item</p>}
        onConfirm={() => {}}
        onCancel={onCancel}
      />,
    );
    await user.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});
