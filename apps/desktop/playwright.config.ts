import { defineConfig, devices } from "@playwright/test";

const baseURL = "http://127.0.0.1:4173";

/**
 * L3/L4 desktop E2E: Chromium only, build+preview webServer, HashRouter routes.
 * Snapshot updates are manual via `npm run test:e2e:update` — never silent in CI.
 */
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "list",
  timeout: 60_000,
  expect: {
    timeout: 10_000,
    toHaveScreenshot: {
      // Advisory pixel compare only for wipe dialog; ARIA is primary.
      maxDiffPixelRatio: 0.02,
    },
  },
  use: {
    baseURL,
    trace: "on-first-retry",
    video: "retain-on-failure",
    headless: true,
    viewport: { width: 1280, height: 720 },
    launchOptions: {
      args: ["--font-render-hinting=none"],
    },
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  webServer: {
    // MUST build then preview only — never vite dev (HMR / unsafe-inline).
    command: "node ./scripts/e2e-serve.mjs",
    url: baseURL,
    reuseExistingServer: !process.env.CI,
    timeout: 180_000,
  },
});
