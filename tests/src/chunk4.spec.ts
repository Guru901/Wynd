// tests/websocket-performance.test.ts
import { test, expect } from "@playwright/test";
import WebSocket from "ws";
import { closeWebSocket, createWebSocket, measureTime, WS_URL } from "./shared";

test.describe("WebSocket Performance Tests", () => {
  test("should measure connection establishment time", async () => {
    const attempts = 5;
    const durations: number[] = [];

    for (let i = 0; i < attempts; i++) {
      const { duration, result: ws } = await measureTime(() =>
        createWebSocket(WS_URL)
      );
      durations.push(duration);
      await closeWebSocket(ws);
    }

    const avgDuration = durations.reduce((a, b) => a + b, 0) / durations.length;
    console.log(`Average connection time: ${avgDuration}ms`);
    console.log(`Connection times: ${durations.join(", ")}ms`);

    // Connection should generally be fast (under 1000ms)
    expect(avgDuration).toBeLessThan(1000);
  });

  test("should handle high-frequency message sending", async () => {
    const ws = await createWebSocket(WS_URL);
    const messageCount = 1000;

    const { duration } = await measureTime(async () => {
      for (let i = 0; i < messageCount; i++) {
        ws.send(`High frequency message ${i}`);
      }

      // Wait for all messages to be queued/sent
      await new Promise((resolve) => setTimeout(resolve, 100));
    });

    console.log(`Sent ${messageCount} messages in ${duration}ms`);
    console.log(
      `Rate: ${((messageCount / duration) * 1000).toFixed(2)} messages/second`
    );

    // Should be able to send messages reasonably fast
    expect(duration).toBeLessThan(5000); // Within 5 seconds

    await closeWebSocket(ws);
  });

  test("should handle concurrent connections load", async () => {
    const connectionCount = 50;

    const { duration, result: connections } = await measureTime(async () => {
      const connectionPromises = Array.from(
        { length: connectionCount },
        (_, i) => createWebSocket(WS_URL)
      );

      return await Promise.all(connectionPromises);
    });

    console.log(`Created ${connectionCount} connections in ${duration}ms`);

    // All connections should be successful
    expect(connections).toHaveLength(connectionCount);
    connections.forEach((ws) => {
      expect(ws.readyState).toBe(WebSocket.OPEN);
    });

    // Clean up all connections
    await Promise.all(connections.map(closeWebSocket));

    // Should handle reasonable concurrent connections
    expect(duration).toBeLessThan(10000); // Within 10 seconds
  });

  test("should measure message throughput with various sizes", async () => {
    const ws = await createWebSocket(WS_URL);
    const messageSizes = [10, 100, 1000, 10000]; // bytes

    for (const size of messageSizes) {
      const message = "x".repeat(size);
      const messageCount = Math.max(100, Math.floor(10000 / size)); // Adjust count based on size

      const { duration } = await measureTime(async () => {
        for (let i = 0; i < messageCount; i++) {
          ws.send(message);
        }
        await new Promise((resolve) => setTimeout(resolve, 50));
      });

      const bytesPerSecond = ((messageCount * size) / duration) * 1000;
      console.log(
        `${size} byte messages: ${bytesPerSecond.toFixed(2)} bytes/second`
      );

      expect(duration).toBeLessThan(10000); // Should complete within reasonable time
    }

    await closeWebSocket(ws);
  });

  test("should handle memory usage efficiently with many messages", async () => {
    const ws = await createWebSocket(WS_URL);
    const messageCount = 10000;

    // Monitor if process doesn't crash with many messages
    let messagesSent = 0;

    const sendPromise = new Promise<void>((resolve) => {
      const sendBatch = () => {
        const batchSize = 100;
        for (let i = 0; i < batchSize && messagesSent < messageCount; i++) {
          ws.send(`Memory test message ${messagesSent}`);
          messagesSent++;
        }

        if (messagesSent < messageCount) {
          setImmediate(sendBatch); // Non-blocking batch sending
        } else {
          resolve();
        }
      };

      sendBatch();
    });

    await sendPromise;
    expect(messagesSent).toBe(messageCount);

    await closeWebSocket(ws);
  });

  test("should maintain performance under sustained load", async () => {
    const ws = await createWebSocket(WS_URL);
    const duration = 5000; // 5 seconds
    const startTime = Date.now();
    let messagesSent = 0;

    while (Date.now() - startTime < duration) {
      ws.send(`Sustained load message ${messagesSent}`);
      messagesSent++;

      // Small delay to prevent overwhelming
      if (messagesSent % 100 === 0) {
        await new Promise((resolve) => setTimeout(resolve, 1));
      }
    }

    const actualDuration = Date.now() - startTime;
    const messagesPerSecond = (messagesSent / actualDuration) * 1000;

    console.log(
      `Sustained load: ${messagesSent} messages in ${actualDuration}ms`
    );
    console.log(`Rate: ${messagesPerSecond.toFixed(2)} messages/second`);

    expect(messagesSent).toBeGreaterThan(0);
    await closeWebSocket(ws);
  });
});

test.describe("WebSocket Load Testing", () => {
  test("should handle burst traffic patterns", async () => {
    const ws = await createWebSocket(WS_URL);
    const burstSize = 100;
    const burstCount = 10;
    const burstDelay = 100; // ms between bursts

    for (let burst = 0; burst < burstCount; burst++) {
      // Send burst of messages
      for (let i = 0; i < burstSize; i++) {
        ws.send(`Burst ${burst} message ${i}`);
      }

      // Wait between bursts
      await new Promise((resolve) => setTimeout(resolve, burstDelay));
    }

    console.log(`Sent ${burstCount} bursts of ${burstSize} messages each`);
    await closeWebSocket(ws);
  });

  test("should handle connection churn (connect/disconnect cycles)", async () => {
    const cycles = 20;
    const connectionTimes: number[] = [];

    for (let i = 0; i < cycles; i++) {
      const { duration, result: ws } = await measureTime(() =>
        createWebSocket(WS_URL)
      );
      connectionTimes.push(duration);

      // Send a few messages
      ws.send(`Churn test message 1`);
      ws.send(`Churn test message 2`);

      await closeWebSocket(ws);
    }

    const avgConnectionTime =
      connectionTimes.reduce((a, b) => a + b, 0) / connectionTimes.length;
    console.log(`Connection churn - Average time: ${avgConnectionTime}ms`);

    expect(avgConnectionTime).toBeLessThan(1000); // Reasonable connection time
  });

  test("should handle mixed workload (multiple connection types)", async () => {
    const shortLivedConnections = 10;
    const longLivedConnections = 3;
    const messagingConnections = 5;

    // Create long-lived connections
    const longLived = await Promise.all(
      Array.from({ length: longLivedConnections }, () =>
        createWebSocket(WS_URL)
      )
    );

    // Create connections that will send many messages
    const messaging = await Promise.all(
      Array.from({ length: messagingConnections }, () =>
        createWebSocket(WS_URL)
      )
    );

    // Start messaging on dedicated connections
    const messagingPromises = messaging.map(async (ws, index) => {
      for (let i = 0; i < 100; i++) {
        ws.send(`Messaging connection ${index} message ${i}`);
        if (i % 10 === 0) {
          await new Promise((resolve) => setTimeout(resolve, 10));
        }
      }
    });

    // Create and immediately close short-lived connections
    const shortLivedPromises = Array.from(
      { length: shortLivedConnections },
      async () => {
        const ws = await createWebSocket(WS_URL);
        ws.send("Short lived message");
        await new Promise((resolve) => setTimeout(resolve, 50));
        await closeWebSocket(ws);
      }
    );

    // Wait for all workloads to complete
    await Promise.all([...messagingPromises, ...shortLivedPromises]);

    // Clean up long-lived connections
    await Promise.all(longLived.map(closeWebSocket));
    await Promise.all(messaging.map(closeWebSocket));

    console.log("Mixed workload test completed successfully");
  });

  test("should maintain stability under resource constraints", async () => {
    // Simulate resource-intensive operations
    const connections = await Promise.all(
      Array.from({ length: 20 }, () => createWebSocket(WS_URL))
    );

    // Each connection sends different patterns
    const workloads = connections.map(async (ws, index) => {
      switch (index % 4) {
        case 0: // High frequency small messages
          for (let i = 0; i < 200; i++) {
            ws.send(`HF${i}`);
          }
          break;
        case 1: // Low frequency large messages
          for (let i = 0; i < 10; i++) {
            ws.send("Large message: " + "x".repeat(5000));
            await new Promise((resolve) => setTimeout(resolve, 100));
          }
          break;
        case 2: // Binary data
          for (let i = 0; i < 50; i++) {
            ws.send(Buffer.from(`Binary data ${i}`));
          }
          break;
        case 3: // JSON messages
          for (let i = 0; i < 100; i++) {
            ws.send(
              JSON.stringify({
                type: "test",
                data: `JSON message ${i}`,
                timestamp: Date.now(),
              })
            );
          }
          break;
      }
    });

    await Promise.all(workloads);
    await Promise.all(connections.map(closeWebSocket));

    console.log("Resource constraint test completed");
  });
});

test.describe("WebSocket Stress Testing", () => {
  test("should handle maximum concurrent connections", async () => {
    const maxConnections = 100; // Adjust based on your server limits

    let successfulConnections = 0;
    let failedConnections = 0;

    const connectionPromises = Array.from(
      { length: maxConnections },
      async () => {
        try {
          const ws = await createWebSocket(WS_URL);
          successfulConnections++;
          return ws;
        } catch (error) {
          failedConnections++;
          return null;
        }
      }
    );

    const connections = await Promise.all(connectionPromises);
    const validConnections = connections.filter(
      (ws) => ws !== null
    ) as WebSocket[];

    console.log(`Successful connections: ${successfulConnections}`);
    console.log(`Failed connections: ${failedConnections}`);

    expect(successfulConnections).toBeGreaterThan(0);

    // Clean up successful connections
    await Promise.all(validConnections.map(closeWebSocket));
  });

  test("should recover from connection limit exhaustion", async () => {
    // First, try to exhaust connections
    const connections: WebSocket[] = [];

    try {
      for (let i = 0; i < 200; i++) {
        // Attempt many connections
        const ws = await createWebSocket(WS_URL);
        connections.push(ws);
      }
    } catch (error) {
      console.log("Connection limit reached as expected");
    }

    // Close half the connections
    const halfPoint = Math.floor(connections.length / 2);
    await Promise.all(connections.slice(0, halfPoint).map(closeWebSocket));

    // Should be able to create new connections now
    const newWs = await createWebSocket(WS_URL);
    expect(newWs.readyState).toBe(WebSocket.OPEN);

    // Clean up
    await closeWebSocket(newWs);
    await Promise.all(connections.slice(halfPoint).map(closeWebSocket));
  });
});
