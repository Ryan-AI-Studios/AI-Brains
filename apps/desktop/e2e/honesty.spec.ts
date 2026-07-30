import { expect, test } from "@playwright/test";
import { gotoRoute } from "./helpers/gotoRoute";
import { defaultMockTable, installMockInvoke } from "./helpers/mockInvoke";

test.describe("Honest unavailable surfaces", () => {
  test("connectors__unavailable_copy__shown", async ({ browser }) => {
    const context = await browser.newContext();
    await installMockInvoke(context, defaultMockTable());
    const page = await context.newPage();
    await gotoRoute(page, "connectors");

    await expect(page.getByRole("heading", { name: "Connectors" })).toBeVisible();
    await expect(page.locator('[data-status="unavailable"]')).toBeVisible();
    await expect(
      page.getByText(/Connectors are unavailable in this desktop build/i),
    ).toBeVisible();

    await context.close();
  });

  test("erasure__retention_plan__honest_unavailable_copy", async ({
    browser,
  }) => {
    const context = await browser.newContext();
    await installMockInvoke(context, defaultMockTable());
    const page = await context.newPage();
    await gotoRoute(page, "erasure");

    await expect(
      page.getByRole("heading", { name: "Retention plan" }),
    ).toBeVisible();
    await expect(
      page.getByText(/Honest unavailable — class-based retention plan UI/i),
    ).toBeVisible();
    await expect(
      page.getByText(/Do not invent retention status in the client/i),
    ).toBeVisible();

    await context.close();
  });
});
