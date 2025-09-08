import { test, expect } from "@playwright/test";
import { WS_URL } from "./shared";

test.describe("Broadcast messaging", () => {
  test("should broadcast a text message to all connected clients", async ({
    browser,
  }) => {
    const clientCount = 3;
    const contexts = await Promise.all(
      Array.from({ length: clientCount }, () => browser.newContext()),
    );

    const pages = await Promise.all(contexts.map((ctx) => ctx.newPage()));

    try {
      // Connect all clients
      await Promise.all(
        pages.map((page) =>
          page.evaluate((wsUrl) => {
            return new Promise((resolve, reject) => {
              const ws = new WebSocket(wsUrl);
              ws.onopen = () => {
                window.testWs = ws;
                window.wsMessages = [];
                resolve(undefined);
              };
              ws.onmessage = (e) => {
                window.wsMessages?.push(e.data);
              };
              ws.onerror = reject;
            });
          }, WS_URL),
        ),
      );

      // Wait for any initial/welcome messages and clear them
      await Promise.all(
        pages.map((page) =>
          page
            .waitForFunction(
              () => window.wsMessages && window.wsMessages.length >= 0,
            )
            .then(() =>
              page.evaluate(() => {
                window.wsMessages = [];
              }),
            ),
        ),
      );

      const message = "broadcast-hello";

      // Send from the first client
      await pages[0].evaluate((msg) => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.send(msg);
        }
      }, message);

      // All clients should receive it (including sender)
      await Promise.all(
        pages.map((page) =>
          page.waitForFunction(
            (expected) =>
              Array.isArray(window.wsMessages) &&
              window.wsMessages.includes(expected),
            message,
            { timeout: 5000 },
          ),
        ),
      );

      const results = await Promise.all(
        pages.map((page) => page.evaluate(() => window.wsMessages)),
      );
      results.forEach((msgs) => {
        // Received exactly one copy
        const copies = msgs!.filter((m: any) => m === message).length;
        expect(copies).toBe(1);
      });
    } finally {
      await Promise.all(contexts.map((ctx) => ctx.close()));
    }
  });
});
