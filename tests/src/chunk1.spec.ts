import { test, expect } from "@playwright/test";
import WebSocket from "ws";

const WS_URL = "ws://localhost:3000";
const CONNECTION_TIMEOUT = 5000;

function createWebSocket(url: string): Promise<WebSocket> {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    const timeout = setTimeout(() => {
      ws.close();
      reject(new Error("Connection timeout"));
    }, CONNECTION_TIMEOUT);

    ws.on("open", () => {
      clearTimeout(timeout);
      resolve(ws);
    });

    ws.on("error", (error) => {
      clearTimeout(timeout);
      reject(error);
    });
  });
}

function closeWebSocket(ws: WebSocket): Promise<void> {
  return new Promise((resolve) => {
    if (ws.readyState === WebSocket.CLOSED) {
      resolve();
      return;
    }

    ws.on("close", () => resolve());
    ws.close();
  });
}

test.describe("WebSocket Basic Connection Tests", () => {
  test("should successfully connect to WebSocket server", async () => {
    const ws = await createWebSocket(WS_URL);
    expect(ws.readyState).toBe(WebSocket.OPEN);
    await closeWebSocket(ws);
  });

  test("should handle connection rejection on invalid port", async () => {
    await expect(createWebSocket("ws://localhost:9999")).rejects.toThrow();
  });

  test("should handle connection rejection on invalid host", async () => {
    await expect(createWebSocket("ws://invalid-host:3000")).rejects.toThrow();
  });

  test("should handle graceful connection close", async () => {
    const ws = await createWebSocket(WS_URL);
    expect(ws.readyState).toBe(WebSocket.OPEN);

    const closePromise = new Promise((resolve) => {
      ws.on("close", (code, reason) => {
        resolve({ code, reason: reason.toString() });
      });
    });

    ws.close();
    const closeEvent = await closePromise;
    expect(closeEvent).toBeDefined();
    expect(ws.readyState).toBe(WebSocket.CLOSED);
  });

  test("should handle server-initiated connection close", async () => {
    const ws = await createWebSocket(WS_URL);

    const closePromise = new Promise((resolve) => {
      ws.on("close", (code, reason) => {
        resolve({ code, reason: reason.toString() });
      });
    });

    // Send a message that might trigger server close (if implemented)
    ws.send("CLOSE");

    // Wait a bit to see if server closes, otherwise close ourselves
    const closeResult = await Promise.race([
      closePromise,
      new Promise((resolve) => setTimeout(() => resolve(null), 2000)),
    ]);

    if (!closeResult) {
      ws.close();
      await closePromise;
    }

    expect(ws.readyState).toBe(WebSocket.CLOSED);
  });

  test("should handle multiple concurrent connections", async () => {
    const connectionPromises = Array.from({ length: 5 }, () =>
      createWebSocket(WS_URL)
    );
    const connections = await Promise.all(connectionPromises);

    connections.forEach((ws) => {
      expect(ws.readyState).toBe(WebSocket.OPEN);
    });

    // Close all connections
    await Promise.all(connections.map(closeWebSocket));
  });

  test("should maintain connection state correctly", async () => {
    const ws = await createWebSocket(WS_URL);
    expect(ws.readyState).toBe(WebSocket.OPEN);

    // Connection should remain open for a reasonable time
    await new Promise((resolve) => setTimeout(resolve, 1000));
    expect(ws.readyState).toBe(WebSocket.OPEN);

    await closeWebSocket(ws);
    expect(ws.readyState).toBe(WebSocket.CLOSED);
  });
});

test.describe("WebSocket Connection Events", () => {
  test("should emit open event on successful connection", async () => {
    let openEventFired = false;
    const ws = new WebSocket(WS_URL);

    const openPromise = new Promise<void>((resolve) => {
      ws.on("open", () => {
        openEventFired = true;
        resolve();
      });
    });

    await openPromise;
    expect(openEventFired).toBe(true);
    expect(ws.readyState).toBe(WebSocket.OPEN);

    await closeWebSocket(ws);
  });

  test("should emit error event on connection failure", async () => {
    let errorEventFired = false;
    const ws = new WebSocket("ws://localhost:9999");

    const errorPromise = new Promise<void>((resolve) => {
      ws.on("error", () => {
        errorEventFired = true;
        resolve();
      });
    });

    await errorPromise;
    expect(errorEventFired).toBe(true);
  });

  test("should emit close event when connection terminates", async () => {
    const ws = await createWebSocket(WS_URL);

    let closeEventFired = false;
    const closePromise = new Promise<void>((resolve) => {
      ws.on("close", () => {
        closeEventFired = true;
        resolve();
      });
    });

    ws.close();
    await closePromise;
    expect(closeEventFired).toBe(true);
  });
});
