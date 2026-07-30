import { expect, test } from "@playwright/test";
import { gotoRoute } from "./helpers/gotoRoute";
import {
  defaultMockTable,
  installMockInvoke,
  loadFixture,
} from "./helpers/mockInvoke";

test.describe("Source locator honesty (D26)", () => {
  test("source__https_locator__shows_Open_URL_button", async ({ browser }) => {
    const context = await browser.newContext();
    await installMockInvoke(
      context,
      defaultMockTable({
        inspect_source: {
          ok: true,
          value: loadFixture("source-https.json"),
        },
      }),
    );
    const page = await context.newPage();
    await gotoRoute(page, "source/src-https-1");

    await page.getByPlaceholder("Repository:{uuid}").fill("Repository:e2e-fixture");
    await page.getByRole("button", { name: "Inspect" }).click();

    await expect(page.getByRole("button", { name: "Open URL" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Reveal path" })).toHaveCount(0);
    await expect(page.getByText("No locator available")).toHaveCount(0);

    await context.close();
  });

  test("source__file_path_locator__shows_Reveal_path_button", async ({
    browser,
  }) => {
    const context = await browser.newContext();
    await installMockInvoke(
      context,
      defaultMockTable({
        inspect_source: {
          ok: true,
          value: loadFixture("source-path.json"),
        },
      }),
    );
    const page = await context.newPage();
    await gotoRoute(page, "source/src-path-1");

    await page.getByPlaceholder("Repository:{uuid}").fill("Repository:e2e-fixture");
    await page.getByRole("button", { name: "Inspect" }).click();

    await expect(page.getByRole("button", { name: "Reveal path" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Open URL" })).toHaveCount(0);

    await context.close();
  });

  test("source__null_locator__No_locator_available_no_Open_or_Reveal", async ({
    browser,
  }) => {
    const context = await browser.newContext();
    await installMockInvoke(
      context,
      defaultMockTable({
        inspect_source: {
          ok: true,
          value: loadFixture("source-missing.json"),
        },
      }),
    );
    const page = await context.newPage();
    await gotoRoute(page, "source/src-none-1");

    await page.getByPlaceholder("Repository:{uuid}").fill("Repository:e2e-fixture");
    await page.getByRole("button", { name: "Inspect" }).click();

    await expect(page.getByText("No locator available")).toBeVisible();
    await expect(page.getByRole("button", { name: "Open URL" })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Reveal path" })).toHaveCount(0);

    await context.close();
  });

  test("source__missing_locator_property__No_locator_available_no_Open_or_Reveal", async ({
    browser,
  }) => {
    const context = await browser.newContext();
    await installMockInvoke(
      context,
      defaultMockTable({
        inspect_source: {
          ok: true,
          value: loadFixture("source-no-locator-key.json"),
        },
      }),
    );
    const page = await context.newPage();
    await gotoRoute(page, "source/src-no-key-1");

    await page.getByPlaceholder("Repository:{uuid}").fill("Repository:e2e-fixture");
    await page.getByRole("button", { name: "Inspect" }).click();

    await expect(page.getByText("No locator available")).toBeVisible();
    await expect(page.getByRole("button", { name: "Open URL" })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Reveal path" })).toHaveCount(0);

    await context.close();
  });

  test("source__http_locator__display_only_no_Open_or_Reveal", async ({
    browser,
  }) => {
    const context = await browser.newContext();
    await installMockInvoke(
      context,
      defaultMockTable({
        inspect_source: {
          ok: true,
          value: loadFixture("source-http.json"),
        },
      }),
    );
    const page = await context.newPage();
    await gotoRoute(page, "source/src-http-1");

    await page.getByPlaceholder("Repository:{uuid}").fill("Repository:e2e-fixture");
    await page.getByRole("button", { name: "Inspect" }).click();

    await expect(page.getByText(/Locator \(display only\)/i)).toBeVisible();
    await expect(
      page.locator(".locator-row code").filter({ hasText: "http://evil.example" }),
    ).toBeVisible();
    await expect(page.getByRole("button", { name: "Open URL" })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Reveal path" })).toHaveCount(0);

    await context.close();
  });
});
