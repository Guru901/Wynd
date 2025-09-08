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

  test("should broadcast subsequent messages to newly joined clients too", async ({
    browser,
  }) => {
    const ctxA = await browser.newContext();
    const pageA = await ctxA.newPage();

    // Connect first client
    await pageA.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);
        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          resolve(undefined);
        };
        ws.onmessage = (e) => window.wsMessages?.push(e.data);
        ws.onerror = reject;
      });
    }, WS_URL);

    // Clear any welcome
    await pageA.evaluate(() => {
      window.wsMessages = [];
    });

    // Send a first message before second client joins
    const beforeJoinMsg = "msg-before-join";
    await pageA.evaluate((msg) => window.testWs!.send(msg), beforeJoinMsg);

    // Wait for it locally and verify
    await pageA.waitForFunction(
      (expected) => window.wsMessages?.includes(expected),
      beforeJoinMsg,
    );

    // Now connect second client
    const ctxB = await browser.newContext();
    const pageB = await ctxB.newPage();
    await pageB.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);
        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          resolve(undefined);
        };
        ws.onmessage = (e) => window.wsMessages?.push(e.data);
        ws.onerror = reject;
      });
    }, WS_URL);

    // Clear any welcome on B
    await pageB.evaluate(() => {
      window.wsMessages = [];
    });

    // Send second message now that both are connected
    const afterJoinMsg = "msg-after-join";
    await pageA.evaluate((msg) => window.testWs!.send(msg), afterJoinMsg);

    // Both A and B should receive the second message
    await Promise.all([
      pageA.waitForFunction(
        (expected) => window.wsMessages?.includes(expected),
        afterJoinMsg,
      ),
      pageB.waitForFunction(
        (expected) => window.wsMessages?.includes(expected),
        afterJoinMsg,
      ),
    ]);

    // B should not have the pre-join message
    const msgsB = await pageB.evaluate(() => window.wsMessages);
    expect(msgsB).not.toContain(beforeJoinMsg);

    await ctxA.close();
    await ctxB.close();
  });
});
