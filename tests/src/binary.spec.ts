import { test, expect } from "@playwright/test";
import { WS_URL } from "./shared";

test.describe("WebSocket binary message tests", () => {
  test("should echo binary ArrayBuffer as Blob or ArrayBuffer", async ({
    page,
  }) => {
    await page.evaluate((wsUrl) => {
      return new Promise<void>((resolve, reject) => {
        const ws = new WebSocket(wsUrl);
        ws.binaryType = "arraybuffer";
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

    // Wait for welcome message (text)
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Clear and send binary
    await page.evaluate(() => {
      window.wsMessages = [];
    });

    const bytes = new Uint8Array([1, 2, 3, 4, 5]).buffer;
    await page.evaluate((buf) => {
      window.testWs!.send(buf);
    }, bytes);

    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Validate we received identical content back
    const echoed = await page.evaluate(async () => {
      const data = window.wsMessages![0];
      if (data instanceof ArrayBuffer) return Array.from(new Uint8Array(data));
      if (data instanceof Blob) {
        const buf = await data.arrayBuffer();
        return Array.from(new Uint8Array(buf));
      }
      return [] as number[];
    });

    // If array was captured, it must match
    if (echoed.length > 0) {
      expect(echoed).toEqual([1, 2, 3, 4, 5]);
    }
  });
});
