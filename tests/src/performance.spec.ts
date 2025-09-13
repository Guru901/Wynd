import { expect, test } from "@playwright/test";
import { WS_URL } from "./shared";

test.describe("WebSocket Performance and Load Tests", () => {
  test.beforeAll(async () => {
    await new Promise((resolve) => setTimeout(resolve, 1000));
  });

  test("should handle high message throughput", async ({ page }) => {
    const messageCount = 1000;
    const messageSize = 100; // 100 bytes per message

    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.startTime = Date.now();
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

    // Clear welcome message and start timing
    await page.evaluate(() => {
      window.wsMessages = [];
      window.startTime = Date.now();
    });

    // Send messages rapidly
    const messages = Array.from(
      { length: messageCount },
      (_, i) => "A".repeat(messageSize) + `-${i}`
    );

    await page.evaluate((msgs) => {
      msgs.forEach((msg) => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.send(msg);
        }
      });
    }, messages);

    // Wait for all messages to be received
    await page.waitForFunction(
      (expectedCount) =>
        window.wsMessages && window.wsMessages.length >= expectedCount,
      messageCount,
      { timeout: 30000 }
    );

    const endTime = await page.evaluate(() => Date.now());
    const startTime = await page.evaluate(() => window.startTime);
    const receivedCount = await page.evaluate(() => window.wsMessages!.length);

    const duration = endTime - Number(startTime);
    const messagesPerSecond = (receivedCount / duration) * 1000;

    expect(receivedCount).toBe(messageCount);
    expect(messagesPerSecond).toBeGreaterThan(100); // At least 100 messages/second
    expect(duration).toBeLessThan(10000); // Should complete within 10 seconds
  });

  test("should handle many concurrent connections", async ({ browser }) => {
    const connectionCount = 50;
    const messagesPerConnection = 10;

    const contexts = await Promise.all(
      Array.from({ length: connectionCount }, () => browser.newContext())
    );

    const pages = await Promise.all(contexts.map((ctx) => ctx.newPage()));

    try {
      const startTime = Date.now();

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
                  window.messageCount = 0;
                  resolve(undefined);
                };

                ws.onmessage = (event) => {
                  window.wsMessages?.push(event.data);
                  window.messageCount!++;
                };

                ws.onerror = reject;
              });
            },
            { wsUrl: WS_URL, clientId: `client-${index}` }
          )
        )
      );

      const connectionTime = Date.now() - startTime;

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
            window.messageCount = 0;
          })
        )
      );

      // Send messages from all clients
      await Promise.all(
        pages.map((page, index) =>
          page.evaluate(
            ({ clientId, messageCount }) => {
              for (let i = 0; i < messageCount; i++) {
                if (
                  window.testWs &&
                  window.testWs.readyState === WebSocket.OPEN
                ) {
                  window.testWs.send(`Message ${i} from ${clientId}`);
                }
              }
            },
            {
              clientId: `client-${index}`,
              messageCount: messagesPerConnection,
            }
          )
        )
      );

      // Wait for all messages to be received
      await Promise.all(
        pages.map((page) =>
          page.waitForFunction(
            (expectedCount) => Number(window.messageCount) >= expectedCount,
            messagesPerConnection,
            { timeout: 30000 }
          )
        )
      );

      const endTime = Date.now();
      const totalDuration = endTime - startTime;

      // Verify all connections are still alive
      const results = await Promise.all(
        pages.map((page) =>
          page.evaluate(() => ({
            clientId: window.clientId,
            messageCount: window.messageCount,
            connectionAlive:
              window.testWs && window.testWs.readyState === WebSocket.OPEN,
          }))
        )
      );

      const aliveConnections = results.filter((r) => r.connectionAlive).length;

      expect(aliveConnections).toBeGreaterThan(connectionCount * 0.95); // 95% success rate
      expect(results[0]?.messageCount).toBe(
        connectionCount * messagesPerConnection
      );
      expect(connectionTime).toBeLessThan(5000); // Connections should establish within 5 seconds
      expect(totalDuration).toBeLessThan(30000); // Total test should complete within 30 seconds
    } finally {
      await Promise.all(contexts.map((ctx) => ctx.close()));
    }
  });

  test("should handle large message payloads efficiently", async ({ page }) => {
    const largeMessageSizes = [1024, 10240, 102400, 1048576]; // 1KB, 10KB, 100KB, 1MB
    const results: { size: number; duration: number; success: boolean }[] = [];

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

    for (const size of largeMessageSizes) {
      // Clear messages
      await page.evaluate(() => {
        window.wsMessages = [];
      });

      const largeMessage = "A".repeat(size);
      const startTime = Date.now();

      await page.evaluate((msg) => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.send(msg);
        }
      }, largeMessage);

      // Wait for echo with timeout based on message size
      const timeout = Math.max(5000, size / 100); // 1ms per 100 bytes, minimum 5s

      try {
        await page.waitForFunction(
          () => window.wsMessages && window.wsMessages.length > 0,
          { timeout }
        );

        const endTime = Date.now();
        const duration = endTime - startTime;

        const messages = await page.evaluate(() => window.wsMessages);
        const success = messages![0] === largeMessage;

        results.push({ size, duration, success });

        expect(success).toBe(true);
        expect(duration).toBeLessThan(timeout);
      } catch (error) {
        results.push({ size, duration: 0, success: false });
      }
    }

    // Verify all large messages were handled successfully
    const successfulResults = results.filter((r) => r.success);
    expect(successfulResults.length).toBeGreaterThan(
      largeMessageSizes.length * 0.8
    );
  });

  test("should maintain low latency under load", async ({ browser }) => {
    const connectionCount = 10;
    const messagesPerConnection = 100;

    const contexts = await Promise.all(
      Array.from({ length: connectionCount }, () => browser.newContext())
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
                  window.latencies = [];
                  resolve(undefined);
                };

                ws.onmessage = (event) => {
                  const receiveTime = Date.now();
                  const messageData = event.data;

                  if (messageData.startsWith("ping-")) {
                    const sendTime = parseInt(messageData.split("-")[1]);
                    const latency = receiveTime - sendTime;
                    window.latencies?.push(latency);
                  }

                  window.wsMessages?.push(messageData);
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

      // Send ping messages to measure latency
      for (let i = 0; i < messagesPerConnection; i++) {
        await Promise.all(
          pages.map((page) =>
            page.evaluate(() => {
              const sendTime = Date.now();
              if (
                window.testWs &&
                window.testWs.readyState === WebSocket.OPEN
              ) {
                window.testWs.send(`ping-${sendTime}`);
              }
            })
          )
        );

        // Small delay between pings
        await new Promise((resolve) => setTimeout(resolve, 10));
      }

      // Wait for all ping responses
      await Promise.all(
        pages.map((page) =>
          page.waitForFunction(
            (expectedCount) =>
              window.latencies && window.latencies.length >= expectedCount,
            messagesPerConnection,
            { timeout: 30000 }
          )
        )
      );

      // Collect latency data
      const latencyResults = await Promise.all(
        pages.map((page) =>
          page.evaluate(() => ({
            clientId: window.clientId,
            latencies: window.latencies,
            avgLatency:
              window.latencies!.reduce((a, b) => a + b, 0) /
              window.latencies!.length,
            maxLatency: Math.max(...window.latencies!),
            minLatency: Math.min(...window.latencies!),
          }))
        )
      );

      // Calculate overall statistics
      const allLatencies = latencyResults.flatMap(
        (r) => r.latencies
      ) as number[];
      const avgLatency =
        allLatencies.reduce((a, b) => a + b, 0) / allLatencies.length;
      const maxLatency = Math.max(...allLatencies);
      const minLatency = Math.min(...allLatencies);

      // Verify latency is reasonable
      expect(avgLatency).toBeLessThan(100); // Average latency under 100ms
      expect(maxLatency).toBeLessThan(500); // Max latency under 500ms
      expect(minLatency).toBeGreaterThanOrEqual(0); // Min latency should be positive

      // Verify all clients have reasonable latency
      latencyResults.forEach((result) => {
        expect(result.avgLatency).toBeLessThan(200);
        expect(result.maxLatency).toBeLessThan(1000);
      });
    } finally {
      await Promise.all(contexts.map((ctx) => ctx.close()));
    }
  });

  test("should handle memory usage efficiently", async ({ page }) => {
    const messageCount = 5000;
    const messageSize = 200;

    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.messageCount = 0;
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages?.push(event.data);
          window.messageCount!++;

          // Keep only last 100 messages to prevent memory issues
          if (window.wsMessages!.length > 100) {
            window.wsMessages = window.wsMessages?.slice(-100);
          }
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
      window.messageCount = 0;
    });

    // Send many messages
    for (let i = 0; i < messageCount; i++) {
      const message = "A".repeat(messageSize) + `-${i}`;

      await page.evaluate((msg) => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.send(msg);
        }
      }, message);

      // Clear messages every 100 to prevent memory buildup
      if (i % 100 === 0) {
        await page.evaluate(() => {
          window.wsMessages = [];
        });
      }
    }

    // Wait for final messages
    await page.waitForFunction(
      () => window.messageCount! >= messageCount - 100,
      { timeout: 60000 }
    );

    const finalMessageCount = await page.evaluate(() => window.messageCount);
    const finalMessageArrayLength = await page.evaluate(
      () => window.wsMessages!.length
    );

    expect(finalMessageCount).toBeGreaterThan(messageCount * 0.95);
    expect(finalMessageArrayLength).toBeLessThanOrEqual(100); // Memory management working
  });

  // test("should handle connection burst scenarios", async ({ browser }) => {
  //   const burstSize = 10;
  //   const burstCount = 1;
  //   const delayBetweenBursts = 2000; // 2 seconds

  //   const contexts = await Promise.all(
  //     Array.from({ length: burstSize }, () => browser.newContext())
  //   );

  //   const pages = await Promise.all(contexts.map((ctx) => ctx.newPage()));

  //   try {
  //     for (let burst = 0; burst < burstCount; burst++) {
  //       console.log(`Starting burst ${burst + 1}/${burstCount}`);

  //       // Connect all clients in this burst
  //       await Promise.all(
  //         pages.map((page, index) =>
  //           page.evaluate(
  //             ({ wsUrl, clientId }) => {
  //               return new Promise((resolve, reject) => {
  //                 const ws = new WebSocket(wsUrl);

  //                 ws.onopen = () => {
  //                   window.testWs = ws;
  //                   window.wsMessages = [];
  //                   window.clientId = clientId;
  //                   window.burstNumber = burst;
  //                   resolve(undefined);
  //                 };

  //                 ws.onmessage = (event) => {
  //                   window.wsMessages?.push(event.data);
  //                 };

  //                 ws.onerror = reject;
  //               });
  //             },
  //             { wsUrl: WS_URL, clientId: `burst-${burst}-client-${index}` }
  //           )
  //         )
  //       );

  //       // Wait for all welcome messages
  //       await Promise.all(
  //         pages.map((page) =>
  //           page.waitForFunction(
  //             () => window.wsMessages && window.wsMessages.length > 0
  //           )
  //         )
  //       );

  //       // Send messages from all clients
  //       await Promise.all(
  //         pages.map((page, index) =>
  //           page.evaluate((clientId) => {
  //             for (let i = 0; i < 10; i++) {
  //               if (
  //                 window.testWs &&
  //                 window.testWs.readyState === WebSocket.OPEN
  //               ) {
  //                 window.testWs.send(`Burst message ${i} from ${clientId}`);
  //               }
  //             }
  //           }, `burst-${burst}-client-${index}`)
  //         )
  //       );

  //       // Wait for messages to be processed
  //       await Promise.all(
  //         pages.map((page) =>
  //           page.waitForFunction(
  //             () => window.wsMessages && window.wsMessages.length > 10
  //           )
  //         )
  //       );

  //       // Close all connections
  //       await Promise.all(
  //         pages.map((page) =>
  //           page.evaluate(() => {
  //             if (
  //               window.testWs &&
  //               window.testWs.readyState === WebSocket.OPEN
  //             ) {
  //               window.testWs.close();
  //             }
  //           })
  //         )
  //       );

  //       // Wait for connections to close
  //       await Promise.all(
  //         pages.map((page) =>
  //           page.waitForFunction(
  //             () =>
  //               !window.testWs || window.testWs.readyState === WebSocket.CLOSED
  //           )
  //         )
  //       );

  //       // Wait before next burst
  //       if (burst < burstCount - 1) {
  //         await new Promise((resolve) =>
  //           setTimeout(resolve, delayBetweenBursts)
  //         );
  //       }
  //     }

  //     // Verify all bursts completed successfully
  //     const results = await Promise.all(
  //       pages.map((page) =>
  //         page.evaluate(() => ({
  //           clientId: window.clientId,
  //           burstNumber: window.burstNumber,
  //           messageCount: window.wsMessages ? window.wsMessages.length : 0,
  //         }))
  //       )
  //     );

  //     results.forEach((result) => {
  //       expect(result.messageCount).toBeGreaterThan(10);
  //       expect(result.burstNumber).toBe(burstCount - 1); // Last burst
  //     });
  //   } finally {
  //     await Promise.all(contexts.map((ctx) => ctx.close()));
  //   }
  // });

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
