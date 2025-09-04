import { test, expect } from "@playwright/test";

test.describe("WebSocket error handling tests", () => {
  test("should fail to connect to invalid endpoint", async ({ page }) => {
    const invalidUrl = "ws://localhost:3999/does-not-exist";

    const result = await page.evaluate((wsUrl) => {
      return new Promise<{ opened: boolean; errored: boolean }>((resolve) => {
        const ws = new WebSocket(wsUrl);
        let opened = false;
        let errored = false;
        ws.onopen = () => {
          opened = true;
        };
        ws.onerror = () => {
          errored = true;
          resolve({ opened, errored });
        };
      });
    }, invalidUrl);

    expect(result.opened).toBe(false);
    expect(result.errored).toBe(true);
  });

  test("should receive close event with code when server closes", async ({
    page,
  }) => {
    const WS_URL = "ws://localhost:3000/ws";

    const code = await page.evaluate((wsUrl) => {
      return new Promise<number>((resolve, reject) => {
        const ws = new WebSocket(wsUrl);
        ws.onopen = () => {
          // Immediately request close from client side; server should echo close
          ws.close(1000, "client closing");
        };
        ws.onclose = (e) => {
          resolve(e.code);
        };
        ws.onerror = reject;
      });
    }, WS_URL);

    expect(code).toBe(1000);
  });
});
