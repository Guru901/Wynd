import { defineConfig } from "@playwright/test";

export default defineConfig({
  // Run tests sequentially to avoid interference between tests
  workers: 1,
  // Retry flaky tests a couple of times
  retries: 2,
  // Global test timeout (per test)
  timeout: 30_000,
  use: {
    extraHTTPHeaders: {
      "User-Agent": "Playwright-Test",
    },
    // Give assertions a bit more time in CI-like environments
    actionTimeout: 10_000,
    navigationTimeout: 10_000,
  },
});
