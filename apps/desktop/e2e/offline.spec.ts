import { expect, test } from "@playwright/test";
import { gotoRoute } from "./helpers/gotoRoute";
import { defaultMockTable, installMockInvoke } from "./helpers/mockInvoke";

test.describe("offline Home", () => {
  test("home__project_briefing_offline__shows_Offline_StatusBadge_promptly", async ({
    browser,
  }) => {
    const context = await browser.newContext();
    await installMockInvoke(context, defaultMockTable());
    const page = await context.newPage();

    await gotoRoute(page, "");

    // DT13 / B3: init-script order smoke — mock installed before app eval.
    const tauriReady = await page.evaluate(() => {
      const w = window as Window & {
        __TAURI_INTERNALS__?: { invoke?: unknown };
      };
      return typeof w.__TAURI_INTERNALS__?.invoke === "function";
    });
    expect(tauriReady).toBe(true);

    // Offline must paint without retry delay (queryClient retry:false).
    const offlineBadge = page.locator('[data-status="offline"]');
    await expect(offlineBadge).toBeVisible({ timeout: 5000 });
    await expect(offlineBadge).toContainText("Offline");
    await expect(page.getByText("Daemon offline")).toBeVisible();
    // Icon present (not color-only).
    await expect(offlineBadge.locator("svg")).toHaveCount(1);

    await context.close();
  });
});
