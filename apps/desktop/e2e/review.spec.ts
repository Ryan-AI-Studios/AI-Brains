import { expect, test } from "@playwright/test";
import { gotoRoute } from "./helpers/gotoRoute";
import {
  defaultMockTable,
  installMockInvoke,
  loadFixture,
} from "./helpers/mockInvoke";

test.describe("Review screen", () => {
  test("review__empty_list_fixture__shows_empty_message", async ({ browser }) => {
    const context = await browser.newContext();
    await installMockInvoke(
      context,
      defaultMockTable({
        list_review_items: {
          ok: true,
          value: loadFixture("review-empty.json"),
        },
      }),
    );
    const page = await context.newPage();
    await gotoRoute(page, "review");

    // Scope is required — fill scope so list query runs.
    await page.getByPlaceholder("Repository:{uuid}").fill("Repository:e2e-fixture");
    await page.getByRole("button", { name: "Refresh" }).click();

    await expect(
      page.getByText("No review items for this filter."),
    ).toBeVisible();

    await context.close();
  });

  test("review__with_items__resolve_dialog_Escape_cancels_confirm_shows_body", async ({
    browser,
  }) => {
    const context = await browser.newContext();
    await installMockInvoke(
      context,
      defaultMockTable({
        list_review_items: {
          ok: true,
          value: loadFixture("review-items.json"),
        },
      }),
    );
    const page = await context.newPage();
    await gotoRoute(page, "review");

    await page.getByPlaceholder("Repository:{uuid}").fill("Repository:e2e-fixture");
    await page.getByRole("button", { name: "Refresh" }).click();

    await expect(page.getByText("Fixture review subject")).toBeVisible();
    await page.getByRole("button", { name: "approved" }).click();

    const dialog = page.locator("dialog");
    await expect(dialog).toBeVisible();
    await expect(page.getByText("Resolve review item")).toBeVisible();
    await expect(dialog.locator("code").first()).toHaveText("review-item-1");

    // Escape cancels (native dialog + onCancel).
    await page.keyboard.press("Escape");
    await expect(dialog).toBeHidden();

    // Re-open and confirm path shows warnings after success.
    await page.getByRole("button", { name: "approved" }).click();
    await page.getByRole("button", { name: /Resolve as approved/i }).click();
    await expect(page.getByText("Last resolve warnings")).toBeVisible();
    await expect(
      page.getByText("resolution recorded; verify governed state on next briefing"),
    ).toBeVisible();

    await context.close();
  });
});
