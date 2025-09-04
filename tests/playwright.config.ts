import { defineConfig } from "@playwright/test";

export default defineConfig({
  use: {
    baseURL: "ws://127.0.0.1:3000",
    extraHTTPHeaders: {
      "User-Agent": "Playwright-Test",
    },
  },
});
