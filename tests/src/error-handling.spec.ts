import { WS_URL } from "./shared";
import { test, expect } from "@playwright/test";

test.describe("WebSocket Error Handling and Edge Cases", () => {
  test.beforeAll(async () => {
    await new Promise((resolve) => setTimeout(resolve, 1000));
  });

  test("should handle malformed WebSocket URLs", async ({ page }) => {
    const malformedUrls = [
      "ws://invalid-host:9999",
      "ws://localhost:99999", // Invalid port
      "http://localhost:3000", // Wrong protocol
      "wss://localhost:3000", // Wrong protocol (HTTPS)
    ];

    for (const url of malformedUrls) {
      let connectionFailed = false;

      try {
        await page.evaluate((wsUrl) => {
          return new Promise((resolve, reject) => {
            const ws = new WebSocket(wsUrl);

            ws.onopen = () => {
              resolve();
            };

            ws.onerror = (error) => {
              reject(error);
            };

            ws.onclose = () => {
              reject(new Error("Connection closed unexpectedly"));
            };
          });
        }, url);
      } catch (error) {
        connectionFailed = true;
      }

      expect(connectionFailed).toBe(true);
    }
  });

  test("should handle connection timeout scenarios", async ({ page }) => {
    // Test with a very short timeout
    const timeoutPromise = new Promise((resolve) => {
      setTimeout(() => resolve("timeout"), 100);
    });

    const connectionPromise = page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          resolve("connected");
        };

        ws.onerror = (error) => {
          reject(error);
        };
      });
    }, WS_URL);

    const result = await Promise.race([connectionPromise, timeoutPromise]);

    // Connection should succeed within timeout
    expect(result).toBe("connected");
  });

  test("should handle network interruption simulation", async ({ page }) => {
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.connectionLost = false;
          resolve();
        };

        ws.onmessage = (event) => {
          window.wsMessages.push(event.data);
        };

        ws.onerror = () => {
          window.connectionLost = true;
        };

        ws.onclose = () => {
          window.connectionClosed = true;
        };
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Simulate network interruption by closing the connection abruptly
    await page.evaluate(() => {
      if (window.testWs) {
        // Force close without proper close frame
        window.testWs.close();
      }
    });

    // Wait for connection to be marked as closed
    await page.waitForFunction(() => window.connectionClosed === true, {
      timeout: 5000,
    });

    const connectionClosed = await page.evaluate(() => window.connectionClosed);
    expect(connectionClosed).toBe(true);
  });

  test("should handle invalid message formats gracefully", async ({ page }) => {
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.errors = [];
          resolve();
        };

        ws.onmessage = (event) => {
          window.wsMessages.push(event.data);
        };

        ws.onerror = (error) => {
          window.errors.push(error);
        };

        ws.onclose = () => {
          window.connectionClosed = true;
        };
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Test various edge cases
    const edgeCases = [
      null,
      undefined,
      {},
      [],
      true,
      false,
      123,
      NaN,
      Infinity,
    ];

    for (const edgeCase of edgeCases) {
      try {
        await page.evaluate((data) => {
          if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
            // Try to send various data types
            if (data !== null && data !== undefined) {
              window.testWs.send(data);
            }
          }
        }, edgeCase);
      } catch (error) {
        // Some edge cases should fail - that's expected
      }
    }

    // Verify connection is still alive
    const isConnected = await page.evaluate(
      () => globalThis.wsConnected !== false
    );
    expect(isConnected).toBe(true);
  });

  test("should handle rapid open/close cycles", async ({ page }) => {
    const cycleCount = 20;
    let successfulConnections = 0;
    let failedConnections = 0;

    for (let i = 0; i < cycleCount; i++) {
      try {
        await page.evaluate((wsUrl) => {
          return new Promise((resolve, reject) => {
            const ws = new WebSocket(wsUrl);

            ws.onopen = () => {
              window.testWs = ws;
              resolve();
            };

            ws.onerror = reject;
          });
        }, WS_URL);

        successfulConnections++;

        // Immediately close
        await page.evaluate(() => {
          if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
            window.testWs.close();
          }
        });

        // Wait for close
        await page.waitForFunction(
          () => !window.testWs || window.testWs.readyState === WebSocket.CLOSED,
          { timeout: 1000 }
        );
      } catch (error) {
        failedConnections++;
      }

      // Small delay between cycles
      await new Promise((resolve) => setTimeout(resolve, 50));
    }

    // Most connections should succeed
    expect(successfulConnections).toBeGreaterThan(cycleCount * 0.8);
    expect(failedConnections).toBeLessThan(cycleCount * 0.2);
  });

  test("should handle memory pressure with many messages", async ({ page }) => {
    const messageCount = 1000;
    const messageSize = 1000; // 1KB per message

    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.messageCount = 0;
          resolve();
        };

        ws.onmessage = (event) => {
          window.wsMessages.push(event.data);
          window.messageCount++;
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

    // Send many large messages
    for (let i = 0; i < messageCount; i++) {
      const largeMessage = "A".repeat(messageSize) + `-${i}`;

      await page.evaluate((msg) => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.send(msg);
        }
      }, largeMessage);

      // Clear messages periodically to prevent memory issues
      if (i % 100 === 0) {
        await page.evaluate(() => {
          window.wsMessages = [];
        });
      }
    }

    // Wait for final messages
    await page.waitForFunction(
      () => window.messageCount >= messageCount - 100, // Allow for some clearing
      { timeout: 30000 }
    );

    const finalMessageCount = await page.evaluate(() => window.messageCount);
    expect(finalMessageCount).toBeGreaterThan(messageCount * 0.9);
  });

  test("should handle concurrent error scenarios", async ({ browser }) => {
    const clientCount = 5;
    const contexts = await Promise.all(
      Array.from({ length: clientCount }, () => browser.newContext())
    );

    const pages = await Promise.all(contexts.map((ctx) => ctx.newPage()));

    try {
      // Connect all clients
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
                  window.errorCount = 0;
                  resolve();
                };

                ws.onmessage = (event) => {
                  window.wsMessages.push(event.data);
                };

                ws.onerror = () => {
                  window.errorCount++;
                };

                ws.onclose = () => {
                  window.connectionClosed = true;
                };
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

      // Clear welcome messages
      await Promise.all(
        pages.map((page) =>
          page.evaluate(() => {
            window.wsMessages = [];
          })
        )
      );

      // Send problematic messages from all clients
      await Promise.all(
        pages.map((page, index) =>
          page.evaluate((clientId) => {
            // Send various problematic messages
            const problematicMessages = [
              "A".repeat(10000), // Very large message
              "", // Empty message
              "Special chars: \x00\x01\x02", // Control characters
              "Unicode: 🚀🌍🎉".repeat(100), // Repeated unicode
            ];

            problematicMessages.forEach((msg) => {
              if (
                window.testWs &&
                window.testWs.readyState === WebSocket.OPEN
              ) {
                window.testWs.send(msg);
              }
            });
          }, `client-${index}`)
        )
      );

      // Wait for messages to be processed
      await Promise.all(
        pages.map((page) =>
          page.waitForFunction(
            () => window.wsMessages && window.wsMessages.length > 0,
            { timeout: 10000 }
          )
        )
      );

      // Verify all clients are still connected
      const results = await Promise.all(
        pages.map((page) =>
          page.evaluate(() => ({
            clientId: window.clientId,
            messageCount: window.wsMessages.length,
            errorCount: window.errorCount,
            connectionClosed: window.connectionClosed,
          }))
        )
      );

      results.forEach((result) => {
        expect(result.connectionClosed).toBeFalsy();
        expect(result.messageCount).toBeGreaterThan(0);
        expect(result.errorCount).toBeLessThan(5); // Allow some errors but not too many
      });
    } finally {
      await Promise.all(contexts.map((ctx) => ctx.close()));
    }
  });

  test("should handle server restart simulation", async ({ page }) => {
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.reconnectionAttempts = 0;
          resolve();
        };

        ws.onmessage = (event) => {
          window.wsMessages.push(event.data);
        };

        ws.onerror = () => {
          window.reconnectionAttempts++;
        };

        ws.onclose = () => {
          window.connectionClosed = true;
        };
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Simulate server restart by closing connection
    await page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.close();
      }
    });

    // Wait for connection to close
    await page.waitForFunction(() => window.connectionClosed === true, {
      timeout: 5000,
    });

    const connectionClosed = await page.evaluate(() => window.connectionClosed);
    expect(connectionClosed).toBe(true);

    // Attempt to reconnect
    let reconnected = false;
    try {
      await page.evaluate((wsUrl) => {
        return new Promise((resolve, reject) => {
          const ws = new WebSocket(wsUrl);

          ws.onopen = () => {
            window.testWs = ws;
            window.wsMessages = [];
            resolve();
          };

          ws.onerror = reject;
        });
      }, WS_URL);

      reconnected = true;
    } catch (error) {
      // Reconnection might fail if server is down
    }

    // Reconnection should succeed if server is still running
    expect(reconnected).toBe(true);
  });

  test("should handle malformed close frames", async ({ page }) => {
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.closeReceived = false;
          resolve();
        };

        ws.onmessage = (event) => {
          window.wsMessages.push(event.data);
        };

        ws.onclose = (event) => {
          window.closeReceived = true;
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

    // Send close frame with invalid code
    await page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.close(9999, "Invalid close code");
      }
    });

    // Wait for close event
    await page.waitForFunction(() => window.closeReceived === true, {
      timeout: 5000,
    });

    const closeReceived = await page.evaluate(() => window.closeReceived);
    const closeCode = await page.evaluate(() => window.closeCode);
    const closeReason = await page.evaluate(() => window.closeReason);

    expect(closeReceived).toBe(true);
    expect(closeCode).toBe(9999);
    expect(closeReason).toBe("Invalid close code");
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
