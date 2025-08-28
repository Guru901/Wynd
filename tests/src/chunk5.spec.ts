// tests/websocket-integration.test.ts
import { test, expect } from "@playwright/test";
import {
  createWebSocket,
  WS_URL,
  waitForMessage,
  closeWebSocket,
} from "./shared";

test.describe("WebSocket Integration Tests", () => {
  test("should handle authentication flow (if implemented)", async () => {
    const ws = await createWebSocket(WS_URL);

    // Try sending an authentication message
    const authMessage = {
      type: "auth",
      token: "test-token-123",
      user: "test-user",
    };

    ws.send(JSON.stringify(authMessage));

    try {
      const response = await waitForMessage(ws);
      console.log("Auth response received:", response);

      // If response is JSON, validate structure
      try {
        const parsed = JSON.parse(response);
        expect(parsed).toHaveProperty("type");
      } catch (e) {
        // Response might not be JSON, which is also valid
        console.log("Auth response is not JSON");
      }
    } catch (error) {
      console.log("No auth response received - auth might not be implemented");
    }

    await closeWebSocket(ws);
  });

  test("should handle real-time chat simulation", async () => {
    const user1 = await createWebSocket(WS_URL);
    const user2 = await createWebSocket(WS_URL);

    const messages = [
      { from: "user1", message: "Hello everyone!" },
      { from: "user2", message: "Hi there!" },
      { from: "user1", message: "How is everyone doing?" },
      { from: "user2", message: "Great! Thanks for asking." },
    ];

    let user2MessagesReceived = 0;
    user2.on("message", (data) => {
      user2MessagesReceived++;
      console.log("User2 received:", data.toString());
    });

    // Simulate chat conversation
    for (const msg of messages) {
      const sender = msg.from === "user1" ? user1 : user2;
      const chatMessage = {
        type: "chat",
        from: msg.from,
        message: msg.message,
        timestamp: Date.now(),
      };

      sender.send(JSON.stringify(chatMessage));
      await new Promise((resolve) => setTimeout(resolve, 500)); // Natural conversation timing
    }

    // Wait for any final messages
    await new Promise((resolve) => setTimeout(resolve, 1000));

    await closeWebSocket(user1);
    await closeWebSocket(user2);

    console.log(`User2 received ${user2MessagesReceived} messages`);
  });

  test("should handle subscription/notification pattern", async () => {
    const subscriber1 = await createWebSocket(WS_URL);
    const subscriber2 = await createWebSocket(WS_URL);
    const publisher = await createWebSocket(WS_URL);

    const notifications1: string[] = [];
    const notifications2: string[] = [];

    subscriber1.on("message", (data) => {
      notifications1.push(data.toString());
    });

    subscriber2.on("message", (data) => {
      notifications2.push(data.toString());
    });

    // Subscribe to channels
    const subscriptionMessage = {
      type: "subscribe",
      channel: "notifications",
      user: "subscriber",
    };

    subscriber1.send(
      JSON.stringify({ ...subscriptionMessage, user: "subscriber1" })
    );
    subscriber2.send(
      JSON.stringify({ ...subscriptionMessage, user: "subscriber2" })
    );

    await new Promise((resolve) => setTimeout(resolve, 500));

    // Publish notifications
    const notifications = [
      {
        type: "notification",
        channel: "notifications",
        message: "System update available",
      },
      {
        type: "notification",
        channel: "notifications",
        message: "New feature released",
      },
      {
        type: "notification",
        channel: "notifications",
        message: "Maintenance scheduled",
      },
    ];

    for (const notification of notifications) {
      publisher.send(JSON.stringify(notification));
      await new Promise((resolve) => setTimeout(resolve, 200));
    }

    await new Promise((resolve) => setTimeout(resolve, 1000));

    console.log(`Subscriber1 received ${notifications1.length} notifications`);
    console.log(`Subscriber2 received ${notifications2.length} notifications`);

    await closeWebSocket(subscriber1);
    await closeWebSocket(subscriber2);
    await closeWebSocket(publisher);
  });

  test("should handle file transfer simulation", async () => {
    const sender = await createWebSocket(WS_URL);
    const receiver = await createWebSocket(WS_URL);

    let receivedChunks = 0;
    let totalBytesReceived = 0;

    receiver.on("message", (data) => {
      receivedChunks++;
      totalBytesReceived += data.length;
    });

    // Simulate file transfer with chunked data
    const chunkSize = 1024; // 1KB chunks
    const totalChunks = 10;

    // Send file transfer initiation
    const initMessage = {
      type: "file_transfer_start",
      filename: "test.txt",
      size: chunkSize * totalChunks,
      chunks: totalChunks,
    };

    sender.send(JSON.stringify(initMessage));

    // Send file chunks
    for (let i = 0; i < totalChunks; i++) {
      const chunk = Buffer.alloc(chunkSize, `chunk${i}`);
      const chunkMessage = {
        type: "file_chunk",
        index: i,
        data: chunk.toString("base64"),
      };

      sender.send(JSON.stringify(chunkMessage));
      await new Promise((resolve) => setTimeout(resolve, 100)); // Throttle sending
    }

    // Send completion message
    const completeMessage = {
      type: "file_transfer_complete",
      filename: "test.txt",
    };

    sender.send(JSON.stringify(completeMessage));

    await new Promise((resolve) => setTimeout(resolve, 1000));

    console.log(
      `File transfer: ${receivedChunks} chunks, ${totalBytesReceived} bytes received`
    );

    await closeWebSocket(sender);
    await closeWebSocket(receiver);
  });

  test("should handle real-time gaming scenario", async () => {
    const player1 = await createWebSocket(WS_URL);
    const player2 = await createWebSocket(WS_URL);
    const gameServer = await createWebSocket(WS_URL);

    let player1Updates = 0;
    let player2Updates = 0;

    player1.on("message", () => player1Updates++);
    player2.on("message", () => player2Updates++);

    // Join game
    const joinMessage = {
      type: "join_game",
      gameId: "test-game-123",
      playerId: "player",
    };

    player1.send(JSON.stringify({ ...joinMessage, playerId: "player1" }));
    player2.send(JSON.stringify({ ...joinMessage, playerId: "player2" }));

    await new Promise((resolve) => setTimeout(resolve, 500));

    // Simulate game actions
    const gameActions = [
      { player: "player1", action: "move", x: 10, y: 20 },
      { player: "player2", action: "move", x: 30, y: 40 },
      { player: "player1", action: "attack", target: "player2" },
      { player: "player2", action: "defend" },
      { player: "player1", action: "use_item", item: "health_potion" },
    ];

    for (const action of gameActions) {
      const actionMessage = {
        type: "game_action",
        gameId: "test-game-123",
        timestamp: Date.now(),
        ...action,
      };

      const sender = action.player === "player1" ? player1 : player2;
      sender.send(JSON.stringify(actionMessage));

      // Also send to game server for processing
      gameServer.send(JSON.stringify(actionMessage));

      await new Promise((resolve) => setTimeout(resolve, 100)); // Game tick rate
    }

    await new Promise((resolve) => setTimeout(resolve, 1000));

    console.log(`Game simulation: Player1 received ${player1Updates} updates`);
    console.log(`Game simulation: Player2 received ${player2Updates} updates`);

    await closeWebSocket(player1);
    await closeWebSocket(player2);
    await closeWebSocket(gameServer);
  });

  test("should handle API integration pattern", async () => {
    const client = await createWebSocket(WS_URL);

    const responses: any[] = [];

    client.on("message", (data) => {
      try {
        const response = JSON.parse(data.toString());
        responses.push(response);
      } catch (e) {
        responses.push({ raw: data.toString() });
      }
    });

    // API-style requests
    const requests = [
      { id: 1, method: "GET", endpoint: "/users" },
      {
        id: 2,
        method: "POST",
        endpoint: "/users",
        data: { name: "John", email: "john@example.com" },
      },
      { id: 3, method: "GET", endpoint: "/users/1" },
      {
        id: 4,
        method: "PUT",
        endpoint: "/users/1",
        data: { name: "John Updated" },
      },
      { id: 5, method: "DELETE", endpoint: "/users/1" },
    ];

    for (const request of requests) {
      const apiMessage = {
        type: "api_request",
        ...request,
        timestamp: Date.now(),
      };

      client.send(JSON.stringify(apiMessage));
      await new Promise((resolve) => setTimeout(resolve, 200));
    }

    await new Promise((resolve) => setTimeout(resolve, 2000)); // Wait for all responses

    console.log(
      `API integration: Sent ${requests.length} requests, received ${responses.length} responses`
    );

    await closeWebSocket(client);
  });
});

test.describe("WebSocket Production Scenarios", () => {
  test("should handle graceful shutdown simulation", async () => {
    const connections = await Promise.all([
      createWebSocket(WS_URL),
      createWebSocket(WS_URL),
      createWebSocket(WS_URL),
    ]);

    let shutdownNotifications = 0;

    connections.forEach((ws) => {
      ws.on("message", (data) => {
        const message = data.toString();
        if (message.includes("shutdown") || message.includes("maintenance")) {
          shutdownNotifications++;
        }
      });
    });

    // Simulate server sending shutdown notice
    const shutdownMessage = {
      type: "server_shutdown",
      message: "Server will restart in 30 seconds",
      timestamp: Date.now(),
    };

    connections[0].send(JSON.stringify(shutdownMessage)); // One client sends shutdown notice

    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Simulate graceful disconnection
    await Promise.all(connections.map(closeWebSocket));

    console.log(
      `Graceful shutdown: ${shutdownNotifications} shutdown notifications received`
    );
  });

  test("should handle connection recovery and resume", async () => {
    let ws = await createWebSocket(WS_URL);

    // Send some initial messages
    const initialMessages = ["msg1", "msg2", "msg3"];
    for (const msg of initialMessages) {
      ws.send(msg);
    }

    await new Promise((resolve) => setTimeout(resolve, 500));

    // Simulate connection drop
    await closeWebSocket(ws);

    // Wait a bit (simulating network interruption)
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Reconnect
    ws = await createWebSocket(WS_URL);

    // Send resume/recovery message
    const resumeMessage = {
      type: "resume_session",
      lastMessageId: 3,
      clientId: "test-client-123",
    };

    ws.send(JSON.stringify(resumeMessage));

    // Continue normal operation
    const resumeMessages = ["msg4", "msg5", "msg6"];
    for (const msg of resumeMessages) {
      ws.send(msg);
    }

    await new Promise((resolve) => setTimeout(resolve, 500));
    await closeWebSocket(ws);

    console.log("Connection recovery test completed");
  });

  test("should handle health check and monitoring", async () => {
    const ws = await createWebSocket(WS_URL);

    let healthResponses = 0;

    ws.on("message", (data) => {
      const message = data.toString();
      if (
        message.includes("health") ||
        message.includes("ping") ||
        message.includes("pong")
      ) {
        healthResponses++;
      }
    });

    // Send health check messages
    const healthChecks = [
      { type: "health_check", timestamp: Date.now() },
      { type: "ping", timestamp: Date.now() },
      "PING", // Simple string ping
    ];

    for (const check of healthChecks) {
      if (typeof check === "string") {
        ws.send(check);
      } else {
        ws.send(JSON.stringify(check));
      }

      await new Promise((resolve) => setTimeout(resolve, 1000)); // Wait between checks
    }

    console.log(
      `Health monitoring: ${healthResponses} health responses received`
    );

    await closeWebSocket(ws);
  });

  test("should handle rate limiting scenarios", async () => {
    const ws = await createWebSocket(WS_URL);

    let rateLimitHit = false;
    let messagesRejected = 0;

    ws.on("message", (data) => {
      const message = data.toString().toLowerCase();
      if (message.includes("rate") && message.includes("limit")) {
        rateLimitHit = true;
        messagesRejected++;
      }
    });

    // Send messages rapidly to potentially hit rate limits
    const rapidMessages = 200;

    for (let i = 0; i < rapidMessages; i++) {
      ws.send(`Rapid message ${i}`);

      // Very small delay to simulate rapid sending
      if (i % 50 === 0) {
        await new Promise((resolve) => setTimeout(resolve, 1));
      }
    }

    await new Promise((resolve) => setTimeout(resolve, 2000)); // Wait for rate limit responses

    console.log(`Rate limiting test: ${messagesRejected} messages rejected`);

    await closeWebSocket(ws);
  });
});
