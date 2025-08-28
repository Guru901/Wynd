import { defineConfig } from "@playwright/test";

export default defineConfig({
  use: {
    baseURL: "http://127.0.0.1:8080",
    extraHTTPHeaders: {
      "User-Agent": "Playwright-Test",
    },
  },
});
