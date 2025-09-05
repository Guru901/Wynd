import { WS_URL } from "./shared";
import { test, expect } from "@playwright/test";

test.describe("WebSocket Connection Management", () => {
  test.beforeAll(async () => {
    // Add a small delay to ensure server is ready
    await new Promise((resolve) => setTimeout(resolve, 1000));
  });

  test("should establish connection and receive welcome message", async ({
    page,
  }) => {
    let connectionEstablished = false;
    let welcomeMessage = "";

    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.wsConnected = true;
          connectionEstablished = true;
          resolve();
        };

        ws.onmessage = (event) => {
          window.wsMessages.push(event.data);
          if (window.wsMessages.length === 1) {
            welcomeMessage = event.data;
          }
        };

        ws.onerror = (error) => {
          reject(error);
        };

        ws.onclose = () => {
          window.wsConnected = false;
        };
      });
    }, WS_URL);

    // Wait for connection and initial message
    await page.waitForFunction(
      () => globalThis.wsMessages && globalThis.wsMessages.length > 0
    );

    const messages = await page.evaluate(() => globalThis.wsMessages);
    const isConnected = await page.evaluate(() => globalThis.wsConnected);

    expect(isConnected).toBe(true);
    expect(connectionEstablished).toBe(true);
    expect(messages[0]).toBe("Hello from ripress and wynd!");
  });

  test("should handle multiple concurrent connections", async ({ browser }) => {
    const connectionCount = 5;
    const contexts = await Promise.all(
      Array.from({ length: connectionCount }, () => browser.newContext())
    );

    const pages = await Promise.all(contexts.map((ctx) => ctx.newPage()));

    try {
      // Connect all clients simultaneously
      await Promise.all(
        pages.map((page, index) =>
          page.evaluate(
            ({ wsUrl, clientId }) => {
              return new Promise((resolve, reject) => {
                const ws = new WebSocket(wsUrl);

                ws.onopen = () => {
                  window.testWs = ws;
                  window.wsMessages = [];
                  window.clientId = clientId;
                  window.connectionTime = Date.now();
                  resolve();
                };

                ws.onmessage = (event) => {
                  window.wsMessages.push(event.data);
                };

                ws.onerror = reject;
              });
            },
            { wsUrl: WS_URL, clientId: `client-${index}` }
          )
        )
      );

      // Wait for all welcome messages
      await Promise.all(
        pages.map((page) =>
          page.waitForFunction(
            () => window.wsMessages && window.wsMessages.length > 0
          )
        )
      );

      // Verify all connections received welcome messages
      const results = await Promise.all(
        pages.map((page) =>
          page.evaluate(() => ({
            clientId: window.clientId,
            messages: window.wsMessages,
            connectionTime: window.connectionTime,
          }))
        )
      );

      results.forEach((result, index) => {
        expect(result.clientId).toBe(`client-${index}`);
        expect(result.messages[0]).toBe("Hello from ripress and wynd!");
        expect(result.connectionTime).toBeGreaterThan(0);
      });
    } finally {
      // Clean up all contexts
      await Promise.all(contexts.map((ctx) => ctx.close()));
    }
  });

  test("should handle rapid connection/disconnection cycles", async ({
    page,
  }) => {
    const cycleCount = 10;
    const connectionTimes: number[] = [];
    const disconnectionTimes: number[] = [];

    for (let i = 0; i < cycleCount; i++) {
      const connectionStart = Date.now();

      await page.evaluate((wsUrl) => {
        return new Promise((resolve, reject) => {
          const ws = new WebSocket(wsUrl);

          ws.onopen = () => {
            window.testWs = ws;
            window.wsMessages = [];
            resolve();
          };

          ws.onmessage = (event) => {
            window.wsMessages.push(event.data);
          };

          ws.onerror = reject;
        });
      }, WS_URL);

      // Wait for welcome message
      await page.waitForFunction(
        () => window.wsMessages && window.wsMessages.length > 0
      );

      const connectionTime = Date.now() - connectionStart;
      connectionTimes.push(connectionTime);

      // Verify welcome message
      const messages = await page.evaluate(() => window.wsMessages);
      expect(messages[0]).toBe("Hello from ripress and wynd!");

      // Close connection
      const disconnectionStart = Date.now();
      await page.evaluate(() => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.close();
        }
      });

      // Wait for connection to close
      await page.waitForFunction(
        () => !window.testWs || window.testWs.readyState === WebSocket.CLOSED
      );

      const disconnectionTime = Date.now() - disconnectionStart;
      disconnectionTimes.push(disconnectionTime);

      // Small delay between cycles
      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    // Verify connection times are reasonable (less than 1 second)
    connectionTimes.forEach((time) => {
      expect(time).toBeLessThan(1000);
    });

    // Verify disconnection times are reasonable (less than 500ms)
    disconnectionTimes.forEach((time) => {
      expect(time).toBeLessThan(500);
    });
  });

  test("should maintain connection stability over time", async ({ page }) => {
    const testDuration = 30000; // 30 seconds
    const startTime = Date.now();

    // Establish connection
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.messageCount = 0;
          window.lastMessageTime = Date.now();
          resolve();
        };

        ws.onmessage = (event) => {
          window.wsMessages.push(event.data);
          window.messageCount++;
          window.lastMessageTime = Date.now();
        };

        ws.onerror = reject;

        ws.onclose = () => {
          window.wsConnected = false;
        };
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Send periodic ping messages
    const pingInterval = setInterval(async () => {
      await page.evaluate(() => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.send(`ping-${Date.now()}`);
        }
      });
    }, 1000);

    // Wait for test duration
    await new Promise((resolve) => setTimeout(resolve, testDuration));

    clearInterval(pingInterval);

    // Verify connection is still alive
    const isConnected = await page.evaluate(
      () => globalThis.wsConnected !== false
    );
    const messageCount = await page.evaluate(() => globalThis.messageCount);
    const lastMessageTime = await page.evaluate(
      () => globalThis.lastMessageTime
    );

    expect(isConnected).toBe(true);
    expect(messageCount).toBeGreaterThan(0);
    expect(lastMessageTime).toBeGreaterThan(startTime);

    // Clean up
    await page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.close();
      }
    });
  });

  test("should handle connection close gracefully", async ({ page }) => {
    let connectionClosed = false;
    let closeCode: number | undefined;
    let closeReason: string | undefined;

    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.connectionClosed = false;
          resolve();
        };

        ws.onmessage = (event) => {
          window.wsMessages.push(event.data);
        };

        ws.onclose = (event) => {
          window.connectionClosed = true;
          window.closeCode = event.code;
          window.closeReason = event.reason;
        };

        ws.onerror = reject;
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Close connection with specific code and reason
    await page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.close(1000, "Test close");
      }
    });

    // Wait for close event
    await page.waitForFunction(() => window.connectionClosed === true);

    const isClosed = await page.evaluate(() => window.connectionClosed);
    const code = await page.evaluate(() => window.closeCode);
    const reason = await page.evaluate(() => window.closeReason);

    expect(isClosed).toBe(true);
    expect(code).toBe(1000);
    expect(reason).toBe("Test close");
  });

  test("should handle server-side connection termination", async ({ page }) => {
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.serverClosed = false;
          resolve();
        };

        ws.onmessage = (event) => {
          window.wsMessages.push(event.data);
        };

        ws.onclose = (event) => {
          window.serverClosed = true;
          window.serverCloseCode = event.code;
        };

        ws.onerror = reject;
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Send a special message that might trigger server-side close
    await page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.send("CLOSE_CONNECTION");
      }
    });

    // Wait for server-side close (if implemented)
    // This test documents expected behavior - server may or may not close
    await page.waitForFunction(
      () => window.serverClosed === true || window.wsMessages.length > 1,
      { timeout: 5000 }
    );

    // Verify either server closed or message was processed
    const serverClosed = await page.evaluate(() => window.serverClosed);
    const messageCount = await page.evaluate(() => window.wsMessages.length);

    expect(serverClosed || messageCount > 1).toBe(true);
  });

  test.afterEach(async ({ page }) => {
    // Clean up WebSocket connections
    await page
      .evaluate(() => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.close();
        }
      })
      .catch(() => {
        // Ignore errors during cleanup
      });
  });
});
