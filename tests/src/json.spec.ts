import { test, expect } from "@playwright/test";
import { WS_URL } from "./shared";

test.describe("WebSocket JSON message tests", () => {
  test("should echo JSON messages as text payloads", async ({ page }) => {
    // Connect
    await page.evaluate((wsUrl) => {
      return new Promise<void>((resolve, reject) => {
        const ws = new WebSocket(wsUrl);
        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          resolve();
        };
        ws.onmessage = (e) => {
          window.wsMessages!.push(e.data);
        };
        ws.onerror = (err) => reject(err);
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Clear then send JSON
    await page.evaluate(() => {
      window.wsMessages = [];
    });

    const payload = { type: "greet", value: "hello", id: 123 };
    await page.evaluate((obj) => {
      window.testWs!.send(JSON.stringify(obj));
    }, payload);

    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );
    const msg = await page.evaluate(() => window.wsMessages![0] as string);

    // Server is an echo server for text; expect identical string
    expect(msg).toBe(JSON.stringify(payload));
  });

  test("should handle JSON parse round-trip in client", async ({ page }) => {
    await page.evaluate((wsUrl) => {
      return new Promise<void>((resolve, reject) => {
        const ws = new WebSocket(wsUrl);
        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          resolve();
        };
        ws.onmessage = (e) => {
          window.wsMessages!.push(e.data);
        };
        ws.onerror = reject;
      });
    }, WS_URL);

    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );
    await page.evaluate(() => {
      window.wsMessages = [];
    });

    const payload = { kind: "ping", when: Date.now() };
    await page.evaluate(
      (obj) => window.testWs!.send(JSON.stringify(obj)),
      payload
    );

    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );
    const parsed = await page.evaluate(
      () =>
        JSON.parse(String(window.wsMessages![0])) as {
          kind: string;
          when: number;
        }
    );

    expect(parsed.kind).toBe("ping");
    expect(typeof parsed.when).toBe("number");
  });
});
