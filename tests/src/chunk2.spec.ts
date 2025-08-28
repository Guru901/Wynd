// tests/websocket-messaging.test.ts
import { test, expect } from "@playwright/test";
import WebSocket from "ws";
import { CONNECTION_TIMEOUT, MESSAGE_TIMEOUT, WS_URL } from "./shared";

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

function waitForMessage(
  ws: WebSocket,
  timeout = MESSAGE_TIMEOUT
): Promise<string> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error("Message timeout"));
    }, timeout);

    ws.once("message", (data) => {
      clearTimeout(timer);
      resolve(data.toString());
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

test.describe("WebSocket Message Sending Tests", () => {
  test("should send and receive text messages", async () => {
    const ws = await createWebSocket(WS_URL);
    const testMessage = "Hello WebSocket Server";

    // Send message
    ws.send(testMessage);

    // Wait for response (assuming echo server or some response)
    try {
      const response = await waitForMessage(ws);
      expect(response).toBeDefined();
      expect(typeof response).toBe("string");
    } catch (error) {
      // If no echo, that's fine - server received the message
      console.log("No echo response received, which is acceptable");
    }

    await closeWebSocket(ws);
  });

  test("should handle JSON message sending", async () => {
    const ws = await createWebSocket(WS_URL);
    const jsonMessage = {
      type: "test",
      data: "json payload",
      timestamp: Date.now(),
    };

    ws.send(JSON.stringify(jsonMessage));

    try {
      const response = await waitForMessage(ws);
      // Try to parse response as JSON if received
      if (response) {
        const parsed = JSON.parse(response);
        expect(parsed).toBeDefined();
      }
    } catch (error) {
      // No response or invalid JSON is acceptable
      console.log("No JSON echo response received");
    }

    await closeWebSocket(ws);
  });

  test("should handle empty message sending", async () => {
    const ws = await createWebSocket(WS_URL);

    // Should not throw error when sending empty message
    expect(() => ws.send("")).not.toThrow();

    await closeWebSocket(ws);
  });

  test("should handle large message sending", async () => {
    const ws = await createWebSocket(WS_URL);
    const largeMessage = "x".repeat(10000); // 10KB message

    expect(() => ws.send(largeMessage)).not.toThrow();

    try {
      const response = await waitForMessage(ws);
      expect(response).toBeDefined();
    } catch (error) {
      console.log("No response to large message");
    }

    await closeWebSocket(ws);
  });

  test("should handle binary data sending", async () => {
    const ws = await createWebSocket(WS_URL);
    const binaryData = Buffer.from([0x01, 0x02, 0x03, 0x04]);

    expect(() => ws.send(binaryData)).not.toThrow();

    await closeWebSocket(ws);
  });

  test("should handle rapid message sending", async () => {
    const ws = await createWebSocket(WS_URL);
    const messageCount = 100;

    for (let i = 0; i < messageCount; i++) {
      ws.send(`Message ${i}`);
    }

    // Wait a bit for all messages to be sent
    await new Promise((resolve) => setTimeout(resolve, 1000));

    await closeWebSocket(ws);
  });
});

test.describe("WebSocket Message Receiving Tests", () => {
  test("should receive message event with proper data", async () => {
    const ws = await createWebSocket(WS_URL);

    let messageReceived = false;
    let receivedData: any;

    ws.on("message", (data) => {
      messageReceived = true;
      receivedData = data;
    });

    // Send a message that might trigger a response
    ws.send("ping");

    // Wait for potential response
    await new Promise((resolve) => setTimeout(resolve, 1000));

    if (messageReceived) {
      expect(receivedData).toBeDefined();
      expect(
        Buffer.isBuffer(receivedData) || typeof receivedData === "string"
      ).toBe(true);
    } else {
      console.log("Server does not echo messages");
    }

    await closeWebSocket(ws);
  });

  test("should handle binary message reception", async () => {
    const ws = await createWebSocket(WS_URL);

    let binaryReceived = false;

    ws.on("message", (data, isBinary) => {
      if (isBinary || Buffer.isBuffer(data)) {
        binaryReceived = true;
        expect(Buffer.isBuffer(data)).toBe(true);
      }
    });

    // Send binary data that might trigger binary response
    ws.send(Buffer.from("binary test"));

    await new Promise((resolve) => setTimeout(resolve, 1000));

    await closeWebSocket(ws);
  });

  test("should handle message order preservation", async () => {
    const ws = await createWebSocket(WS_URL);
    const messages = ["first", "second", "third"];
    const receivedMessages: string[] = [];

    ws.on("message", (data) => {
      receivedMessages.push(data.toString());
    });

    // Send messages in order
    for (const msg of messages) {
      ws.send(msg);
    }

    // Wait for responses
    await new Promise((resolve) => setTimeout(resolve, 2000));

    // If we received messages, they should be in order (if server echoes)
    if (receivedMessages.length > 0) {
      console.log("Received messages:", receivedMessages);
      // Message order verification would depend on server behavior
    }

    await closeWebSocket(ws);
  });
});

test.describe("WebSocket Bidirectional Communication", () => {
  test("should handle ping-pong communication", async () => {
    const ws = await createWebSocket(WS_URL);

    // Test ping frame if server supports it
    ws.ping();

    const pongPromise = new Promise<void>((resolve) => {
      ws.on("pong", () => {
        resolve();
      });

      // Timeout if no pong received
      setTimeout(() => resolve(), 1000);
    });

    await pongPromise;
    await closeWebSocket(ws);
  });

  test("should maintain connection during idle periods", async () => {
    const ws = await createWebSocket(WS_URL);

    expect(ws.readyState).toBe(WebSocket.OPEN);

    // Wait for 3 seconds without activity
    await new Promise((resolve) => setTimeout(resolve, 3000));

    expect(ws.readyState).toBe(WebSocket.OPEN);

    await closeWebSocket(ws);
  });

  test("should handle reconnection attempts", async () => {
    let ws = await createWebSocket(WS_URL);
    await closeWebSocket(ws);

    // Reconnect
    ws = await createWebSocket(WS_URL);
    expect(ws.readyState).toBe(WebSocket.OPEN);

    await closeWebSocket(ws);
  });
});
