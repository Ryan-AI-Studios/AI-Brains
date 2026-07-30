import { expect, test } from "@playwright/test";
import { gotoRoute } from "./helpers/gotoRoute";
import {
  defaultMockTable,
  installMockInvoke,
  loadFixture,
} from "./helpers/mockInvoke";

test.describe("Erasure screen", () => {
  test("erasure__dry_run__confirm_without_typed_WIPE", async ({ browser }) => {
    const context = await browser.newContext();
    await installMockInvoke(
      context,
      defaultMockTable({
        wipe_content_envelope: {
          ok: true,
          value: loadFixture("wipe-dry-run.json"),
        },
      }),
    );
    const page = await context.newPage();
    await gotoRoute(page, "erasure");

    await page.getByRole("textbox", { name: "content_key_id" }).fill(
      "00000000-0000-0000-0000-000000000001",
    );
    // Wipe-section scope (ticket path has a separate "Repository:{uuid}" field).
    await page
      .getByPlaceholder("Scope identity key")
      .fill("Repository:e2e-fixture");

    // dry_run is default true.
    await page.getByRole("button", { name: /Preview wipe \(dry-run\)/i }).click();

    const dialog = page.locator("dialog");
    await expect(dialog).toBeVisible();
    await expect(page.getByText("Confirm dry-run wipe")).toBeVisible();
    // No typed WIPE field for dry-run.
    await expect(page.getByLabel(/Type WIPE to confirm/i)).toHaveCount(0);

    await page.getByRole("button", { name: "Run dry-run" }).click();
    await expect(page.getByText(/status:/i)).toBeVisible();
    await expect(page.getByText("Wipe honesty warnings")).toBeVisible();

    await context.close();
  });

  test("erasure__execute__needs_WIPE_Enter_no_auto_submit_Escape_cancels", async ({
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

    // Uncheck dry_run for execute path.
    await page.getByLabel(/dry_run \(default true\)/i).uncheck();
    await page.getByRole("button", { name: /Wipe \(type WIPE to confirm\)/i }).click();

    const dialog = page.locator("dialog");
    await expect(dialog).toBeVisible();
    await expect(page.getByText("Confirm content-envelope wipe")).toBeVisible();

    const confirm = page.getByRole("button", { name: "Execute wipe" });
    await expect(confirm).toBeDisabled();

    const phrase = page.getByLabel(/Type WIPE to confirm/i);
    await phrase.fill("WIPE");
    await expect(confirm).toBeEnabled();
    await expect(page.getByText("Phrase matched")).toBeVisible();

    // Enter focuses Confirm — does not auto-submit.
    await phrase.press("Enter");
    await expect(confirm).toBeFocused();
    // Dialog still open; no result yet.
    await expect(dialog).toBeVisible();

    // Escape cancels.
    await page.keyboard.press("Escape");
    await expect(dialog).toBeHidden();

    await context.close();
  });

  test("erasure__execute__WIPE_confirm__dialog_closes_success_and_honesty", async ({
    browser,
  }) => {
    const context = await browser.newContext();
    await installMockInvoke(
      context,
      defaultMockTable({
        wipe_content_envelope: {
          ok: true,
          value: loadFixture("wipe-execute.json"),
        },
      }),
    );
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

    const phrase = page.getByLabel(/Type WIPE to confirm/i);
    await phrase.fill("WIPE");
    await page.getByRole("button", { name: "Execute wipe" }).click();

    // Dialog closes on success; result box shows wiped status + honesty.
    await expect(dialog).toBeHidden();
    const result = page.locator(".result-box");
    await expect(result.getByText(/status:/i)).toBeVisible();
    await expect(result.getByText("wiped")).toBeVisible();
    await expect(result.getByText(/wrap_destroyed:\s*true/i)).toBeVisible();
    await expect(result.getByText("Wipe honesty warnings")).toBeVisible();
    await expect(
      result.getByText(/not NIST Purge\/Destroy/i),
    ).toBeVisible();

    await context.close();
  });
});

