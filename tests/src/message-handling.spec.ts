import { arraysEqual, WS_URL } from "./shared";
import { test, expect } from "@playwright/test";

test.describe("WebSocket Message Handling", () => {
  test.beforeAll(async () => {
    await new Promise((resolve) => setTimeout(resolve, 1000));
  });

  test("should echo text messages correctly", async ({ page }) => {
    const testMessages = [
      "Hello World!",
      "Test message with special chars: !@#$%^&*()",
      "Unicode: 🚀🌍🎉",
      "Multiline\nmessage\nwith\nbreaks",
      "Empty string test:",
      "",
      "Very long message: " + "A".repeat(1000),
    ];

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
      () => window.wsMessages && window.wsMessages.length > 0,
    );

    // Clear welcome message
    await page.evaluate(() => {
      window.wsMessages = [];
    });

    // Test each message
    for (const message of testMessages) {
      await page.evaluate((msg) => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.send(msg);
        }
      }, message);

      // Wait for echo
      await page.waitForFunction(
        () => window.wsMessages && window.wsMessages.length > 0,
      );

      const messages = await page.evaluate(() => window.wsMessages);
      expect(messages![messages!.length - 1]).toBe(message);

      // Clear for next message
      await page.evaluate(() => {
        window.wsMessages = [];
      });
    }
  });

  test("should handle rapid text message sequences", async ({ page }) => {
    const messageCount = 100;
    const messages = Array.from(
      { length: messageCount },
      (_, i) => `Message ${i}`,
    );

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
      () => window.wsMessages && window.wsMessages.length > 0,
    );

    // Clear welcome message
    await page.evaluate(() => {
      window.wsMessages = [];
    });

    // Send all messages rapidly
    await page.evaluate((msgs) => {
      msgs.forEach((msg) => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.send(msg);
        }
      });
    }, messages);

    // Wait for all echoes
    await page.waitForFunction(
      (expectedCount) =>
        window.wsMessages && window.wsMessages.length >= expectedCount,
      messageCount,
      { timeout: 10000 },
    );

    const receivedMessages = await page.evaluate(() => window.wsMessages);

    expect(receivedMessages?.length).toBe(messageCount);

    // Verify all messages were echoed correctly
    messages.forEach((message) => {
      expect(receivedMessages).toContain(message);
    });
  });

  // test("should handle binary messages correctly", async ({ page }) => {
  //   const binaryTestCases = [
  //     new Uint8Array([0, 1, 2, 3, 4, 5]),
  //     new Uint8Array([255, 254, 253, 252]),
  //     new Uint8Array(1000).fill(42), // Large binary data
  //     new Uint8Array([72, 101, 108, 108, 111]), // "Hello" in ASCII
  //     new Uint8Array([]), // Empty binary
  //   ];

  //   // Establish connection
  //   await page.evaluate(
  //     ({ wsUrl }) => {
  //       return new Promise((resolve, reject) => {
  //         const ws = new WebSocket(wsUrl);

  //         ws.onopen = () => {
  //           window.testWs = ws;
  //           window.wsMessages = [];
  //           window.binaryMessages = [];
  //           resolve(undefined);
  //         };

  //         ws.onmessage = (event) => {
  //           if (typeof event.data === "string") {
  //             window.wsMessages?.push(event.data);
  //           } else {
  //             // Store binary data as ArrayBuffer for better compatibility
  //             window.binaryMessages?.push(event.data);
  //           }
  //         };

  //         ws.onerror = (error) => {
  //           console.error("WebSocket error:", error);
  //           reject(new Error("WebSocket connection failed"));
  //         };

  //         ws.onclose = (event) => {
  //           if (event.code !== 1000) {
  //             reject(
  //               new Error(
  //                 `WebSocket closed unexpectedly with code: ${event.code}`
  //               )
  //             );
  //           }
  //         };
  //       });
  //     },
  //     { wsUrl: WS_URL }
  //   );

  //   // Wait for welcome message
  //   await page.waitForFunction(
  //     () => window.wsMessages && window.wsMessages.length > 0,
  //     { timeout: 5000 }
  //   );

  //   // Verify welcome message
  //   const welcomeMessages = await page.evaluate(() => window.wsMessages);
  //   expect(welcomeMessages![0]).toBe("Hello from ripress and wynd!");

  //   // Clear welcome message
  //   await page.evaluate(() => {
  //     window.wsMessages = [];
  //   });

  //   // Test each binary message
  //   for (let i = 0; i < binaryTestCases.length; i++) {
  //     const binaryData = binaryTestCases[i];
  //     console.log(`Testing binary case ${i + 1}: ${binaryData?.length} bytes`);

  //     // Clear previous binary messages
  //     await page.evaluate(() => {
  //       window.binaryMessages = [];
  //     });

  //     // Convert Uint8Array to regular array for page.evaluate compatibility
  //     const dataArray = Array.from(binaryData!);

  //     // Send binary data
  //     await page.evaluate(
  //       ({ data }) => {
  //         if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
  //           const uint8Array = new Uint8Array(data);
  //           window.testWs.send(uint8Array);
  //         } else {
  //           throw new Error("WebSocket is not open");
  //         }
  //       },
  //       { data: dataArray }
  //     );

  //     // Wait for binary echo with timeout
  //     try {
  //       await page.waitForFunction(
  //         () => window.binaryMessages && window.binaryMessages.length > 0,
  //         { timeout: 3000 }
  //       );
  //     } catch (timeoutError) {
  //       throw new Error(
  //         `Timeout waiting for binary message echo (case ${i + 1})`
  //       );
  //     }

  //     // Get and verify the received binary message
  //     const binaryMessages = await page.evaluate(() => {
  //       // Convert ArrayBuffer to Array for comparison
  //       return window.binaryMessages?.map((msg) =>
  //         Array.from(new Uint8Array(msg))
  //       );
  //     });

  //     expect(binaryMessages).toHaveLength(1);

  //     const receivedData = binaryMessages![0];
  //     const expectedData = Array.from(binaryData!);

  //     // Detailed comparison for better error messages
  //     console.log(receivedData, expectedData);
  //     expect(arraysEqual(receivedData!, expectedData)).toBe(true);

  //     // Additional verification for edge cases
  //     if (binaryData?.length === 0) {
  //       expect(receivedData).toHaveLength(0);
  //     } else {
  //       expect(receivedData).toHaveLength(binaryData!.length);
  //       // Spot check first and last bytes for non-empty arrays
  //       expect(receivedData![0]).toBe(binaryData![0]);
  //       if (binaryData!.length > 1) {
  //         expect(receivedData![receivedData!.length - 1]).toBe(
  //           binaryData![binaryData!.length - 1]
  //         );
  //       }
  //     }
  //   }

  //   // Clean up connection
  //   await page.evaluate(() => {
  //     if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
  //       window.testWs.close(1000, "Test completed");
  //     }
  //   });

  //   // Wait for connection to close gracefully
  //   await page.waitForFunction(
  //     () => !window.testWs || window.testWs.readyState === WebSocket.CLOSED,
  //     { timeout: 2000 }
  //   );
  // });

  // test("should handle mixed text and binary messages", async ({ page }) => {
  //   const mixedMessages = [
  //     { type: "text", data: "Hello World" },
  //     { type: "binary", data: new Uint8Array([1, 2, 3, 4, 5]) },
  //     { type: "text", data: "Binary data received" },
  //     { type: "binary", data: new Uint8Array([255, 254, 253]) },
  //     { type: "text", data: "Mixed message test complete" },
  //   ];

  //   await page.evaluate((wsUrl) => {
  //     return new Promise((resolve, reject) => {
  //       const ws = new WebSocket(wsUrl);

  //       ws.onopen = () => {
  //         window.testWs = ws;
  //         window.wsMessages = [];
  //         window.binaryMessages = [];
  //         resolve(undefined);
  //       };

  //       ws.onmessage = (event) => {
  //         if (typeof event.data === "string") {
  //           window.wsMessages?.push(event.data);
  //         } else {
  //           window.binaryMessages?.push(event.data);
  //         }
  //       };

  //       ws.onerror = reject;
  //     });
  //   }, WS_URL);

  //   // Wait for welcome message
  //   await page.waitForFunction(
  //     () => window.wsMessages && window.wsMessages.length > 0
  //   );

  //   // Clear welcome message
  //   await page.evaluate(() => {
  //     window.wsMessages = [];
  //   });

  //   // Send mixed messages
  //   for (const message of mixedMessages) {
  //     await page.evaluate((msg) => {
  //       if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
  //         window.testWs.send(msg.data);
  //       }
  //     }, message);

  //     // Wait for response
  //     // await page.waitForFunction(
  //     //   () =>
  //     //     (msg.type === "text" && window.wsMessages.length > 0) ||
  //     //     (msg.type === "binary" && window.binaryMessages.length > 0)
  //     // );

  //     if (message.type === "text") {
  //       const textMessages = await page.evaluate(() => window.wsMessages);
  //       expect(textMessages![textMessages!.length - 1]).toBe(message.data);
  //     } else {
  //       const binaryMessages = await page.evaluate(() => window.binaryMessages);
  //       const lastBinaryMessage = binaryMessages![binaryMessages!.length - 1];
  //       const receivedData = new Uint8Array(lastBinaryMessage!);
  //       expect(receivedData).toEqual(message.data);
  //     }
  //   }
  // });

  test("should handle very large messages", async ({ page }) => {
    const largeMessageSizes = [1024, 10240, 102400]; // 1KB, 10KB, 100KB

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
      () => window.wsMessages && window.wsMessages.length > 0,
    );

    // Clear welcome message
    await page.evaluate(() => {
      window.wsMessages = [];
    });

    for (const size of largeMessageSizes) {
      const largeMessage = "A".repeat(size);

      await page.evaluate((msg) => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.send(msg);
        }
      }, largeMessage);

      // Wait for echo with longer timeout for large messages
      await page.waitForFunction(
        () => window.wsMessages && window.wsMessages.length > 0,
        { timeout: 15000 },
      );

      const messages = await page.evaluate(() => window.wsMessages);
      const receivedMessage = messages![messages!.length - 1];

      expect(receivedMessage).toBe(largeMessage);
      expect(receivedMessage!.length).toBe(size);

      // Clear for next test
      await page.evaluate(() => {
        window.wsMessages = [];
      });
    }
  });

  test("should handle message ordering correctly", async ({ page }) => {
    const orderedMessages = [
      "First message",
      "Second message",
      "Third message",
      "Fourth message",
      "Fifth message",
    ];

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
      () => window.wsMessages && window.wsMessages.length > 0,
    );

    // Clear welcome message
    await page.evaluate(() => {
      window.wsMessages = [];
    });

    // Send messages in order
    for (const message of orderedMessages) {
      await page.evaluate((msg) => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.send(msg);
        }
      }, message);

      // Small delay to ensure ordering
      await new Promise((resolve) => setTimeout(resolve, 10));
    }

    // Wait for all echoes
    await page.waitForFunction(
      (expectedCount) =>
        window.wsMessages && window.wsMessages.length >= expectedCount,
      orderedMessages.length,
      { timeout: 5000 },
    );

    const receivedMessages = await page.evaluate(() => window.wsMessages);

    // Verify messages are in correct order
    expect(receivedMessages!.length).toBe(orderedMessages.length);
    orderedMessages.forEach((message, index) => {
      expect(receivedMessages![index]).toBe(message);
    });
  });

  test("should handle concurrent message sending from multiple clients", async ({
    browser,
  }) => {
    const clientCount = 3;
    const messagesPerClient = 10;

    const contexts = await Promise.all(
      Array.from({ length: clientCount }, () => browser.newContext()),
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
                  resolve(undefined);
                };

                ws.onmessage = (event) => {
                  window.wsMessages?.push(event.data);
                };

                ws.onerror = reject;
              });
            },
            { wsUrl: WS_URL, clientId: `client-${index}` },
          ),
        ),
      );

      // Wait for all welcome messages
      await Promise.all(
        pages.map((page) =>
          page.waitForFunction(
            () => window.wsMessages && window.wsMessages.length > 0,
          ),
        ),
      );

      // Clear welcome messages
      await Promise.all(
        pages.map((page) =>
          page.evaluate(() => {
            window.wsMessages = [];
          }),
        ),
      );

      // Send messages from all clients concurrently
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
            { clientId: `client-${index}`, messageCount: messagesPerClient },
          ),
        ),
      );

      // Wait for all messages to be received
      await Promise.all(
        pages.map((page) =>
          page.waitForFunction(
            (expectedCount) =>
              window.wsMessages && window.wsMessages.length >= expectedCount,
            messagesPerClient,
            { timeout: 10000 },
          ),
        ),
      );

      // Verify each client received its own messages
      const results = await Promise.all(
        pages.map((page) =>
          page.evaluate(() => ({
            clientId: window.clientId,
            messages: window.wsMessages,
          })),
        ),
      );

      results.forEach((result, index) => {
        expect(result.clientId).toBe(`client-${index}`);
        expect(result.messages!.length).toBe(messagesPerClient);

        // Verify all messages are from this client
        result.messages!.forEach((message) => {
          expect(message).toContain(`client-${index}`);
        });
      });
    } finally {
      await Promise.all(contexts.map((ctx) => ctx.close()));
    }
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
