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
    let welcomeMessage = "";

    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.wsConnected = true;
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages?.push(event.data);
          if (window.wsMessages?.length === 1) {
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
      () => window.wsMessages && window.wsMessages.length > 0
    );

    const messages = await page.evaluate(() => window.wsMessages);
    const isConnected = await page.evaluate(() => window.wsConnected);

    expect(isConnected).toBe(true);
    expect(messages![0]).toBe("Hello from ripress and wynd!");
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
                  resolve(undefined);
                };

                ws.onmessage = (event) => {
                  window.wsMessages?.push(event.data);
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
        expect(result.messages![0]).toBe("Hello from ripress and wynd!");
        expect(result.connectionTime).toBeGreaterThan(0);
      });
    } finally {
      // Clean up all contexts
      await Promise.all(contexts.map((ctx) => ctx.close()));
    }
  });

  // test("should handle rapid connection/disconnection cycles", async ({
  //   page,
  // }) => {
  //   const cycleCount = 10;
  //   const connectionTimes: number[] = [];
  //   const disconnectionTimes: number[] = [];

  //   // Configure timeouts (make them environment-aware)
  //   const CONNECTION_TIMEOUT = process.env.CI ? 5000 : 2000;
  //   const DISCONNECTION_TIMEOUT = process.env.CI ? 3000 : 1500;
  //   const MAX_CONNECTION_TIME = process.env.CI ? 2000 : 1000;
  //   const MAX_DISCONNECTION_TIME = process.env.CI ? 2000 : 1000;

  //   for (let i = 0; i < cycleCount; i++) {
  //     // Clean up state before each cycle
  //     await page.evaluate(() => {
  //       window.wsMessages = [];
  //       window.testWs = null;
  //       window.connectionMetrics = {};
  //     });

  //     // Establish connection with timeout and better error handling
  //     const connectionStart = Date.now();
  //     try {
  //       await page.evaluate(
  //         ({ wsUrl, timeout }) => {
  //           return Promise.race([
  //             new Promise((resolve, reject) => {
  //               const ws = new WebSocket(wsUrl);
  //               window.connectionMetrics.connectStart = Date.now();

  //               ws.onopen = () => {
  //                 window.connectionMetrics.connectEnd = Date.now();
  //                 window.testWs = ws;
  //                 window.wsMessages = [];
  //                 resolve(undefined);
  //               };

  //               ws.onmessage = (event) => {
  //                 window.wsMessages?.push(event.data);
  //               };

  //               ws.onerror = (error) => {
  //                 console.error("WebSocket error:", error);
  //                 reject(new Error("WebSocket connection failed"));
  //               };

  //               ws.onclose = (event) => {
  //                 if (event.code !== 1000) {
  //                   reject(
  //                     new Error(`WebSocket closed with code: ${event.code}`)
  //                   );
  //                 }
  //               };
  //             }),
  //             new Promise((_, reject) =>
  //               setTimeout(
  //                 () =>
  //                   reject(new Error(`Connection timeout after ${timeout}ms`)),
  //                 timeout
  //               )
  //             ),
  //           ]);
  //         },
  //         WS_URL,
  //         CONNECTION_TIMEOUT
  //       );

  //       // Wait for welcome message with timeout
  //       await page.waitForFunction(
  //         () => window.wsMessages && window.wsMessages.length > 0,
  //         { timeout: CONNECTION_TIMEOUT }
  //       );

  //       const connectionTime = Date.now() - connectionStart;
  //       connectionTimes.push(connectionTime);

  //       // Verify welcome message
  //       const messages = await page.evaluate(() => window.wsMessages);
  //       expect(messages![0]).toBe("Hello from ripress and wynd!");

  //       // Close connection with proper event handling and timing
  //       try {
  //         await page.evaluate(
  //           ({ timeout }) => {
  //             return Promise.race([
  //               new Promise((resolve, reject) => {
  //                 if (
  //                   !window.testWs ||
  //                   window.testWs.readyState !== WebSocket.OPEN
  //                 ) {
  //                   resolve(undefined);
  //                   return;
  //                 }

  //                 window.connectionMetrics.closeStart = Date.now();

  //                 const ws = window.testWs;
  //                 const closeTimeout = setTimeout(() => {
  //                   reject(
  //                     new Error(
  //                       "Close timeout - connection did not close gracefully"
  //                     )
  //                   );
  //                 }, timeout);

  //                 ws.onclose = (event) => {
  //                   window.connectionMetrics.closeEnd = Date.now();
  //                   clearTimeout(closeTimeout);
  //                   resolve(undefined);
  //                 };

  //                 ws.onerror = (error) => {
  //                   clearTimeout(closeTimeout);
  //                   reject(error);
  //                 };

  //                 // Initiate close
  //                 ws.close(1000, "Test completed");
  //               }),
  //               new Promise((_, reject) =>
  //                 setTimeout(
  //                   () =>
  //                     reject(
  //                       new Error(`Disconnection timeout after ${timeout}ms`)
  //                     ),
  //                   timeout
  //                 )
  //               ),
  //             ]);
  //           },
  //           { timeout: DISCONNECTION_TIMEOUT }
  //         );

  //         // Get actual disconnection time from the close event
  //         const metrics = await page.evaluate(() => window.connectionMetrics);
  //         const disconnectionTime = metrics.closeEnd - metrics.closeStart;
  //         disconnectionTimes.push(disconnectionTime);
  //       } catch (closeError) {
  //         console.warn(`Cycle ${i + 1}: Close error:`, closeError);
  //         // Force close if graceful close failed
  //         await page.evaluate(() => {
  //           if (window.testWs) {
  //             window.testWs.close();
  //             window.testWs = null;
  //           }
  //         });
  //         // Still record a time, but mark it as failed
  //         disconnectionTimes.push(DISCONNECTION_TIMEOUT);
  //       }
  //     } catch (connectionError) {
  //       console.error(`Cycle ${i + 1}: Connection error:`, connectionError);
  //       // Clean up any partial connection
  //       await page.evaluate(() => {
  //         if (window.testWs) {
  //           try {
  //             window.testWs.close();
  //           } catch (e) {
  //             // Ignore errors during cleanup
  //           }
  //           window.testWs = null;
  //         }
  //       });
  //       throw connectionError; // Re-throw to fail the test
  //     }

  //     // Wait between cycles to avoid overwhelming the server
  //     await new Promise((resolve) => setTimeout(resolve, 100));
  //   }

  //   // Analyze results
  //   console.log("Connection times (ms):", connectionTimes);
  //   console.log("Disconnection times (ms):", disconnectionTimes);
  //   console.log(
  //     "Average connection time:",
  //     connectionTimes.reduce((a, b) => a + b, 0) / connectionTimes.length
  //   );
  //   console.log(
  //     "Average disconnection time:",
  //     disconnectionTimes.reduce((a, b) => a + b, 0) / disconnectionTimes.length
  //   );

  //   // Verify performance expectations
  //   const slowConnections = connectionTimes.filter(
  //     (time) => time > MAX_CONNECTION_TIME
  //   );
  //   const slowDisconnections = disconnectionTimes.filter(
  //     (time) => time > MAX_DISCONNECTION_TIME
  //   );

  //   // Provide detailed failure information
  //   if (slowConnections.length > 0) {
  //     console.warn(
  //       `${slowConnections.length} connections took longer than ${MAX_CONNECTION_TIME}ms:`,
  //       slowConnections
  //     );
  //   }

  //   if (slowDisconnections.length > 0) {
  //     console.warn(
  //       `${slowDisconnections.length} disconnections took longer than ${MAX_DISCONNECTION_TIME}ms:`,
  //       slowDisconnections
  //     );
  //   }

  //   // Allow some tolerance for slow operations (e.g., max 20% can be slow)
  //   const maxSlowConnections = Math.ceil(cycleCount * 0.2);
  //   const maxSlowDisconnections = Math.ceil(cycleCount * 0.2);

  //   expect(slowConnections.length).toBeLessThanOrEqual(maxSlowConnections);
  //   expect(slowDisconnections.length).toBeLessThanOrEqual(
  //     maxSlowDisconnections
  //   );

  //   // Ensure all cycles completed
  //   expect(connectionTimes).toHaveLength(cycleCount);
  //   expect(disconnectionTimes).toHaveLength(cycleCount);
  // });

  test("should maintain connection stability over time", async ({ page }) => {
    const testDuration = 25000; // 30 seconds (should be senough for most connections)
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
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages?.push(event.data);
          window.messageCount!++;
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
    const isConnected = await page.evaluate(() => window.wsConnected !== false);
    const messageCount = await page.evaluate(() => window.messageCount);
    const lastMessageTime = await page.evaluate(() => window.lastMessageTime);

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
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages?.push(event.data);
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
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages?.push(event.data);
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
      () => window.serverClosed === true || window.wsMessages!.length > 1,
      { timeout: 5000 }
    );

    // Verify either server closed or message was processed
    const serverClosed = await page.evaluate(() => window.serverClosed);
    const messageCount = await page.evaluate(() => window.wsMessages!.length);

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
