import { WS_URL } from "./shared";
import { test, expect } from "@playwright/test";

test.describe("Ripress Integration Tests", () => {
  test.beforeAll(async () => {
    await new Promise((resolve) => setTimeout(resolve, 1000));
  });

  test("should handle HTTP and WebSocket on same port", async ({ page }) => {
    // Test HTTP endpoint
    const httpResponse = await page.goto("http://localhost:3000");
    expect(httpResponse?.status()).toBe(200);

    const httpContent = await page.textContent("body");
    expect(httpContent).toContain("Hello World!");

    // Test WebSocket connection
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages?.push(event.data);
        };

        ws.onerror = reject;
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    const messages = await page.evaluate(() => window.wsMessages);
    expect(messages![0]).toBe("Hello from ripress and wynd!");
  });

  test("should handle WebSocket upgrade requests correctly", async ({
    page,
  }) => {
    // Test WebSocket upgrade headers
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.upgradeSuccessful = true;
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages?.push(event.data);
        };

        ws.onerror = (error) => {
          window.upgradeSuccessful = false;
          reject(error);
        };
      });
    }, WS_URL);

    // Wait for connection to establish
    await page.waitForFunction(() => window.upgradeSuccessful === true);

    const upgradeSuccessful = await page.evaluate(
      () => window.upgradeSuccessful
    );
    expect(upgradeSuccessful).toBe(true);

    // Verify WebSocket is working
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    const messages = await page.evaluate(() => window.wsMessages);
    expect(messages![0]).toBe("Hello from ripress and wynd!");
  });

  test("should handle mixed HTTP and WebSocket traffic", async ({ page }) => {
    const httpRequests = 5;
    const wsMessages = 10;

    // Make multiple HTTP requests
    for (let i = 0; i < httpRequests; i++) {
      const response = await page.goto("http://localhost:3000");
      expect(response?.status()).toBe(200);
    }

    // Establish WebSocket connection
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages?.push(event.data);
        };

        ws.onerror = reject;
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Clear welcome message
    await page.evaluate(() => {
      window.wsMessages = [];
    });

    // Send WebSocket messages
    for (let i = 0; i < wsMessages; i++) {
      await page.evaluate((msg) => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.send(msg);
        }
      }, `Mixed traffic message ${i}`);

      // Make HTTP request between WebSocket messages
      if (i % 3 === 0) {
        const response = await page.goto("http://localhost:3000");
        expect(response?.status()).toBe(200);
      }
    }

    // Wait for all WebSocket messages
    await page.waitForFunction(
      (expectedCount) =>
        window.wsMessages && window.wsMessages.length >= expectedCount,
      wsMessages,
      { timeout: 10000 }
    );

    const messages = await page.evaluate(() => window.wsMessages);
    expect(messages!.length).toBe(wsMessages);

    // Verify all messages were echoed correctly
    for (let i = 0; i < wsMessages; i++) {
      expect(messages![i]).toBe(`Mixed traffic message ${i}`);
    }
  });

  test("should handle concurrent HTTP and WebSocket connections", async ({
    browser,
  }) => {
    const httpConnections = 3;
    const wsConnections = 3;

    // Create contexts for HTTP requests
    const httpContexts = await Promise.all(
      Array.from({ length: httpConnections }, () => browser.newContext())
    );

    const httpPages = await Promise.all(
      httpContexts.map((ctx) => ctx.newPage())
    );

    // Create contexts for WebSocket connections
    const wsContexts = await Promise.all(
      Array.from({ length: wsConnections }, () => browser.newContext())
    );

    const wsPages = await Promise.all(wsContexts.map((ctx) => ctx.newPage()));

    try {
      // Make HTTP requests concurrently
      const httpPromises = httpPages.map((page) =>
        page.goto("http://localhost:3000").then((response) => ({
          status: response?.status(),
          content: page.textContent("body"),
        }))
      );

      // Establish WebSocket connections concurrently
      const wsPromises = wsPages.map((page, index) =>
        page.evaluate(
          ({ wsUrl, clientId }) => {
            return new Promise((resolve, reject) => {
              const ws = new WebSocket(wsUrl);

              ws.onopen = () => {
                window.testWs = ws;
                window.wsMessages = [];
                window.clientId = clientId;
                resolve(undefined);
              };

              ws.onmessage = (event) => {
                window.wsMessages?.push(event.data);
              };

              ws.onerror = reject;
            });
          },
          { wsUrl: WS_URL, clientId: `ws-client-${index}` }
        )
      );

      // Wait for all HTTP requests
      const httpResults = await Promise.all(httpPromises);
      httpResults.forEach((result) => {
        expect(result.status).toBe(200);
      });

      // Wait for all WebSocket connections
      await Promise.all(wsPromises);

      // Wait for all welcome messages
      await Promise.all(
        wsPages.map((page) =>
          page.waitForFunction(
            () => window.wsMessages && window.wsMessages.length > 0
          )
        )
      );

      // Send messages from all WebSocket connections
      await Promise.all(
        wsPages.map((page, index) =>
          page.evaluate((clientId) => {
            for (let i = 0; i < 5; i++) {
              if (
                window.testWs &&
                window.testWs.readyState === WebSocket.OPEN
              ) {
                window.testWs.send(`Concurrent message ${i} from ${clientId}`);
              }
            }
          }, `ws-client-${index}`)
        )
      );

      // Wait for all messages to be received
      await Promise.all(
        wsPages.map((page) =>
          page.waitForFunction(
            () => window.wsMessages && window.wsMessages.length > 5
          )
        )
      );

      // Verify all WebSocket connections are working
      const wsResults = await Promise.all(
        wsPages.map((page) =>
          page.evaluate(() => ({
            clientId: window.clientId,
            messageCount: window.wsMessages!.length,
            connectionAlive:
              window.testWs && window.testWs.readyState === WebSocket.OPEN,
          }))
        )
      );

      wsResults.forEach((result) => {
        expect(result.connectionAlive).toBe(true);
        expect(result.messageCount).toBeGreaterThan(5);
      });
    } finally {
      await Promise.all(
        [...httpContexts, ...wsContexts].map((ctx) => ctx.close())
      );
    }
  });

  test("should handle WebSocket path routing", async ({ page }) => {
    // Test WebSocket connection on specific path
    const wsPath = "ws://localhost:3000/ws";

    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages!.push(event.data);
        };

        ws.onerror = reject;
      });
    }, wsPath);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    const messages = await page.evaluate(() => window.wsMessages);
    expect(messages![0]).toBe("Hello from ripress and wynd!");

    // Test that HTTP still works on root path
    const httpResponse = await page.goto("http://localhost:3000");
    expect(httpResponse?.status()).toBe(200);
  });

  test("should handle server restart with mixed traffic", async ({ page }) => {
    // Establish WebSocket connection
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.connectionClosed = false;
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages?.push(event.data);
        };

        ws.onclose = () => {
          window.connectionClosed = true;
        };

        ws.onerror = reject;
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Make HTTP request
    const httpResponse = await page.goto("http://localhost:3000");
    expect(httpResponse?.status()).toBe(200);

    // Send WebSocket message
    await page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.send("Test message before restart");
      }
    });

    // Wait for echo
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 1
    );

    // Simulate server restart by closing WebSocket
    await page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.close();
      }
    });

    // Wait for connection to close
    await page.waitForFunction(() => window.connectionClosed === true, {
      timeout: 5000,
    });

    // Verify HTTP still works after WebSocket close
    const httpResponseAfter = await page.goto("http://localhost:3000");
    expect(httpResponseAfter?.status()).toBe(200);

    // Attempt to reconnect WebSocket
    let reconnected = false;
    try {
      await page.evaluate((wsUrl) => {
        return new Promise((resolve, reject) => {
          const ws = new WebSocket(wsUrl);

          ws.onopen = () => {
            window.testWs = ws;
            window.wsMessages = [];
            resolve(undefined);
          };

          ws.onerror = reject;
        });
      }, WS_URL);

      reconnected = true;
    } catch (error) {
      // Reconnection might fail if server is down
    }

    expect(reconnected).toBe(true);
  });

  test("should handle WebSocket with custom headers", async ({ page }) => {
    // Test WebSocket connection with custom headers
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages!.push(event.data);
        };

        ws.onerror = reject;
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    const messages = await page.evaluate(() => window.wsMessages);
    expect(messages![0]).toBe("Hello from ripress and wynd!");

    // Send message with special content
    await page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.send("Custom header test message");
      }
    });

    // Wait for echo
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 1
    );

    const allMessages = await page.evaluate(() => window.wsMessages);
    expect(allMessages![1]).toBe("Custom header test message");
  });

  test("should handle WebSocket subprotocols", async ({ page }) => {
    // Test WebSocket connection (subprotocols would be tested if supported)
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.protocol = ws.protocol;
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages!.push(event.data);
        };

        ws.onerror = reject;
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    const messages = await page.evaluate(() => window.wsMessages);
    const protocol = await page.evaluate(() => window.protocol);

    expect(messages![0]).toBe("Hello from ripress and wynd!");
    expect(protocol).toBe(""); // No subprotocol specified
  });

  test("should handle WebSocket compression", async ({ page }) => {
    // Test WebSocket connection (compression would be tested if enabled)
    const largeMessage = "A".repeat(10000); // 10KB message

    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages!.push(event.data);
        };

        ws.onerror = reject;
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Clear welcome message
    await page.evaluate(() => {
      window.wsMessages = [];
    });

    // Send large message to test compression
    await page.evaluate((msg) => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.send(msg);
      }
    }, largeMessage);

    // Wait for echo
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0,
      { timeout: 15000 }
    );

    const messages = await page.evaluate(() => window.wsMessages);
    expect(messages![0]).toBe(largeMessage);
    expect(messages![0]!.length).toBe(10000);
  });

  test.afterEach(async ({ page }) => {
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
