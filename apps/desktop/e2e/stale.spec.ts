import { expect, test } from "@playwright/test";
import { gotoRoute } from "./helpers/gotoRoute";
import {
  defaultMockTable,
  installMockInvoke,
  loadFixture,
} from "./helpers/mockInvoke";

test.describe("stale StatusBadge (SU9)", () => {
  test("home__project_briefing_stale_freshness__shows_stale_StatusBadge", async ({
    browser,
  }) => {
    const context = await browser.newContext();
    await installMockInvoke(
      context,
      defaultMockTable({
        project_briefing: {
          ok: true,
          value: loadFixture("briefing-stale.json"),
        },
      }),
    );
    const page = await context.newPage();
    await gotoRoute(page, "");

    // Freshness section must paint stale badge with icon + text (not color-only).
    const staleBadge = page.locator('[data-status="stale"]');
    await expect(staleBadge).toBeVisible({ timeout: 5000 });
    await expect(staleBadge).toContainText(/stale/i);
    await expect(staleBadge.locator("svg")).toHaveCount(1);
    await expect(page.getByText(/worst stale/i)).toBeVisible();

    await context.close();
  });
});
