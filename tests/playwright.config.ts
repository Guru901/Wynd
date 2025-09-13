import { defineConfig } from "@playwright/test";

export default defineConfig({
  // Run tests sequentially to avoid interference between tests
  workers: 1,
  use: {
    extraHTTPHeaders: {
      "User-Agent": "Playwright-Test",
    },
  },
});
