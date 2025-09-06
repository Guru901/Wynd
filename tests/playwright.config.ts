import { defineConfig } from "@playwright/test";

export default defineConfig({
  use: {
    extraHTTPHeaders: {
      "User-Agent": "Playwright-Test",
    },
  },
});
