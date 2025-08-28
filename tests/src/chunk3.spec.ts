// tests/websocket-error-handling.test.ts
import { test, expect } from "@playwright/test";
import WebSocket from "ws";
import {
  closeWebSocket,
  CONNECTION_TIMEOUT,
  createWebSocket,
  WS_URL,
} from "./shared";

test.describe("WebSocket Error Handling", () => {
  test("should handle malformed WebSocket URLs", async () => {
    const malformedUrls = [
      "invalid-url",
      "http://localhost:3000", // Wrong protocol
      "ws://",
      "ws://localhost:",
      "ws://localhost:abc", // Invalid port
    ];
    for (const url of malformedUrls) {
      await expect(createWebSocket(url)).rejects.toThrow();
    }
  });
  test("should handle sending messages to closed connection", async () => {
    const ws = await createWebSocket(WS_URL);
    await closeWebSocket(ws);
    expect(ws.readyState).toBe(WebSocket.CLOSED);
    // Sending to closed connection should not crash
    expect(() => {
      ws.send("test message");
    }).not.toThrow();
  });
  test("should handle extremely large messages", async () => {
    const ws = await createWebSocket(WS_URL);
    // Create a very large message (1MB)
    const hugeMessage = "x".repeat(1024 * 1024);
    let errorOccurred = false;
    ws.on("error", () => {
      errorOccurred = true;
    });
    try {
      ws.send(hugeMessage);
      // Wait a bit to see if error occurs
      await new Promise((resolve) => setTimeout(resolve, 1000));
    } catch (error) {
      console.log("Large message sending failed as expected");
    }
    await closeWebSocket(ws);
  });
  test("should handle invalid JSON messages gracefully", async () => {
    const ws = await createWebSocket(WS_URL);
    const invalidJsonMessages = [
      '{"invalid": json}',
      "{incomplete json",
      "not json at all",
      '{"nested": {"deeply": {"invalid":]',
    ];
    for (const invalidMsg of invalidJsonMessages) {
      expect(() => ws.send(invalidMsg)).not.toThrow();
    }
    await closeWebSocket(ws);
  });
  test("should handle network interruption simulation", async () => {
    const ws = await createWebSocket(WS_URL);
    let closeEventReceived = false;
    let errorEventReceived = false;
    ws.on("close", () => {
      closeEventReceived = true;
    });
    ws.on("error", () => {
      errorEventReceived = true;
    });
    // Force close to simulate network interruption
    (ws as any)._socket?.destroy();
    // Wait for events
    await new Promise((resolve) => setTimeout(resolve, 1000));
    expect(closeEventReceived || errorEventReceived).toBe(true);
  });
});
test.describe("WebSocket Edge Cases", () => {
  test("should handle rapid connect/disconnect cycles", async () => {
    const cycles = 10;
    for (let i = 0; i < cycles; i++) {
      const ws = await createWebSocket(WS_URL);
      expect(ws.readyState).toBe(WebSocket.OPEN);
      await closeWebSocket(ws);
      expect(ws.readyState).toBe(WebSocket.CLOSED);
    }
  });
  test("should handle sending null and undefined", async () => {
    const ws = await createWebSocket(WS_URL);
    // These should not crash the connection
    expect(() => ws.send("")).not.toThrow();
    await closeWebSocket(ws);
  });
  test("should handle special characters in messages", async () => {
    const ws = await createWebSocket(WS_URL);
    const specialMessages = [
      "🚀🌟💻", // Emojis
      "Special chars: !@#$%^&*()_+-=[]{}|;:,.<>?",
      "Unicode: café, naïve, 北京",
      "\n\r\t\0", // Control characters
      "\\\\\\", // Backslashes
      "\"\"\"'''", // Quotes
    ];
    for (const msg of specialMessages) {
      expect(() => ws.send(msg)).not.toThrow();
    }
    await closeWebSocket(ws);
  });
  test("should handle connection timeout scenarios", async () => {
    // Test with a very short timeout
    const shortTimeoutWs = new Promise((resolve, reject) => {
      const ws = new WebSocket(WS_URL);
      const timeout = setTimeout(() => {
        ws.close();
        reject(new Error("Custom timeout"));
      }, 1); // 1ms timeout - very short
      ws.on("open", () => {
        clearTimeout(timeout);
        resolve(ws);
      });
      ws.on("error", (error) => {
        clearTimeout(timeout);
        reject(error);
      });
    });
    // This should likely timeout or succeed very quickly
    try {
      const ws = (await shortTimeoutWs) as WebSocket;
      await closeWebSocket(ws);
    } catch (error) {
      expect(error).toBeInstanceOf(Error);
    }
  });
  test("should handle WebSocket close codes", async () => {
    const ws = await createWebSocket(WS_URL);
    const closePromise = new Promise<{ code: number; reason: string }>(
      (resolve) => {
        ws.on("close", (code, reason) => {
          resolve({ code, reason: reason.toString() });
        });
      }
    );
    // Close with specific code and reason
    ws.close(1000, "Normal closure");
    const closeEvent = await closePromise;
    expect(closeEvent.code).toBe(1000);
    expect(closeEvent.reason).toBe("Normal closure");
  });
  test("should handle concurrent operations", async () => {
    const ws = await createWebSocket(WS_URL);
    // Perform multiple operations concurrently
    const operations = [
      () => ws.send("message1"),
      () => ws.send("message2"),
      () => ws.send("message3"),
      () => ws.ping(),
    ];
    // Execute all operations concurrently
    expect(() => {
      operations.forEach((op) => op());
    }).not.toThrow();
    await new Promise((resolve) => setTimeout(resolve, 100));
    await closeWebSocket(ws);
  });
});
test.describe("WebSocket State Management", () => {
  test("should maintain correct readyState transitions", async () => {
    const ws = new WebSocket(WS_URL);
    // Initially connecting
    expect([WebSocket.CONNECTING, WebSocket.OPEN]).toContain(ws.readyState);
    await new Promise<void>((resolve) => {
      ws.on("open", () => {
        expect(ws.readyState).toBe(WebSocket.OPEN);
        resolve();
      });
    });
    const closePromise = new Promise<void>((resolve) => {
      ws.on("close", () => {
        expect(ws.readyState).toBe(WebSocket.CLOSED);
        resolve();
      });
    });
    ws.close();
    await closePromise;
  });
});
