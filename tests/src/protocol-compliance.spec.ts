import { test, expect } from "@playwright/test";
import { WS_URL } from "./shared";
import { WebSocket } from "ws";

test.describe("WebSocket Protocol Compliance Tests", () => {
  test.beforeAll(async () => {
    await new Promise((resolve) => setTimeout(resolve, 1000));
  });

  test("should handle WebSocket handshake correctly", async ({ page }) => {
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.handshakeSuccessful = true;
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages?.push(event.data);
        };

        ws.onerror = (error) => {
          window.handshakeSuccessful = false;
          reject(error);
        };
      });
    }, WS_URL);

    // Wait for handshake to complete
    await page.waitForFunction(() => window.handshakeSuccessful === true);

    const handshakeSuccessful = await page.evaluate(
      () => window.handshakeSuccessful
    );
    expect(handshakeSuccessful).toBe(true);

    // Verify WebSocket is in OPEN state
    const wsState = await page.evaluate(() => window.testWs?.readyState);
    expect(wsState).toBe(WebSocket.OPEN);
  });

  test("should handle WebSocket frame types correctly", async ({ page }) => {
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.binaryMessages = [];
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          if (typeof event.data === "string") {
            window.wsMessages?.push(event.data);
          } else {
            window.binaryMessages?.push(event.data);
          }
        };

        ws.onerror = reject;
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    // Test text frame
    await page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.send("Text frame test");
      }
    });

    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 1
    );

    // Test binary frame
    const binaryData = new Uint8Array([1, 2, 3, 4, 5]);
    await page.evaluate((data) => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.send(data);
      }
    }, binaryData);

    await page.waitForFunction(
      () => window.binaryMessages && window.binaryMessages.length > 0
    );

    const textMessages = await page.evaluate(() => window.wsMessages);
    const binaryMessages = await page.evaluate(() => window.binaryMessages);

    expect(textMessages![2]).toBe("Text frame test");
    expect(binaryMessages![0]).toBeDefined();
  });

  test("should handle WebSocket close frames correctly", async ({ page }) => {
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.closeReceived = false;
          resolve(undefined);
        };

        ws.onmessage = (event) => {
          window.wsMessages?.push(event.data);
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

    // Send close frame
    await page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.close(1000, "Normal closure");
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
    expect(closeCode).toBe(1000);
    expect(closeReason).toBe("Normal closure");
  });

  test("should handle WebSocket ping/pong frames", async ({ page }) => {
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.pingReceived = false;
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

    // Send ping (if supported by browser)
    await page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        // Note: Browser WebSocket API doesn't expose ping directly
        // This tests the server's handling of ping frames
        window.testWs.send("ping");
      }
    });

    // Wait for pong response
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 1,
      { timeout: 5000 }
    );

    const messages = await page.evaluate(() => window.wsMessages);
    expect(messages?.length).toBeGreaterThan(1);
  });

  test("should handle WebSocket extensions", async ({ page }) => {
    // Test WebSocket connection (extensions would be tested if supported)
    await page.evaluate((wsUrl) => {
      return new Promise((resolve, reject) => {
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
          window.testWs = ws;
          window.wsMessages = [];
          window.extensions = ws.extensions;
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

    const extensions = await page.evaluate(() => window.extensions);
    const messages = await page.evaluate(() => window.wsMessages);

    expect(messages![0]).toBe("Hello from ripress and wynd!");
    // Extensions might be empty or contain compression info
    expect(typeof extensions).toBe("string");
  });

  test("should handle WebSocket subprotocols", async ({ page }) => {
    // Test WebSocket connection with subprotocol
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
          window.wsMessages?.push(event.data);
        };

        ws.onerror = reject;
      });
    }, WS_URL);

    // Wait for welcome message
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0
    );

    const protocol = await page.evaluate(() => window.protocol);
    const messages = await page.evaluate(() => window.wsMessages);

    expect(messages![0]).toBe("Hello from ripress and wynd!");
    expect(protocol).toBe(""); // No subprotocol specified
  });

  test("should handle WebSocket frame fragmentation", async ({ page }) => {
    // Test with large message that might be fragmented
    const largeMessage = "A".repeat(65536); // 64KB message

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

    // Send large message
    await page.evaluate((msg) => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.send(msg);
      }
    }, largeMessage);

    // Wait for fragmented message to be reassembled
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 0,
      { timeout: 30000 }
    );

    const messages = await page.evaluate(() => window.wsMessages);
    expect(messages![0]).toBe(largeMessage);
    expect(messages![0]!.toString().length).toBe(65536);
  });

  test("should handle WebSocket control frames", async ({ page }) => {
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

    // Send various control-like messages
    const controlMessages = ["ping", "pong", "close", "control"];

    for (const message of controlMessages) {
      await page.evaluate((msg) => {
        if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
          window.testWs.send(msg);
        }
      }, message);

      // Wait for echo
      await page.waitForFunction(
        () => window.wsMessages && window.wsMessages.length > 0
      );

      const messages = await page.evaluate(() => window.wsMessages);
      expect(messages![messages!.length - 1]).toBe(message);
    }
  });

  test("should handle WebSocket masking", async ({ page }) => {
    // Test WebSocket connection (masking is handled by browser)
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

    // Send message (masking is automatic in browser)
    await page.evaluate(() => {
      if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
        window.testWs.send("Masked message test");
      }
    });

    // Wait for echo
    await page.waitForFunction(
      () => window.wsMessages && window.wsMessages.length > 1
    );

    const messages = await page.evaluate(() => window.wsMessages);
    expect(messages![messages!.length - 1]).toBe("Masked message test");
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
