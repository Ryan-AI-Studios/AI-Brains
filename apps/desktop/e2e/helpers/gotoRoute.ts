import type { Page } from "@playwright/test";

/**
 * Navigate via HashRouter. NEVER use page.goto('/path') alone —
 * HashRouter requires `#/route` form.
 */
export async function gotoRoute(
  page: Page,
  route: string,
  base = "http://127.0.0.1:4173",
): Promise<void> {
  const normalized = route.replace(/^\/+/, "");
  const hash = normalized === "" ? "#/" : `#/${normalized}`;
  await page.goto(`${base}${hash}`);
}
