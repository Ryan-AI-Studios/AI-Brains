import { expect, test } from "@playwright/test";
import { gotoRoute } from "./helpers/gotoRoute";
import {
  defaultMockTable,
  installMockInvoke,
  loadFixture,
} from "./helpers/mockInvoke";

/**
 * L4 visual: ARIA snapshots primary (@visual tag).
 * Optional pixel screenshot for wipe dialog only (pinned viewport).
 */
test.describe("visual @visual", () => {
  test("visual__offline_home__aria_snapshot @visual", async ({ browser }) => {
    const context = await browser.newContext();
    await installMockInvoke(context, defaultMockTable());
    const page = await context.newPage();
    await gotoRoute(page, "");

    await expect(page.locator('[data-status="offline"]')).toBeVisible();
    await expect(page.locator(".state-panel.state-offline")).toMatchAriaSnapshot(`
      - alert:
        - heading /Offline.*Daemon offline/ [level=3]
        - paragraph: /daemon unreachable/
        - button "Retry"
    `);

    await context.close();
  });

  test("visual__denied_home__aria_snapshot @visual", async ({ browser }) => {
    const denied = loadFixture("denied-error.json");
    const context = await browser.newContext();
    await installMockInvoke(
      context,
      defaultMockTable({
        project_briefing: { ok: false, error: denied },
        personal_briefing: { ok: false, error: denied },
      }),
    );
    const page = await context.newPage();
    await gotoRoute(page, "");

    await expect(page.locator('[data-status="denied"]')).toBeVisible();
    await expect(page.locator(".state-panel.state-denied")).toMatchAriaSnapshot(`
      - alert:
        - heading /Denied.*Access denied/ [level=3]
        - paragraph: /session token missing|not authorized|token/
        - button "Retry"
    `);

    await context.close();
  });

  test("visual__empty_review__aria_snapshot @visual", async ({ browser }) => {
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
    await page
      .getByRole("textbox", { name: /Scope \(required\)/i })
      .fill("Repository:e2e-fixture");
    await page.getByRole("button", { name: "Refresh" }).click();
    await expect(
      page.getByText("No review items for this filter."),
    ).toBeVisible();

    await expect(page.locator(".state-panel.state-empty")).toMatchAriaSnapshot(`
      - status:
        - paragraph: No review items for this filter.
    `);

    await context.close();
  });

  test("visual__wipe_dialog_structure__aria_and_optional_pixel @visual", async ({
    browser,
  }) => {
    const context = await browser.newContext();
    await installMockInvoke(context, defaultMockTable());
    const page = await context.newPage();
    await gotoRoute(page, "erasure");

    await page.getByRole("textbox", { name: "content_key_id" }).fill(
      "00000000-0000-0000-0000-000000000001",
    );
    await page
      .getByPlaceholder("Scope identity key")
      .fill("Repository:e2e-fixture");
    await page.getByLabel(/dry_run \(default true\)/i).uncheck();
    await page.getByRole("button", { name: /Wipe \(type WIPE to confirm\)/i }).click();

    const dialog = page.locator("dialog");
    await expect(dialog).toBeVisible();

    await expect(dialog).toMatchAriaSnapshot(`
      - dialog "Confirm content-envelope wipe":
        - heading "Confirm content-envelope wipe" [level=2]
        - paragraph: /content_key_id/
        - paragraph: /scope/
        - paragraph: /dry_run=false/
        - paragraph: /cryptographic erasure|Not NIST/
        - heading "Honesty (contract)" [level=3]
        - list:
          - listitem: /not NIST Purge/
          - listitem: /backups/
          - listitem: /erasure ticket/
          - listitem: /cryptographic erasure/
          - listitem: /SQLCipher/
        - text: /Type/
        - code: WIPE
        - text: /to confirm/
        - textbox /Type WIPE/
        - paragraph: /Type WIPE to enable confirm/
        - button "Cancel"
        - button "Execute wipe" [disabled]
    `);

    // Optional pixel golden — wipe dialog only, pinned viewport (1280x720).
    await expect(dialog).toHaveScreenshot("wipe-dialog.png", {
      animations: "disabled",
    });

    await context.close();
  });

  test("visual__connectors_unavailable__aria_snapshot @visual", async ({
    browser,
  }) => {
    const context = await browser.newContext();
    await installMockInvoke(context, defaultMockTable());
    const page = await context.newPage();
    await gotoRoute(page, "connectors");

    await expect(page.locator(".state-panel.state-unavailable")).toBeVisible();
    await expect(page.locator(".state-panel.state-unavailable")).toMatchAriaSnapshot(`
      - status:
        - heading /Unavailable/ [level=3]
        - paragraph: /Connectors are unavailable/
    `);

    await context.close();
  });
});
